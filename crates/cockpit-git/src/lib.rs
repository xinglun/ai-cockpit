use serde::{Deserialize, Serialize};
use sha2::{Digest as ShaDigest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::PathBuf;
use std::path::{Component, Path};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, SyncSender};
use std::thread;
use std::time::UNIX_EPOCH;
use thiserror::Error;

#[cfg(unix)]
unsafe extern "C" {
    fn kill(pid: i32, signal: i32) -> i32;
}

pub const MAX_CHANGE_TEXT_BYTES: usize = 262_144;
pub const MAX_BOUNDED_GIT_OUTPUT_BYTES: usize = 64 * 1024 * 1024;
const GIT_OUTPUT_CHUNK_BYTES: usize = 8 * 1024;

/// A bounded, repository-local content identity cache. It hashes only declared
/// relative files and derives a deterministic Merkle root from their
/// path/digest pairs. Metadata is checked around every read, but never proves
/// that a prior digest is still current; an unreadable, ambiguous, or changing
/// path is an error rather than an authorization to reuse a stale digest.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IncrementalMerkle {
    entries: BTreeMap<String, ContentIdentityEntry>,
    root_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ContentIdentityEntry {
    size: u64,
    modified_ns: u128,
    digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MerkleRefresh {
    pub root_digest: String,
    pub files_read: usize,
    pub files_hashed: usize,
    pub files_reused: usize,
}

#[derive(Debug, Error)]
pub enum ContentIdentityError {
    #[error("content identity path must be relative: {0}")]
    AbsolutePath(PathBuf),
    #[error("content identity path escapes repository: {0}")]
    PathEscape(PathBuf),
    #[error("content identity path is not a regular file: {0}")]
    NotAFile(PathBuf),
    #[error("failed to inspect content identity path {path}: {source}")]
    Metadata {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to read content identity path {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("content identity path changed while being read: {0}")]
    ChangedDuringRead(PathBuf),
    #[error("content identity timestamp is before Unix epoch: {0}")]
    InvalidTimestamp(PathBuf),
}

impl IncrementalMerkle {
    /// Refresh the identity for exactly `paths`. A removed path is deleted
    /// from the Merkle set. The caller must provide the same repository root
    /// used for all refreshes; no process-global cache is involved.
    pub fn refresh<I, P>(
        &mut self,
        root: &Path,
        paths: I,
    ) -> Result<MerkleRefresh, ContentIdentityError>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let mut normalized = BTreeSet::new();
        for path in paths {
            normalized.insert(normalize_identity_path(path.as_ref())?);
        }
        let old_paths = self.entries.keys().cloned().collect::<BTreeSet<_>>();
        for removed in old_paths.difference(&normalized) {
            self.entries.remove(removed);
        }
        let mut files_read = 0;
        let mut files_hashed = 0;
        let files_reused = 0;
        for relative in normalized {
            let path = root.join(&relative);
            let metadata = match std::fs::symlink_metadata(&path) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    self.entries.remove(&relative);
                    continue;
                }
                Err(source) => {
                    return Err(ContentIdentityError::Metadata { path, source });
                }
            };
            if !metadata.is_file() {
                return Err(ContentIdentityError::NotAFile(path));
            }
            let before = file_observation(&path, metadata)?;
            let (bytes, after) = read_stable_file(&path, before)?;
            files_read += 1;
            files_hashed += 1;
            self.entries.insert(
                relative,
                ContentIdentityEntry {
                    size: after.size,
                    modified_ns: after.modified_ns,
                    digest: digest(&bytes),
                },
            );
        }
        self.root_digest = merkle_root(&self.entries);
        Ok(MerkleRefresh {
            root_digest: self.root_digest.clone(),
            files_read,
            files_hashed,
            files_reused,
        })
    }

    pub fn root_digest(&self) -> Option<&str> {
        (!self.root_digest.is_empty()).then_some(self.root_digest.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FileObservation {
    size: u64,
    modified_ns: u128,
}

fn file_observation(
    path: &Path,
    metadata: std::fs::Metadata,
) -> Result<FileObservation, ContentIdentityError> {
    let modified_ns = metadata
        .modified()
        .map_err(|source| ContentIdentityError::Metadata {
            path: path.to_path_buf(),
            source,
        })?
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ContentIdentityError::InvalidTimestamp(path.to_path_buf()))?
        .as_nanos();
    Ok(FileObservation {
        size: metadata.len(),
        modified_ns,
    })
}

fn read_stable_file(
    path: &Path,
    before: FileObservation,
) -> Result<(Vec<u8>, FileObservation), ContentIdentityError> {
    read_stable_file_with(path, before, |path| std::fs::read(path))
}

fn read_stable_file_with<F>(
    path: &Path,
    before: FileObservation,
    read: F,
) -> Result<(Vec<u8>, FileObservation), ContentIdentityError>
where
    F: FnOnce(&Path) -> std::io::Result<Vec<u8>>,
{
    let bytes = read(path).map_err(|source| ContentIdentityError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let after_metadata = std::fs::symlink_metadata(path)
        .map_err(|_| ContentIdentityError::ChangedDuringRead(path.to_path_buf()))?;
    if !after_metadata.is_file() {
        return Err(ContentIdentityError::ChangedDuringRead(path.to_path_buf()));
    }
    let after = file_observation(path, after_metadata)
        .map_err(|_| ContentIdentityError::ChangedDuringRead(path.to_path_buf()))?;
    if before != after {
        return Err(ContentIdentityError::ChangedDuringRead(path.to_path_buf()));
    }
    Ok((bytes, after))
}

#[cfg(test)]
mod incremental_merkle_tests {
    use super::{ContentIdentityError, file_observation, read_stable_file_with};
    use std::fs;

    #[test]
    fn stable_read_rejects_detectable_change_during_read() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("changing.txt");
        fs::write(&path, "before\n").expect("initial content");
        let before = file_observation(
            &path,
            fs::symlink_metadata(&path).expect("initial metadata"),
        )
        .expect("initial observation");

        let result = read_stable_file_with(&path, before, |path| {
            fs::write(path, "after\n").expect("change during read");
            Ok(b"before\n".to_vec())
        });
        assert!(matches!(
            result,
            Err(ContentIdentityError::ChangedDuringRead(changed)) if changed == path
        ));
    }
}

fn normalize_identity_path(path: &Path) -> Result<String, ContentIdentityError> {
    if path.is_absolute() {
        return Err(ContentIdentityError::AbsolutePath(path.to_path_buf()));
    }
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(value) => components.push(value.to_string_lossy().into_owned()),
            Component::ParentDir => {
                if components.pop().is_none() {
                    return Err(ContentIdentityError::PathEscape(path.to_path_buf()));
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(ContentIdentityError::AbsolutePath(path.to_path_buf()));
            }
        }
    }
    let normalized = components.join("/");
    if normalized.is_empty() {
        return Err(ContentIdentityError::PathEscape(path.to_path_buf()));
    }
    Ok(normalized)
}

fn merkle_root(entries: &BTreeMap<String, ContentIdentityEntry>) -> String {
    let mut hasher = Sha256::new();
    for (path, entry) in entries {
        hasher.update(path.as_bytes());
        hasher.update([0]);
        hasher.update(entry.digest.as_bytes());
        hasher.update([0]);
    }
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChangeContentState {
    Text,
    Binary,
    TooLarge,
    Deleted,
    Unavailable,
}

/// The line in the post-change file and the zero-based Git hunk that supplied
/// an added line. An untracked file has no patch origin and is inspected from
/// its bounded full text instead.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddedLineOrigin {
    pub after_line: usize,
    pub hunk_index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangeEvidence {
    pub path: String,
    pub kind: ChangeKind,
    pub added_lines: Vec<String>,
    pub added_line_origins: Vec<AddedLineOrigin>,
    pub removed_lines: Vec<String>,
    pub after_text: Option<String>,
    pub content_state: ChangeContentState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySnapshot {
    pub root: PathBuf,
    pub git_root: PathBuf,
    pub head: Option<String>,
    pub changed_paths: Vec<String>,
    #[serde(skip, default)]
    pub change_evidence: Vec<ChangeEvidence>,
    pub git_calls: usize,
    pub tree_digest: String,
    pub diff_digest: String,
    pub dependency_fingerprint: String,
    pub files_read: usize,
    pub files_hashed: usize,
    /// Request-scoped byte counters for performance diagnostics. They are
    /// omitted from serialized snapshots so existing repository identities
    /// and historical evidence remain unchanged.
    #[serde(skip, default)]
    pub bytes_read: u64,
    #[serde(skip, default)]
    pub bytes_hashed: u64,
    /// Source-only tree identity captured while reading the Git index. This
    /// is an internal request-scoped optimization hint; it is intentionally
    /// omitted from serialized snapshots so existing wire/digest semantics
    /// remain unchanged.
    #[serde(skip, default)]
    pub source_tree_digest: Option<String>,
}

/// Minimal current-source observation for bounded request construction. The
/// status output contains working-tree changes while the tree digests are
/// derived from the index; `.ai` paths are excluded from source_tree_digest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundedSourceSnapshot {
    pub head: Option<String>,
    pub changed_paths: Vec<String>,
    pub tree_digest: String,
    pub source_tree_digest: String,
}

/// Captured stdout and stderr from one bounded Git subprocess. Each stream is
/// capped independently at the requested byte limit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundedGitOutput {
    pub success: bool,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    exit_code: Option<i32>,
}

pub struct GitRepository {
    root: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GitTopologyKind {
    PrimaryWorktree,
    LinkedWorktree,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GitTopologyCompatibility {
    SharedCommonDirectory,
    UnsupportedIndependentClone,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitTopology {
    pub repository_root: PathBuf,
    pub git_dir: PathBuf,
    pub common_dir: PathBuf,
    pub worktree_path: PathBuf,
    pub branch: Option<String>,
    pub head: Option<String>,
    pub kind: GitTopologyKind,
}

impl GitTopology {
    pub fn compatibility_with(&self, other: &Self) -> GitTopologyCompatibility {
        if self.common_dir == other.common_dir {
            GitTopologyCompatibility::SharedCommonDirectory
        } else {
            GitTopologyCompatibility::UnsupportedIndependentClone
        }
    }
}

#[derive(Debug, Error)]
pub enum GitError {
    #[error("path is not a git repository: {0}")]
    NotRepository(PathBuf),
    #[error("git command failed: {0}")]
    Command(String),
    #[error("git output was not valid UTF-8")]
    InvalidUtf8,
    #[error("git output exceeded the bounded limit of {limit} bytes")]
    OutputLimitExceeded { limit: usize },
    #[error("Git revision must be a full 40- or 64-digit object ID: {0}")]
    InvalidRevision(String),
    #[error("git topology path could not be resolved: {0}")]
    InvalidTopology(PathBuf),
}

impl GitRepository {
    pub fn discover(path: impl Into<PathBuf>) -> Result<Self, GitError> {
        let requested = path.into();
        let output = Command::new("git")
            .args(["-C"])
            .arg(&requested)
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .map_err(|error| GitError::Command(error.to_string()))?;
        if !output.status.success() {
            return Err(GitError::NotRepository(requested));
        }
        Ok(Self {
            root: PathBuf::from(String::from_utf8_lossy(&output.stdout).trim()),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Run a Git subprocess while bounding stdout and stderr independently.
    /// If either stream exceeds `max_output_bytes`, the child is killed and
    /// waited for before this method returns an error.
    fn output_bounded<const N: usize>(
        &self,
        args: [&str; N],
        max_output_bytes: usize,
    ) -> Result<BoundedGitOutput, GitError> {
        let mut command = Command::new("git");
        command
            .args(["-C"])
            .arg(&self.root)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        bounded_process_output(command, max_output_bytes)
    }

    /// Read the tracked index flags through a fixed, read-only Git command.
    pub fn index_flags_bounded(
        &self,
        max_output_bytes: usize,
    ) -> Result<BoundedGitOutput, GitError> {
        self.output_bounded(["ls-files", "-v", "-z"], max_output_bytes)
    }

    /// Read a committed tree through Git's NUL-delimited tree format.
    pub fn committed_tree_bounded(
        &self,
        revision: &str,
        max_output_bytes: usize,
    ) -> Result<BoundedGitOutput, GitError> {
        if revision != "HEAD" && !valid_commit_oid(revision) {
            return Err(GitError::InvalidRevision(revision.to_owned()));
        }
        let mut command = Command::new("git");
        command
            .args(["-C"])
            .arg(&self.root)
            .args(["ls-tree", "-r", "-z"])
            .arg(revision)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        bounded_process_output(command, max_output_bytes)
    }

    /// Read one committed blob by object ID through a fixed Git command.
    pub fn blob_bounded(
        &self,
        object_id: &str,
        max_output_bytes: usize,
    ) -> Result<BoundedGitOutput, GitError> {
        if !valid_commit_oid(object_id) {
            return Err(GitError::InvalidRevision(object_id.to_owned()));
        }
        let mut command = Command::new("git");
        command
            .args(["-C"])
            .arg(&self.root)
            .args(["cat-file", "blob"])
            .arg(object_id)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        bounded_process_output(command, max_output_bytes)
    }

    /// Test ancestry with `merge-base --is-ancestor`, distinguishing its
    /// normal false result from a Git command failure.
    pub fn is_ancestor_bounded(
        &self,
        base: &str,
        head: &str,
        max_output_bytes: usize,
    ) -> Result<bool, GitError> {
        if !valid_commit_oid(base) {
            return Err(GitError::InvalidRevision(base.to_owned()));
        }
        if !valid_commit_oid(head) {
            return Err(GitError::InvalidRevision(head.to_owned()));
        }
        let output = self.output_bounded(
            ["merge-base", "--is-ancestor", base, head],
            max_output_bytes,
        )?;
        match output.exit_code {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            _ => Err(command_error(&output.stderr)),
        }
    }

    /// Capture only current Git status and index-tree identities. This avoids
    /// reading changed checkout files while a material request checks that
    /// source is clean.
    pub fn source_snapshot_bounded(
        &self,
        max_output_bytes: usize,
    ) -> Result<BoundedSourceSnapshot, GitError> {
        let status = self.output_bounded(
            [
                "-c",
                "core.fsmonitor=false",
                "status",
                "--porcelain=v2",
                "--branch",
                "--untracked-files=all",
                "-z",
            ],
            max_output_bytes,
        )?;
        if !status.success {
            return Err(command_error(&status.stderr));
        }
        let tree = self.output_bounded(["ls-files", "-s", "-z"], max_output_bytes)?;
        if !tree.success {
            return Err(command_error(&tree.stderr));
        }
        let mut source_tree_hasher = Sha256::new();
        for record in tree
            .stdout
            .split(|byte| *byte == 0)
            .filter(|record| !record.is_empty())
        {
            let Some(tab) = record.iter().position(|byte| *byte == b'\t') else {
                continue;
            };
            let path = &record[tab + 1..];
            if is_ai_path(path) {
                continue;
            }
            source_tree_hasher.update(record);
            source_tree_hasher.update([0]);
        }
        let (changed_paths, _) = status_change_facts_nul(&status.stdout)?;
        Ok(BoundedSourceSnapshot {
            head: status_v2_head_nul(&status.stdout)?,
            changed_paths,
            tree_digest: digest(&tree.stdout),
            source_tree_digest: format!("sha256:{}", hex::encode(source_tree_hasher.finalize())),
        })
    }

    /// Resolve the actual Git common directory and worktree identity.  This
    /// intentionally asks Git instead of inferring `.git` from the filesystem
    /// because linked worktrees expose a `.git` file and keep their common
    /// metadata elsewhere.
    pub fn topology(&self) -> Result<GitTopology, GitError> {
        let repository_root = std::fs::canonicalize(&self.root)
            .map_err(|_| GitError::InvalidTopology(self.root.clone()))?;
        let git_dir = resolve_git_path(
            &repository_root,
            self.run(["rev-parse", "--git-dir"])?.trim(),
        )?;
        let common_dir = resolve_git_path(
            &repository_root,
            self.run(["rev-parse", "--git-common-dir"])?.trim(),
        )?;
        let branch = self.run(["branch", "--show-current"])?.trim().to_owned();
        let head_output = self.run(["rev-parse", "--verify", "HEAD"]);
        let head = head_output.ok().map(|value| value.trim().to_owned());
        let kind = if git_dir == common_dir {
            GitTopologyKind::PrimaryWorktree
        } else {
            GitTopologyKind::LinkedWorktree
        };
        Ok(GitTopology {
            repository_root: repository_root.clone(),
            git_dir,
            common_dir,
            worktree_path: repository_root,
            branch: (!branch.is_empty()).then_some(branch),
            head,
            kind,
        })
    }

    pub fn snapshot(&self) -> Result<RepositorySnapshot, GitError> {
        // Porcelain v2 exposes the current HEAD alongside working-tree facts.
        // Reading both from one Git invocation avoids a redundant `rev-parse`
        // process without weakening the snapshot boundary: the branch OID and
        // status records are produced by the same Git observation.
        let status = self.run([
            "status",
            "--porcelain=v2",
            "--branch",
            "--untracked-files=all",
        ])?;
        let head = status_v2_head(&status);
        // A clean status proves that the equivalent diff is empty, so avoid
        // spawning a fourth Git process on the hot status path. Dirty or
        // otherwise uncertain input retains the full patch inspection path.
        let mut git_calls = 2;
        let diff = if !status_v2_has_changes(&status) {
            String::new()
        } else if head.is_some() {
            git_calls += 1;
            self.run([
                "-c",
                "core.quotePath=false",
                "diff",
                "HEAD",
                "--no-ext-diff",
                "--no-color",
                "--unified=0",
            ])?
        } else {
            git_calls += 1;
            self.run([
                "-c",
                "core.quotePath=false",
                "diff",
                "--cached",
                "--no-ext-diff",
                "--no-color",
                "--unified=0",
            ])?
        };
        let tree = self.run(["ls-files", "-s"])?;
        let (changed_paths, change_kinds) = status_change_facts(&status);
        let mut change_evidence = changed_paths
            .iter()
            .map(|path| {
                (
                    path.clone(),
                    ChangeEvidence {
                        path: path.clone(),
                        kind: change_kinds
                            .get(path)
                            .cloned()
                            .unwrap_or(ChangeKind::Unknown),
                        added_lines: Vec::new(),
                        added_line_origins: Vec::new(),
                        removed_lines: Vec::new(),
                        after_text: None,
                        content_state: ChangeContentState::Unavailable,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        apply_patch_facts(&diff, &mut change_evidence, None);
        let mut changed_hasher = Sha256::new();
        let mut changed_files_read = 0;
        let mut changed_files_hashed = 0;
        let mut changed_bytes_read = 0_u64;
        let mut changed_bytes_hashed = 0_u64;
        let mut hashed_paths = BTreeSet::new();
        for change in change_evidence
            .values()
            .filter(|change| !change.path.starts_with(".ai/"))
        {
            changed_hasher.update(change.path.as_bytes());
            changed_hasher.update([0]);
            changed_hasher.update(change_kind_name(&change.kind).as_bytes());
            changed_hasher.update([0]);
            for line in &change.removed_lines {
                changed_hasher.update(b"-");
                changed_hasher.update(line.as_bytes());
                changed_hasher.update([0]);
            }
            for line in &change.added_lines {
                changed_hasher.update(b"+");
                changed_hasher.update(line.as_bytes());
                changed_hasher.update([0]);
            }
        }
        for relative in &changed_paths {
            if relative.starts_with(".ai/") {
                continue;
            }
            if !hashed_paths.insert(relative.clone()) {
                continue;
            }
            let path = self.root.join(relative);
            if let Ok(bytes) = std::fs::read(path) {
                changed_hasher.update(relative.as_bytes());
                changed_hasher.update([0]);
                changed_hasher.update(&bytes);
                changed_files_read += 1;
                changed_files_hashed += 1;
                changed_bytes_read = changed_bytes_read.saturating_add(bytes.len() as u64);
                changed_bytes_hashed = changed_bytes_hashed.saturating_add(bytes.len() as u64);
                if let Some(change) = change_evidence.get_mut(relative) {
                    if change.content_state == ChangeContentState::TooLarge {
                        // Patch overflow is irreversible evidence loss. A
                        // retained short deletion must not restore Text.
                        change.after_text = None;
                    } else if bytes.len() > MAX_CHANGE_TEXT_BYTES {
                        change.after_text = None;
                        if change.added_lines.is_empty() && change.removed_lines.is_empty() {
                            change.content_state = ChangeContentState::TooLarge;
                        }
                    } else if bytes.contains(&0) {
                        change.after_text = None;
                        change.content_state = ChangeContentState::Binary;
                    } else if let Ok(text) = String::from_utf8(bytes) {
                        change.after_text = Some(text);
                        change.content_state = ChangeContentState::Text;
                    } else {
                        change.after_text = None;
                        change.content_state = ChangeContentState::Binary;
                    }
                }
            } else if let Some(change) = change_evidence.get_mut(relative)
                && change.kind == ChangeKind::Deleted
            {
                change.content_state = ChangeContentState::Deleted;
            }
        }
        let dependency_paths = [
            "Cargo.toml",
            "Cargo.lock",
            "package.json",
            "package-lock.json",
            "pnpm-lock.yaml",
            "yarn.lock",
            "pyproject.toml",
            "poetry.lock",
            "go.mod",
            "go.sum",
        ];
        let mut dependency_hasher = Sha256::new();
        let mut files_read = 0;
        let mut files_hashed = 0;
        let mut bytes_read = 0_u64;
        let mut bytes_hashed = 0_u64;
        for relative in dependency_paths {
            if !hashed_paths.insert(relative.into()) {
                continue;
            }
            let path = self.root.join(relative);
            if let Ok(bytes) = std::fs::read(&path) {
                dependency_hasher.update(relative.as_bytes());
                dependency_hasher.update([0]);
                dependency_hasher.update(&bytes);
                files_read += 1;
                files_hashed += 1;
                bytes_read = bytes_read.saturating_add(bytes.len() as u64);
                bytes_hashed = bytes_hashed.saturating_add(bytes.len() as u64);
            }
        }
        let mut source_tree_hasher = Sha256::new();
        for line in tree.lines() {
            let Some((_, path)) = line.split_once('\t') else {
                continue;
            };
            if path == ".ai" || path.starts_with(".ai/") {
                continue;
            }
            source_tree_hasher.update(path.as_bytes());
            source_tree_hasher.update([0]);
            source_tree_hasher.update(line.as_bytes());
            source_tree_hasher.update([0]);
        }
        Ok(RepositorySnapshot {
            root: self.root.clone(),
            git_root: self.root.clone(),
            head: head.filter(|value| !value.is_empty()),
            changed_paths,
            change_evidence: change_evidence.into_values().collect(),
            git_calls,
            tree_digest: digest(tree.as_bytes()),
            diff_digest: format!("sha256:{}", hex::encode(changed_hasher.finalize())),
            dependency_fingerprint: format!("sha256:{}", hex::encode(dependency_hasher.finalize())),
            files_read: files_read + changed_files_read,
            files_hashed: files_hashed + changed_files_hashed,
            bytes_read: bytes_read.saturating_add(changed_bytes_read),
            bytes_hashed: bytes_hashed.saturating_add(changed_bytes_hashed),
            source_tree_digest: Some(digest(&source_tree_hasher.finalize())),
        })
    }

    /// Capture the current repository facts while binding change evidence to
    /// the committed comparison between `base` and HEAD.  `snapshot()` is
    /// intentionally about the working tree; hosted quality gates run on a
    /// clean checkout, so using it alone would hide every change already
    /// committed to a pull-request branch.
    pub fn snapshot_against(&self, base: &str) -> Result<RepositorySnapshot, GitError> {
        let mut snapshot = self.snapshot()?;
        // A base..HEAD comparison intentionally omits uncommitted paths, but
        // callers also use this snapshot to bind current source verification.
        // Retain the working-tree evidence collected above so that binding
        // cannot regress to the index when source is still dirty.
        let working_change_evidence = std::mem::take(&mut snapshot.change_evidence);
        let Some(head) = snapshot.head.clone() else {
            return Err(GitError::Command(
                "comparison snapshot requires a committed HEAD".into(),
            ));
        };
        let name_status = self.run([
            "-c",
            "core.quotePath=false",
            "diff",
            "--name-status",
            "--no-renames",
            "-z",
            "--no-ext-diff",
            "--no-color",
            base,
            head.as_str(),
        ])?;
        let patch = self.run([
            "-c",
            "core.quotePath=false",
            "diff",
            "--no-ext-diff",
            "--no-color",
            "--unified=0",
            base,
            head.as_str(),
        ])?;
        let (mut changed_paths, change_kinds) = comparison_change_facts(&name_status);
        let mut change_evidence = changed_paths
            .iter()
            .map(|path| {
                (
                    path.clone(),
                    ChangeEvidence {
                        path: path.clone(),
                        kind: change_kinds
                            .get(path)
                            .cloned()
                            .unwrap_or(ChangeKind::Unknown),
                        added_lines: Vec::new(),
                        added_line_origins: Vec::new(),
                        removed_lines: Vec::new(),
                        after_text: None,
                        content_state: ChangeContentState::Unavailable,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        apply_patch_facts(&patch, &mut change_evidence, None);
        for change in working_change_evidence {
            if !changed_paths.iter().any(|path| path == &change.path) {
                changed_paths.push(change.path.clone());
                change_evidence.insert(change.path.clone(), change);
            }
        }
        changed_paths.sort();
        changed_paths.dedup();
        for change in change_evidence.values_mut() {
            let path = self.root.join(&change.path);
            match std::fs::read(&path) {
                Ok(bytes) => {
                    if change.content_state == ChangeContentState::TooLarge {
                        change.after_text = None;
                    } else if bytes.len() > MAX_CHANGE_TEXT_BYTES {
                        change.after_text = None;
                        if change.added_lines.is_empty() && change.removed_lines.is_empty() {
                            change.content_state = ChangeContentState::TooLarge;
                        }
                    } else if bytes.contains(&0) {
                        change.content_state = ChangeContentState::Binary;
                        change.after_text = None;
                    } else if let Ok(text) = String::from_utf8(bytes) {
                        change.content_state = ChangeContentState::Text;
                        change.after_text = Some(text);
                    } else {
                        change.content_state = ChangeContentState::Binary;
                        change.after_text = None;
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    change.content_state = ChangeContentState::Deleted;
                }
                Err(_) => {
                    change.content_state = ChangeContentState::Unavailable;
                }
            }
        }
        snapshot.changed_paths = changed_paths;
        snapshot.change_evidence = change_evidence.into_values().collect();
        snapshot.git_calls = snapshot.git_calls.saturating_add(2);
        snapshot.diff_digest = digest(patch.as_bytes());
        Ok(snapshot)
    }

    /// Capture committed source changes against `base` with bounded Git
    /// output. The resulting change and patch identities exclude `.ai` paths;
    /// unlike `snapshot_against`, this request-oriented API does not read
    /// working-tree file contents or merge uncommitted evidence.
    pub fn source_snapshot_against_bounded(
        &self,
        base: &str,
        max_output_bytes: usize,
    ) -> Result<RepositorySnapshot, GitError> {
        if !valid_commit_oid(base) {
            return Err(GitError::InvalidRevision(base.to_owned()));
        }
        let working = self.source_snapshot_bounded(max_output_bytes)?;
        let head = working.head.clone().ok_or_else(|| {
            GitError::Command("comparison snapshot requires a committed HEAD".into())
        })?;
        let name_status = self.output_bounded(
            [
                "-c",
                "core.quotePath=false",
                "diff",
                "--name-status",
                "--no-renames",
                "-z",
                "-O/dev/null",
                "--no-ext-diff",
                "--no-color",
                base,
                head.as_str(),
                "--",
                ".",
                ":(exclude).ai",
                ":(exclude).ai/**",
            ],
            max_output_bytes,
        )?;
        if !name_status.success {
            return Err(command_error(&name_status.stderr));
        }
        let name_status_text =
            String::from_utf8(name_status.stdout).map_err(|_| GitError::InvalidUtf8)?;
        let (mut changed_paths, change_kinds) = comparison_change_facts(&name_status_text);
        changed_paths.retain(|path| path != ".ai" && !path.starts_with(".ai/"));
        let mut change_evidence = changed_paths
            .iter()
            .map(|path| {
                (
                    path.clone(),
                    ChangeEvidence {
                        path: path.clone(),
                        kind: change_kinds
                            .get(path)
                            .cloned()
                            .unwrap_or(ChangeKind::Unknown),
                        added_lines: Vec::new(),
                        added_line_origins: Vec::new(),
                        removed_lines: Vec::new(),
                        after_text: None,
                        content_state: ChangeContentState::Unavailable,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        let patch = self.output_bounded(
            [
                "-c",
                "core.quotePath=false",
                "diff",
                "--no-ext-diff",
                "--no-textconv",
                "--no-color",
                "--no-renames",
                "--diff-algorithm=myers",
                "--binary",
                "--unified=0",
                "-O/dev/null",
                base,
                head.as_str(),
                "--",
                ".",
                ":(exclude).ai",
                ":(exclude).ai/**",
            ],
            max_output_bytes,
        )?;
        if !patch.success {
            return Err(command_error(&patch.stderr));
        }
        let patch_text = String::from_utf8_lossy(&patch.stdout);
        apply_patch_facts(&patch_text, &mut change_evidence, Some(&changed_paths));
        for change in change_evidence.values_mut() {
            if change.kind == ChangeKind::Deleted {
                change.content_state = ChangeContentState::Deleted;
            }
        }
        let empty_digest = digest(b"");
        Ok(RepositorySnapshot {
            root: self.root.clone(),
            git_root: self.root.clone(),
            head: Some(head),
            changed_paths,
            change_evidence: change_evidence.into_values().collect(),
            git_calls: 4,
            tree_digest: working.tree_digest,
            diff_digest: digest(&patch.stdout),
            dependency_fingerprint: empty_digest,
            files_read: 0,
            files_hashed: 0,
            bytes_read: 0,
            bytes_hashed: 0,
            source_tree_digest: Some(working.source_tree_digest),
        })
    }

    fn run<const N: usize>(&self, args: [&str; N]) -> Result<String, GitError> {
        let output = Command::new("git")
            .args(["-C"])
            .arg(&self.root)
            .args(args)
            .output()
            .map_err(|error| GitError::Command(error.to_string()))?;
        if !output.status.success() {
            return Err(GitError::Command(
                String::from_utf8_lossy(&output.stderr).trim().into(),
            ));
        }
        String::from_utf8(output.stdout).map_err(|_| GitError::InvalidUtf8)
    }
}

#[derive(Clone, Copy)]
enum BoundedStream {
    Stdout,
    Stderr,
}

enum BoundedReadMessage {
    Chunk(BoundedStream, Vec<u8>),
    Error(String),
}

fn forward_pipe<R: Read>(
    mut reader: R,
    stream: BoundedStream,
    sender: SyncSender<BoundedReadMessage>,
) {
    let mut buffer = [0_u8; GIT_OUTPUT_CHUNK_BYTES];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                if sender
                    .send(BoundedReadMessage::Chunk(stream, buffer[..count].to_vec()))
                    .is_err()
                {
                    break;
                }
            }
            Err(error) => {
                let _ = sender.send(BoundedReadMessage::Error(error.to_string()));
                break;
            }
        }
    }
}

fn bounded_process_output(
    mut command: Command,
    max_output_bytes: usize,
) -> Result<BoundedGitOutput, GitError> {
    command.env("GIT_NO_LAZY_FETCH", "1");
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| GitError::Command(error.to_string()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| GitError::Command("git stdout pipe was unavailable".into()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| GitError::Command("git stderr pipe was unavailable".into()))?;
    let (sender, receiver) = mpsc::sync_channel(4);
    let stdout_sender = sender.clone();
    let stdout_reader =
        thread::spawn(move || forward_pipe(stdout, BoundedStream::Stdout, stdout_sender));
    let stderr_reader = thread::spawn(move || forward_pipe(stderr, BoundedStream::Stderr, sender));

    let mut stdout_bytes = Vec::new();
    let mut stderr_bytes = Vec::new();
    let mut overflow = false;
    let mut read_error = None;
    while let Ok(message) = receiver.recv() {
        match message {
            BoundedReadMessage::Chunk(stream, bytes) => {
                let target = match stream {
                    BoundedStream::Stdout => &mut stdout_bytes,
                    BoundedStream::Stderr => &mut stderr_bytes,
                };
                if target.len().saturating_add(bytes.len()) > max_output_bytes {
                    if !overflow {
                        overflow = true;
                        terminate_bounded_process_group(&mut child);
                    }
                } else if !overflow {
                    target.extend_from_slice(&bytes);
                }
            }
            BoundedReadMessage::Error(error) => {
                if read_error.is_none() {
                    read_error = Some(error);
                    terminate_bounded_process_group(&mut child);
                }
            }
        }
    }
    let status = child
        .wait()
        .map_err(|error| GitError::Command(error.to_string()))?;
    let stdout_panicked = stdout_reader.join().is_err();
    let stderr_panicked = stderr_reader.join().is_err();
    if overflow {
        return Err(GitError::OutputLimitExceeded {
            limit: max_output_bytes,
        });
    }
    if let Some(error) = read_error {
        return Err(GitError::Command(format!(
            "failed reading git output: {error}"
        )));
    }
    if stdout_panicked || stderr_panicked {
        return Err(GitError::Command("git output reader failed".into()));
    }
    Ok(BoundedGitOutput {
        success: status.success(),
        stdout: stdout_bytes,
        stderr: stderr_bytes,
        exit_code: status.code(),
    })
}

fn terminate_bounded_process_group(child: &mut Child) {
    #[cfg(unix)]
    {
        // Every bounded Git child owns a fresh process group. Killing the
        // group also closes pipes inherited by Git filters and helpers.
        const SIGKILL: i32 = 9;
        let process_group = -(child.id() as i32);
        // SAFETY: the process group ID is the PID assigned by CommandExt.
        let _ = unsafe { kill(process_group, SIGKILL) };
    }
    let _ = child.kill();
}

fn command_error(stderr: &[u8]) -> GitError {
    GitError::Command(String::from_utf8_lossy(stderr).trim().to_owned())
}

fn resolve_git_path(root: &Path, value: &str) -> Result<PathBuf, GitError> {
    let path = if Path::new(value).is_absolute() {
        PathBuf::from(value)
    } else {
        root.join(value)
    };
    std::fs::canonicalize(&path).map_err(|_| GitError::InvalidTopology(path))
}

fn status_change_facts(status: &str) -> (Vec<String>, BTreeMap<String, ChangeKind>) {
    let mut kinds = BTreeMap::new();
    for line in status.lines() {
        let (code, raw_path) = if let Some(path) = line.strip_prefix("? ") {
            ("??", path)
        } else if let Some(path) = line.strip_prefix("1 ") {
            let mut fields = path.splitn(8, ' ');
            let Some(code) = fields.next() else {
                continue;
            };
            let Some(path) = fields.nth(6) else {
                continue;
            };
            (code, path)
        } else if let Some(path) = line.strip_prefix("2 ") {
            let mut fields = path.splitn(9, ' ');
            let Some(code) = fields.next() else {
                continue;
            };
            let Some(path) = fields.nth(7) else {
                continue;
            };
            (
                code,
                path.split_once('\t').map_or(path, |(target, _)| target),
            )
        } else if let Some(path) = line.strip_prefix("u ") {
            let mut fields = path.splitn(10, ' ');
            let Some(code) = fields.next() else {
                continue;
            };
            let Some(path) = fields.nth(8) else {
                continue;
            };
            (code, path)
        } else {
            continue;
        };
        let Some(path) = normalize_changed_paths([raw_path]).into_iter().next() else {
            continue;
        };
        let kind = change_kind_from_status_code(code);
        kinds.insert(path, kind);
    }
    (kinds.keys().cloned().collect(), kinds)
}

fn status_change_facts_nul(
    status: &[u8],
) -> Result<(Vec<String>, BTreeMap<String, ChangeKind>), GitError> {
    let mut kinds = BTreeMap::new();
    let mut records = status
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty());
    while let Some(record) = records.next() {
        let record = std::str::from_utf8(record).map_err(|_| GitError::InvalidUtf8)?;
        let mut renamed_source = None;
        let parsed = if let Some(path) = record.strip_prefix("? ") {
            Some(("??", path))
        } else if let Some(fields) = record.strip_prefix("1 ") {
            let columns = fields.splitn(8, ' ').collect::<Vec<_>>();
            (columns.len() == 8).then(|| (columns[0], columns[7]))
        } else if let Some(fields) = record.strip_prefix("2 ") {
            let columns = fields.splitn(9, ' ').collect::<Vec<_>>();
            // Porcelain v2 emits the original rename path as the next NUL
            // record. Keep it as a source deletion so moving a source file
            // into `.ai` cannot hide the dirty source path.
            let original_path = records.next();
            if columns.len() != 9 {
                None
            } else {
                if columns[0].contains('R')
                    && let Some(original_path) = original_path
                {
                    renamed_source = Some(
                        std::str::from_utf8(original_path).map_err(|_| GitError::InvalidUtf8)?,
                    );
                }
                Some((columns[0], columns[8]))
            }
        } else if let Some(fields) = record.strip_prefix("u ") {
            let columns = fields.splitn(10, ' ').collect::<Vec<_>>();
            (columns.len() == 10).then(|| (columns[0], columns[9]))
        } else {
            None
        };
        let Some((code, raw_path)) = parsed else {
            continue;
        };
        let Some(path) = normalize_changed_paths([raw_path]).into_iter().next() else {
            continue;
        };
        kinds.insert(path, change_kind_from_status_code(code));
        if let Some(original_path) = renamed_source
            && let Some(path) = normalize_changed_paths([original_path]).into_iter().next()
        {
            kinds.insert(path, ChangeKind::Deleted);
        }
    }
    Ok((kinds.keys().cloned().collect(), kinds))
}

fn status_v2_head(status: &str) -> Option<String> {
    let head = status
        .lines()
        .find_map(|line| line.strip_prefix("# branch.oid "))?;
    valid_commit_oid(head).then(|| head.to_owned())
}

fn status_v2_head_nul(status: &[u8]) -> Result<Option<String>, GitError> {
    for record in status.split(|byte| *byte == 0) {
        let Some(head) = record.strip_prefix(b"# branch.oid ") else {
            continue;
        };
        let head = std::str::from_utf8(head).map_err(|_| GitError::InvalidUtf8)?;
        return Ok(valid_commit_oid(head).then(|| head.to_owned()));
    }
    Ok(None)
}

fn valid_commit_oid(head: &str) -> bool {
    matches!(head.len(), 40 | 64) && head.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn status_v2_has_changes(status: &str) -> bool {
    status
        .lines()
        .any(|line| !line.is_empty() && !line.starts_with("# "))
}

fn change_kind_from_status_code(code: &str) -> ChangeKind {
    if code.contains('D') {
        ChangeKind::Deleted
    } else if code.contains('R') {
        ChangeKind::Renamed
    } else if code.contains('C') {
        ChangeKind::Copied
    } else if code.contains('A') || code == "??" {
        ChangeKind::Added
    } else if code.trim().is_empty() {
        ChangeKind::Unknown
    } else {
        ChangeKind::Modified
    }
}

fn comparison_change_facts(status: &str) -> (Vec<String>, BTreeMap<String, ChangeKind>) {
    let mut kinds = BTreeMap::new();
    let mut fields = status.split('\0');
    while let Some(code) = fields.next() {
        if code.is_empty() {
            continue;
        }
        let Some(raw_path) = fields.next() else {
            break;
        };
        let Some(path) = normalize_changed_paths([raw_path]).into_iter().next() else {
            continue;
        };
        let kind = match code.as_bytes().first().copied() {
            Some(b'A') => ChangeKind::Added,
            Some(b'D') => ChangeKind::Deleted,
            Some(b'M') => ChangeKind::Modified,
            Some(b'R') => ChangeKind::Renamed,
            Some(b'C') => ChangeKind::Copied,
            _ => ChangeKind::Unknown,
        };
        kinds.insert(path, kind);
    }
    (kinds.keys().cloned().collect(), kinds)
}

fn change_kind_name(kind: &ChangeKind) -> &'static str {
    match kind {
        ChangeKind::Added => "added",
        ChangeKind::Modified => "modified",
        ChangeKind::Deleted => "deleted",
        ChangeKind::Renamed => "renamed",
        ChangeKind::Copied => "copied",
        ChangeKind::Unknown => "unknown",
    }
}

fn diff_path(value: &str, prefix: &str) -> Option<String> {
    let value = value.strip_prefix(prefix)?;
    if value == "/dev/null" {
        return None;
    }
    normalize_changed_paths([value]).into_iter().next()
}

fn push_bounded(
    target: &mut Vec<String>,
    value: &str,
    state: &mut ChangeContentState,
    retained: &mut usize,
) {
    if retained.saturating_add(value.len()) > MAX_CHANGE_TEXT_BYTES {
        target.clear();
        *retained = MAX_CHANGE_TEXT_BYTES.saturating_add(1);
        *state = ChangeContentState::TooLarge;
    } else {
        target.push(value.to_owned());
        *retained += value.len();
    }
}

fn apply_patch_facts(
    patch: &str,
    evidence: &mut BTreeMap<String, ChangeEvidence>,
    diff_header_paths: Option<&[String]>,
) {
    let mut previous_path = None;
    let mut current_path = None;
    let mut retained_bytes = BTreeMap::<String, usize>::new();
    let mut hunk_counts = BTreeMap::<String, usize>::new();
    let mut current_hunk = None;
    let mut after_line = None;
    let mut diff_header_index = 0;
    for line in patch.lines() {
        if line.starts_with("diff --git ") {
            current_path =
                diff_header_paths.and_then(|paths| paths.get(diff_header_index).cloned());
            diff_header_index = diff_header_index.saturating_add(1);
            previous_path = current_path.clone();
            current_hunk = None;
            after_line = None;
        } else if let Some(path) = diff_path(line, "--- a/") {
            previous_path = Some(path);
        } else if line == "--- /dev/null" {
            previous_path = None;
        } else if let Some(path) = diff_path(line, "+++ b/") {
            current_path = Some(path);
        } else if line == "+++ /dev/null" {
            current_path = previous_path.clone();
        } else if let Some(path) = current_path.as_ref()
            && line.starts_with("@@ ")
        {
            after_line = line
                .split_whitespace()
                .find(|part| part.starts_with('+'))
                .and_then(|part| part[1..].split(',').next())
                .and_then(|number| number.parse::<usize>().ok());
            let next = hunk_counts.entry(path.clone()).or_default();
            current_hunk = Some(*next);
            *next = next.saturating_add(1);
        } else if line == "GIT binary patch"
            && let Some(path) = current_path.as_ref()
            && let Some(change) = evidence.get_mut(path)
        {
            change.content_state = ChangeContentState::Binary;
        } else if let Some(path) = current_path.as_ref()
            && let Some(change) = evidence.get_mut(path)
        {
            let retained = retained_bytes.entry(path.clone()).or_default();
            if let Some(added) = line.strip_prefix('+') {
                if let (Some(number), Some(hunk_index)) = (after_line, current_hunk) {
                    change.added_line_origins.push(AddedLineOrigin {
                        after_line: number,
                        hunk_index,
                    });
                    after_line = number.checked_add(1);
                }
                push_bounded(
                    &mut change.added_lines,
                    added,
                    &mut change.content_state,
                    retained,
                );
            } else if let Some(removed) = line.strip_prefix('-') {
                push_bounded(
                    &mut change.removed_lines,
                    removed,
                    &mut change.content_state,
                    retained,
                );
            } else if line.starts_with(' ') {
                after_line = after_line.and_then(|number| number.checked_add(1));
            }
        }
    }
    for change in evidence.values_mut() {
        // A large tracked file can still have a small, bounded patch. Keep
        // those patch facts available for governance inspection; only a
        // patch that itself exceeds the bound remains uninspectable.
        let patch_bytes = change
            .added_lines
            .iter()
            .chain(change.removed_lines.iter())
            .map(String::len)
            .sum::<usize>();
        if retained_bytes
            .get(&change.path)
            .is_some_and(|retained| *retained <= MAX_CHANGE_TEXT_BYTES)
            && patch_bytes <= MAX_CHANGE_TEXT_BYTES
            && (!change.added_lines.is_empty() || !change.removed_lines.is_empty())
        {
            change.content_state = ChangeContentState::Text;
        }
    }
}

fn is_ai_path(path: &[u8]) -> bool {
    path == b".ai" || path.starts_with(b".ai/")
}

fn digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{}", hex::encode(hasher.finalize()))
}
pub fn normalize_changed_paths<I, S>(paths: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut normalized = paths
        .into_iter()
        .map(|path| {
            let mut parts = Vec::new();
            for component in Path::new(path.as_ref()).components() {
                match component {
                    Component::CurDir | Component::RootDir | Component::Prefix(_) => {}
                    Component::ParentDir => {
                        parts.pop();
                    }
                    Component::Normal(value) => parts.push(value.to_string_lossy().into_owned()),
                }
            }
            parts.join("/")
        })
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

use serde::{Deserialize, Serialize};
use sha2::{Digest as ShaDigest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::path::{Component, Path};
use std::process::Command;
use std::time::UNIX_EPOCH;
use thiserror::Error;

pub const MAX_CHANGE_TEXT_BYTES: usize = 262_144;

/// A bounded, repository-local content identity cache.  It hashes only
/// declared relative files and derives a deterministic Merkle root from their
/// path/digest pairs.  Metadata is used solely as a cache hint; an unreadable
/// or ambiguous path is an error rather than an authorization to reuse a
/// stale digest.
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
        let mut files_reused = 0;
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
            let modified_ns = metadata
                .modified()
                .map_err(|source| ContentIdentityError::Metadata {
                    path: path.clone(),
                    source,
                })?
                .duration_since(UNIX_EPOCH)
                .map_err(|_| ContentIdentityError::InvalidTimestamp(path.clone()))?
                .as_nanos();
            let unchanged = self.entries.get(&relative).is_some_and(|entry| {
                entry.size == metadata.len() && entry.modified_ns == modified_ns
            });
            if unchanged {
                files_reused += 1;
                continue;
            }
            let bytes = std::fs::read(&path).map_err(|source| ContentIdentityError::Read {
                path: path.clone(),
                source,
            })?;
            files_read += 1;
            files_hashed += 1;
            self.entries.insert(
                relative,
                ContentIdentityEntry {
                    size: metadata.len(),
                    modified_ns,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangeEvidence {
    pub path: String,
    pub kind: ChangeKind,
    pub added_lines: Vec<String>,
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
    /// Source-only tree identity captured while reading the Git index. This
    /// is an internal request-scoped optimization hint; it is intentionally
    /// omitted from serialized snapshots so existing wire/digest semantics
    /// remain unchanged.
    #[serde(skip, default)]
    pub source_tree_digest: Option<String>,
}

pub struct GitRepository {
    root: PathBuf,
}

#[derive(Debug, Error)]
pub enum GitError {
    #[error("path is not a git repository: {0}")]
    NotRepository(PathBuf),
    #[error("git command failed: {0}")]
    Command(String),
    #[error("git output was not valid UTF-8")]
    InvalidUtf8,
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

    pub fn snapshot(&self) -> Result<RepositorySnapshot, GitError> {
        let head = self
            .run(["rev-parse", "HEAD"])
            .ok()
            .map(|value| value.trim().to_owned());
        let status = self.run(["status", "--porcelain=v1", "--untracked-files=all"])?;
        // A clean status proves that the equivalent diff is empty, so avoid
        // spawning a fourth Git process on the hot status path. Dirty or
        // otherwise uncertain input retains the full patch inspection path.
        let mut git_calls = 3;
        let diff = if status.is_empty() {
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
                        removed_lines: Vec::new(),
                        after_text: None,
                        content_state: ChangeContentState::Unavailable,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        apply_patch_facts(&diff, &mut change_evidence);
        let mut changed_hasher = Sha256::new();
        let mut changed_files_read = 0;
        let mut changed_files_hashed = 0;
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
                if let Some(change) = change_evidence.get_mut(relative) {
                    if change.content_state == ChangeContentState::TooLarge
                        || bytes.len() > MAX_CHANGE_TEXT_BYTES
                    {
                        change.after_text = None;
                        let patch_bytes = change
                            .added_lines
                            .iter()
                            .chain(change.removed_lines.iter())
                            .map(String::len)
                            .sum::<usize>();
                        change.content_state = if patch_bytes <= MAX_CHANGE_TEXT_BYTES
                            && (!change.added_lines.is_empty() || !change.removed_lines.is_empty())
                        {
                            ChangeContentState::Text
                        } else {
                            ChangeContentState::TooLarge
                        };
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
            source_tree_digest: Some(digest(&source_tree_hasher.finalize())),
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

fn status_change_facts(status: &str) -> (Vec<String>, BTreeMap<String, ChangeKind>) {
    let mut kinds = BTreeMap::new();
    for line in status.lines() {
        let Some(raw_path) = line.get(3..) else {
            continue;
        };
        let path = raw_path
            .rsplit_once(" -> ")
            .map_or(raw_path, |(_, target)| target);
        let Some(path) = normalize_changed_paths([path]).into_iter().next() else {
            continue;
        };
        let code = line.get(..2).unwrap_or("  ");
        let kind = if code.contains('D') {
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

fn apply_patch_facts(patch: &str, evidence: &mut BTreeMap<String, ChangeEvidence>) {
    let mut previous_path = None;
    let mut current_path = None;
    let mut retained_bytes = BTreeMap::<String, usize>::new();
    for line in patch.lines() {
        if line.starts_with("diff --git ") {
            previous_path = None;
            current_path = None;
        } else if let Some(path) = diff_path(line, "--- a/") {
            previous_path = Some(path);
        } else if line == "--- /dev/null" {
            previous_path = None;
        } else if let Some(path) = diff_path(line, "+++ b/") {
            current_path = Some(path);
        } else if line == "+++ /dev/null" {
            current_path = previous_path.clone();
        } else if let Some(path) = current_path.as_ref()
            && let Some(change) = evidence.get_mut(path)
        {
            let retained = retained_bytes.entry(path.clone()).or_default();
            if let Some(added) = line.strip_prefix('+') {
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
        if change.content_state == ChangeContentState::TooLarge
            && patch_bytes <= MAX_CHANGE_TEXT_BYTES
            && (!change.added_lines.is_empty() || !change.removed_lines.is_empty())
        {
            change.content_state = ChangeContentState::Text;
        }
    }
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

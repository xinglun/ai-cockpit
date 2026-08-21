use serde::{Deserialize, Serialize};
use sha2::{Digest as ShaDigest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::path::{Component, Path};
use std::process::Command;
use thiserror::Error;

pub const MAX_CHANGE_TEXT_BYTES: usize = 262_144;

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
        let diff = if head.is_some() {
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
                        change.content_state = ChangeContentState::TooLarge;
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
        Ok(RepositorySnapshot {
            root: self.root.clone(),
            git_root: self.root.clone(),
            head: head.filter(|value| !value.is_empty()),
            changed_paths,
            change_evidence: change_evidence.into_values().collect(),
            git_calls: 4,
            tree_digest: digest(tree.as_bytes()),
            diff_digest: format!("sha256:{}", hex::encode(changed_hasher.finalize())),
            dependency_fingerprint: format!("sha256:{}", hex::encode(dependency_hasher.finalize())),
            files_read: files_read + changed_files_read,
            files_hashed: files_hashed + changed_files_hashed,
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
        *state = ChangeContentState::TooLarge;
    } else if *state != ChangeContentState::TooLarge {
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

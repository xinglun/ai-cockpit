use serde::{Deserialize, Serialize};
use sha2::{Digest as ShaDigest, Sha256};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::path::{Component, Path};
use std::process::Command;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySnapshot {
    pub root: PathBuf,
    pub git_root: PathBuf,
    pub head: Option<String>,
    pub changed_paths: Vec<String>,
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
        let diff = self.run(["diff", "--no-ext-diff", "--name-status"])?;
        let tree = self.run(["ls-files", "-s"])?;
        let changed_paths = normalize_changed_paths(
            status
                .lines()
                .filter_map(|line| line.get(3..))
                .map(|line| line.rsplit_once(" -> ").map_or(line, |(_, path)| path)),
        );
        let mut changed_hasher = Sha256::new();
        let mut changed_files_read = 0;
        let mut changed_files_hashed = 0;
        let mut hashed_paths = BTreeSet::new();
        for line in diff.lines().filter(|line| !line.contains(".ai/")) {
            changed_hasher.update(line.as_bytes());
            changed_hasher.update([0]);
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

//! Fixed V1 material Git semantics, audited from 84fb04ee43e33842865a1544a5588466174c0639.
//! Flags, parsing, ordering, limits, and digest bytes are version-owned here.
//! The current bounded process and pipe safety implementation is reused.

use super::{
    AddedLineOrigin, BoundedGitOutput, BoundedSourceSnapshot, ChangeContentState, ChangeEvidence,
    ChangeKind, GitError, GitRepository, RepositorySnapshot, command_error,
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Component, Path},
};

const MAX_CHANGE_TEXT_BYTES: usize = 262_144;

impl GitRepository {
    /// V1 fixed index flags operation; the current bounded transport remains in force.
    pub fn material_v1_index_flags_bounded(
        &self,
        limit: usize,
    ) -> Result<BoundedGitOutput, GitError> {
        self.output_bounded(["ls-files", "-v", "-z"], limit)
    }
    /// V1 committed tree bytes, without interpreting checkout filters.
    pub fn material_v1_committed_tree_bounded(
        &self,
        revision: &str,
        limit: usize,
    ) -> Result<BoundedGitOutput, GitError> {
        if revision != "HEAD" && !valid_commit_oid(revision) {
            return Err(GitError::InvalidRevision(revision.to_owned()));
        }
        self.output_bounded(["ls-tree", "-r", "-z", revision], limit)
    }
    /// V1 raw blob operation; only full object IDs are accepted.
    pub fn material_v1_blob_bounded(
        &self,
        object_id: &str,
        limit: usize,
    ) -> Result<BoundedGitOutput, GitError> {
        if !valid_commit_oid(object_id) {
            return Err(GitError::InvalidRevision(object_id.to_owned()));
        }
        self.output_bounded(["cat-file", "blob", object_id], limit)
    }
    /// V1 ancestry operation, preserving the normal false result separately from errors.
    pub fn material_v1_is_ancestor_bounded(
        &self,
        base: &str,
        head: &str,
        limit: usize,
    ) -> Result<bool, GitError> {
        for revision in [base, head] {
            if !valid_commit_oid(revision) {
                return Err(GitError::InvalidRevision(revision.to_owned()));
            }
        }
        let output = self.output_bounded(["merge-base", "--is-ancestor", base, head], limit)?;
        match output.exit_code {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            _ => Err(command_error(&output.stderr)),
        }
    }
    pub fn material_source_snapshot_v1_bounded(
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

    pub fn material_source_snapshot_against_v1_bounded(
        &self,
        base: &str,
        max_output_bytes: usize,
    ) -> Result<RepositorySnapshot, GitError> {
        if !valid_commit_oid(base) {
            return Err(GitError::InvalidRevision(base.to_owned()));
        }
        let working = self.material_source_snapshot_v1_bounded(max_output_bytes)?;
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

#[cfg(test)]
mod tests {
    use super::super::{ChangeContentState, GitError, GitRepository};
    use std::{fs, path::Path, process::Command};

    fn git(root: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }

    fn fixture() -> (tempfile::TempDir, String) {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        git(root, &["init", "-q"]);
        git(root, &["config", "user.name", "Synthetic V1 fixture"]);
        git(
            root,
            &["config", "user.email", "synthetic-v1@example.invalid"],
        );
        git(root, &["config", "core.autocrlf", "false"]);
        git(root, &["config", "core.abbrev", "7"]);
        fs::create_dir(root.join("src")).unwrap();
        fs::write(
            root.join("src/sample.rs"),
            "fn sample() {\n    let value = \"old\";\n}\n",
        )
        .unwrap();
        git(root, &["add", "."]);
        git(root, &["commit", "-qm", "base"]);
        let base = git(root, &["rev-parse", "HEAD"]).trim().to_owned();
        (directory, base)
    }

    #[test]
    fn frozen_snapshot_has_exact_raw_patch_and_hunk_provenance() {
        let (directory, base) = fixture();
        let root = directory.path();
        fs::write(
            root.join("src/sample.rs"),
            "fn sample() {\n    let value = \"new\";\n}\n",
        )
        .unwrap();
        fs::create_dir(root.join(".ai")).unwrap();
        fs::write(root.join(".ai/ignored.json"), "{}\n").unwrap();
        git(root, &["add", "."]);
        git(root, &["commit", "-qm", "changed"]);
        let repository = GitRepository::discover(root).unwrap();
        let working = repository
            .material_source_snapshot_v1_bounded(1024 * 1024)
            .unwrap();
        assert!(working.changed_paths.is_empty());
        let snapshot = repository
            .material_source_snapshot_against_v1_bounded(&base, 1024 * 1024)
            .unwrap();
        assert_eq!(snapshot.changed_paths, ["src/sample.rs"]);
        let change = &snapshot.change_evidence[0];
        assert_eq!(change.content_state, ChangeContentState::Text);
        assert_eq!(change.added_lines, ["    let value = \"new\";"]);
        assert_eq!(change.removed_lines, ["    let value = \"old\";"]);
        assert_eq!(change.added_line_origins.len(), 1);
        assert_eq!(change.added_line_origins[0].after_line, 2);
        assert_eq!(change.added_line_origins[0].hunk_index, 0);
        assert_eq!(
            snapshot.diff_digest,
            "sha256:775686386838630790596946344b8d042a30c5f2d8d7d930570509c059b26e04"
        );
        assert_eq!(
            snapshot.source_tree_digest.as_deref(),
            Some("sha256:92e4d68a137c7468be88dd93872e24e783492b8926d737764fe42bda82c8cebc")
        );
    }

    #[test]
    fn frozen_status_keeps_the_original_source_of_a_rename_into_ai() {
        let (directory, _) = fixture();
        let root = directory.path();
        fs::create_dir(root.join(".ai")).unwrap();
        fs::rename(root.join("src/sample.rs"), root.join(".ai/sample.rs")).unwrap();
        git(root, &["add", "-A"]);
        let repository = GitRepository::discover(root).unwrap();
        let snapshot = repository
            .material_source_snapshot_v1_bounded(1024 * 1024)
            .unwrap();
        assert!(
            snapshot
                .changed_paths
                .iter()
                .any(|path| path == "src/sample.rs")
        );
    }

    #[test]
    fn frozen_snapshot_rejects_revision_expressions_and_bounds_output() {
        let (directory, _) = fixture();
        let repository = GitRepository::discover(directory.path()).unwrap();
        assert!(matches!(
            repository.material_source_snapshot_against_v1_bounded("HEAD~1", 1024),
            Err(GitError::InvalidRevision(_))
        ));
        assert!(matches!(
            repository.material_source_snapshot_v1_bounded(1),
            Err(GitError::OutputLimitExceeded { .. })
        ));
    }
}

//! Real Git regressions for the bounded legacy-v1 committed-source patch.
//! Display prefixes are never the source or blob content identity.

use cockpit_git::{ChangeKind, GitError, GitRepository, RepositorySnapshot};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

const LIMIT: usize = 1024 * 1024;
const FIRST_COLLISION: &[u8] = b"wi1097 collision fixture 21087\n";
const SECOND_COLLISION: &[u8] = b"wi1097 collision fixture 32356\n";
const FIRST_OID: &str = "7e4b69a26b2743edc5c158cabb4ad933e9dc67a0";
const SECOND_OID: &str = "7e4b69a21476aad3c6be57336ce8a39b0e8689f2";

struct Fixture {
    _directory: TempDir,
    root: PathBuf,
    base: String,
}

fn output(root: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("Git starts")
}

fn git(root: &Path, args: &[&str]) -> String {
    let result = output(root, args);
    assert!(
        result.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout)
        .expect("fixture output is UTF-8")
        .trim()
        .to_owned()
}

fn git_input(root: &Path, args: &[&str], input: &[u8]) -> Vec<u8> {
    let mut child = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Git starts with stdin");
    child
        .stdin
        .take()
        .expect("Git stdin")
        .write_all(input)
        .expect("write fixture input");
    let result = child.wait_with_output().expect("Git exits");
    assert!(
        result.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    result.stdout
}

impl Fixture {
    fn new(initial: &[u8], algorithm: &str) -> Self {
        let directory = tempfile::tempdir().expect("fixture directory");
        let root = directory.path().join("repository with spaces");
        std::fs::create_dir(&root).expect("repository directory");
        git(&root, &["init", "-q", "--initial-branch=main", algorithm]);
        for (key, value) in [
            ("user.name", "Fixture"),
            ("user.email", "fixture@example.invalid"),
            ("core.autocrlf", "false"),
            ("core.filemode", "false"),
            ("core.abbrev", "8"),
            ("commit.gpgsign", "false"),
        ] {
            git(&root, &["config", key, value]);
        }
        std::fs::write(root.join("subject"), initial).expect("base source");
        git(&root, &["add", "subject"]);
        git(&root, &["commit", "-q", "-m", "base"]);
        let base = git(&root, &["rev-parse", "HEAD"]);
        Self {
            _directory: directory,
            root,
            base,
        }
    }

    fn change(&self, bytes: &[u8]) {
        std::fs::write(self.root.join("subject"), bytes).expect("changed source");
        self.commit();
    }

    fn commit(&self) {
        git(&self.root, &["add", "-A"]);
        git(&self.root, &["commit", "-q", "-m", "change"]);
    }

    fn snapshot(&self) -> RepositorySnapshot {
        GitRepository::discover(&self.root)
            .expect("repository")
            .source_snapshot_against_bounded(&self.base, LIMIT)
            .expect("bounded source snapshot")
    }

    // Independent byte oracle: actual pre-fix Git output at the explicitly
    // supported eight-character legacy setting, before adding collisions.
    fn legacy_patch(&self) -> Vec<u8> {
        let result = output(
            &self.root,
            &[
                "-c",
                "core.quotePath=false",
                "-c",
                "core.abbrev=8",
                "diff",
                "--no-ext-diff",
                "--no-textconv",
                "--no-color",
                "--no-renames",
                "--diff-algorithm=myers",
                "--binary",
                "--unified=0",
                "-O/dev/null",
                &self.base,
                "HEAD",
                "--",
                ".",
                ":(exclude).ai",
                ":(exclude).ai/**",
            ],
        );
        assert!(result.status.success());
        result.stdout
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

#[test]
fn textual_v1_identity_ignores_abbrev_configuration() {
    let fixture = Fixture::new(b"before\n", "--object-format=sha1");
    fixture.change(b"after\n");
    let expected = digest(&fixture.legacy_patch());
    let source = fixture.snapshot().source_tree_digest;
    for abbreviation in ["7", "8", "9", "40", "no", "auto"] {
        git(&fixture.root, &["config", "core.abbrev", abbreviation]);
        let snapshot = fixture.snapshot();
        assert_eq!(snapshot.source_tree_digest, source);
        assert_eq!(snapshot.diff_digest, expected, "core.abbrev={abbreviation}");
    }
}

#[test]
fn unrelated_object_growth_does_not_change_v1_identity() {
    let fixture = Fixture::new(b"before\n", "--object-format=sha1");
    fixture.change(b"after\n");
    git(&fixture.root, &["config", "core.abbrev", "auto"]);
    let before = fixture.snapshot();
    let mut stream = Vec::new();
    stream.extend_from_slice(b"feature done\n");
    for n in 0..66_000 {
        let bytes = format!("unrelated object {n}\n");
        write!(stream, "blob\ndata {}\n{}\n", bytes.len(), bytes).unwrap();
    }
    stream.extend_from_slice(b"done\n");
    git_input(&fixture.root, &["fast-import", "--quiet"], &stream);
    let counts = git(&fixture.root, &["count-objects", "-v"]);
    let packed: usize = counts
        .lines()
        .find_map(|line| line.strip_prefix("in-pack: "))
        .expect("packed count")
        .parse()
        .expect("numeric packed count");
    assert!(
        packed >= 66_000,
        "fixture must actually grow its object store"
    );
    let after = fixture.snapshot();
    assert_eq!(before.head, after.head);
    assert_eq!(before.source_tree_digest, after.source_tree_digest);
    assert_eq!(before.diff_digest, after.diff_digest);
}

#[test]
fn prefix_collision_does_not_change_display_identity_or_hide_source_tampering() {
    let fixture = Fixture::new(b"base\n", "--object-format=sha1");
    fixture.change(FIRST_COLLISION);
    assert_eq!(
        git(&fixture.root, &["rev-parse", "HEAD:subject"]),
        FIRST_OID
    );
    let expected = digest(&fixture.legacy_patch());
    let before = fixture.snapshot();
    let collision = git_input(
        &fixture.root,
        &["hash-object", "-w", "--stdin"],
        SECOND_COLLISION,
    );
    assert_eq!(String::from_utf8(collision).unwrap().trim(), SECOND_OID);
    assert_eq!(&FIRST_OID[..8], &SECOND_OID[..8]);
    assert_ne!(FIRST_OID, SECOND_OID);
    let after_object = fixture.snapshot();
    assert_eq!(before.source_tree_digest, after_object.source_tree_digest);
    assert_eq!(expected, after_object.diff_digest);

    fixture.change(SECOND_COLLISION);
    let tampered = fixture.snapshot();
    assert_eq!(
        git(&fixture.root, &["rev-parse", "HEAD:subject"]),
        SECOND_OID
    );
    assert_ne!(before.source_tree_digest, tampered.source_tree_digest);
    assert_ne!(expected, tampered.diff_digest);
    assert_eq!(
        output(&fixture.root, &["show", "HEAD:subject"]).stdout,
        SECOND_COLLISION
    );
    assert_eq!(
        tampered.change_evidence[0].added_lines,
        [std::str::from_utf8(SECOND_COLLISION).unwrap().trim_end()]
    );
}

#[test]
fn index_and_binary_markers_inside_hunks_remain_source_text() {
    let fixture = Fixture::new(b"before\n", "--object-format=sha1");
    let text = concat!(
        "index 1111111111111111111111111111111111111111..2222222222222222222222222222222222222222 100644\n",
        "diff --git a/fake b/fake\n",
        "GIT binary patch\n",
        "literal 1\n",
        "@@ -1 +1 @@\n"
    );
    fixture.change(text.as_bytes());
    let expected = digest(&fixture.legacy_patch());
    git(&fixture.root, &["config", "core.abbrev", "40"]);
    let snapshot = fixture.snapshot();
    assert_eq!(snapshot.diff_digest, expected);
    assert_eq!(
        output(&fixture.root, &["show", "HEAD:subject"]).stdout,
        text.as_bytes()
    );
    for line in text.lines() {
        assert!(
            snapshot.change_evidence[0]
                .added_lines
                .iter()
                .any(|actual| actual == line)
        );
    }
}

#[test]
fn binary_full_index_and_payload_remain_bound() {
    let fixture = Fixture::new(b"before\0binary\n", "--object-format=sha1");
    fixture.change(b"after\0binary\n");
    let patch = fixture.legacy_patch();
    let text = String::from_utf8(patch.clone()).expect("Git binary patch is encoded text");
    let index = text
        .lines()
        .find(|line| line.starts_with("index "))
        .unwrap();
    let ids = index
        .strip_prefix("index ")
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap();
    assert_eq!(ids.split("..").map(str::len).collect::<Vec<_>>(), [40, 40]);
    assert!(text.contains("GIT binary patch\n"));
    let before = fixture.snapshot();
    for abbreviation in ["7", "9", "40"] {
        git(&fixture.root, &["config", "core.abbrev", abbreviation]);
        assert_eq!(fixture.snapshot().diff_digest, digest(&patch));
    }
    fixture.change(b"changed again\0binary\n");
    let tampered = fixture.snapshot();
    assert_ne!(before.diff_digest, tampered.diff_digest);
    assert_ne!(before.source_tree_digest, tampered.source_tree_digest);
}

#[test]
fn additions_deletions_and_mode_metadata_preserve_v1_bytes() {
    let fixture = Fixture::new(b"same\n", "--object-format=sha1");
    git(&fixture.root, &["mv", "subject", "renamed"]);
    std::fs::write(fixture.root.join("empty"), b"").unwrap();
    git(&fixture.root, &["add", "empty"]);
    git(&fixture.root, &["update-index", "--chmod=+x", "renamed"]);
    git(
        &fixture.root,
        &["commit", "-q", "-m", "rename mode empty add"],
    );
    let expected = digest(&fixture.legacy_patch());
    git(&fixture.root, &["config", "core.abbrev", "40"]);
    let snapshot = fixture.snapshot();
    assert_eq!(snapshot.diff_digest, expected);
    assert!(
        snapshot
            .change_evidence
            .iter()
            .any(|item| item.path == "subject" && item.kind == ChangeKind::Deleted)
    );
    assert!(
        snapshot
            .change_evidence
            .iter()
            .any(|item| item.path == "renamed" && item.kind == ChangeKind::Added)
    );
    assert!(
        snapshot
            .change_evidence
            .iter()
            .any(|item| item.path == "empty" && item.kind == ChangeKind::Added)
    );
}

#[test]
fn expanded_full_index_output_must_fit_the_original_read_budget() {
    let fixture = Fixture::new(b"before\n", "--object-format=sha1");
    fixture.change(b"after\n");
    let legacy_len = fixture.legacy_patch().len();
    let result = GitRepository::discover(&fixture.root)
        .unwrap()
        .source_snapshot_against_bounded(&fixture.base, legacy_len);
    assert!(matches!(result, Err(GitError::OutputLimitExceeded { limit }) if limit == legacy_len));
}

#[test]
fn binary_addition_and_deletion_keep_full_zero_object_ids() {
    let fixture = Fixture::new(b"deleted\0binary\n", "--object-format=sha1");
    std::fs::remove_file(fixture.root.join("subject")).unwrap();
    std::fs::write(fixture.root.join("added"), b"added\0binary\n").unwrap();
    fixture.commit();
    let patch = fixture.legacy_patch();
    let text = String::from_utf8(patch.clone()).unwrap();
    let zero = "0".repeat(40);
    assert!(text.contains(&format!("index {zero}..")));
    assert!(text.contains(&format!("..{zero}\n")));
    assert_eq!(text.matches("GIT binary patch\n").count(), 2);
    git(&fixture.root, &["config", "core.abbrev", "7"]);
    assert_eq!(fixture.snapshot().diff_digest, digest(&patch));
}

#[test]
fn mode_only_change_stays_bound_without_an_index_header() {
    let fixture = Fixture::new(b"unchanged content\n", "--object-format=sha1");
    let before = fixture.snapshot();
    git(&fixture.root, &["update-index", "--chmod=+x", "subject"]);
    git(&fixture.root, &["commit", "-q", "-m", "mode only"]);
    let patch = fixture.legacy_patch();
    let text = std::str::from_utf8(&patch).unwrap();
    assert!(text.contains("old mode 100644\nnew mode 100755\n"));
    assert!(!text.lines().any(|line| line.starts_with("index ")));
    git(&fixture.root, &["config", "core.abbrev", "40"]);
    let after = fixture.snapshot();
    assert_eq!(after.diff_digest, digest(&patch));
    assert_ne!(before.source_tree_digest, after.source_tree_digest);
    assert_eq!(
        output(&fixture.root, &["show", "HEAD:subject"]).stdout,
        b"unchanged content\n"
    );
}

#[test]
fn non_blob_gitlink_mode_is_explicitly_unsupported_by_v1() {
    let fixture = Fixture::new(b"base\n", "--object-format=sha1");
    let entry = format!("160000,{},module", fixture.base);
    git(
        &fixture.root,
        &["update-index", "--add", "--cacheinfo", &entry],
    );
    git(&fixture.root, &["commit", "-q", "-m", "gitlink"]);
    let result = GitRepository::discover(&fixture.root)
        .unwrap()
        .source_snapshot_against_bounded(&fixture.base, LIMIT);
    assert!(
        matches!(result, Err(GitError::Command(message)) if message.contains("v1") && message.contains("non-blob"))
    );
}

#[test]
fn mixed_gitlink_change_is_rejected_under_every_submodule_display() {
    let mut fixture = Fixture::new(b"before\n", "--object-format=sha1");
    let first = fixture.base.clone();
    fixture.change(b"middle\n");
    let second = git(&fixture.root, &["rev-parse", "HEAD"]);
    let entry = format!("160000,{first},zz-module");
    git(
        &fixture.root,
        &["update-index", "--add", "--cacheinfo", &entry],
    );
    git(&fixture.root, &["commit", "-q", "-m", "gitlink base"]);
    fixture.base = git(&fixture.root, &["rev-parse", "HEAD"]);
    std::fs::write(fixture.root.join("subject"), b"after\n").unwrap();
    git(&fixture.root, &["add", "subject"]);
    let entry = format!("160000,{second},zz-module");
    git(&fixture.root, &["update-index", "--cacheinfo", &entry]);
    git(
        &fixture.root,
        &["commit", "-q", "-m", "mixed gitlink change"],
    );
    for display in ["log", "diff", "short"] {
        git(&fixture.root, &["config", "diff.submodule", display]);
        for ignore in ["none", "all"] {
            git(&fixture.root, &["config", "diff.ignoreSubmodules", ignore]);
            let result = GitRepository::discover(&fixture.root)
                .unwrap()
                .source_snapshot_against_bounded(&fixture.base, LIMIT);
            assert!(
                matches!(result, Err(GitError::Command(ref message)) if message.contains("v1") && message.contains("non-blob")),
                "diff.submodule={display}, ignore={ignore} must reject the gitlink, got {result:?}"
            );
        }
    }
}

#[test]
fn sha256_objects_are_not_silently_claimed_as_legacy_sha1_v1() {
    let fixture = Fixture::new(b"before\n", "--object-format=sha256");
    fixture.change(b"after\n");
    assert_eq!(fixture.base.len(), 64);
    let result = GitRepository::discover(&fixture.root)
        .unwrap()
        .source_snapshot_against_bounded(&fixture.base, LIMIT);
    assert!(
        matches!(result, Err(GitError::Command(message)) if message.contains("v1") && message.contains("SHA-1"))
    );
}

#[test]
fn invalid_abbreviation_configuration_remains_an_explicit_error() {
    let fixture = Fixture::new(b"before\n", "--object-format=sha1");
    fixture.change(b"after\n");
    let repository = GitRepository::discover(&fixture.root).unwrap();
    git(&fixture.root, &["config", "core.abbrev", "invalid"]);
    assert!(
        repository
            .source_snapshot_against_bounded(&fixture.base, LIMIT)
            .is_err()
    );
}

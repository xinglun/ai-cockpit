use cockpit_git::{ChangeContentState, ChangeKind, GitRepository, MAX_CHANGE_TEXT_BYTES};
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_REPOSITORY_ID: AtomicU64 = AtomicU64::new(0);

fn temporary_repository() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "cockpit-git-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("temp repo directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&path)
        .status()
        .expect("git init");
    Command::new("git")
        .args(["config", "user.email", "test@example.invalid"])
        .current_dir(&path)
        .status()
        .expect("git config");
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(&path)
        .status()
        .expect("git config");
    fs::write(path.join("README.md"), "initial\n").expect("write");
    Command::new("git")
        .args(["add", "."])
        .current_dir(&path)
        .status()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-qm", "initial"])
        .current_dir(&path)
        .status()
        .expect("git commit");
    path
}

#[test]
fn snapshot_observes_head_and_untracked_paths_with_one_snapshot_api() {
    let path = temporary_repository();
    fs::write(path.join("src.txt"), "change\n").expect("write");
    let repository = GitRepository::discover(&path).expect("discover");
    let snapshot = repository.snapshot().expect("snapshot");
    assert!(snapshot.head.is_some());
    assert_eq!(snapshot.changed_paths, vec!["src.txt"]);
    assert_eq!(snapshot.git_calls, 4);
    assert_eq!(snapshot.change_evidence.len(), 1);
    assert_eq!(snapshot.change_evidence[0].kind, ChangeKind::Added);
    assert_eq!(
        snapshot.change_evidence[0].content_state,
        ChangeContentState::Text
    );
    assert_eq!(
        snapshot.change_evidence[0].after_text.as_deref(),
        Some("change\n")
    );
    assert!(snapshot.tree_digest.starts_with("sha256:"));
    assert!(
        snapshot
            .source_tree_digest
            .as_deref()
            .is_some_and(|value| value.starts_with("sha256:"))
    );
    assert!(snapshot.diff_digest.starts_with("sha256:"));
    assert!(snapshot.dependency_fingerprint.starts_with("sha256:"));
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn snapshot_reuses_tracked_patch_facts_without_serializing_source_text() {
    let path = temporary_repository();
    fs::write(path.join("README.md"), "changed\nSENTINEL_NEW_TEXT\n").expect("write");

    let snapshot = GitRepository::discover(&path)
        .expect("discover")
        .snapshot()
        .expect("snapshot");

    let change = snapshot
        .change_evidence
        .iter()
        .find(|change| change.path == "README.md")
        .expect("README change evidence");
    assert_eq!(change.kind, ChangeKind::Modified);
    assert!(change.removed_lines.iter().any(|line| line == "initial"));
    assert!(
        change
            .added_lines
            .iter()
            .any(|line| line == "SENTINEL_NEW_TEXT")
    );
    let serialized = serde_json::to_string(&snapshot).expect("serialize snapshot");
    assert!(!serialized.contains("SENTINEL_NEW_TEXT"));
    assert!(!serialized.contains("changeEvidence"));
    assert!(!serialized.contains("sourceTreeDigest"));
    assert_eq!(snapshot.git_calls, 4);
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn snapshot_marks_oversized_changed_text_without_retaining_it() {
    let path = temporary_repository();
    fs::write(
        path.join("policy.md"),
        vec![b'x'; MAX_CHANGE_TEXT_BYTES + 1],
    )
    .expect("write");

    let snapshot = GitRepository::discover(&path)
        .expect("discover")
        .snapshot()
        .expect("snapshot");

    let change = snapshot
        .change_evidence
        .iter()
        .find(|change| change.path == "policy.md")
        .expect("policy change evidence");
    assert_eq!(change.content_state, ChangeContentState::TooLarge);
    assert!(change.after_text.is_none());
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn snapshot_diff_digest_excludes_ai_protocol_fact_changes() {
    let path = temporary_repository();
    fs::create_dir_all(path.join(".ai")).expect("ai directory");
    fs::write(path.join(".ai/fact.json"), "{\"state\":\"created\"}\n").expect("fact");
    Command::new("git")
        .args(["add", ".ai/fact.json"])
        .current_dir(&path)
        .status()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-qm", "protocol fact"])
        .current_dir(&path)
        .status()
        .expect("git commit");
    let clean = GitRepository::discover(&path)
        .expect("discover")
        .snapshot()
        .expect("clean snapshot");

    fs::write(path.join(".ai/fact.json"), "{\"state\":\"verified\"}\n").expect("fact");
    let changed = GitRepository::discover(&path)
        .expect("discover")
        .snapshot()
        .expect("changed snapshot");

    assert_eq!(changed.diff_digest, clean.diff_digest);
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn oversized_tracked_patch_remains_uninspectable_when_after_text_is_small() {
    let path = temporary_repository();
    fs::write(
        path.join("policy.md"),
        vec![b'x'; MAX_CHANGE_TEXT_BYTES + 1],
    )
    .expect("write");
    Command::new("git")
        .args(["add", "policy.md"])
        .current_dir(&path)
        .status()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-qm", "large policy"])
        .current_dir(&path)
        .status()
        .expect("git commit");
    fs::write(path.join("policy.md"), "small\n").expect("write");

    let snapshot = GitRepository::discover(&path)
        .expect("discover")
        .snapshot()
        .expect("snapshot");
    let change = snapshot
        .change_evidence
        .iter()
        .find(|change| change.path == "policy.md")
        .expect("policy change");
    assert_eq!(change.content_state, ChangeContentState::TooLarge);
    assert!(change.after_text.is_none());
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn oversized_tracked_file_with_bounded_patch_keeps_patch_facts_inspectable() {
    let path = temporary_repository();
    let mut large = String::new();
    for _ in 0..(MAX_CHANGE_TEXT_BYTES / 8) {
        large.push_str("stable-line\n");
    }
    fs::write(path.join("large-policy.md"), &large).expect("write");
    Command::new("git")
        .args(["add", "large-policy.md"])
        .current_dir(&path)
        .status()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-qm", "large policy"])
        .current_dir(&path)
        .status()
        .expect("git commit");

    let mut changed = large;
    changed = changed.replacen("stable-line", "changed-line", 1);
    fs::write(path.join("large-policy.md"), changed).expect("write change");
    let snapshot = GitRepository::discover(&path)
        .expect("discover")
        .snapshot()
        .expect("snapshot");
    let change = snapshot
        .change_evidence
        .iter()
        .find(|change| change.path == "large-policy.md")
        .expect("large policy change");
    assert_eq!(change.content_state, ChangeContentState::Text);
    assert!(change.added_lines.iter().any(|line| line == "changed-line"));
    assert!(
        change
            .removed_lines
            .iter()
            .any(|line| line == "stable-line")
    );
    assert!(change.after_text.is_none());
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn snapshot_hashes_overlapping_changed_and_dependency_paths_once() {
    let path = temporary_repository();
    fs::write(path.join("Cargo.toml"), "[workspace]\nmembers=[]\n").expect("write");
    let snapshot = GitRepository::discover(&path)
        .expect("discover")
        .snapshot()
        .expect("snapshot");
    assert_eq!(snapshot.changed_paths, vec!["Cargo.toml"]);
    assert_eq!(snapshot.files_hashed, 1);
    assert_eq!(snapshot.files_read, 1);
    fs::remove_dir_all(path).expect("cleanup");
}

use cockpit_git::{
    AddedLineOrigin, ChangeContentState, ChangeKind, GitError, GitRepository,
    MAX_BOUNDED_GIT_OUTPUT_BYTES, MAX_CHANGE_TEXT_BYTES,
};
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
    assert_eq!(snapshot.git_calls, 3);
    assert_eq!(snapshot.bytes_read, b"change\n".len() as u64);
    assert_eq!(snapshot.bytes_hashed, b"change\n".len() as u64);
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
fn changed_rust_lines_keep_exact_after_line_and_hunk_origin() {
    let path = temporary_repository();
    let before = "fn main() {\n    let first = \"old\";\n    keep_one();\n    keep_two();\n    keep_three();\n    let second = \"old\";\n}\n";
    let after = before
        .replacen("first = \"old\"", "first = \"new\"", 1)
        .replacen("second = \"old\"", "second = \"new\"", 1);
    fs::write(path.join("src.rs"), before).expect("write source");
    Command::new("git")
        .args(["add", "src.rs"])
        .current_dir(&path)
        .status()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-qm", "source baseline"])
        .current_dir(&path)
        .status()
        .expect("git commit");
    fs::write(path.join("src.rs"), after).expect("edit source");

    let snapshot = GitRepository::discover(&path)
        .expect("discover")
        .snapshot()
        .expect("snapshot");
    let change = snapshot
        .change_evidence
        .iter()
        .find(|change| change.path == "src.rs")
        .expect("source change");
    assert_eq!(change.added_lines.len(), 2);
    assert_eq!(
        change.added_line_origins,
        vec![
            AddedLineOrigin {
                after_line: 2,
                hunk_index: 0,
            },
            AddedLineOrigin {
                after_line: 6,
                hunk_index: 1,
            }
        ]
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn overflowing_patch_with_a_short_removed_line_never_becomes_complete_text() {
    let path = temporary_repository();
    fs::write(path.join("src.rs"), "fn old() {}\n").expect("write baseline");
    Command::new("git")
        .args(["add", "src.rs"])
        .current_dir(&path)
        .status()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-qm", "source baseline"])
        .current_dir(&path)
        .status()
        .expect("git commit");
    let source = format!(
        "fn prompt() {{ let material = \"{}\"; }}\n",
        "x".repeat(MAX_CHANGE_TEXT_BYTES + 1)
    );
    fs::write(path.join("src.rs"), source).expect("write oversized source");
    let snapshot = GitRepository::discover(&path)
        .expect("discover")
        .snapshot()
        .expect("snapshot");
    let change = snapshot
        .change_evidence
        .iter()
        .find(|change| change.path == "src.rs")
        .expect("source change");
    assert_eq!(change.content_state, ChangeContentState::TooLarge);
    assert!(change.after_text.is_none());
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn clean_snapshot_skips_redundant_diff_subprocess() {
    let path = temporary_repository();
    let repository = GitRepository::discover(&path).expect("discover");
    let snapshot = repository.snapshot().expect("snapshot");
    assert!(snapshot.changed_paths.is_empty());
    assert_eq!(snapshot.git_calls, 2);
    assert!(snapshot.diff_digest.starts_with("sha256:"));
    assert!(snapshot.source_tree_digest.is_some());
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn snapshot_keeps_an_unborn_repository_headless() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("cockpit-git-unborn-{suffix}"));
    fs::create_dir_all(&path).expect("temp repository");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&path)
        .status()
        .expect("git init");
    fs::write(path.join("README.md"), "unborn\n").expect("write change");

    let snapshot = GitRepository::discover(&path)
        .expect("discover")
        .snapshot()
        .expect("snapshot");
    assert!(snapshot.head.is_none());
    assert_eq!(snapshot.changed_paths, vec!["README.md"]);
    assert_eq!(snapshot.change_evidence[0].kind, ChangeKind::Added);
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn snapshot_uses_the_v2_rename_target_path() {
    let path = temporary_repository();
    Command::new("git")
        .args(["mv", "README.md", "renamed.md"])
        .current_dir(&path)
        .status()
        .expect("rename");

    let snapshot = GitRepository::discover(&path)
        .expect("discover")
        .snapshot()
        .expect("snapshot");
    assert_eq!(snapshot.changed_paths, vec!["renamed.md"]);
    assert_eq!(snapshot.change_evidence[0].kind, ChangeKind::Renamed);
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn comparison_snapshot_includes_committed_changes_on_a_clean_worktree() {
    let path = temporary_repository();
    let base = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&path)
            .output()
            .expect("git revision")
            .stdout,
    )
    .expect("revision UTF-8")
    .trim()
    .to_owned();
    fs::create_dir_all(path.join("tests")).expect("tests directory");
    fs::write(
        path.join("tests/committed.rs"),
        "#[test]\nfn committed() {}\n",
    )
    .expect("write");
    Command::new("git")
        .args(["add", "tests/committed.rs"])
        .current_dir(&path)
        .status()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-qm", "committed source"])
        .current_dir(&path)
        .status()
        .expect("git commit");

    let snapshot = GitRepository::discover(&path)
        .expect("discover")
        .snapshot_against(&base)
        .expect("comparison snapshot");
    assert_eq!(snapshot.changed_paths, vec!["tests/committed.rs"]);
    let change = &snapshot.change_evidence[0];
    assert_eq!(change.kind, ChangeKind::Added);
    assert_eq!(change.content_state, ChangeContentState::Text);
    assert!(change.added_lines.iter().any(|line| line == "#[test]"));
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn comparison_snapshot_preserves_uncommitted_source_paths() {
    let path = temporary_repository();
    let base = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&path)
            .output()
            .expect("git revision")
            .stdout,
    )
    .expect("revision UTF-8")
    .trim()
    .to_owned();
    fs::write(path.join("README.md"), "working-tree change\n").expect("write");

    let snapshot = GitRepository::discover(&path)
        .expect("discover")
        .snapshot_against(&base)
        .expect("comparison snapshot");

    assert_eq!(snapshot.changed_paths, vec!["README.md"]);
    let change = snapshot
        .change_evidence
        .iter()
        .find(|change| change.path == "README.md")
        .expect("working-tree change evidence");
    assert_eq!(change.kind, ChangeKind::Modified);
    assert_eq!(change.after_text.as_deref(), Some("working-tree change\n"));
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
    assert_eq!(snapshot.git_calls, 3);
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

#[test]
fn bounded_source_snapshot_rejects_output_overflow_and_can_run_again() {
    let path = temporary_repository();
    let repository = GitRepository::discover(&path).expect("discover");

    let overflow = repository.source_snapshot_bounded(1).unwrap_err();
    assert!(matches!(
        overflow,
        GitError::OutputLimitExceeded { limit: 1 }
    ));

    let snapshot = repository
        .source_snapshot_bounded(MAX_BOUNDED_GIT_OUTPUT_BYTES)
        .expect("bounded retry after overflow");
    assert!(snapshot.head.is_some());
    assert!(snapshot.source_tree_digest.starts_with("sha256:"));
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn bounded_git_revision_arguments_reject_option_injection() {
    let path = temporary_repository();
    let repository = GitRepository::discover(&path).expect("discover");
    assert!(matches!(
        repository.source_snapshot_against_bounded(
            "--ext-diff",
            MAX_BOUNDED_GIT_OUTPUT_BYTES
        ),
        Err(GitError::InvalidRevision(value)) if value == "--ext-diff"
    ));
    fs::remove_dir_all(path).expect("cleanup");
}

#[cfg(windows)]
#[test]
fn windows_job_object_allows_normal_bounded_git_snapshot() {
    let path = temporary_repository();
    let repository = GitRepository::discover(&path).expect("discover");
    let snapshot = repository
        .source_snapshot_bounded(MAX_BOUNDED_GIT_OUTPUT_BYTES)
        .expect("normal Git process completes under its Job Object");

    assert!(snapshot.head.is_some());
    assert!(snapshot.changed_paths.is_empty());
    assert!(snapshot.source_tree_digest.starts_with("sha256:"));
    fs::remove_dir_all(path).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn bounded_output_overflow_terminates_git_filter_process_group() {
    use std::{
        os::unix::fs::PermissionsExt,
        time::{Duration, Instant},
    };

    let path = temporary_repository();
    let helper = path.join(".git/clean");
    let marker = path.join(".git/clean.called");
    fs::write(path.join(".gitattributes"), "README.md filter=delay\n").expect("write attributes");
    Command::new("git")
        .args(["config", "filter.delay.clean", "cat"])
        .current_dir(&path)
        .status()
        .expect("configure baseline clean filter");
    Command::new("git")
        .args(["add", ".gitattributes"])
        .current_dir(&path)
        .status()
        .expect("stage attributes");
    Command::new("git")
        .args(["commit", "-qm", "configure clean filter"])
        .current_dir(&path)
        .status()
        .expect("commit attributes");
    fs::write(
        &helper,
        format!(
            "#!/bin/sh\nprintf called > '{}'\nprintf 'filter overflow\\n' >&2\nsleep 2\ncat\n",
            marker.display()
        ),
    )
    .expect("write filter helper");
    let mut permissions = fs::metadata(&helper)
        .expect("helper metadata")
        .permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&helper, permissions).expect("executable helper");
    Command::new("git")
        .args(["config", "filter.delay.clean"])
        .arg(&helper)
        .current_dir(&path)
        .status()
        .expect("configure clean filter");
    fs::write(path.join("README.md"), "initial\n").expect("rewrite checkout source");
    let source = fs::File::open(path.join("README.md")).expect("open checkout source");
    source
        .set_times(
            fs::FileTimes::new()
                .set_modified(std::time::UNIX_EPOCH + Duration::from_secs(1_000_000_000)),
        )
        .expect("set stale source timestamp");

    let repository = GitRepository::discover(&path).expect("discover");
    let started = Instant::now();
    let result = repository.source_snapshot_bounded(1);
    assert!(matches!(
        result,
        Err(GitError::OutputLimitExceeded { limit: 1 })
    ));
    assert!(marker.exists(), "the configured clean filter must run");
    assert!(started.elapsed() < Duration::from_secs(1));
    fs::remove_dir_all(path).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn bounded_blob_read_never_lazily_fetches_from_promisor_remote() {
    use std::{
        os::unix::fs::PermissionsExt,
        time::{Duration, Instant},
    };

    let source = temporary_repository();
    let partial = source.with_extension("partial");
    let helper = source.with_extension("upload-pack");
    let marker = source.with_extension("upload-pack.called");
    let object_id = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD:README.md"])
            .current_dir(&source)
            .output()
            .expect("blob revision")
            .stdout,
    )
    .expect("blob ID UTF-8")
    .trim()
    .to_owned();
    Command::new("git")
        .args(["config", "uploadpack.allowFilter", "true"])
        .current_dir(&source)
        .status()
        .expect("enable partial clone filter");
    let clone_url = format!("file://{}", source.display());
    let clone = Command::new("git")
        .args(["clone", "--quiet", "--filter=blob:none", "--no-checkout"])
        .arg(clone_url)
        .arg(&partial)
        .status()
        .expect("clone partial repository");
    assert!(clone.success(), "partial clone must succeed");
    fs::write(
        &helper,
        format!(
            "#!/bin/sh\nprintf called > '{}'\nsleep 2\nexec git-upload-pack \"$@\"\n",
            marker.display()
        ),
    )
    .expect("write upload-pack helper");
    let mut permissions = fs::metadata(&helper)
        .expect("helper metadata")
        .permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&helper, permissions).expect("executable helper");
    Command::new("git")
        .args(["config", "remote.origin.uploadpack"])
        .arg(&helper)
        .current_dir(&partial)
        .status()
        .expect("configure promisor helper");

    let repository = GitRepository::discover(&partial).expect("discover partial repository");
    let started = Instant::now();
    let result = repository.blob_bounded(&object_id, 1);
    assert!(result.is_err());
    assert!(
        !marker.exists(),
        "bounded reads must not invoke promisor helpers"
    );
    assert!(started.elapsed() < Duration::from_secs(1));
    fs::remove_file(helper).expect("cleanup helper");
    fs::remove_dir_all(partial).expect("cleanup partial repository");
    fs::remove_dir_all(source).expect("cleanup source repository");
}

#[test]
fn bounded_binary_patch_mapping_ignores_configured_diff_order() {
    let path = temporary_repository();
    let base = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&path)
            .output()
            .expect("git revision")
            .stdout,
    )
    .expect("revision UTF-8")
    .trim()
    .to_owned();
    fs::write(path.join("a.bin"), b"binary\0material").expect("binary source");
    fs::write(path.join("z.rs"), "fn visible() {}\n").expect("text source");
    Command::new("git")
        .args(["add", "a.bin", "z.rs"])
        .current_dir(&path)
        .status()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-qm", "mixed source"])
        .current_dir(&path)
        .status()
        .expect("git commit");
    let order_file = path.with_extension("diff-order");
    fs::write(&order_file, "z.rs\na.bin\n").expect("diff order file");
    Command::new("git")
        .args(["config", "diff.orderFile"])
        .arg(&order_file)
        .current_dir(&path)
        .status()
        .expect("configure diff order");

    let snapshot = GitRepository::discover(&path)
        .expect("discover")
        .source_snapshot_against_bounded(&base, MAX_BOUNDED_GIT_OUTPUT_BYTES)
        .expect("bounded comparison");
    let binary = snapshot
        .change_evidence
        .iter()
        .find(|change| change.path == "a.bin")
        .expect("binary change");
    let rust = snapshot
        .change_evidence
        .iter()
        .find(|change| change.path == "z.rs")
        .expect("Rust change");
    assert_eq!(binary.content_state, ChangeContentState::Binary);
    assert_eq!(rust.content_state, ChangeContentState::Text);
    fs::remove_file(order_file).expect("cleanup order file");
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn bounded_source_comparison_preserves_existing_hunk_facts() {
    let path = temporary_repository();
    let before = "fn main() {\n    let first = \"old\";\n    keep_one();\n    keep_two();\n    keep_three();\n    let second = \"old\";\n}\n";
    fs::write(path.join("src.rs"), before).expect("write source");
    Command::new("git")
        .args(["add", "src.rs"])
        .current_dir(&path)
        .status()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-qm", "source baseline"])
        .current_dir(&path)
        .status()
        .expect("git commit");
    let base = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&path)
            .output()
            .expect("git revision")
            .stdout,
    )
    .expect("revision UTF-8")
    .trim()
    .to_owned();
    let after = before
        .replacen("first = \"old\"", "first = \"new\"", 1)
        .replacen("second = \"old\"", "second = \"new\"", 1);
    fs::write(path.join("src.rs"), after).expect("write changed source");
    Command::new("git")
        .args(["add", "src.rs"])
        .current_dir(&path)
        .status()
        .expect("git add changed source");
    Command::new("git")
        .args(["commit", "-qm", "source change"])
        .current_dir(&path)
        .status()
        .expect("git commit changed source");

    let repository = GitRepository::discover(&path).expect("discover");
    let existing = repository
        .snapshot_against(&base)
        .expect("existing comparison snapshot");
    let bounded = repository
        .source_snapshot_against_bounded(&base, MAX_BOUNDED_GIT_OUTPUT_BYTES)
        .expect("bounded source comparison");
    let existing_change = existing
        .change_evidence
        .iter()
        .find(|change| change.path == "src.rs")
        .expect("existing source change");
    let bounded_change = bounded
        .change_evidence
        .iter()
        .find(|change| change.path == "src.rs")
        .expect("bounded source change");

    assert_eq!(bounded.changed_paths, existing.changed_paths);
    assert_eq!(bounded.diff_digest, existing.diff_digest);
    assert_eq!(bounded_change.kind, existing_change.kind);
    assert_eq!(bounded_change.added_lines, existing_change.added_lines);
    assert_eq!(
        bounded_change.added_line_origins,
        existing_change.added_line_origins
    );
    assert_eq!(bounded_change.removed_lines, existing_change.removed_lines);
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn bounded_source_identity_ignores_ai_only_commits() {
    let path = temporary_repository();
    let base = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&path)
            .output()
            .expect("git revision")
            .stdout,
    )
    .expect("revision UTF-8")
    .trim()
    .to_owned();
    let repository = GitRepository::discover(&path).expect("discover");
    let before = repository
        .source_snapshot_against_bounded(&base, MAX_BOUNDED_GIT_OUTPUT_BYTES)
        .expect("initial bounded source comparison");

    fs::create_dir(path.join(".ai")).expect("ai directory");
    fs::write(path.join(".ai/fact.json"), "{\"state\":\"created\"}\n").expect("fact");
    Command::new("git")
        .args(["add", ".ai/fact.json"])
        .current_dir(&path)
        .status()
        .expect("git add ai fact");
    Command::new("git")
        .args(["commit", "-qm", "protocol fact"])
        .current_dir(&path)
        .status()
        .expect("git commit ai fact");

    let after = repository
        .source_snapshot_against_bounded(&base, MAX_BOUNDED_GIT_OUTPUT_BYTES)
        .expect("ai-only descendant source comparison");
    let working = repository
        .source_snapshot_bounded(MAX_BOUNDED_GIT_OUTPUT_BYTES)
        .expect("ai-only descendant working snapshot");
    assert_eq!(before.diff_digest, after.diff_digest);
    assert_eq!(before.source_tree_digest, after.source_tree_digest);
    assert!(after.changed_paths.is_empty());
    assert!(working.changed_paths.is_empty());
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn bounded_source_comparison_marks_git_binary_patch_as_binary() {
    let path = temporary_repository();
    let base = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&path)
            .output()
            .expect("git revision")
            .stdout,
    )
    .expect("revision UTF-8")
    .trim()
    .to_owned();
    fs::create_dir(path.join("src")).expect("source directory");
    let binary_path = "src/matériel file.rs";
    fs::write(path.join(binary_path), b"binary\0material").expect("binary source");
    Command::new("git")
        .args(["add", binary_path])
        .current_dir(&path)
        .status()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-qm", "binary source"])
        .current_dir(&path)
        .status()
        .expect("git commit");

    let snapshot = GitRepository::discover(&path)
        .expect("discover")
        .source_snapshot_against_bounded(&base, MAX_BOUNDED_GIT_OUTPUT_BYTES)
        .expect("bounded source comparison");
    let change = snapshot
        .change_evidence
        .iter()
        .find(|change| change.path == binary_path)
        .expect("binary change");
    assert_eq!(change.content_state, ChangeContentState::Binary);
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn bounded_source_tree_digest_excludes_unicode_ai_paths() {
    let path = temporary_repository();
    let base = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&path)
            .output()
            .expect("git revision")
            .stdout,
    )
    .expect("revision UTF-8")
    .trim()
    .to_owned();
    let repository = GitRepository::discover(&path).expect("discover");
    let before = repository
        .source_snapshot_against_bounded(&base, MAX_BOUNDED_GIT_OUTPUT_BYTES)
        .expect("initial bounded source comparison");

    fs::create_dir(path.join(".ai")).expect("ai directory");
    fs::write(path.join(".ai/évidence.json"), "{}\n").expect("unicode ai fact");
    Command::new("git")
        .args(["add", ".ai/évidence.json"])
        .current_dir(&path)
        .status()
        .expect("git add ai fact");
    Command::new("git")
        .args(["commit", "-qm", "unicode protocol fact"])
        .current_dir(&path)
        .status()
        .expect("git commit ai fact");

    let after = repository
        .source_snapshot_against_bounded(&base, MAX_BOUNDED_GIT_OUTPUT_BYTES)
        .expect("unicode ai-only descendant comparison");
    assert_eq!(before.source_tree_digest, after.source_tree_digest);
    assert_eq!(before.diff_digest, after.diff_digest);
    assert!(after.changed_paths.is_empty());
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn bounded_source_patch_digest_excludes_root_ai_file() {
    let path = temporary_repository();
    let base = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&path)
            .output()
            .expect("git revision")
            .stdout,
    )
    .expect("revision UTF-8")
    .trim()
    .to_owned();
    let repository = GitRepository::discover(&path).expect("discover");
    let before = repository
        .source_snapshot_against_bounded(&base, MAX_BOUNDED_GIT_OUTPUT_BYTES)
        .expect("initial bounded source comparison");

    fs::write(path.join(".ai"), "protocol root fact\n").expect("ai root fact");
    Command::new("git")
        .args(["add", ".ai"])
        .current_dir(&path)
        .status()
        .expect("git add ai root fact");
    Command::new("git")
        .args(["commit", "-qm", "root protocol fact"])
        .current_dir(&path)
        .status()
        .expect("git commit ai root fact");

    let after = repository
        .source_snapshot_against_bounded(&base, MAX_BOUNDED_GIT_OUTPUT_BYTES)
        .expect("ai root descendant comparison");
    assert_eq!(before.diff_digest, after.diff_digest);
    assert!(after.changed_paths.is_empty());
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn bounded_source_status_preserves_unicode_ai_worktree_paths() {
    let path = temporary_repository();
    fs::create_dir(path.join(".ai")).expect("ai directory");
    fs::write(path.join(".ai/évidence.json"), "{}\n").expect("unicode ai fact");
    let repository = GitRepository::discover(&path).expect("discover");

    let snapshot = repository
        .source_snapshot_bounded(MAX_BOUNDED_GIT_OUTPUT_BYTES)
        .expect("bounded source status");
    assert_eq!(snapshot.changed_paths, vec![".ai/évidence.json"]);
    fs::remove_dir_all(path).expect("cleanup");
}

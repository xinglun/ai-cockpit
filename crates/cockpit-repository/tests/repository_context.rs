use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use cockpit_repository::{RepositoryExecutionContext, RuntimeSession, attach, scaffold_work_item};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn repository(name: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "cockpit-repository-context-{name}-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("repository");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .status()
            .expect("git init")
            .success()
    );
    root
}

#[test]
fn parallel_repository_contexts_do_not_share_scaffold_state() {
    let left = repository("left");
    let right = repository("right");
    fs::write(left.join("left.txt"), "left\n").expect("left fact");
    fs::write(right.join("right.txt"), "right\n").expect("right fact");
    std::thread::scope(|scope| {
        let left_handle = scope.spawn(|| {
            let profile = attach(&left).expect("attach left");
            let scaffold = scaffold_work_item(&left, "WI-LEFT", "code").expect("scaffold left");
            (profile, scaffold)
        });
        let right_handle = scope.spawn(|| {
            let profile = attach(&right).expect("attach right");
            let scaffold = scaffold_work_item(&right, "WI-RIGHT", "docs").expect("scaffold right");
            (profile, scaffold)
        });
        let (left_profile, left_scaffold) = left_handle.join().expect("left thread");
        let (right_profile, right_scaffold) = right_handle.join().expect("right thread");
        assert_ne!(left_profile.repository_id, right_profile.repository_id);
        assert_eq!(
            left_scaffold.known_facts.repository_id,
            left_profile.repository_id
        );
        assert_eq!(
            right_scaffold.known_facts.repository_id,
            right_profile.repository_id
        );
        assert_ne!(
            left_scaffold.known_facts.repository_snapshot_digest,
            right_scaffold.known_facts.repository_snapshot_digest
        );
        assert!(
            left.join(".ai/work-items/active/WI-LEFT.contract.json")
                .is_file()
        );
        assert!(
            !left
                .join(".ai/work-items/active/WI-RIGHT.contract.json")
                .exists()
        );
        assert!(
            right
                .join(".ai/work-items/active/WI-RIGHT.contract.json")
                .is_file()
        );
        assert!(
            !right
                .join(".ai/work-items/active/WI-LEFT.contract.json")
                .exists()
        );
    });
    fs::remove_dir_all(left).expect("cleanup left");
    fs::remove_dir_all(right).expect("cleanup right");
}

#[test]
fn execution_context_captures_one_snapshot_and_memoizes_observation() {
    let root = repository("memoized");
    fs::write(root.join("src.rs"), "fn value() -> u8 { 1 }\n").expect("source");
    attach(&root).expect("attach");
    let context = RepositoryExecutionContext::capture(&root).expect("capture");
    assert_eq!(context.snapshot().git_calls, 4);
    let original_tree = context.snapshot().tree_digest.clone();
    let first = context.observe().expect("observe");
    let second = context.observe().expect("observe again");
    assert!(std::ptr::eq(first, second));
    assert_eq!(first, second);

    fs::write(root.join("src.rs"), "fn value() -> u8 { 2 }\n").expect("change");
    // The request-scoped context remains bound to its original snapshot. A
    // caller that wants current facts must explicitly capture a new one.
    assert!(!context.snapshot().changed_paths.is_empty());
    assert_eq!(context.snapshot().tree_digest, original_tree);
    let fresh = RepositoryExecutionContext::capture(&root).expect("fresh context");
    assert_ne!(context.snapshot().diff_digest, fresh.snapshot().diff_digest);
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn runtime_session_reuses_only_explicit_repository_bindings() {
    let left = repository("session-left");
    let right = repository("session-right");
    attach(&left).expect("left attach");
    attach(&right).expect("right attach");
    let session = RuntimeSession::new();
    let left_first = session.bind(&left).expect("left bind");
    let left_second = session.bind(&left).expect("left bind again");
    let right_bound = session.bind(&right).expect("right bind");
    assert!(std::sync::Arc::ptr_eq(&left_first, &left_second));
    assert_ne!(left_first.repository_id(), right_bound.repository_id());
    assert_eq!(session.active_repositories().expect("active").len(), 2);
    assert!(session.unbind(&left).expect("unbind left"));
    assert_eq!(session.active_repositories().expect("active").len(), 1);
    fs::remove_dir_all(left).expect("cleanup left");
    fs::remove_dir_all(right).expect("cleanup right");
}

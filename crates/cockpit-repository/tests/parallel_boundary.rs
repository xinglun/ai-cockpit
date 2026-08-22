use cockpit_protocol::ConcurrencyBoundary;
use cockpit_repository::{
    WorkItemStartOptions, acquire_parallel_slot, attach, list_parallel_slots, preflight_work_item,
    release_parallel_slot, set_work_item_concurrency_boundary, set_work_item_intelligence,
    start_work_item_with_options, work_item_compatibility,
};
use std::process::Command;
use std::sync::Arc;
use std::thread;

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("tempdir");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    attach(directory.path()).expect("attach");
    directory
}

fn start_item(root: &std::path::Path, id: &str, scope: &str) {
    start_work_item_with_options(
        root,
        id,
        id,
        "parallel boundary",
        &[scope.into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start Work Item");
    set_work_item_intelligence(root, id, Vec::new(), Vec::new(), true)
        .expect("declare parallel Work Item");
}

fn boundary(path: &str, max_workers: u32) -> ConcurrencyBoundary {
    ConcurrencyBoundary {
        schema_version: 1,
        implementation_paths: vec![path.into()],
        generated_evidence_paths: vec![format!(".ai/evidence/{path}")],
        verification_output_paths: vec![format!("target/{path}")],
        serialized_projection_paths: vec![format!(".ai/work-items/{path}")],
        max_workers,
        reason: "explicit test boundary".into(),
    }
}

#[test]
fn contract_boundary_overlap_is_conservative_and_windows_safe() {
    let directory = repository();
    start_item(directory.path(), "WI-PARENT", "legacy-parent");
    start_item(directory.path(), "WI-CHILD", "legacy-child");
    set_work_item_concurrency_boundary(directory.path(), "WI-PARENT", boundary(r"src\**", 2))
        .expect("parent boundary");
    set_work_item_concurrency_boundary(directory.path(), "WI-CHILD", boundary("src/main.rs", 2))
        .expect("child boundary");

    let compatibility =
        work_item_compatibility(directory.path(), "WI-PARENT").expect("compatibility");
    assert!(!compatibility.compatible);
    assert_eq!(compatibility.conflicts, vec!["WI-CHILD"]);
    assert!(
        compatibility
            .reasons
            .iter()
            .any(|reason| reason.starts_with("concurrency_boundary_overlap:WI-CHILD:"))
    );
}

#[test]
fn binding_boundary_preserves_contract_readability_for_preflight_and_lifecycle() {
    let directory = repository();
    start_item(directory.path(), "WI-CONTRACT-BOUNDARY", "src/main.rs");
    set_work_item_concurrency_boundary(
        directory.path(),
        "WI-CONTRACT-BOUNDARY",
        boundary("src/main.rs", 1),
    )
    .expect("boundary");
    let contract_path = directory
        .path()
        .join(".ai/work-items/active/WI-CONTRACT-BOUNDARY.contract.json");
    let decision = preflight_work_item(directory.path(), &contract_path).expect("preflight");
    assert_ne!(decision.state, cockpit_core::DecisionState::Red);
}

#[test]
fn malformed_or_missing_boundary_fails_closed_for_slot_acquisition() {
    let directory = repository();
    start_item(directory.path(), "WI-NO-BOUNDARY", "src/**");
    assert!(acquire_parallel_slot(directory.path(), "WI-NO-BOUNDARY").is_err());
    assert!(list_parallel_slots(directory.path()).unwrap().is_empty());
}

#[test]
fn slot_capacity_and_duplicate_work_item_are_race_safe() {
    let directory = repository();
    for (id, path) in [
        ("WI-A", "src/a.rs"),
        ("WI-B", "src/b.rs"),
        ("WI-C", "src/c.rs"),
    ] {
        start_item(directory.path(), id, path);
        set_work_item_concurrency_boundary(directory.path(), id, boundary(path, 2))
            .expect("boundary");
    }
    let first = acquire_parallel_slot(directory.path(), "WI-A").expect("first slot");
    let second = acquire_parallel_slot(directory.path(), "WI-B").expect("second slot");
    assert_ne!(first.slot_id, second.slot_id);
    assert!(acquire_parallel_slot(directory.path(), "WI-C").is_err());
    assert!(acquire_parallel_slot(directory.path(), "WI-A").is_err());
    release_parallel_slot(directory.path(), "WI-A", &first.lease_id).expect("release first");
    release_parallel_slot(directory.path(), "WI-B", &second.lease_id).expect("release second");

    let concurrent = Arc::new(directory);
    let mut handles = Vec::new();
    for _ in 0..8 {
        let root = Arc::clone(&concurrent);
        handles.push(thread::spawn(move || {
            acquire_parallel_slot(root.path(), "WI-C")
        }));
    }
    let successes = handles
        .into_iter()
        .filter_map(|handle| handle.join().expect("thread join").ok())
        .collect::<Vec<_>>();
    assert_eq!(
        successes.len(),
        1,
        "duplicate Work Item must reserve one slot"
    );
    let lease = &successes[0];
    release_parallel_slot(concurrent.path(), "WI-C", &lease.lease_id).expect("release race lease");
    assert!(list_parallel_slots(concurrent.path()).unwrap().is_empty());
}

#[test]
fn first_use_parallel_directories_are_created_race_safe() {
    let directory = repository();
    for (id, path) in [
        ("WI-FIRST-A", "src/first-a.rs"),
        ("WI-FIRST-B", "src/first-b.rs"),
    ] {
        start_item(directory.path(), id, path);
        set_work_item_concurrency_boundary(directory.path(), id, boundary(path, 2))
            .expect("boundary");
    }
    assert!(!directory.path().join(".ai/parallel").exists());
    let concurrent = Arc::new(directory);
    let handles = ["WI-FIRST-A", "WI-FIRST-B"]
        .into_iter()
        .map(|id| {
            let root = Arc::clone(&concurrent);
            thread::spawn(move || acquire_parallel_slot(root.path(), id))
        })
        .collect::<Vec<_>>();
    let leases = handles
        .into_iter()
        .map(|handle| handle.join().expect("thread join").expect("fresh slot"))
        .collect::<Vec<_>>();
    assert_eq!(leases.len(), 2);
    for lease in leases {
        release_parallel_slot(concurrent.path(), &lease.work_item_id, &lease.lease_id)
            .expect("release fresh slot");
    }
    assert!(list_parallel_slots(concurrent.path()).unwrap().is_empty());
}

#[test]
fn slots_are_isolated_between_repositories() {
    let left = repository();
    let right = repository();
    start_item(left.path(), "WI-SAME", "src/left.rs");
    start_item(right.path(), "WI-SAME", "src/right.rs");
    set_work_item_concurrency_boundary(left.path(), "WI-SAME", boundary("src/left.rs", 1))
        .expect("left boundary");
    set_work_item_concurrency_boundary(right.path(), "WI-SAME", boundary("src/right.rs", 1))
        .expect("right boundary");
    let left_lease = acquire_parallel_slot(left.path(), "WI-SAME").expect("left slot");
    let right_lease = acquire_parallel_slot(right.path(), "WI-SAME").expect("right slot");
    assert_ne!(left_lease.repository_id, right_lease.repository_id);
    release_parallel_slot(left.path(), "WI-SAME", &left_lease.lease_id).expect("left release");
    release_parallel_slot(right.path(), "WI-SAME", &right_lease.lease_id).expect("right release");
}

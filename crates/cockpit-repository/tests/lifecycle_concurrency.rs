//! P2-C fault injection: two concurrent invocations of the same lifecycle
//! transition for the same Work Item. The existing test suite already
//! covers sequential retry-after-failure recovery (recovery_decision.rs) and
//! partial-write rollback (finish_work_item_internal's hand-rolled
//! compensating writes), but nothing exercised two callers racing the same
//! multi-file operation for the same Work Item before this Work Item.

use cockpit_core::Digest;
use cockpit_protocol::ResourceFinalizationContext;
use cockpit_repository::{
    WorkItemStartOptions, attach, checkpoint_work_item, finish_work_item,
    plan_resource_finalization, preflight_work_item, record_verification,
    start_work_item_with_options,
};
use std::sync::{Arc, Barrier};
use std::{fs, process::Command};

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

fn contract_path(root: &std::path::Path, id: &str) -> std::path::PathBuf {
    root.join(".ai/work-items/active")
        .join(format!("{id}.contract.json"))
}

/// Brings a Work Item to the point where `finish_work_item` succeeds:
/// start -> plan finalization -> preflight -> checkpoint -> verification.
fn prepare_finish_ready(root: &std::path::Path, work_item_id: &str) {
    start_work_item_with_options(
        root,
        work_item_id,
        "lifecycle concurrency fault injection",
        "prove finish behavior under a concurrent duplicate call",
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["concurrent calls do not corrupt lifecycle records".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    plan_resource_finalization(
        root,
        work_item_id,
        &ResourceFinalizationContext {
            branch: format!("feature/{work_item_id}"),
            worktree: root.display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: format!("https://github.com/example/ai-cockpit/pull/{work_item_id}"),
        },
    )
    .expect("finalization plan");
    preflight_work_item(root, &contract_path(root, work_item_id)).expect("preflight");
    checkpoint_work_item(root, work_item_id).expect("checkpoint");
    record_verification(
        root,
        work_item_id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.87",
        &Digest::sha256_bytes(b"lifecycle-concurrency-runtime"),
    )
    .expect("verification");
}

/// Two threads call `finish_work_item` for the same Work Item at the same
/// instant (forced by a barrier).
///
/// Before this Work Item, `atomic_write`'s temp file name was derived from
/// `std::process::id()` alone, so two same-process callers writing the same
/// destination path raced for the identical temp file: whichever renamed
/// second found its own temp file already consumed by the other, failing
/// with a misleading filesystem "not found" on the destination path rather
/// than a business-level rejection. That race is fixed by pairing the pid
/// with `NEXT_ATOMIC_WRITE_ID` (the same pattern already used by
/// `write_cap_immutable` and the parallel-slot lease writers). This test
/// pins that fix: every result must be `Ok` or a recognized, well-formed
/// business rejection, never a raw filesystem race artifact, and every JSON
/// record left on disk must parse -- no torn/partial write.
///
/// This test does **not** assert that the Work Item always ends up in
/// `finish_ready`: a second, deeper issue remains (see
/// `docs/work-items/WI-657-lifecycle-concurrency-fault-injection.md`) where
/// a losing thread's own failure-rollback can restore its pre-attempt
/// snapshot over a *different* thread's meanwhile-successful commit. Fixing
/// that requires a real mutual-exclusion boundary around finish/archive/
/// close, which is out of scope for this Work Item; asserting a fixed
/// eventual state here would either hide that gap or make this test flaky
/// pending that larger fix.
#[test]
fn concurrent_finish_calls_never_produce_a_filesystem_race_artifact() {
    let directory = repository();
    let work_item_id = "WI-CONCURRENT-FINISH";
    prepare_finish_ready(directory.path(), work_item_id);

    let barrier = Arc::new(Barrier::new(2));
    let workers = (0..2)
        .map(|_| {
            let root = directory.path().to_path_buf();
            let barrier = Arc::clone(&barrier);
            let id = work_item_id.to_string();
            std::thread::spawn(move || {
                barrier.wait();
                finish_work_item(&root, &id)
            })
        })
        .collect::<Vec<_>>();
    let results = workers
        .into_iter()
        .map(|worker| worker.join().expect("finish worker thread"))
        .collect::<Vec<_>>();

    let outcomes_ok = results.iter().filter(|result| result.is_ok()).count();
    assert!(
        outcomes_ok >= 1,
        "at least one concurrent finish must succeed: {results:?}"
    );
    for result in &results {
        if let Err(error) = result {
            let message = error.to_string();
            assert!(
                !message.contains("No such file or directory") && !message.contains("os error 2"),
                "a losing concurrent finish must fail with a recognized business rejection, \
                 not a raw filesystem race artifact: {message}"
            );
        }
    }

    // Every JSON record left behind must be well-formed, whichever attempt's
    // write ended up on disk last -- proving no torn/partial content.
    for suffix in ["summary.json", "outcome.json"] {
        let path = directory
            .path()
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.{suffix}"));
        let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {suffix}: {error}"));
        let _: serde_json::Value = serde_json::from_slice(&bytes)
            .unwrap_or_else(|error| panic!("parse {suffix}: {error}"));
    }
}

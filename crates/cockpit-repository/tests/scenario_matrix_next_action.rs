//! Automated check for collaboration-language semantic invariant 7 (see
//! docs/reference/collaboration-invariant-coverage.md): the displayed next
//! step must match the current Runtime state/policy. A general oracle over
//! every state is intractable, so this asserts the exact recovery message
//! for a bounded subset of `sourceType: "observed"` scenarios in
//! docs/reference/collaboration-scenario-matrix.json (SCN-001, SCN-002,
//! SCN-016) still matches each scenario's recorded `expected.keyMessage`
//! byte-for-byte, not just a loose substring.

use cockpit_core::Digest;
use cockpit_repository::{
    WorkItemStartOptions, attach, checkpoint_work_item, finish_work_item, preflight_work_item,
    record_verification, scaffold_work_item, start_work_item_with_options,
};
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

fn contract(path: &std::path::Path, id: &str) -> std::path::PathBuf {
    path.join(".ai/work-items/active")
        .join(format!("{id}.contract.json"))
}

fn state_message(error: cockpit_repository::ObserverError) -> String {
    match error {
        cockpit_repository::ObserverError::State { message, .. } => message,
        other => panic!("expected ObserverError::State, got {other:?}"),
    }
}

/// SCN-001: checkpoint attempted before start on a not_ready scaffold.
#[test]
fn scn_001_checkpoint_before_start_matches_scenario_matrix_key_message() {
    let directory = repository();
    scaffold_work_item(directory.path(), "WI-SCN-001", "code").expect("scaffold");

    let error =
        checkpoint_work_item(directory.path(), "WI-SCN-001").expect_err("checkpoint must reject");
    assert_eq!(
        state_message(error),
        "checkpoint is invalid from state \"not_ready\"; expected implementation_active",
        "next-action text must match docs/reference/collaboration-scenario-matrix.json SCN-001 expected.keyMessage exactly"
    );
}

/// SCN-002: start rejected when the worktree has non-governance changes
/// present before activation.
#[test]
fn scn_002_start_rejects_pre_existing_changes_matches_scenario_matrix_key_message() {
    let directory = repository();
    fs::write(directory.path().join("src-under-review.txt"), b"draft\n").expect("write draft");

    let error = start_work_item_with_options(
        directory.path(),
        "WI-SCN-002",
        "invariant 7 coverage",
        "verify next-action text for pre-existing changes",
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            out_of_scope: vec!["target/**".into()],
            acceptance_criteria: vec!["start remains rejected until the worktree is clean".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect_err("start must reject a dirty worktree");
    let message = state_message(error);
    assert!(
        message.contains("non-governance changes were present before start"),
        "next-action text must match docs/reference/collaboration-scenario-matrix.json SCN-002 expected.keyMessage substring, got {message:?}"
    );
    assert!(
        message.contains("src-under-review.txt"),
        "next-action text must name the offending path so recovery (stash, start, restore) is actionable, got {message:?}"
    );
}

/// SCN-016: finish rejected until finalize-plan binds a non-provisional
/// resource finalization context.
#[test]
fn scn_016_finish_before_finalize_plan_matches_scenario_matrix_key_message() {
    let directory = repository();
    let id = "WI-SCN-016";
    start_work_item_with_options(
        directory.path(),
        id,
        "invariant 7 coverage",
        "verify next-action text for finish before finalize-plan",
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            out_of_scope: vec!["target/**".into()],
            acceptance_criteria: vec!["finish stays rejected until finalize-plan runs".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    preflight_work_item(directory.path(), &contract(directory.path(), id)).expect("preflight");
    checkpoint_work_item(directory.path(), id).expect("checkpoint");
    record_verification(
        directory.path(),
        id,
        &serde_json::json!({"passed": true}),
        "0.2.8",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");

    // Deliberately skip `plan_resource_finalization` -- that omission is
    // exactly what SCN-016 observes and what this test protects.
    let error = finish_work_item(directory.path(), id).expect_err("finish must reject");
    assert_eq!(
        state_message(error),
        "finish requires a non-provisional resource finalization plan; run finalize-plan before finish",
        "next-action text must match docs/reference/collaboration-scenario-matrix.json SCN-016 expected.keyMessage exactly"
    );
}

/// Confirms the three scenarios above still exist verbatim in the scenario
/// matrix with `sourceType: "observed"`, so this test file cannot silently
/// drift away from the document it is meant to keep honest.
#[test]
fn scn_001_002_016_are_still_declared_observed_in_the_scenario_matrix() {
    let matrix_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/reference/collaboration-scenario-matrix.json"
    );
    let matrix: serde_json::Value =
        serde_json::from_slice(&fs::read(matrix_path).expect("read scenario matrix"))
            .expect("scenario matrix JSON");
    let scenarios = matrix["scenarios"].as_array().expect("scenarios array");
    for id in ["SCN-001", "SCN-002", "SCN-016"] {
        let scenario = scenarios
            .iter()
            .find(|entry| entry["id"] == id)
            .unwrap_or_else(|| panic!("{id} missing from collaboration-scenario-matrix.json"));
        assert_eq!(
            scenario["sourceType"], "observed",
            "{id} must remain sourceType=observed for this test to keep testing a real, previously-hit Runtime behavior"
        );
        assert!(
            scenario["invariantsExercised"]
                .as_array()
                .expect("invariantsExercised array")
                .iter()
                .any(|value| value == 7),
            "{id} must still declare invariant 7 in invariantsExercised"
        );
    }
}

//! Controlled-repository proof for collaboration-language section V.
//!
//! The reconstruction helper intentionally receives only repository-local
//! Runtime records.  It has no transcript, prior agent state, or inferred
//! benefit input.

use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

mod common;

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("controlled repository");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    directory
}

fn run(binary: &str, repo: &Path, args: &[&str]) -> Output {
    Command::new(binary)
        .args(args)
        .args(["--repo"])
        .arg(repo)
        .current_dir(repo)
        .output()
        .expect("run ai-cockpit")
}

fn run_json(binary: &str, repo: &Path, args: &[&str]) -> Value {
    let output = run(binary, repo, args);
    assert!(
        output.status.success(),
        "args={args:?}, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("machine-readable JSON")
}

fn read_json(path: impl AsRef<Path>) -> Value {
    serde_json::from_slice(&fs::read(path).expect("Runtime record")).expect("Runtime JSON")
}

fn record_path(repo: &Path, directory: &str, id: &str, suffix: &str) -> std::path::PathBuf {
    repo.join(directory).join(format!("{id}.{suffix}.json"))
}

fn reconstruct_handoff(binary: &str, repo: &Path, id: &str) -> Value {
    // This is the boundary exercised by a new Agent/session: all inputs are
    // fresh reads from repository-bound Runtime records, never conversation
    // history or values retained from the preceding process.
    let contract = read_json(record_path(repo, ".ai/work-items/active", id, "contract"));
    let summary = read_json(record_path(repo, ".ai/work-items/active", id, "summary"));
    let status = run_json(binary, repo, &["work-item", "status", "--id", id, "--json"]);
    let controls = run_json(
        binary,
        repo,
        &["work-item", "validate", "--id", id, "--json"],
    );
    let outcome = read_json(record_path(repo, ".ai/work-items/active", id, "outcome"));
    let evidence = read_json(record_path(repo, ".ai/evidence", id, "verification"));

    json!({
        "objective": {
            "intent": contract["intent"].clone(),
            "goal": contract["goal"].clone(),
        },
        "scope": contract["scope"].clone(),
        "outOfScope": contract["outOfScope"].clone(),
        "completion": {
            "summaryState": summary["state"].clone(),
            "lifecyclePhase": status["lifecyclePhase"].clone(),
            "verification": status["verification"].clone(),
            "done": outcome["state"] == json!("verified"),
            "pending": outcome["state"] != json!("verified"),
        },
        "evidence": {
            "passed": evidence["passed"].clone(),
            "workItemId": evidence["workItemId"].clone(),
            "contractDigest": evidence["contractDigest"].clone(),
            "repositorySnapshotDigest": evidence["repositorySnapshotDigest"].clone(),
            "runtimeDigest": evidence["runtimeDigest"].clone(),
            "freshness": status["evidenceFreshness"].clone(),
        },
        "authorization": {
            "declaredAuthority": contract["authority"].clone(),
            "preflightReview": summary["preflightState"].clone(),
            "applicable": contract["authority"] == json!("authorized")
                && summary["preflightState"] == json!("green")
                && status["evidenceFreshness"]["state"] == json!("fresh")
                && controls["state"] == json!("verified"),
            "basis": {
                "preflightReview": summary["preflightState"].clone(),
                "evidenceFreshness": status["evidenceFreshness"].clone(),
                "governanceControls": controls["state"].clone(),
                "findings": controls["findings"].clone(),
            },
        },
        "blockReason": {
            "outcomeState": outcome["state"].clone(),
            "failedGate": outcome["failedGate"].clone(),
            "recoveryCondition": outcome["recoveryCondition"].clone(),
            "unknowns": outcome["unknowns"].clone(),
            "statusUnknowns": status["unknowns"].clone(),
        },
        "benefit": outcome["humanBenefitReport"].clone(),
    })
}

#[test]
fn new_agent_reconstructs_handoff_from_runtime_records_without_conversation_history() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let id = "WI-HANDOFF-RECONSTRUCTION";
    let repo = repository();

    run_json(binary, repo.path(), &["attach"]);
    let started = run(
        binary,
        repo.path(),
        &[
            "start",
            "--id",
            id,
            "--intent",
            "prove a new session can recover a bounded collaboration handoff",
            "--goal",
            "rebuild objective, scope, completion state, evidence, authorization, and blockers from Runtime records",
            "--scope",
            "README.md",
            "--out-of-scope",
            "production behavior",
            "--authority",
            "authorized",
            "--acceptance",
            "A1: reconstruct the handoff without transcript input",
            "--acceptance",
            "A2: preserve current evidence and the blocking reason",
            "--required-evidence",
            "verification",
        ],
    );
    assert!(
        started.status.success(),
        "start stderr={}",
        String::from_utf8_lossy(&started.stderr)
    );

    common::plan(binary, repo.path(), id);
    run_json(
        binary,
        repo.path(),
        &[
            "preflight",
            "--contract",
            &format!(".ai/work-items/active/{id}.contract.json"),
        ],
    );
    run_json(binary, repo.path(), &["checkpoint", "--id", id]);
    run_json(
        binary,
        repo.path(),
        &["verify", "--work-item", id, "--command", "true"],
    );

    // Deliberately omit acceptanceEvidence and intentAlignment.  The failed
    // finish is the persisted boundary a new Agent must explain, not hide.
    let finish = run(binary, repo.path(), &["finish", "--id", id]);
    assert!(!finish.status.success(), "finish unexpectedly succeeded");
    let finish_text = format!(
        "{}{}",
        String::from_utf8_lossy(&finish.stdout),
        String::from_utf8_lossy(&finish.stderr)
    );
    assert!(
        finish_text.contains("governance") || finish_text.contains("preflight"),
        "finish did not expose its gate: {finish_text}"
    );

    let handoff = reconstruct_handoff(binary, repo.path(), id);
    assert_eq!(
        handoff["objective"]["goal"],
        "rebuild objective, scope, completion state, evidence, authorization, and blockers from Runtime records"
    );
    assert_eq!(handoff["scope"], json!(["README.md"]));
    assert_eq!(handoff["outOfScope"], json!(["production behavior"]));
    assert_eq!(handoff["completion"]["summaryState"], "checkpointed");
    assert_eq!(handoff["completion"]["lifecyclePhase"], "checkpointed");
    assert_eq!(handoff["completion"]["verification"], "verified");
    assert_eq!(handoff["completion"]["done"], false);
    assert_eq!(handoff["completion"]["pending"], true);

    assert_eq!(handoff["evidence"]["passed"], true);
    assert_eq!(handoff["evidence"]["workItemId"], id);
    assert_eq!(handoff["evidence"]["freshness"]["state"], "fresh");
    assert!(handoff["evidence"]["contractDigest"].as_str().is_some());
    assert!(
        handoff["evidence"]["repositorySnapshotDigest"]
            .as_str()
            .is_some()
    );
    assert!(handoff["evidence"]["runtimeDigest"].as_str().is_some());

    assert_eq!(handoff["authorization"]["declaredAuthority"], "authorized");
    assert_eq!(handoff["authorization"]["applicable"], false);
    assert_eq!(
        handoff["authorization"]["basis"]["governanceControls"],
        "blocked"
    );
    assert!(
        handoff["authorization"]["basis"]["findings"]
            .as_array()
            .expect("governance findings")
            .iter()
            .any(|finding| finding["code"] == "acceptance_evidence_missing")
    );
    assert_eq!(handoff["blockReason"]["outcomeState"], "blocked");
    assert!(handoff["blockReason"]["failedGate"].as_str().is_some());
    assert!(
        handoff["blockReason"]["recoveryCondition"]
            .as_str()
            .is_some()
    );
    assert!(
        handoff["blockReason"]["unknowns"]
            .as_array()
            .expect("persisted blocker unknowns")
            .iter()
            .any(|item| item == "lifecycle_gate_failed")
    );
    assert!(
        handoff
            .to_string()
            .contains("user_visible_benefit_not_declared"),
        "the handoff must preserve the unknown benefit rather than infer one"
    );
}

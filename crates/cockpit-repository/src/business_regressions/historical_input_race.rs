//! Native private-port regression for captured historical recovery inputs.
//! Historical fixture bytes are synthetic data, never a real human capability.
#![cfg(test)]
use crate::review_authority::pipeline_tests::TestAuthority;
use crate::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_work_item_with_runtime, attach, checkpoint_work_item, finish_work_item_with_runtime,
    historical_finalization_recovery_plan, preflight_work_item_with_runtime,
    record_historical_finalization_recovery, record_resource_finalization,
    record_verification_with_runtime, run_repository_verification, start_work_item_with_options,
    verify_resource_finalization,
};
use cockpit_core::Digest;
use cockpit_protocol::review_authority::HumanReviewOperation;
use cockpit_protocol::{ResourceFinalizationContext, RuntimeContext};
use serde_json::{Value, json};
use std::{fs, process::Command};
const ID: &str = "WI-FINALIZATION-TRANSITION";
fn commit_empty_baseline(path: &std::path::Path) {
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=AI Cockpit test fixture",
                "-c",
                "user.email=ai-cockpit-test@example.invalid",
                "commit",
                "--allow-empty",
                "-qm",
                "test fixture baseline",
            ])
            .current_dir(path)
            .status()
            .unwrap()
            .success()
    );
}

const ID: &str = "WI-FINALIZATION-TRANSITION";

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "test-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"test-runtime"),
    }
}

fn repository_with_worktree(
    authority: &TestAuthority,
    primary: bool,
) -> (tempfile::TempDir, ResourceFinalizationContext, Digest) {
    let directory = tempfile::tempdir().unwrap();
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .unwrap()
            .success()
    );
    commit_empty_baseline(directory.path());
    attach(directory.path()).unwrap();
    start_work_item_with_options(
        directory.path(),
        ID,
        "append finalization transitions",
        "preserve history",
        &[".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["transition chain is linear".into()],
            ..Default::default()
        },
    )
    .unwrap();
    let worktree_path = if primary {
        directory.path().to_string_lossy().to_string()
    } else {
        "/tmp/removed-finalization-transition".into()
    };
    let context = ResourceFinalizationContext {
        branch: "feature/finalization-transition".into(),
        worktree: worktree_path,
        base_branch: "main".into(),
        base_remote: "origin".into(),
        provider: if primary { "local" } else { "github" }.into(),
        pull_request: "https://github.com/example/project/pull/191".into(),
    };
    plan_resource_finalization(directory.path(), ID, &context).unwrap();
    let current = runtime();
    let contract = directory
        .path()
        .join(format!(".ai/work-items/active/{ID}.contract.json"));
    preflight_work_item_with_runtime(directory.path(), &contract, &current).unwrap();
    authority.approve(
        directory.path(),
        ID,
        HumanReviewOperation::PreflightReview,
        &json!({"decision":"confirm_review"}),
    );
    checkpoint_work_item(directory.path(), ID).unwrap();
    let run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "project-command-0".into(),
            program: "true".into(),
            args: vec![],
            scope: vec![".ai/**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: current.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: None,
            timeout_seconds: None,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .unwrap();
    let mut evidence = serde_json::to_value(&run.receipt).unwrap();
    evidence["runtimeVersion"] = current.runtime_version.clone().into();
    evidence["runtimeDigest"] = current.runtime_digest.to_string().into();
    record_verification_with_runtime(
        directory.path(),
        ID,
        &evidence,
        &current,
        &run.final_snapshot,
    )
    .unwrap();
    finish_work_item_with_runtime(directory.path(), ID, &current).unwrap();
    archive_work_item_with_runtime(directory.path(), ID, &current).unwrap();
    let archived = directory
        .path()
        .join(format!(".ai/work-items/archive/{ID}.contract.json"));
    let digest = Digest::sha256_bytes(&fs::read(archived).unwrap());
    (directory, context, digest)
}

fn archived_contract_base_revision(directory: &tempfile::TempDir) -> String {
    let path = directory
        .path()
        .join(format!(".ai/work-items/archive/{ID}.contract.json"));
    let contract: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    contract["baseRevision"].as_str().unwrap().to_owned()
}

fn blocked(
    directory: &tempfile::TempDir,
    repository_id: &str,
    context: &ResourceFinalizationContext,
    contract: &Digest,
) -> Value {
    json!({
        "schemaVersion":1,"receiptId":"blocked-1","operationId":"operation-1",
        "repositoryId":repository_id,
        "workItemId":ID,"runtimeVersion":"test-runtime","runtimeDigest":runtime().runtime_digest,
        "provider":context.provider,"pullRequest":{"number":191,"url":context.pull_request,"headRevision":"head-191","baseBranch":"main","baseRemote":"origin","baseRevision":archived_contract_base_revision(directory)},
        "branch":{"name":context.branch,"remote":"origin","headRevision":"head-191"},
        "worktree":{"worktreeId":"worktree-191","path":context.worktree,"branch":context.branch,"headRevision":"head-191"},
        "before":{"pullRequest":"unmerged","branch":"present","worktree":"clean"},
        "after":{"pullRequest":"unmerged","branch":"present","worktree":"clean"},
        "result":{"disposition":"blocked","failureCodes":["unmerged_pull_request"],"unknownCodes":[]},
        "actor":"human:test","authoritySource":"test","reason":"await merge","timestamp":"2026-08-23T00:00:00Z",
        "contractDigest":contract,"resourceContext":context
    })
}

fn write_input(directory: &tempfile::TempDir, name: &str, value: &Value) -> std::path::PathBuf {
    let inputs = directory.path().join(".ai/test-inputs");
    fs::create_dir_all(&inputs).unwrap();
    let path = inputs.join(name);
    fs::write(&path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
    path
}

#[test]
fn historical_recovery_uses_approved_bytes_after_waiting_for_lifecycle_lock() {
    let authority = TestAuthority::install();
    let (directory, context, contract) = repository_with_worktree(&authority, false);
    let root = directory.path();
    let repository_id = crate::repository_id(root).to_string();
    let legacy_runtime = RuntimeContext {
        runtime_version: "0.2.33".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"legacy-runtime-0.2.33"),
    };
    let mut receipt = blocked(&directory, &repository_id, &context, &contract);
    receipt["runtimeVersion"] = legacy_runtime.runtime_version.clone().into();
    receipt["runtimeDigest"] = legacy_runtime.runtime_digest.to_string().into();
    receipt["pullRequest"]["mergeCommit"] = "merge-191".into();
    receipt["before"]["pullRequest"] = "merged".into();
    receipt["after"] = json!({"pullRequest":"merged","branch":"deleted","worktree":"removed"});
    receipt["result"] = json!({"disposition":"deleted","failureCodes":[],"unknownCodes":[]});
    let input = write_input(&directory, "legacy-normal-pr.json", &receipt);
    record_resource_finalization(root, ID, &input, &legacy_runtime).unwrap();
    let predecessor_path = root.join(format!(".ai/decisions/{ID}.finalize.json"));
    let predecessor = fs::read(&predecessor_path).unwrap();
    let current = runtime();
    let plan = historical_finalization_recovery_plan(root, ID, &current, None).unwrap();
    assert_eq!(plan["historicalKind"], "legacy_runtime");
    let mut approved = plan["suggestedRecovery"].clone();
    approved["schemaVersion"] = 1.into();
    approved["recoveryId"] = "immutable-approved-A".into();
    approved["kind"] = "historical_finalization_recovery".into();
    approved["actor"] = "human:approved-A".into();
    approved["authoritySource"] = "synthetic-exact-test-port".into();
    approved["reason"] = "approved immutable input A".into();
    approved["evidenceRefs"] = json!([format!(".ai/decisions/{ID}.finalize.json")]);
    approved["decidedAt"] = "2026-08-23T00:22:00Z".into();
    let input = write_input(&directory, "approved-recovery.json", &approved);
    let approved_bytes = fs::read(&input).unwrap();
    authority.approve(
        root,
        ID,
        HumanReviewOperation::HistoricalFinalizationRecovery,
        &json!({"inputRawDigest":Digest::sha256_bytes(&approved_bytes),"runtime":current}),
    );
    let mut substituted = approved.clone();
    substituted["actor"] = "human:unapproved-B".into();
    substituted["reason"] = "unapproved input B after capture check".into();
    let writer_root = root.to_path_buf();
    let writer_input = input.clone();
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    let (checked_tx, checked_rx) = std::sync::mpsc::channel();
    let writer = std::thread::spawn(move || {
        let guard = crate::acquire_lifecycle_lock(&writer_root, ID).unwrap();
        ready_tx.send(()).unwrap();
        if checked_rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .is_ok()
        {
            fs::write(
                writer_input,
                serde_json::to_vec_pretty(&substituted).unwrap(),
            )
            .unwrap();
        }
        drop(guard);
    });
    ready_rx
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    authority.on_next_stage("historical_recovery_input_checked", move || {
        checked_tx.send(()).unwrap()
    });
    let result = record_historical_finalization_recovery(root, ID, &input, &current);
    writer.join().unwrap();
    assert_eq!(
        result.expect("native classification after the cooperative lock wait")["state"],
        "recorded"
    );
    let published: Value = serde_json::from_slice(
        &fs::read(root.join(format!(".ai/decisions/{ID}.finalize-recovery.json"))).unwrap(),
    )
    .unwrap();
    assert_eq!(
        published["actor"], approved["actor"],
        "the approved raw A digest must never execute substituted B"
    );
    assert_eq!(published["reason"], approved["reason"]);
    assert_ne!(
        fs::read(&input).unwrap(),
        approved_bytes,
        "the real writer must have changed the external input"
    );
    crate::review_authority::require_current_human_effect(
        root,
        ID,
        HumanReviewOperation::HistoricalFinalizationRecovery,
        &published,
    )
    .unwrap();
    assert_eq!(
        verify_resource_finalization(root, ID, &current).unwrap()["state"],
        "verified"
    );
    assert_eq!(fs::read(predecessor_path).unwrap(), predecessor);
    assert_eq!(
        authority.execution_count(),
        1,
        "one protected execution of the exact approved input"
    );
}

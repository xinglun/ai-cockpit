use cockpit_core::Digest;
use cockpit_protocol::{HumanDecision, ResourceFinalizationContext, RuntimeContext};
use cockpit_repository::{
    ArchivedContractRevalidationRequest, RepositoryVerificationPolicy,
    RepositoryVerificationRequest, WorkItemStartOptions, archive_work_item, attach,
    checkpoint_work_item, close_work_item_with_structured_decision_and_runtime, finish_work_item,
    outcome_v2_with_runtime, plan_resource_finalization, preflight_work_item,
    record_resource_finalization, record_verification_with_runtime,
    revalidate_archived_work_item_with_runtime, run_repository_verification,
    start_work_item_with_options, status,
};
use serde_json::json;
use std::fs;
use std::process::Command;

const ID: &str = "WI-ARCHIVED-AMENDMENT";
const SUCCESSOR: &str = "WI-ARCHIVED-AMENDMENT-REVALIDATION";

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "0.2.75".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"wi583-runtime"),
    }
}

fn commit(path: &std::path::Path, message: &str) {
    assert!(
        Command::new("git")
            .args(["add", "."])
            .current_dir(path)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.email=test@example.invalid",
                "-c",
                "user.name=test",
                "commit",
                "-qm",
                message
            ])
            .current_dir(path)
            .status()
            .unwrap()
            .success()
    );
}

fn archived_amended_repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .unwrap()
            .success()
    );
    attach(directory.path()).unwrap();
    start_work_item_with_options(
        directory.path(),
        ID,
        "preserve an amended archived Work Item",
        "revalidate an archived Contract without rewriting evidence",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["historical bytes remain auditable".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .unwrap();
    plan_resource_finalization(
        directory.path(),
        ID,
        &ResourceFinalizationContext {
            branch: "feature/archived-amendment".into(),
            worktree: directory.path().display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "local".into(),
            pull_request: "local://archived-amendment".into(),
        },
    )
    .unwrap();
    let contract_path = directory
        .path()
        .join(format!(".ai/work-items/active/{ID}.contract.json"));
    preflight_work_item(directory.path(), &contract_path).unwrap();
    checkpoint_work_item(directory.path(), ID).unwrap();
    let run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "archived-amendment-check".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec!["src/**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: runtime().runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .unwrap();
    let receipt = serde_json::to_value(&run.receipt).unwrap();
    record_verification_with_runtime(
        directory.path(),
        ID,
        &receipt,
        &runtime(),
        &run.final_snapshot,
    )
    .unwrap();
    finish_work_item(directory.path(), ID).unwrap();
    archive_work_item(directory.path(), ID).unwrap();

    // Record the predecessor's historical shared-worktree finalization before
    // the reviewed Contract amendment.  The revalidation path must retain and
    // re-bind this old receipt instead of asking the caller to rewrite it.
    let predecessor_contract = directory
        .path()
        .join(format!(".ai/work-items/archive/{ID}.contract.json"));
    let predecessor_digest = Digest::sha256_bytes(&fs::read(&predecessor_contract).unwrap());
    let predecessor_context = ResourceFinalizationContext {
        branch: "feature/archived-amendment".into(),
        worktree: directory.path().display().to_string(),
        base_branch: "main".into(),
        base_remote: "origin".into(),
        provider: "local".into(),
        pull_request: "local://archived-amendment".into(),
    };
    let repository_id = status(directory.path()).unwrap().repository_id;
    let predecessor_receipt = json!({
        "schemaVersion": 1,
        "receiptId": "receipt-archived-amendment",
        "operationId": "operation-archived-amendment",
        "repositoryId": repository_id,
        "workItemId": ID,
        "runtimeVersion": runtime().runtime_version,
        "runtimeDigest": runtime().runtime_digest,
        "provider": "local",
        "pullRequest": {
            "number": 1,
            "url": predecessor_context.pull_request,
            "headRevision": "head-archived-amendment",
            "baseBranch": "main",
            "baseRemote": "origin",
            "baseRevision": "unborn",
            "mergeCommit": "merge-archived-amendment"
        },
        "branch": {
            "name": predecessor_context.branch,
            "remote": "origin",
            "headRevision": "head-archived-amendment"
        },
        "worktree": {
            "worktreeId": "wt-archived-amendment",
            "path": predecessor_context.worktree,
            "branch": predecessor_context.branch,
            "headRevision": "head-archived-amendment"
        },
        "before": {"pullRequest": "merged", "branch": "present", "worktree": "clean"},
        "after": {"pullRequest": "merged", "branch": "present", "worktree": "clean"},
        "result": {"disposition": "retained", "failureCodes": [], "unknownCodes": []},
        "actor": "human:test",
        "authoritySource": "test-policy",
        "reason": "historical shared-worktree resources were retained",
        "timestamp": "2026-09-05T00:00:00Z",
        "contractDigest": predecessor_digest,
        "resourceContext": predecessor_context
    });
    let predecessor_input = tempfile::NamedTempFile::new().unwrap();
    fs::write(
        predecessor_input.path(),
        serde_json::to_vec_pretty(&predecessor_receipt).unwrap(),
    )
    .unwrap();
    record_resource_finalization(directory.path(), ID, predecessor_input.path(), &runtime())
        .unwrap();

    // This models a reviewed repository repair: the current archived Contract
    // and its manifest are updated together, while the original verification
    // evidence bytes remain untouched and still carry the old Contract digest.
    let archive = directory.path().join(".ai/work-items/archive");
    let contract_path = archive.join(format!("{ID}.contract.json"));
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).unwrap()).unwrap();
    contract["goal"] = json!("revalidate the reviewed archived Contract amendment");
    let contract_bytes = serde_json::to_vec_pretty(&contract).unwrap();
    fs::write(&contract_path, &contract_bytes).unwrap();
    let manifest_path = archive.join(format!("{ID}.archive.json"));
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["files"]["contractDigest"] = json!(Digest::sha256_bytes(&contract_bytes).to_string());
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    commit(directory.path(), "model reviewed archived Contract repair");
    directory
}

fn retained_successor_finalization(
    root: &std::path::Path,
    work_item_id: &str,
    runtime: &RuntimeContext,
) {
    let contract_path = root.join(format!(
        ".ai/work-items/archive/{work_item_id}.contract.json"
    ));
    let contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).unwrap()).unwrap();
    let context = contract["resourceContext"].clone();
    let repository_id = status(root).unwrap().repository_id;
    let receipt = json!({
        "schemaVersion": 1,
        "receiptId": format!("receipt-{work_item_id}"),
        "operationId": format!("operation-{work_item_id}"),
        "repositoryId": repository_id,
        "workItemId": work_item_id,
        "runtimeVersion": runtime.runtime_version,
        "runtimeDigest": runtime.runtime_digest,
        "provider": context["provider"],
        "pullRequest": {
            "number": 1,
            "url": context["pullRequest"],
            "headRevision": "head-successor",
            "baseBranch": context["baseBranch"],
            "baseRemote": context["baseRemote"],
            "baseRevision": contract["baseRevision"],
            "mergeCommit": "merge-successor"
        },
        "branch": {"name": context["branch"], "remote": "origin", "headRevision": "head-successor"},
        "worktree": {"worktreeId": format!("wt-{work_item_id}"), "path": context["worktree"], "branch": context["branch"], "headRevision": "head-successor"},
        "before": {"pullRequest": "merged", "branch": "present", "worktree": "clean"},
        "after": {"pullRequest": "merged", "branch": "present", "worktree": "clean"},
        "result": {"disposition": "retained", "failureCodes": [], "unknownCodes": []},
        "actor": "human:test",
        "authoritySource": "test-policy",
        "reason": "successor shared-worktree resources are retained",
        "timestamp": "2026-09-05T00:00:00Z",
        "contractDigest": Digest::sha256_bytes(&fs::read(&contract_path).unwrap()),
        "resourceContext": context
    });
    let input = tempfile::NamedTempFile::new().unwrap();
    fs::write(input.path(), serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
    record_resource_finalization(root, work_item_id, input.path(), runtime).unwrap();
}

#[test]
fn archived_contract_revalidation_appends_current_binding_and_scaffolds_successor() {
    let directory = archived_amended_repository();
    let evidence_before = fs::read(
        directory
            .path()
            .join(format!(".ai/evidence/{ID}.verification.json")),
    )
    .unwrap();
    let archive_before = fs::read(
        directory
            .path()
            .join(format!(".ai/work-items/archive/{ID}.archive.json")),
    )
    .unwrap();
    let result =
        revalidate_archived_work_item_with_runtime(
            directory.path(),
            ID,
            &ArchivedContractRevalidationRequest {
                successor_work_item_id: SUCCESSOR.into(),
                reason: "reviewed Contract repair requires fresh successor verification".into(),
                actor: "human:repository-owner".into(),
                authority_source: "explicit-user-authorized-revalidation".into(),
                resume_condition:
                    "run fresh verification on the successor and then close both lineage records"
                        .into(),
                evidence_refs: vec![format!(".ai/evidence/{ID}.verification.json")],
                policy_refs: vec!["docs/reference/agent-workflow.md".into()],
            },
            &runtime(),
        )
        .unwrap();
    assert_eq!(result["state"], "recorded");
    assert_eq!(result["successorWorkItemId"], SUCCESSOR);
    assert_eq!(
        result["successorBindingMode"],
        "contract_amendment_revalidation"
    );
    assert!(
        directory
            .path()
            .join(format!(".ai/work-items/active/{SUCCESSOR}.contract.json"))
            .is_file()
    );
    assert_eq!(
        evidence_before,
        fs::read(
            directory
                .path()
                .join(format!(".ai/evidence/{ID}.verification.json"))
        )
        .unwrap()
    );
    assert_eq!(
        archive_before,
        fs::read(
            directory
                .path()
                .join(format!(".ai/work-items/archive/{ID}.archive.json"))
        )
        .unwrap()
    );
    let successor: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(format!(".ai/work-items/active/{SUCCESSOR}.contract.json")),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(successor["predecessorWorkItemId"], ID);
    assert_eq!(
        successor["predecessorContractDigest"],
        result["currentContractDigest"]
    );
    assert!(
        status(directory.path())
            .unwrap()
            .readiness
            .unclosed_archived_work_items
            .contains(&ID.to_owned())
    );
    let outcome = outcome_v2_with_runtime(directory.path(), ID, &runtime()).unwrap();
    assert_eq!(outcome.state, cockpit_protocol::OutcomeState::NotReady);
    assert_eq!(
        outcome.decision_state,
        Some(cockpit_core::DecisionState::Yellow)
    );
    assert_eq!(
        outcome.historical_status.as_deref(),
        Some("contract_amendment_revalidation")
    );
    let repeated =
        revalidate_archived_work_item_with_runtime(
            directory.path(),
            ID,
            &ArchivedContractRevalidationRequest {
                successor_work_item_id: SUCCESSOR.into(),
                reason: "reviewed Contract repair requires fresh successor verification".into(),
                actor: "human:repository-owner".into(),
                authority_source: "explicit-user-authorized-revalidation".into(),
                resume_condition:
                    "run fresh verification on the successor and then close both lineage records"
                        .into(),
                evidence_refs: vec![format!(".ai/evidence/{ID}.verification.json")],
                policy_refs: vec!["docs/reference/agent-workflow.md".into()],
            },
            &runtime(),
        )
        .unwrap();
    assert_eq!(repeated["state"], "idempotent");
    let close_error = close_work_item_with_structured_decision_and_runtime(
        directory.path(),
        ID,
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:repository-owner".into(),
            authority_source: "explicit-user-authorization".into(),
            reason: "successor is not terminal yet".into(),
            evidence_refs: Vec::new(),
            policy_refs: Vec::new(),
            decided_at: "2026-09-05T00:00:00Z".into(),
            resume_condition: Some("complete successor lifecycle".into()),
        },
        &runtime(),
    )
    .expect_err("predecessor must remain blocked until successor is terminal");
    assert!(
        close_error
            .to_string()
            .contains("valid verification evidence")
    );
}

#[test]
fn archived_contract_revalidation_rejects_tampered_historical_evidence_without_writes() {
    let directory = archived_amended_repository();
    let evidence_path = directory
        .path()
        .join(format!(".ai/evidence/{ID}.verification.json"));
    let original = fs::read(&evidence_path).unwrap();
    let mut evidence: serde_json::Value = serde_json::from_slice(&original).unwrap();
    evidence["passed"] = json!(false);
    fs::write(
        &evidence_path,
        serde_json::to_vec_pretty(&evidence).unwrap(),
    )
    .unwrap();
    let error = revalidate_archived_work_item_with_runtime(
        directory.path(),
        ID,
        &ArchivedContractRevalidationRequest {
            successor_work_item_id: SUCCESSOR.into(),
            reason: "reject tampered evidence".into(),
            actor: "human:repository-owner".into(),
            authority_source: "explicit-user-authorized-revalidation".into(),
            resume_condition: "repair the evidence and retry".into(),
            evidence_refs: Vec::new(),
            policy_refs: Vec::new(),
        },
        &runtime(),
    )
    .expect_err("tampered evidence must fail closed");
    assert!(
        error
            .to_string()
            .contains("historical verification evidence")
    );
    assert!(
        !directory
            .path()
            .join(format!(".ai/decisions/{ID}.recovery.json"))
            .exists()
    );
    assert!(
        !directory
            .path()
            .join(format!(".ai/work-items/active/{SUCCESSOR}.contract.json"))
            .exists()
    );
}

#[test]
fn successor_terminal_lifecycle_allows_predecessor_historical_close() {
    let directory = archived_amended_repository();
    let runtime = runtime();
    revalidate_archived_work_item_with_runtime(
        directory.path(),
        ID,
        &ArchivedContractRevalidationRequest {
            successor_work_item_id: SUCCESSOR.into(),
            reason: "reviewed Contract repair requires fresh successor verification".into(),
            actor: "human:repository-owner".into(),
            authority_source: "explicit-user-authorized-revalidation".into(),
            resume_condition: "complete successor lifecycle before predecessor close".into(),
            evidence_refs: vec![format!(".ai/evidence/{ID}.verification.json")],
            policy_refs: vec!["docs/reference/agent-workflow.md".into()],
        },
        &runtime,
    )
    .unwrap();
    start_work_item_with_options(
        directory.path(),
        SUCCESSOR,
        "revalidate the amended archived Contract",
        "produce current evidence without rewriting historical evidence",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["current Contract is freshly verified".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .unwrap();
    let context = ResourceFinalizationContext {
        branch: "feature/archived-amendment-successor".into(),
        worktree: directory.path().display().to_string(),
        base_branch: "main".into(),
        base_remote: "origin".into(),
        provider: "local".into(),
        pull_request: "local://archived-amendment-successor".into(),
    };
    plan_resource_finalization(directory.path(), SUCCESSOR, &context).unwrap();
    let active_contract = directory
        .path()
        .join(format!(".ai/work-items/active/{SUCCESSOR}.contract.json"));
    preflight_work_item(directory.path(), &active_contract).unwrap();
    checkpoint_work_item(directory.path(), SUCCESSOR).unwrap();
    let run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "successor-check".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec!["src/**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .unwrap();
    record_verification_with_runtime(
        directory.path(),
        SUCCESSOR,
        &serde_json::to_value(&run.receipt).unwrap(),
        &runtime,
        &run.final_snapshot,
    )
    .unwrap();
    finish_work_item(directory.path(), SUCCESSOR).unwrap();
    archive_work_item(directory.path(), SUCCESSOR).unwrap();
    retained_successor_finalization(directory.path(), SUCCESSOR, &runtime);
    close_work_item_with_structured_decision_and_runtime(
        directory.path(),
        SUCCESSOR,
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:repository-owner".into(),
            authority_source: "explicit-user-authorized-revalidation".into(),
            reason: "successor evidence is current and complete".into(),
            evidence_refs: vec![format!(".ai/evidence/{SUCCESSOR}.verification.json")],
            policy_refs: vec!["docs/reference/agent-workflow.md".into()],
            decided_at: "2026-09-05T00:00:00Z".into(),
            resume_condition: None,
        },
        &runtime,
    )
    .unwrap();
    close_work_item_with_structured_decision_and_runtime(
        directory.path(),
        ID,
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:repository-owner".into(),
            authority_source: "explicit-user-authorized-revalidation".into(),
            reason: "successor revalidated the amended Contract".into(),
            evidence_refs: vec![format!(".ai/evidence/{SUCCESSOR}.verification.json")],
            policy_refs: vec!["docs/reference/agent-workflow.md".into()],
            decided_at: "2026-09-05T00:00:00Z".into(),
            resume_condition: None,
        },
        &runtime,
    )
    .unwrap();
    assert_eq!(
        status(directory.path())
            .unwrap()
            .readiness
            .unclosed_archived_work_items,
        Vec::<String>::new()
    );
    let decision: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(format!(".ai/decisions/{ID}.close.json")),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        decision["historicalRevalidation"]["state"],
        "current_successor_revalidated"
    );
    assert_eq!(
        decision["historicalRevalidation"]["successorWorkItemId"],
        SUCCESSOR
    );
}

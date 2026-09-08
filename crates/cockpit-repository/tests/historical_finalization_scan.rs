use cockpit_core::Digest;
use cockpit_protocol::{
    HumanDecision, ResourceFinalizationContext, ResourceFinalizationTransitionReceipt,
    RuntimeContext,
};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_work_item_with_runtime, attach, checkpoint_work_item,
    close_work_item_with_structured_decision_and_runtime, finish_work_item_with_runtime,
    plan_resource_finalization, preflight_work_item_with_runtime, record_resource_finalization,
    record_verification_with_runtime, run_repository_verification, start_work_item_with_options,
    status_with_runtime, verify_resource_finalization,
};
use serde_json::{Value, json};
use std::{collections::HashMap, fs, path::Path, process::Command};

fn legacy_runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "legacy-test-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"legacy-test-runtime"),
    }
}

fn current_runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "current-test-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"current-test-runtime"),
    }
}

fn write_input(root: &Path, name: &str, value: &Value) -> std::path::PathBuf {
    let path = root.join(name);
    fs::write(&path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
    path
}

fn blocked_receipt(
    work_item_id: &str,
    repository_id: &str,
    context: &ResourceFinalizationContext,
    contract_digest: &Digest,
) -> Value {
    json!({
        "schemaVersion":1,
        "receiptId": format!("{work_item_id}-receipt-1"),
        "operationId": format!("{work_item_id}-operation-1"),
        "repositoryId": repository_id,
        "workItemId": work_item_id,
        "runtimeVersion": legacy_runtime().runtime_version,
        "runtimeDigest": legacy_runtime().runtime_digest,
        "provider": context.provider,
        "pullRequest": {"number":1,"url":context.pull_request,"headRevision":"head-1","baseBranch":"main","baseRemote":"origin","baseRevision":"unborn"},
        "branch": {"name":context.branch,"remote":"origin","headRevision":"head-1"},
        "worktree": {"worktreeId":"worktree-1","path":context.worktree,"branch":context.branch,"headRevision":"head-1"},
        "before": {"pullRequest":"unmerged","branch":"present","worktree":"clean"},
        "after": {"pullRequest":"unmerged","branch":"present","worktree":"clean"},
        "result": {"disposition":"blocked","failureCodes":["unmerged_pull_request"],"unknownCodes":[]},
        "actor":"human:test","authoritySource":"test","reason":"await merge","timestamp":"2026-08-23T00:00:00Z",
        "contractDigest": contract_digest,
        "resourceContext": context
    })
}

fn transition_receipt(previous: &Value, sequence: u64, deleted: bool) -> Value {
    let previous_receipt: cockpit_protocol::ResourceFinalizationReceipt =
        serde_json::from_value(previous.clone()).unwrap();
    let mut next = previous_receipt.clone();
    next.receipt_id = format!("{}-receipt-{}", previous_receipt.work_item_id, sequence + 1);
    next.operation_id = format!(
        "{}-operation-{}",
        previous_receipt.work_item_id,
        sequence + 1
    );
    next.pull_request.merge_commit = Some("merge-1".into());
    next.before = previous_receipt.after.clone();
    next.after.pull_request = cockpit_protocol::ResourceFinalizationPullRequestState::Merged;
    next.result.disposition = cockpit_protocol::ResourceFinalizationDisposition::Retained;
    next.result.failure_codes.clear();
    if deleted {
        next.after.branch = cockpit_protocol::ResourceFinalizationBranchState::Deleted;
        next.after.worktree = cockpit_protocol::ResourceFinalizationWorktreeState::Removed;
        next.result.disposition = cockpit_protocol::ResourceFinalizationDisposition::Deleted;
    }
    serde_json::to_value(ResourceFinalizationTransitionReceipt {
        schema_version: 1,
        transition_id: format!("{}-transition-{sequence}", previous_receipt.work_item_id),
        sequence,
        predecessor_receipt_digest: cockpit_protocol::digest_json(previous).unwrap(),
        governance_append_revision: None,
        receipt: next,
    })
    .unwrap()
}

/// Runs a legacy Work Item through the normal lifecycle far enough to record
/// a canonical `.ai/decisions/{id}.finalize.json`, sharing one repository
/// across calls so `historical_finalization_inventory` sees multiple
/// independent legacy receipts side by side.
fn build_legacy_work_item(
    directory: &tempfile::TempDir,
    id: &str,
    first_in_repository: bool,
) -> (ResourceFinalizationContext, Digest) {
    if first_in_repository {
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .current_dir(directory.path())
                .status()
                .unwrap()
                .success()
        );
        attach(directory.path()).unwrap();
    }
    start_work_item_with_options(
        directory.path(),
        id,
        "record a legacy finalization receipt",
        "preserve history",
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["transition chain is linear".into()],
            ..Default::default()
        },
    )
    .unwrap();
    let context = ResourceFinalizationContext {
        branch: format!("feature/{id}"),
        worktree: format!("/tmp/removed-{id}"),
        base_branch: "main".into(),
        base_remote: "origin".into(),
        provider: "github".into(),
        pull_request: format!("https://github.com/example/project/pull/{id}"),
    };
    plan_resource_finalization(directory.path(), id, &context).unwrap();
    let current = current_runtime();
    let contract = directory
        .path()
        .join(format!(".ai/work-items/active/{id}.contract.json"));
    preflight_work_item_with_runtime(directory.path(), &contract, &current).unwrap();
    checkpoint_work_item(directory.path(), id).unwrap();
    let run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "project-command-0".into(),
            program: "true".into(),
            args: vec![],
            scope: vec!["**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: current.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .unwrap();
    let mut evidence = serde_json::to_value(&run.receipt).unwrap();
    evidence["runtimeVersion"] = current.runtime_version.clone().into();
    evidence["runtimeDigest"] = current.runtime_digest.to_string().into();
    record_verification_with_runtime(
        directory.path(),
        id,
        &evidence,
        &current,
        &run.final_snapshot,
    )
    .unwrap();
    finish_work_item_with_runtime(directory.path(), id, &current).unwrap();
    archive_work_item_with_runtime(directory.path(), id, &current).unwrap();
    let archived_contract = directory
        .path()
        .join(format!(".ai/work-items/archive/{id}.contract.json"));
    let contract_digest = Digest::sha256_bytes(&fs::read(archived_contract).unwrap());
    (context, contract_digest)
}

/// Pins the fix for the O(n^2) `.ai/decisions` re-scan (WI-648/WI-649):
/// `historical_finalization_inventory` must resolve each legacy receipt using
/// only its own transition candidates, matching what the still-unindexed
/// `resolve_resource_finalization_head` (exercised here via
/// `verify_resource_finalization`) independently computes for the same
/// receipts.
#[test]
fn historical_finalization_inventory_resolves_independent_legacy_receipts_without_cross_contamination()
 {
    let directory = tempfile::tempdir().unwrap();
    let (context_a, contract_digest_a) =
        build_legacy_work_item(&directory, "WI-LEGACY-ALPHA", true);
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();

    // ALPHA gets a two-transition chain (blocked -> observed -> deleted, so it
    // both exercises chain-walking through non-empty candidates and reaches a
    // disposition close will accept, matching the existing
    // wi190_topology_appends_two_transitions_and_resolves_deleted_head
    // pattern) and is closed so a second Work Item can start in this repository.
    let alpha_blocked = blocked_receipt(
        "WI-LEGACY-ALPHA",
        &repository_id,
        &context_a,
        &contract_digest_a,
    );
    let alpha_input = write_input(directory.path(), "alpha-blocked.json", &alpha_blocked);
    record_resource_finalization(
        directory.path(),
        "WI-LEGACY-ALPHA",
        &alpha_input,
        &legacy_runtime(),
    )
    .unwrap();
    fs::remove_file(&alpha_input).unwrap();
    let alpha_observed = transition_receipt(&alpha_blocked, 1, false);
    let alpha_observed_input =
        write_input(directory.path(), "alpha-observed.json", &alpha_observed);
    record_resource_finalization(
        directory.path(),
        "WI-LEGACY-ALPHA",
        &alpha_observed_input,
        &legacy_runtime(),
    )
    .unwrap();
    fs::remove_file(&alpha_observed_input).unwrap();
    let alpha_deleted = transition_receipt(&alpha_observed["receipt"], 2, true);
    let alpha_deleted_input = write_input(directory.path(), "alpha-deleted.json", &alpha_deleted);
    record_resource_finalization(
        directory.path(),
        "WI-LEGACY-ALPHA",
        &alpha_deleted_input,
        &legacy_runtime(),
    )
    .unwrap();
    fs::remove_file(&alpha_deleted_input).unwrap();
    let alpha_verified =
        verify_resource_finalization(directory.path(), "WI-LEGACY-ALPHA", &legacy_runtime())
            .unwrap();
    assert_eq!(alpha_verified["sequence"], 2);
    close_work_item_with_structured_decision_and_runtime(
        directory.path(),
        "WI-LEGACY-ALPHA",
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:test".into(),
            authority_source: "test".into(),
            reason: "verified transition head".into(),
            evidence_refs: vec![],
            policy_refs: vec![],
            decided_at: "2026-08-23T00:10:00Z".into(),
            resume_condition: None,
        },
        &legacy_runtime(),
    )
    .unwrap();

    // BETA starts only after ALPHA is closed; it gets a canonical receipt
    // only (resolves at sequence 0), so its transition candidate list is
    // empty while ALPHA's is not.
    let (context_b, contract_digest_b) =
        build_legacy_work_item(&directory, "WI-LEGACY-BETA", false);
    let beta_blocked = blocked_receipt(
        "WI-LEGACY-BETA",
        &repository_id,
        &context_b,
        &contract_digest_b,
    );
    let beta_input = write_input(directory.path(), "beta-blocked.json", &beta_blocked);
    record_resource_finalization(
        directory.path(),
        "WI-LEGACY-BETA",
        &beta_input,
        &legacy_runtime(),
    )
    .unwrap();
    fs::remove_file(&beta_input).unwrap();
    let beta_verified =
        verify_resource_finalization(directory.path(), "WI-LEGACY-BETA", &legacy_runtime())
            .unwrap();
    assert_eq!(beta_verified["sequence"], 0);

    let current = current_runtime();
    let status = status_with_runtime(directory.path(), Some(&current)).unwrap();
    let by_id: HashMap<_, _> = status
        .readiness
        .historical_finalization
        .iter()
        .map(|item| (item.work_item_id.clone(), item))
        .collect();

    let alpha_item = *by_id.get("WI-LEGACY-ALPHA").expect("alpha present");
    let beta_item = *by_id.get("WI-LEGACY-BETA").expect("beta present");
    assert_eq!(
        alpha_item.sequence, 2,
        "alpha must resolve its own two-step transition chain"
    );
    assert_eq!(
        beta_item.sequence, 0,
        "beta must not pick up alpha's transition candidates"
    );
    assert_eq!(
        alpha_item
            .predecessor_digest
            .as_ref()
            .map(Digest::to_string),
        alpha_verified["headDigest"].as_str().map(str::to_owned)
    );
    assert_eq!(
        beta_item.predecessor_digest.as_ref().map(Digest::to_string),
        beta_verified["headDigest"].as_str().map(str::to_owned)
    );
}

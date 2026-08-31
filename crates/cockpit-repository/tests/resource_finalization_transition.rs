use cockpit_core::Digest;
use cockpit_protocol::{
    HumanDecision, ResourceFinalizationContext, ResourceFinalizationTransitionReceipt,
    RuntimeContext,
};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_work_item_with_runtime, attach, checkpoint_work_item,
    close_work_item_with_structured_decision_and_runtime, finish_work_item_with_runtime,
    plan_resource_finalization, preflight_work_item_with_runtime,
    record_historical_finalization_recovery, record_resource_finalization,
    record_verification_with_runtime, run_repository_verification, start_work_item_with_options,
    verify_resource_finalization,
};
use serde_json::{Value, json};
use std::{fs, process::Command};

const ID: &str = "WI-FINALIZATION-TRANSITION";

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "test-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"test-runtime"),
    }
}

fn repository() -> (tempfile::TempDir, ResourceFinalizationContext, Digest) {
    repository_with_worktree(false)
}

fn primary_repository() -> (tempfile::TempDir, ResourceFinalizationContext, Digest) {
    repository_with_worktree(true)
}

fn repository_with_worktree(
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
    attach(directory.path()).unwrap();
    start_work_item_with_options(
        directory.path(),
        ID,
        "append finalization transitions",
        "preserve history",
        &["**".into()],
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
        provider: "github".into(),
        pull_request: "https://github.com/example/project/pull/191".into(),
    };
    plan_resource_finalization(directory.path(), ID, &context).unwrap();
    let current = runtime();
    let contract = directory
        .path()
        .join(format!(".ai/work-items/active/{ID}.contract.json"));
    preflight_work_item_with_runtime(directory.path(), &contract, &current).unwrap();
    checkpoint_work_item(directory.path(), ID).unwrap();
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

fn blocked(repository_id: &str, context: &ResourceFinalizationContext, contract: &Digest) -> Value {
    json!({
        "schemaVersion":1,"receiptId":"blocked-1","operationId":"operation-1",
        "repositoryId":repository_id,
        "workItemId":ID,"runtimeVersion":"test-runtime","runtimeDigest":runtime().runtime_digest,
        "provider":"github","pullRequest":{"number":191,"url":context.pull_request,"headRevision":"head-191","baseBranch":"main","baseRemote":"origin","baseRevision":"unborn"},
        "branch":{"name":context.branch,"remote":"origin","headRevision":"head-191"},
        "worktree":{"worktreeId":"worktree-191","path":context.worktree,"branch":context.branch,"headRevision":"head-191"},
        "before":{"pullRequest":"unmerged","branch":"present","worktree":"clean"},
        "after":{"pullRequest":"unmerged","branch":"present","worktree":"clean"},
        "result":{"disposition":"blocked","failureCodes":["unmerged_pull_request"],"unknownCodes":[]},
        "actor":"human:test","authoritySource":"test","reason":"await merge","timestamp":"2026-08-23T00:00:00Z",
        "contractDigest":contract,"resourceContext":context
    })
}

fn transition(previous: &Value, sequence: u64, deleted: bool) -> Value {
    let previous_receipt: cockpit_protocol::ResourceFinalizationReceipt =
        serde_json::from_value(previous.clone()).unwrap();
    let mut next = previous_receipt.clone();
    next.receipt_id = format!("receipt-{}", sequence + 1);
    next.operation_id = format!("operation-{}", sequence + 1);
    next.pull_request.merge_commit = Some("merge-191".into());
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
        transition_id: format!("transition-{sequence}"),
        sequence,
        predecessor_receipt_digest: cockpit_protocol::digest_json(previous).unwrap(),
        governance_append_revision: None,
        receipt: next,
    })
    .unwrap()
}

fn retained_root(previous: &Value) -> Value {
    let mut receipt = previous.clone();
    receipt["receiptId"] = "retained-root".into();
    receipt["operationId"] = "retained-root-operation".into();
    receipt["pullRequest"]["mergeCommit"] = "merge-191".into();
    receipt["before"]["pullRequest"] = "merged".into();
    receipt["after"]["pullRequest"] = "merged".into();
    receipt["result"] = json!({
        "disposition": "retained",
        "failureCodes": [],
        "unknownCodes": []
    });
    receipt
}

fn set_receipt_head(value: &mut Value, head: &str) {
    value["pullRequest"]["headRevision"] = head.into();
    value["branch"]["headRevision"] = head.into();
    value["worktree"]["headRevision"] = head.into();
}

fn historical_recovery(
    repository_id: &str,
    predecessor_digest: &Digest,
    kind: &str,
    base_revision: &str,
) -> Value {
    json!({
        "schemaVersion": 1,
        "recoveryId": format!("{ID}-historical-recovery"),
        "kind": "historical_finalization_recovery",
        "historicalKind": kind,
        "assurance": "historical_low",
        "workItemId": ID,
        "repositoryId": repository_id,
        "predecessorPath": format!(".ai/decisions/{ID}.finalize.json"),
        "predecessorReceiptDigest": predecessor_digest,
        "baseRevision": base_revision,
        "runtimeVersion": runtime().runtime_version,
        "runtimeDigest": runtime().runtime_digest,
        "actor": "human:test",
        "authoritySource": "historical-recovery-test",
        "reason": "classify immutable legacy finalization without rewriting it",
        "decidedAt": "2026-08-23T00:20:00Z"
    })
}

fn git(directory: &tempfile::TempDir, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(directory.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}

fn commit_archive(directory: &tempfile::TempDir) -> String {
    git(
        directory,
        &["config", "user.email", "tests@example.invalid"],
    );
    git(directory, &["config", "user.name", "AI Cockpit Tests"]);
    git(directory, &["add", "."]);
    git(directory, &["commit", "-q", "-m", "archive"]);
    git(directory, &["rev-parse", "HEAD"])
}

fn write_input(directory: &tempfile::TempDir, name: &str, value: &Value) -> std::path::PathBuf {
    let path = directory.path().join(name);
    fs::write(&path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
    path
}

fn direct_merge_repository() -> (
    tempfile::TempDir,
    ResourceFinalizationContext,
    Digest,
    String,
    String,
    String,
) {
    let directory = tempfile::tempdir().unwrap();
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .unwrap()
            .success()
    );
    git(
        &directory,
        &["config", "user.email", "tests@example.invalid"],
    );
    git(&directory, &["config", "user.name", "AI Cockpit Tests"]);
    git(&directory, &["branch", "-M", "main"]);
    fs::write(directory.path().join("README.md"), "direct merge fixture\n").unwrap();
    git(&directory, &["add", "README.md"]);
    git(&directory, &["commit", "-q", "-m", "base"]);
    let base_revision = git(&directory, &["rev-parse", "HEAD"]);

    git(
        &directory,
        &["checkout", "-q", "-b", "feature/direct-merge"],
    );
    fs::write(
        directory.path().join("direct-merge.txt"),
        "merged history\n",
    )
    .unwrap();
    git(&directory, &["add", "direct-merge.txt"]);
    git(&directory, &["commit", "-q", "-m", "direct merge feature"]);
    let feature_head = git(&directory, &["rev-parse", "HEAD"]);
    git(&directory, &["checkout", "-q", "main"]);
    git(
        &directory,
        &[
            "merge",
            "--no-ff",
            "-q",
            "feature/direct-merge",
            "-m",
            "direct merge",
        ],
    );
    let merge_commit = git(&directory, &["rev-parse", "HEAD"]);
    git(&directory, &["reset", "--hard", "-q", &base_revision]);
    git(&directory, &["branch", "-D", "feature/direct-merge"]);

    attach(directory.path()).unwrap();
    start_work_item_with_options(
        directory.path(),
        ID,
        "record a historical direct merge",
        "preserve direct merge history",
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["direct merge facts remain auditable".into()],
            ..Default::default()
        },
    )
    .unwrap();
    let context = ResourceFinalizationContext {
        branch: "feature/direct-merge".into(),
        worktree: directory.path().to_string_lossy().to_string(),
        base_branch: "main".into(),
        base_remote: "origin".into(),
        provider: "historical".into(),
        pull_request: format!("historical://direct-merge/{merge_commit}"),
    };
    plan_resource_finalization(directory.path(), ID, &context).unwrap();
    let current = runtime();
    let contract_path = directory
        .path()
        .join(format!(".ai/work-items/active/{ID}.contract.json"));
    preflight_work_item_with_runtime(directory.path(), &contract_path, &current).unwrap();
    checkpoint_work_item(directory.path(), ID).unwrap();
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
        ID,
        &evidence,
        &current,
        &run.final_snapshot,
    )
    .unwrap();
    finish_work_item_with_runtime(directory.path(), ID, &current).unwrap();
    archive_work_item_with_runtime(directory.path(), ID, &current).unwrap();
    let contract_path = directory
        .path()
        .join(format!(".ai/work-items/archive/{ID}.contract.json"));
    let contract_digest = Digest::sha256_bytes(&fs::read(contract_path).unwrap());
    (
        directory,
        context,
        contract_digest,
        base_revision,
        feature_head,
        merge_commit,
    )
}

#[test]
fn canonical_record_rejects_pull_request_base_that_differs_from_archived_contract() {
    let (directory, context, contract) = repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let mut receipt = blocked(&repository_id, &context, &contract);
    receipt["pullRequest"]["baseRevision"] = "different-base-revision".into();
    let input = write_input(&directory, "mismatched-base.json", &receipt);

    let error = record_resource_finalization(directory.path(), ID, &input, &runtime())
        .expect_err("record must reject a PR base that differs from the archived Contract base");
    assert!(
        error.to_string().contains(
            "resource finalization pull request base revision does not match the archived Contract base revision"
        ),
        "unexpected error: {error}"
    );
    assert!(
        !directory
            .path()
            .join(format!(".ai/decisions/{ID}.finalize.json"))
            .exists(),
        "a rejected base binding must not create a canonical decision"
    );
}

#[test]
fn canonical_verify_rejects_stored_pull_request_base_that_differs_from_archived_contract() {
    let (directory, context, contract) = repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let receipt = blocked(&repository_id, &context, &contract);
    let input = write_input(&directory, "matching-base.json", &receipt);
    record_resource_finalization(directory.path(), ID, &input, &runtime()).unwrap();

    let decision = directory
        .path()
        .join(format!(".ai/decisions/{ID}.finalize.json"));
    let mut stored: Value = serde_json::from_slice(&fs::read(&decision).unwrap()).unwrap();
    stored["pullRequest"]["baseRevision"] = "different-base-revision".into();
    fs::write(&decision, serde_json::to_vec_pretty(&stored).unwrap()).unwrap();

    let error = verify_resource_finalization(directory.path(), ID, &runtime())
        .expect_err("sequence=0 verify must reject a stale PR base binding");
    assert!(
        error.to_string().contains(
            "resource finalization pull request base revision does not match the archived Contract base revision"
        ),
        "unexpected error: {error}"
    );
}

fn transition_path(decisions: &std::path::Path, value: &Value) -> std::path::PathBuf {
    let digest = cockpit_protocol::digest_json(value).unwrap().to_string();
    decisions.join(format!(
        "{ID}.finalize.{}.json",
        digest.strip_prefix("sha256:").unwrap_or(&digest)
    ))
}

fn post_finalize_evidence(contract: &Digest, head_revision: &str) -> (Value, Value) {
    let manifest_digest = Digest::sha256_bytes(b"repository-gate-manifest").to_string();
    let mut quality_route = json!({
        "automaticProfile": "strict",
        "baseRevision": "unborn",
        "changedPaths": [format!(".ai/decisions/{ID}.finalize.json")],
        "contractDigest": contract,
        "contractPath": format!(".ai/work-items/archive/{ID}.contract.json"),
        "headRevision": head_revision,
        "schemaVersion": 1,
        "kind": "repository_quality_route",
        "manifestDigest": manifest_digest,
        "pathDecisions": [{
            "path": format!(".ai/decisions/{ID}.finalize.json"),
            "profile": "strict",
            "reason": "strict governance path"
        }],
        "reasons": ["strict governance path"],
        "requestedProfile": "strict",
        "requestedRisk": "high",
        "requiredGateIds": ["workspace_package_tests"],
        "risk": "high",
        "selectedProfile": "strict",
        "stage": "pull_request",
    });
    let route_digest = cockpit_protocol::digest_json(&quality_route)
        .unwrap()
        .to_string();
    quality_route["receiptDigest"] = route_digest.into();
    let repository_gates = json!({
        "schemaVersion": 2,
        "state": "passed",
        "route": {
            "manifestDigest": quality_route["manifestDigest"],
            "receiptDigest": quality_route["receiptDigest"],
            "requiredGateIds": quality_route["requiredGateIds"],
            "selectedProfile": quality_route["selectedProfile"]
        },
        "gates": [
            {
                "category": "workspace",
                "command": ["cargo", "test", "--locked", "--workspace"],
                "id": "workspace_package_tests",
                "state": "passed",
                "exitCode": 0
            }
        ]
    });
    (quality_route, repository_gates)
}

fn commit_post_finalize_evidence(
    directory: &tempfile::TempDir,
    contract: &Digest,
    head_revision: &str,
) -> String {
    let (quality_route, repository_gates) = post_finalize_evidence(contract, head_revision);
    commit_post_finalize_evidence_values(directory, &quality_route, &repository_gates)
}

fn commit_post_finalize_evidence_values(
    directory: &tempfile::TempDir,
    quality_route: &Value,
    repository_gates: &Value,
) -> String {
    let evidence_relative = format!(".ai/evidence/{ID}");
    let evidence = directory.path().join(&evidence_relative);
    fs::create_dir_all(&evidence).unwrap();
    fs::write(
        evidence.join("quality-route-post-finalize.json"),
        serde_json::to_vec_pretty(quality_route).unwrap(),
    )
    .unwrap();
    fs::write(
        evidence.join("repository-gates-post-finalize.json"),
        serde_json::to_vec_pretty(repository_gates).unwrap(),
    )
    .unwrap();
    git(directory, &["add", &evidence_relative]);
    git(
        directory,
        &["commit", "-q", "-m", "append post-finalize evidence"],
    );
    git(directory, &["rev-parse", "HEAD"])
}

fn refresh_quality_route_digest(quality_route: &mut Value, repository_gates: &mut Value) {
    quality_route
        .as_object_mut()
        .unwrap()
        .remove("receiptDigest");
    let digest = cockpit_protocol::digest_json(quality_route)
        .unwrap()
        .to_string();
    quality_route["receiptDigest"] = digest.clone().into();
    repository_gates["route"]["receiptDigest"] = digest.into();
}

#[test]
fn wi190_topology_appends_two_transitions_and_resolves_deleted_head() {
    let (directory, context, contract) = repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let blocked = blocked(&repository_id, &context, &contract);
    let canonical_input = write_input(&directory, "blocked.json", &blocked);
    record_resource_finalization(directory.path(), ID, &canonical_input, &runtime()).unwrap();
    let canonical = directory
        .path()
        .join(format!(".ai/decisions/{ID}.finalize.json"));
    let original = fs::read(&canonical).unwrap();

    let observed = transition(&blocked, 1, false);
    let observed_input = write_input(&directory, "observed.json", &observed);
    let result =
        record_resource_finalization(directory.path(), ID, &observed_input, &runtime()).unwrap();
    assert_eq!(result["state"], "appended");
    let observed_receipt = observed["receipt"].clone();
    let deleted = transition(&observed_receipt, 2, true);
    let deleted_input = write_input(&directory, "deleted.json", &deleted);
    record_resource_finalization(directory.path(), ID, &deleted_input, &runtime()).unwrap();

    assert_eq!(fs::read(canonical).unwrap(), original);
    let verified = verify_resource_finalization(directory.path(), ID, &runtime()).unwrap();
    assert_eq!(verified["disposition"], "deleted");
    assert_eq!(verified["sequence"], 2);

    fs::remove_file(canonical_input).unwrap();
    fs::remove_file(observed_input).unwrap();
    fs::remove_file(deleted_input).unwrap();
    close_work_item_with_structured_decision_and_runtime(
        directory.path(),
        ID,
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
        &runtime(),
    )
    .unwrap();
    let close: Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(format!(".ai/decisions/{ID}.close.json")),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(close["resourceFinalizationSequence"], 2);
    assert_eq!(
        close["resourceFinalizationHeadDigest"],
        verified["headDigest"]
    );
}

#[test]
fn close_rejects_retained_finalization_before_writing_decision() {
    let (directory, context, contract) = repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let blocked = blocked(&repository_id, &context, &contract);
    let retained = retained_root(&blocked);
    let input = write_input(&directory, "retained-root.json", &retained);
    record_resource_finalization(directory.path(), ID, &input, &runtime()).unwrap();

    let error = cockpit_repository::close_work_item_with_structured_decision(
        directory.path(),
        ID,
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:test".into(),
            authority_source: "test".into(),
            reason: "retained resources must be cleaned first".into(),
            evidence_refs: vec![],
            policy_refs: vec![],
            decided_at: "2026-08-23T00:10:00Z".into(),
            resume_condition: None,
        },
    )
    .expect_err("close must fail before retained resources are deleted");
    assert!(
        error
            .to_string()
            .contains("requires resource finalization disposition deleted")
    );
    assert!(
        !directory
            .path()
            .join(format!(".ai/decisions/{ID}.close.json"))
            .exists()
    );
}

#[test]
fn historical_runtime_recovery_allows_shared_retained_close_without_rewriting_predecessor() {
    let (directory, context, contract) = primary_repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let mut retained = retained_root(&blocked(&repository_id, &context, &contract));
    retained["runtimeVersion"] = "0.2.47".into();
    retained["runtimeDigest"] = Digest::sha256_bytes(b"runtime-0.2.47").to_string().into();
    let canonical = directory
        .path()
        .join(format!(".ai/decisions/{ID}.finalize.json"));
    fs::write(&canonical, serde_json::to_vec_pretty(&retained).unwrap()).unwrap();
    let predecessor = fs::read(&canonical).unwrap();
    let predecessor_digest = cockpit_protocol::digest_json(&retained).unwrap();
    let before = predecessor.clone();

    assert!(
        verify_resource_finalization(directory.path(), ID, &runtime()).is_err(),
        "an old Runtime receipt must stay blocked before explicit recovery"
    );
    let recovery = historical_recovery(
        &repository_id,
        &predecessor_digest,
        "shared_worktree_retained",
        "unborn",
    );
    let recovery_input = write_input(&directory, "historical-recovery.json", &recovery);
    let recorded =
        record_historical_finalization_recovery(directory.path(), ID, &recovery_input, &runtime())
            .expect("explicit historical recovery should be accepted");
    assert_eq!(recorded["state"], "recorded");

    let verified = verify_resource_finalization(directory.path(), ID, &runtime())
        .expect("historical recovery should permit verification");
    assert_eq!(verified["state"], "verified");
    assert_eq!(verified["historical"], true);
    assert_eq!(verified["assurance"], "historical_low");
    assert_eq!(fs::read(&canonical).unwrap(), before);
    assert!(
        !String::from_utf8_lossy(&predecessor).contains("historical"),
        "the immutable predecessor must not be rewritten"
    );

    close_work_item_with_structured_decision_and_runtime(
        directory.path(),
        ID,
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:test".into(),
            authority_source: "historical-recovery-test".into(),
            reason: "close a classified legacy shared worktree".into(),
            evidence_refs: vec![
                ".ai/decisions/WI-FINALIZATION-TRANSITION.finalize-recovery.json".into(),
            ],
            policy_refs: vec![],
            decided_at: "2026-08-23T00:21:00Z".into(),
            resume_condition: None,
        },
        &runtime(),
    )
    .expect("classified historical retained receipt should close");
}

#[test]
fn historical_runtime_recovery_rejects_foreign_and_tampered_bindings() {
    let (directory, context, contract) = primary_repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let mut retained = retained_root(&blocked(&repository_id, &context, &contract));
    retained["runtimeVersion"] = "0.2.47".into();
    retained["runtimeDigest"] = Digest::sha256_bytes(b"runtime-0.2.47").to_string().into();
    let canonical = directory
        .path()
        .join(format!(".ai/decisions/{ID}.finalize.json"));
    fs::write(&canonical, serde_json::to_vec_pretty(&retained).unwrap()).unwrap();
    let predecessor_digest = cockpit_protocol::digest_json(&retained).unwrap();

    let mut foreign = historical_recovery(
        &repository_id,
        &predecessor_digest,
        "shared_worktree_retained",
        "unborn",
    );
    foreign["repositoryId"] = Digest::sha256_bytes(b"foreign-repository")
        .to_string()
        .into();
    let foreign_input = write_input(&directory, "foreign-recovery.json", &foreign);
    assert!(
        record_historical_finalization_recovery(directory.path(), ID, &foreign_input, &runtime(),)
            .is_err(),
        "foreign repository recovery must fail closed"
    );

    let mut tampered = historical_recovery(
        &repository_id,
        &predecessor_digest,
        "shared_worktree_retained",
        "unborn",
    );
    tampered["predecessorReceiptDigest"] = Digest::sha256_bytes(b"tampered").to_string().into();
    let tampered_input = write_input(&directory, "tampered-recovery.json", &tampered);
    assert!(
        record_historical_finalization_recovery(directory.path(), ID, &tampered_input, &runtime(),)
            .is_err(),
        "tampered predecessor binding must fail closed"
    );
    assert!(
        !directory
            .path()
            .join(format!(".ai/decisions/{ID}.finalize-recovery.json"))
            .exists(),
        "rejected recovery must not publish a destination"
    );
}

#[test]
fn historical_direct_merge_without_pr_is_verifiable_and_closeable() {
    let (directory, context, contract, base_revision, feature_head, merge_commit) =
        direct_merge_repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let receipt = json!({
        "schemaVersion": 1,
        "receiptId": "direct-merge-receipt",
        "operationId": "direct-merge-operation",
        "repositoryId": repository_id,
        "workItemId": ID,
        "runtimeVersion": "test-runtime",
        "runtimeDigest": runtime().runtime_digest,
        "provider": "historical",
        "pullRequest": {
            "number": 0,
            "url": context.pull_request,
            "headRevision": feature_head,
            "baseBranch": "main",
            "baseRemote": "origin",
            "baseRevision": base_revision,
            "mergeCommit": merge_commit
        },
        "branch": {
            "name": context.branch,
            "remote": "origin",
            "headRevision": feature_head
        },
        "worktree": {
            "worktreeId": "primary-repository",
            "path": context.worktree,
            "branch": context.branch,
            "headRevision": feature_head
        },
        "before": {
            "pullRequest": "merged",
            "branch": "present",
            "worktree": "clean"
        },
        "after": {
            "pullRequest": "merged",
            "branch": "deleted",
            "worktree": "clean"
        },
        "result": {
            "disposition": "retained",
            "failureCodes": [],
            "unknownCodes": []
        },
        "actor": "human:test",
        "authoritySource": "historical-test",
        "reason": "record a direct merge without inventing a pull request",
        "timestamp": "2026-08-23T00:30:00Z",
        "contractDigest": contract,
        "resourceContext": context,
        "historical": {
            "kind": "direct_merge_no_pr",
            "assurance": "historical_low",
            "mergeCommit": merge_commit,
            "mergeParents": [base_revision, feature_head],
            "baseRevision": base_revision
        }
    });
    let input = write_input(&directory, "direct-merge.json", &receipt);
    let recorded = record_resource_finalization(directory.path(), ID, &input, &runtime())
        .expect("complete direct merge receipt should be accepted");
    assert_eq!(recorded["state"], "recorded");
    let verified = verify_resource_finalization(directory.path(), ID, &runtime())
        .expect("direct merge receipt should verify against Git");
    assert_eq!(verified["state"], "verified");
    assert_eq!(verified["historical"], true);
    assert_eq!(verified["historicalKind"], "direct_merge_no_pr");
    assert_eq!(verified["assurance"], "historical_low");
    assert_eq!(
        verified["receipt"]["historical"]["kind"],
        "direct_merge_no_pr"
    );

    close_work_item_with_structured_decision_and_runtime(
        directory.path(),
        ID,
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:test".into(),
            authority_source: "historical-test".into(),
            reason: "close a verified historical direct merge".into(),
            evidence_refs: vec![format!(".ai/decisions/{ID}.finalize.json")],
            policy_refs: vec![],
            decided_at: "2026-08-23T00:31:00Z".into(),
            resume_condition: None,
        },
        &runtime(),
    )
    .expect("historical direct merge should permit explicit close");
}

#[cfg(unix)]
#[test]
fn historical_runtime_recovery_rejects_symlink_destination() {
    use std::os::unix::fs::symlink;

    let (directory, context, contract) = primary_repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let mut retained = retained_root(&blocked(&repository_id, &context, &contract));
    retained["runtimeVersion"] = "0.2.47".into();
    retained["runtimeDigest"] = Digest::sha256_bytes(b"runtime-0.2.47").to_string().into();
    let canonical = directory
        .path()
        .join(format!(".ai/decisions/{ID}.finalize.json"));
    fs::write(&canonical, serde_json::to_vec_pretty(&retained).unwrap()).unwrap();
    let predecessor_digest = cockpit_protocol::digest_json(&retained).unwrap();
    let recovery = historical_recovery(
        &repository_id,
        &predecessor_digest,
        "shared_worktree_retained",
        "unborn",
    );
    let input = write_input(&directory, "symlink-recovery-input.json", &recovery);
    let destination = directory
        .path()
        .join(format!(".ai/decisions/{ID}.finalize-recovery.json"));
    symlink(&input, &destination).unwrap();
    assert!(
        record_historical_finalization_recovery(directory.path(), ID, &input, &runtime()).is_err(),
        "symlink recovery destination must fail closed"
    );
}

#[test]
fn post_close_reconciliation_appends_deleted_head_without_rewriting_close() {
    let (directory, context, contract) = repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let blocked = blocked(&repository_id, &context, &contract);
    let retained = retained_root(&blocked);
    let input = write_input(&directory, "retained-root.json", &retained);
    record_resource_finalization(directory.path(), ID, &input, &runtime()).unwrap();
    let canonical = directory
        .path()
        .join(format!(".ai/decisions/{ID}.finalize.json"));
    let canonical_value: Value = serde_json::from_slice(&fs::read(&canonical).unwrap()).unwrap();
    let canonical_digest = cockpit_protocol::digest_json(&canonical_value).unwrap();
    let close = json!({
        "schemaVersion": 1,
        "state": "closed",
        "workItemId": ID,
        "repositoryId": repository_id,
        "decisionState": "confirmed",
        "humanDecision": "approved",
        "resourceFinalizationSequence": 0,
        "resourceFinalizationHeadPath": format!(".ai/decisions/{ID}.finalize.json"),
        "resourceFinalizationHeadDigest": canonical_digest
    });
    let close_path = directory
        .path()
        .join(format!(".ai/decisions/{ID}.close.json"));
    fs::write(&close_path, serde_json::to_vec_pretty(&close).unwrap()).unwrap();
    let close_before = fs::read(&close_path).unwrap();

    let deleted = transition(&canonical_value, 1, true);
    let deleted_input = write_input(&directory, "deleted-after-close.json", &deleted);
    let result = record_resource_finalization(directory.path(), ID, &deleted_input, &runtime())
        .expect("legacy post-close cleanup transition");
    assert_eq!(result["state"], "appended");
    assert_eq!(result["sequence"], 1);
    assert_eq!(fs::read(&close_path).unwrap(), close_before);

    let verified = verify_resource_finalization(directory.path(), ID, &runtime()).unwrap();
    assert_eq!(verified["sequence"], 1);
    assert_eq!(verified["disposition"], "deleted");
}

#[test]
fn post_close_reconciliation_rejects_foreign_transition_runtime_identity() {
    let (directory, context, contract) = repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let blocked = blocked(&repository_id, &context, &contract);
    let retained = retained_root(&blocked);
    let input = write_input(&directory, "retained-root.json", &retained);
    record_resource_finalization(directory.path(), ID, &input, &runtime()).unwrap();
    let canonical = directory
        .path()
        .join(format!(".ai/decisions/{ID}.finalize.json"));
    let canonical_value: Value = serde_json::from_slice(&fs::read(&canonical).unwrap()).unwrap();
    let canonical_digest = cockpit_protocol::digest_json(&canonical_value).unwrap();
    let close = json!({
        "schemaVersion": 1,
        "state": "closed",
        "workItemId": ID,
        "repositoryId": repository_id,
        "decisionState": "confirmed",
        "humanDecision": "approved",
        "resourceFinalizationSequence": 0,
        "resourceFinalizationHeadPath": format!(".ai/decisions/{ID}.finalize.json"),
        "resourceFinalizationHeadDigest": canonical_digest
    });
    let close_path = directory
        .path()
        .join(format!(".ai/decisions/{ID}.close.json"));
    fs::write(&close_path, serde_json::to_vec_pretty(&close).unwrap()).unwrap();

    let mut foreign = transition(&canonical_value, 1, true);
    foreign["receipt"]["runtimeVersion"] = "foreign-runtime".into();
    foreign["receipt"]["runtimeDigest"] =
        Digest::sha256_bytes(b"foreign-runtime").to_string().into();
    let foreign_input = write_input(&directory, "foreign-after-close.json", &foreign);
    let error = record_resource_finalization(
        directory.path(),
        ID,
        &foreign_input,
        &RuntimeContext {
            runtime_version: "new-runtime".into(),
            protocol_version: 1,
            runtime_digest: Digest::sha256_bytes(b"new-runtime"),
        },
    )
    .expect_err("post-close reconciliation must preserve predecessor Runtime identity");
    assert!(
        error.to_string().contains("Runtime identity")
            || error.to_string().contains("runtime identity"),
        "unexpected error: {error}"
    );
}

#[test]
fn wi191_governance_receipt_append_allows_bounded_merge_observation() {
    // This models the real WI-191 governance-only 70c17e4 -> 8f5a025 append:
    // the second commit adds only the canonical finalization receipt.
    let (directory, context, contract) = repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let archive_head = commit_archive(&directory);
    let mut blocked = blocked(&repository_id, &context, &contract);
    set_receipt_head(&mut blocked, &archive_head);
    let canonical_input = write_input(&directory, "blocked.json", &blocked);
    record_resource_finalization(directory.path(), ID, &canonical_input, &runtime()).unwrap();
    let canonical_relative = format!(".ai/decisions/{ID}.finalize.json");
    git(&directory, &["add", &canonical_relative]);
    git(
        &directory,
        &["commit", "-q", "-m", "append canonical governance receipt"],
    );
    let append_head = git(&directory, &["rev-parse", "HEAD"]);

    let mut observed = transition(&blocked, 1, false);
    set_receipt_head(&mut observed["receipt"], &append_head);
    observed["governanceAppendRevision"] = append_head.clone().into();
    let observed_input = write_input(&directory, "observed.json", &observed);
    record_resource_finalization(directory.path(), ID, &observed_input, &runtime()).unwrap();

    let observed_receipt = observed["receipt"].clone();
    let deleted = transition(&observed_receipt, 2, true);
    let deleted_input = write_input(&directory, "deleted.json", &deleted);
    record_resource_finalization(directory.path(), ID, &deleted_input, &runtime()).unwrap();
    let verified = verify_resource_finalization(directory.path(), ID, &runtime()).unwrap();
    assert_eq!(verified["sequence"], 2);
    assert_eq!(verified["disposition"], "deleted");
}

#[test]
fn bounded_same_work_item_post_finalize_evidence_append_is_accepted() {
    let (directory, context, contract) = repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let archive_head = commit_archive(&directory);
    let mut blocked = blocked(&repository_id, &context, &contract);
    set_receipt_head(&mut blocked, &archive_head);
    let canonical_input = write_input(&directory, "blocked.json", &blocked);
    record_resource_finalization(directory.path(), ID, &canonical_input, &runtime()).unwrap();
    let canonical_relative = format!(".ai/decisions/{ID}.finalize.json");
    git(&directory, &["add", &canonical_relative]);
    git(
        &directory,
        &["commit", "-q", "-m", "append canonical governance receipt"],
    );
    let governance_head = git(&directory, &["rev-parse", "HEAD"]);
    let append_head = commit_post_finalize_evidence(&directory, &contract, &governance_head);

    let mut observed = transition(&blocked, 1, false);
    set_receipt_head(&mut observed["receipt"], &append_head);
    observed["governanceAppendRevision"] = append_head.into();
    let observed_input = write_input(&directory, "observed.json", &observed);
    record_resource_finalization(directory.path(), ID, &observed_input, &runtime()).unwrap();

    let verified = verify_resource_finalization(directory.path(), ID, &runtime()).unwrap();
    assert_eq!(verified["sequence"], 1);
    assert_eq!(verified["disposition"], "retained");
}

#[test]
fn malformed_or_cross_bound_post_finalize_evidence_fails_closed() {
    for case in [
        "quality_schema",
        "quality_contract",
        "quality_base",
        "quality_head",
        "quality_digest",
        "gates_schema",
        "gates_route",
        "failed_gate",
    ] {
        let (directory, context, contract) = repository();
        let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
        let archive_head = commit_archive(&directory);
        let mut blocked = blocked(&repository_id, &context, &contract);
        set_receipt_head(&mut blocked, &archive_head);
        let canonical_input = write_input(&directory, "blocked.json", &blocked);
        record_resource_finalization(directory.path(), ID, &canonical_input, &runtime()).unwrap();
        let canonical_relative = format!(".ai/decisions/{ID}.finalize.json");
        git(&directory, &["add", &canonical_relative]);
        git(
            &directory,
            &["commit", "-q", "-m", "append canonical governance receipt"],
        );
        let governance_head = git(&directory, &["rev-parse", "HEAD"]);
        let (mut quality_route, mut repository_gates) =
            post_finalize_evidence(&contract, &governance_head);
        match case {
            "quality_schema" => quality_route["schemaVersion"] = 2.into(),
            "quality_contract" => {
                quality_route["contractPath"] =
                    ".ai/work-items/archive/WI-FOREIGN.contract.json".into();
            }
            "quality_base" => quality_route["baseRevision"] = "foreign-base".into(),
            "quality_head" => quality_route["headRevision"] = "f".repeat(40).into(),
            "quality_digest" => {
                quality_route["receiptDigest"] = Digest::sha256_bytes(b"foreign").to_string().into()
            }
            "gates_schema" => repository_gates["schemaVersion"] = 1.into(),
            "gates_route" => {
                repository_gates["route"]["manifestDigest"] =
                    Digest::sha256_bytes(b"foreign").to_string().into();
            }
            "failed_gate" => {
                repository_gates["gates"][0]["state"] = "failed".into();
                repository_gates["gates"][0]["exitCode"] = 1.into();
            }
            _ => unreachable!(),
        }
        if case != "quality_digest" {
            refresh_quality_route_digest(&mut quality_route, &mut repository_gates);
        }
        let append_head =
            commit_post_finalize_evidence_values(&directory, &quality_route, &repository_gates);
        let mut observed = transition(&blocked, 1, false);
        set_receipt_head(&mut observed["receipt"], &append_head);
        observed["governanceAppendRevision"] = append_head.into();
        let observed_input = write_input(&directory, "observed.json", &observed);
        assert!(
            record_resource_finalization(directory.path(), ID, &observed_input, &runtime())
                .is_err(),
            "{case} must fail closed"
        );
    }
}

#[test]
fn unrelated_or_malformed_governance_append_fails_closed() {
    for foreign in [
        "unrelated.txt".to_string(),
        format!(".ai/decisions/{ID}.finalize.bad.json"),
        ".ai/evidence/WI-FOREIGN/quality-route-post-finalize.json".to_string(),
        format!(".ai/evidence/{ID}/unexpected-post-finalize.json"),
    ] {
        let (directory, context, contract) = repository();
        let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
        let archive_head = commit_archive(&directory);
        let mut blocked = blocked(&repository_id, &context, &contract);
        set_receipt_head(&mut blocked, &archive_head);
        let canonical_input = write_input(&directory, "blocked.json", &blocked);
        record_resource_finalization(directory.path(), ID, &canonical_input, &runtime()).unwrap();
        let foreign_path = directory.path().join(&foreign);
        if let Some(parent) = foreign_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(foreign_path, b"foreign").unwrap();
        let canonical_relative = format!(".ai/decisions/{ID}.finalize.json");
        git(&directory, &["add", &canonical_relative, &foreign]);
        git(&directory, &["commit", "-q", "-m", "foreign append"]);
        let append_head = git(&directory, &["rev-parse", "HEAD"]);
        let mut observed = transition(&blocked, 1, false);
        set_receipt_head(&mut observed["receipt"], &append_head);
        observed["governanceAppendRevision"] = append_head.into();
        let observed_input = write_input(&directory, "observed.json", &observed);
        assert!(
            record_resource_finalization(directory.path(), ID, &observed_input, &runtime())
                .is_err()
        );
    }
}

#[test]
fn incomplete_or_invalid_json_post_finalize_evidence_fails_closed() {
    for case in ["incomplete", "malformed", "duplicate_key"] {
        let (directory, context, contract) = repository();
        let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
        let archive_head = commit_archive(&directory);
        let mut blocked = blocked(&repository_id, &context, &contract);
        set_receipt_head(&mut blocked, &archive_head);
        let canonical_input = write_input(&directory, "blocked.json", &blocked);
        record_resource_finalization(directory.path(), ID, &canonical_input, &runtime()).unwrap();
        let canonical_relative = format!(".ai/decisions/{ID}.finalize.json");
        let evidence_relative = format!(".ai/evidence/{ID}");
        let evidence = directory.path().join(&evidence_relative);
        fs::create_dir_all(&evidence).unwrap();
        let governance_head = git(&directory, &["rev-parse", "HEAD"]);
        let (quality_route, repository_gates) = post_finalize_evidence(&contract, &governance_head);
        match case {
            "incomplete" => fs::write(
                evidence.join("quality-route-post-finalize.json"),
                serde_json::to_vec_pretty(&quality_route).unwrap(),
            )
            .unwrap(),
            "malformed" => {
                fs::write(
                    evidence.join("quality-route-post-finalize.json"),
                    b"{not-json",
                )
                .unwrap();
                fs::write(
                    evidence.join("repository-gates-post-finalize.json"),
                    serde_json::to_vec_pretty(&repository_gates).unwrap(),
                )
                .unwrap();
            }
            "duplicate_key" => {
                fs::write(
                    evidence.join("quality-route-post-finalize.json"),
                    b"{\"schemaVersion\":1,\"schemaVersion\":1}",
                )
                .unwrap();
                fs::write(
                    evidence.join("repository-gates-post-finalize.json"),
                    serde_json::to_vec_pretty(&repository_gates).unwrap(),
                )
                .unwrap();
            }
            _ => unreachable!(),
        }
        git(
            &directory,
            &["add", &canonical_relative, &evidence_relative],
        );
        git(
            &directory,
            &["commit", "-q", "-m", "invalid evidence append"],
        );
        let append_head = git(&directory, &["rev-parse", "HEAD"]);
        let mut observed = transition(&blocked, 1, false);
        set_receipt_head(&mut observed["receipt"], &append_head);
        observed["governanceAppendRevision"] = append_head.into();
        let observed_input = write_input(&directory, "observed.json", &observed);
        assert!(
            record_resource_finalization(directory.path(), ID, &observed_input, &runtime())
                .is_err(),
            "{case} must fail closed"
        );
    }
}

#[test]
fn modified_deleted_or_renamed_post_finalize_evidence_fails_closed() {
    for case in ["modified", "deleted", "renamed"] {
        let (directory, context, contract) = repository();
        let evidence_relative = format!(".ai/evidence/{ID}");
        let evidence = directory.path().join(&evidence_relative);
        fs::create_dir_all(&evidence).unwrap();
        fs::write(evidence.join("quality-route-post-finalize.json"), b"{}\n").unwrap();
        fs::write(
            evidence.join("repository-gates-post-finalize.json"),
            b"{}\n",
        )
        .unwrap();
        let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
        let archive_head = commit_archive(&directory);
        let mut blocked = blocked(&repository_id, &context, &contract);
        set_receipt_head(&mut blocked, &archive_head);
        let canonical_input = write_input(&directory, "blocked.json", &blocked);
        record_resource_finalization(directory.path(), ID, &canonical_input, &runtime()).unwrap();
        match case {
            "modified" => {
                fs::write(evidence.join("quality-route-post-finalize.json"), b"{ }\n").unwrap()
            }
            "deleted" => {
                fs::remove_file(evidence.join("repository-gates-post-finalize.json")).unwrap()
            }
            "renamed" => fs::rename(
                evidence.join("quality-route-post-finalize.json"),
                evidence.join("unexpected-post-finalize.json"),
            )
            .unwrap(),
            _ => unreachable!(),
        }
        let canonical_relative = format!(".ai/decisions/{ID}.finalize.json");
        git(
            &directory,
            &["add", "-A", &canonical_relative, &evidence_relative],
        );
        git(
            &directory,
            &["commit", "-q", "-m", "non-append evidence change"],
        );
        let append_head = git(&directory, &["rev-parse", "HEAD"]);
        let mut observed = transition(&blocked, 1, false);
        set_receipt_head(&mut observed["receipt"], &append_head);
        observed["governanceAppendRevision"] = append_head.into();
        let observed_input = write_input(&directory, "observed.json", &observed);
        assert!(
            record_resource_finalization(directory.path(), ID, &observed_input, &runtime())
                .is_err(),
            "{case} must fail closed"
        );
    }
}

#[cfg(unix)]
#[test]
fn symlinked_governance_append_receipt_fails_closed() {
    let (directory, context, contract) = repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let archive_head = commit_archive(&directory);
    let mut blocked = blocked(&repository_id, &context, &contract);
    set_receipt_head(&mut blocked, &archive_head);
    let canonical_input = write_input(&directory, "blocked.json", &blocked);
    record_resource_finalization(directory.path(), ID, &canonical_input, &runtime()).unwrap();
    let canonical_relative = format!(".ai/decisions/{ID}.finalize.json");
    let symlink_relative = format!(".ai/decisions/{ID}.finalize.{}.json", "0".repeat(64));
    std::os::unix::fs::symlink(
        format!("{ID}.finalize.json"),
        directory.path().join(&symlink_relative),
    )
    .unwrap();
    git(&directory, &["add", &canonical_relative, &symlink_relative]);
    git(&directory, &["commit", "-q", "-m", "symlink append"]);
    let append_head = git(&directory, &["rev-parse", "HEAD"]);
    let mut observed = transition(&blocked, 1, false);
    set_receipt_head(&mut observed["receipt"], &append_head);
    observed["governanceAppendRevision"] = append_head.into();
    let observed_input = write_input(&directory, "observed.json", &observed);
    assert!(
        record_resource_finalization(directory.path(), ID, &observed_input, &runtime()).is_err()
    );
}

#[cfg(unix)]
#[test]
fn symlinked_post_finalize_evidence_fails_closed() {
    let (directory, context, contract) = repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let archive_head = commit_archive(&directory);
    let mut blocked = blocked(&repository_id, &context, &contract);
    set_receipt_head(&mut blocked, &archive_head);
    let canonical_input = write_input(&directory, "blocked.json", &blocked);
    record_resource_finalization(directory.path(), ID, &canonical_input, &runtime()).unwrap();
    let canonical_relative = format!(".ai/decisions/{ID}.finalize.json");
    let evidence_relative = format!(".ai/evidence/{ID}");
    let evidence = directory.path().join(&evidence_relative);
    fs::create_dir_all(&evidence).unwrap();
    fs::write(
        evidence.join("repository-gates-post-finalize.json"),
        b"{}\n",
    )
    .unwrap();
    std::os::unix::fs::symlink(
        "repository-gates-post-finalize.json",
        evidence.join("quality-route-post-finalize.json"),
    )
    .unwrap();
    git(
        &directory,
        &["add", &canonical_relative, &evidence_relative],
    );
    git(
        &directory,
        &["commit", "-q", "-m", "symlink evidence append"],
    );
    let append_head = git(&directory, &["rev-parse", "HEAD"]);
    let mut observed = transition(&blocked, 1, false);
    set_receipt_head(&mut observed["receipt"], &append_head);
    observed["governanceAppendRevision"] = append_head.into();
    let observed_input = write_input(&directory, "observed.json", &observed);
    assert!(
        record_resource_finalization(directory.path(), ID, &observed_input, &runtime()).is_err()
    );
}

#[test]
fn forked_and_symlinked_transition_candidates_fail_closed() {
    let (directory, context, contract) = repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let blocked = blocked(&repository_id, &context, &contract);
    let input = write_input(&directory, "blocked.json", &blocked);
    record_resource_finalization(directory.path(), ID, &input, &runtime()).unwrap();
    let first = transition(&blocked, 1, false);
    let mut second = first.clone();
    second["transitionId"] = "fork".into();
    second["receipt"]["receiptId"] = "fork-receipt".into();
    let decisions = directory.path().join(".ai/decisions");
    let first_path = transition_path(&decisions, &first);
    let second_path = transition_path(&decisions, &second);
    fs::write(&first_path, serde_json::to_vec(&first).unwrap()).unwrap();
    fs::write(&second_path, serde_json::to_vec(&second).unwrap()).unwrap();
    assert!(verify_resource_finalization(directory.path(), ID, &runtime()).is_err());

    fs::remove_file(second_path).unwrap();
    #[cfg(unix)]
    {
        let symlink = decisions.join(format!("{ID}.finalize.symlink.json"));
        std::os::unix::fs::symlink("missing", &symlink).unwrap();
        assert!(verify_resource_finalization(directory.path(), ID, &runtime()).is_err());
        fs::remove_file(symlink).unwrap();
    }
    fs::write(
        decisions.join(format!("{ID}.finalize.malformed.json")),
        b"{\"schemaVersion\":1,\"schemaVersion\":1}",
    )
    .unwrap();
    assert!(verify_resource_finalization(directory.path(), ID, &runtime()).is_err());
}

#[test]
fn stale_missing_foreign_runtime_and_contract_transitions_fail_closed() {
    let (directory, context, contract) = repository();
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let blocked = blocked(&repository_id, &context, &contract);
    let input = write_input(&directory, "blocked.json", &blocked);
    record_resource_finalization(directory.path(), ID, &input, &runtime()).unwrap();

    let mut stale = transition(&blocked, 1, false);
    stale["predecessorReceiptDigest"] = Digest::sha256_bytes(b"missing").to_string().into();
    let stale_input = write_input(&directory, "stale.json", &stale);
    assert!(record_resource_finalization(directory.path(), ID, &stale_input, &runtime()).is_err());

    let mut foreign_runtime = transition(&blocked, 1, false);
    foreign_runtime["receipt"]["runtimeDigest"] =
        Digest::sha256_bytes(b"foreign").to_string().into();
    let foreign_input = write_input(&directory, "foreign-runtime.json", &foreign_runtime);
    assert!(
        record_resource_finalization(directory.path(), ID, &foreign_input, &runtime()).is_err()
    );

    let mut foreign_contract = transition(&blocked, 1, false);
    foreign_contract["receipt"]["contractDigest"] =
        Digest::sha256_bytes(b"foreign-contract").to_string().into();
    let contract_input = write_input(&directory, "foreign-contract.json", &foreign_contract);
    assert!(
        record_resource_finalization(directory.path(), ID, &contract_input, &runtime()).is_err()
    );

    let decisions = directory.path().join(".ai/decisions");
    fs::write(
        transition_path(&decisions, &stale),
        serde_json::to_vec(&stale).unwrap(),
    )
    .unwrap();
    assert!(verify_resource_finalization(directory.path(), ID, &runtime()).is_err());
}

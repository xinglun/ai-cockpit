use cockpit_core::{DecisionState, Digest};
use cockpit_protocol::{ApprovalMode, PolicyLayer, RuntimeContext, VerificationStage};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions, attach,
    checkpoint_work_item, outcome_v2_with_runtime, preflight_work_item_with_runtime,
    record_verification_with_runtime, resolve_verification_route, run_repository_verification,
    start_work_item_with_options,
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::TempDir;

fn git(root: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .status()
            .unwrap()
            .success()
    );
}

fn repository() -> (TempDir, PathBuf) {
    let directory = tempfile::tempdir().expect("temp repository");
    let root = directory.path().to_path_buf();
    fs::write(root.join("tracked.txt"), "before\n").expect("tracked file");
    git(&root, &["init", "-q"]);
    git(&root, &["add", "."]);
    git(
        &root,
        &[
            "-c",
            "user.name=AI Cockpit Test",
            "-c",
            "user.email=ai-cockpit@example.invalid",
            "commit",
            "-qm",
            "fixture",
        ],
    );
    (directory, root)
}

fn policy_for(operation: &str, stage: &str, tier: &str, assurance: &str) -> serde_json::Value {
    json!({
        "schemaVersion": 1,
        "organization": {
            "policyId": "org-route-v1",
            "layer": "organization",
            "rules": [{
                "operation": operation,
                "approvalMode": "no_human_approval_for_low_risk",
                "requiredEvidence": [],
                "verificationRequirement": {
                    "schemaVersion": 1,
        "requiredTier": tier,
                    "requiredAssurance": assurance,
                    "policyRefs": ["org-route-v1"],
                    "stageRefs": [stage],
                    "gateRefs": [],
                    "reason": "protected verification route"
                }
            }]
        }
    })
}

fn start(root: &Path) {
    attach(root).expect("attach");
    start_work_item_with_options(
        root,
        "WI-ROUTE",
        "exercise policy route",
        "verify policy route",
        &["tracked.txt".into()],
        &WorkItemStartOptions {
            risk: "normal".into(),
            authority: "authorized".into(),
            acceptance_criteria: vec!["route is enforced".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
}

#[test]
fn policy_t3_provider_requirement_fails_before_execution_and_evidence() {
    let (_directory, root) = repository();
    start(&root);
    fs::write(
        root.join(".ai/policy.json"),
        serde_json::to_vec_pretty(&policy_for(
            "modify_source",
            "pre_ci",
            "T3",
            "provider_verified",
        ))
        .unwrap(),
    )
    .expect("policy");
    let snapshot = cockpit_git::GitRepository::discover(&root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let error = resolve_verification_route(
        &root,
        "WI-ROUTE",
        VerificationStage::PreCi,
        "local",
        &snapshot,
    )
    .expect_err("local verification cannot satisfy T3/provider policy");
    let message = error.to_string();
    assert!(message.contains("not satisfied"), "{message}");
    assert!(
        !root
            .join(".ai/evidence/WI-ROUTE.verification.json")
            .exists()
    );
}

#[test]
fn release_route_requires_contract_base_revision() {
    let (_directory, root) = repository();
    start(&root);
    let contract_path = root.join(".ai/work-items/active/WI-ROUTE.contract.json");
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).unwrap()).unwrap();
    contract["baseRevision"] = json!("");
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract).unwrap(),
    )
    .unwrap();
    let snapshot = cockpit_git::GitRepository::discover(&root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let error = resolve_verification_route(
        &root,
        "WI-ROUTE",
        VerificationStage::Release,
        "local",
        &snapshot,
    )
    .expect_err("release requires base revision");
    assert!(error.to_string().contains("baseRevision"));
}

#[test]
fn no_policy_route_remains_compatible() {
    let (_directory, root) = repository();
    start(&root);
    let snapshot = cockpit_git::GitRepository::discover(&root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let route = resolve_verification_route(
        &root,
        "WI-ROUTE",
        VerificationStage::Task,
        "local",
        &snapshot,
    )
    .expect("no policy route");
    assert!(route.policy_plan.is_none());
    assert_eq!(route.stage, VerificationStage::Task);
}

#[test]
fn tampered_policy_plan_required_tier_blocks_outcome() {
    let (_directory, root) = repository();
    start(&root);
    fs::write(
        root.join(".ai/policy.json"),
        serde_json::to_vec_pretty(&policy_for(
            "modify_source",
            "task",
            "T0",
            "repository_verified",
        ))
        .unwrap(),
    )
    .expect("policy");
    let current = RuntimeContext {
        runtime_version: "route-test".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"route-test"),
    };
    let contract_path = root.join(".ai/work-items/active/WI-ROUTE.contract.json");
    preflight_work_item_with_runtime(&root, &contract_path, &current).expect("preflight");
    checkpoint_work_item(&root, "WI-ROUTE").expect("checkpoint");
    let run = run_repository_verification(
        &root,
        &RepositoryVerificationRequest {
            node_id: "project-command-0".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec!["**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: current.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .expect("verify");
    let mut raw = serde_json::to_value(&run.receipt).expect("receipt");
    raw["runtimeVersion"] = current.runtime_version.clone().into();
    raw["runtimeDigest"] = current.runtime_digest.to_string().into();
    record_verification_with_runtime(&root, "WI-ROUTE", &raw, &current, &run.final_snapshot)
        .expect("record");
    let evidence_path = root.join(".ai/evidence/WI-ROUTE.verification.json");
    let mut evidence: serde_json::Value =
        serde_json::from_slice(&fs::read(&evidence_path).unwrap()).unwrap();
    let contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).unwrap()).unwrap();
    let repository_id = evidence["repositoryId"].clone();
    let snapshot_digest = evidence["repositorySnapshotDigest"].clone();
    let base_revision = contract["baseRevision"].clone();
    let plan = evidence["receipt"]["planReceipt"]
        .as_object_mut()
        .expect("plan receipt");
    plan.insert("workItemId".into(), json!("WI-ROUTE"));
    plan.insert("repositoryId".into(), repository_id);
    plan.insert("repositorySnapshotDigest".into(), snapshot_digest);
    plan.insert("baseRevision".into(), base_revision);
    plan.insert("requiredTier".into(), json!("T0"));
    plan.insert("requiredAssurance".into(), json!("repository_verified"));
    plan.insert("policyRefs".into(), json!(["org-route-v1"]));
    plan.insert("dependencyConfidence".into(), json!("unknown"));
    plan.insert("assurance".into(), json!("repository_verified"));
    let receipt_digest = cockpit_protocol::digest_json(&evidence["receipt"]).unwrap();
    evidence["receiptDigest"] = json!(receipt_digest.to_string());
    fs::write(
        &evidence_path,
        serde_json::to_vec_pretty(&evidence).unwrap(),
    )
    .unwrap();
    let valid = outcome_v2_with_runtime(&root, "WI-ROUTE", &current).expect("valid outcome");
    assert_eq!(valid.decision_state, Some(DecisionState::Green));

    let mut tampered: serde_json::Value =
        serde_json::from_slice(&fs::read(&evidence_path).unwrap()).unwrap();
    tampered["receipt"]["planReceipt"]["requiredTier"] = json!("T3");
    let receipt_digest = cockpit_protocol::digest_json(&tampered["receipt"]).unwrap();
    tampered["receiptDigest"] = json!(receipt_digest.to_string());
    fs::write(
        &evidence_path,
        serde_json::to_vec_pretty(&tampered).unwrap(),
    )
    .unwrap();
    let rejected = outcome_v2_with_runtime(&root, "WI-ROUTE", &current).expect("tampered outcome");
    assert_eq!(rejected.decision_state, Some(DecisionState::Red));
}

#[test]
fn protocol_policy_layer_type_remains_available_to_route_fixtures() {
    assert_eq!(format!("{:?}", PolicyLayer::Organization), "Organization");
    assert_eq!(
        format!("{:?}", ApprovalMode::NoHumanApprovalForLowRisk),
        "NoHumanApprovalForLowRisk"
    );
}

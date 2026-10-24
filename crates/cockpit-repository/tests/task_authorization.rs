use cockpit_core::Digest;
use cockpit_protocol::RuntimeContext;
use cockpit_repository::{
    WorkItemStartOptions, amend_work_item_contract_with_runtime, attach,
    preflight_work_item_with_runtime_report, record_work_item_governance_controls_with_runtime,
    repository_id, start_work_item_with_options, work_item_status_snapshot_with_runtime,
};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

const WI: &str = "WI-TASK-AUTHORIZATION";

fn git(root: &Path, args: &[&str]) {
    let result = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "1.0.1".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"isolated-task-authorization-fixture"),
    }
}

fn fixture(expires_at: &str, owner_review: bool) -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    git(root, &["init", "-q", "--initial-branch=main"]);
    git(root, &["config", "user.name", "Task Authorization Test"]);
    git(
        root,
        &["config", "user.email", "task-authorization@example.invalid"],
    );
    git(root, &["config", "core.autocrlf", "false"]);
    fs::write(root.join("README.md"), "task authorization baseline\n").unwrap();
    git(root, &["add", "README.md"]);
    git(root, &["commit", "-qm", "baseline"]);
    attach(root).unwrap();
    start_work_item_with_options(
        root,
        WI,
        "continue bounded work",
        "deliver the same task",
        &["README.md".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["bounded continuation preserves real review".into()],
            verification_commands: vec!["git diff --check".into()],
            ..Default::default()
        },
    )
    .unwrap();
    let contract_path = root.join(format!(".ai/work-items/active/{WI}.contract.json"));
    let contract: Value = serde_json::from_slice(&fs::read(&contract_path).unwrap()).unwrap();
    amend_work_item_contract_with_runtime(
        root,
        WI,
        &json!({
            "schemaVersion":1, "changeId":"explicit-task-grant",
            "expectedContractDigest":cockpit_protocol::digest_json(&contract).unwrap(),
            "reason":"isolated owner plan for a finite task",
            "changes":[{"path":"/governanceProfile", "operation":"set", "value":{
                "taskAuthorization":{
                    "schemaVersion":1, "agentActor":"agent:task-worker",
                    "authoritySource":"conversation:fixture-owner",
                    "allowedActions":["run_preflight","run_verification"],
                    "closeoutActions":[], "expiresAt":expires_at, "publicTarget":null
                }
            }}]
        }),
        "isolated owner plan for a finite task",
        &runtime(),
    )
    .unwrap();
    preflight_work_item_with_runtime_report(root, &contract_path, &runtime()).unwrap();
    if owner_review {
        record_owner(root);
    }
    directory
}

fn record_owner(root: &Path) {
    let contract_path = root.join(format!(".ai/work-items/active/{WI}.contract.json"));
    let contract: Value = serde_json::from_slice(&fs::read(&contract_path).unwrap()).unwrap();
    let summary: Value = serde_json::from_slice(
        &fs::read(root.join(format!(".ai/work-items/active/{WI}.summary.json"))).unwrap(),
    )
    .unwrap();
    record_work_item_governance_controls_with_runtime(root, WI, &json!({"decisionEvidence":{
        "schemaVersion":1, "decisionId":"contract-preflight-review", "decision":"confirm_review",
        "workItemId":WI, "repositoryId":repository_id(root),
        "contractDigest":cockpit_protocol::digest_json(&contract).unwrap(),
        "preflightDecisionDigest":summary["preflightDecisionDigest"],
        "repositorySnapshotDigest":summary["preflightRepositorySnapshotDigest"],
        "recordedAt":chrono::Utc::now().to_rfc3339(), "recordedBy":"agent:test-recorder",
        "reason":"Synthetic isolated owner confirmation; not external identity authentication."
    }}), &runtime()).unwrap();
}

fn status(root: &Path) -> Value {
    serde_json::to_value(work_item_status_snapshot_with_runtime(root, WI, &runtime()).unwrap())
        .unwrap()
}

#[test]
fn native_owner_review_makes_exact_task_authority_visible() {
    let directory = fixture("2099-01-01T00:00:00Z", true);
    let value = status(directory.path());
    assert_eq!(value["taskAuthorization"]["state"], "active");
    assert_eq!(
        value["taskAuthorization"]["agentActor"],
        "agent:task-worker"
    );
    assert_eq!(value["taskAuthorization"]["assurance"], "self_declared");
    assert!(
        value["taskAuthorization"]["grantId"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );
}

#[test]
fn an_actor_string_without_native_owner_review_does_not_grant_authority() {
    let directory = fixture("2099-01-01T00:00:00Z", false);
    assert_eq!(
        status(directory.path())["taskAuthorization"]["state"],
        "owner_review_required"
    );
}

#[test]
fn an_expired_reviewed_grant_cannot_authorize_future_work() {
    let directory = fixture("2000-01-01T00:00:00Z", true);
    assert_eq!(
        status(directory.path())["taskAuthorization"]["state"],
        "expired"
    );
}

#[test]
fn source_commit_preserves_authority_without_claiming_current_test_evidence() {
    let directory = fixture("2099-01-01T00:00:00Z", true);
    let before = status(directory.path());
    fs::write(
        directory.path().join("README.md"),
        "normal in-scope source revision\n",
    )
    .unwrap();
    git(directory.path(), &["add", "README.md"]);
    git(
        directory.path(),
        &["commit", "-qm", "ordinary source revision"],
    );
    let after = status(directory.path());
    assert_eq!(after["taskAuthorization"]["state"], "active");
    assert_eq!(
        after["taskAuthorization"]["grantId"],
        before["taskAuthorization"]["grantId"]
    );
    assert_eq!(after["evidenceFreshness"]["state"], "missing");
}

fn termination(root: &Path, decision: &str) -> Value {
    json!({"taskAuthorizationTermination": {
        "schemaVersion": 1,
        "grantId": status(root)["taskAuthorization"]["grantId"],
        "decision": decision,
        "caller": {"schemaVersion": 1, "actor": "agent:task-worker",
            "authoritySource": "conversation:fixture-owner", "assurance": "self_declared"},
        "reason": "The exact delegated agent narrows its own task authority."
    }})
}

#[test]
fn withdrawing_exact_authority_is_immutable_idempotent_and_not_business_completion() {
    let directory = fixture("2099-01-01T00:00:00Z", true);
    let root = directory.path();
    let request = termination(root, "withdrawn");
    let first = record_work_item_governance_controls_with_runtime(root, WI, &request, &runtime())
        .expect("the exact authorized agent may narrow its own grant");
    let second = record_work_item_governance_controls_with_runtime(root, WI, &request, &runtime())
        .expect("the identical terminal request is idempotent");
    assert_eq!(first, second);
    let projected = status(root);
    assert_eq!(projected["taskAuthorization"]["state"], "withdrawn");
    assert_eq!(projected["lifecyclePhase"], "implementation_active");
    assert_ne!(projected["verification"], "verified");
    assert!(!root.join(format!(".ai/decisions/{WI}.close.json")).exists());
    let mut conflicting = request;
    conflicting["taskAuthorizationTermination"]["decision"] = json!("completed");
    assert!(
        record_work_item_governance_controls_with_runtime(root, WI, &conflicting, &runtime())
            .is_err()
    );
    assert_eq!(status(root)["taskAuthorization"]["state"], "withdrawn");
}

#[test]
fn a_foreign_caller_or_grant_cannot_terminate_the_current_authority() {
    let directory = fixture("2099-01-01T00:00:00Z", true);
    let root = directory.path();
    for field in ["caller", "grantId"] {
        let mut request = termination(root, "withdrawn");
        if field == "caller" {
            request["taskAuthorizationTermination"]["caller"]["actor"] =
                json!("agent:someone-else");
        } else {
            request["taskAuthorizationTermination"]["grantId"] =
                json!(Digest::sha256_bytes(b"another grant"));
        }
        assert!(
            record_work_item_governance_controls_with_runtime(root, WI, &request, &runtime())
                .is_err()
        );
        assert_eq!(status(root)["taskAuthorization"]["state"], "active");
    }
}

#[test]
fn termination_cannot_smuggle_an_unrelated_governance_write() {
    let directory = fixture("2099-01-01T00:00:00Z", true);
    let root = directory.path();
    let path = root.join(format!(".ai/work-items/active/{WI}.summary.json"));
    let before = fs::read(&path).unwrap();
    let mut request = termination(root, "withdrawn");
    request["intentAlignment"] = json!({"state":"aligned"});
    assert!(
        record_work_item_governance_controls_with_runtime(root, WI, &request, &runtime()).is_err()
    );
    assert_eq!(fs::read(path).unwrap(), before);
    assert_eq!(status(root)["taskAuthorization"]["state"], "active");
}

#[test]
fn an_invalid_deadline_is_rejected_before_any_contract_or_amendment_write() {
    let directory = fixture("2099-01-01T00:00:00Z", true);
    let root = directory.path();
    let path = root.join(format!(".ai/work-items/active/{WI}.contract.json"));
    let before = fs::read(&path).unwrap();
    let contract: Value = serde_json::from_slice(&before).unwrap();
    let count = cockpit_repository::read_work_item_contract_amendments(root, WI)
        .unwrap()
        .len();
    let result = amend_work_item_contract_with_runtime(
        root,
        WI,
        &json!({
            "schemaVersion":1,"changeId":"invalid-time","expectedContractDigest":cockpit_protocol::digest_json(&contract).unwrap(),
            "reason":"check malformed timestamp refusal",
            "changes":[{"path":"/governanceProfile/taskAuthorization/expiresAt","operation":"set","value":"not-a-timestamp"}]
        }),
        "check malformed timestamp refusal",
        &runtime(),
    );
    assert!(
        result.is_err(),
        "invalid task deadline must be rejected before journaling"
    );
    assert_eq!(fs::read(path).unwrap(), before);
    assert_eq!(
        cockpit_repository::read_work_item_contract_amendments(root, WI)
            .unwrap()
            .len(),
        count
    );
}

#[test]
fn a_verified_mechanical_amendment_retains_the_original_task_grant_identity() {
    let directory = fixture("2099-01-01T00:00:00Z", false);
    let root = directory.path();
    let path = root.join(format!(".ai/work-items/active/{WI}.contract.json"));
    let contract: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let mut profile = contract["governanceProfile"].clone();
    profile["developmentReview"] = json!({
        "schemaVersion":1,"reviewerActor":"agent:task-worker","authoritySource":"conversation:fixture-owner",
        "allowedChanges":["append_scenarios"],"preapprovedCommands":[]
    });
    profile["taskAuthorization"]["allowedActions"]
        .as_array_mut()
        .unwrap()
        .push(json!("review_development_amendment"));
    amend_work_item_contract_with_runtime(root, WI, &json!({
        "schemaVersion":1,"changeId":"combined-local-review-plan","expectedContractDigest":cockpit_protocol::digest_json(&contract).unwrap(),
        "reason":"explicit isolated owner plan for mechanical review and task authority",
        "changes":[{"path":"/governanceProfile","operation":"replace","value":profile}]
    }), "explicit isolated owner plan for mechanical review and task authority", &runtime()).unwrap();
    preflight_work_item_with_runtime_report(root, &path, &runtime()).unwrap();
    record_owner(root);
    let before = status(root);
    assert_eq!(before["taskAuthorization"]["state"], "active");
    let contract: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    amend_work_item_contract_with_runtime(root, WI, &json!({
        "schemaVersion":1,"changeId":"permitted-mechanical-scenario","expectedContractDigest":cockpit_protocol::digest_json(&contract).unwrap(),
        "reason":"append one explicitly permitted unverified scenario",
        "developmentReview":{"assurance":"self_declared","actor":"agent:task-worker","authoritySource":"conversation:fixture-owner",
            "operations":["review_development_amendment"],"policyRefs":[],"evidenceRefs":[]},
        "changes":[{"path":"/scenarioCoverage","operation":"add","value":{
            "scenario":"additional-declared-case","required":true,"status":"unverified","evidence":[],
            "expected":"the new case is tested after implementation","verificationPlan":"git diff --check"}}]
    }), "append one explicitly permitted unverified scenario", &runtime()).unwrap();
    let after = status(root);
    assert_eq!(after["taskAuthorization"]["state"], "active");
    assert_eq!(
        after["taskAuthorization"]["grantId"],
        before["taskAuthorization"]["grantId"]
    );
    assert!(!after["humanDecisionRequired"].as_bool().unwrap());
}

#[test]
fn refreshing_a_review_for_the_same_contract_cannot_revive_a_terminated_grant() {
    let directory = fixture("2099-01-01T00:00:00Z", true);
    let root = directory.path();
    let before = status(root);
    record_work_item_governance_controls_with_runtime(
        root,
        WI,
        &termination(root, "withdrawn"),
        &runtime(),
    )
    .unwrap();
    record_owner(root);
    let after = status(root);
    assert_eq!(after["taskAuthorization"]["state"], "withdrawn");
    assert_eq!(
        after["taskAuthorization"]["grantId"],
        before["taskAuthorization"]["grantId"]
    );
}

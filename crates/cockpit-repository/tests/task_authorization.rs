use cockpit_core::Digest;
use cockpit_protocol::RuntimeContext;
use cockpit_repository::{
    WorkItemStartOptions, amend_work_item_contract_with_runtime, attach,
    preflight_work_item_with_runtime_report, record_work_item_governance_controls_with_runtime,
    repository_id, start_work_item_with_options, work_item_status_snapshot_with_runtime,
};
use serde_json::Value;
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
        &::serde_json::json!({
            "schemaVersion":1, "changeId":"explicit-task-grant",
            "expectedContractDigest":cockpit_protocol::digest_json(&contract).unwrap(),
            "reason":"isolated owner plan for a finite task",
            "changes":[{"path":"/governanceProfile", "operation":"set", "value":{
                "taskAuthorization":{
                    "schemaVersion":1, "agentActor":"agent:task-worker",
                    "authoritySource":"conversation:fixture-owner",
                    "allowedActions":["checkpoint","run_preflight","run_verification"],
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
    record_work_item_governance_controls_with_runtime(root, WI, &::serde_json::json!({"decisionEvidence":{
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
    ::serde_json::json!({"taskAuthorizationTermination": {
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
    conflicting["taskAuthorizationTermination"]["decision"] = ::serde_json::json!("completed");
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
                ::serde_json::json!("agent:someone-else");
        } else {
            request["taskAuthorizationTermination"]["grantId"] =
                ::serde_json::json!(Digest::sha256_bytes(b"another grant"));
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
    request["intentAlignment"] = ::serde_json::json!({"state":"aligned"});
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
        &::serde_json::json!({
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
    profile["developmentReview"] = ::serde_json::json!({
        "schemaVersion":1,"reviewerActor":"agent:task-worker","authoritySource":"conversation:fixture-owner",
        "allowedChanges":["append_scenarios"],"preapprovedCommands":[]
    });
    profile["taskAuthorization"]["allowedActions"]
        .as_array_mut()
        .unwrap()
        .push(::serde_json::json!("review_development_amendment"));
    amend_work_item_contract_with_runtime(root, WI, &::serde_json::json!({
        "schemaVersion":1,"changeId":"combined-local-review-plan","expectedContractDigest":cockpit_protocol::digest_json(&contract).unwrap(),
        "reason":"explicit isolated owner plan for mechanical review and task authority",
        "changes":[{"path":"/governanceProfile","operation":"replace","value":profile}]
    }), "explicit isolated owner plan for mechanical review and task authority", &runtime()).unwrap();
    preflight_work_item_with_runtime_report(root, &path, &runtime()).unwrap();
    record_owner(root);
    let before = status(root);
    assert_eq!(before["taskAuthorization"]["state"], "active");
    let contract: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    amend_work_item_contract_with_runtime(root, WI, &::serde_json::json!({
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
fn public_actions_require_an_explicit_owner_bound_destination() {
    let directory = fixture("2099-01-01T00:00:00Z", true);
    let root: &::std::path::Path = directory.path();
    let path = root.join(format!(".ai/work-items/active/{WI}.contract.json"));
    let before = fs::read(&path).unwrap();
    let contract: Value = serde_json::from_slice(&before).unwrap();
    let result = amend_work_item_contract_with_runtime(
        root,
        WI,
        &::serde_json::json!({
            "schemaVersion":1,"changeId":"unspecified-public-target","expectedContractDigest":cockpit_protocol::digest_json(&contract).unwrap(),
            "reason":"refuse a public operation with no owner destination",
            "changes":[{"path":"/governanceProfile/taskAuthorization/allowedActions","operation":"replace","value":["publish_branch"]}]
        }),
        "refuse a public operation with no owner destination",
        &runtime(),
    );
    assert!(result.is_err());
    assert_eq!(fs::read(path).unwrap(), before);
}

#[test]
fn unknown_terminal_record_fields_fail_closed_even_when_the_digest_is_resealed() {
    let directory = fixture("2099-01-01T00:00:00Z", true);
    let root = directory.path();
    record_work_item_governance_controls_with_runtime(
        root,
        WI,
        &termination(root, "withdrawn"),
        &runtime(),
    )
    .unwrap();
    let files: Vec<_> = fs::read_dir(root.join(".ai/decisions"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .ends_with(".terminated.json")
        })
        .collect();
    assert_eq!(files.len(), 1);
    let path = &files[0];
    let mut record: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    record.as_object_mut().unwrap().remove("recordDigest");
    record["unexpectedAuthorityField"] = ::serde_json::json!(true);
    record["recordDigest"] = ::serde_json::json!(cockpit_protocol::digest_json(&record).unwrap());
    fs::write(path, serde_json::to_vec(&record).unwrap()).unwrap();
    assert_eq!(status(root)["taskAuthorization"]["state"], "invalid");
}

#[test]
fn a_context_free_admission_cannot_claim_the_configured_agent_identity() {
    let directory = fixture("2099-01-01T00:00:00Z", true);
    let result = cockpit_repository::require_current_action_admission(
        directory.path(),
        WI,
        "run_preflight",
        &runtime(),
    );
    assert!(
        result.is_err(),
        "the policy actor is not evidence of the actual caller"
    );
}

#[test]
fn expired_authority_rejects_preflight_before_summary_side_effects() {
    let directory = fixture("2000-01-01T00:00:00Z", true);
    let root = directory.path();
    let path = root.join(format!(".ai/work-items/active/{WI}.summary.json"));
    let before = fs::read(&path).unwrap();
    let result = preflight_work_item_with_runtime_report(
        root,
        &root.join(format!(".ai/work-items/active/{WI}.contract.json")),
        &runtime(),
    );
    assert!(
        result.is_err(),
        "expired authority must not enter the preflight write"
    );
    assert_eq!(fs::read(path).unwrap(), before);
}

#[test]
fn legacy_controls_without_a_caller_cannot_mutate_a_configured_task() {
    let directory = fixture("2099-01-01T00:00:00Z", true);
    let root = directory.path();
    let path = root.join(format!(".ai/work-items/active/{WI}.summary.json"));
    let before = fs::read(&path).unwrap();
    let result = cockpit_repository::record_work_item_governance_controls(
        root,
        WI,
        &::serde_json::json!({}),
    );
    assert!(
        result.is_err(),
        "a legacy no-context write cannot impersonate the delegated actor"
    );
    assert_eq!(fs::read(path).unwrap(), before);
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

fn caller() -> cockpit_protocol::TaskAuthorizationCaller {
    cockpit_protocol::TaskAuthorizationCaller {
        schema_version: 1,
        actor: "agent:task-worker".into(),
        authority_source: "conversation:fixture-owner".into(),
        assurance: cockpit_protocol::AssuranceLevel::SelfDeclared,
    }
}

fn intent(action: &str, paths: &[&str]) -> cockpit_protocol::TaskAuthorizationRequest {
    cockpit_protocol::TaskAuthorizationRequest {
        schema_version: 1,
        caller: caller(),
        action: action.into(),
        paths: paths.iter().map(|p| p.to_string()).collect(),
        public_target: None,
    }
}

#[test]
fn explicit_caller_continues_without_fabricating_fresh_verification() {
    let directory = fixture("2099-01-01T00:00:00Z", true);
    cockpit_repository::checkpoint_work_item_with_caller(
        directory.path(),
        WI,
        &runtime(),
        Some(&caller()),
    )
    .unwrap();
    let request = intent("run_verification", &[]);
    let before =
        cockpit_repository::assess_task_authorization(directory.path(), WI, &request, &runtime())
            .unwrap();
    assert!(before.authorized);
    assert_eq!(before.next_action, "reverify");
    assert_eq!(before.action_admitted, Some(true));
    cockpit_repository::require_current_action_admission_with_caller(
        directory.path(),
        WI,
        "run_verification",
        &runtime(),
        Some(&caller()),
    )
    .unwrap();
    fs::write(
        directory.path().join("README.md"),
        "new bounded implementation\n",
    )
    .unwrap();
    git(directory.path(), &["add", "README.md"]);
    git(directory.path(), &["commit", "-qm", "ordinary new source"]);
    let after =
        cockpit_repository::assess_task_authorization(directory.path(), WI, &request, &runtime())
            .unwrap();
    assert!(after.authorized);
    assert_eq!(
        after.authority.unwrap().grant_id,
        before.authority.unwrap().grant_id
    );
    assert_ne!(after.evidence_freshness.state, "fresh");
    assert_ne!(after.next_action, "owner_review_required");
    let contract = directory
        .path()
        .join(format!(".ai/work-items/active/{WI}.contract.json"));
    cockpit_repository::preflight_work_item_with_task_authorization(
        directory.path(),
        &contract,
        &runtime(),
        &caller(),
    )
    .unwrap();
    let refreshed = status(directory.path());
    assert!(!refreshed["humanDecisionRequired"].as_bool().unwrap());
    assert_eq!(refreshed["taskAuthorization"]["state"], "active");
}

#[test]
fn an_authorization_question_cannot_expand_files_or_public_destination() {
    let directory = fixture("2099-01-01T00:00:00Z", true);
    for path in [
        "unapproved.rs",
        "../README.md",
        "/README.md",
        "README.md/../foreign",
    ] {
        let request = intent("run_verification", &[path]);
        let result = cockpit_repository::assess_task_authorization(
            directory.path(),
            WI,
            &request,
            &runtime(),
        )
        .unwrap();
        assert!(!result.authorized, "foreign path accepted: {path}");
        assert_eq!(result.next_action, "stop");
    }
    let mut request = intent("run_verification", &["README.md"]);
    request.public_target = Some(cockpit_protocol::TaskAuthorizationTarget {
        provider: "github".into(),
        repository: "foreign/repository".into(),
        base_branch: "main".into(),
    });
    assert!(
        !cockpit_repository::assess_task_authorization(directory.path(), WI, &request, &runtime())
            .unwrap()
            .authorized
    );
}

#[test]
fn caller_context_is_explicit_and_rechecked_after_termination() {
    let directory = fixture("2099-01-01T00:00:00Z", true);
    let root = directory.path();
    let mut foreign = caller();
    foreign.actor = "human:owner".into();
    assert!(
        cockpit_repository::require_current_action_admission_with_caller(
            root,
            WI,
            "run_verification",
            &runtime(),
            Some(&foreign)
        )
        .is_err()
    );
    let request = intent("run_verification", &[]);
    assert!(
        cockpit_repository::assess_task_authorization(root, WI, &request, &runtime())
            .unwrap()
            .authorized
    );
    record_work_item_governance_controls_with_runtime(
        root,
        WI,
        &termination(root, "withdrawn"),
        &runtime(),
    )
    .unwrap();
    let current =
        cockpit_repository::assess_task_authorization(root, WI, &request, &runtime()).unwrap();
    assert!(!current.authorized);
    assert_eq!(current.next_action, "stop");
    assert!(
        cockpit_repository::require_current_action_admission_with_caller(
            root,
            WI,
            "run_verification",
            &runtime(),
            Some(&caller())
        )
        .is_err()
    );
}

#[test]
fn a_node_budget_is_finite_reduced_and_rechecked_at_each_boundary() {
    let deadline = (chrono::Utc::now() + chrono::Duration::seconds(300)).to_rfc3339();
    let directory = fixture(&deadline, true);
    let root = directory.path();
    let bounded =
        cockpit_repository::task_authorization_timeout(root, WI, &runtime(), Some(&caller()), 900)
            .unwrap();
    assert!((1..=300).contains(&bounded));
    assert_eq!(
        cockpit_repository::task_authorization_timeout(root, WI, &runtime(), Some(&caller()), 42)
            .unwrap(),
        42
    );
    for ceiling in [0, 901, u64::MAX] {
        assert!(
            cockpit_repository::task_authorization_timeout(
                root,
                WI,
                &runtime(),
                Some(&caller()),
                ceiling
            )
            .is_err()
        );
    }
    record_work_item_governance_controls_with_runtime(
        root,
        WI,
        &termination(root, "withdrawn"),
        &runtime(),
    )
    .unwrap();
    assert!(
        cockpit_repository::task_authorization_timeout(root, WI, &runtime(), Some(&caller()), 42)
            .is_err()
    );
}

#[test]
fn an_expired_caller_cannot_write_a_blocked_finish_outcome() {
    let directory = fixture("2000-01-01T00:00:00Z", true);
    let root = directory.path();
    let outcome = root.join(format!(".ai/work-items/active/{WI}.outcome.json"));
    let before = fs::read(&outcome).ok();
    assert!(
        cockpit_repository::finish_work_item_with_caller(root, WI, &runtime(), Some(&caller()))
            .is_err()
    );
    assert_eq!(fs::read(outcome).ok(), before);
}

fn command_request(
    program: &str,
    args: &[&str],
) -> cockpit_repository::RepositoryVerificationRequest {
    cockpit_repository::RepositoryVerificationRequest {
        node_id: "project-command-0".into(),
        program: program.into(),
        args: args.iter().map(|s| s.to_string()).collect(),
        scope: vec!["README.md".into()],
        stage: "task".into(),
        runner: "local".into(),
        runtime_digest: runtime().runtime_digest.to_string(),
        base_commit: None,
        workers: 1,
        work_item_id: Some(WI.into()),
        timeout_seconds: None,
        policy: cockpit_repository::RepositoryVerificationPolicy::NeverReuse,
    }
}

#[test]
fn actual_child_execution_requires_explicit_caller_and_declared_argv() {
    let directory = fixture("2099-01-01T00:00:00Z", true);
    let root = directory.path();
    let expected = command_request("git", &["diff", "--check"]);
    assert!(cockpit_repository::run_repository_verification(root, &expected).is_err());
    let foreign = command_request(
        "python3",
        &["-c", "open('foreign-marker', 'w').write('bad')"],
    );
    assert!(
        cockpit_repository::run_repository_verification_with_task_authorization(
            root,
            &foreign,
            &runtime(),
            Some(&caller()),
            |_node, _pid, _started| Ok(())
        )
        .is_err()
    );
    assert!(!root.join("foreign-marker").exists());
    let execution = cockpit_repository::run_repository_verification_with_task_authorization(
        root,
        &expected,
        &runtime(),
        Some(&caller()),
        |_node, _pid, _started| Ok(()),
    )
    .unwrap();
    assert!(execution.receipt.passed);
    assert_eq!(execution.receipt.nodes_executed, 1);
    assert_eq!(execution.receipt.execution_records.len(), 1);
    record_work_item_governance_controls_with_runtime(
        root,
        WI,
        &termination(root, "withdrawn"),
        &runtime(),
    )
    .unwrap();
    assert!(
        cockpit_repository::run_repository_verification_with_task_authorization(
            root,
            &expected,
            &runtime(),
            Some(&caller()),
            |_node, _pid, _started| Ok(())
        )
        .is_err()
    );
}

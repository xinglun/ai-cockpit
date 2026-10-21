//! Native domain fixtures use an explicit private test port, never a production source.
#![cfg(test)]
use crate as cockpit_repository;
use crate::review_authority::pipeline_tests::TestAuthority;
use cockpit_repository::WorkItemStartOptions;
use cockpit_repository::amend_work_item_contract;
use cockpit_repository::attach;
use cockpit_repository::checkpoint_work_item;
use cockpit_repository::preflight_work_item;
use cockpit_repository::read_work_item_contract_amendments;
use cockpit_repository::record_verification;
use cockpit_repository::require_verification_preconditions;
use cockpit_repository::start_work_item_with_options;
use serde_json::Value;
use serde_json::json;
use std::fs;
use std::path::Path;
use std::process::Command;
fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("repository tempdir");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=AI Cockpit Test",
                "-c",
                "user.email=ai-cockpit-test@example.invalid",
                "commit",
                "--allow-empty",
                "-m",
                "fixture baseline",
            ])
            .current_dir(directory.path())
            .status()
            .expect("git baseline commit")
            .success()
    );
    attach(directory.path()).expect("attach repository");
    directory
}

fn read_json(path: impl AsRef<Path>) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read JSON file")).expect("valid JSON")
}

fn record_human_preflight_review(
    authority: &TestAuthority,
    root: &Path,
    work_item_id: &str,
    contract_path: &Path,
) {
    crate::preflight_work_item(root, contract_path)
        .expect("fresh native preflight before exact fixture review");
    assert_eq!(
        fs::canonicalize(contract_path).unwrap(),
        fs::canonicalize(root.join(format!(
            ".ai/work-items/active/{work_item_id}.contract.json"
        )))
        .unwrap()
    );
    authority.approve(
        root,
        work_item_id,
        cockpit_protocol::review_authority::HumanReviewOperation::PreflightReview,
        &json!({"decision":"confirm_review"}),
    );
}

fn start_uncheckpointed(root: &Path, work_item_id: &str) -> std::path::PathBuf {
    start_work_item_with_options(
        root,
        work_item_id,
        "exercise recoverable Contract amendments",
        "preserve a reasoned audit trail across interrupted amendments",
        &["crates/cockpit-repository/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["amendment history is verifiable".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start Work Item");
    root.join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"))
}

fn start_checkpointed(root: &Path, work_item_id: &str) -> std::path::PathBuf {
    let contract_path = start_uncheckpointed(root, work_item_id);
    preflight_work_item(root, &contract_path).expect("preflight");
    checkpoint_work_item(root, work_item_id).expect("checkpoint");
    contract_path
}

fn typed_request(
    contract_path: &Path,
    change_id: &str,
    reason: &str,
    path: &str,
    operation: &str,
    value: Value,
) -> Value {
    let contract = read_json(contract_path);
    json!({
        "schemaVersion": 1,
        "changeId": change_id,
        "expectedContractDigest": cockpit_protocol::digest_json(&contract).expect("Contract digest"),
        "reason": reason,
        "changes": [{ "path": path, "operation": operation, "value": value }]
    })
}

#[test]
fn sensitive_amendment_requires_human_review_before_verification() {
    let authority = TestAuthority::install();
    let directory = repository();
    let root = directory.path();
    let work_item_id = "WI-SENSITIVE-AMENDMENT-REVIEW-GATE";
    let contract_path = start_checkpointed(root, work_item_id);
    let reason = "a scope change requires fresh human review before verification";
    let request = typed_request(
        &contract_path,
        "sensitive-scope-review-gate",
        reason,
        "/scope",
        "replace",
        json!(["crates/cockpit-repository/**", "crates/cockpit-protocol/**"]),
    );
    amend_work_item_contract(root, work_item_id, &request, reason)
        .expect("sensitive amendment is recorded");

    let decision = preflight_work_item(root, &contract_path).expect("preflight after amendment");
    assert_eq!(
        decision.review_state.as_deref(),
        Some("needs_human_confirmation")
    );
    assert!(decision.human_decision_request.is_some());

    let runtime = cockpit_protocol::RuntimeContext {
        runtime_version: "test-runtime".into(),
        protocol_version: 1,
        runtime_digest: cockpit_core::Digest::sha256_bytes(b"test-runtime"),
    };
    let snapshot = cockpit_git::GitRepository::discover(root)
        .expect("discover repository")
        .snapshot()
        .expect("capture repository snapshot");
    let error = require_verification_preconditions(root, work_item_id, &runtime, &snapshot)
        .expect_err("verification must wait for the required human review");
    assert!(
        error
            .to_string()
            .contains("contract_amendment_policy_review_required")
    );

    record_human_preflight_review(&authority, root, work_item_id, &contract_path);
    let reviewed = preflight_work_item(root, &contract_path).expect("preflight after review");
    assert_eq!(
        reviewed.review_state.as_deref(),
        Some("human_decision_recorded")
    );
    require_verification_preconditions(root, work_item_id, &runtime, &snapshot)
        .expect("a valid decision receipt satisfies the current verification preconditions");
}

#[test]
fn post_verification_amendment_receipt_lists_invalidated_required_checks() {
    let authority = TestAuthority::install();
    let directory = repository();
    let root = directory.path();
    let work_item_id = "WI-AMENDMENT-INVALIDATED-CHECKS";
    let contract_path = start_uncheckpointed(root, work_item_id);
    let policy_reason = "declare the required check before the initial checkpoint";
    let policy = typed_request(
        &contract_path,
        "declare-required-amendment-check",
        policy_reason,
        "/checkpointPolicy",
        "set",
        json!({
            "schemaVersion": 1,
            "profile": "standard",
            "requiredBeforeFinish": true,
            "requiredStages": ["before_edit", "before_finish"],
            "requiredChecks": ["requiredContractCheck"]
        }),
    );
    amend_work_item_contract(root, work_item_id, &policy, policy_reason)
        .expect("declare the required check");
    preflight_work_item(root, &contract_path).expect("preflight updated Contract");
    record_human_preflight_review(&authority, root, work_item_id, &contract_path);
    checkpoint_work_item(root, work_item_id).expect("checkpoint updated Contract");
    record_verification(
        root,
        work_item_id,
        &json!({"passed": true, "nodesPlanned": 1}),
        "test-runtime",
        &cockpit_core::Digest::sha256_bytes(b"test-runtime"),
    )
    .expect("record predecessor verification");

    let reason = "a plan change after verification invalidates its required check";
    let amendment = typed_request(
        &contract_path,
        "invalidate-required-check",
        reason,
        "/goal",
        "replace",
        json!("revised after required verification"),
    );
    amend_work_item_contract(root, work_item_id, &amendment, reason)
        .expect("record amendment after verification");
    let history =
        read_work_item_contract_amendments(root, work_item_id).expect("history remains valid");

    assert_eq!(history.len(), 2);
    assert_eq!(
        history[1].invalidated_required_checks,
        vec!["requiredContractCheck"]
    );
    assert_eq!(
        history[1].invalidated_evidence,
        vec![format!(".ai/evidence/{work_item_id}.verification.json")]
    );
}

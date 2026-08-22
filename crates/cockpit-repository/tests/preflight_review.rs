use cockpit_core::Digest;
use cockpit_repository::{
    WorkItemStartOptions, attach, checkpoint_work_item, finish_work_item, preflight_work_item,
    record_verification, record_work_item_governance_controls, start_work_item_with_options,
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
    start_work_item_with_options(
        directory.path(),
        "WI-PREFLIGHT",
        "review a bounded implementation",
        "record a human preflight receipt",
        &["crates/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            out_of_scope: vec!["target/**".into()],
            acceptance_criteria: vec!["bounded review remains explicit".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    preflight_work_item(
        directory.path(),
        &directory
            .path()
            .join(".ai/work-items/active/WI-PREFLIGHT.contract.json"),
    )
    .expect("preflight");
    directory
}

fn receipt(directory: &tempfile::TempDir) -> serde_json::Value {
    let summary: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/active/WI-PREFLIGHT.summary.json"),
        )
        .expect("summary"),
    )
    .expect("summary JSON");
    let contract: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/active/WI-PREFLIGHT.contract.json"),
        )
        .expect("contract"),
    )
    .expect("contract JSON");
    let contract_digest = cockpit_protocol::digest_json(&contract).expect("contract digest");
    serde_json::json!({
        "schemaVersion": 1,
        "decisionId": "contract-preflight-review",
        "decision": "confirm_review",
        "workItemId": "WI-PREFLIGHT",
        "repositoryId": cockpit_repository::repository_id(directory.path()),
        "contractDigest": contract_digest,
        "preflightDecisionDigest": summary["preflightDecisionDigest"].clone(),
        "repositorySnapshotDigest": summary["preflightRepositorySnapshotDigest"].clone(),
        "recordedAt": "2026-08-22T00:00:00Z",
        "recordedBy": "human:owner",
        "reason": "bounded implementation review confirmed"
    })
}

fn add_human_review_requirement(directory: &tempfile::TempDir) {
    let path = directory
        .path()
        .join(".ai/work-items/active/WI-PREFLIGHT.contract.json");
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    contract["agentCapability"] = serde_json::json!({
        "canImplement": true,
        "canVerify": true,
        "needsHumanDecision": true,
        "blockedReason": "review the bounded implementation path"
    });
    fs::write(path, serde_json::to_vec_pretty(&contract).unwrap()).unwrap();
}

#[test]
fn valid_preflight_decision_evidence_is_persisted_and_bound() {
    let directory = repository();
    let value = receipt(&directory);
    let summary = record_work_item_governance_controls(
        directory.path(),
        "WI-PREFLIGHT",
        &serde_json::json!({"decisionEvidence": value}),
    )
    .expect("valid receipt");
    assert_eq!(summary["decisionEvidence"]["decision"], "confirm_review");
    assert!(
        directory
            .path()
            .join(".ai/decisions/WI-PREFLIGHT.preflight-review.json")
            .is_file()
    );
}

#[test]
fn foreign_or_stale_preflight_decision_evidence_is_rejected_without_writes() {
    let directory = repository();
    let mut value = receipt(&directory);
    value["repositoryId"] =
        serde_json::Value::String(Digest::sha256_bytes(b"foreign-repository").to_string());
    let error = record_work_item_governance_controls(
        directory.path(),
        "WI-PREFLIGHT",
        &serde_json::json!({"decisionEvidence": value}),
    )
    .expect_err("foreign receipt must fail closed");
    assert!(error.to_string().contains("repository") || error.to_string().contains("identity"));
    assert!(
        !directory
            .path()
            .join(".ai/decisions/WI-PREFLIGHT.preflight-review.json")
            .exists()
    );
}

#[test]
fn bound_human_review_receipt_allows_checkpoint_but_not_stale_reuse() {
    let directory = repository();
    add_human_review_requirement(&directory);
    let contract = directory
        .path()
        .join(".ai/work-items/active/WI-PREFLIGHT.contract.json");
    let decision = preflight_work_item(directory.path(), &contract).expect("preflight review");
    assert_eq!(
        decision.review_state.as_deref(),
        Some("needs_human_confirmation")
    );

    record_work_item_governance_controls(
        directory.path(),
        "WI-PREFLIGHT",
        &serde_json::json!({"decisionEvidence": receipt(&directory)}),
    )
    .expect("record bound review");
    let confirmed = preflight_work_item(directory.path(), &contract).expect("re-preflight");
    assert_eq!(
        confirmed.review_state.as_deref(),
        Some("human_decision_recorded")
    );
    checkpoint_work_item(directory.path(), "WI-PREFLIGHT").expect("checkpoint after review");

    fs::write(directory.path().join("src.rs"), b"changed after review\n").unwrap();
    let stale = preflight_work_item(directory.path(), &contract).expect("stale preflight");
    assert_eq!(
        stale.review_state.as_deref(),
        Some("needs_human_confirmation")
    );
    assert!(checkpoint_work_item(directory.path(), "WI-PREFLIGHT").is_err());
}

#[test]
fn planned_required_scenario_is_implementation_ready_but_not_completion_evidence() {
    let directory = repository();
    let contract = directory
        .path()
        .join(".ai/work-items/active/WI-PREFLIGHT.contract.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract).unwrap()).unwrap();
    value["risk"] = serde_json::json!("high");
    value["operation"] = serde_json::json!("code");
    value["scenarioCoverage"] = serde_json::json!([{
        "scenario": "post-implementation integration check",
        "required": true,
        "status": "unverified",
        "expected": "the integration check passes after implementation",
        "verificationPlan": "run the focused integration test and retain its receipt",
        "evidence": []
    }]);
    fs::write(&contract, serde_json::to_vec_pretty(&value).unwrap()).unwrap();

    let decision = preflight_work_item(directory.path(), &contract).expect("preflight");
    assert_eq!(decision.state, cockpit_core::DecisionState::Green);
    assert!(
        !decision.unknowns.iter().any(|unknown| unknown
            == "required_scenario_unverified:post-implementation integration check")
    );
    checkpoint_work_item(directory.path(), "WI-PREFLIGHT").expect("planned scenario checkpoint");
    record_verification(
        directory.path(),
        "WI-PREFLIGHT",
        &serde_json::json!({"passed": true, "workItemId": "WI-PREFLIGHT"}),
        "test-runtime",
        &Digest::sha256_bytes(b"test-runtime"),
    )
    .expect("verification receipt");
    assert!(finish_work_item(directory.path(), "WI-PREFLIGHT").is_err());
}

#[cfg(unix)]
#[test]
fn existing_preflight_decision_symlink_is_never_replaced() {
    use std::os::unix::fs::symlink;

    let directory = repository();
    let decision_path = directory
        .path()
        .join(".ai/decisions/WI-PREFLIGHT.preflight-review.json");
    let target = directory.path().join("foreign.json");
    fs::write(&target, b"{}").unwrap();
    symlink(&target, &decision_path).unwrap();

    let error = record_work_item_governance_controls(
        directory.path(),
        "WI-PREFLIGHT",
        &serde_json::json!({"decisionEvidence": receipt(&directory)}),
    )
    .expect_err("symlink destination must fail closed");
    assert!(error.to_string().contains("already exists"));
    assert!(
        fs::symlink_metadata(&decision_path)
            .unwrap()
            .file_type()
            .is_symlink()
    );
}

#[test]
fn malformed_preflight_decision_timestamp_is_rejected() {
    let directory = repository();
    let mut value = receipt(&directory);
    value["recordedAt"] = serde_json::json!("not-a-timestamp");
    let error = record_work_item_governance_controls(
        directory.path(),
        "WI-PREFLIGHT",
        &serde_json::json!({"decisionEvidence": value}),
    )
    .expect_err("malformed timestamp must fail closed");
    assert!(error.to_string().contains("RFC3339"));
}

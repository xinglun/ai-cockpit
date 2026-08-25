use cockpit_protocol::Contract;
use cockpit_repository::{
    WorkItemStartOptions, attach, checkpoint_work_item, preflight_work_item,
    start_work_item_with_options, validate_agent_risk_controls,
    validate_checkpoint_evidence_bindings,
};
use serde_json::json;
use std::{fs, process::Command};

fn contract() -> Contract {
    serde_json::from_value(json!({
        "protocolVersion": 1,
        "contractVersion": 2,
        "repositoryId": "sha256:repo",
        "workItemId": "WI-RISK",
        "mode": "code",
        "title": "risk",
        "state": "implementation_active",
        "intent": {
            "businessGoal": "keep gates explicit",
            "userGoal": "stop unsafe execution",
            "problem": "missing gate evidence",
            "constraints": ["preserve history"],
            "nonGoals": [],
            "rationale": "bounded test"
        },
        "goal": "validate risk",
        "scope": ["src/**"],
        "outOfScope": ["global/**"],
        "risk": "high",
        "authority": "authorized",
        "acceptanceCriteria": ["A1: required gate is bound"],
        "requiredEvidenceClasses": ["verification"],
        "verification": [
            {"check": "aiWorkItem", "required": true},
            {"check": "aiScope", "required": true},
            {"check": "aiAgentRisk", "required": true},
            {"check": "aiSummary", "required": true},
            {"check": "aiStatus", "required": true},
            {"check": "aiStatusCheck", "required": true}
        ],
        "baseRevision": "abc",
        "projectProfileDigest": "sha256:profile",
        "repositorySnapshotDigest": "sha256:snapshot",
        "checkpointPolicy": {
            "schemaVersion": 1,
            "profile": "strict",
            "requiredBeforeFinish": true,
            "requiredStages": ["before_edit", "before_finish"],
            "requiredChecks": []
        },
        "unknowns": [],
        "notCodable": false,
        "agentCapability": {
            "canImplement": true,
            "canVerify": true,
            "needsHumanDecision": false
        },
        "executionDecision": {"status": "continue", "reason": "bounded"}
    }))
    .expect("contract")
}

fn summary() -> serde_json::Value {
    json!({
        "verification": [
            {"check": "aiWorkItem", "result": "passed"},
            {"check": "aiScope", "result": "passed"},
            {"check": "aiAgentRisk", "result": "passed"},
            {"check": "aiSummary", "result": "passed"},
            {"check": "aiStatus", "result": "passed"},
            {"check": "aiStatusCheck", "result": "passed"}
        ],
        "checkpointEvidence": [
            {
                "schemaVersion": 1,
                "repositoryId": "sha256:repo",
                "workItemId": "WI-RISK",
                "stage": "before_edit",
                "recorded": true,
                "contractHash": "contract-hash",
                "repositorySnapshotDigest": "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                "acceptanceCount": 1,
                "unknownCount": 0,
                "requiredChecks": 6,
                "requiredChecksPassed": 0,
                "recordedAt": "2026-08-25T00:00:00Z"
            },
            {
                "schemaVersion": 1,
                "repositoryId": "sha256:repo",
                "workItemId": "WI-RISK",
                "stage": "before_finish",
                "recorded": true,
                "contractHash": "contract-hash",
                "repositorySnapshotDigest": "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                "acceptanceCount": 1,
                "unknownCount": 0,
                "requiredChecks": 6,
                "requiredChecksPassed": 6,
                "recordedAt": "2026-08-25T00:00:01Z"
            }
        ]
    })
}

#[test]
fn valid_agent_risk_and_checkpoint_evidence_are_accepted() {
    let contract = contract();
    let summary = summary();
    let (state, unknowns, findings) = validate_agent_risk_controls(&contract, &summary);
    assert_eq!(state, "verified");
    assert!(unknowns.is_empty(), "{unknowns:?}");
    assert!(findings.is_empty(), "{findings:?}");
    validate_checkpoint_evidence_bindings(
        &contract,
        &summary,
        "sha256:repo",
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "contract-hash",
    )
    .expect("checkpoint evidence");
}

#[test]
fn missing_or_failed_required_gate_is_fail_closed() {
    let contract = contract();
    let mut summary = summary();
    summary["verification"][1]["result"] = json!("failed");
    let (state, _unknowns, findings) = validate_agent_risk_controls(&contract, &summary);
    assert_eq!(state, "blocked");
    assert!(
        findings
            .iter()
            .any(|finding| finding.code == "required_verification_failed")
    );
}

#[test]
fn checkpoint_identity_and_unknown_fields_fail_closed() {
    let contract = contract();
    let mut summary = summary();
    summary["checkpointEvidence"][0]["repositoryId"] = json!("sha256:foreign");
    summary["checkpointEvidence"][1]["untrusted"] = json!(true);
    let errors = validate_checkpoint_evidence_bindings(
        &contract,
        &summary,
        "sha256:repo",
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "contract-hash",
    )
    .expect_err("tampered checkpoint evidence");
    assert!(
        errors
            .iter()
            .any(|error| error.contains("repository_identity_mismatch"))
    );
    assert!(
        errors
            .iter()
            .any(|error| error == "checkpoint_evidence_malformed")
    );
}

#[test]
fn checkpoint_invalid_timestamp_fails_closed() {
    let contract = contract();
    let mut summary = summary();
    summary["checkpointEvidence"][0]["recordedAt"] = json!("not-a-timestamp");
    let errors = validate_checkpoint_evidence_bindings(
        &contract,
        &summary,
        "sha256:repo",
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "contract-hash",
    )
    .expect_err("invalid checkpoint timestamp");
    assert!(
        errors
            .iter()
            .any(|error| error == "checkpoint_evidence_recorded_at_invalid")
    );
}

#[test]
fn contract_amendment_chain_invalidates_stale_before_edit_evidence() {
    let contract = contract();
    let mut summary = summary();
    summary["checkpointEvidence"][0]["contractHash"] = json!("old-contract");
    summary["checkpointEvidence"][0]["acceptanceCount"] = json!(0);
    summary["checkpointEvidence"][1]["contractHash"] = json!("new-contract");
    summary["checkpointEvidence"].as_array_mut().unwrap().insert(
        1,
        json!({
            "schemaVersion": 1,
            "repositoryId": "sha256:repo",
            "workItemId": "WI-RISK",
            "stage": "contract_amendment_revalidation",
            "recorded": true,
            "contractHash": "new-contract",
            "repositorySnapshotDigest": "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "acceptanceCount": 1,
            "unknownCount": 0,
            "requiredChecks": 6,
            "requiredChecksPassed": 0,
            "originalBeforeEditContractHash": "old-contract",
            "previousContractHash": "old-contract",
            "reason": "bound the amended route",
            "verificationStarted": true,
            "invalidatedRequiredChecks": ["aiAgentRisk", "aiScope", "aiStatus", "aiStatusCheck", "aiSummary", "aiWorkItem"],
            "requiredChecksPassedAtAmendment": 6,
            "recordedAt": "2026-08-25T00:00:01Z"
        }),
    );
    validate_checkpoint_evidence_bindings(
        &contract,
        &summary,
        "sha256:repo",
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "new-contract",
    )
    .expect("valid amendment chain");
    summary["checkpointEvidence"][1]["previousContractHash"] = json!("forged");
    let errors = validate_checkpoint_evidence_bindings(
        &contract,
        &summary,
        "sha256:repo",
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "new-contract",
    )
    .expect_err("forged amendment chain");
    assert!(
        errors
            .iter()
            .any(|error| error == "checkpoint_evidence_amendment_chain_invalid")
    );
}

#[test]
fn checkpoint_lifecycle_records_before_edit_evidence_for_strict_policy() {
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
        "WI-CHECKPOINT",
        "checkpoint evidence",
        "record strict checkpoint evidence",
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            out_of_scope: vec!["target/**".into()],
            acceptance_criteria: vec!["checkpoint evidence is bound".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    let contract_path = directory
        .path()
        .join(".ai/work-items/active/WI-CHECKPOINT.contract.json");
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract")).expect("json");
    contract["checkpointPolicy"] = json!({
        "schemaVersion": 1,
        "profile": "strict",
        "requiredBeforeFinish": true,
        "requiredStages": ["before_edit", "before_finish"],
        "requiredChecks": []
    });
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract).expect("serialize"),
    )
    .expect("contract amendment");
    preflight_work_item(directory.path(), &contract_path).expect("preflight");
    checkpoint_work_item(directory.path(), "WI-CHECKPOINT").expect("checkpoint");
    let summary: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/active/WI-CHECKPOINT.summary.json"),
        )
        .expect("summary"),
    )
    .expect("summary json");
    assert_eq!(summary["checkpointEvidence"][0]["stage"], "before_edit");
    assert_eq!(summary["checkpointEvidence"][0]["requiredChecksPassed"], 0);
}

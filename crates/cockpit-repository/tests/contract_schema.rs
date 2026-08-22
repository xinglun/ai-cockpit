use cockpit_protocol::Contract;
use serde_json::json;

fn legacy_contract() -> serde_json::Value {
    json!({
        "protocolVersion": 1,
        "repositoryId": "sha256:repo",
        "workItemId": "WI-LEGACY",
        "intent": "keep old bytes readable",
        "goal": "legacy compatibility",
        "scope": ["src/**"],
        "outOfScope": [],
        "risk": "normal",
        "authority": "authorized",
        "acceptanceCriteria": ["legacy contract remains readable"],
        "requiredEvidenceClasses": [],
        "baseRevision": "legacy-base",
        "projectProfileDigest": "sha256:profile",
        "repositorySnapshotDigest": "sha256:snapshot"
    })
}

#[test]
fn repository_contract_schema_accepts_legacy_bytes_without_v2_lineage() {
    let value = legacy_contract();
    let contract: Contract = serde_json::from_value(value).expect("legacy contract parses");
    contract.validate().expect("legacy contract remains valid");
    assert!(contract.resume_history.is_empty());
    assert!(contract.synchronization_history.is_empty());
}

#[test]
fn repository_contract_schema_rejects_unknown_nested_governance_input() {
    let mut value = legacy_contract();
    value["restrictedWriteApproval"] = json!({
        "approved": true,
        "approvedBy": "maintainer",
        "reason": "bounded",
        "untrustedField": "must stop"
    });
    value["contractVersion"] = json!(2);
    value["mode"] = json!("code");
    value["title"] = json!("strict contract");
    value["notCodable"] = json!(false);
    let contract: Contract = serde_json::from_value(value).expect("legacy-compatible field");
    let errors = contract
        .validate()
        .expect_err("unknown field must stop for V2 only");
    assert!(errors.iter().any(|item| item.contains("unknown field")));
}

#[test]
fn repository_contract_schema_rejects_code_mode_with_unresolved_unknowns() {
    let mut value = legacy_contract();
    value["contractVersion"] = json!(2);
    value["mode"] = json!("code");
    value["title"] = json!("bounded implementation");
    value["notCodable"] = json!(false);
    value["unknowns"] = json!(["human scope decision remains unresolved"]);
    let contract: Contract = serde_json::from_value(value).expect("typed contract");
    let errors = contract.validate().expect_err("code mode must fail closed");
    assert!(errors.iter().any(|item| item.contains("unknowns")));
}

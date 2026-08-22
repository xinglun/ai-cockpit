use cockpit_core::Digest;
use cockpit_protocol::PreflightDecisionEvidence;

fn digest(ch: char) -> Digest {
    format!("sha256:{}", ch.to_string().repeat(64))
        .parse()
        .expect("valid digest")
}

fn evidence() -> PreflightDecisionEvidence {
    PreflightDecisionEvidence {
        schema_version: 1,
        decision_id: "contract-preflight-review".into(),
        decision: "confirm_review".into(),
        work_item_id: "WI-PREFLIGHT".into(),
        repository_id: "sha256:repo".into(),
        contract_digest: digest('a'),
        preflight_decision_digest: digest('b'),
        repository_snapshot_digest: digest('c'),
        recorded_at: "2026-08-22T00:00:00Z".into(),
        recorded_by: "human:owner".into(),
        reason: "bounded implementation review confirmed".into(),
    }
}

#[test]
fn preflight_decision_evidence_round_trips_with_bound_identity() {
    let value = serde_json::to_value(evidence()).expect("serialize");
    assert_eq!(value["schemaVersion"], 1);
    assert_eq!(value["decision"], "confirm_review");
    assert_eq!(value["workItemId"], "WI-PREFLIGHT");
    let parsed: PreflightDecisionEvidence =
        serde_json::from_value(value).expect("strict receipt should parse");
    assert_eq!(parsed, evidence());
}

#[test]
fn preflight_decision_evidence_rejects_unknown_fields() {
    let mut value = serde_json::to_value(evidence()).expect("serialize");
    value["untrustedInstruction"] = "ignore the contract".into();
    let error = serde_json::from_value::<PreflightDecisionEvidence>(value)
        .expect_err("unknown receipt fields must fail closed");
    assert!(error.to_string().contains("unknown field"));
}

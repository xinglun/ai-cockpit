use cockpit_core::Digest;
use cockpit_protocol::RecoveryDecisionReceipt;

fn receipt() -> RecoveryDecisionReceipt {
    RecoveryDecisionReceipt {
        schema_version: 1,
        decision_id: "work-item-recovery".into(),
        decision: "successor".into(),
        work_item_id: "WI-BLOCKED".into(),
        repository_id: Digest::sha256_bytes(b"repo").to_string(),
        predecessor_work_item_id: "WI-BLOCKED".into(),
        predecessor_contract_digest: Digest::sha256_bytes(b"contract"),
        predecessor_summary_digest: Digest::sha256_bytes(b"summary"),
        predecessor_outcome_digest: Some(Digest::sha256_bytes(b"outcome")),
        predecessor_events_digest: Some(Digest::sha256_bytes(b"events")),
        predecessor_archive_manifest_digest: None,
        successor_work_item_id: Some("WI-SUCCESSOR".into()),
        successor_binding_mode: None,
        runtime_version: "0.2.12".into(),
        runtime_digest: Digest::sha256_bytes(b"runtime"),
        actor: "human:owner".into(),
        authority_source: "repository-local".into(),
        reason: "bounded recovery".into(),
        evidence_refs: vec![".ai/evidence/WI-BLOCKED.verification.json".into()],
        policy_refs: vec![],
        decided_at: "2026-08-23T00:00:00Z".into(),
        resume_condition: "fresh verification evidence".into(),
    }
}

#[test]
fn recovery_receipt_round_trips() {
    let value = serde_json::to_value(receipt()).unwrap();
    let parsed: RecoveryDecisionReceipt = serde_json::from_value(value).unwrap();
    assert_eq!(parsed.decision, "successor");
    assert_eq!(parsed.predecessor_work_item_id, "WI-BLOCKED");
}

#[test]
fn recovery_receipt_rejects_unknown_fields() {
    let mut value = serde_json::to_value(receipt()).unwrap();
    value["secret"] = serde_json::json!("must not pass");
    assert!(serde_json::from_value::<RecoveryDecisionReceipt>(value).is_err());
}

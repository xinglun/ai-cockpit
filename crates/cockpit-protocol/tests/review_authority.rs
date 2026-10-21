use cockpit_core::Digest;
use cockpit_protocol::review_authority::*;

fn request() -> HumanReviewRequest {
    HumanReviewRequest {
        schema_version: 2,
        kind: HumanReviewKind::HumanReview,
        issuer: "protected-host-fixture".into(),
        request_id: "fixture-request-1".into(),
        nonce: "issuer-minted-nonce-fixture-1".into(),
        authority_epoch: 4,
        issued_at_millis: 1000,
        expires_at_millis: 2000,
        binding: HumanReviewBinding {
            repository_id: Digest::sha256_bytes(b"repo").to_string(),
            work_item_id: "WI-FIXTURE".into(),
            checkout_path: "/fixture".into(),
            contract_digest: Digest::sha256_bytes(b"C"),
            preflight_decision_digest: Some(Digest::sha256_bytes(b"P")),
            repository_snapshot_digest: Digest::sha256_bytes(b"R"),
            operation: HumanReviewOperation::PreflightReview,
            scope: vec!["crates/example.rs".into()],
            operation_digest: review_operation_digest(
                &HumanReviewOperation::PreflightReview,
                &serde_json::json!({"decision":"confirm_review"}),
            )
            .unwrap(),
            operation_payload: serde_json::json!({"decision":"confirm_review"}),
            operation_context: serde_json::Value::Null,
            local_hold_digest: None,
            activation_recovery_digest: None,
        },
    }
}

#[test]
fn every_binding_field_changes_the_request_digest() {
    let original = request();
    let digest = original.digest().unwrap();
    let mut cases = Vec::new();
    let mut changed = original.clone();
    changed.binding.repository_id = "foreign".into();
    cases.push(changed);
    let mut changed = original.clone();
    changed.binding.work_item_id = "WI-OTHER".into();
    cases.push(changed);
    let mut changed = original.clone();
    changed.binding.contract_digest = Digest::sha256_bytes(b"other-C");
    cases.push(changed);
    let mut changed = original.clone();
    changed.binding.preflight_decision_digest = Some(Digest::sha256_bytes(b"other-P"));
    cases.push(changed);
    let mut changed = original.clone();
    changed.binding.repository_snapshot_digest = Digest::sha256_bytes(b"other-R");
    cases.push(changed);
    let mut changed = original.clone();
    changed.binding.operation = HumanReviewOperation::Close;
    cases.push(changed);
    let mut changed = original.clone();
    changed.binding.scope.push("extra.rs".into());
    cases.push(changed);
    let mut changed = original.clone();
    changed.nonce.push('x');
    cases.push(changed);
    let mut changed = original.clone();
    changed.authority_epoch += 1;
    cases.push(changed);
    for field in [
        "checkoutPath",
        "operationPayload",
        "operationContext",
        "localHoldDigest",
        "activationRecoveryDigest",
    ] {
        let mut changed = serde_json::to_value(&original).unwrap();
        changed["binding"][field] = match field {
            "checkoutPath" => serde_json::json!("/different"),
            "localHoldDigest" | "activationRecoveryDigest" => {
                serde_json::json!(Digest::sha256_bytes(b"different"))
            }
            _ => serde_json::json!({"different":true}),
        };
        cases.push(serde_json::from_value(changed).unwrap());
    }
    for changed in cases {
        assert_ne!(digest, changed.digest().unwrap());
    }
}

#[test]
fn time_window_is_half_open_and_unknown_schema_is_rejected() {
    let mut value = request();
    assert!(value.validate_at(1000).is_ok());
    assert_eq!(
        value.validate_at(999).unwrap_err(),
        "review_request_not_yet_valid"
    );
    assert_eq!(
        value.validate_at(2000).unwrap_err(),
        "review_request_expired"
    );
    value.schema_version = 1;
    assert_eq!(
        value.validate_at(1500).unwrap_err(),
        "review_request_schema_unsupported"
    );
}

#[test]
fn technical_review_cannot_deserialize_as_human_approval() {
    let value = TechnicalReviewDecision {
        schema_version: 1,
        kind: TechnicalReviewKind::TechnicalReview,
        reviewer_actor: "agent:reviewer".into(),
        request_digest: request().digest().unwrap(),
        decision: TechnicalReviewChoice::Accepted,
        reason: "technical assessment only".into(),
    };
    assert!(
        serde_json::from_value::<HumanApprovalSubmission>(serde_json::to_value(value).unwrap())
            .is_err()
    );
}

#[test]
fn human_claim_fields_are_not_a_proof_submission_surface() {
    let mut value = serde_json::json!({
        "schemaVersion": 2, "kind": "human_approval_submission",
        "requestDigest": request().digest().unwrap(),
        "issuer": "host", "approvalId": "approval", "sourceEventId": "event",
        "proofHandle": "opaque-host-reference"
    });
    value["actor"] = "human:Ray".into();
    value["reason"] = "the user approved".into();
    assert!(serde_json::from_value::<HumanApprovalSubmission>(value).is_err());
}

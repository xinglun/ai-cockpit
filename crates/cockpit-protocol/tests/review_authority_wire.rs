use cockpit_core::Digest;
use cockpit_protocol::review_authority::*;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

// These are wire fixtures, not an authenticated authority or proof of issuance.
fn request() -> HumanReviewRequest {
    let operation = HumanReviewOperation::PreflightReview;
    let payload = json!({"decision": "confirm_review"});
    HumanReviewRequest {
        schema_version: 2,
        kind: HumanReviewKind::HumanReview,
        issuer: "fixture-host".into(),
        request_id: "fixture-request".into(),
        nonce: "fixture-nonce".into(),
        authority_epoch: 4,
        issued_at_millis: 1000,
        expires_at_millis: 2000,
        binding: HumanReviewBinding {
            repository_id: Digest::sha256_bytes(b"repo").to_string(),
            work_item_id: "WI-FIXTURE".into(),
            checkout_path: "/fixture".into(),
            contract_digest: Digest::sha256_bytes(b"contract"),
            preflight_decision_digest: Some(Digest::sha256_bytes(b"preflight")),
            repository_snapshot_digest: Digest::sha256_bytes(b"snapshot"),
            operation_digest: review_operation_digest(&operation, &payload).unwrap(),
            operation,
            scope: vec!["crates/example.rs".into()],
            operation_payload: payload,
            operation_context: json!({"plan": "fixture"}),
            local_hold_digest: None,
            activation_recovery_digest: None,
        },
    }
}

fn round_trip<T: Serialize + DeserializeOwned>(value: &T) {
    let wire = serde_json::to_string(value).unwrap();
    let decoded: T = serde_json::from_str(&wire).unwrap();
    assert_eq!(
        serde_json::to_value(decoded).unwrap(),
        serde_json::to_value(value).unwrap()
    );
}

fn rejects_extra_and_duplicate<T: Serialize + DeserializeOwned>(value: &T, field: &str) {
    let mut extra = serde_json::to_value(value).unwrap();
    extra["unrecognizedWireField"] = json!(true);
    assert!(serde_json::from_str::<T>(&extra.to_string()).is_err());
    let wire = serde_json::to_string(value).unwrap();
    let original = serde_json::to_value(value).unwrap();
    // Feed original duplicate bytes directly to the typed deserializer.
    let duplicated = format!("{{{field:?}:{},{}", original[field], &wire[1..]);
    let error = serde_json::from_str::<T>(&duplicated)
        .err()
        .expect("duplicate must reject");
    assert!(error.to_string().contains("duplicate field"), "{error}");
}

#[test]
fn request_and_binding_round_trip_keep_camel_case_and_schema() {
    let value = request();
    round_trip(&value);
    round_trip(&value.binding);
    let wire = serde_json::to_value(&value).unwrap();
    assert_eq!(wire["schemaVersion"], 2);
    assert_eq!(wire["kind"], "human_review");
    assert_eq!(wire["binding"]["operation"], "preflight_review");
    assert_eq!(
        wire["binding"]["operationPayload"]["decision"],
        "confirm_review"
    );
    assert!(wire.get("schema_version").is_none());
}

#[test]
fn all_operation_and_authority_state_wire_labels_round_trip() {
    for label in [
        "preflight_review",
        "close",
        "recovery",
        "restricted_write",
        "retire",
        "profile_confirmation",
        "cross_checkout_recovery",
        "activation_recovery",
        "selected_lineage_recovery",
        "archived_verification_recovery",
        "legacy_source_registration",
        "legacy_finalization",
        "historical_finalization_recovery",
        "material_review",
    ] {
        let operation: HumanReviewOperation = serde_json::from_value(json!(label)).unwrap();
        assert_eq!(serde_json::to_value(&operation).unwrap(), json!(label));
        round_trip(&HumanOperationSelection {
            operation,
            payload: json!({"fixture": true}),
        });
    }
    for label in [
        "authority_unavailable",
        "missing",
        "unknown_provenance",
        "invalid",
        "revoked",
        "local_hold",
        "consumed",
        "publication_incomplete",
        "active",
        "denied",
    ] {
        let state: CurrentReviewAuthorityState = serde_json::from_value(json!(label)).unwrap();
        assert_eq!(serde_json::to_value(state).unwrap(), json!(label));
    }
    assert!(
        serde_json::from_value::<HumanReviewOperation>(json!("administrator_override")).is_err()
    );
    assert!(
        serde_json::from_value::<CurrentReviewAuthorityState>(json!("trusted_by_agent")).is_err()
    );
}

#[test]
fn operation_digest_uses_exact_domain_operation_and_payload() {
    let operation = HumanReviewOperation::Close;
    let payload = json!({"sequence": 0, "digest": "fixture"});
    let actual = review_operation_digest(&operation, &payload).unwrap();
    assert_eq!(
        actual,
        cockpit_protocol::digest_json(&json!({
            "domain": "ai-cockpit/human-operation/v1", "operation": "close", "payload": payload,
        }))
        .unwrap()
    );
    assert_ne!(actual, cockpit_protocol::digest_json(&payload).unwrap());
    assert_ne!(
        actual,
        review_operation_digest(&HumanReviewOperation::Recovery, &payload).unwrap()
    );
    assert_ne!(
        actual,
        review_operation_digest(&operation, &json!({"sequence": 1, "digest": "fixture"})).unwrap()
    );
}

#[test]
fn request_digest_binds_every_request_and_binding_field() {
    let value = request();
    let original = serde_json::to_value(&value).unwrap();
    let digest = value.digest().unwrap();
    assert_eq!(
        digest,
        cockpit_protocol::digest_json(&json!({
            "domain": "ai-cockpit/human-review-request/v2", "request": original,
        }))
        .unwrap()
    );
    for (field, replacement) in [
        ("schemaVersion", json!(3)),
        ("issuer", json!("other")),
        ("requestId", json!("other")),
        ("nonce", json!("other")),
        ("authorityEpoch", json!(5)),
        ("issuedAtMillis", json!(1001)),
        ("expiresAtMillis", json!(2001)),
    ] {
        let mut changed = original.clone();
        changed[field] = replacement;
        assert_ne!(
            digest,
            serde_json::from_value::<HumanReviewRequest>(changed)
                .unwrap()
                .digest()
                .unwrap(),
            "{field}"
        );
    }
    for (field, replacement) in [
        ("repositoryId", json!("other")),
        ("workItemId", json!("WI-OTHER")),
        ("checkoutPath", json!("/other")),
        ("contractDigest", json!(Digest::sha256_bytes(b"other"))),
        ("preflightDecisionDigest", Value::Null),
        (
            "repositorySnapshotDigest",
            json!(Digest::sha256_bytes(b"other")),
        ),
        ("operation", json!("close")),
        ("scope", json!(["other.rs"])),
        ("operationDigest", json!(Digest::sha256_bytes(b"other"))),
        ("operationPayload", json!({"decision": "other"})),
        ("operationContext", json!({"plan": "other"})),
        ("localHoldDigest", json!(Digest::sha256_bytes(b"hold"))),
        (
            "activationRecoveryDigest",
            json!(Digest::sha256_bytes(b"recovery")),
        ),
    ] {
        let mut changed = original.clone();
        changed["binding"][field] = replacement;
        assert_ne!(
            digest,
            serde_json::from_value::<HumanReviewRequest>(changed)
                .unwrap()
                .digest()
                .unwrap(),
            "{field}"
        );
    }
}

#[test]
fn payload_or_operation_tampering_rejects_before_shape_validation() {
    let mut value = request();
    value.binding.operation_payload = json!({"decision": "other"});
    assert_eq!(
        value.validate_at(1500),
        Err("review_operation_digest_mismatch")
    );
    let mut value = request();
    value.binding.operation = HumanReviewOperation::Close;
    assert_eq!(
        value.validate_at(1500),
        Err("review_operation_digest_mismatch")
    );
}

#[test]
fn schema_and_preflight_provenance_reject_with_specific_diagnostics() {
    let mut value = request();
    value.schema_version = 1;
    assert_eq!(
        value.validate_at(1500),
        Err("review_request_schema_unsupported")
    );
    let mut value = request();
    value.binding.preflight_decision_digest = None;
    assert_eq!(
        value.validate_at(1500),
        Err("review_preflight_provenance_missing")
    );
    value.binding.operation = HumanReviewOperation::Close;
    value.binding.operation_digest =
        review_operation_digest(&value.binding.operation, &value.binding.operation_payload)
            .unwrap();
    assert_eq!(value.validate_at(1500), Ok(()));
}

#[test]
fn blank_identifiers_scope_and_zero_epoch_reject() {
    let original = serde_json::to_value(request()).unwrap();
    for pointer in [
        "/issuer",
        "/requestId",
        "/nonce",
        "/binding/repositoryId",
        "/binding/workItemId",
        "/binding/checkoutPath",
    ] {
        for blank in ["", " \t\n"] {
            let mut value = original.clone();
            *value.pointer_mut(pointer).unwrap() = json!(blank);
            let value: HumanReviewRequest = serde_json::from_value(value).unwrap();
            assert_eq!(
                value.validate_at(1500),
                Err("review_request_invalid"),
                "{pointer}"
            );
        }
    }
    for scope in [
        vec![],
        vec!["".to_owned()],
        vec!["ok.rs".to_owned(), " ".to_owned()],
    ] {
        let mut value = request();
        value.binding.scope = scope;
        assert_eq!(value.validate_at(1500), Err("review_request_invalid"));
    }
    let mut value = request();
    value.authority_epoch = 0;
    assert_eq!(value.validate_at(1500), Err("review_request_invalid"));
}

#[test]
fn time_window_is_half_open_with_invalid_and_extreme_boundaries() {
    let value = request();
    assert_eq!(value.validate_at(999), Err("review_request_not_yet_valid"));
    assert_eq!(value.validate_at(1000), Ok(()));
    assert_eq!(value.validate_at(1999), Ok(()));
    assert_eq!(value.validate_at(2000), Err("review_request_expired"));
    for (issued, expires) in [(-1, 2000), (1000, 1000), (1000, 999)] {
        let mut value = request();
        value.issued_at_millis = issued;
        value.expires_at_millis = expires;
        assert_eq!(value.validate_at(1500), Err("review_request_invalid"));
    }
    let mut value = request();
    value.issued_at_millis = 0;
    value.expires_at_millis = i64::MAX;
    assert_eq!(value.validate_at(-1), Err("review_request_not_yet_valid"));
    assert_eq!(value.validate_at(0), Ok(()));
    assert_eq!(value.validate_at(i64::MAX - 1), Ok(()));
    assert_eq!(value.validate_at(i64::MAX), Err("review_request_expired"));
}

#[test]
fn direct_typed_request_and_binding_reject_duplicate_and_unknown_fields() {
    let value = request();
    rejects_extra_and_duplicate(&value, "schemaVersion");
    rejects_extra_and_duplicate(&value.binding, "repositoryId");
    let wire = serde_json::to_string(&value).unwrap();
    assert!(serde_json::from_str::<HumanReviewRequest>(&wire[..wire.len() - 1]).is_err());
    let mut malformed = serde_json::to_value(&value).unwrap();
    malformed["authorityEpoch"] = json!(-1);
    assert!(serde_json::from_value::<HumanReviewRequest>(malformed).is_err());
}

#[test]
fn submission_and_technical_review_stay_distinct_typed_records() {
    let submission = HumanApprovalSubmission {
        schema_version: 2,
        kind: HumanApprovalSubmissionKind::HumanApprovalSubmission,
        request_digest: request().digest().unwrap(),
        issuer: "fixture-host".into(),
        approval_id: "approval".into(),
        source_event_id: "event".into(),
        proof_handle: "opaque-reference".into(),
    };
    round_trip(&submission);
    rejects_extra_and_duplicate(&submission, "approvalId");
    for choice in [
        TechnicalReviewChoice::Accepted,
        TechnicalReviewChoice::ChangesRequired,
    ] {
        let technical = TechnicalReviewDecision {
            schema_version: 1,
            kind: TechnicalReviewKind::TechnicalReview,
            reviewer_actor: "agent:fixture-reviewer".into(),
            request_digest: request().digest().unwrap(),
            decision: choice,
            reason: "technical assessment only".into(),
        };
        round_trip(&technical);
        rejects_extra_and_duplicate(&technical, "reviewerActor");
        assert!(
            serde_json::from_value::<HumanApprovalSubmission>(
                serde_json::to_value(technical).unwrap()
            )
            .is_err()
        );
    }
    for field in [
        "actor",
        "authenticatedSubject",
        "decisionTime",
        "sourceBytes",
        "assurance",
        "authorityConfiguration",
    ] {
        let mut value = serde_json::to_value(&submission).unwrap();
        value[field] = json!("agent-supplied-claim");
        assert!(
            serde_json::from_value::<HumanApprovalSubmission>(value).is_err(),
            "{field}"
        );
    }
}

#[test]
fn approval_and_revocation_mirrors_round_trip_without_authenticating_claims() {
    let approval = HumanApprovalReceipt {
        schema_version: 2,
        request: request(),
        request_digest: request().digest().unwrap(),
        approval_id: "approval".into(),
        source_event_id: "event".into(),
        authenticated_subject: "fixture-claim-only".into(),
        authority_epoch: 4,
        resolved_local_hold: Some(Digest::sha256_bytes(b"hold")),
    };
    round_trip(&approval);
    rejects_extra_and_duplicate(&approval, "approvalId");
    let revocation = HumanRevocationReceipt {
        schema_version: 2,
        repository_id: "repo".into(),
        work_item_id: "WI-FIXTURE".into(),
        approval_id: "approval".into(),
        revocation_id: "revocation".into(),
        source_event_id: "event".into(),
        authenticated_subject: "fixture-claim-only".into(),
        authority_epoch: 4,
    };
    round_trip(&revocation);
    rejects_extra_and_duplicate(&revocation, "revocationId");
}

#[test]
fn correction_revocation_and_selection_keep_distinct_wire_shapes() {
    let correction = ReviewCorrectionRequest {
        schema_version: 1,
        repository_id: "repo".into(),
        work_item_id: "WI-FIXTURE".into(),
        target_receipt_raw_digest: Digest::sha256_bytes(b"receipt"),
        reason: "fixture correction".into(),
    };
    round_trip(&correction);
    rejects_extra_and_duplicate(&correction, "reason");
    let event = ReviewCorrectionEvent {
        schema_version: 1,
        sequence: 0,
        parent_digest: None,
        request_digest: Digest::sha256_bytes(b"request"),
        request: correction.clone(),
        recorded_at: "fixture-time".into(),
        recorded_by: "agent:fixture".into(),
        assurance: "claim-only".into(),
        human_authority_granted: false,
    };
    round_trip(&event);
    rejects_extra_and_duplicate(&event, "sequence");
    let revocation = ReviewRevocationRequest {
        schema_version: 1,
        repository_id: "repo".into(),
        work_item_id: "WI-FIXTURE".into(),
        approval_id: "approval".into(),
        proof_handle: "opaque-reference".into(),
    };
    round_trip(&revocation);
    rejects_extra_and_duplicate(&revocation, "proofHandle");
    assert!(
        serde_json::from_value::<ReviewRevocationRequest>(
            serde_json::to_value(correction).unwrap()
        )
        .is_err()
    );
    let selection = HumanOperationSelection {
        operation: HumanReviewOperation::Close,
        payload: json!({"sequence": 0}),
    };
    round_trip(&selection);
    rejects_extra_and_duplicate(&selection, "operation");
    let mut injected = serde_json::to_value(selection).unwrap();
    injected["authorityImplementation"] = json!("agent-provided");
    assert!(serde_json::from_value::<HumanOperationSelection>(injected).is_err());
}

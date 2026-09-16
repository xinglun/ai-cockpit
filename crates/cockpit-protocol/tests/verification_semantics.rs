use cockpit_protocol::{
    ApprovalMode, DEFAULT_VERIFICATION_TIMEOUT_SECONDS, EvidenceAssurance, GovernancePolicy,
    MAX_VERIFICATION_TIMEOUT_SECONDS, PolicyLayer, PolicyRule,
    VERIFICATION_SEMANTICS_SCHEMA_VERSION, VerificationRequirement, VerificationStage,
    VerificationTier, merge_policy_layers, validate_policy_overlay,
};
use serde_json::json;

fn requirement() -> VerificationRequirement {
    VerificationRequirement {
        schema_version: VERIFICATION_SEMANTICS_SCHEMA_VERSION,
        required_tier: VerificationTier::T3,
        required_assurance: EvidenceAssurance::RepositoryVerified,
        policy_refs: vec!["release-policy:v1".into()],
        stage_refs: vec!["release".into()],
        gate_refs: vec!["protected-gate:hosted-ci".into()],
        reason: "release requires authoritative verification".into(),
        max_timeout_seconds: None,
    }
}

#[test]
fn timeout_policy_is_finite_and_serialized_as_an_authorization_ceiling() {
    let mut bounded = requirement();
    bounded.max_timeout_seconds = Some(MAX_VERIFICATION_TIMEOUT_SECONDS);
    let value = serde_json::to_value(&bounded).expect("serialize");
    assert_eq!(
        value["maxTimeoutSeconds"],
        json!(MAX_VERIFICATION_TIMEOUT_SECONDS)
    );
    assert_eq!(
        serde_json::from_value::<VerificationRequirement>(value).expect("round trip"),
        bounded
    );

    let mut zero = bounded.clone();
    zero.max_timeout_seconds = Some(0);
    assert!(zero.validate().is_err());

    let mut above_cap = bounded;
    above_cap.max_timeout_seconds = Some(MAX_VERIFICATION_TIMEOUT_SECONDS + 1);
    assert!(above_cap.validate().is_err());
    assert_eq!(DEFAULT_VERIFICATION_TIMEOUT_SECONDS, 300);
}

#[test]
fn timeout_policy_layers_only_preserve_or_reduce_the_authorized_ceiling() {
    let parent = GovernancePolicy {
        policy_id: "project".into(),
        layer: PolicyLayer::Project,
        rules: vec![PolicyRule {
            operation: "modify_source".into(),
            approval_mode: ApprovalMode::NoHumanApprovalForLowRisk,
            required_evidence: Vec::new(),
            verification_requirement: Some(VerificationRequirement {
                max_timeout_seconds: Some(600),
                ..requirement()
            }),
        }],
    };
    let weaker = GovernancePolicy {
        policy_id: "work-item-weaker".into(),
        layer: PolicyLayer::WorkItem,
        rules: vec![PolicyRule {
            verification_requirement: Some(VerificationRequirement {
                max_timeout_seconds: Some(900),
                ..requirement()
            }),
            ..parent.rules[0].clone()
        }],
    };
    assert!(validate_policy_overlay(&parent, &weaker).is_err());

    let stricter = GovernancePolicy {
        policy_id: "work-item-stricter".into(),
        layer: PolicyLayer::WorkItem,
        rules: vec![PolicyRule {
            verification_requirement: Some(VerificationRequirement {
                max_timeout_seconds: Some(300),
                ..requirement()
            }),
            ..parent.rules[0].clone()
        }],
    };
    let merged = merge_policy_layers(&[&parent, &stricter]).expect("merge policy layers");
    assert_eq!(
        merged.rules[0]
            .verification_requirement
            .as_ref()
            .expect("requirement")
            .max_timeout_seconds,
        Some(300)
    );
}

#[test]
fn tier_and_assurance_are_orthogonal_and_strictly_serialized() {
    let value = serde_json::to_value(requirement()).expect("serialize");
    assert_eq!(value["requiredTier"], json!("T3"));
    assert_eq!(value["requiredAssurance"], json!("repository_verified"));
    let decoded: VerificationRequirement =
        serde_json::from_value(value.clone()).expect("round trip");
    assert_eq!(decoded, requirement());

    // T3 does not imply provider or enterprise assurance.  A self-declared
    // result remains unsatisfied and is never relabeled by the tier.
    assert!(!decoded.is_satisfied_by(VerificationTier::T3, EvidenceAssurance::SelfDeclared));
    assert!(decoded.is_satisfied_by(VerificationTier::T3, EvidenceAssurance::RepositoryVerified));
    assert!(!decoded.is_satisfied_by(VerificationTier::T2, EvidenceAssurance::EnterpriseVerified));
}

#[test]
fn requirement_validation_needs_traceable_references_and_reason() {
    requirement().validate().expect("valid requirement");

    let mut missing_refs = requirement();
    missing_refs.policy_refs.clear();
    missing_refs.stage_refs.clear();
    missing_refs.gate_refs.clear();
    assert!(missing_refs.validate().is_err());

    let mut duplicate = requirement();
    duplicate.gate_refs = vec!["release-policy:v1".into()];
    assert!(duplicate.validate().is_err());

    let mut unsupported = requirement();
    unsupported.schema_version = 2;
    assert!(unsupported.validate().is_err());
}

#[test]
fn unknown_fields_and_invalid_enum_values_fail_closed() {
    let mut value = serde_json::to_value(requirement()).expect("serialize");
    value["futureTier"] = json!("T4");
    assert!(serde_json::from_value::<VerificationRequirement>(value).is_err());

    let mut invalid_tier = serde_json::to_value(requirement()).expect("serialize");
    invalid_tier["requiredTier"] = json!("provider_verified");
    assert!(serde_json::from_value::<VerificationRequirement>(invalid_tier).is_err());

    let mut missing_schema = serde_json::to_value(requirement()).expect("serialize");
    missing_schema
        .as_object_mut()
        .expect("object")
        .remove("schemaVersion");
    assert!(serde_json::from_value::<VerificationRequirement>(missing_schema).is_err());
}

#[test]
fn verification_stage_is_typed_and_pre_ci_is_distinct() {
    for (wire, expected) in [
        ("task", VerificationStage::Task),
        ("pre_ci", VerificationStage::PreCi),
        ("pr", VerificationStage::PullRequest),
        ("merge", VerificationStage::Merge),
        ("release", VerificationStage::Release),
    ] {
        assert_eq!(
            VerificationStage::parse(wire).expect("known stage"),
            expected
        );
        assert_eq!(expected.as_str(), wire);
    }
    assert!(VerificationStage::parse("ci").is_err());
    assert!(serde_json::from_value::<VerificationStage>(json!("ci")).is_err());
}

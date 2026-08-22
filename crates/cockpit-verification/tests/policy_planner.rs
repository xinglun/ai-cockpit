use cockpit_protocol::{
    ApprovalMode, EvidenceAssurance, GovernancePolicy, PolicyLayer, PolicyRule,
    VerificationRequirement, VerificationTier,
};
use cockpit_verification::{PolicyPlannerError, PolicyPlannerInput, plan_policy_requirement};

fn requirement(
    policy: &str,
    tier: VerificationTier,
    assurance: EvidenceAssurance,
) -> VerificationRequirement {
    VerificationRequirement {
        schema_version: 1,
        required_tier: tier,
        required_assurance: assurance,
        policy_refs: vec![policy.into()],
        stage_refs: vec!["release".into()],
        gate_refs: vec!["protected:hosted-ci".into()],
        reason: format!("{policy} release requirement"),
    }
}

fn policy(id: &str, layer: PolicyLayer, requirement: VerificationRequirement) -> GovernancePolicy {
    GovernancePolicy {
        policy_id: id.into(),
        layer,
        rules: vec![PolicyRule {
            operation: "release".into(),
            approval_mode: ApprovalMode::SingleAuthorizedHuman,
            required_evidence: vec!["hosted_ci".into()],
            verification_requirement: Some(requirement),
        }],
    }
}

#[test]
fn planner_merges_tier_and_assurance_independently_with_traceability() {
    let organization = policy(
        "org-release-v1",
        PolicyLayer::Organization,
        requirement(
            "org-release-v1",
            VerificationTier::T3,
            EvidenceAssurance::RepositoryVerified,
        ),
    );
    let project = policy(
        "project-release-v2",
        PolicyLayer::Project,
        requirement(
            "project-release-v2",
            VerificationTier::T3,
            EvidenceAssurance::ProviderVerified,
        ),
    );
    let plan = plan_policy_requirement(&PolicyPlannerInput {
        operation: "release".into(),
        stage: "release".into(),
        protected_gate: Some("protected:hosted-ci".into()),
        policies: vec![organization, project],
    })
    .expect("policy plan");
    assert_eq!(plan.requirement.required_tier, VerificationTier::T3);
    assert_eq!(
        plan.requirement.required_assurance,
        EvidenceAssurance::ProviderVerified
    );
    assert_eq!(
        plan.source_policy_ids,
        vec!["org-release-v1", "project-release-v2"]
    );
    assert!(
        plan.requirement
            .policy_refs
            .contains(&"org-release-v1".into())
    );
    assert!(
        plan.requirement
            .policy_refs
            .contains(&"project-release-v2".into())
    );
}

#[test]
fn planner_does_not_infer_t3_without_an_explicit_policy_requirement() {
    let policy = GovernancePolicy {
        policy_id: "org-release-v1".into(),
        layer: PolicyLayer::Organization,
        rules: vec![PolicyRule {
            operation: "release".into(),
            approval_mode: ApprovalMode::SingleAuthorizedHuman,
            required_evidence: vec!["hosted_ci".into()],
            verification_requirement: None,
        }],
    };
    let error = plan_policy_requirement(&PolicyPlannerInput {
        operation: "release".into(),
        stage: "release".into(),
        protected_gate: Some("protected:hosted-ci".into()),
        policies: vec![policy],
    })
    .expect_err("missing requirement must fail closed");
    assert!(matches!(
        error,
        PolicyPlannerError::VerificationRequirementMissing(_, _)
    ));
}

#[test]
fn planner_rejects_untraceable_stage_or_protected_gate() {
    let mut requirement = requirement(
        "org-release-v1",
        VerificationTier::T3,
        EvidenceAssurance::RepositoryVerified,
    );
    requirement.stage_refs = vec!["pull_request".into()];
    let error = plan_policy_requirement(&PolicyPlannerInput {
        operation: "release".into(),
        stage: "release".into(),
        protected_gate: Some("protected:hosted-ci".into()),
        policies: vec![policy(
            "org-release-v1",
            PolicyLayer::Organization,
            requirement,
        )],
    })
    .expect_err("stage must be traceable");
    assert!(matches!(
        error,
        PolicyPlannerError::StageReferenceMissing(_, _)
    ));
}

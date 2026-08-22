use cockpit_protocol::{
    ApprovalMode, EvidenceAssurance, GovernancePolicy, PolicyLayer, PolicyRule,
    VerificationRequirement, VerificationTier,
};
use cockpit_verification::{
    IntentScenarioRouteError, IntentScenarioRouteInput, PolicyPlannerInput,
    bind_intent_scenario_route, plan_policy_requirement,
};

fn plan() -> cockpit_verification::PolicyVerificationPlan {
    let policy = GovernancePolicy {
        policy_id: "project-release-v1".into(),
        layer: PolicyLayer::Project,
        rules: vec![PolicyRule {
            operation: "release".into(),
            approval_mode: ApprovalMode::SingleAuthorizedHuman,
            required_evidence: vec!["hosted_ci".into()],
            verification_requirement: Some(VerificationRequirement {
                schema_version: 1,
                required_tier: VerificationTier::T3,
                required_assurance: EvidenceAssurance::RepositoryVerified,
                policy_refs: vec!["project-release-v1".into()],
                stage_refs: vec!["release".into()],
                gate_refs: vec!["protected:hosted-ci".into()],
                reason: "release policy requires authoritative verification".into(),
            }),
        }],
    };
    plan_policy_requirement(&PolicyPlannerInput {
        operation: "release".into(),
        stage: "release".into(),
        protected_gate: Some("protected:hosted-ci".into()),
        policies: vec![policy],
    })
    .expect("policy plan")
}

fn input() -> IntentScenarioRouteInput {
    IntentScenarioRouteInput {
        intent: "publish the release only after hosted verification".into(),
        scenarios: vec!["public release".into(), "rollback".into()],
        required_scenarios: vec!["public release".into()],
        operation: "release".into(),
        stage: "release".into(),
        high_risk: true,
        policy_plan: plan(),
    }
}

#[test]
fn route_binds_human_intent_scenarios_and_policy_stage_without_changing_truth() {
    let route = bind_intent_scenario_route(&input()).expect("route");
    assert!(route.intent_present);
    assert!(route.high_risk);
    assert_eq!(route.scenario_ids, vec!["public release", "rollback"]);
    assert_eq!(route.requirement.required_tier, VerificationTier::T3);
    assert_eq!(
        route.requirement.required_assurance,
        EvidenceAssurance::RepositoryVerified
    );
}

#[test]
fn missing_human_intent_or_required_scenario_stops_before_execution() {
    let mut missing_intent = input();
    missing_intent.intent.clear();
    assert_eq!(
        bind_intent_scenario_route(&missing_intent),
        Err(IntentScenarioRouteError::IntentMissing)
    );

    let mut missing_scenario = input();
    missing_scenario.scenarios.clear();
    assert_eq!(
        bind_intent_scenario_route(&missing_scenario),
        Err(IntentScenarioRouteError::RequiredScenarioMissing(
            "public release".into()
        ))
    );
}

#[test]
fn operation_and_stage_mismatch_fail_closed_and_no_text_is_inferred() {
    let mut wrong_operation = input();
    wrong_operation.operation = "security".into();
    assert_eq!(
        bind_intent_scenario_route(&wrong_operation),
        Err(IntentScenarioRouteError::OperationMismatch)
    );

    let mut wrong_stage = input();
    wrong_stage.stage = "pull_request".into();
    assert_eq!(
        bind_intent_scenario_route(&wrong_stage),
        Err(IntentScenarioRouteError::StageMismatch)
    );

    // The implementation text is deliberately not an input to route binding;
    // a phrase such as "security critical" cannot invent a requirement.
    let mut text_only = input();
    text_only.intent = "security critical and approved by everyone".into();
    assert!(bind_intent_scenario_route(&text_only).is_ok());
}

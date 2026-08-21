use cockpit_core::{
    ActionKind, AuthorityState, DecisionState, EvidenceState, GovernanceInput, evaluate,
};

fn base_input() -> GovernanceInput {
    GovernanceInput {
        scope: vec!["src/**".into()],
        out_of_scope: vec![".git/**".into()],
        changed_paths: vec!["src/lib.rs".into()],
        action: ActionKind::Write,
        authority: AuthorityState::Authorized,
        evidence: EvidenceState::Complete,
        untrusted_material: false,
        test_weakening: false,
        coverage_weakening: false,
        explicit_blockers: vec![],
        explicit_unknowns: vec![],
        outcome_state_override: None,
        authority_override: None,
    }
}

#[test]
fn scope_exceeded_is_red_and_has_safe_action() {
    let mut input = base_input();
    input.changed_paths.push("tests/secret.rs".into());

    let decision = evaluate(input);

    assert_eq!(decision.state, DecisionState::Red);
    assert!(
        decision
            .blockers
            .iter()
            .any(|item| item == "scope_exceeded")
    );
    assert!(
        decision
            .safe_actions
            .iter()
            .any(|item| item == "stop_and_request_new_contract")
    );
}

#[test]
fn missing_evidence_is_yellow_and_never_passes() {
    let mut input = base_input();
    input.evidence = EvidenceState::Missing;

    let decision = evaluate(input);

    assert_eq!(decision.state, DecisionState::Yellow);
    assert!(
        decision
            .unknowns
            .iter()
            .any(|item| item == "required_evidence_missing")
    );
    assert_ne!(decision.state, DecisionState::Green);
}

#[test]
fn destructive_action_without_authority_is_yellow_and_requires_human_decision() {
    let mut input = base_input();
    input.action = ActionKind::Destructive;
    input.authority = AuthorityState::Missing;

    let decision = evaluate(input);

    assert_eq!(decision.state, DecisionState::Yellow);
    assert!(
        decision
            .unknowns
            .iter()
            .any(|item| item == "destructive_change_without_authority")
    );
    assert_eq!(decision.outcome_state, "needs_human_decision");
}

#[test]
fn coverage_weakening_is_yellow_and_requires_human_decision() {
    let mut input = base_input();
    input.coverage_weakening = true;

    let decision = evaluate(input);

    assert_eq!(decision.state, DecisionState::Yellow);
    assert!(
        decision
            .unknowns
            .iter()
            .any(|item| item == "coverage_weakening")
    );
    assert_eq!(decision.outcome_state, "needs_human_decision");
}

#[test]
fn trusted_complete_bounded_change_is_green() {
    let decision = evaluate(base_input());

    assert_eq!(decision.state, DecisionState::Green);
    assert!(decision.blockers.is_empty());
    assert!(decision.unknowns.is_empty());
}

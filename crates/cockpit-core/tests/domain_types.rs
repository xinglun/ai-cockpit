use cockpit_core::{Blocker, EvolutionClass, HumanDecisionRequirement, SafeAction, WorkItemState};

#[test]
fn domain_types_expose_governance_vocabulary_without_io() {
    let blocker = Blocker {
        code: "scope_exceeded".into(),
        message: "scope is exceeded".into(),
    };
    let action = SafeAction {
        code: "stop".into(),
        description: "stop and request a new contract".into(),
    };
    let human = HumanDecisionRequirement {
        question: "May this scope change?".into(),
        options: vec!["approve".into(), "reject".into()],
    };
    assert_eq!(blocker.code, "scope_exceeded");
    assert_eq!(action.code, "stop");
    assert_eq!(human.options.len(), 2);
    assert_eq!(WorkItemState::Created, WorkItemState::Created);
    assert_eq!(EvolutionClass::L2, EvolutionClass::L2);
}

use cockpit_core::{
    AuthorityState, Blocker, DecisionState, EvidenceState, EvolutionClass,
    HumanDecisionRequirement, SafeAction, WorkItemState, WorkItemTransitionError,
};

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

#[test]
fn lifecycle_transition_boundary_accepts_only_explicit_successors() {
    assert!(WorkItemState::Created.can_transition_to(&WorkItemState::PreflightReady));
    assert!(WorkItemState::PreflightReady.can_transition_to(&WorkItemState::ImplementationActive));
    assert!(
        WorkItemState::ImplementationActive.can_transition_to(&WorkItemState::VerificationPending)
    );
    assert!(WorkItemState::VerificationPending.can_transition_to(&WorkItemState::FinishReady));
    assert!(WorkItemState::FinishReady.can_transition_to(&WorkItemState::Archived));
    assert!(WorkItemState::Archived.can_transition_to(&WorkItemState::Closed));

    assert!(WorkItemState::ImplementationActive.can_transition_to(&WorkItemState::Paused));
    assert!(WorkItemState::Paused.can_transition_to(&WorkItemState::ImplementationActive));
    assert!(WorkItemState::Blocked.can_transition_to(&WorkItemState::PreflightReady));
    assert!(WorkItemState::Stale.can_transition_to(&WorkItemState::PreflightReady));

    assert!(!WorkItemState::ImplementationActive.can_transition_to(&WorkItemState::FinishReady));
    assert!(!WorkItemState::FinishReady.can_transition_to(&WorkItemState::Closed));
    assert!(!WorkItemState::Closed.can_transition_to(&WorkItemState::ImplementationActive));
    assert!(!WorkItemState::Cancelled.can_transition_to(&WorkItemState::Created));

    assert_eq!(
        WorkItemState::Created
            .transition_to(WorkItemState::PreflightReady)
            .expect("forward transition"),
        WorkItemState::PreflightReady
    );
    assert!(matches!(
        WorkItemState::Closed.transition_to(WorkItemState::Created),
        Err(WorkItemTransitionError::Illegal {
            from: WorkItemState::Closed,
            to: WorkItemState::Created
        })
    ));
}

#[test]
fn lifecycle_and_governance_dimensions_keep_compatible_wire_values() {
    assert_eq!(
        serde_json::to_string(&WorkItemState::FinishReady).expect("serialize state"),
        r#""finish_ready""#
    );
    assert_eq!(
        serde_json::from_str::<WorkItemState>(r#""closed""#).expect("legacy state"),
        WorkItemState::Closed
    );
    assert!(serde_json::from_str::<WorkItemState>(r#""future_state""#).is_err());

    // These dimensions remain independently representable.  A lifecycle
    // transition cannot manufacture authority, evidence, or a decision.
    assert_eq!(DecisionState::Yellow, DecisionState::Yellow);
    assert_eq!(AuthorityState::Missing, AuthorityState::Missing);
    assert_eq!(EvidenceState::Stale, EvidenceState::Stale);
    assert!(WorkItemState::Closed.is_terminal());
    assert!(WorkItemState::Blocked.is_recovery_state());
}

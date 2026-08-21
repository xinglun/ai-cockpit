use cockpit_verification::{
    PerformanceBaseline, PerformanceBudget, PerformanceSample, PlannedAction, PlannedReason,
    PlannedSatisfaction, PlannedState, VerificationGraph, VerificationNode, VerificationNodeKind,
    VerificationPlan, VerificationResult,
};

#[test]
fn graph_rejects_dependency_cycles() {
    let mut graph = VerificationGraph::default();
    graph
        .add(VerificationNode::new(
            "a",
            VerificationNodeKind::Governance,
            vec!["b".into()],
        ))
        .expect("add a");
    graph
        .add(VerificationNode::new(
            "b",
            VerificationNodeKind::Governance,
            vec!["a".into()],
        ))
        .expect("add b");
    assert!(graph.plan().is_err());
}

#[test]
fn performance_baseline_requires_identity_and_enforces_budgets() {
    let baseline = PerformanceBaseline {
        schema_version: 1,
        runtime_version: "0.2.2".into(),
        runtime_digest: format!("sha256:{}", "a".repeat(64)),
        repository_id: format!("sha256:{}", "b".repeat(64)),
        captured_at: "2026-08-22T00:00:00Z".into(),
        samples: vec![PerformanceSample {
            name: "status".into(),
            elapsed_ms: 9,
            iterations: 12,
        }],
        budgets: vec![PerformanceBudget {
            name: "status".into(),
            max_elapsed_ms: 10,
        }],
    };
    let assessment = baseline.assess();
    assert_eq!(assessment.state, "passed");
    assert_eq!(assessment.passed, 1);

    let mut missing_identity = baseline;
    missing_identity.runtime_digest = "unknown".into();
    assert_eq!(missing_identity.assess().state, "failed");
    assert!(
        missing_identity
            .assess()
            .failures
            .contains(&"runtime_or_repository_identity_missing".into())
    );
}

#[test]
fn graph_rejects_duplicate_nodes() {
    let mut graph = VerificationGraph::default();
    graph
        .add(VerificationNode::new(
            "same",
            VerificationNodeKind::Governance,
            vec![],
        ))
        .expect("first node");
    assert!(matches!(
        graph.add(VerificationNode::new(
            "same",
            VerificationNodeKind::Governance,
            vec![],
        )),
        Err(cockpit_verification::GraphError::Duplicate(_))
    ));
}

#[test]
fn verification_plan_and_result_are_explicit_runtime_models() {
    let plan = VerificationPlan {
        node_ids: vec!["quality".into()],
        max_workers: 2,
    };
    let result = VerificationResult {
        node_id: "quality".into(),
        passed: true,
        reused: false,
        protected: false,
        action: PlannedAction::Execute,
        state: PlannedState::NotApplicable,
        reason: PlannedReason::ReuseNotConfigured.code().into(),
        receipt_id: None,
        output_digest: None,
        output_truncated: false,
        timed_out: false,
        satisfied_by: PlannedSatisfaction::Execution,
    };
    assert_eq!(plan.max_workers, 2);
    assert!(result.passed);
}

use cockpit_verification::{
    VerificationGraph, VerificationNode, VerificationNodeKind, VerificationPlan, VerificationResult,
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
    };
    assert_eq!(plan.max_workers, 2);
    assert!(result.passed);
}

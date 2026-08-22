use cockpit_protocol::VerificationTier;
use cockpit_verification::{
    AffectedVerificationError, DependencyConfidence, VerificationGraph, VerificationNode,
    VerificationNodeKind,
};

fn graph() -> VerificationGraph {
    let mut graph = VerificationGraph::default();
    graph
        .add(VerificationNode::new(
            "compile",
            VerificationNodeKind::ProjectCommand,
            Vec::new(),
        ))
        .expect("compile");
    graph
        .add(VerificationNode::new(
            "unit",
            VerificationNodeKind::Reusable,
            vec!["compile".into()],
        ))
        .expect("unit");
    graph
        .add(VerificationNode::new(
            "release-gate",
            VerificationNodeKind::Protected,
            vec!["unit".into()],
        ))
        .expect("release gate");
    graph
}

#[test]
fn complete_graph_returns_only_deterministically_affected_descendants() {
    let plan = graph()
        .affected_verification_plan(&["compile".into()], VerificationTier::T1)
        .expect("affected plan");
    assert_eq!(plan.confidence, DependencyConfidence::Complete);
    assert_eq!(
        plan.affected_node_ids,
        vec!["compile", "unit", "release-gate"]
    );
    assert!(plan.escalated_node_ids.is_empty());
    assert!(plan.unknowns.is_empty());
}

#[test]
fn partial_graph_escalates_only_known_affected_nodes_and_remains_visible() {
    let mut graph = graph();
    graph.set_dependency_confidence(DependencyConfidence::Partial);
    let plan = graph
        .affected_verification_plan(&["unit".into()], VerificationTier::T1)
        .expect("affected plan");
    assert_eq!(plan.affected_node_ids, vec!["unit", "release-gate"]);
    assert_eq!(plan.escalated_node_ids, plan.affected_node_ids);
    assert_eq!(plan.escalated_tier, Some(VerificationTier::T2));
    assert_eq!(plan.unknowns, vec!["dependency_graph_partial"]);
    assert_ne!(plan.confidence, DependencyConfidence::Complete);
}

#[test]
fn unknown_graph_is_conservative_and_never_green() {
    let mut graph = graph();
    graph.set_dependency_confidence(DependencyConfidence::Unknown);
    let plan = graph
        .affected_verification_plan(&["compile".into()], VerificationTier::T1)
        .expect("affected plan");
    assert_eq!(
        plan.affected_node_ids,
        vec!["compile", "unit", "release-gate"]
    );
    assert_eq!(plan.escalated_node_ids, plan.affected_node_ids);
    assert_eq!(plan.escalated_tier, Some(VerificationTier::T2));
    assert_eq!(plan.unknowns, vec!["dependency_graph_unknown"]);
}

#[test]
fn unknown_changed_node_fails_closed() {
    let error = graph()
        .affected_verification_plan(&["missing".into()], VerificationTier::T1)
        .expect_err("missing changed node must fail closed");
    assert!(matches!(
        error,
        AffectedVerificationError::ChangedNodeMissing(node) if node == "missing"
    ));
}

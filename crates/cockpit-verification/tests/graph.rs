use cockpit_verification::{VerificationGraph, VerificationNode, VerificationNodeKind};

#[test]
fn graph_rejects_dependency_cycles() {
    let mut graph = VerificationGraph::default();
    graph.add(VerificationNode::new(
        "a",
        VerificationNodeKind::Governance,
        vec!["b".into()],
    ));
    graph.add(VerificationNode::new(
        "b",
        VerificationNodeKind::Governance,
        vec!["a".into()],
    ));
    assert!(graph.plan().is_err());
}

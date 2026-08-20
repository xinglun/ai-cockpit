use cockpit_verification::{VerificationCommand, execute_bounded};

#[test]
fn bounded_execution_reports_plan_and_process_telemetry() {
    let receipt = execute_bounded(
        vec![
            VerificationCommand::new("one", "true", vec![]),
            VerificationCommand::new("two", "true", vec![]),
        ],
        2,
    )
    .expect("execute");
    assert_eq!(receipt.nodes_planned, 2);
    assert_eq!(receipt.nodes_executed, 2);
    assert_eq!(receipt.processes_spawned, 2);
    assert!(receipt.passed);
}

#[test]
fn protected_node_is_not_skipped_by_reuse() {
    let command = VerificationCommand::new("protected", "true", vec![])
        .with_reuse(true)
        .with_protected(true);
    let receipt = execute_bounded(vec![command], 1).expect("execute");
    assert_eq!(receipt.nodes_reused, 0);
    assert_eq!(receipt.nodes_executed, 1);
}

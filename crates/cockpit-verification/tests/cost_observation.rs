use cockpit_evidence::{DiffIdentity, EvidenceContext, ReusableReceipt};
use cockpit_verification::{
    PlannedAction, VerificationCommand, VerificationCostConfidence, VerificationReusePolicy,
    execute_bounded_at, plan_verification_commands,
};

const NOW: i64 = 1_800_000_000;

fn digest(fill: char) -> String {
    format!("sha256:{}", fill.to_string().repeat(64))
}

fn context(command_digest: String) -> EvidenceContext {
    EvidenceContext {
        content_digest: digest('a'),
        diff: DiffIdentity {
            base_commit: "1111111111111111111111111111111111111111".into(),
            head_commit: "2222222222222222222222222222222222222222".into(),
            changed_paths_digest: digest('b'),
        },
        environment_digest: digest('c'),
        command_digest,
        scope_digest: digest('d'),
        governance_digest: digest('e'),
        toolchain_digest: digest('f'),
        policy_digest: digest('1'),
        profile_digest: digest('2'),
        stage: "task".into(),
        runner: "local".into(),
    }
}

#[test]
fn planner_cost_estimate_is_advisory() {
    let protected = VerificationCommand::new(
        "protected",
        "true",
        vec![],
        VerificationReusePolicy::Protected(cockpit_verification::ProtectedGateClass::Scope),
    );
    let plan = plan_verification_commands(
        vec![
            protected,
            VerificationCommand::new(
                "ordinary",
                "true",
                vec![],
                VerificationReusePolicy::NeverReuse,
            ),
        ],
        NOW,
    )
    .expect("plan");
    let estimate = plan.cost_estimate(2, 2);
    assert_eq!(estimate.confidence, VerificationCostConfidence::Partial);
    assert_eq!(estimate.nodes_planned, 2);
    assert_eq!(estimate.nodes_to_execute, 2);
    assert_eq!(estimate.estimated_parallelism, 2);
    assert!(estimate.advisory_only);
    assert!(
        estimate
            .unknowns
            .iter()
            .any(|unknown| unknown == "verification_state_unknown")
    );
    assert!(
        plan.commands()
            .iter()
            .all(|entry| entry.action == PlannedAction::Execute)
    );
}

#[test]
fn receipt_cost_observation_reports_parallel_and_reuse_facts() {
    let parallel = ["one", "two"]
        .into_iter()
        .map(|id| {
            VerificationCommand::new(
                id,
                "python3",
                vec!["-c".into(), "import time; time.sleep(0.05)".into()],
                VerificationReusePolicy::NeverReuse,
            )
        })
        .collect();
    let mut parallel_receipt = execute_bounded_at(parallel, 2, NOW).expect("parallel execute");
    parallel_receipt.repository_id =
        Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into());
    parallel_receipt.runtime_version = Some("0.2.15".into());
    parallel_receipt.runtime_digest =
        Some("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into());
    let observation = parallel_receipt.cost_observation();
    assert_eq!(observation.confidence, VerificationCostConfidence::Complete);
    assert_eq!(observation.nodes_executed, 2);
    assert!(observation.max_concurrent_processes >= 2);
    assert!(observation.advisory_only);

    let command =
        VerificationCommand::new("reuse", "false", vec![], VerificationReusePolicy::Reusable);
    let current = context(command.command_digest());
    let reusable = ReusableReceipt::new(
        "reuse",
        true,
        current.clone(),
        &digest('3'),
        NOW - 60,
        NOW + 60,
    )
    .expect("reusable receipt");
    let receipt = execute_bounded_at(
        vec![command.with_reuse_candidate(Some(reusable), current)],
        1,
        NOW,
    )
    .expect("reuse execute");
    assert_eq!(receipt.nodes_reused, 1);
    assert_eq!(receipt.nodes_executed, 0);
}

#[test]
fn unknown_cost_confidence_never_becomes_governance_green() {
    let command = VerificationCommand::new(
        "ordinary",
        "true",
        vec![],
        VerificationReusePolicy::NeverReuse,
    );
    let plan = plan_verification_commands(vec![command], NOW).expect("plan");
    let estimate = plan.cost_estimate(0, 1);
    assert_eq!(estimate.confidence, VerificationCostConfidence::Unknown);
    assert!(
        estimate
            .unknowns
            .iter()
            .any(|unknown| unknown == "worker_budget_unknown")
    );
    assert!(estimate.advisory_only);

    let receipt = execute_bounded_at(vec![], 1, NOW).expect("empty receipt");
    let observation = receipt.cost_observation();
    assert_eq!(observation.confidence, VerificationCostConfidence::Unknown);
    assert!(!observation.unknowns.is_empty());
    assert!(observation.advisory_only);
}

#[test]
fn protected_nodes_remain_executed_when_cost_is_observed() {
    let command = VerificationCommand::new(
        "scope",
        "true",
        vec![],
        VerificationReusePolicy::Protected(cockpit_verification::ProtectedGateClass::Scope),
    );
    let receipt = execute_bounded_at(vec![command], 1, NOW).expect("protected execute");
    assert_eq!(receipt.protected_nodes_executed, 1);
    assert_eq!(receipt.nodes_executed, 1);
    assert_eq!(receipt.cost_observation().nodes_executed, 1);
}

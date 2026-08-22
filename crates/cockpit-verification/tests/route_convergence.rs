use cockpit_protocol::{EvidenceAssurance, VerificationStage, VerificationTier};
use cockpit_verification::{
    VerificationCostConfidence, VerificationPlanReceipt, execute_bounded_at,
};

const NOW: i64 = 1_800_000_000;

#[test]
fn plan_receipt_preserves_monotonic_tier_and_assurance_dimensions() {
    let receipt = VerificationPlanReceipt::new(
        VerificationStage::PreCi,
        VerificationTier::T1,
        VerificationTier::T2,
        EvidenceAssurance::RepositoryVerified,
        vec!["bounded_dependency_closure".into()],
        vec!["dependency_unknown".into()],
    )
    .expect("monotonic route");
    assert_eq!(receipt.initial_tier, VerificationTier::T1);
    assert_eq!(receipt.final_tier, VerificationTier::T2);
    assert_eq!(receipt.stage, VerificationStage::PreCi);
    assert_eq!(receipt.assurance, EvidenceAssurance::RepositoryVerified);

    assert!(
        VerificationPlanReceipt::new(
            VerificationStage::PreCi,
            VerificationTier::T2,
            VerificationTier::T1,
            EvidenceAssurance::RepositoryVerified,
            vec![],
            vec!["attempted_downgrade".into()],
        )
        .is_err()
    );
}

#[test]
fn empty_plan_has_zero_parallelism_and_unknown_identity_is_not_complete() {
    let plan = cockpit_verification::plan_verification_commands(Vec::new(), NOW)
        .expect("empty plan is representable");
    let estimate = plan.cost_estimate(2, 2);
    assert_eq!(estimate.estimated_parallelism, 0);

    let receipt = execute_bounded_at(Vec::new(), 1, NOW).expect("empty execution");
    let observation = receipt.cost_observation();
    assert_eq!(observation.confidence, VerificationCostConfidence::Unknown);
    assert!(
        observation
            .unknowns
            .contains(&"repository_identity_unknown".into())
    );
}

#[test]
fn malformed_identity_cannot_produce_complete_cost_observation() {
    let mut receipt = execute_bounded_at(Vec::new(), 1, NOW).expect("empty execution");
    receipt.repository_id = Some("not-a-sha256".into());
    receipt.runtime_version = Some("0.2.15".into());
    receipt.runtime_digest = Some("not-a-sha256".into());
    let observation = receipt.cost_observation();
    assert_eq!(observation.confidence, VerificationCostConfidence::Unknown);
    assert!(
        observation
            .unknowns
            .iter()
            .any(|unknown| unknown == "repository_identity_invalid")
    );
    assert!(
        observation
            .unknowns
            .iter()
            .any(|unknown| unknown == "runtime_identity_invalid")
    );
}

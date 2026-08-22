use cockpit_protocol::{OutcomeState, OutcomeV2};

#[test]
fn blocked_outcome_round_trips_explicit_recovery_fields() {
    let value = serde_json::json!({
        "schemaVersion": 2,
        "repositoryId": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "workItemId": "WI-RECOVERY",
        "state": "unknown",
        "decisionState": "red",
        "summary": "finish gate failed",
        "acceptanceResults": [],
        "unknowns": ["verification_missing"],
        "evidenceRefs": [],
        "humanBenefitReport": {
            "state": "unknown",
            "userVisibleChanges": [],
            "affectedUsers": [],
            "unknowns": ["user_visible_benefit_not_declared"],
            "evidenceRefs": []
        },
        "failedGate": "finish.verification",
        "recoveryCondition": "record current verification evidence and rerun finish"
    });
    let outcome: OutcomeV2 = serde_json::from_value(value).expect("blocked outcome");
    assert_eq!(outcome.failed_gate.as_deref(), Some("finish.verification"));
    assert_eq!(
        outcome.recovery_condition.as_deref(),
        Some("record current verification evidence and rerun finish")
    );
    assert_eq!(outcome.state, OutcomeState::Unknown);
}

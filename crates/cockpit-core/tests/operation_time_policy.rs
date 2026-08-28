use cockpit_core::{
    OPERATION_TIME_REQUEST_SCHEMA_VERSION, OperationTimeDecisionKind, OperationTimeOperation,
    OperationTimeRequest, evaluate_operation_time_policy,
};

fn request() -> OperationTimeRequest {
    OperationTimeRequest {
        schema_version: OPERATION_TIME_REQUEST_SCHEMA_VERSION,
        requested_operation: "execute_script".into(),
        actual_tool_call: "execute_script".into(),
        target_resource: "scripts/cleanup.sh".into(),
        declared_scope: vec!["scripts/cleanup.sh".into()],
        approved_operation: "execute_script".into(),
        approved_target_resource: "scripts/cleanup.sh".into(),
        approved_scope: vec!["scripts/cleanup.sh".into()],
        current_authority: "human:owner".into(),
        evidence_fresh: true,
        destructive_impact: "high".into(),
        input_trust: "authority".into(),
    }
}

#[test]
fn matching_facts_allow_only_the_local_policy_step() {
    let decision = evaluate_operation_time_policy(&request());
    assert_eq!(decision.decision, OperationTimeDecisionKind::Allow);
    assert!(decision.may_proceed_automatically());
}

#[test]
fn every_source_high_risk_category_is_recognized() {
    for operation in [
        OperationTimeOperation::DeleteFiles,
        OperationTimeOperation::ModifyTests,
        OperationTimeOperation::ModifyCi,
        OperationTimeOperation::ModifyBranchProtection,
        OperationTimeOperation::WriteSecret,
        OperationTimeOperation::Push,
        OperationTimeOperation::Merge,
        OperationTimeOperation::Release,
        OperationTimeOperation::DataMigration,
        OperationTimeOperation::ExecuteScript,
        OperationTimeOperation::ExternalApiWrite,
        OperationTimeOperation::InstallOrUpgrade,
        OperationTimeOperation::UninstallGovernance,
    ] {
        let name = serde_json::to_string(&operation).unwrap();
        let mut candidate = request();
        candidate.requested_operation = name.trim_matches('"').into();
        candidate.actual_tool_call = candidate.requested_operation.clone();
        candidate.approved_operation = candidate.requested_operation.clone();
        assert_eq!(
            evaluate_operation_time_policy(&candidate).decision,
            OperationTimeDecisionKind::Allow,
            "{name}"
        );
    }
}

#[test]
fn create_script_cannot_authorize_later_execution() {
    let mut candidate = request();
    candidate.requested_operation = "create_script".into();
    let decision = evaluate_operation_time_policy(&candidate);
    assert_eq!(decision.decision, OperationTimeDecisionKind::Block);
    assert_eq!(
        decision.reason,
        "actual tool call does not match the requested operation"
    );
}

#[test]
fn changed_target_requires_confirmation() {
    let mut candidate = request();
    candidate.target_resource = "scripts/release.sh".into();
    let decision = evaluate_operation_time_policy(&candidate);
    assert_eq!(decision.decision, OperationTimeDecisionKind::Confirm);
    assert!(!decision.may_proceed_automatically());
}

#[test]
fn stale_evidence_requires_confirmation() {
    let mut candidate = request();
    candidate.evidence_fresh = false;
    let decision = evaluate_operation_time_policy(&candidate);
    assert_eq!(decision.decision, OperationTimeDecisionKind::Confirm);
    assert_eq!(decision.reason, "operation evidence is stale");
}

#[test]
fn untrusted_input_cannot_become_authority() {
    let mut candidate = request();
    candidate.input_trust = "untrusted_content".into();
    let decision = evaluate_operation_time_policy(&candidate);
    assert_eq!(decision.decision, OperationTimeDecisionKind::Confirm);
    assert!(!decision.may_proceed_automatically());
}

#[test]
fn unknown_impact_and_unknown_operation_fail_closed() {
    let mut impact = request();
    impact.destructive_impact = "unclassified".into();
    assert_eq!(
        evaluate_operation_time_policy(&impact).decision,
        OperationTimeDecisionKind::Block
    );

    let mut operation = request();
    operation.actual_tool_call = "run_anything".into();
    assert_eq!(
        evaluate_operation_time_policy(&operation).decision,
        OperationTimeDecisionKind::Block
    );
}

#[test]
fn missing_scope_or_schema_fails_closed() {
    let mut scope = request();
    scope.declared_scope.clear();
    assert_eq!(
        evaluate_operation_time_policy(&scope).decision,
        OperationTimeDecisionKind::Block
    );

    let mut schema = request();
    schema.schema_version = 99;
    assert_eq!(
        evaluate_operation_time_policy(&schema).decision,
        OperationTimeDecisionKind::Block
    );
}

#[test]
fn request_wire_shape_rejects_unknown_fields() {
    let mut value = serde_json::to_value(request()).unwrap();
    value["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<OperationTimeRequest>(value).is_err());
}

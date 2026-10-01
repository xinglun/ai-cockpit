use cockpit_core::Digest;
use cockpit_protocol::{
    Contract, ContractAmendmentChange, ContractAmendmentFieldClass, ContractAmendmentOperation,
    ContractAmendmentRequest, apply_contract_amendment, contract_amendment_field_class,
};
use serde_json::json;

fn contract() -> Contract {
    serde_json::from_value(json!({
        "protocolVersion": 1,
        "contractVersion": 2,
        "repositoryId": "repository-id",
        "workItemId": "WI-CONTRACT-AMENDMENT",
        "mode": "code",
        "title": "Typed amendment fixture",
        "intent": {
            "businessGoal": "make the plan adaptable",
            "userGoal": "revise human-owned decisions",
            "problem": "append-only editing cannot correct mistakes",
            "constraints": ["preserve Runtime identity"],
            "nonGoals": ["publish a release"],
            "rationale": "plans must follow observed reality"
        },
        "goal": "append a missing detail",
        "scope": ["src/**", "tests/**"],
        "outOfScope": ["global/**"],
        "risk": "medium",
        "authority": "authorized",
        "acceptanceCriteria": ["A1: amendments preserve identity"],
        "acceptance": ["A1: amendments preserve identity"],
        "requiredEvidenceClasses": ["verification"],
        "sources": [{"path": "docs/plan.md", "reason": "the active plan"}],
        "verification": [{"check": "cargo test", "required": true}],
        "baseRevision": "base-revision",
        "projectProfileDigest": "sha256:profile",
        "repositorySnapshotDigest": "sha256:snapshot",
        "rollbackNote": "restore the prior reviewed plan",
        "rollbackPlan": {"steps": ["revert the amendment"]},
        "unknowns": [],
        "notCodable": false,
        "scenarioCoverage": [{
            "scenario": "serial-path",
            "required": true,
            "status": "unverified",
            "evidence": [],
            "expected": "serial operation remains available",
            "verificationPlan": "run the serial lifecycle test"
        }]
    }))
    .expect("valid Contract fixture")
}

fn request(changes: Vec<ContractAmendmentChange>) -> ContractAmendmentRequest {
    ContractAmendmentRequest {
        schema_version: 1,
        change_id: "change-1".into(),
        expected_contract_digest: Digest::sha256_bytes(b"fixture-contract"),
        reason: "the accepted implementation plan needs correction".into(),
        changes,
        authorization: None,
    }
}

fn change(
    path: &str,
    operation: ContractAmendmentOperation,
    value: Option<serde_json::Value>,
) -> ContractAmendmentChange {
    ContractAmendmentChange {
        path: path.into(),
        operation,
        value,
    }
}

#[test]
fn replaces_scalar_and_recomputes_acceptance_alias() {
    let contract = contract();
    let amended = apply_contract_amendment(
        &contract,
        &request(vec![
            change(
                "/goal",
                ContractAmendmentOperation::Replace,
                Some(json!("replace the incorrect command")),
            ),
            change(
                "/acceptanceCriteria/0",
                ContractAmendmentOperation::Replace,
                Some(json!("A1: retain exact amendment identity")),
            ),
        ]),
    )
    .expect("valid replacement batch");

    assert_eq!(amended.goal, "replace the incorrect command");
    assert_eq!(
        amended.acceptance.as_deref(),
        Some(amended.acceptance_criteria.as_slice())
    );
}

#[test]
fn edits_nested_intent_and_clears_only_optional_fields() {
    let contract = contract();
    let amended = apply_contract_amendment(
        &contract,
        &request(vec![
            change(
                "/intent/userGoal",
                ContractAmendmentOperation::Set,
                Some(json!("account for observed environment changes")),
            ),
            change("/rollbackNote", ContractAmendmentOperation::Clear, None),
        ]),
    )
    .expect("valid nested and clear operations");

    assert_eq!(
        amended
            .intent
            .structured()
            .and_then(|intent| intent.user_goal.as_deref()),
        Some("account for observed environment changes")
    );
    assert_eq!(amended.rollback_note, None);
}

#[test]
fn collection_operations_are_exact_and_later_changes_see_earlier_changes() {
    let contract = contract();
    let amended = apply_contract_amendment(
        &contract,
        &request(vec![
            change(
                "/scope",
                ContractAmendmentOperation::Add,
                Some(json!("docs/**")),
            ),
            change(
                "/scope/2",
                ContractAmendmentOperation::Replace,
                Some(json!("docs/reference/**")),
            ),
            change(
                "/scope",
                ContractAmendmentOperation::Reorder,
                Some(json!(["docs/reference/**", "src/**", "tests/**"])),
            ),
        ]),
    )
    .expect("ordered batch");

    assert_eq!(amended.scope, ["docs/reference/**", "src/**", "tests/**"]);
}

#[test]
fn removes_only_the_exact_existing_collection_element() {
    let contract = contract();
    let amended = apply_contract_amendment(
        &contract,
        &request(vec![change(
            "/scope",
            ContractAmendmentOperation::Remove,
            Some(json!("tests/**")),
        )]),
    )
    .expect("remove exact element");
    assert_eq!(amended.scope, ["src/**"]);

    let error = apply_contract_amendment(
        &contract,
        &request(vec![change(
            "/scope",
            ContractAmendmentOperation::Remove,
            Some(json!("missing/**")),
        )]),
    )
    .expect_err("missing element must not be ignored");
    assert!(error.iter().any(|item| item.code == "element_not_found"));
    assert_eq!(contract.scope, ["src/**", "tests/**"]);
}

#[test]
fn invalid_later_batch_operation_leaves_the_typed_input_unchanged() {
    let contract = contract();
    let original = contract.clone();
    let error = apply_contract_amendment(
        &contract,
        &request(vec![
            change(
                "/goal",
                ContractAmendmentOperation::Replace,
                Some(json!("first operation is locally valid")),
            ),
            change("/goal", ContractAmendmentOperation::Clear, None),
        ]),
    )
    .expect_err("required goal cannot be cleared");
    assert!(error.iter().any(|item| item.code == "not_clearable"));
    assert_eq!(contract, original);
}

#[test]
fn protected_and_unknown_paths_fail_closed() {
    let contract = contract();
    for path in [
        "/repositoryId",
        "/baseRevision",
        "/scenarioCoverage/0/status",
    ] {
        let error = apply_contract_amendment(
            &contract,
            &request(vec![change(
                path,
                ContractAmendmentOperation::Replace,
                Some(json!("forged")),
            )]),
        )
        .expect_err("protected path must reject");
        assert!(error.iter().any(|item| item.path == path), "{error:#?}");
    }

    let error = apply_contract_amendment(
        &contract,
        &request(vec![change(
            "/unknownField",
            ContractAmendmentOperation::Set,
            Some(json!("not accepted")),
        )]),
    )
    .expect_err("unknown path must reject");
    assert!(error.iter().any(|item| item.path == "/unknownField"));
}

#[test]
fn field_registry_distinguishes_sensitive_plan_changes() {
    assert_eq!(
        contract_amendment_field_class("/goal").expect("goal classification"),
        ContractAmendmentFieldClass::PlanEditable
    );
    assert_eq!(
        contract_amendment_field_class("/verification/0/required")
            .expect("verification classification"),
        ContractAmendmentFieldClass::SensitivePlanEditable
    );
    assert!(contract_amendment_field_class("/repositoryId").is_err());
}

#[test]
fn contract_validate_rejects_empty_source_and_verification_declarations() {
    let mut value = serde_json::to_value(contract()).expect("Contract JSON");
    value["sources"] = serde_json::json!([
        {"path": " \t", "reason": "a path is required"},
        {"path": "docs/reference/contract.md", "reason": "  "}
    ]);
    value["verification"] = serde_json::json!([
        {"check": " \n", "required": true},
        "  "
    ]);
    let contract: Contract = serde_json::from_value(value).expect("typed Contract");

    let errors = contract
        .validate()
        .expect_err("empty declarations must be rejected");
    assert!(
        errors
            .iter()
            .any(|error| error == "sources[0].path must be non-empty")
    );
    assert!(
        errors
            .iter()
            .any(|error| error == "sources[1].reason must be non-empty")
    );
    assert!(
        errors
            .iter()
            .any(|error| error == "verification[0].check must be non-empty")
    );
    assert!(
        errors
            .iter()
            .any(|error| error == "verification[1] must be non-empty")
    );
}

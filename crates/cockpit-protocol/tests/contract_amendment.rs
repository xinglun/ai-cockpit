use cockpit_core::Digest;
use cockpit_protocol::{
    Contract, ContractAmendmentChange, ContractAmendmentFieldClass, ContractAmendmentOperation,
    ContractAmendmentRequest, apply_contract_amendment, contract_amendment_field_class,
};
use serde_json::json;

fn material_review_profile() -> serde_json::Value {
    json!({
        "schemaVersion": 1,
        "permittedUnknown": "repository_material_inspection_unavailable",
        "permittedCause": "readable_committed_rust_syntax_unknown",
        "assurance": "self_declared",
        "reviewerActor": "agent:Raydot",
        "authoritySource": "user:Ray 2026-10-07 current-conversation delegation to agent:Raydot (WI-1068 Contract.sources)",
        "acceptResidualRisk": true
    })
}

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
fn material_review_profile_requires_strict_fields_and_matching_capability_guard() {
    let mut guarded = contract();
    guarded.governance_profile = Some(json!({
        "unrelatedProfileField": {"preserve": true},
        "materialInspectionReview": material_review_profile()
    }));
    let missing_guard = guarded
        .validate()
        .expect_err("opt-in without guard must fail");
    assert!(
        missing_guard
            .iter()
            .any(|error| error.contains("material-inspection-review")),
        "missing protected capability guard: {missing_guard:?}"
    );

    guarded
        .required_runtime_capabilities
        .push("material-inspection-review".into());
    guarded
        .validate()
        .expect("strict opt-in with guard is valid");
    assert_eq!(
        guarded.governance_profile.as_ref().unwrap()["unrelatedProfileField"]["preserve"],
        true
    );

    let mut unknown_field = material_review_profile();
    unknown_field["unexpected"] = json!(true);
    guarded.governance_profile = Some(json!({"materialInspectionReview": unknown_field}));
    let errors = guarded
        .validate()
        .expect_err("unknown nested opt-in field must fail");
    assert!(
        errors.iter().any(|error| error.contains("unexpected")),
        "unknown nested field was not identified: {errors:?}"
    );

    guarded.governance_profile = None;
    let errors = guarded
        .validate()
        .expect_err("guard without opt-in must fail");
    assert!(
        errors
            .iter()
            .any(|error| error.contains("material-inspection-review"))
    );
}

#[test]
fn typed_profile_amendment_adds_guard_without_editing_protected_field() {
    let amended = apply_contract_amendment(
        &contract(),
        &request(vec![change(
            "/governanceProfile",
            ContractAmendmentOperation::Set,
            Some(json!({
                "unrelatedProfileField": {"preserve": true},
                "materialInspectionReview": material_review_profile()
            })),
        )]),
    )
    .expect("Runtime writer adds the protected capability guard");

    assert_eq!(
        amended.required_runtime_capabilities,
        ["material-inspection-review"]
    );
    assert_eq!(
        amended.governance_profile.as_ref().unwrap()["unrelatedProfileField"]["preserve"],
        true
    );

    let direct_guard = apply_contract_amendment(
        &contract(),
        &request(vec![change(
            "/requiredRuntimeCapabilities",
            ContractAmendmentOperation::Add,
            Some(json!("material-inspection-review")),
        )]),
    )
    .expect_err("the capability remains protected from caller changes");
    assert!(
        direct_guard
            .iter()
            .any(|error| error.code == "protected_field")
    );
}

#[test]
fn material_review_profile_rejects_mismatched_values_and_incomplete_authority() {
    let mut guarded = contract();
    guarded
        .required_runtime_capabilities
        .push("material-inspection-review".into());
    for (field, value) in [
        ("schemaVersion", json!(2)),
        ("permittedUnknown", json!("all_unknowns")),
        ("permittedCause", json!("unreadable_source")),
        ("assurance", json!("host_authenticated")),
        ("reviewerActor", json!("human:Ray")),
        ("authoritySource", json!("  ")),
        ("acceptResidualRisk", json!(false)),
    ] {
        let mut profile = material_review_profile();
        profile[field] = value;
        guarded.governance_profile = Some(json!({"materialInspectionReview": profile}));
        let errors = guarded
            .validate()
            .expect_err("mismatched material review opt-in must fail closed");
        assert!(
            errors.iter().any(|error| error.contains(field)),
            "{field} mismatch was not diagnosed: {errors:?}"
        );
    }
}

#[test]
fn material_review_profile_accepts_another_explicit_agent_for_another_work_item() {
    let mut guarded = contract();
    guarded
        .required_runtime_capabilities
        .push("material-inspection-review".into());
    let mut profile = material_review_profile();
    profile["reviewerActor"] = json!("agent:SecondReviewer");
    guarded.governance_profile = Some(json!({"materialInspectionReview": profile}));
    guarded
        .validate()
        .expect("product schema accepts an explicitly selected agent actor");

    for invalid in [
        "human:Ray",
        "agent:",
        "agent:human:Ray",
        "agent: reviewer",
        "agent:Raydot\n",
    ] {
        let mut profile = material_review_profile();
        profile["reviewerActor"] = json!(invalid);
        guarded.governance_profile = Some(json!({"materialInspectionReview": profile}));
        assert!(
            guarded.validate().is_err(),
            "invalid actor {invalid:?} must be rejected"
        );
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

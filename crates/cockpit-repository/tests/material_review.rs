use cockpit_protocol::{
    Contract, MaterialInspectionReviewDecision, MaterialInspectionReviewDecisionInput,
    MaterialInspectionReviewDecisionReceipt,
};
use cockpit_repository::{
    MaterialReviewDecisionValidationError, MaterialReviewRequestError, MaterialUnknownCause,
    amend_work_item_contract_with_runtime, material_review_request, plan_work_item_material_review,
    read_work_item_contract_amendments, record_work_item_governance_controls_with_runtime,
    validate_material_review_decision,
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn review_runtime() -> cockpit_protocol::RuntimeContext {
    cockpit_protocol::RuntimeContext {
        runtime_version: "material-review-test-runtime".into(),
        protocol_version: cockpit_protocol::PROTOCOL_VERSION,
        runtime_digest: cockpit_core::Digest::sha256_bytes(b"material-review-test-runtime"),
    }
}

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn fixture() -> (tempfile::TempDir, Contract) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    git(root, &["init", "-q"]);
    fs::write(root.join("README.md"), "baseline\n").unwrap();
    git(root, &["add", "."]);
    git(
        root,
        &[
            "-c",
            "user.name=fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "base",
        ],
    );
    let base = git(root, &["rev-parse", "HEAD"]);
    let contract: Contract = serde_json::from_value(json!({
        "protocolVersion": 1,
        "repositoryId": cockpit_repository::repository_id(root).to_string(),
        "workItemId": "WI-MATERIAL",
        "intent": "review bounded committed material",
        "goal": "retain exact material evidence",
        "scope": ["README.md", "src/material.rs"],
        "outOfScope": [],
        "risk": "high",
        "authority": "authorized",
        "acceptanceCriteria": ["exact material request"],
        "requiredEvidenceClasses": [],
        "baseRevision": base,
        "projectProfileDigest": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "repositorySnapshotDigest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"
    })).unwrap();
    (directory, contract)
}

fn commit(root: &Path) {
    git(root, &["add", "."]);
    git(
        root,
        &[
            "-c",
            "user.name=fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "candidate",
        ],
    );
}

fn active_recorded_material_review_fixture() -> (
    tempfile::TempDir,
    cockpit_protocol::RuntimeContext,
    PathBuf,
    PathBuf,
    PathBuf,
    MaterialInspectionReviewDecisionInput,
) {
    let (directory, mut contract, _initial_request, mut input) = material_review_decision_fixture();
    let root = directory.path();
    cockpit_repository::attach(root).expect("attach fixture");
    contract.repository_id = cockpit_repository::repository_id(root).to_string();
    let project: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join(".ai/project.json")).unwrap()).unwrap();
    contract.project_profile_digest = project["profileDigest"].as_str().unwrap().parse().unwrap();
    let active = root.join(".ai/work-items/active");
    fs::create_dir_all(&active).expect("active directory");
    let contract_path = active.join("WI-MATERIAL.contract.json");
    let mut persisted_contract = serde_json::to_value(&contract).expect("Contract value");
    persisted_contract["predecessorWorkItemId"] = serde_json::Value::Null;
    persisted_contract["predecessorContractDigest"] = serde_json::Value::Null;
    persisted_contract["recoveryDecisionPath"] = serde_json::Value::Null;
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&persisted_contract).expect("Contract JSON"),
    )
    .expect("write active Contract");
    let request = plan_work_item_material_review(root, "WI-MATERIAL").expect("initial request");
    input.request_digest = request.request_digest.clone();
    let summary_path = active.join("WI-MATERIAL.summary.json");
    fs::write(
        &summary_path,
        serde_json::to_vec_pretty(&json!({
            "workItemId": "WI-MATERIAL",
            "state": "implementation_active",
            "checkpointCount": 0,
            "createdAt": "2026-10-07T00:00:00Z"
        }))
        .expect("Summary JSON"),
    )
    .expect("write active Summary");
    let runtime = review_runtime();
    cockpit_repository::preflight_work_item_with_runtime(root, &contract_path, &runtime)
        .expect("fresh preflight");
    let receipt =
        cockpit_repository::record_work_item_material_review(root, "WI-MATERIAL", &input, &runtime)
            .expect("initial typed material-review receipt");
    assert_eq!(receipt.request_digest, request.request_digest);
    let summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary bytes"))
            .expect("Summary JSON");
    let sidecar_path = root.join(
        summary["materialReviewReceipt"]["path"]
            .as_str()
            .expect("receipt sidecar path"),
    );
    (
        directory,
        runtime,
        contract_path,
        summary_path,
        sidecar_path,
        input,
    )
}

#[test]
fn active_material_plan_digest_matches_persisted_contract_across_json_shapes() {
    let (directory, mut contract) = fixture();
    let root = directory.path();
    fs::write(root.join("README.md"), "candidate\n").unwrap();
    commit(root);
    cockpit_repository::attach(root).expect("attach fixture");
    contract.repository_id = cockpit_repository::repository_id(root).to_string();
    let project: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join(".ai/project.json")).unwrap()).unwrap();
    contract.project_profile_digest = project["profileDigest"].as_str().unwrap().parse().unwrap();

    let active = root.join(".ai/work-items/active");
    fs::create_dir_all(&active).unwrap();
    let path = active.join("WI-MATERIAL.contract.json");
    let mut explicit_nulls = serde_json::to_value(&contract).unwrap();
    explicit_nulls["predecessorWorkItemId"] = serde_json::Value::Null;
    explicit_nulls["predecessorContractDigest"] = serde_json::Value::Null;
    explicit_nulls["recoveryDecisionPath"] = serde_json::Value::Null;

    fs::write(&path, serde_json::to_vec_pretty(&explicit_nulls).unwrap()).unwrap();
    let explicit_digest = cockpit_protocol::digest_json(&explicit_nulls).unwrap();
    let explicit_plan = plan_work_item_material_review(root, "WI-MATERIAL").unwrap();
    assert_eq!(explicit_plan.contract_digest, explicit_digest);
    let runtime = review_runtime();
    let explicit_status =
        cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .unwrap();
    assert_eq!(
        explicit_status.source_digests.get("contract"),
        Some(&explicit_plan.contract_digest)
    );

    let object = explicit_nulls.as_object().unwrap();
    let reversed_fields = object
        .iter()
        .rev()
        .map(|(key, value)| {
            format!(
                "{}:{}",
                serde_json::to_string(key).unwrap(),
                serde_json::to_string(value).unwrap()
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    fs::write(&path, format!("{{{reversed_fields}}}")).unwrap();
    let reordered_raw: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let reordered_digest = cockpit_protocol::digest_json(&reordered_raw).unwrap();
    let reordered_plan = plan_work_item_material_review(root, "WI-MATERIAL").unwrap();
    assert_eq!(reordered_plan.contract_digest, reordered_digest);
    assert_eq!(
        reordered_plan.contract_digest,
        explicit_plan.contract_digest
    );
    assert_eq!(reordered_plan.request_digest, explicit_plan.request_digest);
    let reordered_status =
        cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .unwrap();
    assert_eq!(
        reordered_status.source_digests.get("contract"),
        Some(&reordered_plan.contract_digest)
    );

    let mut omitted_nulls = explicit_nulls;
    for field in [
        "predecessorWorkItemId",
        "predecessorContractDigest",
        "recoveryDecisionPath",
    ] {
        omitted_nulls.as_object_mut().unwrap().remove(field);
    }
    fs::write(&path, serde_json::to_vec_pretty(&omitted_nulls).unwrap()).unwrap();
    let omitted_digest = cockpit_protocol::digest_json(&omitted_nulls).unwrap();
    let omitted_plan = plan_work_item_material_review(root, "WI-MATERIAL").unwrap();
    assert_eq!(omitted_plan.contract_digest, omitted_digest);
    assert_ne!(omitted_plan.contract_digest, explicit_plan.contract_digest);
    assert_ne!(omitted_plan.request_digest, explicit_plan.request_digest);
    let omitted_status =
        cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .unwrap();
    assert_eq!(
        omitted_status.source_digests.get("contract"),
        Some(&omitted_plan.contract_digest)
    );
}

fn commit_large_text_changes(
    root: &Path,
    contract: &mut Contract,
    count: usize,
    padding_bytes: usize,
) {
    let padding = "p".repeat(padding_bytes);
    fs::create_dir_all(root.join("assets")).unwrap();
    for index in 0..count {
        fs::write(
            root.join(format!("assets/large-{index}.txt")),
            format!("revision=0-{index}\n{padding}\n"),
        )
        .unwrap();
    }
    commit(root);
    contract.base_revision = git(root, &["rev-parse", "HEAD"]);

    for index in 0..count {
        fs::write(
            root.join(format!("assets/large-{index}.txt")),
            format!("revision=1-{index}\n{padding}\n"),
        )
        .unwrap();
    }
    commit(root);
}

fn assert_budget_error(
    result: Result<impl Sized, MaterialReviewRequestError>,
    expected_budget: &str,
) {
    let error = match result {
        Err(error) => error.to_string(),
        Ok(_) => panic!("material request should reject the {expected_budget} budget"),
    };
    assert!(
        error.contains(expected_budget),
        "unexpected material budget error: {error}"
    );
}

fn material_review_decision_fixture() -> (
    tempfile::TempDir,
    Contract,
    cockpit_repository::MaterialReviewRequest,
    MaterialInspectionReviewDecisionInput,
) {
    let (directory, mut contract) = fixture();
    contract.governance_profile = Some(json!({
        "materialInspectionReview": {
            "schemaVersion": 1,
            "permittedUnknown": "repository_material_inspection_unavailable",
            "permittedCause": "readable_committed_rust_syntax_unknown",
            "assurance": "self_declared",
            "reviewerActor": "agent:Raydot",
            "authoritySource": "user-delegation:ray-approved-WI1068",
            "acceptResidualRisk": true
        }
    }));
    contract.required_runtime_capabilities =
        vec![cockpit_protocol::MATERIAL_INSPECTION_REVIEW_CAPABILITY.to_owned()];

    let root = directory.path();
    fs::create_dir_all(root.join("src")).unwrap();
    let (marker, _) = include_str!(
        "../../../tests/conformance/fixtures/repository-prompt-injection/repository/material.txt"
    )
    .trim()
    .split_once(';')
    .unwrap();
    let operation = ["de", "lete"].concat();
    fs::write(
        root.join("src/material.rs"),
        format!("fn material() {{ let marker = {marker:?}; let operation = {operation:?}; consume(marker, operation); }}\n"),
    )
    .unwrap();
    commit(root);

    let request = material_review_request(root, &contract).unwrap();
    let input: MaterialInspectionReviewDecisionInput = serde_json::from_value(json!({
        "schemaVersion": 1,
        "decision": "accept_permitted_unknowns",
        "requestDigest": request.request_digest.to_string(),
        "reviewerActor": "agent:Raydot",
        "authoritySource": "user-delegation:ray-approved-WI1068",
        "assurance": "self_declared",
        "evidenceRefs": [{
            "path": "docs/review-evidence.md",
            "digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"
        }],
        "rationale": "Review the exact bounded syntax unknown.",
        "residualRisk": "The bounded source scanner remains incomplete for this syntax."
    }))
    .unwrap();
    (directory, contract, request, input)
}

#[test]
fn material_review_decision_validator_rejects_unapproved_profile_actor() {
    let (_directory, contract, request, mut input) = material_review_decision_fixture();
    input.reviewer_actor = "agent:OtherReviewer".into();

    let error = validate_material_review_decision(
        &contract,
        &request,
        &input,
        "agent:codex-executor",
        "2026-10-07T14:00:00Z",
    )
    .unwrap_err();
    assert_eq!(
        error,
        MaterialReviewDecisionValidationError::ReviewerMismatch
    );
    assert_eq!(
        input.decision,
        MaterialInspectionReviewDecision::AcceptPermittedUnknowns
    );
}

#[test]
fn material_review_decision_types_reject_unknown_fields_and_receipt_tampering() {
    let (_directory, contract, request, input) = material_review_decision_fixture();
    let mut input_value = serde_json::to_value(&input).unwrap();
    input_value["unapproved"] = json!(true);
    assert!(serde_json::from_value::<MaterialInspectionReviewDecisionInput>(input_value).is_err());

    let mut receipt = validate_material_review_decision(
        &contract,
        &request,
        &input,
        "agent:codex-executor",
        "2026-10-07T14:00:00Z",
    )
    .unwrap();
    assert_eq!(receipt.reviewer_actor, "agent:Raydot");
    assert_eq!(receipt.recorded_by, "agent:codex-executor");
    assert_eq!(
        receipt.reviewed_source_head,
        Some(request.reviewed_source_head)
    );
    assert_eq!(receipt.canonical_digest().unwrap(), receipt.receipt_digest);

    let mut receipt_value = serde_json::to_value(&receipt).unwrap();
    receipt_value["unexpected"] = json!("rejected");
    assert!(
        serde_json::from_value::<MaterialInspectionReviewDecisionReceipt>(receipt_value).is_err()
    );

    receipt.rationale.push_str(" Tampered after hashing.");
    assert!(receipt.validate_integrity().is_err());
}

#[test]
fn material_review_decision_validator_rejects_stale_input_and_tampered_request() {
    let (_directory, contract, request, input) = material_review_decision_fixture();
    let mut stale_input_value = serde_json::to_value(&input).unwrap();
    stale_input_value["requestDigest"] =
        json!("sha256:1111111111111111111111111111111111111111111111111111111111111111");
    let stale_input: MaterialInspectionReviewDecisionInput =
        serde_json::from_value(stale_input_value).unwrap();
    let stale_error = validate_material_review_decision(
        &contract,
        &request,
        &stale_input,
        "agent:codex-executor",
        "2026-10-07T14:00:00Z",
    )
    .unwrap_err();
    assert_eq!(
        stale_error,
        MaterialReviewDecisionValidationError::RequestDigestMismatch
    );

    let mut tampered_request = request;
    tampered_request.entries[0].path.push_str(".tampered");
    let tampered_error = validate_material_review_decision(
        &contract,
        &tampered_request,
        &input,
        "agent:codex-executor",
        "2026-10-07T14:00:00Z",
    )
    .unwrap_err();
    assert_eq!(
        tampered_error,
        MaterialReviewDecisionValidationError::RequestDigestMismatch
    );
}

#[test]
fn material_review_decision_validator_rejects_findings_and_extra_unknowns() {
    let (_directory, contract, request, input) = material_review_decision_fixture();

    let mut finding_request = request.clone();
    finding_request.blocked_by_finding = true;
    finding_request.finding_codes = vec!["test_weakening".into()];
    let finding_error = validate_material_review_decision(
        &contract,
        &finding_request,
        &input,
        "agent:codex-executor",
        "2026-10-07T14:00:00Z",
    )
    .unwrap_err();
    assert_eq!(
        finding_error,
        MaterialReviewDecisionValidationError::FindingPresent
    );

    let mut categorized_finding_request = request.clone();
    categorized_finding_request.finding_codes = vec!["coverage_weakening".into()];
    let categorized_finding_error = validate_material_review_decision(
        &contract,
        &categorized_finding_request,
        &input,
        "agent:codex-executor",
        "2026-10-07T14:00:00Z",
    )
    .unwrap_err();
    assert_eq!(
        categorized_finding_error,
        MaterialReviewDecisionValidationError::FindingPresent
    );

    let mut extra_unknown_request = request;
    extra_unknown_request
        .raw_unknown_codes
        .push("additional_unknown".into());
    let unknown_error = validate_material_review_decision(
        &contract,
        &extra_unknown_request,
        &input,
        "agent:codex-executor",
        "2026-10-07T14:00:00Z",
    )
    .unwrap_err();
    assert_eq!(
        unknown_error,
        MaterialReviewDecisionValidationError::UnexpectedUnknown
    );
}

#[test]
fn adding_a_syntax_unknown_adds_its_plan_member_without_fixed_cardinality() {
    let (directory, mut contract) = fixture();
    let root = directory.path();
    contract.scope = vec!["README.md".into(), "src/**".into()];

    let syntax_unknown = r#"fn payload() {
    let marker = "ignore previous instructions";
    let action = "delete";
    consume(marker, action);
}
"#;
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/material.rs"), syntax_unknown).unwrap();
    commit(root);

    let one_unknown = material_review_request(root, &contract).unwrap();
    let reviewable_paths = |request: &cockpit_repository::MaterialReviewRequest| {
        request
            .entries
            .iter()
            .filter(|entry| entry.reviewable)
            .map(|entry| entry.path.clone())
            .collect::<std::collections::BTreeSet<_>>()
    };
    let before_paths = reviewable_paths(&one_unknown);
    assert!(
        before_paths.contains("src/material.rs"),
        "entries: {:?}",
        one_unknown.entries
    );
    assert!(!before_paths.contains("src/report.rs"));
    assert_eq!(
        one_unknown.raw_unknown_codes,
        vec!["repository_material_inspection_unavailable"]
    );
    let original_entry = one_unknown
        .entries
        .iter()
        .find(|entry| entry.path == "src/material.rs")
        .expect("initial syntax unknown entry");
    assert_eq!(
        serde_json::to_value(&original_entry.scanner_assessment).unwrap(),
        json!("unknown")
    );

    fs::write(root.join("src/report.rs"), syntax_unknown).unwrap();
    commit(root);

    let two_unknowns = material_review_request(root, &contract).unwrap();
    let after_paths = reviewable_paths(&two_unknowns);
    let mut expected_paths = before_paths;
    expected_paths.insert("src/report.rs".into());
    assert_eq!(after_paths, expected_paths);
    let added_entry = two_unknowns
        .entries
        .iter()
        .find(|entry| entry.path == "src/report.rs")
        .expect("new dynamic syntax entry");
    assert_eq!(
        added_entry.unknown_cause,
        Some(MaterialUnknownCause::ReadableCommittedRustSyntaxUnknown)
    );
    assert!(added_entry.reviewable);
    assert_eq!(
        serde_json::to_value(&added_entry.scanner_assessment).unwrap(),
        json!("unknown")
    );
    assert_eq!(
        two_unknowns.raw_unknown_codes,
        vec!["repository_material_inspection_unavailable"]
    );
    assert!(two_unknowns.finding_codes.is_empty());
    assert_ne!(one_unknown.request_digest, two_unknowns.request_digest);
}

#[test]
fn material_review_decision_validator_rejects_wrong_authority_source() {
    let (_directory, contract, request, mut input) = material_review_decision_fixture();
    input.authority_source = "user-delegation:unrelated".into();

    let error = validate_material_review_decision(
        &contract,
        &request,
        &input,
        "agent:codex-executor",
        "2026-10-07T14:00:00Z",
    )
    .unwrap_err();
    assert_eq!(
        error,
        MaterialReviewDecisionValidationError::AuthorityMismatch
    );
}

#[test]
fn admitted_material_review_writes_immutable_receipt_and_exact_summary_pointer() {
    let (directory, mut contract, _initial_request, mut input) = material_review_decision_fixture();
    let root = directory.path();
    cockpit_repository::attach(root).expect("attach fixture");
    contract.repository_id = cockpit_repository::repository_id(root).to_string();
    let project: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join(".ai/project.json")).unwrap()).unwrap();
    contract.project_profile_digest = project["profileDigest"].as_str().unwrap().parse().unwrap();
    let active = root.join(".ai/work-items/active");
    fs::create_dir_all(&active).expect("active directory");
    let contract_path = active.join("WI-MATERIAL.contract.json");
    let mut persisted_contract = serde_json::to_value(&contract).expect("Contract value");
    persisted_contract["predecessorWorkItemId"] = serde_json::Value::Null;
    persisted_contract["predecessorContractDigest"] = serde_json::Value::Null;
    persisted_contract["recoveryDecisionPath"] = serde_json::Value::Null;
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&persisted_contract).expect("Contract JSON"),
    )
    .expect("write active Contract");
    let request =
        plan_work_item_material_review(root, "WI-MATERIAL").expect("recompute attached request");
    assert_eq!(
        request.contract_digest,
        cockpit_protocol::digest_json(&persisted_contract).unwrap()
    );
    input.request_digest = request.request_digest.clone();
    let summary_path = active.join("WI-MATERIAL.summary.json");
    fs::write(
        &summary_path,
        serde_json::to_vec_pretty(&json!({
            "workItemId": "WI-MATERIAL",
            "state": "implementation_active",
            "checkpointCount": 0,
            "createdAt": "2026-10-07T00:00:00Z"
        }))
        .expect("Summary JSON"),
    )
    .expect("write active Summary");

    let runtime = review_runtime();
    let preflight =
        cockpit_repository::preflight_work_item_with_runtime(root, &contract_path, &runtime)
            .expect("fresh preflight");
    assert!(
        preflight
            .unknowns
            .contains(&"repository_material_inspection_unavailable".into())
    );
    let status =
        cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .expect("status after preflight");
    assert!(
        status
            .safe_actions
            .contains(&"record_material_review_decision".into())
    );

    let receipt =
        cockpit_repository::record_work_item_material_review(root, "WI-MATERIAL", &input, &runtime)
            .expect("admitted typed material-review decision");
    assert_eq!(receipt.request_digest, request.request_digest);
    assert_eq!(
        receipt.reviewed_source_head.as_deref(),
        Some(request.reviewed_source_head.as_str())
    );
    let pointer: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("updated Summary"))
            .expect("Summary JSON");
    let sidecar_path = root.join(pointer["materialReviewReceipt"]["path"].as_str().unwrap());
    let sidecar_bytes = fs::read(&sidecar_path).expect("immutable receipt sidecar");
    assert_eq!(
        pointer["materialReviewReceipt"]["digest"],
        cockpit_core::Digest::sha256_bytes(&sidecar_bytes).to_string()
    );
    assert_eq!(
        pointer["materialReviewReceipt"]["receiptDigest"],
        receipt.receipt_digest.to_string()
    );

    let after =
        cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .expect("status after receipt");
    assert_eq!(
        after.review_assurance,
        Some(cockpit_protocol::MaterialInspectionReviewAssurance::SelfDeclared)
    );
    assert_eq!(
        after.review_receipt_digest,
        Some(receipt.receipt_digest.clone())
    );
    assert!(
        !after
            .effective_unknowns
            .contains(&"repository_material_inspection_unavailable".into())
    );

    let summary_before_replay = fs::read(&summary_path).expect("Summary bytes");
    assert!(cockpit_repository::record_work_item_material_review(
        root,
        "WI-MATERIAL",
        &input,
        &runtime,
    )
    .is_err());
    assert_eq!(
        fs::read(&summary_path).expect("Summary after replay attempt"),
        summary_before_replay,
        "replay must not rewrite the Summary pointer"
    );

    git(root, &["add", "-f", ".ai"]);
    git(
        root,
        &[
            "-c",
            "user.name=fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "descendant .ai-only review record",
        ],
    );
    let descendant =
        cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .expect("status on .ai-only descendant");
    assert_eq!(
        descendant.review_receipt_digest,
        Some(receipt.receipt_digest.clone())
    );
    assert_eq!(descendant.review_assurance, after.review_assurance);
    assert!(
        descendant
            .raw_scanner_unknowns
            .contains(&"repository_material_inspection_unavailable".into())
    );
    assert!(
        !descendant
            .effective_unknowns
            .contains(&"repository_material_inspection_unavailable".into())
    );

    let old_pointer = pointer["materialReviewReceipt"].clone();
    let old_sidecar_path = root.join(old_pointer["path"].as_str().unwrap());
    let old_sidecar_bytes = fs::read(&old_sidecar_path).expect("old immutable sidecar");
    fs::write(
        root.join("README.md"),
        "candidate with refreshed material identity\n",
    )
    .expect("change source snapshot");
    commit(root);
    let refreshed_request =
        plan_work_item_material_review(root, "WI-MATERIAL").expect("refreshed request");
    assert_ne!(refreshed_request.request_digest, request.request_digest);
    let refreshed_preflight =
        cockpit_repository::preflight_work_item_with_runtime(root, &contract_path, &runtime)
            .expect("preflight the new committed source candidate");
    assert!(
        refreshed_preflight
            .unknowns
            .contains(&"repository_material_inspection_unavailable".into())
    );
    let stale_status =
        cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .expect("status with stale prior-request receipt");
    assert!(
        stale_status
            .safe_actions
            .contains(&"record_material_review_decision".into()),
        "a valid prior receipt must allow a new exact-request decision without discharging the new Unknown: {stale_status:#?}"
    );
    assert!(
        stale_status
            .effective_unknowns
            .contains(&"repository_material_inspection_unavailable".into())
    );
    assert!(
        stale_status
            .effective_unknowns
            .contains(&"material_review_receipt_stale".into())
    );

    let mut refreshed_input = input.clone();
    refreshed_input.request_digest = refreshed_request.request_digest.clone();
    let refreshed_receipt = cockpit_repository::record_work_item_material_review(
        root,
        "WI-MATERIAL",
        &refreshed_input,
        &runtime,
    )
    .expect("record the new exact-request receipt");
    assert_eq!(
        refreshed_receipt.request_digest,
        refreshed_request.request_digest
    );
    assert_eq!(
        fs::read(&old_sidecar_path).expect("preserved old sidecar"),
        old_sidecar_bytes,
        "the previous immutable receipt must remain unchanged"
    );
    let refreshed_summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("refreshed Summary"))
            .expect("Summary JSON");
    assert_eq!(
        refreshed_summary["materialReviewReceiptHistory"],
        json!([old_pointer]),
        "Summary keeps the prior receipt pointer in append-only history"
    );
    assert_ne!(
        refreshed_summary["materialReviewReceipt"]["requestDigest"],
        old_pointer["requestDigest"]
    );
    let refreshed_status =
        cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .expect("status after refreshed receipt");
    assert_eq!(
        refreshed_status.review_receipt_digest,
        Some(refreshed_receipt.receipt_digest.clone())
    );
    assert!(
        !refreshed_status
            .effective_unknowns
            .contains(&"repository_material_inspection_unavailable".into())
    );
    assert!(
        !refreshed_status
            .effective_unknowns
            .contains(&"material_review_receipt_stale".into())
    );

    git(
        root,
        &["checkout", "--orphan", "non-ancestor-material-review"],
    );
    git(root, &["add", "-A"]);
    git(root, &["add", "-f", ".ai"]);
    git(
        root,
        &[
            "-c",
            "user.name=fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "non-ancestor material snapshot",
        ],
    );
    let non_ancestor =
        cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .expect("status remains available after non-ancestor rewrite");
    assert!(
        non_ancestor
            .blockers
            .contains(&"material_review_projection_unavailable".into())
    );
    assert_eq!(non_ancestor.review_receipt_digest, None);
    assert!(
        non_ancestor
            .effective_unknowns
            .contains(&"material_review_projection_unavailable".into())
    );
}

#[test]
fn stale_material_review_receipt_requires_valid_prior_sidecar() {
    for tamper in ["corrupt", "wrong_identity", "missing_file"] {
        let (directory, runtime, contract_path, summary_path, sidecar_path, mut input) =
            active_recorded_material_review_fixture();
        let root = directory.path();
        fs::write(root.join("README.md"), "new source request\n").expect("source change");
        commit(root);
        let request = plan_work_item_material_review(root, "WI-MATERIAL").expect("new request");
        input.request_digest = request.request_digest.clone();

        match tamper {
            "corrupt" => fs::write(&sidecar_path, b"{corrupt receipt").expect("corrupt sidecar"),
            "wrong_identity" => {
                let mut receipt: MaterialInspectionReviewDecisionReceipt =
                    serde_json::from_slice(&fs::read(&sidecar_path).expect("old sidecar"))
                        .expect("old receipt");
                receipt.repository_id =
                    cockpit_core::Digest::sha256_bytes(b"foreign repository").to_string();
                receipt.receipt_digest = receipt.canonical_digest().expect("receipt digest");
                let bytes = serde_json::to_vec_pretty(&receipt).expect("forged receipt bytes");
                fs::write(&sidecar_path, &bytes).expect("write resealed foreign receipt");
                let mut summary: serde_json::Value =
                    serde_json::from_slice(&fs::read(&summary_path).expect("Summary"))
                        .expect("Summary JSON");
                summary["materialReviewReceipt"]["digest"] =
                    cockpit_core::Digest::sha256_bytes(&bytes)
                        .to_string()
                        .into();
                summary["materialReviewReceipt"]["receiptDigest"] =
                    receipt.receipt_digest.to_string().into();
                fs::write(
                    &summary_path,
                    serde_json::to_vec_pretty(&summary).expect("Summary bytes"),
                )
                .expect("reseal prior pointer");
            }
            "missing_file" => fs::remove_file(&sidecar_path).expect("remove prior sidecar"),
            _ => unreachable!(),
        }

        let preflight =
            cockpit_repository::preflight_work_item_with_runtime(root, &contract_path, &runtime)
                .expect("preflight preserves invalid prior receipt as an Unknown");
        assert!(
            preflight
                .unknowns
                .contains(&"material_review_receipt_invalid".into()),
            "tamper case {tamper} must remain invalid"
        );
        let status = cockpit_repository::work_item_status_snapshot_with_runtime(
            root,
            "WI-MATERIAL",
            &runtime,
        )
        .expect("status after invalid prior receipt");
        assert!(
            !status
                .safe_actions
                .contains(&"record_material_review_decision".into()),
            "tamper case {tamper} must not admit a replacement receipt: {status:#?}"
        );
        assert!(
            status
                .effective_unknowns
                .contains(&"material_review_receipt_invalid".into())
        );

        let summary_before = fs::read(&summary_path).expect("Summary before rejected record");
        let error = cockpit_repository::record_work_item_material_review(
            root,
            "WI-MATERIAL",
            &input,
            &runtime,
        )
        .expect_err("invalid prior receipt must fail closed");
        assert!(
            error
                .to_string()
                .contains("record_material_review_decision")
                || error.to_string().contains("material-review"),
            "unexpected refusal for {tamper}: {error}"
        );
        assert_eq!(
            fs::read(&summary_path).expect("Summary after rejected record"),
            summary_before,
            "rejected renewal must preserve Summary bytes for {tamper}"
        );
        let current_sidecar = root.join(format!(
            ".ai/evidence/material-inspection-review/WI-MATERIAL/{}.json",
            request
                .request_digest
                .as_str()
                .strip_prefix("sha256:")
                .expect("request digest prefix")
        ));
        assert!(
            !current_sidecar.exists(),
            "rejected renewal must not write a current receipt for {tamper}"
        );
    }
}

#[test]
fn stale_receipt_history_shape_is_validated_before_sidecar_write() {
    let (directory, runtime, contract_path, summary_path, old_sidecar_path, mut input) =
        active_recorded_material_review_fixture();
    let root = directory.path();
    fs::write(
        root.join("README.md"),
        "new source request with malformed history\n",
    )
    .expect("change source");
    commit(root);
    let request = plan_work_item_material_review(root, "WI-MATERIAL").expect("new request");
    input.request_digest = request.request_digest.clone();

    cockpit_repository::preflight_work_item_with_runtime(root, &contract_path, &runtime)
        .expect("preflight exact new request");
    let status =
        cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .expect("admitted stale receipt renewal");
    assert!(
        status
            .safe_actions
            .contains(&"record_material_review_decision".into())
    );

    let mut summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    summary["materialReviewReceiptHistory"] = json!({"malformed": true});
    fs::write(
        &summary_path,
        serde_json::to_vec_pretty(&summary).expect("malformed-history Summary"),
    )
    .expect("write malformed-history Summary");
    let summary_before = fs::read(&summary_path).expect("Summary bytes before record");
    let current_sidecar = root.join(format!(
        ".ai/evidence/material-inspection-review/WI-MATERIAL/{}.json",
        request
            .request_digest
            .as_str()
            .strip_prefix("sha256:")
            .expect("request digest prefix")
    ));
    assert!(!current_sidecar.exists());

    let error =
        cockpit_repository::record_work_item_material_review(root, "WI-MATERIAL", &input, &runtime)
            .expect_err("malformed receipt history must be rejected");
    assert!(error.to_string().contains("history must be an array"));
    assert_eq!(
        fs::read(&summary_path).expect("Summary remains unchanged"),
        summary_before
    );
    assert!(
        !current_sidecar.exists(),
        "validation failure must not leave an orphan immutable sidecar"
    );
    assert!(old_sidecar_path.exists(), "prior sidecar remains preserved");
}

#[test]
fn stale_receipt_with_changed_contract_profile_is_not_renewable() {
    let (directory, runtime, contract_path, summary_path, old_sidecar_path, mut input) =
        active_recorded_material_review_fixture();
    let root = directory.path();
    fs::write(
        root.join("README.md"),
        "new source request under changed profile\n",
    )
    .expect("change source");
    commit(root);

    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("Contract"))
            .expect("Contract JSON");
    contract["governanceProfile"]["materialInspectionReview"]["reviewerActor"] =
        json!("agent:Raydot-Updated");
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract).expect("changed Contract bytes"),
    )
    .expect("change the review profile");
    let request = plan_work_item_material_review(root, "WI-MATERIAL").expect("new request");
    input.request_digest = request.request_digest.clone();
    input.reviewer_actor = "agent:Raydot-Updated".into();

    let preflight =
        cockpit_repository::preflight_work_item_with_runtime(root, &contract_path, &runtime)
            .expect("preflight changed Contract request");
    assert!(
        preflight
            .unknowns
            .contains(&"material_review_receipt_invalid".into())
    );
    let status =
        cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .expect("status after changed review profile");
    assert!(
        !status
            .safe_actions
            .contains(&"record_material_review_decision".into()),
        "a prior receipt bound to a different Contract/profile must not authorize renewal"
    );
    assert!(
        status
            .effective_unknowns
            .contains(&"material_review_receipt_invalid".into())
    );

    let summary_before = fs::read(&summary_path).expect("Summary before refused renewal");
    let current_sidecar = root.join(format!(
        ".ai/evidence/material-inspection-review/WI-MATERIAL/{}.json",
        request
            .request_digest
            .as_str()
            .strip_prefix("sha256:")
            .expect("request digest prefix")
    ));
    let error =
        cockpit_repository::record_work_item_material_review(root, "WI-MATERIAL", &input, &runtime)
            .expect_err("changed Contract/profile must reject prior receipt renewal");
    assert!(
        error
            .to_string()
            .contains("record_material_review_decision")
            || error.to_string().contains("material-review")
    );
    assert_eq!(
        fs::read(&summary_path).expect("Summary unchanged"),
        summary_before
    );
    assert!(!current_sidecar.exists());
    assert!(old_sidecar_path.exists());
}

#[test]
fn stale_receipt_after_valid_contract_amendment_is_classified_stale() {
    let (directory, runtime, contract_path, summary_path, old_sidecar_path, mut input) =
        active_recorded_material_review_fixture();
    let root = directory.path();
    let original_summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    let old_pointer = original_summary["materialReviewReceipt"].clone();
    let old_sidecar_bytes = fs::read(&old_sidecar_path).expect("old immutable sidecar");
    let old_contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("Contract"))
            .expect("Contract JSON");
    let old_contract_digest = cockpit_protocol::digest_json(&old_contract).expect("old digest");
    let reason = "preserve the prior review as history after an authorized Contract amendment";
    amend_work_item_contract_with_runtime(
        root,
        "WI-MATERIAL",
        &json!({
            "schemaVersion": 1,
            "changeId": "material-review-safe-goal-update",
            "expectedContractDigest": old_contract_digest,
            "reason": reason,
            "changes": [{
                "path": "/goal",
                "operation": "replace",
                "value": "retain exact material evidence after the approved plan update"
            }]
        }),
        reason,
        &runtime,
    )
    .expect("record a typed amendment through the official journal route");

    let current_contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("amended Contract"))
            .expect("amended Contract JSON");
    let current_contract_digest =
        cockpit_protocol::digest_json(&current_contract).expect("current Contract digest");
    let amendments = read_work_item_contract_amendments(root, "WI-MATERIAL")
        .expect("validated append-only amendment journal");
    assert_eq!(amendments.len(), 1);
    assert_eq!(amendments[0].previous_contract_digest, old_contract_digest);
    assert_eq!(amendments[0].new_contract_digest, current_contract_digest);

    let request = plan_work_item_material_review(root, "WI-MATERIAL")
        .expect("new exact request after Contract amendment");
    assert_ne!(request.contract_digest, old_contract_digest);
    assert_ne!(
        request.request_digest.to_string(),
        old_pointer["requestDigest"]
            .as_str()
            .expect("old request digest")
    );
    input.request_digest = request.request_digest.clone();

    cockpit_repository::preflight_work_item_with_runtime(root, &contract_path, &runtime)
        .expect("refresh preflight for the amended Contract");
    let current_summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("preflight Summary"))
            .expect("preflight Summary JSON");
    let preflight_decision = json!({
        "schemaVersion": 1,
        "decisionId": "contract-preflight-review",
        "decision": "confirm_review",
        "workItemId": "WI-MATERIAL",
        "repositoryId": cockpit_repository::repository_id(root).to_string(),
        "contractDigest": current_contract_digest,
        "preflightDecisionDigest": current_summary["preflightDecisionDigest"],
        "repositorySnapshotDigest": current_summary["preflightRepositorySnapshotDigest"],
        "recordedAt": "2026-10-08T00:00:00Z",
        "recordedBy": "human:test-fixture",
        "reason": "test fixture confirms its exact amended Contract"
    });
    record_work_item_governance_controls_with_runtime(
        root,
        "WI-MATERIAL",
        &json!({"decisionEvidence": preflight_decision}),
        &runtime,
    )
    .expect("record only the test fixture's exact preflight decision");

    let status =
        cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .expect("status after valid amendment");
    assert!(
        status
            .effective_unknowns
            .contains(&"material_review_receipt_stale".into()),
        "a valid superseded receipt is stale history, not malformed: {status:#?}"
    );
    assert!(
        !status
            .effective_unknowns
            .contains(&"material_review_receipt_invalid".into())
    );
    assert!(
        status
            .safe_actions
            .contains(&"record_material_review_decision".into()),
        "the old receipt must remain stale until a fresh current-request decision is recorded"
    );

    let renewed =
        cockpit_repository::record_work_item_material_review(root, "WI-MATERIAL", &input, &runtime)
            .expect("record the fresh exact-request test decision");
    assert_eq!(renewed.request_digest, request.request_digest);
    assert_eq!(
        fs::read(&old_sidecar_path).expect("prior sidecar remains immutable"),
        old_sidecar_bytes
    );
    let renewed_summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("updated Summary"))
            .expect("updated Summary JSON");
    assert_eq!(
        renewed_summary["materialReviewReceiptHistory"],
        json!([old_pointer])
    );
    assert_eq!(
        renewed_summary["materialReviewReceipt"]["requestDigest"],
        request.request_digest.to_string()
    );
}

#[test]
fn stale_receipt_without_contract_amendment_chain_remains_invalid() {
    let (directory, runtime, contract_path, _summary_path, _old_sidecar_path, _input) =
        active_recorded_material_review_fixture();
    let root = directory.path();
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("Contract"))
            .expect("Contract JSON");
    contract["goal"] = json!("unrecorded direct Contract mutation");
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract).expect("mutated Contract bytes"),
    )
    .expect("write Contract without journal entry");

    let status =
        cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .expect("status preserves the invalid old pointer as a diagnostic");
    assert!(
        status
            .effective_unknowns
            .contains(&"material_review_receipt_invalid".into()),
        "a digest mismatch without a validated amendment chain remains invalid: {status:#?}"
    );
    assert!(
        !status
            .safe_actions
            .contains(&"record_material_review_decision".into()),
        "an unjournaled Contract edit must not admit receipt renewal"
    );
}

#[test]
fn stale_receipt_with_corrupt_contract_amendment_chain_remains_invalid() {
    let (directory, runtime, contract_path, _summary_path, _old_sidecar_path, _input) =
        active_recorded_material_review_fixture();
    let root = directory.path();
    let contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("Contract"))
            .expect("Contract JSON");
    let contract_digest = cockpit_protocol::digest_json(&contract).expect("Contract digest");
    let reason = "create an amendment chain that the reader must validate";
    amend_work_item_contract_with_runtime(
        root,
        "WI-MATERIAL",
        &json!({
            "schemaVersion": 1,
            "changeId": "corrupted-material-review-journal",
            "expectedContractDigest": contract_digest,
            "reason": reason,
            "changes": [{
                "path": "/goal",
                "operation": "replace",
                "value": "amended before the journal is corrupted"
            }]
        }),
        reason,
        &runtime,
    )
    .expect("record an amendment before tampering with the test journal");

    let committed_path =
        root.join(".ai/evidence/WI-MATERIAL.contract-amendments/00000001.committed.json");
    let mut committed: serde_json::Value =
        serde_json::from_slice(&fs::read(&committed_path).expect("committed amendment"))
            .expect("committed amendment JSON");
    committed["receipt"]["journalDigest"] =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000".into();
    fs::write(
        &committed_path,
        serde_json::to_vec_pretty(&committed).expect("corrupted journal bytes"),
    )
    .expect("corrupt committed journal digest");

    let error =
        cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .expect_err("a damaged amendment journal must fail closed");
    assert!(
        error.to_string().contains("invalid digest chain"),
        "unexpected journal rejection: {error}"
    );
}

#[test]
fn stale_receipt_does_not_block_when_current_request_has_no_reviewable_unknown() {
    let (directory, runtime, contract_path, _summary_path, old_sidecar_path, _input) =
        active_recorded_material_review_fixture();
    let root = directory.path();
    fs::remove_file(root.join("src/material.rs")).expect("remove reviewed Rust material");
    commit(root);

    let request = plan_work_item_material_review(root, "WI-MATERIAL").expect("clean request");
    assert!(request.entries.is_empty());
    assert!(request.raw_unknown_codes.is_empty());
    cockpit_repository::preflight_work_item_with_runtime(root, &contract_path, &runtime)
        .expect("preflight the request with no reviewable Unknown");
    let status =
        cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .expect("status after the prior Unknown is removed");
    assert!(status.raw_scanner_unknowns.is_empty());
    assert!(
        !status
            .effective_unknowns
            .contains(&"material_review_receipt_stale".into()),
        "stale historical evidence must not synthesize an Unknown for a clean current request"
    );
    assert!(old_sidecar_path.exists(), "old evidence remains preserved");
}

#[test]
fn committed_manifest_is_scanned_with_empty_worktree_diff_and_profile_absent() {
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::create_dir(root.join("src")).unwrap();
    let (marker, _) = include_str!(
        "../../../tests/conformance/fixtures/repository-prompt-injection/repository/material.txt"
    )
    .trim()
    .split_once(';')
    .unwrap();
    let operation = ["de", "lete"].concat();
    fs::write(root.join("src/material.rs"), format!("fn material() {{ let marker = {marker:?}; let operation = {operation:?}; consume(marker, operation); }}\n")).unwrap();
    fs::write(root.join("README.md"), "new source\n").unwrap();
    commit(root);
    let request = material_review_request(root, &contract).unwrap();
    assert_eq!(request.entries.len(), 2);
    assert!(
        request
            .entries
            .iter()
            .any(|entry| entry.path == "README.md")
    );
    let rust = request
        .entries
        .iter()
        .find(|entry| entry.path == "src/material.rs")
        .unwrap();
    assert_eq!(
        rust.unknown_cause,
        Some(MaterialUnknownCause::ReadableCommittedRustSyntaxUnknown)
    );
    assert_eq!(
        request.raw_unknown_codes,
        ["repository_material_inspection_unavailable"]
    );
    assert!(!request.review_enabled);
    assert_eq!(
        request.review_diagnostic.as_deref(),
        Some("material_review_not_enabled")
    );
    assert!(!request.material_manifest_digest.as_str().is_empty());
}

#[test]
fn dirty_and_untracked_source_refuse_request() {
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::write(root.join("README.md"), "dirty\n").unwrap();
    assert!(material_review_request(root, &contract).is_err());
    git(root, &["checkout", "--", "README.md"]);
    fs::write(root.join("untracked.txt"), "new\n").unwrap();
    assert!(material_review_request(root, &contract).is_err());
}

#[test]
fn untracked_unicode_ai_fact_is_not_misclassified_as_dirty_source() {
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::create_dir(root.join(".ai")).unwrap();
    fs::write(root.join(".ai/évidence.json"), "{}\n").unwrap();

    let request = material_review_request(root, &contract).unwrap();
    assert!(request.entries.is_empty());
}

#[test]
fn finding_is_never_reviewable() {
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::create_dir(root.join("src")).unwrap();
    let payload = include_str!(
        "../../../tests/conformance/fixtures/repository-prompt-injection/repository/material.txt"
    )
    .trim();
    fs::write(
        root.join("src/material.rs"),
        format!("fn material() {{ let instruction = {payload:?}; }}\n"),
    )
    .unwrap();
    commit(root);
    let request = material_review_request(root, &contract).unwrap();
    assert!(request.blocked_by_finding);
    assert_eq!(request.finding_codes, vec!["repository_prompt_injection"]);
    assert!(request.entries.iter().all(|entry| entry.unknown_cause
        != Some(MaterialUnknownCause::ReadableCommittedRustSyntaxUnknown)));
}

#[test]
fn request_binds_each_distinct_blocking_finding_category() {
    let (directory, mut contract) = fixture();
    let root = directory.path();
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(
        root.join("tests/security.py"),
        "def test_security():\n    pass\n",
    )
    .unwrap();
    fs::write(root.join("pyproject.toml"), "fail_under = 90\n").unwrap();
    commit(root);
    contract.base_revision = git(root, &["rev-parse", "HEAD"]);

    let injection = include_str!(
        "../../../tests/conformance/fixtures/repository-prompt-injection/repository/material.txt"
    )
    .trim();
    let skip_decorator = ["@", "pytest", ".mark.skip", "(", "reason='disabled'", ")"].concat();
    fs::write(root.join("README.md"), format!("{injection}\n")).unwrap();
    fs::write(
        root.join("tests/security.py"),
        format!("{skip_decorator}\ndef test_security():\n    pass\n"),
    )
    .unwrap();
    fs::write(root.join("pyproject.toml"), "fail_under = 70\n").unwrap();
    commit(root);

    let request = material_review_request(root, &contract).unwrap();
    let value = serde_json::to_value(&request).unwrap();
    assert_eq!(
        value["findingCodes"],
        json!([
            "coverage_weakening",
            "repository_prompt_injection",
            "test_weakening"
        ])
    );
    assert_eq!(value["blockedByFinding"], true);
}

#[cfg(unix)]
#[test]
fn committed_symlink_refuses_request() {
    use std::os::unix::fs::symlink;
    let (directory, contract) = fixture();
    let root = directory.path();
    symlink("README.md", root.join("linked.txt")).unwrap();
    commit(root);
    assert!(material_review_request(root, &contract).is_err());
}

#[test]
fn ai_only_descendant_retains_request_identity_but_source_change_stales_it() {
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::write(root.join("README.md"), "candidate\n").unwrap();
    commit(root);
    let first = material_review_request(root, &contract).unwrap();
    fs::create_dir(root.join(".ai")).unwrap();
    fs::write(root.join(".ai/note.txt"), "metadata only\n").unwrap();
    commit(root);
    let descendant = material_review_request(root, &contract).unwrap();
    assert_ne!(first.reviewed_source_head, descendant.reviewed_source_head);
    assert_eq!(
        first.material_manifest_digest,
        descendant.material_manifest_digest
    );
    assert_eq!(
        first.source_snapshot_digest,
        descendant.source_snapshot_digest
    );
    assert_eq!(first.request_digest, descendant.request_digest);
    fs::write(root.join("README.md"), "different source\n").unwrap();
    commit(root);
    let different = material_review_request(root, &contract).unwrap();
    assert_ne!(
        first.material_manifest_digest,
        different.material_manifest_digest
    );
    assert_ne!(first.request_digest, different.request_digest);
}

#[test]
fn normalized_crlf_checkout_is_not_compared_to_committed_blob_bytes() {
    let (directory, contract) = fixture();
    let root = directory.path();
    git(root, &["config", "core.autocrlf", "true"]);
    fs::write(root.join("README.md"), b"candidate\r\n").unwrap();
    commit(root);

    let checkout_bytes = fs::read(root.join("README.md")).unwrap();
    let blob_id = git(root, &["rev-parse", "HEAD:README.md"]);
    let committed_blob = Command::new("git")
        .args(["cat-file", "blob", &blob_id])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(committed_blob.status.success());
    assert_eq!(checkout_bytes, b"candidate\r\n");
    assert_eq!(committed_blob.stdout, b"candidate\n");

    let request = material_review_request(root, &contract).unwrap();
    let readme = request
        .entries
        .iter()
        .find(|entry| entry.path == "README.md")
        .unwrap();
    assert!(readme.after_blob_digest.is_some());
}

#[cfg(unix)]
#[test]
fn no_hunk_text_changes_still_bind_complete_committed_source() {
    use std::os::unix::fs::PermissionsExt;

    let (directory, mut contract) = fixture();
    let root = directory.path();
    fs::create_dir(root.join("src")).unwrap();
    fs::write(root.join("src/chmod.rs"), "fn permission() {}\n").unwrap();
    commit(root);
    contract.base_revision = git(root, &["rev-parse", "HEAD"]);

    fs::write(root.join("src/empty.rs"), "").unwrap();
    let chmod_file = root.join("src/chmod.rs");
    let mut permissions = fs::metadata(&chmod_file).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&chmod_file, permissions).unwrap();
    commit(root);

    let request = material_review_request(root, &contract).unwrap();
    for path in ["src/chmod.rs", "src/empty.rs"] {
        let entry = request
            .entries
            .iter()
            .find(|entry| entry.path == path)
            .unwrap();
        assert_eq!(entry.content_state, "text");
        assert_eq!(entry.unknown_cause, None);
        assert!(entry.after_blob_digest.is_some());
    }
}

#[cfg(unix)]
#[test]
fn material_review_hunkless_git_worker() {
    if std::env::var_os("AI_COCKPIT_TEST_HUNKLESS_WORKER").is_none() {
        return;
    }
    let root = std::path::PathBuf::from(std::env::var_os("AI_COCKPIT_TEST_REPO").unwrap());
    let contract_path =
        std::path::PathBuf::from(std::env::var_os("AI_COCKPIT_TEST_CONTRACT").unwrap());
    let contract: Contract = serde_json::from_slice(&fs::read(contract_path).unwrap()).unwrap();
    let expected_path = std::env::var("AI_COCKPIT_TEST_PATH").unwrap();
    let expectation = std::env::var("AI_COCKPIT_TEST_EXPECTATION").unwrap();
    let request = material_review_request(&root, &contract).unwrap();
    let entry = request
        .entries
        .iter()
        .find(|entry| entry.path == expected_path)
        .unwrap();

    match expectation.as_str() {
        "unknown" => {
            assert_eq!(entry.content_state, "unavailable");
            assert_eq!(format!("{:?}", entry.scanner_assessment), "Unknown");
            assert_eq!(
                entry.unknown_cause,
                Some(MaterialUnknownCause::MissingCompleteSource)
            );
            assert!(!entry.reviewable);
        }
        "clean" => {
            assert_eq!(entry.content_state, "text");
            assert_eq!(format!("{:?}", entry.scanner_assessment), "Clean");
            assert_eq!(entry.unknown_cause, None);
            assert!(!entry.reviewable);
            assert!(entry.after_blob_digest.is_some());
        }
        other => panic!("unsupported test expectation: {other}"),
    }
}

#[cfg(unix)]
fn run_hunkless_git_worker(
    root: &Path,
    contract: &Contract,
    expected_path: &str,
    expectation: &str,
) {
    use std::os::unix::fs::PermissionsExt;

    let real_git = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|directory| directory.join("git"))
        .find(|candidate| candidate.is_file())
        .expect("locate real git executable");
    let wrapper_directory = tempfile::tempdir().unwrap();
    let wrapper = wrapper_directory.path().join("git");
    fs::write(
        &wrapper,
        "#!/bin/sh\nfor arg in \"$@\"; do\n  if [ \"$arg\" = \"--binary\" ]; then\n    tmp=\"${TMPDIR:-/tmp}/ai-cockpit-git-hunkless-$$\"\n    \"$AI_COCKPIT_TEST_REAL_GIT\" \"$@\" >\"$tmp\"\n    result=$?\n    if [ \"$result\" -eq 0 ]; then\n      sed -n -e '/^diff --git /p' -e '/^index /p' -e '/^old mode /p' -e '/^new mode /p' -e '/^new file mode /p' -e '/^deleted file mode /p' -e '/^--- /p' -e '/^+++ /p' \"$tmp\"\n    else\n      cat \"$tmp\"\n    fi\n    rm -f \"$tmp\"\n    exit \"$result\"\n  fi\ndone\nexec \"$AI_COCKPIT_TEST_REAL_GIT\" \"$@\"\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&wrapper).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&wrapper, permissions).unwrap();

    let contract_directory = tempfile::tempdir().unwrap();
    let contract_path = contract_directory.path().join("contract.json");
    fs::write(&contract_path, serde_json::to_vec(contract).unwrap()).unwrap();
    let child_path = std::env::join_paths(
        std::iter::once(wrapper_directory.path().to_path_buf())
            .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "material_review_hunkless_git_worker",
            "--nocapture",
        ])
        .env("AI_COCKPIT_TEST_HUNKLESS_WORKER", "1")
        .env("AI_COCKPIT_TEST_REPO", root)
        .env("AI_COCKPIT_TEST_CONTRACT", contract_path)
        .env("AI_COCKPIT_TEST_PATH", expected_path)
        .env("AI_COCKPIT_TEST_EXPECTATION", expectation)
        .env("AI_COCKPIT_TEST_REAL_GIT", real_git)
        .env("PATH", child_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "hunkless Git worker failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(unix)]
#[test]
fn changed_rust_blob_without_hunks_is_unknown_and_not_reviewable() {
    let (directory, mut contract) = fixture();
    let root = directory.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/material.rs"), "fn material() {}\n").unwrap();
    commit(root);
    contract.base_revision = git(root, &["rev-parse", "HEAD"]);

    let (marker, _) = include_str!(
        "../../../tests/conformance/fixtures/repository-prompt-injection/repository/material.txt"
    )
    .trim()
    .split_once(';')
    .unwrap();
    let action = ["de", "lete"].concat();
    fs::write(
        root.join("src/material.rs"),
        format!("fn material() {{ let marker = {marker:?}; let action = {action:?}; consume(marker, action); }}\n"),
    )
    .unwrap();
    commit(root);

    run_hunkless_git_worker(root, &contract, "src/material.rs", "unknown");
}

#[cfg(unix)]
#[test]
fn no_hunk_same_blob_rename_remains_safe_by_blob_identity() {
    let (directory, mut contract) = fixture();
    let root = directory.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/original.rs"), "fn safe() {}\n").unwrap();
    commit(root);
    contract.base_revision = git(root, &["rev-parse", "HEAD"]);

    git(root, &["mv", "src/original.rs", "src/renamed.rs"]);
    commit(root);

    run_hunkless_git_worker(root, &contract, "src/renamed.rs", "clean");
}

#[test]
fn staged_source_refuses_request() {
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::write(root.join("README.md"), "staged\n").unwrap();
    git(root, &["add", "README.md"]);
    assert!(material_review_request(root, &contract).is_err());
}

#[test]
fn staged_source_rename_into_ai_does_not_hide_original_source_change() {
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::create_dir(root.join(".ai")).unwrap();
    git(root, &["mv", "README.md", ".ai/readme.md"]);

    assert!(material_review_request(root, &contract).is_err());
}

#[test]
fn non_ancestor_rebase_cannot_become_a_canonical_request() {
    let (directory, contract) = fixture();
    let root = directory.path();
    git(root, &["checkout", "-q", "--orphan", "unrelated"]);
    git(root, &["rm", "-q", "-f", "README.md"]);
    fs::write(root.join("README.md"), "unrelated history\n").unwrap();
    commit(root);
    assert!(material_review_request(root, &contract).is_err());
}

#[test]
fn assume_unchanged_source_cannot_masquerade_as_committed_material() {
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::write(root.join("README.md"), "candidate\n").unwrap();
    commit(root);
    git(root, &["update-index", "--assume-unchanged", "README.md"]);
    fs::write(root.join("README.md"), "uncommitted substitute\n").unwrap();
    let outcome = material_review_request(root, &contract);
    git(
        root,
        &["update-index", "--no-assume-unchanged", "README.md"],
    );
    assert!(outcome.is_err());
}

#[test]
fn binary_rust_material_is_unknown_but_not_reviewable() {
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::create_dir(root.join("src")).unwrap();
    fs::write(root.join("src/material.rs"), b"binary\0material").unwrap();
    commit(root);
    let request = material_review_request(root, &contract).unwrap();
    let entry = request
        .entries
        .iter()
        .find(|entry| entry.path == "src/material.rs")
        .unwrap();
    assert_eq!(
        entry.unknown_cause,
        Some(MaterialUnknownCause::NonTextMaterial)
    );
    assert!(!entry.reviewable);
    assert!(
        request
            .raw_unknown_codes
            .contains(&"repository_material_inspection_unavailable".into())
    );
}

#[test]
fn bounded_patch_in_large_committed_rust_source_uses_committed_blob() {
    let (directory, mut contract) = fixture();
    let root = directory.path();
    fs::create_dir(root.join("src")).unwrap();
    let large = format!(
        "fn material() {{ let value = 1; }}\n{}",
        "// filler\n".repeat(35_000)
    );
    assert!(large.len() > cockpit_git::MAX_CHANGE_TEXT_BYTES);
    fs::write(root.join("src/material.rs"), &large).unwrap();
    commit(root);
    contract.base_revision = git(root, &["rev-parse", "HEAD"]);
    fs::write(
        root.join("src/material.rs"),
        large.replacen("value = 1", "value = 2", 1),
    )
    .unwrap();
    commit(root);
    let request = material_review_request(root, &contract).unwrap();
    let entry = request
        .entries
        .iter()
        .find(|entry| entry.path == "src/material.rs")
        .unwrap();
    assert!(!entry.reviewable);
    assert_eq!(entry.unknown_cause, None);
}

#[test]
fn bounded_patch_with_over_budget_source_is_typed_unreviewable() {
    let (directory, mut contract) = fixture();
    let root = directory.path();
    fs::create_dir(root.join("src")).unwrap();
    let huge = format!(
        "fn material() {{ let value = 1; }}\n/*{}*/\n",
        "x".repeat(4 * 1024 * 1024)
    );
    fs::write(root.join("src/material.rs"), &huge).unwrap();
    commit(root);
    contract.base_revision = git(root, &["rev-parse", "HEAD"]);
    fs::write(
        root.join("src/material.rs"),
        huge.replacen("value = 1", "value = 2", 1),
    )
    .unwrap();
    commit(root);
    let request = material_review_request(root, &contract).unwrap();
    let entry = request
        .entries
        .iter()
        .find(|entry| entry.path == "src/material.rs")
        .unwrap();
    assert_eq!(
        entry.unknown_cause,
        Some(MaterialUnknownCause::SourceOverBudget)
    );
    assert!(!entry.reviewable);
}

#[test]
fn cumulative_text_budget_rejects_small_patches_across_large_files() {
    let (directory, mut contract) = fixture();
    commit_large_text_changes(directory.path(), &mut contract, 5, 3_500_000);

    assert_budget_error(
        material_review_request(directory.path(), &contract),
        "total_text_bytes",
    );
}

#[test]
fn cumulative_text_budget_allows_multiple_large_files_with_small_patches() {
    let (directory, mut contract) = fixture();
    commit_large_text_changes(directory.path(), &mut contract, 3, 3_000_000);

    let request = material_review_request(directory.path(), &contract).unwrap();
    assert_eq!(
        request
            .entries
            .iter()
            .filter(|entry| entry.path.starts_with("assets/large-"))
            .count(),
        3
    );
}

#[test]
fn cumulative_blob_budget_rejects_small_binary_patches_across_large_files() {
    let (directory, mut contract) = fixture();
    let root = directory.path();
    let mut initial = vec![0_u8; 7 * 1024 * 1024];
    fs::create_dir_all(root.join("assets")).unwrap();
    for index in 0..5 {
        fs::write(root.join(format!("assets/binary-{index}.bin")), &initial).unwrap();
    }
    commit(root);
    contract.base_revision = git(root, &["rev-parse", "HEAD"]);

    let last = initial.len() - 1;
    for index in 0..5 {
        initial[last] = 1;
        fs::write(root.join(format!("assets/binary-{index}.bin")), &initial).unwrap();
        initial[last] = 0;
    }
    commit(root);

    assert_budget_error(material_review_request(root, &contract), "total_blob_bytes");
}

#[test]
fn changed_file_count_budget_rejects_before_material_scan() {
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::create_dir_all(root.join("assets")).unwrap();
    for index in 0..257 {
        fs::write(root.join(format!("assets/new-{index:03}.txt")), "new\n").unwrap();
    }
    commit(root);

    assert_budget_error(
        material_review_request(root, &contract),
        "changed_file_count",
    );
}

#[cfg(unix)]
#[test]
fn unreadable_committed_source_refuses_request() {
    use std::os::unix::fs::PermissionsExt;
    let (directory, contract) = fixture();
    let root = directory.path();
    fs::write(root.join("README.md"), "candidate\n").unwrap();
    commit(root);
    let file = root.join("README.md");
    let original = fs::metadata(&file).unwrap().permissions();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o000)).unwrap();
    let outcome = material_review_request(root, &contract);
    fs::set_permissions(&file, original).unwrap();
    assert!(outcome.is_err());
}

//! Additional material business fixtures; the private port simulates no production human source.
#![cfg(test)]

use crate as cockpit_repository;
use crate::review_authority::pipeline_tests::TestAuthority;
use cockpit_protocol::review_authority::HumanReviewOperation;
use cockpit_protocol::{
    Contract, MaterialInspectionReviewDecisionInput, MaterialInspectionReviewDecisionReceipt,
};
use cockpit_repository::{
    amend_work_item_contract_with_runtime, material_review_request, plan_work_item_material_review,
    read_work_item_contract_amendments,
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SegmentedAttackPrepared {
    schema_version: u32,
    repository_id: String,
    work_item_id: String,
    sequence: u64,
    request: cockpit_protocol::ContractAmendmentRequest,
    request_digest: cockpit_core::Digest,
    previous_contract: serde_json::Value,
    previous_contract_digest: cockpit_core::Digest,
    new_contract: serde_json::Value,
    new_contract_digest: cockpit_core::Digest,
    previous_summary_digest: cockpit_core::Digest,
    repository_snapshot_digest: cockpit_core::Digest,
    environment_observation_digest: cockpit_core::Digest,
    runtime: cockpit_protocol::RuntimeContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    verification_started: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    checkpointed: Option<bool>,
    previous_journal_digest: cockpit_core::Digest,
    policy_review_requirements: Vec<String>,
    legacy_input_digest: Option<cockpit_core::Digest>,
    prepared_at: String,
}

type SegmentedMaterialFixture = (
    tempfile::TempDir,
    cockpit_protocol::RuntimeContext,
    PathBuf,
    Vec<u8>,
    serde_json::Value,
    MaterialInspectionReviewDecisionInput,
);

fn bind_review_input_to_request(
    input: &mut MaterialInspectionReviewDecisionInput,
    request: &cockpit_repository::MaterialReviewRequest,
) {
    input.schema_version = request.schema_version;
    input.request_digest = request.request_digest.clone();
    input.request_artifact_digest = request
        .request_artifact
        .as_ref()
        .map(|artifact| artifact.digest.clone());
    input.request_artifact_byte_length = request
        .request_artifact
        .as_ref()
        .map(|artifact| artifact.byte_length);
}

fn material_receipt_path(
    root: &Path,
    request: &cockpit_repository::MaterialReviewRequest,
) -> PathBuf {
    let semantic = request
        .request_digest
        .as_str()
        .strip_prefix("sha256:")
        .unwrap();
    let raw = request
        .request_artifact
        .as_ref()
        .unwrap()
        .digest
        .as_str()
        .strip_prefix("sha256:")
        .unwrap();
    root.join(format!(
        ".ai/evidence/material-inspection-review/{}/{semantic}.{raw}.receipt.json",
        request.work_item_id
    ))
}

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

fn benign_syntax_unknown_source() -> &'static str {
    r#"
fn material() -> String {
    let mut label = String::from("token");
    label.push_str("ization");
    label
}
"#
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

fn active_recorded_material_review_fixture(
    authority: &TestAuthority,
) -> (
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
    bind_review_input_to_request(&mut input, &request);
    let summary_path = active.join("WI-MATERIAL.summary.json");
    fs::write(
        &summary_path,
        serde_json::to_vec_pretty(&json!({
            "workItemId": "WI-MATERIAL",
            "repositoryId": cockpit_repository::repository_id(root).to_string(),
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
    authority.approve(
        root,
        "WI-MATERIAL",
        HumanReviewOperation::MaterialReview,
        &json!({"input": input, "runtime": runtime}),
    );
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

fn confirm_segmented_fixture(
    authority: &TestAuthority,
    root: &Path,
    runtime: &cockpit_protocol::RuntimeContext,
) {
    let contract_path = root.join(".ai/work-items/active/WI-MATERIAL.contract.json");
    cockpit_repository::preflight_work_item_with_runtime(root, &contract_path, runtime)
        .expect("native fixture preflight");
    authority.approve(
        root,
        "WI-MATERIAL",
        HumanReviewOperation::PreflightReview,
        &json!({"decision":"confirm_review"}),
    );
}

fn amend_segmented_fixture(
    authority: &TestAuthority,
    root: &Path,
    runtime: &cockpit_protocol::RuntimeContext,
    change_id: &str,
    changes: serde_json::Value,
) {
    let contract: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(".ai/work-items/active/WI-MATERIAL.contract.json")).unwrap(),
    )
    .unwrap();
    let request = json!({
        "schemaVersion": 1,
        "changeId": change_id,
        "expectedContractDigest": cockpit_protocol::digest_json(&contract).unwrap(),
        "reason": "Synthetic native mixed-history replacement fixture, not real consumer authority.",
        "changes": changes
    });
    amend_work_item_contract_with_runtime(root, "WI-MATERIAL", &request, "", runtime)
        .expect("native journaled fixture amendment");
    confirm_segmented_fixture(authority, root, runtime);
}

fn segmented_authority_fixture(authority: &TestAuthority) -> SegmentedMaterialFixture {
    segmented_authority_fixture_with_changes(authority, &[], false, &[])
}

fn segmented_authority_fixture_with_changes(
    authority: &TestAuthority,
    historical_changes: &[serde_json::Value],
    whole_profile_replace: bool,
    adjacent_extra: &[serde_json::Value],
) -> SegmentedMaterialFixture {
    segmented_fixture_with_profile_field(
        authority,
        historical_changes,
        whole_profile_replace,
        adjacent_extra,
        false,
    )
}

fn segmented_fixture_with_profile_field(
    authority: &TestAuthority,
    historical_changes: &[serde_json::Value],
    whole_profile_replace: bool,
    adjacent_extra: &[serde_json::Value],
    add_null_profile_field: bool,
) -> SegmentedMaterialFixture {
    let (directory, _) = fixture();
    let root = directory.path();
    let runtime = review_runtime();
    cockpit_repository::attach(root).unwrap();
    cockpit_repository::start_work_item_with_options_and_runtime(
        root,
        "WI-MATERIAL",
        "prove finite material replacement",
        "retain historical evidence and independently decide current material",
        &["README.md".into(), "src/**".into()],
        &cockpit_repository::WorkItemStartOptions {
            authority: "authorized".into(),
            risk: "high".into(),
            acceptance_criteria: vec!["independent current decision".into()],
            ..Default::default()
        },
        &runtime,
    )
    .expect("native fixture start");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/material.rs"), benign_syntax_unknown_source()).unwrap();
    commit(root);
    amend_segmented_fixture(
        authority,
        root,
        &runtime,
        "fixture-initial-profile-R",
        json!([{
            "path": "/governanceProfile", "operation": "set", "value": {
                "materialInspectionReview": {
                    "schemaVersion": 1,
                    "permittedUnknown": "repository_material_inspection_unavailable",
                    "permittedCause": "readable_committed_rust_syntax_unknown",
                    "assurance": "self_declared",
                    "reviewerActor": "agent:material-fixture",
                    "authoritySource": "fixture:old-risk-source",
                    "acceptResidualRisk": true
                }
            }
        }]),
    );
    let old_request = plan_work_item_material_review(root, "WI-MATERIAL").unwrap();
    let mut input: MaterialInspectionReviewDecisionInput = serde_json::from_value(json!({
        "schemaVersion": 2, "decision": "accept_permitted_unknowns",
        "requestDigest": old_request.request_digest,
        "requestArtifactDigest":old_request.request_artifact.as_ref().unwrap().digest,
        "requestArtifactByteLength":old_request.request_artifact.as_ref().unwrap().byte_length,
        "reviewerActor": "agent:material-fixture", "authoritySource": "fixture:old-risk-source",
        "assurance": "self_declared",
        "evidenceRefs": [{"path":"fixture-review", "digest":cockpit_core::Digest::sha256_bytes(b"synthetic fixture review")}],
        "rationale": "Synthetic exact material review, not an actual person decision.",
        "residualRisk": "Synthetic fixture accepts only its exact readable syntax unknown."
    })).unwrap();
    authority.approve(
        root,
        "WI-MATERIAL",
        HumanReviewOperation::MaterialReview,
        &json!({"input": input, "runtime": runtime}),
    );
    cockpit_repository::record_work_item_material_review(root, "WI-MATERIAL", &input, &runtime)
        .expect("native original immutable material receipt R");
    let summary_path = root.join(".ai/work-items/active/WI-MATERIAL.summary.json");
    let summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
    let old_pointer = summary["materialReviewReceipt"].clone();
    let old_sidecar = root.join(old_pointer["path"].as_str().unwrap());
    let old_bytes = fs::read(&old_sidecar).unwrap();
    // Mirror the real 17c→a5 operation shape: 2 scope, 8 acceptance,
    // 3 scenario and 7 source additions, with the entire profile unchanged.
    let mut mixed = Vec::new();
    for index in 0..2 {
        mixed.push(json!({"path":"/scope","operation":"add","value":format!("docs/fixture-review-{index}.md")}));
    }
    for index in 0..8 {
        mixed.push(json!({"path":"/acceptanceCriteria","operation":"add","value":format!("retain reviewed historical requirement {index}")}));
    }
    for index in 0..3 {
        mixed.push(json!({"path":"/scenarioCoverage","operation":"add","value":{
        "scenario":format!("native mixed historical amendment {index}"),"required":true,
        "expected":"old receipt remains historical","verificationPlan":"cargo test --test material_review","status":"unverified","evidence":[]
    }}));
    }
    for index in 0..7 {
        mixed.push(json!({"path":"/sources","operation":"add","value":{
        "path":format!(".ai/evidence/fixture-reviewed-history-{index}"),"reason":"synthetic native context"
    }}));
    }
    amend_segmented_fixture(
        authority,
        root,
        &runtime,
        "fixture-reviewed-mixed-history-A",
        json!(mixed),
    );
    assert_eq!(
        read_work_item_contract_amendments(root, "WI-MATERIAL").unwrap()[1]
            .changed_values
            .len(),
        20
    );
    let a =
        cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .unwrap();
    assert!(
        a.effective_unknowns
            .contains(&"material_review_receipt_stale".into())
    );
    for (index, changes) in historical_changes.iter().enumerate() {
        amend_segmented_fixture(
            authority,
            root,
            &runtime,
            &format!("fixture-extra-history-{index}"),
            changes.clone(),
        );
    }
    // Historical reviews keep their own old snapshots. Current material may
    // legitimately differ before the final source-only transition and review.
    fs::write(
        root.join("README.md"),
        "current material changed after reviewed history, before adjacent source binding\n",
    )
    .unwrap();
    commit(root);
    let mut adjacent = if whole_profile_replace {
        let current: serde_json::Value = serde_json::from_slice(
            &fs::read(root.join(".ai/work-items/active/WI-MATERIAL.contract.json")).unwrap(),
        )
        .unwrap();
        let mut profile = current["governanceProfile"].clone();
        profile["materialInspectionReview"]["authoritySource"] =
            json!("fixture:new-current-risk-source");
        if add_null_profile_field {
            profile["anotherProfileField"] = serde_json::Value::Null;
        }
        vec![json!({"path":"/governanceProfile", "operation":"replace", "value":profile})]
    } else {
        vec![
            json!({"path":"/governanceProfile/materialInspectionReview/authoritySource", "operation":"set", "value":"fixture:new-current-risk-source"}),
        ]
    };
    adjacent.extend_from_slice(adjacent_extra);
    amend_segmented_fixture(
        authority,
        root,
        &runtime,
        "fixture-adjacent-authority-source-B",
        json!(adjacent),
    );
    let request = plan_work_item_material_review(root, "WI-MATERIAL").unwrap();
    assert_ne!(
        request.source_snapshot_digest,
        old_request.source_snapshot_digest
    );
    bind_review_input_to_request(&mut input, &request);
    input.authority_source = "fixture:new-current-risk-source".into();
    (
        directory,
        runtime,
        old_sidecar,
        old_bytes,
        old_pointer,
        input,
    )
}

fn assert_segmented_rejection_is_read_only(
    root: &Path,
    runtime: &cockpit_protocol::RuntimeContext,
    input: &MaterialInspectionReviewDecisionInput,
) {
    let governance_before = segmented_governance_bytes(root);
    let summary = root.join(".ai/work-items/active/WI-MATERIAL.summary.json");
    let before = fs::read(&summary).unwrap();
    let directory = root.join(".ai/evidence/material-inspection-review/WI-MATERIAL");
    let entries = || {
        let mut names = fs::read_dir(&directory)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        names.sort();
        names
    };
    let original_entries = entries();
    if let Ok(status) =
        cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", runtime)
    {
        assert!(
            !status
                .safe_actions
                .contains(&"record_material_review_decision".into()),
            "invalid finite proof cannot offer a new-decision action: {status:#?}"
        );
    }
    cockpit_repository::record_work_item_material_review(root, "WI-MATERIAL", input, runtime)
        .expect_err("invalid finite proof rejects record");
    assert_eq!(fs::read(summary).unwrap(), before);
    assert_eq!(
        entries(),
        original_entries,
        "rejection must publish no new material artifact"
    );
    assert_eq!(
        segmented_governance_bytes(root),
        governance_before,
        "rejection must preserve all governance bytes, links and paths"
    );
}

pub(super) fn segmented_governance_bytes(
    root: &Path,
) -> std::collections::BTreeMap<PathBuf, (u8, Vec<u8>)> {
    fn visit(
        path: &Path,
        base: &Path,
        entries: &mut std::collections::BTreeMap<PathBuf, (u8, Vec<u8>)>,
    ) {
        let kind = fs::symlink_metadata(path).unwrap().file_type();
        let value = if kind.is_symlink() {
            (
                b'l',
                fs::read_link(path)
                    .unwrap()
                    .into_os_string()
                    .into_encoded_bytes(),
            )
        } else if kind.is_dir() {
            (b'd', Vec::new())
        } else {
            assert!(kind.is_file());
            (b'f', fs::read(path).unwrap())
        };
        entries.insert(path.strip_prefix(base).unwrap().to_path_buf(), value);
        if kind.is_dir() {
            for entry in fs::read_dir(path).unwrap() {
                visit(&entry.unwrap().path(), base, entries);
            }
        }
    }
    let mut entries = std::collections::BTreeMap::new();
    visit(&root.join(".ai"), root, &mut entries);
    entries
}

fn copy_segmented_fixture(source: &Path, target: &Path) {
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let destination = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            fs::create_dir(&destination).unwrap();
            copy_segmented_fixture(&entry.path(), &destination);
        } else {
            assert!(entry.file_type().unwrap().is_file());
            fs::copy(entry.path(), destination).unwrap();
        }
    }
}

fn reseal_segmented_journal_attack(
    journal: &Path,
    sequence: u64,
    mut prepared: serde_json::Value,
    mut committed: serde_json::Value,
) {
    let typed: SegmentedAttackPrepared =
        serde_json::from_value(prepared["prepared"].clone()).unwrap();
    prepared["preparedDigest"] = json!(cockpit_protocol::digest_json(&typed).unwrap());
    committed["preparedDigest"] = prepared["preparedDigest"].clone();
    let mut core = committed["receipt"].clone();
    core.as_object_mut().unwrap().remove("journalDigest");
    committed["receipt"]["journalDigest"] = json!(
        cockpit_protocol::digest_json(&json!({
            "previousJournalDigest": core["previousJournalDigest"], "receipt": core
        }))
        .unwrap()
    );
    for (suffix, value) in [("prepared", prepared), ("committed", committed)] {
        fs::write(
            journal.join(format!("{sequence:08}.{suffix}.json")),
            serde_json::to_vec_pretty(&value).unwrap(),
        )
        .unwrap();
    }
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
    fs::write(root.join("src/material.rs"), benign_syntax_unknown_source()).unwrap();
    commit(root);

    let request = material_review_request(root, &contract).unwrap();
    let input: MaterialInspectionReviewDecisionInput = serde_json::from_value(json!({
        "schemaVersion": 2,
        "decision": "accept_permitted_unknowns",
        "requestDigest": request.request_digest.to_string(),
        "requestArtifactDigest":request.request_artifact.as_ref().unwrap().digest,
        "requestArtifactByteLength":request.request_artifact.as_ref().unwrap().byte_length,
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
fn segmented_authority_source_change_exposes_only_new_decision() {
    let authority = TestAuthority::install();
    let (directory, runtime, old_sidecar, old_bytes, _pointer, _input) =
        segmented_authority_fixture(&authority);
    let root = directory.path();
    let status =
        cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .expect("current exact fixture status");
    assert!(
        status
            .effective_unknowns
            .contains(&"material_review_receipt_invalid".into())
    );
    assert!(
        status
            .effective_unknowns
            .contains(&"repository_material_inspection_unavailable".into())
    );
    assert!(status.review_receipt_digest.is_none());
    assert_eq!(fs::read(old_sidecar).unwrap(), old_bytes);
    if !status
        .safe_actions
        .contains(&"record_material_review_decision".into())
    {
        let retained = directory.keep();
        panic!(
            "reviewed mixed history plus an adjacent source-only transition must permit only a NEW exact decision; retained fixture {retained:?}: {status:#?}"
        );
    }
}

#[test]
fn segmented_new_decision_preserves_history_and_replays_only_its_terminal_result() {
    let authority = TestAuthority::install();
    for whole_profile_replace in [false, true] {
        let (directory, runtime, old_sidecar, old_bytes, pointer, input) =
            segmented_authority_fixture_with_changes(&authority, &[], whole_profile_replace, &[]);
        let root = directory.path();
        authority.approve(
            root,
            "WI-MATERIAL",
            HumanReviewOperation::MaterialReview,
            &json!({"input": input, "runtime": runtime}),
        );
        let receipt = cockpit_repository::record_work_item_material_review(
            root,
            "WI-MATERIAL",
            &input,
            &runtime,
        )
        .expect("new exact decision must be recorded independently of invalid historical receipt");
        assert_eq!(receipt.request_digest, input.request_digest);
        assert_eq!(fs::read(old_sidecar).unwrap(), old_bytes);
        let summary_path = root.join(".ai/work-items/active/WI-MATERIAL.summary.json");
        let bytes = fs::read(&summary_path).unwrap();
        let summary: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(summary["materialReviewReceiptHistory"], json!([pointer]));
        assert_eq!(
            summary["materialReviewReceipt"]["requestDigest"],
            json!(input.request_digest)
        );
        let status = cockpit_repository::work_item_status_snapshot_with_runtime(
            root,
            "WI-MATERIAL",
            &runtime,
        )
        .unwrap();
        assert_eq!(
            status.review_receipt_digest,
            Some(receipt.receipt_digest.clone())
        );
        assert!(
            !status
                .effective_unknowns
                .contains(&"repository_material_inspection_unavailable".into())
        );
        let governance_before_replay = segmented_governance_bytes(root);
        let executions_before_replay = authority.execution_count();
        let replay = cockpit_repository::record_work_item_material_review(
            root,
            "WI-MATERIAL",
            &input,
            &runtime,
        )
        .expect("the exact completed operation returns its immutable historical result");
        assert_eq!(
            serde_json::to_value(replay).unwrap(),
            serde_json::to_value(&receipt).unwrap()
        );
        assert_eq!(
            authority.execution_count(),
            executions_before_replay,
            "terminal replay must not execute again"
        );
        assert_eq!(segmented_governance_bytes(root), governance_before_replay);
        assert_eq!(fs::read(summary_path).unwrap(), bytes);
    }
}

#[test]
fn segmented_rejects_forbidden_intermediate_history_and_mixed_adjacency() {
    let authority = TestAuthority::install();
    let cases = [
        (
            vec![json!([{"path":"/goal","operation":"set","value":"forbidden historical goal"}])],
            vec![],
        ),
        (
            vec![json!([{"path":"/risk","operation":"set","value":"normal"}])],
            vec![],
        ),
        (
            vec![
                json!([{"path":"/governanceProfile/materialInspectionReview/reviewerActor","operation":"set","value":"agent:other-fixture"}]),
                json!([{"path":"/governanceProfile/materialInspectionReview/reviewerActor","operation":"set","value":"agent:material-fixture"}]),
            ],
            vec![],
        ),
        (
            vec![],
            vec![json!({"path":"/scope","operation":"add","value":"src/extra.rs"})],
        ),
        (
            vec![],
            vec![
                json!({"path":"/goal","operation":"set","value":"retain historical evidence and independently decide current material"}),
            ],
        ),
    ];
    for (historical, adjacent) in cases {
        let (directory, runtime, _path, _bytes, _pointer, input) =
            segmented_authority_fixture_with_changes(&authority, &historical, false, &adjacent);
        assert_segmented_rejection_is_read_only(directory.path(), &runtime, &input);
    }
}

#[test]
fn segmented_requires_each_historical_review_even_with_valid_current_confirmation() {
    let authority = TestAuthority::install();
    let (directory, runtime, _path, _bytes, _pointer, input) =
        segmented_authority_fixture(&authority);
    let root = directory.path();
    let history = read_work_item_contract_amendments(root, "WI-MATERIAL").unwrap();
    let digest = &history[0].new_contract_digest;
    let mut removed = 0;
    for (value, raw) in crate::review_ledger::approval_mirrors(root, "WI-MATERIAL").unwrap() {
        let receipt: cockpit_protocol::review_authority::HumanApprovalReceipt =
            serde_json::from_value(value).unwrap();
        if receipt.request.binding.operation == HumanReviewOperation::PreflightReview
            && receipt.request.binding.contract_digest == *digest
        {
            let path = root.join(format!(
                ".ai/decisions/WI-MATERIAL.human-approval.{}.json",
                raw.as_str().trim_start_matches("sha256:")
            ));
            fs::remove_file(path).unwrap();
            removed += 1;
        }
    }
    assert_eq!(removed, 1);
    let status =
        cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .unwrap();
    assert!(
        !status.human_decision_required,
        "current confirmation is deliberately intact"
    );
    assert_segmented_rejection_is_read_only(root, &runtime, &input);
}

#[test]
fn segmented_rejects_corrupt_old_receipt_and_foreign_pointer_history() {
    let authority = TestAuthority::install();
    for corrupt_sidecar in [false, true] {
        let (directory, runtime, old_sidecar, old_bytes, _pointer, input) =
            segmented_authority_fixture(&authority);
        let root = directory.path();
        if corrupt_sidecar {
            let mut corrupt = old_bytes;
            corrupt.push(b' ');
            fs::write(old_sidecar, corrupt).unwrap();
        } else {
            let path = root.join(".ai/work-items/active/WI-MATERIAL.summary.json");
            let mut summary: serde_json::Value =
                serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            summary["materialReviewReceiptHistory"] = json!(["foreign-history-member"]);
            fs::write(path, serde_json::to_vec_pretty(&summary).unwrap()).unwrap();
        }
        assert_segmented_rejection_is_read_only(root, &runtime, &input);
    }
}

#[test]
fn segmented_rejects_committed_source_changed_after_adjacent_anchor() {
    let authority = TestAuthority::install();
    let (directory, runtime, _path, _bytes, _pointer, mut input) =
        segmented_authority_fixture(&authority);
    let root = directory.path();
    fs::write(
        root.join("README.md"),
        "changed material after source-only anchor\n",
    )
    .unwrap();
    commit(root);
    let refreshed = plan_work_item_material_review(root, "WI-MATERIAL").unwrap();

    bind_review_input_to_request(&mut input, &refreshed);
    confirm_segmented_fixture(&authority, root, &runtime);
    assert_segmented_rejection_is_read_only(root, &runtime, &input);
}

#[test]
fn segmented_whole_profile_replace_has_source_only_projection() {
    let authority = TestAuthority::install();
    let (directory, runtime, old_sidecar, old_bytes, _pointer, _input) =
        segmented_authority_fixture_with_changes(&authority, &[], true, &[]);
    let status = cockpit_repository::work_item_status_snapshot_with_runtime(
        directory.path(),
        "WI-MATERIAL",
        &runtime,
    )
    .unwrap();
    assert!(
        status
            .safe_actions
            .contains(&"record_material_review_decision".into())
    );
    assert!(
        status
            .effective_unknowns
            .contains(&"material_review_receipt_invalid".into())
    );
    assert!(status.review_receipt_digest.is_none());
    assert_eq!(fs::read(old_sidecar).unwrap(), old_bytes);
}

#[test]
fn segmented_rejects_source_replace_that_adds_another_null_profile_field() {
    let authority = TestAuthority::install();
    let (directory, runtime, _path, _bytes, _pointer, input) =
        segmented_fixture_with_profile_field(&authority, &[], true, &[], true);
    assert_segmented_rejection_is_read_only(directory.path(), &runtime, &input);
}

#[test]
fn segmented_native_history_integrity_and_false_anchor_matrix() {
    let authority = TestAuthority::install();
    for mutation in [
        "broken-journal",
        "pending-native-tail",
        "duplicate-native-json",
        "fake-summary-adjacency",
        "missing-current-review",
        "duplicate-receipt-json",
    ] {
        let (directory, runtime, _path, _bytes, _pointer, input) =
            segmented_authority_fixture(&authority);
        let root = directory.path();
        let status = cockpit_repository::work_item_status_snapshot_with_runtime(
            root,
            "WI-MATERIAL",
            &runtime,
        )
        .unwrap();
        assert!(
            status
                .safe_actions
                .contains(&"record_material_review_decision".into()),
            "independently approved native context must be eligible BEFORE mutation {mutation}: {status:#?}"
        );
        let journal = root.join(".ai/evidence/WI-MATERIAL.contract-amendments");
        let summary_path = root.join(".ai/work-items/active/WI-MATERIAL.summary.json");
        let mut summary: serde_json::Value =
            serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
        match mutation {
            "broken-journal" => {
                let path = journal.join("00000002.committed.json");
                let mut value: serde_json::Value =
                    serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                value["receipt"]["journalDigest"] =
                    json!(cockpit_core::Digest::sha256_bytes(b"foreign journal"));
                fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
            }
            "pending-native-tail" => {
                fs::remove_file(journal.join("00000003.committed.json")).unwrap()
            }
            "duplicate-native-json" => {
                let path = journal.join("00000002.prepared.json");
                let text = fs::read_to_string(&path).unwrap();
                fs::write(
                    path,
                    format!(
                        "{{\"preparedDigest\":\"{}\",{}",
                        cockpit_core::Digest::sha256_bytes(b"duplicate"),
                        text.trim_start_matches('{')
                    ),
                )
                .unwrap();
            }
            "fake-summary-adjacency" => {
                fs::remove_file(journal.join("00000003.prepared.json")).unwrap();
                fs::remove_file(journal.join("00000003.committed.json")).unwrap();
                summary["authoritySourceReplacementAnchor"] = json!({"claimedCommitted":true,"currentContract":summary["preflightContractDigest"]});
                fs::write(&summary_path, serde_json::to_vec_pretty(&summary).unwrap()).unwrap();
            }
            "missing-current-review" => {
                let (path, _receipt) = current_preflight_mirror(root);
                let entries_before = fs::read_dir(root.join(".ai/decisions")).unwrap().count();
                fs::remove_file(path).unwrap();
                let removed =
                    entries_before - fs::read_dir(root.join(".ai/decisions")).unwrap().count();
                assert_eq!(removed, 1);
            }
            "duplicate-receipt-json" => {
                let path = root.join(summary["materialReviewReceipt"]["path"].as_str().unwrap());
                let text = fs::read_to_string(&path).unwrap();
                let duplicate = format!("{{\"schemaVersion\":1,{}", text.trim_start_matches('{'));
                fs::write(&path, duplicate.as_bytes()).unwrap();
                summary["materialReviewReceipt"]["digest"] =
                    json!(cockpit_core::Digest::sha256_bytes(duplicate.as_bytes()));
                fs::write(&summary_path, serde_json::to_vec_pretty(&summary).unwrap()).unwrap();
            }
            _ => unreachable!(),
        }
        if std::panic::catch_unwind(|| {
            assert_segmented_rejection_is_read_only(root, &runtime, &input)
        })
        .is_err()
        {
            let retained = directory.keep();
            panic!("negative native case {mutation} failed, retained {retained:?}");
        }
    }
}

#[test]
fn segmented_wrong_current_inputs_publish_no_prepare_or_receipt() {
    let authority = TestAuthority::install();
    let (directory, runtime, _path, _bytes, _pointer, input) =
        segmented_authority_fixture(&authority);
    let root = directory.path();
    let summary_path = root.join(".ai/work-items/active/WI-MATERIAL.summary.json");
    let before = fs::read(&summary_path).unwrap();
    let evidence = root.join(".ai/evidence/material-inspection-review/WI-MATERIAL");
    let names = || {
        fs::read_dir(&evidence)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<std::collections::BTreeSet<_>>()
    };
    let original = names();
    for mutation in [
        "request",
        "reviewer",
        "source",
        "schema",
        "evidence",
        "rationale",
    ] {
        let mut wrong = input.clone();
        match mutation {
            "request" => {
                wrong.request_digest = cockpit_core::Digest::sha256_bytes(b"other request")
            }
            "reviewer" => wrong.reviewer_actor = "agent:other-fixture".into(),
            "source" => wrong.authority_source = "fixture:foreign-risk".into(),
            "schema" => wrong.schema_version += 1,
            "evidence" => wrong.evidence_refs.clear(),
            "rationale" => wrong.rationale = " ".into(),
            _ => unreachable!(),
        }
        cockpit_repository::record_work_item_material_review(root, "WI-MATERIAL", &wrong, &runtime)
            .expect_err("wrong actual current input must reject before publication");
        assert_eq!(fs::read(&summary_path).unwrap(), before, "{mutation}");
        assert_eq!(names(), original, "{mutation}");
    }
}

#[test]
fn segmented_minimal_independent_integrity_boundaries() {
    let authority = TestAuthority::install();
    let (baseline, _baseline_runtime, _path, _bytes, _pointer, _baseline_input) =
        segmented_authority_fixture(&authority);
    let (foreign, _, foreign_sidecar, foreign_bytes, foreign_pointer, _) =
        segmented_authority_fixture(&authority);
    assert_ne!(
        cockpit_repository::repository_id(baseline.path()),
        cockpit_repository::repository_id(foreign.path())
    );
    for mutation in [
        "foreign-native-identity",
        "disconnected-predecessor",
        "forbidden-base",
        "nonterminal-source-anchor",
        "direct-contract-edit",
        "stale-current-review",
        "foreign-current-review",
        "rust-blob-hunk-after-anchor",
        "genuine-foreign-pointer-history",
        "journal-symlink",
    ] {
        if mutation == "journal-symlink" && !cfg!(unix) {
            continue;
        }
        let (directory, runtime, _path, _bytes, _pointer, input) =
            segmented_authority_fixture(&authority);
        let root = directory.path();
        let status = cockpit_repository::work_item_status_snapshot_with_runtime(
            root,
            "WI-MATERIAL",
            &runtime,
        )
        .unwrap();
        assert!(
            status
                .safe_actions
                .contains(&"record_material_review_decision".into()),
            "native baseline must be eligible before {mutation}"
        );
        let journal = root.join(".ai/evidence/WI-MATERIAL.contract-amendments");
        let summary_path = root.join(".ai/work-items/active/WI-MATERIAL.summary.json");
        let contract_path = root.join(".ai/work-items/active/WI-MATERIAL.contract.json");
        let read = |path: &Path| -> serde_json::Value {
            serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
        };
        let mut summary = read(&summary_path);
        let old_sidecar = root.join(summary["materialReviewReceipt"]["path"].as_str().unwrap());
        let old_bytes = fs::read(&old_sidecar).unwrap();
        let mut current_input = input.clone();
        match mutation {
            "foreign-native-identity" => {
                for suffix in ["prepared", "committed"] {
                    let name = format!("00000002.{suffix}.json");
                    fs::copy(
                        foreign
                            .path()
                            .join(".ai/evidence/WI-MATERIAL.contract-amendments")
                            .join(&name),
                        journal.join(name),
                    )
                    .unwrap();
                }
                assert!(read_work_item_contract_amendments(root, "WI-MATERIAL").is_err());
            }
            "disconnected-predecessor" => {
                let mut prepared = read(&journal.join("00000003.prepared.json"));
                let mut committed = read(&journal.join("00000003.committed.json"));
                let r = read(&journal.join("00000001.prepared.json"))["prepared"]["newContract"]
                    .clone();
                let before: Contract = serde_json::from_value(r.clone()).unwrap();
                let r_digest = cockpit_protocol::digest_json(&r).unwrap();
                prepared["prepared"]["previousContract"] = r;
                prepared["prepared"]["previousContractDigest"] = json!(r_digest);
                prepared["prepared"]["request"]["expectedContractDigest"] = json!(r_digest);
                let request =
                    serde_json::from_value(prepared["prepared"]["request"].clone()).unwrap();
                let (after, _) =
                    cockpit_protocol::apply_contract_amendment_with_trace(&before, &request)
                        .unwrap();
                let after = serde_json::to_value(after).unwrap();
                let after_digest = cockpit_protocol::digest_json(&after).unwrap();
                let request_digest = cockpit_protocol::digest_json(&request).unwrap();
                prepared["prepared"]["newContract"] = after;
                prepared["prepared"]["newContractDigest"] = json!(after_digest);
                prepared["prepared"]["requestDigest"] = json!(request_digest);
                committed["receipt"]["previousContractDigest"] = json!(r_digest);
                committed["receipt"]["newContractDigest"] = json!(after_digest);
                committed["receipt"]["requestDigest"] = json!(request_digest);
                reseal_segmented_journal_attack(&journal, 3, prepared, committed);
                let history = read_work_item_contract_amendments(root, "WI-MATERIAL")
                    .expect("individual native pairs remain internally valid, but predecessor forks R instead of A");
                assert_ne!(
                    history[1].new_contract_digest,
                    history[2].previous_contract_digest
                );
            }
            "forbidden-base" => {
                let before = segmented_governance_bytes(root);
                let request = json!({"schemaVersion":1,"changeId":"forbidden-native-base",
                    "expectedContractDigest":cockpit_protocol::digest_json(&read(&contract_path)).unwrap(),
                    "reason":"Synthetic negative: base is protected, never a successful native amendment.",
                    "changes":[{"path":"/baseRevision","operation":"set","value":"0000000000000000000000000000000000000000"}]});
                let error = amend_work_item_contract_with_runtime(
                    root,
                    "WI-MATERIAL",
                    &request,
                    "",
                    &runtime,
                )
                .expect_err("native base amendment must be forbidden");
                assert!(error.to_string().contains("baseRevision"), "{error}");
                assert_eq!(segmented_governance_bytes(root), before);
                let mut prepared = read(&journal.join("00000002.prepared.json"));
                let mut committed = read(&journal.join("00000002.committed.json"));
                prepared["prepared"]["request"]["changes"]
                    .as_array_mut()
                    .unwrap()
                    .push(request["changes"][0].clone());
                prepared["prepared"]["requestDigest"] =
                    json!(cockpit_protocol::digest_json(&prepared["prepared"]["request"]).unwrap());
                committed["receipt"]["requestDigest"] =
                    prepared["prepared"]["requestDigest"].clone();
                prepared["prepared"]["newContract"]["baseRevision"] =
                    request["changes"][0]["value"].clone();
                prepared["prepared"]["newContractDigest"] = json!(
                    cockpit_protocol::digest_json(&prepared["prepared"]["newContract"]).unwrap()
                );
                committed["receipt"]["newContractDigest"] =
                    prepared["prepared"]["newContractDigest"].clone();
                prepared["prepared"]["policyReviewRequirements"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!("/baseRevision"));
                committed["receipt"]["policyReviewRequirements"] =
                    prepared["prepared"]["policyReviewRequirements"].clone();
                reseal_segmented_journal_attack(&journal, 2, prepared, committed);
                assert!(read_work_item_contract_amendments(root, "WI-MATERIAL").is_err());
            }
            "nonterminal-source-anchor" => {
                amend_segmented_fixture(
                    &authority,
                    root,
                    &runtime,
                    "after-source-anchor-C",
                    json!([
                        {"path":"/sources","operation":"add","value":{"path":"fixture:later-source","reason":"native later non-source-only transition"}}
                    ]),
                );
                let history = read_work_item_contract_amendments(root, "WI-MATERIAL").unwrap();
                assert_eq!(history.len(), 4);
                assert_eq!(
                    history[2].changed_values[0].path,
                    "/governanceProfile/materialInspectionReview/authoritySource"
                );
                assert_eq!(history[3].changed_values[0].path, "/sources");
                let refreshed = plan_work_item_material_review(root, "WI-MATERIAL").unwrap();

                bind_review_input_to_request(&mut current_input, &refreshed);
            }
            "direct-contract-edit" => {
                let mut contract = read(&contract_path);
                contract["scope"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!("docs/unjournaled.md"));
                fs::write(
                    &contract_path,
                    serde_json::to_vec_pretty(&contract).unwrap(),
                )
                .unwrap();
            }
            "stale-current-review" | "foreign-current-review" => {
                let (path, original) = current_preflight_mirror(root);
                let mut changed = original.clone();
                if mutation == "stale-current-review" {
                    changed["request"]["binding"]["repositorySnapshotDigest"] = read(
                        &journal.join("00000001.prepared.json"),
                    )["prepared"]["repositorySnapshotDigest"]
                        .clone();
                    assert_ne!(
                        changed["request"]["binding"]["repositorySnapshotDigest"],
                        original["request"]["binding"]["repositorySnapshotDigest"]
                    );
                } else {
                    changed["request"]["binding"]["repositoryId"] =
                        json!(cockpit_repository::repository_id(foreign.path()));
                }
                let before = segmented_governance_bytes(root);
                fs::write(path, serde_json::to_vec_pretty(&changed).unwrap()).unwrap();
                let after = segmented_governance_bytes(root);
                let replaced = before
                    .iter()
                    .filter(|(path, bytes)| after.get(*path) != Some(*bytes))
                    .count();
                assert_eq!(replaced, 1);
                assert_ne!(
                    crate::current_review_authority_state(root, "WI-MATERIAL").unwrap(),
                    cockpit_protocol::review_authority::CurrentReviewAuthorityState::Active,
                    "wire tampering must not change the protected source binding"
                );
            }
            "rust-blob-hunk-after-anchor" => {
                let before = plan_work_item_material_review(root, "WI-MATERIAL").unwrap();
                fs::write(
                    root.join("src/material.rs"),
                    format!(
                        "{}\n// new committed Rust material after P\n",
                        benign_syntax_unknown_source()
                    ),
                )
                .unwrap();
                commit(root);
                let after = plan_work_item_material_review(root, "WI-MATERIAL").unwrap();
                let rust = |request: &cockpit_repository::MaterialReviewRequest| {
                    let entry = request
                        .entries
                        .iter()
                        .find(|entry| entry.path == "src/material.rs")
                        .unwrap();
                    (
                        entry.after_blob_digest.clone(),
                        entry.changed_hunk_digest.clone(),
                    )
                };
                let (old_blob, old_hunk) = rust(&before);
                let (new_blob, new_hunk) = rust(&after);
                assert_ne!(old_blob, new_blob);
                assert_ne!(old_hunk, new_hunk);
                assert_ne!(before.request_digest, after.request_digest);
                confirm_segmented_fixture(&authority, root, &runtime);
                // The old decision binding is rejected independently of the
                // fresh input, which also cannot repair a stale P source anchor.
                assert_segmented_rejection_is_read_only(root, &runtime, &input);
                bind_review_input_to_request(&mut current_input, &after);
            }
            "genuine-foreign-pointer-history" => {
                let receipt: MaterialInspectionReviewDecisionReceipt =
                    serde_json::from_slice(&foreign_bytes).unwrap();
                assert_eq!(receipt.canonical_digest().unwrap(), receipt.receipt_digest);
                assert_eq!(
                    foreign_sidecar.file_name(),
                    Path::new(foreign_pointer["path"].as_str().unwrap()).file_name()
                );
                assert_ne!(
                    receipt.repository_id,
                    cockpit_repository::repository_id(root).to_string()
                );
                let target = root.join(foreign_pointer["path"].as_str().unwrap());
                assert_ne!(target, old_sidecar);
                fs::write(target, &foreign_bytes).unwrap();
                summary["materialReviewReceiptHistory"] = json!([foreign_pointer]);
                fs::write(&summary_path, serde_json::to_vec_pretty(&summary).unwrap()).unwrap();
            }
            "journal-symlink" => {
                #[cfg(unix)]
                {
                    let original = journal.join("00000002.prepared.json");
                    let retained = root.join(".ai/evidence/retained-native-prepared.json");
                    fs::copy(&original, &retained).unwrap();
                    fs::remove_file(&original).unwrap();
                    std::os::unix::fs::symlink(retained, original).unwrap();
                }
            }
            _ => unreachable!(),
        }
        if std::panic::catch_unwind(|| {
            assert_segmented_rejection_is_read_only(root, &runtime, &current_input);
            assert_eq!(fs::read(&old_sidecar).unwrap(), old_bytes);
        })
        .is_err()
        {
            let retained = directory.keep();
            panic!("independent negative {mutation} failed; retained {retained:?}");
        }
    }
}

#[cfg(unix)]
#[test]
fn segmented_old_sidecar_symlink_does_not_gain_replacement_eligibility() {
    let authority = TestAuthority::install();
    let (directory, runtime, old_sidecar, old_bytes, _pointer, input) =
        segmented_authority_fixture(&authority);
    let root = directory.path();
    let retained = root.join(".ai/evidence/retained-old-material.json");
    fs::write(&retained, old_bytes).unwrap();
    fs::remove_file(&old_sidecar).unwrap();
    std::os::unix::fs::symlink(retained, old_sidecar).unwrap();
    assert_segmented_rejection_is_read_only(root, &runtime, &input);
}

#[test]
fn admitted_material_review_writes_immutable_receipt_and_exact_summary_pointer() {
    let authority = TestAuthority::install();
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
    bind_review_input_to_request(&mut input, &request);
    let summary_path = active.join("WI-MATERIAL.summary.json");
    fs::write(
        &summary_path,
        serde_json::to_vec_pretty(&json!({
            "workItemId": "WI-MATERIAL",
            "repositoryId": cockpit_repository::repository_id(root).to_string(),
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

    authority.approve(
        root,
        "WI-MATERIAL",
        HumanReviewOperation::MaterialReview,
        &json!({"input": input, "runtime": runtime}),
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
    let executions_before_replay = authority.execution_count();
    let governance_before_replay = segmented_governance_bytes(root);
    let replay =
        cockpit_repository::record_work_item_material_review(root, "WI-MATERIAL", &input, &runtime)
            .expect("return the already completed receipt without another business execution");
    assert_eq!(
        serde_json::to_value(replay).unwrap(),
        serde_json::to_value(&receipt).unwrap()
    );
    assert_eq!(authority.execution_count(), executions_before_replay);
    assert_eq!(segmented_governance_bytes(root), governance_before_replay);
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
    bind_review_input_to_request(&mut refreshed_input, &refreshed_request);
    authority.approve(
        root,
        "WI-MATERIAL",
        HumanReviewOperation::MaterialReview,
        &json!({"input": refreshed_input, "runtime": runtime}),
    );
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
fn material_review_rejects_a_summary_pointer_to_a_noncanonical_sidecar_path() {
    let authority = TestAuthority::install();
    let (directory, runtime, _contract_path, summary_path, sidecar_path, _input) =
        active_recorded_material_review_fixture(&authority);
    let root = directory.path();
    let mut summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary bytes"))
            .expect("Summary JSON");
    let pointer = &mut summary["materialReviewReceipt"];
    let wrong_relative_path =
        ".ai/evidence/material-inspection-review/WI-MATERIAL/copied-receipt.json";
    fs::write(
        root.join(wrong_relative_path),
        fs::read(&sidecar_path).expect("valid immutable sidecar"),
    )
    .expect("copy valid receipt under a wrong path");
    pointer["path"] = json!(wrong_relative_path);
    fs::write(
        &summary_path,
        serde_json::to_vec_pretty(&summary).expect("tampered Summary bytes"),
    )
    .expect("write Summary with wrong receipt path");

    let status =
        cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .expect("status with a noncanonical receipt path");
    assert_eq!(status.review_receipt_digest, None);
    assert!(
        status
            .effective_unknowns
            .contains(&"material_review_receipt_invalid".into())
    );
    assert!(
        status
            .effective_unknowns
            .contains(&"repository_material_inspection_unavailable".into())
    );
    assert!(
        !status
            .safe_actions
            .contains(&"record_material_review_decision".into()),
        "a copied sidecar at a noncanonical path must not authorize renewal"
    );
}

#[test]
fn stale_material_review_receipt_requires_valid_prior_sidecar() {
    let authority = TestAuthority::install();
    for tamper in ["corrupt", "wrong_identity", "missing_file"] {
        let (directory, runtime, contract_path, summary_path, sidecar_path, mut input) =
            active_recorded_material_review_fixture(&authority);
        let root = directory.path();
        fs::write(root.join("README.md"), "new source request\n").expect("source change");
        commit(root);
        let request = plan_work_item_material_review(root, "WI-MATERIAL").expect("new request");
        bind_review_input_to_request(&mut input, &request);

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

        // Exact current approval does not repair invalid prior business evidence.
        authority.approve(
            root,
            "WI-MATERIAL",
            HumanReviewOperation::PreflightReview,
            &json!({"decision":"confirm_review"}),
        );
        authority.approve(
            root,
            "WI-MATERIAL",
            HumanReviewOperation::MaterialReview,
            &json!({"input":input,"runtime":runtime}),
        );
        assert!(
            !cockpit_repository::work_item_status_snapshot_with_runtime(
                root,
                "WI-MATERIAL",
                &runtime
            )
            .unwrap()
            .human_decision_required
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
        assert!(
            error
                .to_string()
                .contains("material_review_receipt_invalid"),
            "target prior-receipt validation must reject: {error}"
        );
        assert_eq!(
            fs::read(&summary_path).expect("Summary after rejected record"),
            summary_before,
            "rejected renewal must preserve Summary bytes for {tamper}"
        );
        let current_sidecar = material_receipt_path(root, &request);
        assert!(
            !current_sidecar.exists(),
            "rejected renewal must not write a current receipt for {tamper}"
        );
    }
}

#[test]
fn stale_receipt_history_shape_is_validated_before_sidecar_write() {
    let authority = TestAuthority::install();
    let (directory, runtime, contract_path, summary_path, old_sidecar_path, mut input) =
        active_recorded_material_review_fixture(&authority);
    let root = directory.path();
    fs::write(
        root.join("README.md"),
        "new source request with malformed history\n",
    )
    .expect("change source");
    commit(root);
    let request = plan_work_item_material_review(root, "WI-MATERIAL").expect("new request");
    bind_review_input_to_request(&mut input, &request);

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

    // Exact current approval does not repair invalid prior business evidence.
    authority.approve(
        root,
        "WI-MATERIAL",
        HumanReviewOperation::PreflightReview,
        &json!({"decision":"confirm_review"}),
    );
    authority.approve(
        root,
        "WI-MATERIAL",
        HumanReviewOperation::MaterialReview,
        &json!({"input":input,"runtime":runtime}),
    );
    assert!(
        !cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .unwrap()
            .human_decision_required
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
    let current_sidecar = material_receipt_path(root, &request);
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
    let authority = TestAuthority::install();
    let (directory, runtime, contract_path, summary_path, old_sidecar_path, mut input) =
        active_recorded_material_review_fixture(&authority);
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
    bind_review_input_to_request(&mut input, &request);
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

    // Exact current approval does not repair invalid prior business evidence.
    authority.approve(
        root,
        "WI-MATERIAL",
        HumanReviewOperation::PreflightReview,
        &json!({"decision":"confirm_review"}),
    );
    authority.approve(
        root,
        "WI-MATERIAL",
        HumanReviewOperation::MaterialReview,
        &json!({"input":input,"runtime":runtime}),
    );
    assert!(
        !cockpit_repository::work_item_status_snapshot_with_runtime(root, "WI-MATERIAL", &runtime)
            .unwrap()
            .human_decision_required
    );
    let summary_before = fs::read(&summary_path).expect("Summary before refused renewal");
    let current_sidecar = material_receipt_path(root, &request);
    let error =
        cockpit_repository::record_work_item_material_review(root, "WI-MATERIAL", &input, &runtime)
            .expect_err("changed Contract/profile must reject prior receipt renewal");
    assert!(
        error
            .to_string()
            .contains("record_material_review_decision")
            || error.to_string().contains("material-review")
    );
    assert!(
        error
            .to_string()
            .contains("material_review_receipt_invalid"),
        "target prior-receipt validation must reject: {error}"
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
    let authority = TestAuthority::install();
    let (directory, runtime, contract_path, summary_path, old_sidecar_path, mut input) =
        active_recorded_material_review_fixture(&authority);
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
    bind_review_input_to_request(&mut input, &request);

    cockpit_repository::preflight_work_item_with_runtime(root, &contract_path, &runtime)
        .expect("refresh preflight for the amended Contract");
    authority.approve(
        root,
        "WI-MATERIAL",
        HumanReviewOperation::PreflightReview,
        &json!({"decision":"confirm_review"}),
    );

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

    authority.approve(
        root,
        "WI-MATERIAL",
        HumanReviewOperation::MaterialReview,
        &json!({"input": input, "runtime": runtime}),
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
    let authority = TestAuthority::install();
    let (directory, runtime, contract_path, _summary_path, _old_sidecar_path, _input) =
        active_recorded_material_review_fixture(&authority);
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
    let authority = TestAuthority::install();
    let (directory, runtime, contract_path, _summary_path, _old_sidecar_path, _input) =
        active_recorded_material_review_fixture(&authority);
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
    let authority = TestAuthority::install();
    let (directory, runtime, contract_path, _summary_path, old_sidecar_path, _input) =
        active_recorded_material_review_fixture(&authority);
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

fn current_preflight_mirror(root: &Path) -> (PathBuf, serde_json::Value) {
    let proof = crate::review_authority::current_preflight_review(root, "WI-MATERIAL").unwrap();
    let path = root.join(format!(
        ".ai/decisions/WI-MATERIAL.human-approval.{}.json",
        proof.raw_digest().as_str().trim_start_matches("sha256:")
    ));
    (path, serde_json::to_value(proof.receipt()).unwrap())
}

#[test]
fn copied_checkout_never_inherits_current_material_or_preflight_permission() {
    let authority = TestAuthority::install();
    let (source, runtime, _path, _bytes, _pointer, input) = segmented_authority_fixture(&authority);
    let copied = tempfile::tempdir().unwrap();
    copy_segmented_fixture(source.path(), copied.path());
    let before = segmented_governance_bytes(copied.path());
    assert_eq!(
        crate::current_review_authority_state(source.path(), "WI-MATERIAL").unwrap(),
        cockpit_protocol::review_authority::CurrentReviewAuthorityState::Active
    );
    assert_ne!(
        crate::current_review_authority_state(copied.path(), "WI-MATERIAL").unwrap(),
        cockpit_protocol::review_authority::CurrentReviewAuthorityState::Active
    );
    let executions = authority.execution_count();
    let error =
        crate::record_work_item_material_review(copied.path(), "WI-MATERIAL", &input, &runtime)
            .unwrap_err();
    assert!(
        error.to_string().contains("human_review_not_active"),
        "{error}"
    );
    assert_eq!(authority.execution_count(), executions);
    assert_eq!(segmented_governance_bytes(copied.path()), before);
}

#[test]
fn material_completed_fact_survives_replacement_approval_and_revocation_without_reexecution() {
    let authority = TestAuthority::install();
    let (directory, runtime, _contract, _summary, sidecar, input) =
        active_recorded_material_review_fixture(&authority);
    let root = directory.path();
    let receipt: MaterialInspectionReviewDecisionReceipt =
        serde_json::from_slice(&fs::read(sidecar).unwrap()).unwrap();
    let value = serde_json::to_value(&receipt).unwrap();
    assert!(
        crate::review_authority::has_historical_human_effect(
            root,
            "WI-MATERIAL",
            HumanReviewOperation::MaterialReview,
            &value
        )
        .unwrap()
    );
    let executions = authority.execution_count();
    let before = segmented_governance_bytes(root);
    let replay =
        crate::record_work_item_material_review(root, "WI-MATERIAL", &input, &runtime).unwrap();
    assert_eq!(serde_json::to_value(replay).unwrap(), value);
    assert_eq!(segmented_governance_bytes(root), before);
    assert_eq!(authority.execution_count(), executions);
    let replacement = authority.approve(
        root,
        "WI-MATERIAL",
        HumanReviewOperation::MaterialReview,
        &json!({"input":input,"runtime":runtime}),
    );
    assert!(
        crate::review_authority::has_historical_human_effect(
            root,
            "WI-MATERIAL",
            HumanReviewOperation::MaterialReview,
            &value
        )
        .unwrap()
    );
    assert!(
        crate::review_authority::require_current_human_effect(
            root,
            "WI-MATERIAL",
            HumanReviewOperation::MaterialReview,
            &value
        )
        .is_err()
    );
    let after_approval = segmented_governance_bytes(root);
    let replay =
        crate::record_work_item_material_review(root, "WI-MATERIAL", &input, &runtime).unwrap();
    assert_eq!(serde_json::to_value(replay).unwrap(), value);
    assert_eq!(segmented_governance_bytes(root), after_approval);
    assert_eq!(authority.execution_count(), executions);
    authority.revoke_approval(root, "WI-MATERIAL", &replacement);
    assert!(
        crate::review_authority::has_historical_human_effect(
            root,
            "WI-MATERIAL",
            HumanReviewOperation::MaterialReview,
            &value
        )
        .unwrap()
    );
    assert!(
        crate::review_authority::require_current_human_effect(
            root,
            "WI-MATERIAL",
            HumanReviewOperation::MaterialReview,
            &value
        )
        .is_err()
    );
    let after_revocation = segmented_governance_bytes(root);
    let error =
        crate::record_work_item_material_review(root, "WI-MATERIAL", &input, &runtime).unwrap_err();
    assert!(
        error.to_string().contains("review_approval_revoked"),
        "{error}"
    );
    assert_eq!(segmented_governance_bytes(root), after_revocation);
    assert_eq!(authority.execution_count(), executions);
}

//! Business fixtures use an explicit private test port; no real human or production authority is supplied.
#![cfg(test)]

use crate as cockpit_repository;
use crate::review_authority::pipeline_tests::TestAuthority;
use cockpit_core::{DecisionState, Digest};
use cockpit_protocol::review_authority::HumanReviewOperation;
use cockpit_protocol::{
    MATERIAL_INSPECTION_REVIEW_CAPABILITY, MaterialInspectionReviewDecisionInput,
    ResourceFinalizationContext, RuntimeContext, VerificationStage,
};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_work_item_with_runtime, attach, checkpoint_work_item,
    close_work_item_with_structured_decision, evaluate_contract_quality_gate,
    finish_work_item_with_runtime, persist_verification_attempt, plan_resource_finalization,
    plan_work_item_material_review, preflight_work_item, preflight_work_item_with_runtime,
    record_verification_with_runtime, record_work_item_material_review,
    run_repository_verification, start_work_item_with_options,
    validate_contract_quality_gate_report, work_item_status_snapshot_with_runtime,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn git(repository: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .args(args)
            .current_dir(repository)
            .status()
            .expect("git command")
            .success(),
        "git command failed: {args:?}"
    );
}

fn git_revision(repository: &Path) -> String {
    git_output(repository, &["rev-parse", "HEAD"])
}

fn git_output(repository: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(repository)
        .output()
        .expect("git output");
    assert!(
        output.status.success(),
        "git command failed: {args:?}: {output:?}"
    );
    String::from_utf8(output.stdout)
        .expect("git output UTF-8")
        .trim()
        .to_owned()
}

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("tempdir");
    let root = directory.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.name", "CI gate test"]);
    git(root, &["config", "user.email", "ci-gate@example.invalid"]);
    fs::write(root.join("README.md"), "CI gate fixture\n").expect("fixture");
    fs::write(root.join("pyproject.toml"), "fail_under = 90\n").expect("coverage fixture");
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "base"]);
    attach(root).expect("attach");
    start_work_item_with_options(
        root,
        "WI-CI-GATE",
        "make the CI route consume the Contract",
        "validate a repository-bound read-only quality gate",
        &[
            "crates/**".into(),
            "tests/ci/**".into(),
            "README.md".into(),
            "pyproject.toml".into(),
        ],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            risk: "normal".into(),
            acceptance_criteria: vec!["the gate remains read-only".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    directory
}

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "test-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"test-runtime"),
    }
}

fn verification_exit_command(passed: bool) -> (String, Vec<String>) {
    #[cfg(windows)]
    {
        let command = if passed { "exit 0" } else { "exit 1" };
        (
            std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".into()),
            vec!["/C".into(), command.into()],
        )
    }
    #[cfg(not(windows))]
    {
        (if passed { "true" } else { "false" }.into(), Vec::new())
    }
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

fn contract_path(root: &Path) -> PathBuf {
    root.join(".ai/work-items/active/WI-CI-GATE.contract.json")
}

fn reviewed_material_repository(authority: &TestAuthority) -> (tempfile::TempDir, PathBuf) {
    let directory = repository();
    let root = directory.path();
    let contract = contract_path(root);
    fs::create_dir_all(root.join("crates")).expect("source directory");
    fs::write(
        root.join("crates/material.rs"),
        benign_syntax_unknown_source(),
    )
    .expect("material source");
    git(root, &["add", "crates/material.rs"]);
    git(root, &["commit", "-qm", "add bounded syntax Unknown"]);

    let mut contract_value: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract).expect("Contract bytes"))
            .expect("Contract JSON");
    contract_value["governanceProfile"] = serde_json::json!({
        "materialInspectionReview": {
            "schemaVersion": 1,
            "permittedUnknown": "repository_material_inspection_unavailable",
            "permittedCause": "readable_committed_rust_syntax_unknown",
            "assurance": "self_declared",
            "reviewerActor": "agent:Raydot",
            "authoritySource": "user-delegation:ray-approved-WI1068",
            "acceptResidualRisk": true
        }
    });
    contract_value["requiredRuntimeCapabilities"] =
        serde_json::json!([MATERIAL_INSPECTION_REVIEW_CAPABILITY]);
    fs::write(
        &contract,
        serde_json::to_vec_pretty(&contract_value).expect("Contract JSON bytes"),
    )
    .expect("write material-review Contract");

    let request = plan_work_item_material_review(root, "WI-CI-GATE").expect("plan material review");
    let current_runtime = runtime();
    preflight_work_item_with_runtime(root, &contract, &current_runtime).expect("fresh preflight");
    let input: MaterialInspectionReviewDecisionInput = serde_json::from_value(serde_json::json!({
        "schemaVersion":2,
        "decision":"accept_permitted_unknowns",
        "requestDigest":request.request_digest,
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
    .expect("typed material-review input");
    authority.approve(
        root,
        "WI-CI-GATE",
        HumanReviewOperation::MaterialReview,
        &serde_json::json!({"input": input, "runtime": current_runtime}),
    );
    record_work_item_material_review(root, "WI-CI-GATE", &input, &current_runtime)
        .expect("record typed material-review receipt");
    (directory, contract)
}

fn reviewed_material_repository_with_comparison_base_unknown(
    authority: &TestAuthority,
    comparison_material: &[u8],
) -> (tempfile::TempDir, PathBuf, String) {
    let directory = tempfile::tempdir().expect("tempdir");
    let root = directory.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.name", "CI gate test"]);
    git(root, &["config", "user.email", "ci-gate@example.invalid"]);
    fs::write(root.join("README.md"), "CI gate fixture\n").expect("fixture");
    fs::write(root.join("pyproject.toml"), "fail_under = 90\n").expect("coverage fixture");

    let comparison_path = "crates/comparison_base.rs";
    fs::create_dir_all(root.join("crates")).expect("source directory");
    fs::write(root.join(comparison_path), comparison_material).expect("base comparison material");
    git(root, &["add", "."]);
    git(
        root,
        &["commit", "-qm", "base with Rust comparison material"],
    );
    attach(root).expect("attach");
    start_work_item_with_options(
        root,
        "WI-CI-GATE",
        "make the CI route consume the Contract",
        "validate a repository-bound read-only quality gate",
        &[
            "crates/**".into(),
            "tests/ci/**".into(),
            "README.md".into(),
            "pyproject.toml".into(),
        ],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            risk: "normal".into(),
            acceptance_criteria: vec!["the gate remains read-only".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");

    fs::write(
        root.join(comparison_path),
        "fn comparison_material() -> i32 { 1 }\n",
    )
    .expect("clean comparison-base material");
    git(root, &["add", comparison_path]);
    git(root, &["commit", "-qm", "advance CI comparison base"]);
    let comparison_base = git_revision(root);

    fs::write(root.join(comparison_path), comparison_material)
        .expect("restore base comparison material in candidate");
    fs::write(
        root.join("crates/material.rs"),
        benign_syntax_unknown_source(),
    )
    .expect("reviewed source");
    git(root, &["add", comparison_path, "crates/material.rs"]);
    git(
        root,
        &[
            "commit",
            "-qm",
            "restore base material and add reviewed source",
        ],
    );

    let contract = contract_path(root);
    let mut contract_value: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract).expect("Contract bytes"))
            .expect("Contract JSON");
    contract_value["governanceProfile"] = serde_json::json!({
        "materialInspectionReview": {
            "schemaVersion": 1,
            "permittedUnknown": "repository_material_inspection_unavailable",
            "permittedCause": "readable_committed_rust_syntax_unknown",
            "assurance": "self_declared",
            "reviewerActor": "agent:Raydot",
            "authoritySource": "user-delegation:ray-approved-WI1068",
            "acceptResidualRisk": true
        }
    });
    contract_value["requiredRuntimeCapabilities"] =
        serde_json::json!([MATERIAL_INSPECTION_REVIEW_CAPABILITY]);
    fs::write(
        &contract,
        serde_json::to_vec_pretty(&contract_value).expect("Contract JSON bytes"),
    )
    .expect("write material-review Contract");

    let request =
        plan_work_item_material_review(root, "WI-CI-GATE").expect("plan canonical material review");
    assert!(
        !request
            .entries
            .iter()
            .any(|entry| entry.path == comparison_path),
        "comparison-base-only Rust path must not enter the Contract-base review manifest"
    );
    let current_runtime = runtime();
    preflight_work_item_with_runtime(root, &contract, &current_runtime).expect("fresh preflight");
    let input: MaterialInspectionReviewDecisionInput = serde_json::from_value(serde_json::json!({
        "schemaVersion":2,
        "decision":"accept_permitted_unknowns",
        "requestDigest":request.request_digest,
        "requestArtifactDigest":request.request_artifact.as_ref().unwrap().digest,
        "requestArtifactByteLength":request.request_artifact.as_ref().unwrap().byte_length,
        "reviewerActor": "agent:Raydot",
        "authoritySource": "user-delegation:ray-approved-WI1068",
        "assurance": "self_declared",
        "evidenceRefs": [{
            "path": "docs/review-evidence.md",
            "digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"
        }],
        "rationale": "Review only the exact Contract-base material request.",
        "residualRisk": "The CI comparison base may contain additional unreviewed Rust syntax."
    }))
    .expect("typed material-review input");
    authority.approve(
        root,
        "WI-CI-GATE",
        HumanReviewOperation::MaterialReview,
        &serde_json::json!({"input": input, "runtime": current_runtime}),
    );
    record_work_item_material_review(root, "WI-CI-GATE", &input, &current_runtime)
        .expect("record typed material-review receipt");

    (directory, contract, comparison_base)
}

fn pull_request_gate_report(
    root: &Path,
    contract: &Path,
) -> cockpit_repository::ContractQualityGateReport {
    let base = serde_json::from_slice::<serde_json::Value>(&fs::read(contract).unwrap()).unwrap()
        ["baseRevision"]
        .as_str()
        .unwrap()
        .to_owned();
    evaluate_contract_quality_gate(
        root,
        contract,
        VerificationStage::PullRequest,
        "hosted",
        Some(&base),
        &runtime(),
    )
    .expect("quality gate report")
}

fn archived_resource_repository() -> (tempfile::TempDir, PathBuf) {
    let directory = repository();
    let root = directory.path();
    let contract = contract_path(root);
    let mut value =
        serde_json::from_slice::<serde_json::Value>(&fs::read(&contract).expect("Contract"))
            .expect("Contract JSON");
    value["requiredEvidenceClasses"] =
        serde_json::json!(["verification", "external_evidence", "delegated:github"]);
    for field in [
        "predecessorWorkItemId",
        "predecessorContractDigest",
        "recoveryDecisionPath",
    ] {
        value[field] = serde_json::Value::Null;
    }
    fs::write(
        &contract,
        serde_json::to_vec_pretty(&value).expect("updated Contract JSON"),
    )
    .expect("write updated Contract");
    plan_resource_finalization(
        root,
        "WI-CI-GATE",
        &ResourceFinalizationContext {
            branch: "codex/archived-ci-gate".into(),
            worktree: root.to_string_lossy().into_owned(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: "https://github.com/example/repo/pull/814".into(),
        },
    )
    .expect("resource finalization plan");
    let current_runtime = runtime();
    let contract = contract_path(root);
    preflight_work_item_with_runtime(root, &contract, &current_runtime).expect("preflight");
    checkpoint_work_item(root, "WI-CI-GATE").expect("checkpoint");
    let run = run_repository_verification(
        root,
        &RepositoryVerificationRequest {
            node_id: "archived-ci-gate-verification".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec!["README.md".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: current_runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: None,
            timeout_seconds: None,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .expect("verification");
    record_verification_with_runtime(
        root,
        "WI-CI-GATE",
        &serde_json::to_value(&run.receipt).expect("verification JSON"),
        &current_runtime,
        &run.final_snapshot,
    )
    .expect("record verification");
    finish_work_item_with_runtime(root, "WI-CI-GATE", &current_runtime).expect("finish");
    archive_work_item_with_runtime(root, "WI-CI-GATE", &current_runtime).expect("archive");
    let archived_contract = root.join(".ai/work-items/archive/WI-CI-GATE.contract.json");
    (directory, archived_contract)
}

#[test]
fn quality_gate_recomputes_state_after_material_review_discharge() {
    let authority = TestAuthority::install();
    let (directory, contract) = reviewed_material_repository(&authority);
    let root = directory.path();
    let base = serde_json::from_slice::<serde_json::Value>(&fs::read(&contract).unwrap()).unwrap()
        ["baseRevision"]
        .as_str()
        .unwrap()
        .to_owned();

    let report = evaluate_contract_quality_gate(
        root,
        &contract,
        VerificationStage::PullRequest,
        "hosted",
        Some(&base),
        &runtime(),
    )
    .expect("quality gate report");

    assert_eq!(
        report.raw_scanner_unknowns,
        vec!["repository_material_inspection_unavailable"]
    );
    assert!(report.review_receipt_digest.is_some());
    assert_eq!(
        report.review_assurance,
        Some(cockpit_protocol::MaterialInspectionReviewAssurance::SelfDeclared)
    );
    assert!(report.effective_unknowns.is_empty());
    assert!(report.unknowns.is_empty());
    assert_eq!(report.decision_state, "green");
    assert_eq!(report.state, "passed");
}

#[test]
fn quality_gate_discharges_exact_reviewed_material_across_different_comparison_base() {
    let authority = TestAuthority::install();
    let (directory, contract, comparison_base) =
        reviewed_material_repository_with_comparison_base_unknown(
            &authority,
            b"fn comparison_material() -> i32 { 2 }\n",
        );
    let root = directory.path();

    let report = evaluate_contract_quality_gate(
        root,
        &contract,
        VerificationStage::PullRequest,
        "hosted",
        Some(&comparison_base),
        &runtime(),
    )
    .expect("exact reviewed material may be consumed from a different comparison base");

    assert_eq!(report.comparison_base_revision, comparison_base);
    assert_eq!(
        report.raw_scanner_unknowns,
        vec!["repository_material_inspection_unavailable"]
    );
    assert!(report.review_receipt_digest.is_some());
    assert!(report.effective_unknowns.is_empty());
    assert_eq!(report.state, "passed");
    assert_eq!(report.decision_state, "green");
}

#[test]
fn quality_gate_preserves_same_path_material_when_comparison_hunk_origin_differs() {
    let authority = TestAuthority::install();
    let (directory, contract) = reviewed_material_repository(&authority);
    let root = directory.path();
    let candidate_head = git_revision(root);
    let contract_base =
        serde_json::from_slice::<serde_json::Value>(&fs::read(&contract).expect("Contract bytes"))
            .expect("Contract JSON")["baseRevision"]
            .as_str()
            .expect("Contract base")
            .to_owned();
    let candidate_branch = git_output(root, &["branch", "--show-current"]);

    git(root, &["branch", "comparison-base", &contract_base]);
    git(root, &["checkout", "comparison-base"]);
    fs::create_dir_all(root.join("crates")).expect("source directory");
    fs::write(
        root.join("crates/material.rs"),
        "fn comparison_material() -> i32 { 2 }\n",
    )
    .expect("comparison-base material");
    git(root, &["add", "crates/material.rs"]);
    git(
        root,
        &["commit", "-qm", "add clean comparison-base material"],
    );
    let comparison_base = git_revision(root);

    git(root, &["checkout", &candidate_branch]);
    let candidate_tree = git_output(root, &["rev-parse", &format!("{candidate_head}^{{tree}}")]);
    let merged_head = git_output(
        root,
        &[
            "commit-tree",
            &candidate_tree,
            "-p",
            &candidate_head,
            "-p",
            &comparison_base,
            "-m",
            "merge tested comparison base",
        ],
    );
    git(root, &["reset", "--hard", &merged_head]);
    let merged_request = plan_work_item_material_review(root, "WI-CI-GATE")
        .expect("same canonical material after merge");
    assert_eq!(merged_request.reviewed_source_head, merged_head);

    let report = evaluate_contract_quality_gate(
        root,
        &contract,
        VerificationStage::PullRequest,
        "hosted",
        Some(&comparison_base),
        &runtime(),
    )
    .expect("hosted gate report");

    assert_eq!(report.comparison_base_revision, comparison_base);
    assert!(
        report
            .changed_paths
            .iter()
            .any(|path| path == "crates/material.rs"),
        "the CI diff changes the same path reviewed as an addition against the Contract base"
    );
    assert!(report.review_receipt_digest.is_some());
    assert!(
        report
            .effective_unknowns
            .contains(&"repository_material_inspection_unavailable".into()),
        "same-path material with a different change origin/hunk must remain Unknown"
    );
    assert_eq!(report.state, "blocked");
    assert_eq!(report.decision_state, "yellow");
}

#[test]
fn quality_gate_does_not_discharge_comparison_base_only_material_unknown() {
    let authority = TestAuthority::install();
    let (directory, contract, comparison_base) =
        reviewed_material_repository_with_comparison_base_unknown(
            &authority,
            benign_syntax_unknown_source().as_bytes(),
        );
    let root = directory.path();

    let report = evaluate_contract_quality_gate(
        root,
        &contract,
        VerificationStage::PullRequest,
        "hosted",
        Some(&comparison_base),
        &runtime(),
    )
    .expect("quality gate report");

    assert_eq!(report.comparison_base_revision, comparison_base);
    assert!(
        report
            .changed_paths
            .iter()
            .any(|path| path == "crates/comparison_base.rs"),
        "CI comparison diff must contain its additional Rust path"
    );
    assert!(report.review_receipt_digest.is_some());
    assert_eq!(
        report.review_assurance,
        Some(cockpit_protocol::MaterialInspectionReviewAssurance::SelfDeclared)
    );
    assert!(
        report
            .raw_scanner_unknowns
            .contains(&"repository_material_inspection_unavailable".into())
    );
    assert!(
        report
            .effective_unknowns
            .contains(&"repository_material_inspection_unavailable".into()),
        "the Contract-base receipt must not discharge an extra comparison-base Unknown"
    );
    assert_eq!(report.state, "blocked");
    assert_eq!(report.decision_state, "yellow");
}

#[test]
fn quality_gate_keeps_unreviewable_comparison_base_material_unknown() {
    let authority = TestAuthority::install();
    let (directory, contract, comparison_base) =
        reviewed_material_repository_with_comparison_base_unknown(
            &authority,
            b"\0binary\xffmaterial\0",
        );
    let root = directory.path();

    let report = evaluate_contract_quality_gate(
        root,
        &contract,
        VerificationStage::PullRequest,
        "hosted",
        Some(&comparison_base),
        &runtime(),
    )
    .expect("quality gate report");

    assert_eq!(report.comparison_base_revision, comparison_base);
    assert!(
        report
            .changed_paths
            .iter()
            .any(|path| path == "crates/comparison_base.rs"),
        "CI comparison diff must contain the binary material path"
    );
    assert!(report.review_receipt_digest.is_some());
    assert_eq!(
        report.review_assurance,
        Some(cockpit_protocol::MaterialInspectionReviewAssurance::SelfDeclared)
    );
    assert!(
        report
            .raw_scanner_unknowns
            .contains(&"repository_material_inspection_unavailable".into())
    );
    assert!(
        report
            .effective_unknowns
            .contains(&"repository_material_inspection_unavailable".into()),
        "a canonical receipt cannot discharge unreviewable comparison-base material"
    );
    assert_eq!(report.state, "blocked");
    assert_eq!(report.decision_state, "yellow");
}

#[test]
fn quality_gate_keeps_material_unknown_when_review_receipt_is_tampered() {
    let authority = TestAuthority::install();
    let (directory, contract) = reviewed_material_repository(&authority);
    let root = directory.path();
    let summary_path = root.join(".ai/work-items/active/WI-CI-GATE.summary.json");
    let summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary bytes"))
            .expect("Summary JSON");
    let receipt_path = root.join(
        summary["materialReviewReceipt"]["path"]
            .as_str()
            .expect("receipt sidecar path"),
    );
    fs::write(&receipt_path, b"tampered receipt\n").expect("tamper receipt sidecar");

    let report = pull_request_gate_report(root, &contract);

    assert_eq!(report.state, "blocked");
    assert_eq!(report.decision_state, "yellow");
    assert_eq!(report.review_receipt_digest, None);
    assert!(
        report
            .raw_scanner_unknowns
            .contains(&"repository_material_inspection_unavailable".into())
    );
    assert!(
        report
            .effective_unknowns
            .contains(&"repository_material_inspection_unavailable".into())
    );
    assert!(
        report
            .effective_unknowns
            .contains(&"material_review_receipt_invalid".into())
    );
}

#[test]
fn quality_gate_keeps_material_unknown_when_review_receipt_is_stale() {
    let authority = TestAuthority::install();
    let (directory, contract) = reviewed_material_repository(&authority);
    let root = directory.path();
    let source_path = root.join("crates/material.rs");
    let mut source = fs::read_to_string(&source_path).expect("material source");
    source.push_str("\n// changed after the review receipt\n");
    fs::write(&source_path, source).expect("change reviewed source");
    git(root, &["add", "crates/material.rs"]);
    git(
        root,
        &["commit", "-qm", "change reviewed material after receipt"],
    );

    let report = pull_request_gate_report(root, &contract);

    assert_eq!(report.state, "blocked");
    assert_eq!(report.decision_state, "yellow");
    assert_eq!(report.review_receipt_digest, None);
    assert!(
        report
            .effective_unknowns
            .contains(&"repository_material_inspection_unavailable".into())
    );
    assert!(
        report
            .effective_unknowns
            .contains(&"material_review_receipt_stale".into())
    );
}

#[test]
fn quality_gate_keeps_material_unknown_when_a_second_unknown_is_added() {
    let authority = TestAuthority::install();
    let (directory, contract) = reviewed_material_repository(&authority);
    let root = directory.path();
    fs::write(root.join("crates/second.rs"), b"binary\0material")
        .expect("second non-reviewable Rust Unknown");
    git(root, &["add", "crates/second.rs"]);
    git(
        root,
        &["commit", "-qm", "add a second non-reviewable Unknown"],
    );

    let current_request =
        plan_work_item_material_review(root, "WI-CI-GATE").expect("current material request");
    let second = current_request
        .entries
        .iter()
        .find(|entry| entry.path == "crates/second.rs")
        .expect("second material entry");
    assert_eq!(
        serde_json::to_value(&second.scanner_assessment).expect("assessment JSON"),
        serde_json::json!("unknown")
    );
    assert!(!second.reviewable);

    let report = pull_request_gate_report(root, &contract);

    assert_eq!(report.state, "blocked");
    assert_eq!(report.decision_state, "yellow");
    assert_eq!(report.review_receipt_digest, None);
    assert!(
        report
            .effective_unknowns
            .contains(&"repository_material_inspection_unavailable".into())
    );
    assert!(
        report
            .effective_unknowns
            .contains(&"material_review_receipt_stale".into())
    );
}

#[test]
fn report_validator_rejects_self_reported_green_with_an_extra_material_unknown() {
    let authority = TestAuthority::install();
    let (directory, contract, comparison_base) =
        reviewed_material_repository_with_comparison_base_unknown(
            &authority,
            benign_syntax_unknown_source().as_bytes(),
        );
    let root = directory.path();

    let actual = evaluate_contract_quality_gate(
        root,
        &contract,
        VerificationStage::PullRequest,
        "hosted",
        Some(&comparison_base),
        &runtime(),
    )
    .expect("canonical gate report");
    assert_eq!(actual.state, "blocked");
    assert!(!actual.raw_scanner_unknowns.is_empty());
    assert_eq!(actual.decision_state, "yellow");
    assert!(actual.review_receipt_digest.is_some());
    assert!(
        actual
            .effective_unknowns
            .iter()
            .any(|unknown| { unknown == "repository_material_inspection_unavailable" })
    );
    assert!(
        actual
            .changed_paths
            .iter()
            .any(|path| path == "crates/comparison_base.rs")
    );

    let mut forged = serde_json::to_value(&actual).expect("report JSON");
    forged["state"] = "passed".into();
    forged["decisionState"] = "green".into();
    forged["blockers"] = serde_json::json!([]);
    forged["unknowns"] = serde_json::json!([]);
    forged["effectiveUnknowns"] = serde_json::json!([]);
    let mut payload = forged.clone();
    payload
        .as_object_mut()
        .expect("report object")
        .remove("receiptDigest");
    forged["receiptDigest"] = cockpit_protocol::digest_json(&payload)
        .expect("recomputed forged digest")
        .to_string()
        .into();
    let report_path = root.join(".ai/decisions/forged-quality-gate.json");
    fs::write(
        &report_path,
        serde_json::to_vec_pretty(&forged).expect("forged report bytes"),
    )
    .expect("write forged report");
    let route = serde_json::json!({
        "schemaVersion": 1,
        "kind": "repository_quality_route",
        "contractPath": ".ai/work-items/active/WI-CI-GATE.contract.json",
        "contractDigest": actual.contract_file_digest,
        "baseRevision": actual.comparison_base_revision,
        "stage": "pull_request"
    });
    let route_path = root.join(".ai/decisions/quality-route.json");
    fs::write(&route_path, serde_json::to_vec_pretty(&route).unwrap()).expect("write route");

    let error = validate_contract_quality_gate_report(root, &report_path, &route_path)
        .expect_err("self-reported green must not validate");
    assert!(error.to_string().contains("canonical material"), "{error}");
}

#[test]
fn valid_material_review_does_not_bypass_failed_finish_or_stale_archive_verification() {
    let authority = TestAuthority::install();
    let (directory, _contract) = reviewed_material_repository(&authority);
    let root = directory.path();
    let current_runtime = runtime();
    let status = work_item_status_snapshot_with_runtime(root, "WI-CI-GATE", &current_runtime)
        .expect("status with a current review receipt");
    assert!(status.review_receipt_digest.is_some());
    assert!(
        !status
            .effective_unknowns
            .contains(&"repository_material_inspection_unavailable".into())
    );
    checkpoint_work_item(root, "WI-CI-GATE").expect("checkpoint");

    let (failed_program, failed_args) = verification_exit_command(false);
    let failed_request = RepositoryVerificationRequest {
        node_id: "reviewed-material-failed-verification".into(),
        program: failed_program,
        args: failed_args,
        scope: vec!["crates/**".into()],
        stage: "task".into(),
        runner: "local".into(),
        runtime_digest: current_runtime.runtime_digest.to_string(),
        base_commit: None,
        workers: 1,
        work_item_id: Some("WI-CI-GATE".into()),
        timeout_seconds: None,
        policy: RepositoryVerificationPolicy::NeverReuse,
    };
    let failed_attempt = run_repository_verification(root, &failed_request);
    let failed_run = failed_attempt.expect("verification command should return a failed receipt");
    assert!(
        !failed_run.receipt.passed,
        "the failing command must not pass"
    );
    let failed_receipt = serde_json::to_value(&failed_run.receipt).expect("failed receipt JSON");
    let persisted_attempt = persist_verification_attempt(
        root,
        "WI-CI-GATE",
        std::slice::from_ref(&failed_request),
        &failed_run.final_snapshot,
        &current_runtime,
        "execution_failed",
        Some((
            "verification_execution",
            "verification command did not pass",
        )),
        Some(&failed_receipt),
    )
    .expect("persist the failed verification attempt");
    let attempt_path = persisted_attempt["path"].as_str().expect("attempt path");
    let stored_attempt: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join(attempt_path)).expect("attempt bytes"))
            .expect("attempt JSON");
    assert_eq!(stored_attempt["receipt"]["passed"], false);

    let record_error = record_verification_with_runtime(
        root,
        "WI-CI-GATE",
        &failed_receipt,
        &current_runtime,
        &failed_run.final_snapshot,
    )
    .expect_err("a failed attempt cannot be recorded as completion evidence");
    assert!(
        record_error
            .to_string()
            .contains("failed verification cannot be recorded as completion evidence"),
        "rejection must identify the receipt as not passed: {record_error}"
    );
    let finish_error = finish_work_item_with_runtime(root, "WI-CI-GATE", &current_runtime)
        .expect_err("a valid material review cannot replace failed verification");
    assert!(
        finish_error
            .to_string()
            .contains("verification_evidence_missing"),
        "unexpected finish rejection: {finish_error}"
    );

    let (directory, _contract) = reviewed_material_repository(&authority);
    let root = directory.path();
    let recorded_runtime = RuntimeContext {
        runtime_version: "recorded-verification-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"recorded-verification-runtime"),
    };
    let current_runtime = runtime();
    preflight_work_item_with_runtime(root, &contract_path(root), &recorded_runtime)
        .expect("preflight under the verification Runtime");
    checkpoint_work_item(root, "WI-CI-GATE").expect("checkpoint");
    let (passed_program, passed_args) = verification_exit_command(true);
    let run = run_repository_verification(
        root,
        &RepositoryVerificationRequest {
            node_id: "reviewed-material-stale-runtime-verification".into(),
            program: passed_program,
            args: passed_args,
            scope: vec!["crates/**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: recorded_runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: None,
            timeout_seconds: None,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .expect("verification run");
    record_verification_with_runtime(
        root,
        "WI-CI-GATE",
        &serde_json::to_value(&run.receipt).expect("verification receipt JSON"),
        &recorded_runtime,
        &run.final_snapshot,
    )
    .expect("record verification under its executing Runtime");
    finish_work_item_with_runtime(root, "WI-CI-GATE", &recorded_runtime)
        .expect("finish under the Runtime that produced verification");

    let status = work_item_status_snapshot_with_runtime(root, "WI-CI-GATE", &current_runtime)
        .expect("status under a different current Runtime");
    assert!(status.review_receipt_digest.is_some());
    assert!(
        !status
            .effective_unknowns
            .contains(&"repository_material_inspection_unavailable".into()),
        "the material review remains current while the verification binding is stale"
    );
    assert_ne!(status.verification, "verified");
    let archive_error = archive_work_item_with_runtime(root, "WI-CI-GATE", &current_runtime)
        .expect_err("a current material review cannot replace stale verification");
    assert!(
        archive_error.to_string().contains("verification")
            || archive_error.to_string().contains("Runtime"),
        "unexpected archive rejection: {archive_error}"
    );
}

#[test]
fn archived_resource_contract_still_blocks_close_until_later_evidence_exists() {
    let authority = TestAuthority::install();
    let (directory, _) = archived_resource_repository();
    let error = approved_close(&authority, directory.path(), "WI-CI-GATE")
        .expect_err("close must retain the later provider-evidence requirement");
    assert!(
        error.to_string().contains("required_evidence_missing"),
        "unexpected close error: {error}"
    );
}

#[test]
fn pre_execution_gate_reuses_canonical_preflight_review() {
    let authority = TestAuthority::install();
    let directory = repository();
    let root = directory.path();
    let contract = contract_path(root);
    let preflight = preflight_work_item(root, &contract).expect("preflight");
    assert_eq!(preflight.state, DecisionState::Green);
    authority.approve(
        root,
        "WI-CI-GATE",
        HumanReviewOperation::PreflightReview,
        &serde_json::json!({"decision":"confirm_review"}),
    );

    let base = serde_json::from_slice::<serde_json::Value>(&fs::read(&contract).unwrap()).unwrap()
        ["baseRevision"]
        .as_str()
        .unwrap()
        .to_owned();
    let report = evaluate_contract_quality_gate(
        root,
        &contract,
        VerificationStage::PullRequest,
        "hosted",
        Some(&base),
        &runtime(),
    )
    .expect("entry gate must reuse the canonical preflight review");
    assert_eq!(report.state, "passed");
    assert_eq!(report.decision_state, "green");
}

fn approved_close(
    authority: &TestAuthority,
    root: &Path,
    id: &str,
) -> Result<crate::LifecycleReceipt, crate::ObserverError> {
    let decision = cockpit_protocol::HumanDecision {
        decision: "approved".into(),
        actor: "human:synthetic-ci-fixture".into(),
        authority_source: "private test port only".into(),
        reason: "Exercise later evidence validation".into(),
        evidence_refs: Vec::new(),
        policy_refs: Vec::new(),
        decided_at: "2026-10-20T00:00:00Z".into(),
        resume_condition: None,
    };
    authority.approve(
        root,
        id,
        HumanReviewOperation::Close,
        &serde_json::json!({"decision": decision, "runtime": None::<RuntimeContext>}),
    );
    close_work_item_with_structured_decision(root, id, &decision)
}

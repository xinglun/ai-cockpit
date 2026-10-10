use cockpit_core::{DecisionState, Digest};
use cockpit_git::GitRepository;
use cockpit_protocol::{
    MATERIAL_INSPECTION_REVIEW_CAPABILITY, MaterialInspectionReviewDecisionInput,
    ResourceFinalizationContext, RuntimeContext, VerificationStage,
};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_work_item_with_runtime, attach, checkpoint_work_item, close_work_item_with_decision,
    evaluate_contract_quality_gate, finish_work_item_with_runtime,
    governance_decision_for_contract, persist_verification_attempt, plan_resource_finalization,
    plan_work_item_material_review, preflight_work_item, preflight_work_item_with_runtime,
    record_verification_with_runtime, record_work_item_governance_controls,
    record_work_item_material_review, run_repository_verification, start_work_item_with_options,
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

fn reviewed_material_repository() -> (tempfile::TempDir, PathBuf) {
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
        "schemaVersion": 1,
        "decision": "accept_permitted_unknowns",
        "requestDigest": request.request_digest,
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
    record_work_item_material_review(root, "WI-CI-GATE", &input, &current_runtime)
        .expect("record typed material-review receipt");
    (directory, contract)
}

fn reviewed_material_repository_with_comparison_base_unknown(
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
        "schemaVersion": 1,
        "decision": "accept_permitted_unknowns",
        "requestDigest": request.request_digest,
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

fn ai_bytes(root: &Path) -> Vec<(String, Vec<u8>)> {
    fn visit(root: &Path, current: &Path, output: &mut Vec<(String, Vec<u8>)>) {
        let mut entries = fs::read_dir(current)
            .expect("read directory")
            .map(|entry| entry.expect("directory entry").path())
            .collect::<Vec<_>>();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                visit(root, &path, output);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .expect("relative path")
                    .to_string_lossy()
                    .into_owned();
                output.push((relative, fs::read(&path).expect("read file")));
            }
        }
    }
    let mut output = Vec::new();
    visit(root, &root.join(".ai"), &mut output);
    output
}

#[test]
fn valid_gate_is_identity_bound_and_read_only() {
    let directory = repository();
    let before = ai_bytes(directory.path());
    let contract = contract_path(directory.path());
    let base = serde_json::from_slice::<serde_json::Value>(&fs::read(&contract).unwrap()).unwrap()
        ["baseRevision"]
        .as_str()
        .unwrap()
        .to_owned();
    let report = evaluate_contract_quality_gate(
        directory.path(),
        &contract,
        VerificationStage::PullRequest,
        "hosted",
        Some(&base),
        &runtime(),
    )
    .expect("valid gate");
    assert_eq!(report.state, "passed");
    assert_eq!(report.decision_state, "green");
    assert_eq!(report.stage, "pr");
    assert_eq!(report.runner, "hosted");
    assert_eq!(report.work_item_id, "WI-CI-GATE");
    assert_eq!(report.base_revision, base);
    assert_eq!(report.comparison_base_revision, base);
    assert_eq!(report.raw_scanner_unknowns, Vec::<String>::new());
    assert!(report.material_manifest_digest.is_some());
    assert_eq!(report.review_receipt_digest, None);
    assert_eq!(report.review_assurance, None);
    assert_eq!(report.effective_unknowns, report.unknowns);
    assert_eq!(
        report.repository_id.to_string(),
        cockpit_repository::repository_id(directory.path()).to_string()
    );
    assert!(report.receipt_digest.to_string().starts_with("sha256:"));
    assert_eq!(
        before,
        ai_bytes(directory.path()),
        "read-only gate changed .ai bytes"
    );
    let report_path = directory.path().join(".ai/decisions/quality-gate.json");
    fs::write(
        &report_path,
        serde_json::to_vec_pretty(&report).expect("report bytes"),
    )
    .expect("write report fixture");
    let route = serde_json::json!({
        "schemaVersion": 1,
        "kind": "repository_quality_route",
        "contractPath": ".ai/work-items/active/WI-CI-GATE.contract.json",
        "contractDigest": report.contract_file_digest,
        "baseRevision": report.comparison_base_revision,
        "stage": "pull_request"
    });
    let route_path = directory.path().join(".ai/decisions/quality-route.json");
    fs::write(&route_path, serde_json::to_vec_pretty(&route).unwrap())
        .expect("write route fixture");
    assert_eq!(
        validate_contract_quality_gate_report(directory.path(), &report_path, &route_path)
            .expect("validate freshly recomputed report"),
        report
    );
}

#[test]
fn quality_gate_recomputes_state_after_material_review_discharge() {
    let (directory, contract) = reviewed_material_repository();
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
    let (directory, contract, comparison_base) =
        reviewed_material_repository_with_comparison_base_unknown(
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
    let (directory, contract) = reviewed_material_repository();
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
    let (directory, contract, comparison_base) =
        reviewed_material_repository_with_comparison_base_unknown(
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
    let (directory, contract, comparison_base) =
        reviewed_material_repository_with_comparison_base_unknown(b"\0binary\xffmaterial\0");
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
    let (directory, contract) = reviewed_material_repository();
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
    let (directory, contract) = reviewed_material_repository();
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
    let (directory, contract) = reviewed_material_repository();
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
fn quality_gate_blocks_dirty_non_ai_source_before_material_review() {
    let directory = repository();
    let root = directory.path();
    let contract = contract_path(root);
    let base = serde_json::from_slice::<serde_json::Value>(&fs::read(&contract).unwrap()).unwrap()
        ["baseRevision"]
        .as_str()
        .unwrap()
        .to_owned();
    fs::write(root.join("README.md"), "uncommitted source change\n").expect("dirty source");

    let report = evaluate_contract_quality_gate(
        root,
        &contract,
        VerificationStage::PullRequest,
        "hosted",
        Some(&base),
        &runtime(),
    )
    .expect("quality gate returns a fail-closed projection");

    assert_eq!(report.state, "blocked");
    assert!(
        report
            .blockers
            .contains(&"material_review_projection_unavailable".into())
    );
    assert!(
        report
            .unknowns
            .contains(&"material_review_projection_unavailable".into())
    );
}

#[test]
fn quality_gate_preserves_each_material_finding_category_as_a_blocker() {
    let directory = repository();
    let root = directory.path();
    let contract = contract_path(root);
    let base = serde_json::from_slice::<serde_json::Value>(&fs::read(&contract).unwrap()).unwrap()
        ["baseRevision"]
        .as_str()
        .unwrap()
        .to_owned();
    let injection = include_str!(
        "../../../tests/conformance/fixtures/repository-prompt-injection/repository/material.txt"
    )
    .trim();
    let skip_decorator = ["@", "pytest", ".mark.skip", "(", "reason='disabled'", ")"].concat();
    fs::write(root.join("README.md"), format!("{injection}\n")).expect("injected documentation");
    fs::create_dir_all(root.join("tests/ci")).expect("CI test directory");
    fs::write(
        root.join("tests/ci/security.py"),
        format!("{skip_decorator}\ndef test_security():\n    pass\n"),
    )
    .expect("security test");
    fs::write(root.join("pyproject.toml"), "fail_under = 70\n").expect("lower coverage threshold");
    git(root, &["add", "."]);
    git(
        root,
        &["commit", "-qm", "exercise distinct material findings"],
    );

    let report = evaluate_contract_quality_gate(
        root,
        &contract,
        VerificationStage::PullRequest,
        "hosted",
        Some(&base),
        &runtime(),
    )
    .expect("quality gate report");
    assert_eq!(report.state, "blocked");
    assert_eq!(report.decision_state, "red");
    for finding in [
        "coverage_weakening",
        "repository_prompt_injection",
        "test_weakening",
    ] {
        assert!(
            report.blockers.iter().any(|blocker| blocker == finding),
            "missing {finding} in {:?}",
            report.blockers
        );
    }
}

#[test]
fn report_validator_rejects_self_reported_green_with_an_extra_material_unknown() {
    let (directory, contract, comparison_base) =
        reviewed_material_repository_with_comparison_base_unknown(
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
    let (directory, _contract) = reviewed_material_repository();
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

    let (directory, _contract) = reviewed_material_repository();
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
fn start_rejects_unsupported_required_evidence_class_before_verification() {
    let directory = repository();
    let error = start_work_item_with_options(
        directory.path(),
        "WI-UNSUPPORTED-EVIDENCE",
        "reject an invalid declaration",
        "keep invalid evidence requirements out of verification",
        &["crates/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            required_evidence_classes: vec!["public-install".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect_err("unsupported required evidence class must fail at start");
    let message = error.to_string();
    assert!(
        message.contains("unsupported required evidence class"),
        "{message}"
    );
    assert!(message.contains("verification"), "{message}");
    assert!(message.contains("delegated:<provider>"), "{message}");
    assert!(
        !directory
            .path()
            .join(".ai/work-items/active/WI-UNSUPPORTED-EVIDENCE.contract.json")
            .exists()
    );
}

#[test]
fn preflight_rejects_an_existing_unsupported_evidence_class_before_observation() {
    let directory = repository();
    let contract = contract_path(directory.path());
    let mut value =
        serde_json::from_slice::<serde_json::Value>(&fs::read(&contract).unwrap()).unwrap();
    value["requiredEvidenceClasses"] = serde_json::json!(["public-install"]);
    fs::write(&contract, serde_json::to_vec_pretty(&value).unwrap()).unwrap();

    let error = preflight_work_item(directory.path(), &contract)
        .expect_err("preflight must reject an unsupported existing declaration");
    let message = error.to_string();
    assert!(
        message.contains("unsupported required evidence class"),
        "{message}"
    );
    assert!(message.contains("public-install"), "{message}");
}

#[test]
fn archived_resource_pull_request_gate_is_read_only() {
    let (directory, archived_contract) = archived_resource_repository();
    let root = directory.path();
    let before = ai_bytes(root);
    let value = serde_json::from_slice::<serde_json::Value>(
        &fs::read(&archived_contract).expect("archived Contract"),
    )
    .expect("archived Contract JSON");
    let base = value["baseRevision"]
        .as_str()
        .expect("Contract base")
        .to_owned();

    let report = evaluate_contract_quality_gate(
        root,
        &archived_contract,
        VerificationStage::PullRequest,
        "hosted",
        Some(&base),
        &runtime(),
    )
    .expect("archived resource Contract must be accepted by the read-only PR gate");
    assert_eq!(report.state, "passed");
    assert_eq!(report.decision_state, "green");
    assert_eq!(report.work_item_id, "WI-CI-GATE");
    assert_eq!(
        before,
        ai_bytes(root),
        "archived gate changed mutable .ai bytes"
    );
}

#[test]
fn archived_resource_contract_still_blocks_close_until_later_evidence_exists() {
    let (directory, _) = archived_resource_repository();
    let error = close_work_item_with_decision(directory.path(), "WI-CI-GATE", "approved")
        .expect_err("close must retain the later provider-evidence requirement");
    assert!(
        error.to_string().contains("required_evidence_missing"),
        "unexpected close error: {error}"
    );
}

#[test]
fn archived_resource_contract_is_restricted_to_pull_request_stage() {
    let (directory, archived_contract) = archived_resource_repository();
    let root = directory.path();
    let value = serde_json::from_slice::<serde_json::Value>(
        &fs::read(&archived_contract).expect("archived Contract"),
    )
    .expect("archived Contract JSON");
    let base = value["baseRevision"].as_str().expect("Contract base");
    let error = evaluate_contract_quality_gate(
        root,
        &archived_contract,
        VerificationStage::Merge,
        "hosted",
        Some(base),
        &runtime(),
    )
    .expect_err("archived Contract must not authorize a merge gate");
    assert!(
        error
            .to_string()
            .contains("restricted to pull-request stage")
    );
}

#[test]
fn archived_resource_contract_rejects_active_collision() {
    let (directory, archived_contract) = archived_resource_repository();
    let root = directory.path();
    let active_contract = root.join(".ai/work-items/active/WI-CI-GATE.contract.json");
    fs::copy(&archived_contract, &active_contract).expect("active collision Contract");
    let value = serde_json::from_slice::<serde_json::Value>(
        &fs::read(&archived_contract).expect("archived Contract"),
    )
    .expect("archived Contract JSON");
    let base = value["baseRevision"].as_str().expect("Contract base");
    let error = evaluate_contract_quality_gate(
        root,
        &archived_contract,
        VerificationStage::PullRequest,
        "hosted",
        Some(base),
        &runtime(),
    )
    .expect_err("active/archive collision must fail closed");
    assert!(
        error
            .to_string()
            .contains("active and archived Contract identities collide")
    );
}

#[test]
fn archived_resource_contract_rejects_manifest_digest_mismatch() {
    let (directory, archived_contract) = archived_resource_repository();
    let root = directory.path();
    let manifest_path = root.join(".ai/work-items/archive/WI-CI-GATE.archive.json");
    let mut manifest = serde_json::from_slice::<serde_json::Value>(
        &fs::read(&manifest_path).expect("archive manifest"),
    )
    .expect("archive manifest JSON");
    manifest["files"]["contractDigest"] = serde_json::json!("sha256:tampered");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("manifest bytes"),
    )
    .expect("tamper archive manifest");
    let value = serde_json::from_slice::<serde_json::Value>(
        &fs::read(&archived_contract).expect("archived Contract"),
    )
    .expect("archived Contract JSON");
    let base = value["baseRevision"].as_str().expect("Contract base");
    let error = evaluate_contract_quality_gate(
        root,
        &archived_contract,
        VerificationStage::PullRequest,
        "hosted",
        Some(base),
        &runtime(),
    )
    .expect_err("archive manifest digest mismatch must fail closed");
    assert!(error.to_string().contains("digest"));
}

#[cfg(unix)]
#[test]
fn archived_resource_contract_rejects_symlinked_manifest_file() {
    use std::os::unix::fs::symlink;

    let (directory, archived_contract) = archived_resource_repository();
    let root = directory.path();
    let events = root.join(".ai/work-items/archive/WI-CI-GATE.events.jsonl");
    let target = root.join("archived-events-copy.jsonl");
    fs::copy(&events, &target).expect("copy archive events");
    fs::remove_file(&events).expect("remove archive events");
    symlink(&target, &events).expect("symlink archive events");
    let value = serde_json::from_slice::<serde_json::Value>(
        &fs::read(&archived_contract).expect("archived Contract"),
    )
    .expect("archived Contract JSON");
    let base = value["baseRevision"].as_str().expect("Contract base");
    let error = evaluate_contract_quality_gate(
        root,
        &archived_contract,
        VerificationStage::PullRequest,
        "hosted",
        Some(base),
        &runtime(),
    )
    .expect_err("symlinked archived manifest file must fail closed");
    assert!(error.to_string().contains("regular non-symlink"));
}

#[test]
fn pre_execution_gate_defers_lifecycle_evidence_until_completion_stage() {
    let directory = repository();
    let root = directory.path();
    let contract = contract_path(root);
    let mut value =
        serde_json::from_slice::<serde_json::Value>(&fs::read(&contract).unwrap()).unwrap();
    value["requiredEvidenceClasses"] = serde_json::json!([
        "hosted-ci",
        "release-preflight",
        "public-install",
        "public-upgrade",
        "release-close",
        "cleanup"
    ]);
    fs::write(&contract, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    let base = value["baseRevision"].as_str().unwrap().to_owned();

    for stage in [VerificationStage::PullRequest, VerificationStage::Release] {
        let report = evaluate_contract_quality_gate(
            root,
            &contract,
            stage,
            "hosted",
            Some(&base),
            &runtime(),
        )
        .expect("pre-execution gate must not require future lifecycle evidence");
        assert_eq!(report.state, "passed");
        assert_eq!(report.decision_state, "green");
    }

    let snapshot = GitRepository::discover(root)
        .expect("git repository")
        .snapshot()
        .expect("repository snapshot");
    let lifecycle_decision = governance_decision_for_contract(
        root,
        &serde_json::from_slice(&fs::read(&contract).unwrap()).unwrap(),
        &snapshot,
    )
    .expect("lifecycle governance decision");
    assert_eq!(lifecycle_decision.state, DecisionState::Yellow);
    assert!(
        lifecycle_decision
            .unknowns
            .contains(&"required_evidence_missing".into()),
        "completion governance must still require the declared lifecycle evidence"
    );
}

#[test]
fn hosted_gate_does_not_block_on_a_local_runtime_receipt() {
    let directory = repository();
    let root = directory.path();
    let contract = contract_path(root);
    let base = serde_json::from_slice::<serde_json::Value>(&fs::read(&contract).unwrap()).unwrap()
        ["baseRevision"]
        .as_str()
        .unwrap()
        .to_owned();
    preflight_work_item(root, &contract).expect("preflight");
    checkpoint_work_item(root, "WI-CI-GATE").expect("checkpoint");

    let local_runtime = RuntimeContext {
        runtime_version: "developer-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"developer-runtime"),
    };
    let run = run_repository_verification(
        root,
        &RepositoryVerificationRequest {
            node_id: "local-runtime-receipt".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec!["crates/**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: local_runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: None,
            timeout_seconds: None,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .expect("local verification");
    record_verification_with_runtime(
        root,
        "WI-CI-GATE",
        &serde_json::to_value(&run.receipt).expect("receipt JSON"),
        &local_runtime,
        &run.final_snapshot,
    )
    .expect("record local receipt");

    let report = evaluate_contract_quality_gate(
        root,
        &contract,
        VerificationStage::PullRequest,
        "hosted",
        Some(&base),
        &runtime(),
    )
    .expect("hosted source gate must tolerate a local receipt identity");
    assert_eq!(report.state, "passed");
    assert_eq!(report.decision_state, "green");
}

#[test]
fn pre_execution_gate_reuses_canonical_preflight_review() {
    let directory = repository();
    let root = directory.path();
    let contract = contract_path(root);
    let preflight = preflight_work_item(root, &contract).expect("preflight");
    assert_eq!(preflight.state, DecisionState::Green);
    let summary: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(".ai/work-items/active/WI-CI-GATE.summary.json")).expect("summary"),
    )
    .expect("summary JSON");
    let contract_value: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract).expect("contract")).expect("contract JSON");
    let contract_digest = cockpit_protocol::digest_json(&contract_value).expect("contract digest");
    let review = serde_json::json!({
        "schemaVersion": 1,
        "decisionId": "contract-preflight-review",
        "decision": "confirm_review",
        "workItemId": "WI-CI-GATE",
        "repositoryId": cockpit_repository::repository_id(root),
        "contractDigest": contract_digest,
        "preflightDecisionDigest": summary["preflightDecisionDigest"].clone(),
        "repositorySnapshotDigest": summary["preflightRepositorySnapshotDigest"].clone(),
        "recordedAt": "2026-09-12T01:00:00Z",
        "recordedBy": "human:owner",
        "reason": "bounded preflight review confirmed"
    });
    record_work_item_governance_controls(
        root,
        "WI-CI-GATE",
        &serde_json::json!({"decisionEvidence": review}),
    )
    .expect("record preflight review");

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

#[test]
fn hosted_gate_distinguishes_contract_baseline_from_ci_comparison_baseline() {
    let directory = repository();
    let root = directory.path();
    let contract = contract_path(root);
    let contract_base = serde_json::from_slice::<serde_json::Value>(&fs::read(&contract).unwrap())
        .unwrap()["baseRevision"]
        .as_str()
        .unwrap()
        .to_owned();
    fs::write(root.join("comparison.txt"), "new PR comparison base\n").expect("comparison");
    git(root, &["add", "comparison.txt"]);
    git(root, &["commit", "-qm", "advance comparison base"]);
    let comparison_base = git_revision(root);

    let report = evaluate_contract_quality_gate(
        root,
        &contract,
        VerificationStage::PullRequest,
        "hosted",
        Some(&comparison_base),
        &runtime(),
    )
    .expect("divergent baselines are valid when both are explicit");
    assert_eq!(report.base_revision, contract_base);
    assert_eq!(report.comparison_base_revision, comparison_base);
}

#[test]
fn hosted_gate_includes_committed_changes_from_the_ci_comparison_base() {
    let directory = repository();
    let root = directory.path();
    let contract = contract_path(root);
    let contract_base = serde_json::from_slice::<serde_json::Value>(&fs::read(&contract).unwrap())
        .unwrap()["baseRevision"]
        .as_str()
        .unwrap()
        .to_owned();
    fs::create_dir_all(root.join("crates")).expect("source directory");
    fs::write(root.join("crates/committed.rs"), "pub fn committed() {}\n").expect("change");
    git(root, &["add", "crates/committed.rs"]);
    git(root, &["commit", "-qm", "commit PR source change"]);

    let report = evaluate_contract_quality_gate(
        root,
        &contract,
        VerificationStage::PullRequest,
        "hosted",
        Some(&contract_base),
        &runtime(),
    )
    .expect("quality gate");
    assert_eq!(report.changed_paths, vec!["crates/committed.rs"]);
}

#[test]
fn foreign_base_or_repository_contract_fails_closed() {
    let directory = repository();
    let contract = contract_path(directory.path());
    let value = serde_json::from_slice::<serde_json::Value>(&fs::read(&contract).unwrap()).unwrap();
    let base = value["baseRevision"].as_str().unwrap().to_owned();
    let error = evaluate_contract_quality_gate(
        directory.path(),
        &contract,
        VerificationStage::PullRequest,
        "hosted",
        Some("0000000000000000000000000000000000000000"),
        &runtime(),
    )
    .expect_err("foreign base must fail");
    assert!(error.to_string().contains("baseRevision"));

    let mut foreign = value;
    foreign["repositoryId"] = serde_json::json!("sha256:");
    fs::write(&contract, serde_json::to_vec_pretty(&foreign).unwrap()).unwrap();
    let error = evaluate_contract_quality_gate(
        directory.path(),
        &contract,
        VerificationStage::PullRequest,
        "hosted",
        Some(&base),
        &runtime(),
    )
    .expect_err("foreign repository must fail");
    assert!(error.to_string().contains("repositoryId"));
}

#[cfg(unix)]
#[test]
fn symlink_contract_is_rejected_before_reading() {
    let directory = repository();
    let contract = contract_path(directory.path());
    let link = directory
        .path()
        .join(".ai/work-items/active/WI-CI-GATE-link.contract.json");
    std::os::unix::fs::symlink(&contract, &link).expect("symlink");
    let error = evaluate_contract_quality_gate(
        directory.path(),
        &link,
        VerificationStage::PullRequest,
        "hosted",
        None,
        &runtime(),
    )
    .expect_err("symlink must fail");
    assert!(error.to_string().contains("regular non-symlink"));
}

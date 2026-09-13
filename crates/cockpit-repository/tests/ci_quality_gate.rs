use cockpit_core::{DecisionState, Digest};
use cockpit_git::GitRepository;
use cockpit_protocol::{ResourceFinalizationContext, RuntimeContext, VerificationStage};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_work_item_with_runtime, attach, checkpoint_work_item, evaluate_contract_quality_gate,
    finish_work_item_with_runtime, governance_decision_for_contract, plan_resource_finalization,
    preflight_work_item, preflight_work_item_with_runtime, record_verification_with_runtime,
    record_work_item_governance_controls, run_repository_verification,
    start_work_item_with_options,
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
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repository)
        .output()
        .expect("git revision");
    assert!(output.status.success(), "git rev-parse failed: {output:?}");
    String::from_utf8(output.stdout)
        .expect("git revision UTF-8")
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
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "base"]);
    attach(root).expect("attach");
    start_work_item_with_options(
        root,
        "WI-CI-GATE",
        "make the CI route consume the Contract",
        "validate a repository-bound read-only quality gate",
        &["crates/**".into(), "tests/ci/**".into()],
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

fn contract_path(root: &Path) -> PathBuf {
    root.join(".ai/work-items/active/WI-CI-GATE.contract.json")
}

fn archived_resource_repository() -> (tempfile::TempDir, PathBuf) {
    let directory = repository();
    let root = directory.path();
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

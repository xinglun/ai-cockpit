use cockpit_core::Digest;
use cockpit_protocol::{RuntimeContext, VerificationStage};
use cockpit_repository::{
    WorkItemStartOptions, attach, evaluate_contract_quality_gate, start_work_item_with_options,
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

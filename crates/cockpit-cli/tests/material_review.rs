use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("run git fixture command");
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn plan_fixture() -> tempfile::TempDir {
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
    let contract = json!({
        "protocolVersion": 1,
        "repositoryId": cockpit_repository::repository_id(root).to_string(),
        "workItemId": "WI-MATERIAL",
        "intent": "review bounded committed material",
        "goal": "retain exact material evidence",
        "scope": ["README.md"],
        "outOfScope": [],
        "risk": "high",
        "authority": "authorized",
        "acceptanceCriteria": ["exact material request"],
        "requiredEvidenceClasses": [],
        "verification": ["true"],
        "baseRevision": base,
        "projectProfileDigest": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "repositorySnapshotDigest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"
    });
    let active = root.join(".ai/work-items/active");
    fs::create_dir_all(&active).unwrap();
    fs::write(
        active.join("WI-MATERIAL.contract.json"),
        serde_json::to_vec_pretty(&contract).unwrap(),
    )
    .unwrap();
    directory
}

#[test]
fn material_review_command_is_discoverable_from_work_item_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["work-item", "--help"])
        .output()
        .expect("run ai-cockpit work-item help");

    assert!(
        output.status.success(),
        "work-item help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(
        help.contains("material-review"),
        "work-item help must expose material-review; got:\n{help}"
    );
}

#[test]
fn material_review_record_help_exposes_admission_and_self_declared_limits() {
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["work-item", "material-review", "record", "--help"])
        .output()
        .expect("run material review record help");

    assert!(
        output.status.success(),
        "record help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(
        help.contains("self-declared"),
        "missing assurance boundary: {help}"
    );
    assert!(help.contains("human"), "missing approval boundary: {help}");
    assert!(
        help.contains("--input"),
        "missing strict input option: {help}"
    );
}

#[test]
fn material_review_plan_is_read_only_and_reports_stage_one_disabled_state() {
    let directory = plan_fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args([
            "work-item",
            "material-review",
            "plan",
            "--repo",
            directory.path().to_str().unwrap(),
            "--id",
            "WI-MATERIAL",
        ])
        .output()
        .expect("run material review plan");

    assert!(
        output.status.success(),
        "material-review plan failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let request: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(request["workItemId"], "WI-MATERIAL");
    assert_eq!(request["reviewEnabled"], false);
    assert_eq!(request["reviewDiagnostic"], "material_review_not_enabled");
    assert_eq!(request["rawUnknownCodes"], json!([]));

    let status = git(directory.path(), &["status", "--porcelain"]);
    assert_eq!(status, "?? .ai/");
}

#[test]
fn material_review_record_fails_closed_when_stage_one_has_no_opt_in() {
    let directory = plan_fixture();
    let root = directory.path();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let plan = Command::new(binary)
        .args([
            "work-item",
            "material-review",
            "plan",
            "--repo",
            root.to_str().unwrap(),
            "--id",
            "WI-MATERIAL",
        ])
        .output()
        .expect("run material review plan");
    assert!(plan.status.success());
    let plan: serde_json::Value = serde_json::from_slice(&plan.stdout).unwrap();
    let input = json!({
        "schemaVersion": 1,
        "decision": "accept_permitted_unknowns",
        "requestDigest": plan["requestDigest"],
        "reviewerActor": "agent:Raydot",
        "authoritySource": "user-delegation:ray-approved-WI1068",
        "assurance": "self_declared",
        "evidenceRefs": [{
            "path": "docs/review-evidence.md",
            "digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"
        }],
        "rationale": "Review the exact bounded material request.",
        "residualRisk": "The scanner remains incomplete for permitted syntax."
    });
    let input_path = root.join(".ai/material-review-input.json");
    fs::write(&input_path, serde_json::to_vec_pretty(&input).unwrap()).unwrap();

    let record = Command::new(binary)
        .args([
            "work-item",
            "material-review",
            "record",
            "--repo",
            root.to_str().unwrap(),
            "--id",
            "WI-MATERIAL",
            "--input",
            input_path.to_str().unwrap(),
        ])
        .output()
        .expect("run material review record");

    assert!(!record.status.success(), "disabled review must fail closed");
    let stderr = String::from_utf8_lossy(&record.stderr);
    assert!(
        stderr.contains("not enabled"),
        "expected disabled-profile diagnostic, got: {stderr}"
    );
    assert!(
        !root
            .join(".ai/evidence/material-inspection-review")
            .exists()
    );
    assert!(
        root.join(".ai/work-items/active/WI-MATERIAL.summary.json")
            .exists()
            == false
    );
}

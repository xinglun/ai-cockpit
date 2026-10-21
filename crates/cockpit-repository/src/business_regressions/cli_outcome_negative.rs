//! Native closed fixtures; the actual CLI must report each malformed decision, never a setup error.
#![cfg(test)]
use crate::review_authority::pipeline_tests::TestAuthority;
use std::{fs, path::Path, process::Command};
fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("tempdir");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    for (key, value) in [
        ("user.email", "test@example.invalid"),
        ("user.name", "Test"),
    ] {
        assert!(
            Command::new("git")
                .args(["config", key, value])
                .current_dir(directory.path())
                .status()
                .expect("git config")
                .success()
        );
    }
    fs::write(directory.path().join("README.md"), "fixture baseline\n").expect("baseline README");
    assert!(
        Command::new("git")
            .args(["add", "README.md"])
            .current_dir(directory.path())
            .status()
            .expect("git add baseline")
            .success()
    );
    assert!(
        Command::new("git")
            .args(["commit", "-qm", "fixture baseline"])
            .current_dir(directory.path())
            .status()
            .expect("git commit baseline")
            .success()
    );
    directory
}
fn closed_work_item(authority: &TestAuthority, _binary: &str) -> tempfile::TempDir {
    let repo = repository();
    let root = repo.path();
    crate::attach(root).unwrap();
    let id = "WI-OUTCOME-DECISION";
    crate::start_work_item_with_options(
        root,
        id,
        "show structured human decision",
        "render a safe handoff",
        &["crates/**".into()],
        &crate::WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["human decision is visible".into()],
            required_evidence_classes: vec!["verification".into()],
            ..Default::default()
        },
    )
    .unwrap();
    crate::preflight_work_item(
        root,
        &root.join(format!(".ai/work-items/active/{id}.contract.json")),
    )
    .unwrap();
    crate::checkpoint_work_item(root, id).unwrap();
    let runtime = super::transport_fixture::runtime();
    let execution = crate::run_repository_verification(
        root,
        &crate::RepositoryVerificationRequest {
            node_id: "negative-outcome-fixture".into(),
            program: "true".into(),
            args: vec![],
            scope: vec!["**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: Some(id.into()),
            timeout_seconds: None,
            policy: crate::RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .unwrap();
    crate::record_verification_with_runtime(
        root,
        id,
        &serde_json::to_value(&execution.receipt).unwrap(),
        &runtime,
        &execution.final_snapshot,
    )
    .unwrap();
    crate::finish_work_item_with_runtime(root, id, &runtime).unwrap();
    crate::archive_work_item_with_runtime(root, id, &runtime).unwrap();
    // Ordinary close binds exact committed archive bytes; create that actual
    // commit in this disposable fixture before requesting the close grant.
    for args in [
        vec!["add", "-A"],
        vec!["commit", "-qm", "commit native archive"],
    ] {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    }
    let decision = cockpit_protocol::HumanDecision {
        decision: "approved".into(),
        actor: "human:owner".into(),
        authority_source: "project-policy".into(),
        reason: "fresh verification reviewed".into(),
        evidence_refs: vec![format!(".ai/evidence/{id}.verification.json")],
        policy_refs: vec!["project-policy-v1".into()],
        decided_at: "2026-08-22T00:00:00Z".into(),
        resume_condition: Some("rerun verification after base revision changes".into()),
    };
    super::approved_boundary_fixtures::close_structured_runtime(
        authority, root, id, &decision, &runtime,
    )
    .unwrap();
    repo
}

fn human_outcome(binary: &str, repo: &Path, language: &str) -> String {
    let output = Command::new(binary)
        .args(["work-item", "outcome", "--repo"])
        .arg(repo)
        .args(["--id", "WI-OUTCOME-DECISION", "--view", "full"])
        .env("AI_COCKPIT_LANGUAGE", language)
        .output()
        .expect("human outcome");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("UTF-8 outcome")
}
#[test]
fn malformed_or_foreign_decision_records_are_visible_as_unknown() {
    let authority = TestAuthority::install();
    let binary_path = super::transport_fixture::binary();
    let binary = binary_path.to_str().unwrap();
    let repo = closed_work_item(&authority, binary);
    let decision_path = repo
        .path()
        .join(".ai/decisions/WI-OUTCOME-DECISION.close.json");
    let original: serde_json::Value =
        serde_json::from_slice(&fs::read(&decision_path).expect("decision")).expect("JSON");
    let mut record = original.clone();
    record["structuredDecision"]
        .as_object_mut()
        .expect("structured decision")
        .remove("actor");
    fs::write(
        &decision_path,
        serde_json::to_vec_pretty(&record).expect("serialize malformed decision"),
    )
    .expect("write malformed decision");
    let malformed = human_outcome(binary, repo.path(), "en");
    assert!(malformed.contains("Unknown: structured human decision record is invalid"));
    assert!(
        malformed.contains("structured decision fields are incomplete or unknown"),
        "{malformed}"
    );
    assert!(!malformed.contains("Actor: human:owner"));

    let mut record = original;
    record["workItemId"] = serde_json::Value::String("WI-FOREIGN".into());
    fs::write(
        &decision_path,
        serde_json::to_vec_pretty(&record).expect("serialize foreign decision"),
    )
    .expect("write foreign decision");
    let foreign = human_outcome(binary, repo.path(), "zh-CN");
    assert!(foreign.contains("未知：结构化人工决定记录无效"));
    assert!(
        foreign.contains("decision record Work Item binding is missing or mismatched"),
        "{foreign}"
    );
    assert!(!foreign.contains("决定: approved"));
}
#[cfg(unix)]
#[test]
fn symlink_decision_record_is_not_projected_as_valid() {
    let authority = TestAuthority::install();
    use std::os::unix::fs::symlink;

    let binary_path = super::transport_fixture::binary();
    let binary = binary_path.to_str().unwrap();
    let repo = closed_work_item(&authority, binary);
    let decision_path = repo
        .path()
        .join(".ai/decisions/WI-OUTCOME-DECISION.close.json");
    let target = repo.path().join("decision-target.json");
    fs::rename(&decision_path, &target).expect("move decision");
    symlink(&target, &decision_path).expect("symlink decision");
    let output = human_outcome(binary, repo.path(), "ja");
    assert!(output.contains("不明：構造化された人間の判断記録が無効です"));
    assert!(
        output.contains("decision record is not a regular file"),
        "{output}"
    );
    assert!(!output.contains("判断: approved"));
}

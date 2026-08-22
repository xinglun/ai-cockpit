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
    directory
}

fn run(binary: &str, args: &[&str], repo: &Path) -> serde_json::Value {
    let output = Command::new(binary)
        .args(args)
        .args(["--repo"])
        .arg(repo)
        .output()
        .expect("run command");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("JSON output")
}

fn closed_work_item(binary: &str) -> tempfile::TempDir {
    let repo = repository();
    let attach = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(repo.path())
        .output()
        .expect("attach");
    assert!(attach.status.success());
    run(
        binary,
        &[
            "start",
            "--id",
            "WI-OUTCOME-DECISION",
            "--intent",
            "show structured human decision",
            "--goal",
            "render a safe handoff",
            "--scope",
            "crates/**",
            "--authority",
            "authorized",
            "--acceptance",
            "human decision is visible",
            "--required-evidence",
            "verification",
        ],
        repo.path(),
    );
    run(
        binary,
        &[
            "preflight",
            "--contract",
            ".ai/work-items/active/WI-OUTCOME-DECISION.contract.json",
        ],
        repo.path(),
    );
    run(
        binary,
        &["checkpoint", "--id", "WI-OUTCOME-DECISION"],
        repo.path(),
    );
    run(
        binary,
        &[
            "verify",
            "--work-item",
            "WI-OUTCOME-DECISION",
            "--command",
            "true",
        ],
        repo.path(),
    );
    run(
        binary,
        &["finish", "--id", "WI-OUTCOME-DECISION"],
        repo.path(),
    );
    run(
        binary,
        &["archive", "--id", "WI-OUTCOME-DECISION"],
        repo.path(),
    );
    run(
        binary,
        &[
            "close",
            "--id",
            "WI-OUTCOME-DECISION",
            "--human-decision",
            "approved-by-owner",
            "--actor",
            "human:owner",
            "--authority-source",
            "project-policy",
            "--reason",
            "fresh verification reviewed",
            "--evidence-ref",
            ".ai/evidence/WI-OUTCOME-DECISION.verification.json",
            "--policy-ref",
            "project-policy-v1",
            "--decided-at",
            "2026-08-22T00:00:00Z",
            "--resume-condition",
            "rerun verification after base revision changes",
        ],
        repo.path(),
    );
    repo
}

fn human_outcome(binary: &str, repo: &Path, language: &str) -> String {
    let output = Command::new(binary)
        .args(["work-item", "outcome", "--repo"])
        .arg(repo)
        .args(["--id", "WI-OUTCOME-DECISION"])
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
fn human_outcome_projects_structured_decision_in_all_supported_languages() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let repo = closed_work_item(binary);
    let machine = Command::new(binary)
        .args(["work-item", "outcome", "--repo"])
        .arg(repo.path())
        .args(["--id", "WI-OUTCOME-DECISION", "--json"])
        .output()
        .expect("machine outcome");
    assert!(machine.status.success());
    let machine_json: serde_json::Value = serde_json::from_slice(&machine.stdout).expect("JSON");
    assert_eq!(machine_json["schemaVersion"], 2);
    assert!(machine_json.get("humanDecision").is_none());

    for (language, heading, labels) in [
        (
            "en",
            "Human decisions",
            [
                "Decision: approved-by-owner",
                "Actor: human:owner",
                "Authority source: project-policy",
            ],
        ),
        (
            "zh-CN",
            "人工决定",
            [
                "决定: approved-by-owner",
                "执行人: human:owner",
                "授权来源: project-policy",
            ],
        ),
        (
            "ja",
            "人間の判断",
            [
                "判断: approved-by-owner",
                "実行者: human:owner",
                "権限の出所: project-policy",
            ],
        ),
    ] {
        let text = human_outcome(binary, repo.path(), language);
        assert!(text.contains("🟢"), "language={language}");
        assert!(text.contains(heading), "language={language}");
        for label in labels {
            assert!(text.contains(label), "language={language}, label={label}");
        }
        let next_heading = match language {
            "zh-CN" => "验证",
            "ja" => "検証",
            _ => "Verification",
        };
        let decisions_section = text
            .split(heading)
            .nth(1)
            .expect("decision heading")
            .split(next_heading)
            .next()
            .expect("verification heading");
        assert!(!decisions_section.contains("None"), "language={language}");
        assert!(!decisions_section.contains("无"), "language={language}");
        assert!(!decisions_section.contains("なし"), "language={language}");
    }
}

#[test]
fn malformed_or_foreign_decision_records_are_visible_as_unknown() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let repo = closed_work_item(binary);
    let decision_path = repo
        .path()
        .join(".ai/decisions/WI-OUTCOME-DECISION.close.json");
    let mut record: serde_json::Value =
        serde_json::from_slice(&fs::read(&decision_path).expect("decision")).expect("JSON");
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
    assert!(!malformed.contains("Actor: human:owner"));

    record["workItemId"] = serde_json::Value::String("WI-FOREIGN".into());
    fs::write(
        &decision_path,
        serde_json::to_vec_pretty(&record).expect("serialize foreign decision"),
    )
    .expect("write foreign decision");
    let foreign = human_outcome(binary, repo.path(), "zh-CN");
    assert!(foreign.contains("未知：结构化人工决定记录无效"));
    assert!(!foreign.contains("决定: approved-by-owner"));
}

#[cfg(unix)]
#[test]
fn symlink_decision_record_is_not_projected_as_valid() {
    use std::os::unix::fs::symlink;

    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let repo = closed_work_item(binary);
    let decision_path = repo
        .path()
        .join(".ai/decisions/WI-OUTCOME-DECISION.close.json");
    let target = repo.path().join("decision-target.json");
    fs::rename(&decision_path, &target).expect("move decision");
    symlink(&target, &decision_path).expect("symlink decision");
    let output = human_outcome(binary, repo.path(), "ja");
    assert!(output.contains("不明：構造化された人間の判断記録が無効です"));
    assert!(!output.contains("判断: approved-by-owner"));
}

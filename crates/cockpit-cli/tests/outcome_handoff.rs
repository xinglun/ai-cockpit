use std::{path::Path, process::Command};

mod common;

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

fn run(binary: &str, repo: &Path, args: &[&str]) -> std::process::Output {
    Command::new(binary)
        .args(args)
        .args(["--repo"])
        .arg(repo)
        .current_dir(repo)
        .output()
        .expect("run ai-cockpit")
}

fn run_json(binary: &str, repo: &Path, args: &[&str]) -> serde_json::Value {
    let output = run(binary, repo, args);
    assert!(
        output.status.success(),
        "args={args:?}, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("machine-readable stdout JSON")
}

fn checkpointed(binary: &str, work_item_id: &str, verified: bool) -> tempfile::TempDir {
    let repo = repository();
    run_json(binary, repo.path(), &["attach"]);
    run_json(
        binary,
        repo.path(),
        &[
            "start",
            "--id",
            work_item_id,
            "--intent",
            "make the lifecycle handoff directly visible",
            "--goal",
            "preserve machine output while presenting the human outcome",
            "--scope",
            "README.md",
            "--authority",
            "authorized",
            "--acceptance",
            "human outcome is directly visible",
            "--required-evidence",
            "verification",
        ],
    );
    common::plan(binary, repo.path(), work_item_id);
    run_json(
        binary,
        repo.path(),
        &[
            "preflight",
            "--contract",
            &format!(".ai/work-items/active/{work_item_id}.contract.json"),
        ],
    );
    run_json(binary, repo.path(), &["checkpoint", "--id", work_item_id]);
    if verified {
        run_json(
            binary,
            repo.path(),
            &["verify", "--work-item", work_item_id, "--command", "true"],
        );
    }
    repo
}

fn assert_handoff(stderr: &[u8], prefix: &str, sections: &[&str]) {
    let text = String::from_utf8(stderr.to_vec()).expect("UTF-8 human handoff");
    assert!(text.starts_with(prefix), "stderr={text}");
    for section in sections {
        assert!(
            text.contains(section),
            "missing {section:?} in stderr={text}"
        );
    }
}

#[test]
fn default_lifecycle_commands_emit_localized_handoffs_without_changing_stdout_json() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    for (language, id, success, unknowns, decisions, next, decision_label) in [
        (
            "en",
            "WI-HANDOFF-EN",
            "Outcome: 🟢 Success",
            "Unknowns",
            "Human decisions",
            "Next action",
            "Decision: approved",
        ),
        (
            "zh-CN",
            "WI-HANDOFF-ZH",
            "Outcome: 🟢 成功",
            "未知项",
            "人工决定",
            "下一步",
            "决定: approved",
        ),
        (
            "ja",
            "WI-HANDOFF-JA",
            "Outcome: 🟢 成功",
            "不明点",
            "人間の判断",
            "次のアクション",
            "判断: approved",
        ),
    ] {
        let repo = checkpointed(binary, id, true);
        let finish = Command::new(binary)
            .args(["finish", "--repo"])
            .arg(repo.path())
            .args(["--id", id])
            .env("AI_COCKPIT_LANGUAGE", language)
            .output()
            .expect("finish");
        assert!(finish.status.success());
        let finish_json: serde_json::Value =
            serde_json::from_slice(&finish.stdout).expect("finish stdout JSON");
        assert_eq!(finish_json["workItemId"], id);
        assert_eq!(finish_json["state"], "finish_ready");
        assert_handoff(&finish.stderr, success, &[unknowns, decisions, next]);

        let archive = Command::new(binary)
            .args(["archive", "--repo"])
            .arg(repo.path())
            .args(["--id", id])
            .env("AI_COCKPIT_LANGUAGE", language)
            .output()
            .expect("archive");
        assert!(archive.status.success());
        let archive_json: serde_json::Value =
            serde_json::from_slice(&archive.stdout).expect("archive stdout JSON");
        assert_eq!(archive_json["workItemId"], id);
        // Once the Work Item is archived, a bound provider context still
        // requires a valid provider-side finalization receipt.  The Runtime
        // therefore exposes a visible yellow handoff here; close becomes
        // green only after `record_deleted` binds that receipt below.
        let archive_prefix = match language {
            "en" => "Outcome: 🟡 Needs attention",
            "zh-CN" => "Outcome: 🟡 需要关注",
            "ja" => "Outcome: 🟡 要確認",
            _ => unreachable!(),
        };
        assert_handoff(
            &archive.stderr,
            archive_prefix,
            &[unknowns, decisions, next],
        );

        common::record_deleted(binary, repo.path(), id);
        let close = Command::new(binary)
            .args(["close", "--repo"])
            .arg(repo.path())
            .args([
                "--id",
                id,
                "--human-decision",
                "approved",
                "--actor",
                "human:owner",
                "--authority-source",
                "reviewed-evidence",
                "--reason",
                "the evidence was reviewed",
                "--evidence-ref",
                ".ai/evidence/verification.json",
                "--policy-ref",
                "repository-policy",
                "--decided-at",
                "2026-08-24T00:00:00Z",
                "--resume-condition",
                "none",
            ])
            .env("AI_COCKPIT_LANGUAGE", language)
            .output()
            .expect("close");
        assert!(
            close.status.success(),
            "close stderr={}",
            String::from_utf8_lossy(&close.stderr)
        );
        let close_json: serde_json::Value =
            serde_json::from_slice(&close.stdout).expect("close stdout JSON");
        assert_eq!(close_json["workItemId"], id);
        assert_handoff(
            &close.stderr,
            success,
            &[unknowns, decisions, next, decision_label],
        );
    }
}

#[test]
fn explicit_json_mode_suppresses_handoff_and_keeps_machine_stdout() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let id = "WI-HANDOFF-JSON";
    let repo = checkpointed(binary, id, true);

    for command in ["finish", "archive"] {
        let output = run(binary, repo.path(), &[command, "--id", id, "--json"]);
        assert!(
            output.status.success(),
            "{command} stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let json: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("lifecycle stdout JSON");
        assert_eq!(json["workItemId"], id);
        assert!(!String::from_utf8_lossy(&output.stderr).contains("Outcome:"));
    }

    common::record_deleted(binary, repo.path(), id);
    let close = run(
        binary,
        repo.path(),
        &[
            "close",
            "--id",
            id,
            "--json",
            "--human-decision",
            "approved",
            "--actor",
            "human:owner",
            "--authority-source",
            "reviewed-evidence",
            "--reason",
            "the evidence was reviewed",
            "--evidence-ref",
            ".ai/evidence/verification.json",
            "--policy-ref",
            "repository-policy",
            "--decided-at",
            "2026-08-24T00:00:00Z",
            "--resume-condition",
            "none",
        ],
    );
    assert!(
        close.status.success(),
        "close stderr={}",
        String::from_utf8_lossy(&close.stderr)
    );
    let close_json: serde_json::Value =
        serde_json::from_slice(&close.stdout).expect("close stdout JSON");
    assert_eq!(close_json["workItemId"], id);
    assert!(!String::from_utf8_lossy(&close.stderr).contains("Outcome:"));
}

#[test]
fn blocked_finish_emits_persisted_handoff_and_remains_nonzero() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    for (language, id, prefix, unknowns, next) in [
        (
            "en",
            "WI-HANDOFF-BLOCKED-EN",
            "Outcome: 🔴 Stop",
            "Unknowns",
            "Next action",
        ),
        (
            "zh-CN",
            "WI-HANDOFF-BLOCKED-ZH",
            "Outcome: 🔴 停止",
            "未知项",
            "下一步",
        ),
        (
            "ja",
            "WI-HANDOFF-BLOCKED-JA",
            "Outcome: 🔴 停止",
            "不明点",
            "次のアクション",
        ),
    ] {
        let repo = checkpointed(binary, id, false);
        let output = Command::new(binary)
            .args(["finish", "--repo"])
            .arg(repo.path())
            .args(["--id", id])
            .env("AI_COCKPIT_LANGUAGE", language)
            .output()
            .expect("blocked finish");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert_handoff(&output.stderr, prefix, &[unknowns, next]);
        assert!(
            repo.path()
                .join(format!(".ai/work-items/active/{id}.outcome.json"))
                .is_file()
        );
    }

    let id = "WI-HANDOFF-BLOCKED-JSON";
    let repo = checkpointed(binary, id, false);
    let output = run(binary, repo.path(), &["finish", "--id", id, "--json"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("Outcome:"));
}

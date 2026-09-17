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

fn compile_host_fixture(directory: &Path) -> std::path::PathBuf {
    let source = directory.join("outcome-host.rs");
    let binary = directory.join(if cfg!(windows) {
        "outcome-host.exe"
    } else {
        "outcome-host"
    });
    std::fs::write(
        &source,
        r#"
use std::env;
use std::fs::OpenOptions;
use std::io::{Read, Write};
fn value(input: &str, key: &str) -> String {
    let needle = format!("\"{}\":\"", key);
    let start = input.find(&needle).unwrap() + needle.len();
    input[start..].split('"').next().unwrap().to_owned()
}
fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let path = env::var("AI_COCKPIT_OUTCOME_HOST_EVENT_LOG").unwrap();
    let mut log = OpenOptions::new().create(true).append(true).open(path).unwrap();
    log.write_all(input.as_bytes()).unwrap();
    log.write_all(b"\n").unwrap();
    let delivery = value(&input, "deliveryId");
    let digest = value(&input, "bodyDigest");
    let part = input.split("\"part\":").nth(1).unwrap().split(',').next().unwrap();
    if env::var("AI_COCKPIT_OUTCOME_HOST_FAIL_PART").ok().as_deref() == Some(part) {
        std::process::exit(7);
    }
    println!("{{\"deliveryId\":\"{}\",\"part\":{},\"segmentDigest\":\"{}\",\"accepted\":true,\"displayed\":true}}", delivery, part, digest);
}
"#,
    )
    .expect("write host fixture");
    assert!(
        Command::new("rustc")
            .args(["--edition", "2021"])
            .arg(&source)
            .arg("-o")
            .arg(&binary)
            .status()
            .expect("compile host fixture")
            .success()
    );
    binary
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
            "Outcome: 🟢 Declared verification passed",
            "Unknowns",
            "Human decisions",
            "Next action",
            "Decision: approved",
        ),
        (
            "zh-CN",
            "WI-HANDOFF-ZH",
            "Outcome: 🟢 已声明的验证通过",
            "未知项",
            "人工决定",
            "下一步",
            "决定: approved",
        ),
        (
            "ja",
            "WI-HANDOFF-JA",
            "Outcome: 🟢 宣言された検証済み",
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
        assert_eq!(archive_json["outcomeDelivery"]["view"], "full");
        assert_eq!(
            archive_json["outcomeDelivery"]["language"],
            match language {
                "en" => "en",
                "zh-CN" => "zh",
                "ja" => "ja",
                _ => unreachable!(),
            }
        );
        let archive_body = archive_json["outcomeDelivery"]["body"]
            .as_str()
            .expect("archive full body");
        assert!(!archive_body.is_empty());
        assert_eq!(
            String::from_utf8(archive.stderr.clone())
                .expect("archive handoff UTF-8")
                .trim_end(),
            archive_body
        );
        // Once the Work Item is archived, a bound provider context still
        // requires a valid provider-side finalization receipt.  The Runtime
        // therefore exposes a visible yellow handoff here; close becomes
        // green only after `record_deleted` binds that receipt below.
        let archive_prefix = match language {
            "en" => "Outcome: 🟡 Verification not ready",
            "zh-CN" => "Outcome: 🟡 验证尚未就绪",
            "ja" => "Outcome: 🟡 検証未準備",
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
        if command == "archive" {
            assert_eq!(json["outcomeDelivery"]["view"], "full");
            assert_eq!(
                json["outcomeDelivery"]["deliveryState"],
                "returned_to_consumer"
            );
            assert_eq!(json["outcomeDelivery"]["workItemId"], id);
            let body = json["outcomeDelivery"]["body"].as_str().expect("full body");
            assert!(body.contains("Task Result"));
            assert!(body.contains("Problems found"));
            assert!(body.contains("Evidence"));
            assert!(!body.is_empty());
            assert_eq!(json["hostDeliveryMode"], "full_handoff_only");
            assert_eq!(json["deliveryReport"]["deliveryState"], "unknown");
            assert_eq!(json["deliveryReport"]["hostConfirmation"], "unknown");
            assert_eq!(json["hostDisplayConfirmation"], "unknown");
            let events = json["assistantMessageEvents"]
                .as_array()
                .expect("conversation-facing assistant message events");
            let segments = json["outcomeDelivery"]["segments"]
                .as_array()
                .expect("delivery segments");
            assert_eq!(events.len(), segments.len());
            for (event, segment) in events.iter().zip(segments) {
                assert_eq!(event["schemaVersion"], 1);
                assert_eq!(event["event"], "assistant_message");
                assert_eq!(event["segment"], *segment);
            }
        }
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
fn archive_normal_output_has_the_same_full_body_as_structured_stdout() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let id = "WI-HANDOFF-ARCHIVE-FULL";
    let repo = checkpointed(binary, id, true);
    let finish = run(binary, repo.path(), &["finish", "--id", id]);
    assert!(finish.status.success());
    let archive = run(binary, repo.path(), &["archive", "--id", id]);
    assert!(
        archive.status.success(),
        "{}",
        String::from_utf8_lossy(&archive.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&archive.stdout).expect("archive JSON");
    let body = json["outcomeDelivery"]["body"].as_str().expect("full body");
    let stderr = String::from_utf8(archive.stderr).expect("human UTF-8");
    assert_eq!(stderr, format!("{body}\n"));
    assert!(body.contains("What was completed"));
    assert!(body.contains("Problems found"));
    assert!(body.contains("Human decisions"));
    assert!(body.contains("Next action"));
}

#[test]
fn archived_outcome_delivery_query_reuses_the_full_body_without_rearchiving() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let id = "WI-ARCHIVE-DELIVERY-QUERY";
    let repo = checkpointed(binary, id, true);
    let finish = run(binary, repo.path(), &["finish", "--id", id]);
    assert!(finish.status.success());
    let archive = run(binary, repo.path(), &["archive", "--id", id]);
    assert!(archive.status.success());
    let archive_json: serde_json::Value = serde_json::from_slice(&archive.stdout).expect("archive");
    let archive_identity = archive_json["outcomeDelivery"]["archiveIdentity"]
        .as_str()
        .expect("archive identity")
        .to_owned();

    let delivery = run(
        binary,
        repo.path(),
        &["work-item", "outcome", "--id", id, "--delivery", "--json"],
    );
    assert!(
        delivery.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&delivery.stderr)
    );
    let delivery_json: serde_json::Value =
        serde_json::from_slice(&delivery.stdout).expect("delivery JSON");
    assert_eq!(delivery_json["view"], "full");
    assert_eq!(delivery_json["deliveryState"], "returned_to_consumer");
    assert_eq!(
        delivery_json["body"],
        archive_json["outcomeDelivery"]["body"]
    );
    assert_eq!(delivery_json["archiveIdentity"], archive_identity);
    assert_eq!(delivery_json["hostDeliveryMode"], "full_handoff_only");
    assert_eq!(delivery_json["hostDisplayConfirmation"], "unknown");
    assert_eq!(
        delivery_json["assistantMessageEvents"]
            .as_array()
            .unwrap()
            .len(),
        delivery_json["segments"].as_array().unwrap().len()
    );

    let human = run(
        binary,
        repo.path(),
        &["work-item", "outcome", "--id", id, "--delivery"],
    );
    assert!(human.status.success());
    assert_eq!(
        String::from_utf8(human.stdout).expect("human delivery"),
        archive_json["outcomeDelivery"]["body"]
            .as_str()
            .expect("body")
            .to_owned()
            + "\n"
    );
    let after = run(
        binary,
        repo.path(),
        &["work-item", "outcome", "--id", id, "--delivery", "--json"],
    );
    let after_json: serde_json::Value = serde_json::from_slice(&after.stdout).expect("after JSON");
    assert_eq!(after_json["archiveIdentity"], archive_identity);
}

#[test]
fn archive_uses_configured_host_command_and_reports_actual_display_receipts() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let id = "WI-ARCHIVE-EXTERNAL-HOST";
    let repo = checkpointed(binary, id, true);
    let finish = run(binary, repo.path(), &["finish", "--id", id]);
    assert!(finish.status.success());
    let host_directory = tempfile::tempdir().expect("host tempdir");
    let host = compile_host_fixture(host_directory.path());
    let events = host_directory.path().join("assistant-events.jsonl");
    let archive = Command::new(binary)
        .args(["archive", "--id", id])
        .arg("--repo")
        .arg(repo.path())
        .env("AI_COCKPIT_OUTCOME_HOST_PROGRAM", &host)
        .env("AI_COCKPIT_OUTCOME_HOST_EVENT_LOG", &events)
        .output()
        .expect("archive with host command");
    assert!(
        archive.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&archive.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&archive.stdout).expect("archive JSON");
    assert_eq!(json["hostDeliveryMode"], "external_command");
    assert_eq!(json["hostDisplayConfirmation"], "display_confirmed");
    assert_eq!(
        json["outcomeDelivery"]["deliveryState"],
        "display_confirmed"
    );
    assert_eq!(json["deliveryReport"]["deliveryState"], "display_confirmed");
    assert_eq!(
        String::from_utf8(archive.stderr)
            .expect("handoff")
            .trim_end(),
        json["outcomeDelivery"]["body"].as_str().expect("body")
    );
    let event_log = std::fs::read_to_string(events).expect("assistant events");
    assert!(event_log.contains("\"event\":\"assistant_message\""));
    assert!(event_log.contains(id));
}

#[test]
fn interrupted_host_delivery_is_retried_from_persisted_progress_without_rearchive() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let id = "WI-ARCHIVE-EXTERNAL-RETRY";
    let repo = checkpointed(binary, id, true);
    let finish = run(binary, repo.path(), &["finish", "--id", id]);
    assert!(finish.status.success());
    let host_directory = tempfile::tempdir().expect("host tempdir");
    let host = compile_host_fixture(host_directory.path());
    let events = host_directory.path().join("assistant-events.jsonl");
    let first = Command::new(binary)
        .args(["archive", "--id", id])
        .arg("--repo")
        .arg(repo.path())
        .env("AI_COCKPIT_OUTCOME_HOST_PROGRAM", &host)
        .env("AI_COCKPIT_OUTCOME_HOST_EVENT_LOG", &events)
        .env("AI_COCKPIT_OUTCOME_HOST_FAIL_PART", "1")
        .output()
        .expect("interrupted archive");
    assert!(first.status.success());
    let first_json: serde_json::Value = serde_json::from_slice(&first.stdout).expect("first JSON");
    assert_eq!(
        first_json["outcomeDelivery"]["deliveryState"],
        "delivery_failed"
    );
    assert_eq!(first_json["deliveryReport"]["sentParts"], 0);
    let progress = repo
        .path()
        .join(format!(".ai/outcome-delivery/{id}.progress.json"));
    assert!(progress.is_file());
    assert_eq!(
        std::fs::read_to_string(&events)
            .unwrap_or_default()
            .lines()
            .count(),
        1
    );
    let archive_identity = first_json["outcomeDelivery"]["archiveIdentity"].clone();

    let retry = Command::new(binary)
        .args(["work-item", "outcome", "--id", id, "--delivery", "--json"])
        .arg("--repo")
        .arg(repo.path())
        .env("AI_COCKPIT_OUTCOME_HOST_PROGRAM", &host)
        .env("AI_COCKPIT_OUTCOME_HOST_EVENT_LOG", &events)
        .output()
        .expect("retry delivery");
    assert!(retry.status.success());
    let retry_json: serde_json::Value = serde_json::from_slice(&retry.stdout).expect("retry JSON");
    assert_eq!(
        retry_json["deliveryState"],
        "display_confirmed",
        "retry={retry_json:#} stderr={}",
        String::from_utf8_lossy(&retry.stderr)
    );
    assert_eq!(retry_json["archiveIdentity"], archive_identity);
    assert_eq!(
        std::fs::read_to_string(&events)
            .expect("events")
            .lines()
            .count(),
        2
    );
    assert!(!progress.exists());
}

#[test]
fn outcome_command_defaults_to_summary_and_full_view_is_explicit() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let id = "WI-HANDOFF-SUMMARY";
    let repo = checkpointed(binary, id, true);

    let summary = run(binary, repo.path(), &["work-item", "outcome", "--id", id]);
    assert!(summary.status.success());
    let summary_text = String::from_utf8(summary.stdout).expect("summary UTF-8");
    assert!(summary_text.contains("Result"));
    assert!(summary_text.contains("Key changes"));
    assert!(summary_text.contains("Remaining uncertainty"));
    assert!(summary_text.contains("Human next step"));
    assert!(!summary_text.contains("Problems found"));

    let full = run(
        binary,
        repo.path(),
        &["work-item", "outcome", "--id", id, "--view", "full"],
    );
    assert!(full.status.success());
    let full_text = String::from_utf8(full.stdout).expect("full UTF-8");
    assert!(full_text.contains("Problems found"));
    assert!(full_text.contains("Evidence"));
}

#[test]
fn blocked_finish_emits_persisted_handoff_and_remains_nonzero() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    for (language, id, prefix, unknowns, next) in [
        (
            "en",
            "WI-HANDOFF-BLOCKED-EN",
            "Outcome: 🔴 Verification status unknown",
            "Unknowns",
            "Next action",
        ),
        (
            "zh-CN",
            "WI-HANDOFF-BLOCKED-ZH",
            "Outcome: 🔴 验证状态未知",
            "未知项",
            "下一步",
        ),
        (
            "ja",
            "WI-HANDOFF-BLOCKED-JA",
            "Outcome: 🔴 検証状態不明",
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

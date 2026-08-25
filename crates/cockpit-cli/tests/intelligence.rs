use std::{fs, path::Path, process::Command};

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

fn tree_manifest(root: &Path) -> Vec<(String, String, bool, u64, Vec<u8>)> {
    fn visit(root: &Path, current: &Path, entries: &mut Vec<(String, String, bool, u64, Vec<u8>)>) {
        let mut children = fs::read_dir(current)
            .expect("read manifest directory")
            .collect::<Result<Vec<_>, _>>()
            .expect("read manifest entries");
        children.sort_by_key(|entry| entry.file_name());
        for child in children {
            let path = child.path();
            let metadata = fs::symlink_metadata(&path).expect("manifest metadata");
            let relative = path
                .strip_prefix(root)
                .expect("manifest relative path")
                .to_string_lossy()
                .replace('\\', "/");
            let (kind, bytes) = if metadata.file_type().is_symlink() {
                (
                    "symlink",
                    fs::read_link(&path)
                        .expect("symlink target")
                        .to_string_lossy()
                        .as_bytes()
                        .to_vec(),
                )
            } else if metadata.is_dir() {
                ("directory", Vec::new())
            } else {
                ("file", fs::read(&path).expect("manifest file bytes"))
            };
            entries.push((
                relative,
                kind.into(),
                metadata.permissions().readonly(),
                metadata.len(),
                bytes,
            ));
            if metadata.is_dir() {
                visit(root, &path, entries);
            }
        }
    }

    let mut entries = Vec::new();
    visit(root, root, &mut entries);
    entries
}

#[test]
fn read_only_projection_commands_leave_repository_bytes_unchanged() {
    let directory = repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let run = |args: &[&str]| {
        let output = Command::new(binary)
            .args(args)
            .args(["--repo", directory.path().to_str().expect("repo path")])
            .output()
            .expect("run ai-cockpit");
        assert!(
            output.status.success(),
            "args={args:?}, stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run(&["attach"]);
    run(&[
        "start",
        "--id",
        "WI-READ-ONLY",
        "--intent",
        "prove read-only projections",
        "--goal",
        "preserve repository bytes",
        "--scope",
        "**",
        "--authority",
        "authorized",
    ]);
    fs::create_dir_all(directory.path().join(".ai/evidence/nested"))
        .expect("nested evidence directory");
    fs::write(
        directory.path().join(".ai/evidence/nested/fixture.json"),
        b"{}",
    )
    .expect("fixture bytes");
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        "fixture.json",
        directory
            .path()
            .join(".ai/evidence/nested/fixture-link.json"),
    )
    .expect("fixture symlink");
    let before = tree_manifest(directory.path());

    for _ in 0..2 {
        run(&["observe"]);
        run(&["capability", "show"]);
        run(&["status"]);
        run(&["work-item", "status", "--id", "WI-READ-ONLY", "--json"]);
        run(&["work-item", "status", "--all", "--json"]);
    }

    assert_eq!(tree_manifest(directory.path()), before);
}

#[test]
fn all_work_item_status_cli_emits_stable_repository_index() {
    let directory = repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let run = |args: &[&str]| {
        Command::new(binary)
            .args(args)
            .args(["--repo", directory.path().to_str().expect("repo path")])
            .output()
            .expect("run ai-cockpit")
    };
    assert!(run(&["attach"]).status.success());
    for id in ["WI-CLI-Z", "WI-CLI-A"] {
        let start = run(&[
            "start",
            "--id",
            id,
            "--intent",
            "aggregate status",
            "--goal",
            "stable output",
            "--scope",
            "**",
            "--authority",
            "authorized",
        ]);
        assert!(
            start.status.success(),
            "{}",
            String::from_utf8_lossy(&start.stderr)
        );
    }

    let first = run(&["work-item", "status", "--all", "--json"]);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let second = run(&["work-item", "status", "--all", "--json"]);
    assert!(second.status.success());
    let first_json: serde_json::Value = serde_json::from_slice(&first.stdout).expect("first JSON");
    let second_json: serde_json::Value =
        serde_json::from_slice(&second.stdout).expect("second JSON");
    assert_eq!(first_json["indexDigest"], second_json["indexDigest"]);
    assert_eq!(first_json["counts"]["yellow"], 2);
    assert_eq!(first_json["items"][0]["workItemId"], "WI-CLI-A");
    assert_eq!(first_json["items"][1]["workItemId"], "WI-CLI-Z");
}

#[test]
fn intelligence_commands_emit_repository_bound_json_and_unknowns() {
    let directory = repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let attach = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(directory.path())
        .output()
        .expect("attach");
    assert!(attach.status.success());
    let start = Command::new(binary)
        .args(["start", "--repo"])
        .arg(directory.path())
        .args([
            "--id",
            "WI-INTELLIGENCE",
            "--intent",
            "traceable approach",
            "--goal",
            "test outputs",
            "--scope",
            "crates/**",
            "--authority",
            "authorized",
        ])
        .output()
        .expect("start");
    assert!(
        start.status.success(),
        "{}",
        String::from_utf8_lossy(&start.stderr)
    );

    let approach = Command::new(binary)
        .args(["work-item", "approach", "--repo"])
        .arg(directory.path())
        .args(["--id", "WI-INTELLIGENCE"])
        .output()
        .expect("approach");
    assert!(approach.status.success());
    let approach_json: serde_json::Value = serde_json::from_slice(&approach.stdout).expect("JSON");
    assert_eq!(approach_json["schemaVersion"], 2);
    assert!(!approach_json["facts"].as_array().expect("facts").is_empty());

    let inspect = Command::new(binary)
        .args(["work-item", "inspect", "--repo"])
        .arg(directory.path())
        .args(["--id", "WI-INTELLIGENCE"])
        .output()
        .expect("inspect");
    assert!(inspect.status.success());
    let inspect_json: serde_json::Value = serde_json::from_slice(&inspect.stdout).expect("JSON");
    assert_eq!(inspect_json["compatibility"]["compatible"], false);
    assert_eq!(
        inspect_json["compatibility"]["reasons"][0],
        "parallel_compatibility_not_declared"
    );

    let declare = Command::new(binary)
        .args(["work-item", "declare", "--repo"])
        .arg(directory.path())
        .args(["--id", "WI-INTELLIGENCE", "--parallelizable"])
        .output()
        .expect("declare");
    assert!(declare.status.success());
    let declared: serde_json::Value = serde_json::from_slice(&declare.stdout).expect("JSON");
    assert_eq!(declared["parallelizable"], true);
    let inspected = Command::new(binary)
        .args(["work-item", "inspect", "--repo"])
        .arg(directory.path())
        .args(["--id", "WI-INTELLIGENCE"])
        .output()
        .expect("inspect declared");
    assert!(inspected.status.success());
    let inspected_json: serde_json::Value =
        serde_json::from_slice(&inspected.stdout).expect("JSON");
    assert_eq!(inspected_json["compatibility"]["compatible"], true);

    let capability = Command::new(binary)
        .args(["capability", "show", "--repo"])
        .arg(directory.path())
        .output()
        .expect("capability");
    assert!(capability.status.success());
    let capability_json: serde_json::Value =
        serde_json::from_slice(&capability.stdout).expect("JSON");
    assert_eq!(capability_json["repositoryId"].as_str().unwrap().len(), 71);
    assert_eq!(capability_json["runtimeVersion"], env!("CARGO_PKG_VERSION"));
    assert_eq!(capability_json["projectGovernance"]["schemaVersion"], 1);
    assert_eq!(
        capability_json["projectGovernance"]["repositoryId"],
        capability_json["repositoryId"]
    );
    assert!(
        capability_json["projectGovernance"]["unknowns"]
            .as_array()
            .expect("project governance unknowns")
            .iter()
            .any(|item| item == "project_capabilities_missing")
    );
    assert!(
        capability_json["runtimeDigest"]
            .as_str()
            .is_some_and(|value| value.starts_with("sha256:"))
    );
    assert!(
        capability_json["adopterCapabilities"]
            .as_array()
            .expect("adopter capabilities")
            .iter()
            .any(|item| item["id"] == "work_item_status_interface"
                && item["state"] == "repository_bound")
    );
    assert!(
        capability_json["exclusions"]
            .as_array()
            .expect("exclusions")
            .iter()
            .any(|item| item["id"] == "hosted_ci")
    );

    let diagnosis = Command::new(binary)
        .args(["diagnose", "--repo"])
        .arg(directory.path())
        .output()
        .expect("diagnose");
    assert!(diagnosis.status.success());
    let diagnosis_json: serde_json::Value =
        serde_json::from_slice(&diagnosis.stdout).expect("JSON");
    assert_eq!(diagnosis_json["state"], "unknown");
    assert!(
        diagnosis_json["unknowns"]
            .as_array()
            .expect("unknowns")
            .iter()
            .any(|item| item == "work_item_not_selected")
    );

    let knowledge = Command::new(binary)
        .args(["knowledge", "query", "--repo"])
        .arg(directory.path())
        .args(["--v2"])
        .output()
        .expect("knowledge");
    assert!(knowledge.status.success());
    let knowledge_json: serde_json::Value =
        serde_json::from_slice(&knowledge.stdout).expect("JSON");
    assert_eq!(knowledge_json["schemaVersion"], 2);
    assert!(
        directory
            .path()
            .join(".ai/work-items/active/WI-INTELLIGENCE.approach.json")
            .is_file()
    );
    let outcome = Command::new(binary)
        .args(["work-item", "outcome", "--repo"])
        .arg(directory.path())
        .args(["--id", "WI-INTELLIGENCE"])
        .env("AI_COCKPIT_LANGUAGE", "zh-CN")
        .output()
        .expect("human outcome");
    assert!(outcome.status.success());
    let outcome_text = String::from_utf8(outcome.stdout).expect("UTF-8 outcome");
    assert!(outcome_text.starts_with("Outcome: 🟡 需要关注 — WI-INTELLIGENCE"));
    assert!(outcome_text.contains("🟡 需要关注"));
    assert!(outcome_text.contains("未找到或无法使用验证证据；结果尚未准备好。"));
    assert!(!outcome_text.contains("No verification evidence"));
    assert!(outcome_text.contains("下一步"));
    fs::write(
        directory
            .path()
            .join(".ai/evidence/WI-INTELLIGENCE.verification.json"),
        br#"{"protocolVersion":1,"evidenceSchemaVersion":2,"workItemId":"WI-INTELLIGENCE","passed":false}"#,
    )
    .expect("tampered evidence");
    let red_outcome = Command::new(binary)
        .args(["work-item", "outcome", "--repo"])
        .arg(directory.path())
        .args(["--id", "WI-INTELLIGENCE"])
        .env("AI_COCKPIT_LANGUAGE", "zh-CN")
        .output()
        .expect("red human outcome");
    assert!(red_outcome.status.success());
    let red_text = String::from_utf8(red_outcome.stdout).expect("UTF-8 red outcome");
    assert!(red_text.contains("🔴 停止"));
    assert!(red_text.contains("验证证据无效"));
    for (language, title, status) in [
        ("ja", "Outcome: 🔴 停止 — WI-INTELLIGENCE", "🔴 停止"),
        ("en", "Outcome: 🔴 Stop — WI-INTELLIGENCE", "🔴 Stop"),
    ] {
        let localized = Command::new(binary)
            .args(["work-item", "outcome", "--repo"])
            .arg(directory.path())
            .args(["--id", "WI-INTELLIGENCE"])
            .env("AI_COCKPIT_LANGUAGE", language)
            .output()
            .expect("localized human outcome");
        assert!(localized.status.success(), "language={language}");
        let text = String::from_utf8(localized.stdout).expect("UTF-8 localized outcome");
        assert!(text.contains(title), "language={language}");
        assert!(text.contains(status), "language={language}");
        let summary = match language {
            "ja" => "検証 evidence を確認できないか現在の context と一致しないため、停止しました。",
            _ => {
                "Verification evidence could not be confirmed or does not match this context; the outcome is stopped."
            }
        };
        assert!(text.contains(summary), "language={language}");
    }
    let machine_outcome = Command::new(binary)
        .args(["work-item", "outcome", "--repo"])
        .arg(directory.path())
        .args(["--id", "WI-INTELLIGENCE", "--json"])
        .output()
        .expect("machine outcome");
    assert!(machine_outcome.status.success());
    let machine_json: serde_json::Value =
        serde_json::from_slice(&machine_outcome.stdout).expect("machine outcome JSON");
    assert_eq!(machine_json["schemaVersion"], 2);
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn cli_status_changes_from_archived_to_closed_only_after_valid_close() {
    let directory = repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let id = "WI-CLI-STATUS-CLOSED";
    let run = |args: &[&str]| {
        Command::new(binary)
            .args(args)
            .args(["--repo", directory.path().to_str().expect("repo path")])
            .output()
            .expect("run ai-cockpit")
    };
    assert!(run(&["attach"]).status.success());
    assert!(
        run(&[
            "start",
            "--id",
            id,
            "--intent",
            "project terminal status",
            "--goal",
            "show closed status",
            "--scope",
            "**",
            "--authority",
            "authorized",
            "--acceptance",
            "status is terminal after close",
            "--required-evidence",
            "verification",
        ])
        .status
        .success()
    );
    common::plan(binary, directory.path(), id);
    let contract = format!(".ai/work-items/active/{id}.contract.json");
    assert!(
        run(&["preflight", "--contract", &contract])
            .status
            .success()
    );
    assert!(run(&["checkpoint", "--id", id]).status.success());
    assert!(
        run(&[
            "verify",
            "--work-item",
            id,
            "--command",
            "true",
            "--workers",
            "1",
        ])
        .status
        .success()
    );
    let finish = run(&["finish", "--id", id]);
    assert!(
        finish.status.success(),
        "finish stderr: {}",
        String::from_utf8_lossy(&finish.stderr)
    );
    assert!(run(&["archive", "--id", id]).status.success());
    common::record_retained(binary, directory.path(), id);
    let archived_outcome = run(&["work-item", "outcome", "--id", id, "--json"]);
    assert!(archived_outcome.status.success());
    let archived_outcome_json: serde_json::Value =
        serde_json::from_slice(&archived_outcome.stdout).expect("archived outcome JSON");
    let archived_report = serde_json::to_string(&archived_outcome_json["taskOutcomeReport"])
        .expect("archived task report JSON");
    assert!(!archived_report.contains(&format!(".ai/work-items/active/{id}")));
    let archived_handoff = run(&["work-item", "outcome", "--id", id]);
    assert!(archived_handoff.status.success());
    assert!(
        !String::from_utf8_lossy(&archived_handoff.stdout)
            .contains(&format!(".ai/work-items/active/{id}"))
    );
    let archived = run(&["work-item", "status", "--id", id, "--json"]);
    assert!(archived.status.success());
    let archived: serde_json::Value =
        serde_json::from_slice(&archived.stdout).expect("archived JSON");
    assert_eq!(archived["lifecyclePhase"], "archived");
    assert_eq!(archived["completionDomains"]["closure"], "archived");

    let evidence_ref = format!(".ai/evidence/{id}.verification.json");
    let close = run(&[
        "close",
        "--id",
        id,
        "--human-decision",
        "approved",
        "--actor",
        "human:owner",
        "--authority-source",
        "user-authorized-work-item",
        "--reason",
        "fresh evidence",
        "--evidence-ref",
        &evidence_ref,
        "--policy-ref",
        "status-projection",
        "--decided-at",
        "2026-08-22T12:00:00Z",
        "--resume-condition",
        "rerun verification if the base changes",
    ]);
    assert!(
        close.status.success(),
        "close stderr: {}",
        String::from_utf8_lossy(&close.stderr)
    );
    let closed = run(&["work-item", "status", "--id", id, "--json"]);
    assert!(closed.status.success());
    let closed: serde_json::Value = serde_json::from_slice(&closed.stdout).expect("closed JSON");
    assert_eq!(closed["lifecyclePhase"], "closed");
    assert_eq!(closed["completionDomains"]["closure"], "closed");
    assert_eq!(
        closed["humanDecisions"],
        serde_json::json!(["close_decision_recorded"])
    );
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn cli_rejects_tampered_verification_timestamp_before_finish() {
    let directory = repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let id = "WI-CLI-TIMESTAMP";
    let run = |args: &[&str]| {
        Command::new(binary)
            .args(args)
            .args(["--repo", directory.path().to_str().expect("repo path")])
            .output()
            .expect("run ai-cockpit")
    };
    assert!(run(&["attach"]).status.success());
    assert!(
        run(&[
            "start",
            "--id",
            id,
            "--intent",
            "reject malformed evidence timestamps",
            "--goal",
            "keep lifecycle fail closed",
            "--scope",
            "**",
            "--authority",
            "authorized",
            "--acceptance",
            "invalid timestamps are rejected",
            "--required-evidence",
            "verification",
        ])
        .status
        .success()
    );
    let contract = format!(".ai/work-items/active/{id}.contract.json");
    assert!(
        run(&["preflight", "--contract", &contract])
            .status
            .success()
    );
    assert!(run(&["checkpoint", "--id", id]).status.success());
    assert!(
        run(&[
            "verify",
            "--work-item",
            id,
            "--command",
            "true",
            "--workers",
            "1",
        ])
        .status
        .success()
    );

    let evidence_path = directory
        .path()
        .join(format!(".ai/evidence/{id}.verification.json"));
    let mut evidence: serde_json::Value =
        serde_json::from_slice(&fs::read(&evidence_path).expect("evidence")).expect("JSON");
    evidence["createdAt"] = "not-an-rfc3339-time".into();
    fs::write(
        &evidence_path,
        serde_json::to_vec_pretty(&evidence).expect("JSON"),
    )
    .expect("tamper timestamp");

    let outcome = run(&["work-item", "outcome", "--id", id, "--json"]);
    assert!(outcome.status.success());
    let outcome: serde_json::Value = serde_json::from_slice(&outcome.stdout).expect("outcome");
    assert_eq!(outcome["decisionState"], "red");
    assert_ne!(outcome["state"], "verified");

    let finish = run(&["finish", "--id", id]);
    assert!(
        !finish.status.success(),
        "tampered timestamp must block finish"
    );
    fs::remove_dir_all(directory).expect("cleanup");
}

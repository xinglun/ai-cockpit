use serde_json::json;
use std::{fs, process::Command};

#[test]
fn out_of_scope_change_is_stopped_before_spawn_and_same_intent_amendment_recovers() {
    let directory = tempfile::tempdir().expect("repository");
    let root = directory.path();
    fs::create_dir_all(root.join("src")).expect("src");
    fs::write(root.join("src/main.rs"), "fn main() {}\n").expect("source");
    fs::write(root.join("README.md"), "baseline\n").expect("README");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(root)
            .status()
            .expect("git init")
            .success()
    );
    assert!(
        Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .status()
            .expect("git add")
            .success()
    );
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=AI Cockpit Test",
                "-c",
                "user.email=ai-cockpit@example.invalid",
                "commit",
                "-qm",
                "fixture",
            ])
            .current_dir(root)
            .status()
            .expect("git commit")
            .success()
    );

    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let id = "WI-CLI-SCOPE-RECOVERY";
    let invoke = |args: &[&str]| {
        Command::new(binary)
            .args(args)
            .args(["--repo"])
            .arg(root)
            .current_dir(root)
            .output()
            .expect("Runtime command")
    };
    let output = invoke(&["attach"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = invoke(&[
        "start",
        "--id",
        id,
        "--intent",
        "implement the scoped repository change",
        "--goal",
        "include necessary support paths without widening intent or authority",
        "--scope",
        "src/**",
        "--authority",
        "authorized",
        "--required-evidence",
        "verification",
        "--acceptance",
        "out-of-scope paths do not spawn verification; additive same-intent scope can recover",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let contract = format!(".ai/work-items/active/{id}.contract.json");
    let output = invoke(&["preflight", "--contract", &contract]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = invoke(&["checkpoint", "--id", id]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::write(root.join("README.md"), "required support change\n").expect("out-of-scope change");
    let output = invoke(&["preflight", "--contract", &contract]);
    assert!(output.status.success());
    let preflight: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("preflight JSON");
    assert_eq!(preflight["state"], "red");
    assert!(preflight["blockers"].as_array().is_some_and(|blockers| {
        blockers.iter().any(|blocker| {
            blocker
                .as_str()
                .is_some_and(|value| value.contains("scope"))
        })
    }));

    #[cfg(unix)]
    let program = "sh";
    #[cfg(windows)]
    let program = "cmd";
    let rejected = invoke(&[
        "verify",
        "--work-item",
        id,
        "--command",
        program,
        if cfg!(unix) { "--args=-c" } else { "--args=/C" },
        if cfg!(unix) {
            "--args=touch .scope-verification-spawned"
        } else {
            "--args=type nul > .scope-verification-spawned"
        },
    ]);
    assert!(
        !rejected.status.success(),
        "red scope admission must reject verification"
    );
    assert!(
        !root.join(".scope-verification-spawned").exists(),
        "command must not spawn"
    );
    let evidence_entries = fs::read_dir(root.join(".ai/evidence"))
        .expect("evidence directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    let attempt = evidence_entries
        .iter()
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&format!("{id}.verification-attempt.")))
        })
        .unwrap_or_else(|| {
            panic!(
                "precondition-rejected attempt missing: stderr={}, files={evidence_entries:?}",
                String::from_utf8_lossy(&rejected.stderr)
            )
        });
    let attempt: serde_json::Value =
        serde_json::from_slice(&fs::read(attempt).expect("attempt bytes")).expect("attempt JSON");
    assert_eq!(attempt["state"], "precondition_rejected");
    assert_eq!(attempt["processesSpawned"], 0);

    let amendment = tempfile::NamedTempFile::new().expect("amendment input");
    fs::write(
        amendment.path(),
        serde_json::to_vec(&json!({"scopeAppend": ["README.md"]})).expect("JSON"),
    )
    .expect("write amendment");
    let amended = Command::new(binary)
        .args(["work-item", "amend", "--id", id, "--input"])
        .arg(amendment.path())
        .args([
            "--reason",
            "include the required README support path within the same intent",
        ])
        .args(["--repo"])
        .arg(root)
        .current_dir(root)
        .output()
        .expect("scope amendment");
    assert!(
        amended.status.success(),
        "{}",
        String::from_utf8_lossy(&amended.stderr)
    );
    let amendment_record: serde_json::Value =
        serde_json::from_slice(&amended.stdout).expect("amendment receipt");
    assert_eq!(amendment_record["nextAction"], "record_governance_controls");
    let contract_path = root.join(&contract);
    let amended_bytes = fs::read(&contract_path).expect("amended Contract");

    let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
    let summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary bytes"))
            .expect("Summary JSON");
    let controls = tempfile::NamedTempFile::new().expect("review controls input");
    fs::write(
        controls.path(),
        serde_json::to_vec(&json!({
            "decisionEvidence": {
                "schemaVersion": 1,
                "decisionId": "contract-preflight-review",
                "decision": "confirm_review",
                "workItemId": id,
                "repositoryId": amendment_record["repositoryId"],
                "contractDigest": amendment_record["newContractDigest"],
                "preflightDecisionDigest": summary["preflightDecisionDigest"],
                "repositorySnapshotDigest": summary["preflightRepositorySnapshotDigest"],
                "recordedAt": "2026-09-30T00:00:00Z",
                "recordedBy": "human:test-fixture",
                "reason": "confirm the same-intent scope amendment before continuing"
            }
        }))
        .expect("review controls JSON"),
    )
    .expect("write review controls");
    let reviewed = invoke(&[
        "work-item",
        "controls",
        "--id",
        id,
        "--input",
        controls.path().to_str().expect("controls path"),
    ]);
    assert!(
        reviewed.status.success(),
        "{}",
        String::from_utf8_lossy(&reviewed.stderr)
    );

    let unauthorized = tempfile::NamedTempFile::new().expect("unauthorized amendment input");
    fs::write(
        unauthorized.path(),
        serde_json::to_vec(&json!({"intent": "different intent"})).expect("JSON"),
    )
    .expect("write unauthorized amendment");
    let rejected_amendment = Command::new(binary)
        .args(["work-item", "amend", "--id", id, "--input"])
        .arg(unauthorized.path())
        .args(["--reason", "must not replace human-owned intent"])
        .args(["--repo"])
        .arg(root)
        .current_dir(root)
        .output()
        .expect("reject intent change");
    assert!(!rejected_amendment.status.success());
    assert_eq!(
        fs::read(&contract_path).expect("Contract unchanged"),
        amended_bytes
    );

    let refreshed = invoke(&["preflight", "--contract", &contract]);
    assert!(
        refreshed.status.success(),
        "{}",
        String::from_utf8_lossy(&refreshed.stderr)
    );
    let refreshed: serde_json::Value =
        serde_json::from_slice(&refreshed.stdout).expect("fresh preflight");
    assert_ne!(
        refreshed["state"], "red",
        "fresh preflight must admit the additive path"
    );
    #[cfg(unix)]
    let program = "sh";
    #[cfg(windows)]
    let program = "cmd";
    let accepted = invoke(&[
        "verify",
        "--work-item",
        id,
        "--command",
        program,
        if cfg!(unix) { "--args=-c" } else { "--args=/C" },
        "--args=exit 0",
    ]);
    assert!(
        accepted.status.success(),
        "{}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    let accepted: serde_json::Value =
        serde_json::from_slice(&accepted.stdout).expect("verify JSON");
    assert_eq!(accepted["processesSpawned"], 1);
}

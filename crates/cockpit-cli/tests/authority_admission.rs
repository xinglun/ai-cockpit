//! P2b production defaults only: no test source selector is installed.
use std::{fs, path::Path, process::Command};

fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let active = dir.path().join(".ai/work-items/active");
    fs::create_dir_all(&active).unwrap();
    fs::write(
        active.join("WI-FIXTURE.contract.json"),
        br#"{"governanceProfile":{"goalAuthority":{"required":true}}}"#,
    )
    .unwrap();
    dir
}

fn run(root: &Path, arguments: &[&str], root_flag: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(arguments)
        .arg(root_flag)
        .arg(root)
        .current_dir(root)
        .output()
        .unwrap()
}

#[test]
fn b15_required_transport_entries_reject_before_repository_or_input_reads() {
    let dir = fixture();
    let mut failures = vec![];
    for args in [
        vec!["inspect"],
        vec!["status"],
        vec!["observe"],
        vec!["verify", "--command", "synthetic-program-never-executed"],
        vec![
            "work-item",
            "controls",
            "--id",
            "WI-FIXTURE",
            "--input",
            "missing-input.json",
        ],
        vec![
            "work-item",
            "revalidate-amendment",
            "--id",
            "WI-FIXTURE",
            "--reason",
            "fixture",
        ],
        vec!["agent", "repair", "--provider", "codex"],
    ] {
        let output = run(dir.path(), &args, "--repo");
        if output.status.success()
            || !String::from_utf8_lossy(&output.stderr)
                .contains("goal_authority_source_unsupported")
        {
            failures.push(args.join(" "));
        }
    }
    let output_path = dir.path().join("must-not-be-created.jsonl");
    let output = run(
        dir.path(),
        &[
            "isolation-manifest",
            "--output",
            output_path.to_str().unwrap(),
        ],
        "--root",
    );
    if output.status.success()
        || !String::from_utf8_lossy(&output.stderr).contains("goal_authority_source_unsupported")
    {
        failures.push("isolation-manifest early return".into());
    }
    assert!(!output_path.exists());
    assert!(
        failures.is_empty(),
        "missing early authority rejection: {failures:?}"
    );
}

#[test]
fn b11_static_help_and_schema_remain_available_without_business_admission() {
    let dir = fixture();
    assert!(
        Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
            .arg("--help")
            .current_dir(dir.path())
            .output()
            .unwrap()
            .status
            .success()
    );
    let output = run(
        dir.path(),
        &["capability", "show", "--surface", "work-item-outcome"],
        "--repo",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!dir.path().join(".ai/locks").exists());
}

#[test]
fn b11_mcp_stdio_keeps_discovery_available_and_denies_business_calls() {
    use std::{io::Write, process::Stdio};
    let dir = fixture();
    fs::write(
        dir.path().join(".ai/cockpit.toml"),
        b"intentionally invalid metadata; discovery must not parse it",
    )
    .unwrap();
    fs::write(dir.path().join(".ai/project.json"), b"{}").unwrap();
    fs::write(dir.path().join(".ai/agent-interface.json"), b"{}").unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["mcp", "--repo"])
        .arg(dir.path())
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let messages = concat!(
        "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}\n",
        "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"capability_show\",\"arguments\":{\"surface\":\"work-item-outcome\"}}}\n",
        "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"work_item_list\",\"arguments\":{}}}\n"
    );
    let _ = child.stdin.take().unwrap().write_all(messages.as_bytes());
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let responses: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    assert_eq!(responses.len(), 3);
    assert_eq!(
        responses[0]["result"]["tools"].as_array().unwrap().len(),
        35
    );
    assert_ne!(responses[1]["result"]["isError"], true);
    assert!(
        responses[2]
            .to_string()
            .contains("goal_authority_source_unsupported")
    );
}

#[test]
fn b15_hidden_supervisor_denies_before_backend_or_launch_validation() {
    use std::{io::Write, process::Stdio};
    let dir = fixture();
    let mut child = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .arg("__composition-supervisor")
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let message = serde_json::json!({"repositoryRoot":dir.path()}).to_string() + "\n";
    child
        .stdin
        .take()
        .unwrap()
        .write_all(message.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("goal_authority_source_unsupported"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!dir.path().join(".ai/locks").exists());
}

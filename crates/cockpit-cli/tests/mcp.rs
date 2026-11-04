use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "macos")]
use std::{fs, path::Path};

#[test]
fn mcp_command_serves_json_rpc_over_stdio() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let repository = std::env::temp_dir().join(format!(
        "cockpit-cli-mcp-initialize-{}-{suffix}",
        std::process::id()
    ));
    std::fs::create_dir_all(&repository).expect("repository");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repository)
        .status()
        .expect("git init");
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let mut child = Command::new(binary)
        .args(["mcp", "--repo"])
        .arg(&repository)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("mcp");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(
            br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
"#,
        )
        .expect("request");
    let output = child.wait_with_output().expect("response");
    assert!(output.status.success());
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON");
    assert_eq!(response["result"]["serverInfo"]["name"], "ai-cockpit");
    assert_eq!(
        response["result"]["serverInfo"]["version"],
        env!("CARGO_PKG_VERSION")
    );
    let expected_digest = cockpit_core::Digest::sha256_bytes(
        &std::fs::read(binary).expect("read exact executable under test"),
    )
    .to_string();
    assert_eq!(
        response["result"]["serverInfo"]["runtimeDigest"],
        expected_digest
    );
    std::fs::remove_dir_all(repository).expect("cleanup");
}

#[test]
fn cli_and_mcp_subprocesses_expose_equivalent_verification_decisions_and_metrics() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let repository = std::env::temp_dir().join(format!(
        "cockpit-cli-mcp-parity-{}-{suffix}",
        std::process::id()
    ));
    std::fs::create_dir_all(&repository).expect("repository");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repository)
        .status()
        .expect("git init");
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let cli = Command::new(binary)
        .args(["verify", "--repo"])
        .arg(&repository)
        .args(["--command", "true"])
        .output()
        .expect("CLI verify");
    #[cfg(not(target_os = "macos"))]
    assert!(cli.status.success());
    #[cfg(target_os = "macos")]
    assert!(cli.status.success());
    let cli: serde_json::Value = serde_json::from_slice(&cli.stdout).expect("CLI JSON");

    let mut mcp = Command::new(binary)
        .args(["mcp", "--repo"])
        .arg(&repository)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("MCP");
    mcp.stdin
        .as_mut()
        .expect("stdin")
        .write_all(
            br#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"verify","arguments":{"command":"true"}}}
"#,
        )
        .expect("MCP request");
    let mcp = mcp.wait_with_output().expect("MCP response");
    assert!(mcp.status.success());
    let mcp: serde_json::Value = serde_json::from_slice(&mcp.stdout).expect("MCP JSON");
    let mcp = &mcp["result"]["structuredContent"];

    for field in [
        "nodesPlanned",
        "nodesExecuted",
        "nodesReused",
        "rerunStale",
        "rerunUnknown",
        "protectedNodesExecuted",
        "protectedNodesSkipped",
        "processesSpawned",
        "processSpawnFailures",
        "gitCalls",
        "filesRead",
        "filesHashed",
        "passed",
        "results",
    ] {
        assert_eq!(cli[field], mcp[field], "parity field {field}");
    }
    #[cfg(target_os = "macos")]
    {
        assert_eq!(cli["passed"], true);
        assert_eq!(mcp["passed"], true);
        for receipt in [&cli, mcp] {
            assert_eq!(receipt["executionRecords"][0]["exitCode"], 0);
            assert_eq!(receipt["executionRecords"][0]["commandResult"], "succeeded");
            assert!(receipt["executionRecords"][0]["cleanupUncertainty"].is_null());
            assert_eq!(
                receipt["executionRecords"][0]["unmanagedDescendantsUnproved"],
                true
            );
            assert_eq!(receipt["receiptCandidates"], serde_json::json!([]));
        }
    }
    std::fs::remove_dir_all(repository).expect("cleanup");
}

#[cfg(target_os = "macos")]
fn mcp_request(binary: &str, root: &Path, request: &serde_json::Value) -> serde_json::Value {
    let mut child = Command::new(binary)
        .args(["mcp", "--repo"])
        .arg(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("exact CLI-hosted MCP Runtime");
    let mut bytes = serde_json::to_vec(request).expect("MCP request JSON");
    bytes.push(b'\n');
    child
        .stdin
        .as_mut()
        .expect("MCP stdin")
        .write_all(&bytes)
        .expect("write MCP request");
    let output = child.wait_with_output().expect("MCP response");
    assert!(
        output.status.success(),
        "MCP host failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("MCP response JSON")
}

#[cfg(target_os = "macos")]
#[test]
fn mac_cli_hosted_mcp_preserves_failed_command_and_non_reuse() {
    for (work_item_id, program, args, exit_code, command_result) in [(
        "WI-MCP-MAC-COMMAND-FAIL",
        "python3",
        vec!["-c", "import sys; sys.exit(7)"],
        7,
        "failed",
    )] {
        let directory = tempfile::tempdir().expect("isolated repository");
        let root = fs::canonicalize(directory.path()).expect("canonical repository");
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .current_dir(&root)
                .status()
                .expect("git init")
                .success()
        );
        fs::write(root.join("README.md"), "Mac MCP cleanup fixture\n").expect("README");
        assert!(
            Command::new("git")
                .args(["add", "README.md"])
                .current_dir(&root)
                .status()
                .expect("git add")
                .success()
        );
        assert!(
            Command::new("git")
                .args([
                    "-c",
                    "user.name=Test",
                    "-c",
                    "user.email=test@example.invalid",
                    "commit",
                    "-qm",
                    "fixture",
                ])
                .current_dir(&root)
                .status()
                .expect("git commit")
                .success()
        );
        let binary = env!("CARGO_BIN_EXE_ai-cockpit");
        let run = |args: &[&str]| {
            Command::new(binary)
                .args(args)
                .args(["--repo"])
                .arg(&root)
                .output()
                .expect("exact CLI Runtime")
        };
        for args in [
            vec![
                "start",
                "--id",
                work_item_id,
                "--intent",
                "record Mac MCP command results honestly",
                "--goal",
                "keep descendant cleanup uncertainty visible",
                "--scope",
                "README.md",
                "--authority",
                "authorized",
                "--acceptance",
                "A1: cleanup remains unknown",
                "--required-evidence",
                "verification",
            ],
            vec![
                "preflight",
                "--contract",
                ".ai/work-items/active/WI-MCP-MAC-COMMAND-FAIL.contract.json",
            ],
            vec!["checkpoint", "--id", work_item_id],
        ] {
            let output = run(&args);
            assert!(
                output.status.success(),
                "args={args:?}, stderr={}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let controls = tempfile::NamedTempFile::new().expect("controls input");
        fs::write(
            controls.path(),
            serde_json::to_vec(&serde_json::json!({
                "acceptanceEvidence": [{
                    "acceptanceId": "A1",
                    "evidence": [{
                        "type": "test", "path": "README.md",
                        "locator": "Mac MCP cleanup fixture", "verification": "passed"
                    }]
                }],
                "intentAlignment": {"state": "resolved", "evidence": ["README.md"]}
            }))
            .expect("controls JSON"),
        )
        .expect("write controls");
        let controlled = run(&[
            "work-item",
            "controls",
            "--id",
            work_item_id,
            "--input",
            controls.path().to_str().expect("controls path"),
        ]);
        assert!(
            controlled.status.success(),
            "controls: {}",
            String::from_utf8_lossy(&controlled.stderr)
        );

        let verification = mcp_request(
            binary,
            &root,
            &serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "method": "tools/call",
                "params": {"name": "verify", "arguments": {
                    "workItemId": work_item_id, "command": program, "args": args
                }}
            }),
        );
        assert_eq!(verification["result"]["isError"], true, "{verification}");
        let attempts = fs::read_dir(root.join(".ai/evidence"))
            .expect("durable evidence")
            .map(|entry| entry.expect("evidence entry"))
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(&format!("{work_item_id}.verification-attempt."))
            })
            .map(|entry| {
                serde_json::from_slice::<serde_json::Value>(
                    &fs::read(entry.path()).expect("attempt bytes"),
                )
                .expect("attempt JSON")
            })
            .filter(|value| value["kind"] == "verification_attempt")
            .collect::<Vec<_>>();
        assert_eq!(attempts.len(), 1, "one durable terminal attempt");
        let attempt = &attempts[0];
        let record = &attempt["executionRecords"][0];
        assert_eq!(attempt["state"], "execution_failed");
        assert_eq!(attempt["passed"], false);
        assert_eq!(record["exitCode"], exit_code);
        assert_eq!(record["commandResult"], command_result);
        assert!(record["cleanupUncertainty"].is_null());
        assert_eq!(record["unmanagedDescendantsUnproved"], true);
        assert_eq!(
            attempt["receipt"]["receiptCandidates"],
            serde_json::json!([])
        );
        assert!(
            !root
                .join(format!(".ai/evidence/{work_item_id}.verification.json"))
                .exists()
        );

        let outcome = mcp_request(
            binary,
            &root,
            &serde_json::json!({
                "jsonrpc": "2.0", "id": 2, "method": "tools/call",
                "params": {"name": "work_item_outcome", "arguments": {
                    "workItemId": work_item_id, "delivery": false
                }}
            }),
        );
        assert_eq!(outcome["result"]["isError"], false);
        assert_ne!(
            outcome["result"]["structuredContent"]["outcome"]["decisionState"],
            "green"
        );

        let runtime = cockpit_protocol::RuntimeContext {
            runtime_version: env!("CARGO_PKG_VERSION").into(),
            protocol_version: cockpit_protocol::PROTOCOL_VERSION,
            runtime_digest: cockpit_core::Digest::sha256_bytes(
                &fs::read(binary).expect("exact Runtime bytes"),
            ),
        };
        let snapshot = cockpit_git::GitRepository::discover(&root)
            .expect("repository")
            .snapshot()
            .expect("current snapshot");
        assert_eq!(
            attempt["repositorySnapshotDigest"],
            cockpit_repository::snapshot_digest(&snapshot)
                .expect("snapshot digest")
                .to_string()
        );
        let request = cockpit_repository::RepositoryVerificationRequest {
            node_id: "project-command-0".into(),
            program: program.into(),
            args: args.into_iter().map(str::to_owned).collect(),
            scope: vec!["**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 2,
            work_item_id: Some(work_item_id.into()),
            timeout_seconds: None,
            policy: cockpit_repository::RepositoryVerificationPolicy::NeverReuse,
        };
        assert!(
            cockpit_repository::load_reusable_verification_attempt(
                &root,
                work_item_id,
                &[request],
                &snapshot,
                &runtime,
            )
            .expect("reuse admission")
            .is_none(),
            "a failed command cannot be reused even with matching current identity"
        );
    }
}

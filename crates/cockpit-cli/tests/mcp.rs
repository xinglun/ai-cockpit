use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn mcp_command_serves_json_rpc_over_stdio() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let mut child = Command::new(binary)
        .arg("mcp")
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
    std::fs::remove_dir_all(repository).expect("cleanup");
}

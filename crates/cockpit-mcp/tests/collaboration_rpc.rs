use cockpit_core::Digest;
use cockpit_protocol::{PROTOCOL_VERSION, RuntimeContext};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::process::Command;

fn run_git(root: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .expect("git command")
            .success()
    );
}

fn repository() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("tempdir");
    run_git(root.path(), &["init", "-q"]);
    run_git(
        root.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    run_git(root.path(), &["config", "user.name", "Test"]);
    fs::write(root.path().join("README.md"), "initial\n").expect("write");
    run_git(root.path(), &["add", "."]);
    run_git(root.path(), &["commit", "-qm", "initial"]);
    root
}

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "0.2.113".into(),
        protocol_version: PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(b"candidate-mcp"),
    }
}

fn call(root: &Path, name: &str, arguments: Value) -> Value {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": name, "arguments": arguments}
    });
    let response = cockpit_mcp::handle_request_for_repo(&request, root, &runtime());
    response["result"]["structuredContent"].clone()
}

#[test]
fn coordination_rpc_exposes_explicit_read_write_operations_and_stable_projection() {
    let root = repository();
    let tools = cockpit_mcp::handle_request(
        &json!({"jsonrpc":"2.0", "id":1, "method":"tools/list"}),
        &runtime(),
    );
    let names = tools["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect::<Vec<_>>();
    assert!(names.contains(&"work_item_coordination"));
    assert!(names.contains(&"work_item_composition"));

    let first = call(
        root.path(),
        "work_item_coordination",
        json!({"action":"inspect"}),
    );
    assert!(!root.path().join(".git/.ai-cockpit/coordination").exists());
    let second = call(
        root.path(),
        "work_item_coordination",
        json!({"action":"inspect"}),
    );
    assert_eq!(first, second);

    let error = cockpit_mcp::handle_request_for_repo(
        &json!({
            "jsonrpc":"2.0", "id":2, "method":"tools/call",
            "params":{"name":"work_item_coordination", "arguments":{"action":"inspect", "event":{}}}
        }),
        root.path(),
        &runtime(),
    );
    assert_eq!(error["result"]["isError"], true);
    assert!(!root.path().join(".git/.ai-cockpit/coordination").exists());
}

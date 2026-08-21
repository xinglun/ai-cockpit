use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_REPOSITORY_ID: AtomicU64 = AtomicU64::new(0);

fn test_runtime_context() -> cockpit_protocol::RuntimeContext {
    cockpit_protocol::RuntimeContext {
        runtime_version: "security-test".into(),
        protocol_version: cockpit_protocol::PROTOCOL_VERSION,
        runtime_digest: cockpit_core::Digest::sha256_bytes(b"mcp-security-test-runtime"),
    }
}

fn repo() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "cockpit-mcp-security-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&directory).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    cockpit_repository::attach(&directory).expect("attach");
    directory
}

#[test]
fn read_tools_are_bound_to_repository_and_reject_escape_paths() {
    let directory = repo();
    let listed = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({
            "jsonrpc":"2.0","id":1,"method":"tools/call",
            "params":{"name":"work_item_list","arguments":{}}
        }),
        &directory,
        &test_runtime_context(),
    );
    assert!(listed["result"]["structuredContent"]["items"].is_array());

    let escaped = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({
            "jsonrpc":"2.0","id":2,"method":"tools/call",
            "params":{"name":"evidence_get","arguments":{"path":"../outside"}}
        }),
        &directory,
        &test_runtime_context(),
    );
    assert_eq!(escaped["result"]["isError"], true);
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn verification_surface_rejects_untrusted_programs() {
    let directory = repo();
    let response = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({
            "jsonrpc":"2.0","id":3,"method":"tools/call",
            "params":{"name":"verify","arguments":{"command":"sh","args":["-c","true"]}}
        }),
        &directory,
        &test_runtime_context(),
    );
    assert_eq!(response["result"]["isError"], true);
    fs::remove_dir_all(directory).expect("cleanup");
}

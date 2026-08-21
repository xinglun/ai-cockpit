use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn repo() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "cockpit-mcp-security-{}-{suffix}",
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
    );
    assert!(listed["result"]["structuredContent"]["items"].is_array());

    let escaped = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({
            "jsonrpc":"2.0","id":2,"method":"tools/call",
            "params":{"name":"evidence_get","arguments":{"path":"../outside"}}
        }),
        &directory,
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
    );
    assert_eq!(response["result"]["isError"], true);
    fs::remove_dir_all(directory).expect("cleanup");
}

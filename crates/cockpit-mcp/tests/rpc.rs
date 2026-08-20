use cockpit_mcp::handle_request;
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn mcp_initialize_and_tool_list_are_read_only_and_deterministic() {
    let initialize = handle_request(
        &serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    );
    assert_eq!(initialize["result"]["protocolVersion"], "2025-06-18");
    let tools = handle_request(
        &serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
    );
    let names = tools["result"]["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .map(|tool| tool["name"].as_str().expect("name"))
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        vec![
            "status",
            "work_item_get",
            "work_item_list",
            "blockers",
            "safe_actions",
            "knowledge_query",
            "evidence_get",
            "repository_observe",
            "preflight",
            "verify"
        ]
    );
}

#[test]
fn mcp_unknown_method_returns_json_rpc_error() {
    let response =
        handle_request(&serde_json::json!({"jsonrpc":"2.0","id":3,"method":"archive","params":{}}));
    assert_eq!(response["error"]["code"], -32601);
}

#[test]
fn repository_bound_status_tool_returns_protocol_state() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("cockpit-mcp-repo-{suffix}"));
    fs::create_dir_all(&directory).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    cockpit_repository::attach(&directory).expect("attach");
    let response = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"status","arguments":{}}}),
        &directory,
    );
    assert_eq!(
        response["result"]["structuredContent"]["state"],
        "calibration_required"
    );
    fs::remove_dir_all(directory).expect("cleanup");
}

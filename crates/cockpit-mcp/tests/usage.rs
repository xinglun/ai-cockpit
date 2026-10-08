use cockpit_core::Digest;
use cockpit_protocol::RuntimeContext;
use serde_json::json;

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        protocol_version: cockpit_protocol::PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(b"usage discovery runtime"),
    }
}

#[test]
fn initialize_and_tool_list_expose_native_usage_record_without_hidden_helper() {
    let initialized = cockpit_mcp::handle_request(
        &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
        &runtime(),
    );
    assert_eq!(initialized["result"]["serverInfo"]["name"], "ai-cockpit");
    let listed = cockpit_mcp::handle_request(
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
        &runtime(),
    );
    let tools = listed["result"]["tools"].as_array().expect("tools");
    assert_eq!(tools.len(), 31);
    let usage = tools
        .iter()
        .find(|tool| tool["name"] == "work_item_usage_record")
        .expect("native usage record tool");
    let schema = &usage["inputSchema"];
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["required"], json!(["request"]));
    assert_eq!(
        schema["properties"]["request"]["additionalProperties"],
        false
    );
    assert_eq!(
        schema["properties"]["request"]["description"],
        cockpit_protocol::WORK_ITEM_USAGE_RECORD_REQUEST_DESCRIPTION
    );
    assert_eq!(
        schema["properties"]["request"]["properties"]["sourceKind"]["enum"],
        json!(["provider-reported", "host-reported", "agent-declared"])
    );
    assert_eq!(
        schema["properties"]["request"]["properties"]["inputTokens"]["type"],
        json!(["integer", "null"])
    );
    assert!(
        !tools
            .iter()
            .any(|tool| tool["name"] == "__composition-supervisor")
    );
}

#[test]
fn usage_record_requires_explicit_repository_binding() {
    let response = cockpit_mcp::handle_request(
        &json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{
            "name":"work_item_usage_record","arguments":{"request":{}}
        }}),
        &runtime(),
    );
    assert_eq!(response["error"]["code"], -32001);
}

#[test]
fn usage_capability_description_matches_cli_and_mcp_facts() {
    let response = cockpit_mcp::handle_request_for_repo(
        &json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{
            "name":"capability_show","arguments":{"surface":"work-item-usage-record"}
        }}),
        std::path::Path::new("."),
        &runtime(),
    );
    assert_eq!(response["result"]["isError"], false, "{response}");
    assert_eq!(
        response["result"]["structuredContent"],
        serde_json::to_value(cockpit_protocol::work_item_usage_record_interface_description())
            .expect("canonical description")
    );
}

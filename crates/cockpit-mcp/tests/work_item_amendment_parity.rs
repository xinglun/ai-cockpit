use cockpit_core::Digest;
use cockpit_protocol::RuntimeContext;
use serde_json::json;

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        protocol_version: cockpit_protocol::PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(b"candidate-mcp"),
    }
}

#[test]
fn tool_list_exposes_amendment_write_and_read_tools() {
    let response = cockpit_mcp::handle_request(
        &json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}),
        &runtime(),
    );
    let tools = response["result"]["tools"].as_array().expect("MCP tools");
    let names = tools
        .iter()
        .map(|tool| tool["name"].as_str().expect("tool name"))
        .collect::<Vec<_>>();
    assert!(names.contains(&"work_item_amend"));
    assert!(names.contains(&"work_item_amendments"));
    assert!(names.contains(&"work_item_environment_drift"));

    let by_name = tools
        .iter()
        .map(|tool| (tool["name"].as_str().expect("tool name"), tool))
        .collect::<std::collections::BTreeMap<_, _>>();
    let amend_schema = &by_name["work_item_amend"]["inputSchema"];
    assert_eq!(amend_schema["additionalProperties"], false);
    let request_schema = &amend_schema["properties"]["request"];
    assert_eq!(request_schema["additionalProperties"], false);
    assert_eq!(
        request_schema["properties"]["changes"]["items"]["additionalProperties"],
        false
    );
    let history_schema = &by_name["work_item_amendments"]["inputSchema"];
    assert_eq!(history_schema["additionalProperties"], false);
    let drift_schema = &by_name["work_item_environment_drift"]["inputSchema"];
    assert_eq!(drift_schema["additionalProperties"], false);
    assert_eq!(
        drift_schema["properties"]["action"]["enum"],
        serde_json::json!(["check", "record"])
    );
}

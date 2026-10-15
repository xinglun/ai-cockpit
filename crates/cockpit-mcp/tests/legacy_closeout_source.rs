use cockpit_core::Digest;
use cockpit_mcp::{handle_request, handle_request_for_repo};
use cockpit_protocol::{PROTOCOL_VERSION, RuntimeContext};
use serde_json::json;
fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "legacy-source-mcp-test".into(),
        protocol_version: PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(b"legacy-source-mcp-test"),
    }
}
#[test]
fn legacy_source_tools_expose_strict_typed_plan_and_record() {
    let response = handle_request(
        &json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}),
        &runtime(),
    );
    let tools = response["result"]["tools"].as_array().unwrap();
    for (name, required) in [
        ("work_item_legacy_closeout_source_plan", "request"),
        ("work_item_legacy_closeout_source_record", "plan"),
    ] {
        let tool = tools
            .iter()
            .find(|tool| tool["name"] == name)
            .expect("explicit legacy source tool is discoverable");
        assert_eq!(tool["inputSchema"]["additionalProperties"], false);
        assert!(
            tool["inputSchema"]["required"]
                .as_array()
                .unwrap()
                .contains(&json!(required))
        );
        assert_eq!(
            tool["inputSchema"]["properties"][required]["additionalProperties"],
            false
        );
    }
}
#[test]
fn wrong_nested_request_is_rejected_before_repository_access() {
    let absent = std::env::temp_dir().join("ai-cockpit-mcp-legacy-source-absent");
    let response = handle_request_for_repo(
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"work_item_legacy_closeout_source_plan","arguments":{"workItemId":"WI-LEGACY-SOURCE","request":{"schemaVersion":1,"operation":"extend","unexpected":true}}}}),
        &absent,
        &runtime(),
    );
    assert_eq!(response["result"]["isError"], true, "{response}");
    let message = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(message.contains("unknown field"), "{message}");
    assert!(!absent.exists());
}

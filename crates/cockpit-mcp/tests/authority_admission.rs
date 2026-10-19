//! P2b denial wiring through the real MCP dispatcher; no production fixture provider.
use serde_json::{Value, json};
use std::{fs, path::Path};

struct Fixture(tempfile::TempDir);
impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let active = directory.path().join(".ai/work-items/active");
        fs::create_dir_all(&active).unwrap();
        fs::write(
            active.join("WI-FIXTURE.contract.json"),
            br#"{"governanceProfile":{"goalAuthority":{"required":true}}}"#,
        )
        .unwrap();
        Self(directory)
    }
    fn root(&self) -> &Path {
        self.0.path()
    }
}
fn runtime() -> cockpit_protocol::RuntimeContext {
    cockpit_protocol::RuntimeContext {
        runtime_version: "1.0.1".into(),
        protocol_version: 1,
        runtime_digest: format!("sha256:{}", "a".repeat(64)).parse().unwrap(),
    }
}
fn call(root: &Path, name: &str, arguments: Value) -> Value {
    cockpit_mcp::handle_request_for_repo(
        &json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":name,"arguments":arguments}}),
        root,
        &runtime(),
    )
}

#[test]
fn b15_tools_and_identity_controls_aliases_reject_before_business_reads() {
    let fixture = Fixture::new();
    let mut missing = vec![];
    for (tool, args) in [
        ("status", json!({})),
        ("repository_observe", json!({})),
        ("work_item_list", json!({})),
        ("work_item_get", json!({"workItemId":"WI-FIXTURE"})),
        ("work_item_status", json!({"id":"WI-FIXTURE"})),
        (
            "work_item_controls",
            json!({"workItemId":"WI-FIXTURE","controls":{}}),
        ),
        ("work_item_controls", json!({"id":"WI-FIXTURE","input":{}})),
        (
            "work_item_material_review_plan",
            json!({"workItemId":"WI-FIXTURE"}),
        ),
        ("knowledge_query", json!({})),
        ("audit_query", json!({})),
        (
            "preflight",
            json!({"contract":".ai/work-items/active/WI-FIXTURE.contract.json"}),
        ),
        (
            "verify",
            json!({"workItemId":"WI-FIXTURE","command":"synthetic-program-never-executed"}),
        ),
    ] {
        let response = call(fixture.root(), tool, args);
        if !response
            .to_string()
            .contains("goal_authority_source_unsupported")
        {
            missing.push(tool);
        }
    }
    assert!(
        missing.is_empty(),
        "missing authority rejection: {missing:?}"
    );
    assert!(!fixture.root().join(".ai/locks").exists());
}

#[test]
fn b11_tool_discovery_and_static_surface_do_not_scan_business_state() {
    let fixture = Fixture::new();
    let tools = cockpit_mcp::handle_request_for_repo(
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        fixture.root(),
        &runtime(),
    );
    assert_eq!(tools["result"]["tools"].as_array().unwrap().len(), 35);
    let response = call(
        fixture.root(),
        "capability_show",
        json!({"surface":"work-item-outcome"}),
    );
    assert_ne!(response["result"]["isError"], true, "{response}");
    assert!(!fixture.root().join(".ai/locks").exists());
}

#[test]
fn b01_all_discoverable_business_tools_reject_without_a_source() {
    let fixture = Fixture::new();
    let tools = cockpit_mcp::handle_request_for_repo(
        &json!({"jsonrpc":"2.0","id":3,"method":"tools/list"}),
        fixture.root(),
        &runtime(),
    );
    let mut missing = vec![];
    for tool in tools["result"]["tools"].as_array().unwrap() {
        let name = tool["name"].as_str().unwrap();
        // Argument decoding is metadata work. An unavailable source can reject
        // the known business request before any typed payload/business access.
        if !call(fixture.root(), name, json!({}))
            .to_string()
            .contains("goal_authority_source_unsupported")
        {
            missing.push(name);
        }
    }
    assert!(
        missing.is_empty(),
        "business tools without early rejection: {missing:?}"
    );
}

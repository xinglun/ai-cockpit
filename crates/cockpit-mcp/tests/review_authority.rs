use cockpit_core::Digest;
use cockpit_protocol::RuntimeContext;
use serde_json::json;

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        protocol_version: cockpit_protocol::PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(b"fixture-only"),
    }
}

#[test]
fn discovery_separates_unavailable_authority_from_self_declared_controls() {
    let reply = cockpit_mcp::handle_request(
        &json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}),
        &runtime(),
    );
    let tools = reply["result"]["tools"].as_array().unwrap();
    let matches: Vec<_> = tools
        .iter()
        .filter(|tool| tool["name"] == "work_item_review_authority")
        .collect();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0]["inputSchema"]["additionalProperties"], false);
    assert_eq!(
        matches[0]["inputSchema"]["properties"]["action"]["enum"],
        json!([
            "binding", "status", "request", "submit", "correct", "revoke"
        ])
    );
    assert!(
        matches[0]["description"]
            .as_str()
            .unwrap()
            .contains("unavailable")
    );
}

#[test]
fn caller_supplied_actor_is_not_an_authenticated_transport_context() {
    let temp = tempfile::tempdir().unwrap();
    let reply = cockpit_mcp::handle_request_for_repo(
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"work_item_review_authority","arguments":{"workItemId":"WI-FIXTURE","action":"status","actor":"human:Ray"}}}),
        temp.path(),
        &runtime(),
    );
    assert!(reply.to_string().contains("unknown field actor"));
}

#[test]
fn actual_mcp_schema_enumerates_request_operations_without_factory_injection() {
    let reply = cockpit_mcp::handle_request(
        &json!({"jsonrpc":"2.0","id":3,"method":"tools/list"}),
        &runtime(),
    );
    let tool = reply["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "work_item_review_authority")
        .unwrap();
    let selection = &tool["inputSchema"]["properties"]["operation"];
    assert_eq!(selection["additionalProperties"], false);
    let operations = selection["properties"]["operation"]["enum"]
        .as_array()
        .unwrap();
    for operation in [
        "retire",
        "profile_confirmation",
        "cross_checkout_recovery",
        "activation_recovery",
        "legacy_source_registration",
        "legacy_finalization",
        "historical_finalization_recovery",
    ] {
        assert!(operations.contains(&json!(operation)), "{operation}");
    }
    for forbidden in ["actor", "backend", "keyPath", "authorityBackend"] {
        assert!(tool["inputSchema"]["properties"].get(forbidden).is_none());
    }
    let t = tempfile::tempdir().unwrap();
    for extra in [json!({"backend":"mock"}), json!({"actor":"human:Ray"})] {
        let mut selection = json!({"operation":"retire","payload":{}});
        selection
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        let reply = cockpit_mcp::handle_request_for_repo(
            &json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"work_item_review_authority","arguments":{"workItemId":"WI-MCP","action":"request","operation":selection}}}),
            t.path(),
            &runtime(),
        );
        assert!(reply.to_string().contains("unknown field"));
    }
}
#[test]
fn actual_mcp_production_request_cannot_mint_a_nonce_without_host_authority() {
    use std::process::Command;
    let t = tempfile::tempdir().unwrap();
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(t.path())
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--allow-empty",
                "-qm",
                "baseline"
            ])
            .current_dir(t.path())
            .status()
            .unwrap()
            .success()
    );
    cockpit_repository::attach(t.path()).unwrap();
    cockpit_repository::start_work_item_with_options(
        t.path(),
        "WI-MCP",
        "fixture",
        "request issuance",
        &["src/**".into()],
        &cockpit_repository::WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["source".into()],
            ..Default::default()
        },
    )
    .unwrap();
    cockpit_repository::preflight_work_item(
        t.path(),
        &t.path().join(".ai/work-items/active/WI-MCP.contract.json"),
    )
    .unwrap();
    let reply = cockpit_mcp::handle_request_for_repo(
        &json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"work_item_review_authority","arguments":{"workItemId":"WI-MCP","action":"request"}}}),
        t.path(),
        &runtime(),
    );
    assert!(reply.to_string().contains("authority_unavailable"));
    assert_ne!(reply["result"]["isError"], false);
}

use cockpit_core::Digest;
use cockpit_protocol::{RuntimeContext, UsageRecordRequest, UsageSourceKind, UsageUnit};
use sha2::{Digest as _, Sha256};
use std::{fs, process::Command};

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("repository");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(root.path())
            .status()
            .expect("git init")
            .success()
    );
    cockpit_repository::attach(root.path()).expect("attach");
    cockpit_repository::start_work_item_with_options(
        root.path(),
        "WI-AUDIT-CLI",
        "audit",
        "query",
        &[".ai/**".into()],
        &cockpit_repository::WorkItemStartOptions {
            authority: "authorized".into(),
            ..Default::default()
        },
    )
    .expect("start");
    root
}

fn runtime(binary: &str) -> RuntimeContext {
    RuntimeContext {
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        protocol_version: cockpit_protocol::PROTOCOL_VERSION,
        runtime_digest: format!(
            "sha256:{}",
            hex::encode(Sha256::digest(fs::read(binary).expect("binary")))
        )
        .parse()
        .expect("digest"),
    }
}

#[test]
fn cli_and_mcp_audit_query_share_filtered_page_and_discovery() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let root = fixture();
    let cli = Command::new(binary)
        .args([
            "audit",
            "query",
            "--repo",
            root.path().to_str().expect("repo"),
            "--work-item-id",
            "WI-AUDIT-CLI",
            "--event-type",
            "work_item_started",
            "--limit",
            "1",
            "--display-timezone",
            "Asia/Tokyo",
        ])
        .output()
        .expect("CLI query");
    assert!(
        cli.status.success(),
        "{}",
        String::from_utf8_lossy(&cli.stderr)
    );
    let cli_page: serde_json::Value = serde_json::from_slice(&cli.stdout).expect("CLI JSON");
    assert_eq!(cli_page["returnedCount"], 1);
    assert_eq!(cli_page["items"][0]["eventType"], "work_item_started");
    let mcp = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({
            "jsonrpc":"2.0","id":1,"method":"tools/call","params":{
                "name":"audit_query","arguments":{
                    "workItemId":"WI-AUDIT-CLI", "eventType":"work_item_started",
                    "limit":1,"displayTimezone":"Asia/Tokyo"
                }
            }
        }),
        root.path(),
        &runtime(binary),
    );
    assert!(mcp.get("error").is_none(), "{mcp}");
    let content = mcp["result"]["content"][0]["text"]
        .as_str()
        .expect("MCP text");
    let mcp_page: serde_json::Value = serde_json::from_str(content).expect("MCP page");
    assert_eq!(mcp_page["items"], cli_page["items"]);
    assert_eq!(mcp_page["filters"], cli_page["filters"]);
    let list = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        root.path(),
        &runtime(binary),
    );
    let tools = list["result"]["tools"].as_array().expect("tools");
    let tool = tools
        .iter()
        .find(|tool| tool["name"] == "audit_query")
        .expect("audit query discovery");
    assert!(tool["inputSchema"]["properties"]["displayTimezone"].is_object());
    let invalid = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{
            "name":"audit_query","arguments":{"limit":0}
        }}),
        root.path(),
        &runtime(binary),
    );
    assert_eq!(invalid["result"]["isError"], true);
    let interface = Command::new(binary)
        .args([
            "capability",
            "show",
            "--repo",
            root.path().to_str().expect("repo"),
            "--surface",
            "audit-query",
        ])
        .output()
        .expect("interface description");
    assert!(
        interface.status.success(),
        "{}",
        String::from_utf8_lossy(&interface.stderr)
    );
    let description: serde_json::Value =
        serde_json::from_slice(&interface.stdout).expect("description JSON");
    assert_eq!(description["name"], "audit-query");
}

#[test]
fn cli_and_mcp_share_combined_filters_and_bound_cursor_pages() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let root = fixture();
    let source = root.path().join(".ai/evidence/audit-source.json");
    fs::write(&source, b"{}\n").expect("usage source");
    let mut request = UsageRecordRequest {
        schema_version: 1,
        repository_id: cockpit_repository::repository_id(root.path()).to_string(),
        work_item_id: "WI-AUDIT-CLI".into(),
        source_event_id: "turn-1".into(),
        source_kind: UsageSourceKind::AgentDeclared,
        actor: Some("agent:test".into()),
        configured_model: Some("configured-model".into()),
        reported_model: Some("reported-model".into()),
        role: "implementer".into(),
        phase: "implementation".into(),
        unit: UsageUnit::Turn,
        input_tokens: Some(4),
        output_tokens: None,
        cached_input_tokens: None,
        reasoning_tokens: None,
        source_observed_at: None,
        evidence_ref: ".ai/evidence/audit-source.json".into(),
        evidence_digest: Digest::sha256_bytes(b"{}\n"),
    };
    let first = cockpit_repository::record_work_item_usage(root.path(), &request, &runtime(binary))
        .expect("first usage");
    request.source_event_id = "turn-2".into();
    request.input_tokens = Some(5);
    cockpit_repository::record_work_item_usage(root.path(), &request, &runtime(binary))
        .expect("second usage");
    let from = first.received_at;
    let query = |cursor: Option<&str>| {
        let mut args = vec![
            "audit",
            "query",
            "--repo",
            root.path().to_str().expect("repo"),
            "--work-item-id",
            "WI-AUDIT-CLI",
            "--event-type",
            "usage_recorded",
            "--reported-model",
            "reported-model",
            "--actor",
            "agent:test",
            "--from",
            from.as_str(),
            "--to",
            "2999-01-01T00:00:00Z",
            "--limit",
            "1",
        ];
        if let Some(cursor) = cursor {
            args.extend(["--cursor", cursor]);
        }
        let cli = Command::new(binary).args(args).output().expect("CLI query");
        assert!(
            cli.status.success(),
            "{}",
            String::from_utf8_lossy(&cli.stderr)
        );
        let cli: serde_json::Value = serde_json::from_slice(&cli.stdout).expect("CLI page");
        let mut arguments = serde_json::json!({
            "workItemId":"WI-AUDIT-CLI", "eventType":"usage_recorded",
            "reportedModel":"reported-model", "actor":"agent:test",
            "from":from, "to":"2999-01-01T00:00:00Z", "limit":1
        });
        if let Some(cursor) = cursor {
            arguments["cursor"] = cursor.into();
        }
        let mcp = cockpit_mcp::handle_request_for_repo(
            &serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{
                "name":"audit_query","arguments":arguments
            }}),
            root.path(),
            &runtime(binary),
        );
        assert!(mcp.get("error").is_none(), "{mcp}");
        let mcp: serde_json::Value = serde_json::from_str(
            mcp["result"]["content"][0]["text"]
                .as_str()
                .expect("MCP text"),
        )
        .expect("MCP page");
        assert_eq!(mcp["items"], cli["items"]);
        assert_eq!(mcp["filters"], cli["filters"]);
        assert_eq!(mcp["coverage"], cli["coverage"]);
        assert_eq!(mcp["sourceSnapshotDigest"], cli["sourceSnapshotDigest"]);
        (cli, mcp)
    };
    let (first_cli, _) = query(None);
    assert_eq!(first_cli["returnedCount"], 1);
    assert_eq!(first_cli["truncated"], true);
    assert_eq!(first_cli["pageUsageTotals"]["inputTokens"], 4);
    let cursor = first_cli["nextCursor"].as_str().expect("next cursor");
    let (second_cli, _) = query(Some(cursor));
    assert_eq!(second_cli["returnedCount"], 1);
    assert_eq!(second_cli["truncated"], false);
    assert_eq!(second_cli["pageUsageTotals"]["inputTokens"], 5);
}

#[test]
fn filtered_export_preserves_legacy_wire_and_output_conflict() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let root = fixture();
    let repo = root.path().to_str().expect("repo");
    let filtered = Command::new(binary)
        .args([
            "audit",
            "export",
            "--repo",
            repo,
            "--event-type",
            "work_item_started",
            "--limit",
            "1",
        ])
        .output()
        .expect("filtered export");
    assert!(
        filtered.status.success(),
        "{}",
        String::from_utf8_lossy(&filtered.stderr)
    );
    let page: serde_json::Value = serde_json::from_slice(&filtered.stdout).expect("filtered JSON");
    assert_eq!(page["returnedCount"], 1);
    let legacy = Command::new(binary)
        .args(["audit", "export", "--repo", repo])
        .output()
        .expect("legacy export");
    assert!(
        legacy.status.success(),
        "{}",
        String::from_utf8_lossy(&legacy.stderr)
    );
    let old: serde_json::Value = serde_json::from_slice(&legacy.stdout).expect("legacy JSON");
    assert_eq!(old["schemaVersion"], 1);
    assert!(old.get("exportDigest").is_some());
    assert!(old.get("sourceSnapshotDigest").is_none());
    let output = root.path().join("filtered.json");
    fs::write(&output, b"preserve me").expect("existing output");
    let conflict = Command::new(binary)
        .args([
            "audit",
            "export",
            "--repo",
            repo,
            "--event-type",
            "work_item_started",
            "--output",
            output.to_str().expect("output"),
        ])
        .output()
        .expect("output conflict");
    assert!(!conflict.status.success());
    assert_eq!(fs::read(&output).expect("output"), b"preserve me");
    let fresh = root.path().join("filtered-new.json");
    for _ in 0..2 {
        let saved = Command::new(binary)
            .args([
                "audit",
                "export",
                "--repo",
                repo,
                "--event-type",
                "work_item_started",
                "--limit",
                "1",
                "--output",
                fresh.to_str().expect("output"),
            ])
            .output()
            .expect("idempotent output");
        assert!(
            saved.status.success(),
            "{}",
            String::from_utf8_lossy(&saved.stderr)
        );
    }
    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(&fresh).expect("saved page")).expect("saved JSON");
    assert_eq!(saved["returnedCount"], 1);
}

#[test]
fn unfiltered_export_accepts_existing_schema_one_bytes() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let root = fixture();
    let repo = root.path().to_str().expect("repo");
    let legacy_manifest = cockpit_repository::export_audit_events(root.path(), &runtime(binary))
        .expect("existing schema-one manifest");
    let legacy_bytes = serde_json::to_vec_pretty(&legacy_manifest)
        .expect("pre-feature typed manifest serialization");
    let output = root.path().join("existing-schema-one.json");
    fs::write(&output, &legacy_bytes).expect("existing export bytes");

    let result = Command::new(binary)
        .args([
            "audit",
            "export",
            "--repo",
            repo,
            "--output",
            output.to_str().expect("output"),
        ])
        .output()
        .expect("repeat unfiltered export");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(fs::read(output).expect("preserved export"), legacy_bytes);
    assert_eq!(result.stdout, [legacy_bytes, b"\n".to_vec()].concat());
}

#[test]
fn audit_help_and_mcp_initialize_expose_only_public_query_surface() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    for args in [
        vec!["--help"],
        vec!["audit", "--help"],
        vec!["audit", "query", "--help"],
        vec!["audit", "export", "--help"],
    ] {
        let output = Command::new(binary).args(&args).output().expect("help");
        assert!(output.status.success(), "{args:?}");
        let text = String::from_utf8(output.stdout).expect("help text");
        assert!(!text.contains("__composition-supervisor"), "{args:?}");
        if args.len() == 3 {
            for flag in [
                "--from",
                "--to",
                "--reported-model",
                "--cursor",
                "--display-timezone",
            ] {
                assert!(text.contains(flag), "{args:?}: {flag}");
            }
        }
    }
    let root = fixture();
    let init = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({"jsonrpc":"2.0","id":3,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"audit-test","version":"1"}}}),
        root.path(),
        &runtime(binary),
    );
    assert_eq!(init["result"]["serverInfo"]["name"], "ai-cockpit");
}

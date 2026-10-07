use cockpit_core::Digest;
use cockpit_protocol::{RuntimeContext, UsageRecordRequest, UsageSourceKind, UsageUnit};
use sha2::{Digest as _, Sha256};
use std::{fs, path::Path, process::Command};

fn fixture() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("repository directory");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    cockpit_repository::attach(directory.path()).expect("attach repository");
    cockpit_repository::start_work_item_with_options(
        directory.path(),
        "WI-CLI-MCP-USAGE",
        "record explicit usage",
        "compare CLI and MCP receipt semantics",
        &[".ai/**".into()],
        &cockpit_repository::WorkItemStartOptions {
            authority: "authorized".into(),
            ..Default::default()
        },
    )
    .expect("start Work Item");
    fs::write(
        directory.path().join(".ai/evidence/usage-source.json"),
        b"{}\n",
    )
    .expect("usage source");
    directory
}

fn request(root: &Path) -> UsageRecordRequest {
    UsageRecordRequest {
        schema_version: 1,
        repository_id: cockpit_repository::repository_id(root).to_string(),
        work_item_id: "WI-CLI-MCP-USAGE".into(),
        source_event_id: "turn-1".into(),
        source_kind: UsageSourceKind::HostReported,
        actor: None,
        configured_model: Some("configured-model".into()),
        reported_model: Some("reported-model".into()),
        role: "implementer".into(),
        phase: "implementation".into(),
        unit: UsageUnit::Turn,
        input_tokens: Some(10),
        output_tokens: None,
        cached_input_tokens: Some(3),
        reasoning_tokens: None,
        source_observed_at: None,
        evidence_ref: ".ai/evidence/usage-source.json".into(),
        evidence_digest: Digest::sha256_bytes(b"{}\n"),
    }
}

fn runtime(binary: &str) -> RuntimeContext {
    let bytes = fs::read(binary).expect("CLI binary bytes");
    RuntimeContext {
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        protocol_version: cockpit_protocol::PROTOCOL_VERSION,
        runtime_digest: format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
            .parse()
            .expect("runtime digest"),
    }
}

fn mcp_record(
    root: &Path,
    request: &UsageRecordRequest,
    runtime: &RuntimeContext,
) -> serde_json::Value {
    cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{
            "name":"work_item_usage_record","arguments":{"request":request}
        }}),
        root,
        runtime,
    )
}

#[test]
fn cli_and_mcp_record_have_identical_receipts_and_conflict_outcomes() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let root = fixture();
    let request = request(root.path());
    let input = root.path().join("usage-request.json");
    fs::write(&input, serde_json::to_vec(&request).expect("request JSON")).expect("request file");
    let cli = Command::new(binary)
        .args(["work-item", "usage", "record", "--repo"])
        .arg(root.path())
        .arg("--input")
        .arg(&input)
        .output()
        .expect("CLI usage record");
    assert!(
        cli.status.success(),
        "CLI: {}",
        String::from_utf8_lossy(&cli.stderr)
    );
    let cli_receipt: serde_json::Value = serde_json::from_slice(&cli.stdout).expect("CLI receipt");
    let mcp = mcp_record(root.path(), &request, &runtime(binary));
    assert_eq!(mcp["result"]["isError"], false, "MCP: {mcp}");
    assert_eq!(mcp["result"]["structuredContent"], cli_receipt);
    assert_eq!(cli_receipt["modelAssurance"], "caller_claim");
    assert_eq!(cli_receipt["tokenAssurance"], "caller_claim");
    assert!(cli_receipt["sourceObservedAt"].is_null());

    let mut conflicting = request;
    conflicting.input_tokens = Some(11);
    fs::write(
        &input,
        serde_json::to_vec(&conflicting).expect("conflict JSON"),
    )
    .expect("conflict file");
    let cli_conflict = Command::new(binary)
        .args(["work-item", "usage", "record", "--repo"])
        .arg(root.path())
        .arg("--input")
        .arg(&input)
        .output()
        .expect("CLI conflicting usage");
    assert!(!cli_conflict.status.success());
    let mcp_conflict = mcp_record(root.path(), &conflicting, &runtime(binary));
    assert_eq!(mcp_conflict["result"]["isError"], true);
    for error in [
        String::from_utf8_lossy(&cli_conflict.stderr).into_owned(),
        mcp_conflict["result"]["content"][0]["text"]
            .as_str()
            .expect("MCP error")
            .to_owned(),
    ] {
        assert!(
            error.contains("usage source event is bound to different content"),
            "{error}"
        );
    }
    assert_eq!(
        cockpit_repository::read_work_item_usage(root.path(), "WI-CLI-MCP-USAGE", None)
            .expect("usage query")
            .receipt_refs
            .len(),
        1,
    );
}

#[test]
fn usage_help_and_canonical_description_are_discoverable() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    for args in [
        &["--help"][..],
        &["work-item", "--help"][..],
        &["work-item", "usage", "--help"],
        &["work-item", "usage", "record", "--help"],
    ] {
        let output = Command::new(binary)
            .args(args)
            .output()
            .expect("usage help");
        assert!(output.status.success(), "{args:?}");
        let text = String::from_utf8_lossy(&output.stdout);
        if args == ["--help"] {
            assert!(text.contains("work-item"));
        } else {
            assert!(text.contains("usage") || text.contains("record"));
        }
        if args.ends_with(&["record", "--help"]) {
            assert!(text.contains(cockpit_protocol::WORK_ITEM_USAGE_RECORD_INPUT_DESCRIPTION));
        }
        assert!(!text.contains("__composition-supervisor"));
    }
    let root = fixture();
    let description = Command::new(binary)
        .args(["capability", "show", "--repo"])
        .arg(root.path())
        .args(["--surface", "work-item-usage-record"])
        .output()
        .expect("usage interface description");
    assert!(
        description.status.success(),
        "{}",
        String::from_utf8_lossy(&description.stderr)
    );
    let facts: serde_json::Value =
        serde_json::from_slice(&description.stdout).expect("description");
    assert_eq!(facts["name"], "work-item-usage-record");
    assert_eq!(facts["surfaces"].as_array().expect("surfaces").len(), 2);
}

#[test]
fn closed_outcome_cli_and_mcp_include_usage_recorded_after_archive() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let root = fixture();
    let id = "WI-CLI-MCP-USAGE";
    let runtime = runtime(binary);
    let first = request(root.path());
    cockpit_repository::record_work_item_usage(root.path(), &first, &runtime)
        .expect("record before finish");
    let contract = root
        .path()
        .join(format!(".ai/work-items/active/{id}.contract.json"));
    cockpit_repository::preflight_work_item(root.path(), &contract).expect("preflight");
    cockpit_repository::checkpoint_work_item(root.path(), id).expect("checkpoint");
    cockpit_repository::record_verification(
        root.path(),
        id,
        &serde_json::json!({"passed":true,"nodesPlanned":1}),
        "1.0.1-test",
        &Digest::sha256_bytes(b"usage test runtime"),
    )
    .expect("verification");
    cockpit_repository::finish_work_item(root.path(), id).expect("finish");
    cockpit_repository::archive_work_item(root.path(), id).expect("archive");
    let mut late = first;
    late.source_event_id = "turn-after-archive".into();
    cockpit_repository::record_work_item_usage(root.path(), &late, &runtime)
        .expect("record after archive");
    cockpit_repository::close_work_item_with_decision(root.path(), id, "approved").expect("close");

    let cli = Command::new(binary)
        .args(["work-item", "outcome", "--repo"])
        .arg(root.path())
        .args(["--id", id, "--json"])
        .output()
        .expect("CLI Outcome JSON");
    assert!(
        cli.status.success(),
        "{}",
        String::from_utf8_lossy(&cli.stderr)
    );
    let cli: serde_json::Value = serde_json::from_slice(&cli.stdout).expect("CLI Outcome");
    let cli_usage = &cli["taskOutcomeReport"]["usage"];
    assert_eq!(
        cli_usage["receiptRefs"].as_array().expect("CLI refs").len(),
        2
    );
    assert_eq!(cli_usage["totals"]["inputTokens"], 20);

    let mcp = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({"jsonrpc":"2.0","id":9,"method":"tools/call","params":{
            "name":"work_item_outcome","arguments":{"workItemId":id,"language":"zh-CN"}
        }}),
        root.path(),
        &runtime,
    );
    assert_eq!(mcp["result"]["isError"], false, "{mcp}");
    let mcp_usage = &mcp["result"]["structuredContent"]["outcome"]["taskOutcomeReport"]["usage"];
    assert_eq!(mcp_usage, cli_usage);
    assert!(
        mcp["result"]["structuredContent"]["humanHandoff"]
            .as_str()
            .expect("MCP handoff")
            .contains("记录数: 2")
    );
}

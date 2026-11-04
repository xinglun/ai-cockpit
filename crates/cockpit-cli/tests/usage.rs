use cockpit_core::Digest;
use cockpit_protocol::{RuntimeContext, UsageRecordRequest, UsageSourceKind, UsageUnit};
use sha2::{Digest as _, Sha256};
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

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
    fs::write(directory.path().join("README.md"), "fixture baseline\n").expect("baseline README");
    for (key, value) in [
        ("user.email", "test@example.invalid"),
        ("user.name", "Test"),
    ] {
        assert!(
            Command::new("git")
                .args(["config", key, value])
                .current_dir(directory.path())
                .status()
                .expect("configure fixture Git identity")
                .success()
        );
    }
    assert!(
        Command::new("git")
            .args(["add", "README.md"])
            .current_dir(directory.path())
            .status()
            .expect("stage baseline README")
            .success()
    );
    assert!(
        Command::new("git")
            .args(["commit", "-qm", "fixture baseline"])
            .current_dir(directory.path())
            .status()
            .expect("commit fixture baseline")
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

#[test]
fn process_cleanup_reports_only_this_work_items_unowned_processes() {
    let root = fixture();
    let wi = "WI-CLI-MCP-USAGE";
    let repo_path: &std::path::Path = root.path();
    let summary_path = repo_path.join(format!(".ai/work-items/active/{wi}.summary.json"));
    let mut summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("parse Summary");
    summary["processCleanup"] = ::serde_json::json!({
        "schemaVersion": 1,
        "entries": [
            {"executionId":"first","nodeId":"one","processId":4294967295_u32,
             "identityStrength":"numeric_pid_only","state":"started","directChildReaped":false},
            {"executionId":"second","nodeId":"two","processId":1,
             "identityStrength":"numeric_pid_only","state":"direct_child_reaped","directChildReaped":true}
        ]
    });
    fs::write(
        &summary_path,
        serde_json::to_vec_pretty(&summary).expect("encode Summary"),
    )
    .expect("write fixture Summary");
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["work-item", "process-cleanup", "--repo"])
        .arg(root.path())
        .args(["--id", wi])
        .output()
        .expect("CLI report");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("cleanup report");
    assert_eq!(report["workItemId"], wi);
    assert_eq!(report["state"], "unknown");
    assert_eq!(report["unresolvedCount"], 1);
    assert_eq!(report["action"], "bounded_owned_child_request");
    assert_eq!(report["descendantClosure"], "unknown");
}

#[test]
fn process_cleanup_cold_restart_keeps_an_unowned_child_unknown_and_other_wi_unchanged() {
    let first = fixture();
    let second = fixture();
    let wi = "WI-CLI-MCP-USAGE";
    let mut child = Command::new("git")
        .args(["hash-object", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("start test-owned child awaiting stdin");
    assert!(child.try_wait().expect("observe test Child").is_none());

    let first_summary_path = first
        .path()
        .join(format!(".ai/work-items/active/{wi}.summary.json"));
    let second_summary_path = second
        .path()
        .join(format!(".ai/work-items/active/{wi}.summary.json"));
    let mut first_summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&first_summary_path).expect("first Summary"))
            .expect("parse first Summary");
    first_summary["processCleanup"] = serde_json::json!({
        "schemaVersion": 1,
        "entries": [{
            "executionId": "owner-lost", "nodeId": "test-child",
            "processId": child.id(), "identityStrength": "numeric_pid_only",
            "state": "started", "directChildReaped": false
        }]
    });
    fs::write(
        &first_summary_path,
        serde_json::to_vec_pretty(&first_summary).expect("encode first Summary"),
    )
    .expect("write first fixture Summary");
    let first_before = fs::read(&first_summary_path).expect("unresolved first Summary");
    let second_before = fs::read(&second_summary_path).expect("unrelated Summary");

    for _ in 0..2 {
        // Every call is a new CLI process with no ownership of the Child.
        let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
            .args(["work-item", "process-cleanup", "--repo"])
            .arg(first.path())
            .args(["--id", wi])
            .output()
            .expect("cold cleanup request");
        assert!(
            output.status.success(),
            "cold cleanup: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("cold cleanup report");
        assert_eq!(report["state"], "unknown");
        assert_eq!(report["unresolvedCount"], 1);
        assert_eq!(report["ownersContacted"], 0);
        assert_eq!(report["descendantClosure"], "unknown");
        assert_eq!(fs::read(&first_summary_path).unwrap(), first_before);
        assert_eq!(fs::read(&second_summary_path).unwrap(), second_before);
        assert!(
            child
                .try_wait()
                .expect("observe test Child after cleanup")
                .is_none(),
            "a stored numeric PID must not terminate a Child without its owner handle"
        );
    }
    drop(child.stdin.take());
    assert!(child.wait().expect("reap test-owned Child").success());
}

#[test]
fn usage_import_reads_completed_response_counters_without_claiming_trusted_metering() {
    let root = fixture();
    let source = serde_json::to_vec(&serde_json::json!({
        "object": "response", "id": "resp_fixture_import", "status": "completed",
        "model": "fixture-model", "created_at": 1_700_000_000,
        "usage": {"input_tokens": 11, "output_tokens": 7, "total_tokens": 18,
            "input_tokens_details": {"cached_tokens": 4},
            "output_tokens_details": {"reasoning_tokens": 2}}
    }))
    .expect("source fixture");
    fs::write(root.path().join(".ai/evidence/usage-source.json"), &source)
        .expect("write source fixture");
    let input = serde_json::json!({
        "schemaVersion": 1,
        "repositoryId": cockpit_repository::repository_id(root.path()).to_string(),
        "workItemId": "WI-CLI-MCP-USAGE",
        "sourceFormat": "openai-response-json-v1",
        "sourceRef": ".ai/evidence/usage-source.json",
        "sourceDigest": Digest::sha256_bytes(&source),
        "configuredModel": "configured-fixture-model",
        "role": "implementer", "phase": "implementation"
    });
    let input_path = root.path().join("usage-import.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&input).expect("import request"),
    )
    .expect("write request");
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["work-item", "usage", "import", "--repo"])
        .arg(root.path())
        .arg("--input")
        .arg(input_path)
        .output()
        .expect("CLI import");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: serde_json::Value = serde_json::from_slice(&output.stdout).expect("receipt");
    assert_eq!(
        receipt["request"]["sourceEventId"],
        "openai-response:resp_fixture_import"
    );
    assert_eq!(receipt["request"]["unit"], "invocation");
    assert_eq!(receipt["request"]["inputTokens"], 11);
    assert_eq!(receipt["request"]["outputTokens"], 7);
    assert_eq!(receipt["request"]["cachedInputTokens"], 4);
    assert_eq!(receipt["request"]["reasoningTokens"], 2);
    assert_eq!(receipt["request"]["reportedModel"], "fixture-model");
    assert_eq!(
        receipt["request"]["configuredModel"],
        "configured-fixture-model"
    );
    assert_eq!(receipt["tokenAssurance"], "caller_claim");
    assert_eq!(receipt["modelAssurance"], "caller_claim");
    assert!(receipt["sourceObservedAt"].is_null());
    let mcp = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({"jsonrpc":"2.0","id":71,"method":"tools/call","params":{
            "name":"work_item_usage_import","arguments":{"request":input}
        }}),
        root.path(),
        &runtime(env!("CARGO_BIN_EXE_ai-cockpit")),
    );
    assert_eq!(mcp["result"]["isError"], false, "{mcp}");
    assert_eq!(mcp["result"]["structuredContent"], receipt);
}

#[test]
fn usage_import_rejects_invalid_source_counters_before_publishing_a_receipt() {
    for (mutation, expected) in [
        (
            serde_json::json!({"status": "in_progress"}),
            "completed response",
        ),
        (
            serde_json::json!({"usage": {"input_tokens": -1}}),
            "unsigned integer",
        ),
        (
            serde_json::json!({"usage": {"input_tokens": 1.5}}),
            "unsigned integer",
        ),
        (
            serde_json::json!({"usage": {"input_tokens": 1, "output_tokens": 2,
            "input_tokens_details": {"cached_tokens": 2}}}),
            "subset",
        ),
        (
            serde_json::json!({"usage": {"input_tokens": 1, "output_tokens": 2,
            "output_tokens_details": {"reasoning_tokens": 3}}}),
            "subset",
        ),
        (
            serde_json::json!({"usage": {"input_tokens": 1, "output_tokens": 2,
            "total_tokens": 4}}),
            "total",
        ),
    ] {
        let root = fixture();
        let mut response = serde_json::json!({"object":"response", "id":"resp_boundary",
            "status":"completed", "model":"fixture-model", "usage":null});
        for (key, value) in mutation.as_object().expect("mutation") {
            response[key] = value.clone();
        }
        let source = serde_json::to_vec(&response).expect("source");
        fs::write(root.path().join(".ai/evidence/usage-source.json"), &source).expect("source");
        let input = serde_json::json!({"schemaVersion":1,
            "repositoryId":cockpit_repository::repository_id(root.path()).to_string(),
            "workItemId":"WI-CLI-MCP-USAGE", "sourceFormat":"openai-response-json-v1",
            "sourceRef":".ai/evidence/usage-source.json", "sourceDigest":Digest::sha256_bytes(&source),
            "role":"implementer", "phase":"implementation"});
        let path = root.path().join("usage-import.json");
        fs::write(&path, serde_json::to_vec(&input).expect("request")).expect("request");
        let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
            .args(["work-item", "usage", "import", "--repo"])
            .arg(root.path())
            .arg("--input")
            .arg(path)
            .output()
            .expect("import");
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected),
            "expected {expected}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            cockpit_repository::read_work_item_usage(root.path(), "WI-CLI-MCP-USAGE", None)
                .expect("read only")
                .receipt_refs
                .is_empty()
        );
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

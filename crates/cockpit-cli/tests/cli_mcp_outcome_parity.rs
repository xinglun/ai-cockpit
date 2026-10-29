use sha2::{Digest as ShaDigest, Sha256};
use std::{collections::BTreeSet, fs, path::Path, process::Command};

use cockpit_protocol::work_item_outcome_interface_specs;

mod common;

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("tempdir");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "--quiet",
                "--allow-empty",
                "-m",
                "fixture baseline",
            ])
            .current_dir(directory.path())
            .status()
            .expect("git commit baseline")
            .success()
    );
    directory
}

#[test]
fn cli_outcome_help_projects_protocol_owned_parameter_descriptions() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let output = Command::new(binary)
        .args(["work-item", "outcome", "--help"])
        .output()
        .expect("run outcome help");
    assert!(output.status.success(), "outcome help failed");
    let help = String::from_utf8_lossy(&output.stdout);
    let specs = work_item_outcome_interface_specs("cli").expect("CLI outcome surface");
    for name in ["delivery", "json", "view", "language", "display-timezone"] {
        let spec = specs
            .iter()
            .find(|spec| spec.name == name)
            .unwrap_or_else(|| panic!("missing protocol-owned spec: {name}"));
        assert!(
            help.contains(&spec.description),
            "CLI help for --{name} must be projected from the protocol description: {}",
            spec.description
        );
        assert!(
            help.contains(&format!("--{}", spec.name)),
            "CLI descriptor must use actual public flag spelling: {}",
            spec.name
        );
    }
}

#[test]
fn public_help_localized_tables_and_mcp_discovery_include_timezone_without_hidden_helper() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    for args in [
        vec!["--help"],
        vec!["work-item", "--help"],
        vec!["work-item", "outcome", "--help"],
        vec!["work-item", "usage", "--help"],
        vec!["work-item", "usage", "record", "--help"],
        vec!["work-item", "authorization-manifest", "--help"],
        vec!["work-item", "authorization-manifest", "plan", "--help"],
        vec!["work-item", "authorization-manifest", "review", "--help"],
        vec!["work-item", "authorization-manifest", "start", "--help"],
        vec!["work-item", "authorization-manifest", "check", "--help"],
        vec!["work-item", "authorization-manifest", "outcome", "--help"],
        vec!["work-item", "authorization-manifest", "inspect", "--help"],
        vec!["work-item", "authorization-manifest", "terminate", "--help"],
        vec!["work-item", "authorization-manifest", "ledger", "--help"],
        vec!["audit", "--help"],
        vec!["audit", "query", "--help"],
        vec!["audit", "export", "--help"],
    ] {
        let output = Command::new(binary)
            .args(&args)
            .output()
            .expect("public help");
        assert!(output.status.success(), "{args:?}");
        let help = String::from_utf8(output.stdout).expect("help UTF-8");
        assert!(!help.contains("__composition-supervisor"), "{args:?}");
        if args == ["work-item", "outcome", "--help"] {
            assert!(help.contains("--display-timezone"));
            assert!(help.contains("IANA timezone"));
        }
        if args == ["work-item", "authorization-manifest", "plan", "--help"] {
            assert!(help.contains("--repo"));
            assert!(help.contains("--input"));
        }
    }
    for docs in [
        include_str!("../../../docs/reference/commands.md"),
        include_str!("../../../docs/reference/commands.zh-CN.md"),
        include_str!("../../../docs/reference/commands.ja.md"),
    ] {
        for name in [
            "work_item_outcome",
            "work_item_usage_record",
            "work_item_usage_import",
            "audit_query",
            "displayTimezone",
            "| `display-timezone` |",
            "--display-timezone",
        ] {
            assert!(docs.contains(name), "missing {name}");
        }
        assert!(!docs.contains("| `display_timezone` |"));
        assert!(!docs.contains("__composition-supervisor"));
    }
    let repo = repository();
    cockpit_repository::attach(repo.path()).expect("attach");
    let runtime = current_runtime_context(binary);
    let initialize = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"surface-test","version":"1"}
        }}),
        repo.path(),
        &runtime,
    );
    assert_eq!(initialize["result"]["serverInfo"]["name"], "ai-cockpit");
    let list = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        repo.path(),
        &runtime,
    );
    let tools = list["result"]["tools"].as_array().expect("tools");
    assert!(
        !tools
            .iter()
            .any(|tool| tool["name"] == "__composition-supervisor")
    );
    for name in ["work_item_outcome", "work_item_usage_record", "audit_query"] {
        assert!(
            tools.iter().any(|tool| tool["name"] == name),
            "missing {name}"
        );
    }
    let outcome = tools
        .iter()
        .find(|tool| tool["name"] == "work_item_outcome")
        .expect("outcome tool");
    assert!(outcome["inputSchema"]["properties"]["displayTimezone"].is_object());
}

fn run_json(binary: &str, repo: &Path, args: &[&str]) -> serde_json::Value {
    let output = Command::new(binary)
        .args(args)
        .args(["--repo"])
        .arg(repo)
        .current_dir(repo)
        .output()
        .expect("run ai-cockpit");
    assert!(
        output.status.success(),
        "args={args:?}, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("machine-readable stdout JSON")
}

#[test]
fn cli_outcome_view_uses_protocol_values_for_parsing() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let repo = repository();
    let accepted = Command::new(binary)
        .args(["work-item", "outcome", "--repo"])
        .arg(repo.path())
        .args(["--id", "WI-CLI-VIEW-PARSER", "--view", "full"])
        .output()
        .expect("run accepted view");
    let accepted_stderr = String::from_utf8_lossy(&accepted.stderr);
    assert!(
        !accepted_stderr.contains("invalid value") && !accepted_stderr.contains("invalid view"),
        "full must be accepted by the CLI parser: {accepted_stderr}"
    );

    let rejected = Command::new(binary)
        .args(["work-item", "outcome", "--repo"])
        .arg(repo.path())
        .args(["--id", "WI-CLI-VIEW-PARSER", "--view", "compact"])
        .output()
        .expect("run rejected view");
    let rejected_stderr = String::from_utf8_lossy(&rejected.stderr);
    assert!(
        rejected_stderr.contains("invalid value")
            && rejected_stderr.contains("summary")
            && rejected_stderr.contains("full"),
        "unknown view must fail with protocol-owned values: {rejected_stderr}"
    );
}

#[test]
fn outcome_timezone_view_is_shared_by_cli_and_mcp_without_changing_facts() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let repo = repository();
    cockpit_repository::attach(repo.path()).expect("attach");
    let runtime = current_runtime_context(binary);
    cockpit_repository::start_work_item_with_options_and_runtime(
        repo.path(),
        "WI-OUTCOME-ZONE",
        "show lifecycle time",
        "show a requested IANA view",
        &[".ai/**".into()],
        &cockpit_repository::WorkItemStartOptions {
            authority: "authorized".into(),
            ..Default::default()
        },
        &runtime,
    )
    .expect("start");
    let summary_path = repo
        .path()
        .join(".ai/work-items/active/WI-OUTCOME-ZONE.summary.json");
    let original_summary = fs::read(&summary_path).expect("UTC lifecycle source");
    let default = human_cli_output(binary, repo.path(), "WI-OUTCOME-ZONE", "en");
    let cli = Command::new(binary)
        .args(["work-item", "outcome", "--repo"])
        .arg(repo.path())
        .args([
            "--id",
            "WI-OUTCOME-ZONE",
            "--language",
            "en",
            "--display-timezone",
            "Asia/Tokyo",
        ])
        .output()
        .expect("CLI Tokyo view");
    assert!(
        cli.status.success(),
        "{}",
        String::from_utf8_lossy(&cli.stderr)
    );
    let cli = String::from_utf8(cli.stdout).expect("CLI UTF-8");
    assert!(!default.contains("Time zone: Asia/Tokyo"));
    assert!(cli.contains("Time zone: Asia/Tokyo"));
    assert!(cli.contains("+09:00"));
    assert!(cli.contains("finish: unknown"));
    assert!(cli.contains("wall elapsed (includes waiting): unknown"));
    let response = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{
            "name":"work_item_outcome","arguments":{
                "workItemId":"WI-OUTCOME-ZONE", "language":"en",
                "displayTimezone":"Asia/Tokyo"
            }
        }}),
        repo.path(),
        &runtime,
    );
    assert_eq!(response["result"]["isError"], false, "{response}");
    let mcp_handoff = response["result"]["structuredContent"]["humanHandoff"]
        .as_str()
        .expect("MCP handoff");
    assert_eq!(
        mcp_handoff
            .lines()
            .find(|line| line.starts_with("Time zone:")),
        cli.lines().find(|line| line.starts_with("Time zone:")),
        "the timezone projection must agree across transports"
    );
    let invalid = Command::new(binary)
        .args(["work-item", "outcome", "--repo"])
        .arg(repo.path())
        .args([
            "--id",
            "WI-OUTCOME-ZONE",
            "--display-timezone",
            "Invalid/Zone",
        ])
        .output()
        .expect("invalid zone");
    assert!(!invalid.status.success());
    assert_eq!(
        fs::read(summary_path).expect("unchanged UTC facts"),
        original_summary
    );
}

/// Reconstruct the exact `RuntimeContext` that the `binary` under test would
/// compute for itself (see `crates/cockpit-cli/src/runtime_identity.rs`), so
/// an MCP call made in-process against the same repository is bound to the
/// identical runtime identity as the CLI subprocess, and the two outputs can
/// be compared for genuine equality rather than differing only on
/// incidental runtime metadata.
fn current_runtime_context(binary: &str) -> cockpit_protocol::RuntimeContext {
    let bytes = fs::read(binary).expect("read runtime binary");
    let digest = format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
        .parse::<cockpit_core::Digest>()
        .expect("valid digest");
    cockpit_protocol::RuntimeContext {
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        protocol_version: cockpit_protocol::PROTOCOL_VERSION,
        runtime_digest: digest,
    }
}

#[test]
fn explicit_batch_authorization_manifest_has_cli_mcp_parity() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let repo = repository();
    cockpit_repository::attach(repo.path()).expect("attach repository");
    let runtime = current_runtime_context(binary);
    let work_item_id = "WI-CLI-MCP-BATCH";
    let actor = "agent:batch-worker";
    let authority_source = "conversation:batch-owner-selected-WI-CLI-MCP-BATCH";
    let policy = cockpit_protocol::TaskAuthorizationPolicy {
        schema_version: 1,
        agent_actor: actor.into(),
        authority_source: authority_source.into(),
        allowed_actions: vec!["run_verification".into()],
        closeout_actions: Vec::new(),
        expires_at: Some("2099-01-01T00:00:00Z".into()),
        public_target: None,
    };
    cockpit_repository::start_work_item_with_options(
        repo.path(),
        work_item_id,
        "prepare one explicit batch member",
        "exercise CLI and MCP batch lifecycle parity",
        &["README.md".into()],
        &cockpit_repository::WorkItemStartOptions {
            task_authorization: Some(policy),
            authority: "authorized".into(),
            acceptance_criteria: vec![
                "only the exact member and its finite budget are recorded".into(),
            ],
            verification_commands: vec!["git diff --check".into()],
            ..Default::default()
        },
    )
    .expect("start bounded member");
    let contract_path = repo.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    cockpit_repository::preflight_work_item_with_runtime_report(
        repo.path(),
        &contract_path,
        &runtime,
    )
    .expect("preflight member policy");
    let contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("member Contract"))
            .expect("Contract JSON");
    let summary: serde_json::Value = serde_json::from_slice(
        &fs::read(
            repo.path()
                .join(format!(".ai/work-items/active/{work_item_id}.summary.json")),
        )
        .expect("member Summary"),
    )
    .expect("Summary JSON");
    cockpit_repository::record_work_item_governance_controls_with_runtime(
        repo.path(),
        work_item_id,
        &serde_json::json!({"decisionEvidence":{
            "schemaVersion":1,
            "decisionId":"contract-preflight-review",
            "decision":"confirm_review",
            "workItemId":work_item_id,
            "repositoryId":cockpit_repository::repository_id(repo.path()),
            "contractDigest":cockpit_protocol::digest_json(&contract).expect("Contract digest"),
            "preflightDecisionDigest":summary["preflightDecisionDigest"],
            "repositorySnapshotDigest":summary["preflightRepositorySnapshotDigest"],
            "recordedAt":"2026-10-27T00:00:00Z",
            "recordedBy":"agent:test-recorder",
            "reason":"synthetic isolated member owner confirmation"
        }}),
        &runtime,
    )
    .expect("confirm exact member policy in isolated fixture");

    let make_input = |value: &serde_json::Value| {
        let file = tempfile::NamedTempFile::new().expect("temporary request file");
        fs::write(
            file.path(),
            serde_json::to_vec(value).expect("request JSON"),
        )
        .expect("write request JSON");
        file
    };
    let request = serde_json::json!({
        "schemaVersion":1,
        "repositoryId":cockpit_repository::repository_id(repo.path()),
        "members":[{
            "workItemId":work_item_id,
            "actions":["run_verification"],
            "expiresAt":"2098-01-01T00:00:00Z",
            "budget":{"startQuota":1,"retryBudgetByCheck":{"git diff --check":1}}
        }]
    });
    let request_file = make_input(&request);
    let cli_plan = Command::new(binary)
        .args(["work-item", "authorization-manifest", "plan", "--repo"])
        .arg(repo.path())
        .args(["--input"])
        .arg(request_file.path())
        .output()
        .expect("CLI manifest plan");
    assert!(
        cli_plan.status.success(),
        "CLI plan failed: {}",
        String::from_utf8_lossy(&cli_plan.stderr)
    );
    let cli_manifest: serde_json::Value =
        serde_json::from_slice(&cli_plan.stdout).expect("CLI manifest JSON");
    let mcp_plan = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{
            "name":"work_item_authorization_manifest_plan","arguments":{"request":request}
        }}),
        repo.path(),
        &runtime,
    );
    assert_eq!(mcp_plan["result"]["isError"], false, "{mcp_plan}");
    let mcp_manifest = &mcp_plan["result"]["structuredContent"];
    assert_eq!(
        cli_manifest, *mcp_manifest,
        "canonical member manifest differs"
    );
    let manifest_digest = cli_manifest["manifestDigest"]
        .as_str()
        .expect("manifest digest");

    let manifest_file = make_input(&cli_manifest);
    let review = serde_json::json!({
        "schemaVersion":1,
        "repositoryId":cli_manifest["repositoryId"],
        "manifestDigest":manifest_digest,
        "decision":"confirm_review",
        "reviewerActor":"human:fixture-owner",
        "authoritySource":"conversation:owner-confirmed-exact-batch",
        "recordedAt":"2026-10-27T00:01:00Z",
        "reason":"confirm this exact member, action, destination, and finite budget"
    });
    let review_file = make_input(&review);
    let cli_review = Command::new(binary)
        .args(["work-item", "authorization-manifest", "review", "--repo"])
        .arg(repo.path())
        .args(["--manifest"])
        .arg(manifest_file.path())
        .args(["--review"])
        .arg(review_file.path())
        .output()
        .expect("CLI exact manifest review");
    assert!(
        cli_review.status.success(),
        "CLI review failed: {}",
        String::from_utf8_lossy(&cli_review.stderr)
    );
    let cli_receipt: serde_json::Value =
        serde_json::from_slice(&cli_review.stdout).expect("CLI receipt JSON");
    let mcp_review = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{
            "name":"work_item_authorization_manifest_review","arguments":{
                "manifest":cli_manifest,
                "review":review
            }
        }}),
        repo.path(),
        &runtime,
    );
    assert_eq!(mcp_review["result"]["isError"], false, "{mcp_review}");
    assert_eq!(cli_receipt, mcp_review["result"]["structuredContent"]);

    let member_caller = serde_json::json!({
        "schemaVersion":1,
        "actor":actor,
        "authoritySource":authority_source,
        "assurance":"self_declared"
    });
    let start_request = serde_json::json!({
        "schemaVersion":1,
        "manifestDigest":manifest_digest,
        "workItemId":work_item_id,
        "operationId":"parity-check-run",
        "action":"run_verification",
        "caller":member_caller
    });
    let start_file = make_input(&start_request);
    let cli_start = Command::new(binary)
        .args(["work-item", "authorization-manifest", "start", "--repo"])
        .arg(repo.path())
        .args(["--input"])
        .arg(start_file.path())
        .output()
        .expect("CLI batch start");
    assert!(
        cli_start.status.success(),
        "CLI start failed: {}",
        String::from_utf8_lossy(&cli_start.stderr)
    );
    let cli_start_event: serde_json::Value =
        serde_json::from_slice(&cli_start.stdout).expect("CLI start event");
    assert_eq!(cli_start_event["kind"], "start_reserved");
    let ledger_args = serde_json::json!({
        "manifestDigest":manifest_digest,
        "workItemId":work_item_id
    });
    let mcp_ledger = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{
            "name":"work_item_authorization_batch_ledger","arguments":ledger_args
        }}),
        repo.path(),
        &runtime,
    );
    assert_eq!(mcp_ledger["result"]["isError"], false, "{mcp_ledger}");
    assert_eq!(
        mcp_ledger["result"]["structuredContent"]["events"][0],
        cli_start_event
    );

    let check_request = serde_json::json!({
        "schemaVersion":1,
        "manifestDigest":manifest_digest,
        "workItemId":work_item_id,
        "operationId":"parity-check-run",
        "checkId":"git diff --check",
        "caller":member_caller
    });
    let mcp_check = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{
            "name":"work_item_authorization_batch_check","arguments":{"request":check_request}
        }}),
        repo.path(),
        &runtime,
    );
    assert_eq!(mcp_check["result"]["isError"], false, "{mcp_check}");
    assert_eq!(mcp_check["result"]["structuredContent"]["attempt"], 1);

    let unknown_outcome = serde_json::json!({
        "schemaVersion":1,
        "manifestDigest":manifest_digest,
        "workItemId":work_item_id,
        "operationId":"parity-check-run",
        "checkId":"git diff --check",
        "attempt":1,
        "outcome":"unknown",
        "evidenceRef":"local:interrupted-check",
        "evidenceDigest":cockpit_core::Digest::sha256_bytes(b"interrupted result"),
        "reason":"the check process exited before its result was observed",
        "caller":member_caller
    });
    let outcome_file = make_input(&unknown_outcome);
    let cli_unknown = Command::new(binary)
        .args(["work-item", "authorization-manifest", "outcome", "--repo"])
        .arg(repo.path())
        .args(["--input"])
        .arg(outcome_file.path())
        .output()
        .expect("CLI uncertain result record");
    assert!(
        cli_unknown.status.success(),
        "CLI unknown result failed: {}",
        String::from_utf8_lossy(&cli_unknown.stderr)
    );
    let retry_file = make_input(&check_request);
    let blocked_retry = Command::new(binary)
        .args(["work-item", "authorization-manifest", "check", "--repo"])
        .arg(repo.path())
        .args(["--input"])
        .arg(retry_file.path())
        .output()
        .expect("CLI blocked retry");
    assert!(!blocked_retry.status.success());
    assert!(
        String::from_utf8_lossy(&blocked_retry.stderr)
            .contains("uncertain_outcome_requires_inspection")
    );

    let inspection_request = serde_json::json!({
        "schemaVersion":1,
        "manifestDigest":manifest_digest,
        "workItemId":work_item_id,
        "operationId":"parity-check-run",
        "checkId":"git diff --check",
        "attempt":1,
        "inspectedOutcome":"not_started",
        "evidenceRef":"local:process-inspection",
        "evidenceDigest":cockpit_core::Digest::sha256_bytes(b"verified no process effect"),
        "reason":"inspection confirms the check never started",
        "caller":member_caller
    });
    let mcp_inspection = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{
            "name":"work_item_authorization_batch_inspect","arguments":{"request":inspection_request}
        }}),
        repo.path(),
        &runtime,
    );
    assert_eq!(
        mcp_inspection["result"]["isError"], false,
        "{mcp_inspection}"
    );
    let retry = Command::new(binary)
        .args(["work-item", "authorization-manifest", "check", "--repo"])
        .arg(repo.path())
        .args(["--input"])
        .arg(retry_file.path())
        .output()
        .expect("CLI admitted retry after inspection");
    assert!(
        retry.status.success(),
        "CLI retry failed: {}",
        String::from_utf8_lossy(&retry.stderr)
    );
    let retry_event: serde_json::Value =
        serde_json::from_slice(&retry.stdout).expect("retry event");
    assert_eq!(retry_event["attempt"], 2);

    let termination = serde_json::json!({
        "schemaVersion":1,
        "manifestDigest":manifest_digest,
        "workItemId":work_item_id,
        "decision":"withdrawn",
        "caller":member_caller,
        "reason":"fixture member authority is no longer required"
    });
    let mcp_termination = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({"jsonrpc":"2.0","id":6,"method":"tools/call","params":{
            "name":"work_item_authorization_batch_terminate","arguments":{"request":termination}
        }}),
        repo.path(),
        &runtime,
    );
    assert_eq!(
        mcp_termination["result"]["isError"], false,
        "{mcp_termination}"
    );
    let cli_ledger = Command::new(binary)
        .args(["work-item", "authorization-manifest", "ledger", "--repo"])
        .arg(repo.path())
        .args([
            "--manifest-digest",
            manifest_digest,
            "--work-item-id",
            work_item_id,
        ])
        .output()
        .expect("CLI member ledger");
    assert!(
        cli_ledger.status.success(),
        "CLI ledger failed: {}",
        String::from_utf8_lossy(&cli_ledger.stderr)
    );
    let cli_ledger: serde_json::Value =
        serde_json::from_slice(&cli_ledger.stdout).expect("CLI ledger JSON");
    assert_eq!(cli_ledger["authorizationState"], "withdrawn");
    assert_eq!(cli_ledger["events"].as_array().expect("events").len(), 6);
}

fn human_cli_output(binary: &str, repo: &Path, id: &str, language: &str) -> String {
    let output = Command::new(binary)
        .args(["work-item", "outcome", "--repo"])
        .arg(repo)
        .args(["--id", id])
        .env("AI_COCKPIT_LANGUAGE", language)
        .output()
        .expect("cli human outcome");
    assert!(
        output.status.success(),
        "language={language}, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("human CLI UTF-8")
}

fn continuity_section<'a>(handoff: &'a str, heading: &str) -> &'a str {
    let start = handoff
        .find(heading)
        .expect("continuity heading is present");
    let tail = &handoff[start..];
    let end = [
        "\nCross-WI coordination:",
        "\n跨 WI 协调:",
        "\nCross-WI coordination",
        "\nWI 間調整:",
    ]
    .iter()
    .filter_map(|marker| tail.find(marker))
    .min()
    .unwrap_or(tail.len());
    &tail[..end]
}

fn human_semantic_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| {
            line.starts_with("Outcome:")
                || line.starts_with("- Verification:")
                || line.starts_with("- 验证状态:")
                || line.starts_with("- 検証状態:")
                || line.starts_with("- Lifecycle:")
                || line.starts_with("- 生命周期状态:")
                || line.starts_with("- ライフサイクル状態:")
                || line.starts_with("- Human decision:")
                || line.starts_with("- 人工决定状态:")
                || line.starts_with("- 人間の判断状態:")
                || line.starts_with("- Governance signal:")
                || line.starts_with("- 治理信号:")
                || line.starts_with("- ガバナンスシグナル:")
                || line.starts_with("Human next step")
                || line.starts_with("人的下一步")
                || line.starts_with("人間の次のアクション")
        })
        .collect()
}

fn human_semantic_tokens(text: &str, language: &str) -> BTreeSet<&'static str> {
    let mut tokens = BTreeSet::new();
    let contains_any = |needles: &[&str]| needles.iter().any(|needle| text.contains(needle));
    if text.starts_with("Outcome:") {
        tokens.insert("outcome_marker");
    }
    if contains_any(&["Verification:", "验证状态:", "検証状態:"]) {
        tokens.insert("verification_state");
    }
    if contains_any(&[
        "acceptance evidence",
        "受入れ evidence",
        "受入 evidence",
        "验收证据",
    ]) {
        tokens.insert("acceptance_gap");
    }
    if contains_any(&[
        "intent alignment",
        "intent-alignment",
        "意図",
        "意図の整合",
        "意图对齐",
    ]) {
        tokens.insert("intent_gap");
    }
    if contains_any(&["authorization", "権限", "授权"]) {
        tokens.insert("authorization_scope");
    }
    if contains_any(&["finalization", "finalization", "收尾", "終結"]) {
        tokens.insert("finalization");
    }
    if contains_any(&["Human next step", "人的下一步", "人間の次のアクション"]) {
        tokens.insert("next_action");
    }
    if language == "ja" && text.contains("人間の判断状態:") {
        tokens.insert("localized_human_decision_label");
    }
    tokens
}

fn registry_semantic_tokens(check_id: &str) -> Vec<String> {
    let matrix: serde_json::Value = serde_json::from_str(include_str!(
        "../../../docs/reference/collaboration-scenario-matrix.json"
    ))
    .expect("scenario matrix JSON");
    matrix["executableCheckRegistry"][check_id]["expectedSemantics"]["semanticTokens"]
        .as_array()
        .unwrap_or_else(|| panic!("{check_id} semanticTokens missing"))
        .iter()
        .map(|token| token.as_str().expect("semantic token string").to_owned())
        .collect()
}

/// Automated check for collaboration-language semantic invariant 5 (see
/// docs/reference/collaboration-language-contract.md and
/// docs/reference/collaboration-invariant-coverage.md): the same fact must
/// not contradict itself across the CLI and MCP entry points.
///
/// Unlike `crates/cockpit-mcp/tests/rpc.rs`'s `*_with_cli_parity` tests,
/// which call the MCP handler as an in-process library function and compare
/// it against another in-process rendering call, this test spawns the real
/// `ai-cockpit` CLI binary as a separate subprocess (a genuine second entry
/// point, not just a second function call) and compares its live stdout
/// against a live MCP handler call against the identical repository
/// fixture.
#[test]
fn cli_subprocess_and_mcp_handler_agree_on_the_same_outcome() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let id = "WI-CLI-MCP-PARITY";
    let repo = repository();

    run_json(binary, repo.path(), &["attach"]);
    run_json(
        binary,
        repo.path(),
        &[
            "start",
            "--id",
            id,
            "--intent",
            "prove CLI and MCP agree on the same Outcome for the same repository and Work Item",
            "--goal",
            "a cross-entry-point regression is caught automatically instead of relying on manual review",
            "--scope",
            "README.md",
            "--authority",
            "authorized",
            "--acceptance",
            "A1: CLI stdout outcome and MCP structuredContent.outcome are identical",
            "--required-evidence",
            "verification",
        ],
    );
    common::plan(binary, repo.path(), id);
    run_json(
        binary,
        repo.path(),
        &[
            "preflight",
            "--contract",
            &format!(".ai/work-items/active/{id}.contract.json"),
        ],
    );
    run_json(binary, repo.path(), &["checkpoint", "--id", id]);
    let controls_path = tempfile::NamedTempFile::new().expect("controls input");
    fs::write(
        controls_path.path(),
        serde_json::to_vec_pretty(&serde_json::json!({
            "acceptanceEvidence": [{
                "acceptanceId": "A1",
                "evidence": [{
                    "type": "test",
                    "path": "crates/cockpit-cli/tests/cli_mcp_outcome_parity.rs",
                    "locator": "cli_subprocess_and_mcp_handler_agree_on_the_same_outcome",
                    "verification": "passed"
                }]
            }],
            "intentAlignment": {
                "state": "resolved",
                "evidence": ["crates/cockpit-cli/tests/cli_mcp_outcome_parity.rs"]
            }
        }))
        .expect("controls JSON"),
    )
    .expect("write controls JSON");
    run_json(
        binary,
        repo.path(),
        &[
            "work-item",
            "controls",
            "--id",
            id,
            "--input",
            controls_path.path().to_str().expect("controls path"),
        ],
    );
    run_json(
        binary,
        repo.path(),
        &["verify", "--work-item", id, "--command", "true"],
    );

    // Entry point 1: the real CLI binary, as a separate subprocess.
    let cli_output = Command::new(binary)
        .args(["work-item", "outcome", "--repo"])
        .arg(repo.path())
        .args(["--id", id, "--json"])
        .output()
        .expect("cli outcome");
    assert!(
        cli_output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&cli_output.stderr)
    );
    let cli_outcome: serde_json::Value =
        serde_json::from_slice(&cli_output.stdout).expect("cli outcome JSON");

    // Entry point 2: the production MCP handler, called in-process against
    // the identical repository with a RuntimeContext bound to the exact
    // same binary under test.
    let runtime_context = current_runtime_context(binary);
    let response = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {"name": "work_item_outcome", "arguments": {"workItemId": id}}
        }),
        repo.path(),
        &runtime_context,
    );
    assert_eq!(
        response["result"]["isError"], false,
        "MCP call failed: response={response}"
    );
    let mcp_outcome = &response["result"]["structuredContent"]["outcome"];

    assert_eq!(
        &cli_outcome, mcp_outcome,
        "CLI stdout outcome and MCP structuredContent.outcome diverged for the same repository and Work Item"
    );

    // The comparison above must not be vacuously true because both sides
    // failed to bind to the Work Item at all.
    assert_eq!(cli_outcome["workItemId"], id);
    assert_eq!(mcp_outcome["workItemId"], id);
    assert_eq!(cli_outcome["decisionState"], "green");
    assert!(
        cli_outcome["governanceReasons"].is_null()
            || cli_outcome["governanceReasons"]
                .as_array()
                .is_some_and(Vec::is_empty),
        "machine Outcome must not retain stale governance gaps after the pre-verification controls are recorded: {cli_outcome}"
    );
    assert_eq!(
        cli_outcome["governanceReasons"],
        mcp_outcome["governanceReasons"]
    );
    assert_eq!(cli_outcome["finalization"]["state"], "receipt_missing");
    assert_eq!(
        cli_outcome["finalization"]["action"],
        "record_finalization_receipt"
    );
    assert_eq!(
        cli_outcome["finalization"]["nextAction"]["id"],
        "record_finalization_receipt"
    );

    // Human-language parity is a separate assertion from machine JSON
    // equality. It checks the actual handoff text for stable facts and the
    // non-authorization consequence, while allowing translated labels.
    for (language, required_fragments) in [
        ("en", vec!["Verification evidence is valid"]),
        ("zh", vec!["验证证据有效"]),
        ("ja", vec!["検証 evidence は有効"]),
    ] {
        let cli_handoff = human_cli_output(binary, repo.path(), id, language);
        let mcp_response = cockpit_mcp::handle_request_for_repo(
            &serde_json::json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {"name": "work_item_outcome", "arguments": {"workItemId": id, "language": language}}
            }),
            repo.path(),
            &runtime_context,
        );
        assert_eq!(mcp_response["result"]["isError"], false);
        let mcp_handoff = mcp_response["result"]["structuredContent"]["humanHandoff"]
            .as_str()
            .expect("MCP human handoff");
        assert_eq!(
            human_semantic_lines(&cli_handoff),
            human_semantic_lines(mcp_handoff)
        );
        assert_eq!(
            human_semantic_tokens(&cli_handoff, language),
            human_semantic_tokens(mcp_handoff, language),
            "language={language}: CLI and MCP human handoffs must preserve the same semantic token set"
        );
        let actual_tokens = human_semantic_tokens(&cli_handoff, language);
        for expected in registry_semantic_tokens("human_report_entrypoint_parity") {
            assert!(
                actual_tokens.contains(expected.as_str()),
                "language={language}: human handoff missed registry semantic token {expected}: {cli_handoff}"
            );
        }
        assert!(
            cli_handoff.contains("Outcome:"),
            "{language}: {cli_handoff}"
        );
        assert!(
            cli_handoff.contains("Human next step")
                || cli_handoff.contains("人的下一步")
                || cli_handoff.contains("人間の次のアクション"),
            "{language}: {cli_handoff}"
        );
        for fragment in required_fragments {
            assert!(
                cli_handoff.contains(fragment) && mcp_handoff.contains(fragment),
                "language={language}, fragment={fragment:?}, cli={cli_handoff}, mcp={mcp_handoff}"
            );
        }
    }
}

#[test]
fn successor_continuity_handoff_is_identical_across_cli_and_mcp_without_rewriting_archive() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let predecessor = "WI-CLI-MCP-CONTINUITY-PREDECESSOR";
    let successor = "WI-CLI-MCP-CONTINUITY-SUCCESSOR";
    let repo = repository();
    run_json(binary, repo.path(), &["attach"]);
    run_json(
        binary,
        repo.path(),
        &[
            "start",
            "--id",
            predecessor,
            "--intent",
            "preserve the original local delivery while continuing externally",
            "--goal",
            "show the archived predecessor and active successor in the handoff",
            "--scope",
            "README.md",
            "--authority",
            "authorized",
            "--acceptance",
            "A1: the successor handoff distinguishes local archive from pending external validation",
            "--required-evidence",
            "verification",
        ],
    );
    common::plan(binary, repo.path(), predecessor);
    run_json(
        binary,
        repo.path(),
        &[
            "preflight",
            "--contract",
            &format!(".ai/work-items/active/{predecessor}.contract.json"),
        ],
    );
    run_json(binary, repo.path(), &["checkpoint", "--id", predecessor]);
    let controls_path = tempfile::NamedTempFile::new().expect("controls input");
    fs::write(
        controls_path.path(),
        serde_json::to_vec_pretty(&serde_json::json!({
            "acceptanceEvidence": [{
                "acceptanceId": "A1",
                "evidence": [{
                    "type": "test",
                    "path": "crates/cockpit-cli/tests/cli_mcp_outcome_parity.rs",
                    "locator": "successor_continuity_handoff_is_identical_across_cli_and_mcp_without_rewriting_archive",
                    "verification": "passed"
                }]
            }],
            "intentAlignment": {
                "state": "resolved",
                "evidence": ["crates/cockpit-cli/tests/cli_mcp_outcome_parity.rs"]
            }
        }))
        .expect("controls JSON"),
    )
    .expect("write controls");
    run_json(
        binary,
        repo.path(),
        &[
            "work-item",
            "controls",
            "--id",
            predecessor,
            "--input",
            controls_path.path().to_str().expect("controls path"),
        ],
    );
    run_json(
        binary,
        repo.path(),
        &["verify", "--work-item", predecessor, "--command", "true"],
    );
    run_json(binary, repo.path(), &["finish", "--id", predecessor]);
    run_json(binary, repo.path(), &["archive", "--id", predecessor]);

    let archive = repo.path().join(".ai/work-items/archive");
    let archived_bytes = [
        "contract.json",
        "summary.json",
        "outcome.json",
        "events.jsonl",
        "task-report.json",
        "task-report.md",
        "archive.json",
    ]
    .into_iter()
    .map(|suffix| {
        let path = archive.join(format!("{predecessor}.{suffix}"));
        (
            path.clone(),
            fs::read(path).expect("read frozen archive artifact"),
        )
    })
    .collect::<Vec<_>>();
    let archived_contract: serde_json::Value = serde_json::from_slice(
        &fs::read(archive.join(format!("{predecessor}.contract.json"))).expect("archived Contract"),
    )
    .expect("Contract JSON");
    let archived_summary: serde_json::Value = serde_json::from_slice(
        &fs::read(archive.join(format!("{predecessor}.summary.json"))).expect("archived Summary"),
    )
    .expect("Summary JSON");
    let archived_outcome: serde_json::Value = serde_json::from_slice(
        &fs::read(archive.join(format!("{predecessor}.outcome.json"))).expect("archived Outcome"),
    )
    .expect("Outcome JSON");
    let archived_events =
        fs::read(archive.join(format!("{predecessor}.events.jsonl"))).expect("archived Events");
    let runtime = current_runtime_context(binary);
    let recovery = serde_json::json!({
        "schemaVersion": 1,
        "decisionId": "work-item-recovery",
        "decision": "successor",
        "workItemId": predecessor,
        "repositoryId": archived_contract["repositoryId"],
        "predecessorWorkItemId": predecessor,
        "predecessorContractDigest": cockpit_protocol::digest_json(&archived_contract).unwrap(),
        "predecessorSummaryDigest": cockpit_protocol::digest_json(&archived_summary).unwrap(),
        "predecessorOutcomeDigest": cockpit_protocol::digest_json(&archived_outcome).unwrap(),
        "predecessorEventsDigest": format!("sha256:{}", hex::encode(Sha256::digest(&archived_events))),
        "successorWorkItemId": successor,
        "runtimeVersion": runtime.runtime_version,
        "runtimeDigest": runtime.runtime_digest,
        "actor": "human:test-owner",
        "authoritySource": "test-fixture",
        "reason": "continue the delivery after preserving its verified local archive",
        "evidenceRefs": [format!(".ai/work-items/archive/{predecessor}.outcome.json")],
        "policyRefs": ["docs/reference/agent-workflow.md"],
        "decidedAt": "2026-10-26T00:00:00Z",
        "resumeCondition": "continue and verify the successor Work Item"
    });
    let recovery_path = tempfile::NamedTempFile::new().expect("recovery input");
    fs::write(
        recovery_path.path(),
        serde_json::to_vec_pretty(&recovery).expect("recovery JSON"),
    )
    .expect("write recovery input");
    run_json(
        binary,
        repo.path(),
        &[
            "work-item",
            "recover",
            "--id",
            predecessor,
            "--input",
            recovery_path.path().to_str().expect("recovery path"),
        ],
    );
    run_json(
        binary,
        repo.path(),
        &[
            "start",
            "--id",
            successor,
            "--intent",
            "repair the externally failed delivery without rewriting history",
            "--goal",
            "preserve predecessor history and make the current next action clear",
            "--scope",
            "README.md",
            "--authority",
            "authorized",
            "--acceptance",
            "A1: successor lineage is shown without claiming provider delivery",
            "--required-evidence",
            "verification",
        ],
    );

    let runtime = current_runtime_context(binary);
    for (language, continuity_heading, pending_fragment) in [
        (
            "en",
            "Delivery continuity v1",
            "External validation: pending",
        ),
        ("zh", "交付连续性 v1", "外部验证：待完成"),
        ("ja", "引き継ぎ継続性 v1", "外部検証：保留中"),
    ] {
        let cli_handoff = human_cli_output(binary, repo.path(), successor, language);
        let response = cockpit_mcp::handle_request_for_repo(
            &serde_json::json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {"name": "work_item_outcome", "arguments": {"workItemId": successor, "language": language}}
            }),
            repo.path(),
            &runtime,
        );
        assert_eq!(response["result"]["isError"], false, "{response}");
        let mcp_handoff = response["result"]["structuredContent"]["humanHandoff"]
            .as_str()
            .expect("MCP human handoff");
        assert_eq!(
            continuity_section(&cli_handoff, continuity_heading),
            continuity_section(mcp_handoff, continuity_heading),
            "language={language}: CLI and MCP must render the same continuity facts"
        );
        assert!(cli_handoff.contains(continuity_heading), "{cli_handoff}");
        assert!(cli_handoff.contains(predecessor), "{cli_handoff}");
        assert!(cli_handoff.contains(pending_fragment), "{cli_handoff}");
    }
    for (path, bytes) in archived_bytes {
        assert_eq!(fs::read(&path).expect("read preserved archive"), bytes);
    }
}

fn replaced_continuity_fixture() -> (tempfile::TempDir, cockpit_protocol::RuntimeContext) {
    let repo = repository();
    let root = repo
        .path()
        .canonicalize()
        .expect("canonical continuity fixture");
    let runtime = current_runtime_context(env!("CARGO_BIN_EXE_ai-cockpit"));
    let predecessor = "WI-CONTINUITY-REPLACED";
    let successor = "WI-CONTINUITY-ACTIVE";
    cockpit_repository::attach(&root).expect("attach continuity fixture");
    let options = cockpit_repository::WorkItemStartOptions {
        authority: "authorized".into(),
        acceptance_criteria: vec!["retirement is not verification or provider delivery".into()],
        ..Default::default()
    };
    cockpit_repository::start_work_item_with_options_and_runtime(
        &root,
        predecessor,
        "replace an unverified predecessor",
        "preserve original lineage facts",
        &["README.md".into()],
        &options,
        &runtime,
    )
    .expect("start genuine predecessor");
    // Seed only a historical copy of an actual, unverified Runtime Outcome.
    // Retirement must preserve it through its native artifact manifest too.
    let unverified =
        cockpit_repository::outcome_v2_with_runtime(&root, predecessor, &runtime).unwrap();
    assert_ne!(unverified.state, cockpit_protocol::OutcomeState::Verified);
    fs::write(
        root.join(format!(
            ".ai/work-items/active/{predecessor}.outcome.snapshot.json"
        )),
        serde_json::to_vec_pretty(&unverified).unwrap(),
    )
    .unwrap();
    let read = |suffix: &str| -> serde_json::Value {
        serde_json::from_slice(
            &fs::read(root.join(format!(".ai/work-items/active/{predecessor}.{suffix}.json")))
                .unwrap(),
        )
        .unwrap()
    };
    let recovery = serde_json::json!({
        "schemaVersion": 1, "decisionId": "work-item-recovery", "decision": "successor",
        "workItemId": predecessor, "repositoryId": cockpit_repository::repository_id(&root),
        "predecessorWorkItemId": predecessor,
        "predecessorContractDigest": cockpit_protocol::digest_json(&read("contract")).unwrap(),
        "predecessorSummaryDigest": cockpit_protocol::digest_json(&read("summary")).unwrap(),
        "successorWorkItemId": successor,
        "runtimeVersion": runtime.runtime_version, "runtimeDigest": runtime.runtime_digest,
        "actor": "human:synthetic-fixture-owner", "authoritySource": "isolated-native-test-fixture",
        "reason": "replace without claiming a verified predecessor",
        "evidenceRefs": [], "policyRefs": [], "decidedAt": "2026-10-26T00:00:00Z",
        "resumeCondition": "activate the exact native successor"
    });
    cockpit_repository::record_recovery_decision(&root, predecessor, &recovery, &runtime)
        .expect("record genuine successor recovery");
    let retirement = cockpit_repository::retire_active_work_item_with_runtime(
        &root,
        predecessor,
        &cockpit_protocol::ActiveWorkItemRetirementRequest {
            schema_version: 1,
            decision_id: "work-item-retirement".into(),
            disposition: "replaced".into(),
            successor_work_item_id: Some(successor.into()),
            actor: "human:synthetic-fixture-owner".into(),
            authority_source: "isolated-native-test-fixture".into(),
            reason: "retain unverified original records".into(),
            repository_id: None,
            contract_digest: None,
            summary_digest: None,
            repository_snapshot_digest: None,
        },
        &runtime,
    )
    .expect("native replaced retirement");
    assert_eq!(retirement["disposition"], "replaced");
    assert_eq!(retirement["verificationClaim"], "not_verified");
    cockpit_repository::start_work_item_with_options_and_runtime(
        &root,
        successor,
        "continue the actual successor",
        "show replaced and active facts without completion",
        &["README.md".into()],
        &options,
        &runtime,
    )
    .expect("activate exact native successor after retirement");
    (repo, runtime)
}

#[test]
fn continuity_projection_marks_replaced_predecessor_and_active_successor() {
    let (repo, runtime) = replaced_continuity_fixture();
    let root = repo.path().canonicalize().unwrap();
    let predecessor = "WI-CONTINUITY-REPLACED";
    let successor = "WI-CONTINUITY-ACTIVE";
    let retirement_path = root.join(format!(".ai/decisions/{predecessor}.retirement.json"));
    let retirement_bytes = fs::read(&retirement_path).expect("native retirement receipt");
    let predecessor_outcome =
        cockpit_repository::outcome_v2_with_runtime(&root, predecessor, &runtime)
            .expect("native predecessor outcome");
    assert_eq!(
        predecessor_outcome.historical_status.as_deref(),
        Some("replaced")
    );
    let input = cockpit_repository::outcome_render_input_with_runtime(&root, successor, &runtime)
        .expect("actual successor continuity reader");
    let continuity = input
        .delivery_continuity
        .as_ref()
        .expect("successor continuity projection");
    assert_ne!(
        continuity.state, "unknown",
        "validated native replaced/not_verified lineage must remain distinguishable: {continuity:?}"
    );
    assert_eq!(
        continuity.predecessor_work_item_id.as_deref(),
        Some(predecessor)
    );
    assert_eq!(continuity.external_validation_state, "pending");
    assert_eq!(
        continuity.predecessor_archive_state,
        "replaced_not_verified"
    );
    let status =
        cockpit_repository::work_item_status_snapshot_with_runtime(&root, successor, &runtime)
            .unwrap();
    let next = continuity
        .runtime_admitted_next_action
        .as_ref()
        .expect("current Runtime next action");
    assert!(status.safe_actions.contains(next));
    let human = human_cli_output(env!("CARGO_BIN_EXE_ai-cockpit"), &root, successor, "en");
    assert!(human.contains(predecessor));
    assert!(human.contains("External validation: pending"));
    assert!(human.contains("not_verified"));
    assert!(!human.contains("is verified and preserved"));
    for (language, heading) in [
        ("en", "Delivery continuity v1"),
        ("zh", "交付连续性 v1"),
        ("ja", "引き継ぎ継続性 v1"),
    ] {
        let cli = human_cli_output(env!("CARGO_BIN_EXE_ai-cockpit"), &root, successor, language);
        let mcp = cockpit_mcp::handle_request_for_repo(
            &serde_json::json!({
                "jsonrpc":"2.0","id":51,"method":"tools/call",
                "params":{"name":"work_item_outcome","arguments":{"workItemId":successor,"language":language}}
            }),
            &root,
            &runtime,
        );
        assert_eq!(mcp["result"]["isError"], false, "{mcp}");
        let mcp_human = mcp["result"]["structuredContent"]["humanHandoff"]
            .as_str()
            .unwrap();
        assert_eq!(
            continuity_section(&cli, heading),
            continuity_section(mcp_human, heading),
            "{language}"
        );
        assert!(continuity_section(&cli, heading).contains("not_verified"));
    }
    assert_eq!(fs::read(retirement_path).unwrap(), retirement_bytes);
}

type ContinuityFixtureRecord = (std::path::PathBuf, Vec<u8>, fs::Permissions);

fn continuity_fixture_records(root: &Path) -> Vec<ContinuityFixtureRecord> {
    let mut records = Vec::new();
    for directory in [
        ".ai/work-items/active",
        ".ai/work-items/archive",
        ".ai/decisions",
    ] {
        for entry in fs::read_dir(root.join(directory)).unwrap() {
            let path = entry.unwrap().path();
            if path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("WI-CONTINUITY-")
            {
                records.push((
                    path.clone(),
                    fs::read(&path).unwrap(),
                    fs::metadata(path).unwrap().permissions(),
                ));
            }
        }
    }
    records.sort_by(|a, b| a.0.cmp(&b.0));
    records
}

fn restore_continuity_fixture(records: &[ContinuityFixtureRecord]) {
    for (path, bytes, permissions) in records {
        fs::write(path, bytes).unwrap();
        fs::set_permissions(path, permissions.clone()).unwrap();
        assert_eq!(fs::read(path).unwrap(), *bytes);
        assert_eq!(
            fs::metadata(path).unwrap().permissions().readonly(),
            permissions.readonly()
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode(),
                permissions.mode()
            );
        }
    }
}

fn assert_replaced_continuity(root: &Path, runtime: &cockpit_protocol::RuntimeContext) {
    let input = cockpit_repository::outcome_render_input_with_runtime(
        root,
        "WI-CONTINUITY-ACTIVE",
        runtime,
    )
    .unwrap();
    let continuity = input.delivery_continuity.unwrap();
    assert_eq!(continuity.state, "in_progress");
    assert_eq!(
        continuity.predecessor_archive_state,
        "replaced_not_verified"
    );
    assert_eq!(continuity.external_validation_state, "pending");
}

fn assert_untrusted_continuity(
    root: &Path,
    runtime: &cockpit_protocol::RuntimeContext,
    case: &str,
    own_retirement_check: bool,
) {
    match cockpit_repository::outcome_render_input_with_runtime(
        root,
        "WI-CONTINUITY-ACTIVE",
        runtime,
    ) {
        Ok(input) => {
            let continuity = input
                .delivery_continuity
                .expect("invalid lineage remains explicitly untrusted");
            assert_eq!(continuity.state, "unknown", "{case}: {continuity:?}");
            assert!(
                continuity.runtime_admitted_next_action.is_none(),
                "{case}: no action from invalid continuity"
            );
            if own_retirement_check {
                assert_eq!(
                    continuity.reason.as_deref(),
                    Some("predecessor_retirement_unverified"),
                    "{case}"
                );
            }
            eprintln!(
                "continuity negative rejected: {case}: {:?}",
                continuity.reason
            );
        }
        Err(error) => {
            assert!(
                !own_retirement_check,
                "{case}: must exercise the retirement checker: {error}"
            );
            let text = error.to_string().to_ascii_lowercase();
            assert!(
                [
                    "contract",
                    "recovery",
                    "lineage",
                    "archive",
                    "repository",
                    "outcome",
                    "summary",
                    "receipt",
                    "retirement"
                ]
                .iter()
                .any(|part| text.contains(part)),
                "{case}: unrelated failure: {error}"
            );
            eprintln!("continuity native refusal: {case}: {error}");
        }
    }
}

#[test]
fn continuity_projection_rejects_missing_foreign_or_inconsistent_lineage() {
    let (repo, runtime) = replaced_continuity_fixture();
    let root = repo.path().canonicalize().unwrap();
    let baseline = continuity_fixture_records(&root);
    assert_replaced_continuity(&root, &runtime);
    let files = [
        (
            "successor",
            ".ai/work-items/active/WI-CONTINUITY-ACTIVE.contract.json",
            "repositoryId",
        ),
        (
            "predecessor",
            ".ai/work-items/archive/WI-CONTINUITY-REPLACED.contract.json",
            "repositoryId",
        ),
        (
            "summary",
            ".ai/work-items/archive/WI-CONTINUITY-REPLACED.summary.json",
            "repositoryId",
        ),
        (
            "outcome",
            ".ai/work-items/archive/WI-CONTINUITY-REPLACED.outcome.json",
            "repositoryId",
        ),
        (
            "historical-outcome",
            ".ai/work-items/archive/WI-CONTINUITY-REPLACED.outcome.snapshot.json",
            "repositoryId",
        ),
        (
            "recovery",
            ".ai/decisions/WI-CONTINUITY-REPLACED.recovery.json",
            "repositoryId",
        ),
        (
            "manifest",
            ".ai/work-items/archive/WI-CONTINUITY-REPLACED.archive.json",
            "workItemId",
        ),
        (
            "retirement",
            ".ai/decisions/WI-CONTINUITY-REPLACED.retirement.json",
            "repositoryId",
        ),
    ];
    for (label, relative, field) in files {
        for variant in ["missing", "malformed", "foreign"] {
            let path = root.join(relative);
            match variant {
                "missing" => fs::remove_file(&path).unwrap(),
                "malformed" => fs::write(&path, b"{invalid-json").unwrap(),
                "foreign" => {
                    let mut value: serde_json::Value =
                        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                    value[field] = serde_json::json!(if field == "workItemId" {
                        "WI-FOREIGN"
                    } else {
                        "sha256:1111111111111111111111111111111111111111111111111111111111111111"
                    });
                    fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
                }
                _ => unreachable!(),
            }
            assert_untrusted_continuity(&root, &runtime, &format!("{label}-{variant}"), false);
            restore_continuity_fixture(&baseline);
            assert_replaced_continuity(&root, &runtime);
        }
    }
    let receipt_path = root.join(".ai/decisions/WI-CONTINUITY-REPLACED.retirement.json");
    let manifest_path = root.join(".ai/work-items/archive/WI-CONTINUITY-REPLACED.archive.json");
    for (field, value, own_check) in [
        (
            "archiveManifestDigest",
            serde_json::json!(format!("sha256:{}", "2".repeat(64))),
            true,
        ),
        ("verificationClaim", serde_json::json!("verified"), false),
        ("originalBytesPreserved", serde_json::json!(false), true),
        (
            "successorWorkItemId",
            serde_json::json!("WI-FOREIGN-SUCCESSOR"),
            false,
        ),
        ("schemaVersion", serde_json::json!(2), true),
        ("runtimeDigest", serde_json::json!("malformed"), true),
        (
            "repositorySnapshotDigest",
            serde_json::json!("malformed"),
            true,
        ),
        ("recordedAt", serde_json::json!("not-a-time"), true),
    ] {
        let mut receipt: serde_json::Value =
            serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
        receipt[field] = value;
        fs::write(&receipt_path, serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
        assert_untrusted_continuity(&root, &runtime, field, own_check);
        restore_continuity_fixture(&baseline);
        assert_replaced_continuity(&root, &runtime);
    }
    // Matching bytes and locally rehashed records cannot make a foreign path native.
    let foreign_relative = ".ai/evidence/external/foreign-contract.json";
    let foreign_path = root.join(foreign_relative);
    fs::create_dir_all(foreign_path.parent().unwrap()).unwrap();
    let contract_bytes =
        fs::read(root.join(".ai/work-items/archive/WI-CONTINUITY-REPLACED.contract.json")).unwrap();
    fs::write(&foreign_path, &contract_bytes).unwrap();
    let mut receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    receipt["artifacts"]["contract"]["path"] = serde_json::json!(foreign_relative);
    manifest["files"]["contractPath"] = receipt["artifacts"]["contract"]["path"].clone();
    receipt["archiveManifestDigest"] =
        serde_json::json!(cockpit_protocol::digest_json(&manifest).unwrap());
    fs::write(&receipt_path, serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    assert_untrusted_continuity(&root, &runtime, "locally-resealed-foreign-path", true);
    assert_eq!(fs::read(&foreign_path).unwrap(), contract_bytes);
    restore_continuity_fixture(&baseline);
    assert_replaced_continuity(&root, &runtime);
    let receipt_bytes = fs::read(&receipt_path).unwrap();
    let mut duplicate = receipt_bytes.clone();
    let closing = duplicate.iter().rposition(|b| *b == b'}').unwrap();
    duplicate.splice(closing..closing, b",\"schemaVersion\":1".iter().copied());
    fs::write(&receipt_path, duplicate).unwrap();
    assert_untrusted_continuity(&root, &runtime, "duplicate-retirement-field", false);
    restore_continuity_fixture(&baseline);
    assert_replaced_continuity(&root, &runtime);
    let mut oversized = receipt_bytes;
    oversized.resize(cockpit_git::MAX_BOUNDED_GIT_OUTPUT_BYTES + 1, b' ');
    fs::write(&receipt_path, oversized).unwrap();
    assert_untrusted_continuity(&root, &runtime, "over-budget-retirement-record", false);
    restore_continuity_fixture(&baseline);
    assert_replaced_continuity(&root, &runtime);
}

#[cfg(unix)]
#[test]
fn continuity_projection_rejects_fifo_retirement_artifact_without_waiting_for_peer() {
    use std::io::Read;
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    const CHILD_ROOT: &str = "WI1113_CONTINUITY_FIFO_CHILD_ROOT";
    const CHILD_RUNTIME: &str = "WI1113_CONTINUITY_FIFO_CHILD_RUNTIME";
    if let Some(root) = std::env::var_os(CHILD_ROOT) {
        let runtime = serde_json::from_str(&std::env::var(CHILD_RUNTIME).unwrap()).unwrap();
        assert_untrusted_continuity(Path::new(&root), &runtime, "historical-artifact-fifo", true);
        return;
    }

    let (repo, runtime) = replaced_continuity_fixture();
    let root = repo.path().canonicalize().unwrap();
    let baseline = continuity_fixture_records(&root);
    assert_replaced_continuity(&root, &runtime);
    let artifact = root.join(".ai/work-items/archive/WI-CONTINUITY-REPLACED.outcome.snapshot.json");
    fs::remove_file(&artifact).unwrap();
    assert!(
        Command::new("mkfifo")
            .arg(&artifact)
            .status()
            .unwrap()
            .success()
    );

    struct OwnedProbe(std::process::Child);
    impl Drop for OwnedProbe {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let mut log = tempfile::tempfile().unwrap();
    let mut child = OwnedProbe(
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "continuity_projection_rejects_fifo_retirement_artifact_without_waiting_for_peer",
                "--test-threads=1",
                "--nocapture",
            ])
            .env(CHILD_ROOT, &root)
            .env(CHILD_RUNTIME, serde_json::to_string(&runtime).unwrap())
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log.try_clone().unwrap()))
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break Some(status);
        }
        if Instant::now() >= deadline {
            child.0.kill().unwrap();
            child.0.wait().unwrap();
            break None;
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    use std::io::Seek;
    log.rewind().unwrap();
    let mut output = String::new();
    log.take(65536).read_to_string(&mut output).unwrap();
    // Remove the FIFO before restoring bytes: writing through it would itself wait.
    fs::remove_file(&artifact).unwrap();
    restore_continuity_fixture(&baseline);
    assert_replaced_continuity(&root, &runtime);
    assert!(
        status.is_some(),
        "retirement reader waited for a FIFO peer; owned probe killed and reaped: {output}"
    );
    assert!(
        status.unwrap().success(),
        "FIFO must remain untrusted: {output}"
    );
}

#[test]
fn continuity_projection_keeps_archive_and_provider_delivery_state_separate() {
    for (case, validity) in [
        ("missing", None),
        ("stale", Some(cockpit_protocol::EvidenceValidity::Expired)),
        ("failed", Some(cockpit_protocol::EvidenceValidity::Valid)),
        ("pending", Some(cockpit_protocol::EvidenceValidity::Unknown)),
    ] {
        let (repo, runtime) = replaced_continuity_fixture();
        let root = repo.path().canonicalize().unwrap();
        let baseline = continuity_fixture_records(&root);
        if let Some(validity) = validity {
            // These are explicitly self-declared negative provider fixtures,
            // never authenticated GitHub results or a fabricated success.
            let raw = serde_json::to_vec(&serde_json::json!({"status":case,"conclusion":if case == "failed" { Some("failure") } else { None::<&str> }})).unwrap();
            let evidence = cockpit_protocol::DelegatedEvidence {
                provider: "github".into(),
                subject: format!("https://github.com/example/fixture/actions/runs/{case}"),
                origin: "isolated-negative-provider-fixture".into(),
                assurance: cockpit_protocol::AssuranceLevel::SelfDeclared,
                collected_at: "2026-10-28T00:00:00Z".into(),
                digest: cockpit_core::Digest::sha256_bytes(&raw),
                validity: validity.clone(),
                raw_evidence_ref: format!(".ai/evidence/external/continuity-{case}.json"),
            };
            let observation = cockpit_repository::import_delegated_evidence_observation(
                &root,
                "WI-CONTINUITY-ACTIVE",
                &evidence,
                &raw,
                &runtime,
            )
            .expect("native binding of negative provider fixture bytes");
            assert_eq!(observation.receipt.evidence.validity, validity);
            let observations = cockpit_repository::list_delegated_evidence_observations(
                &root,
                "WI-CONTINUITY-ACTIVE",
            )
            .unwrap();
            assert_eq!(observations.len(), 1);
            assert_eq!(observations[0].selector, observation.selector);
        } else {
            assert!(
                cockpit_repository::list_delegated_evidence_observations(
                    &root,
                    "WI-CONTINUITY-ACTIVE"
                )
                .unwrap()
                .is_empty()
            );
        }
        let input = cockpit_repository::outcome_render_input_with_runtime(
            &root,
            "WI-CONTINUITY-ACTIVE",
            &runtime,
        )
        .unwrap();
        let continuity = input.delivery_continuity.as_ref().unwrap();
        assert_eq!(continuity.state, "in_progress", "{case}");
        assert_eq!(
            continuity.predecessor_archive_state, "replaced_not_verified",
            "{case}"
        );
        assert!(
            matches!(
                continuity.external_validation_state.as_str(),
                "pending" | "unknown"
            ),
            "{case}: no negative provider claim can prove delivery"
        );
        assert_eq!(continuity.external_binding_state, "not_bound");
        assert_ne!(
            input.outcome.state,
            cockpit_protocol::OutcomeState::Verified
        );
        let human = human_cli_output(
            env!("CARGO_BIN_EXE_ai-cockpit"),
            &root,
            "WI-CONTINUITY-ACTIVE",
            "en",
        );
        assert!(
            human.contains("Overall delivery: in progress"),
            "{case}: {human}"
        );
        for (path, bytes, _) in &baseline {
            assert_eq!(
                fs::read(path).unwrap(),
                *bytes,
                "{case}: native predecessor/successor records are read-only"
            );
        }
        eprintln!("negative provider observation remains pending: {case}");
    }
}

#[test]
fn cli_and_mcp_interface_descriptions_share_the_protocol_facts() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let repo = repository();
    let cli_output = Command::new(binary)
        .args(["capability", "show", "--repo"])
        .arg(repo.path())
        .args(["--surface", "work-item-outcome", "--format", "json"])
        .output()
        .expect("cli interface description");
    assert!(
        cli_output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&cli_output.stderr)
    );
    let cli_description: serde_json::Value =
        serde_json::from_slice(&cli_output.stdout).expect("CLI description JSON");

    let response = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 18,
            "method": "tools/call",
            "params": {
                "name": "capability_show",
                "arguments": {"surface": "work-item-outcome", "format": "json"}
            }
        }),
        repo.path(),
        &current_runtime_context(binary),
    );
    assert_eq!(response["result"]["isError"], false);
    assert_eq!(
        cli_description, response["result"]["structuredContent"],
        "CLI and MCP must project the same protocol-owned interface facts"
    );
}

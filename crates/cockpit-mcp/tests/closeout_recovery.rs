use cockpit_core::Digest;
use cockpit_mcp::handle_request_for_repo;
use cockpit_protocol::{HumanDecision, PROTOCOL_VERSION, RuntimeContext};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_work_item_with_runtime, attach, checkpoint_work_item,
    close_work_item_with_structured_decision_and_runtime, finish_work_item_with_runtime,
    record_verification_with_runtime, repository_id, run_repository_verification,
    start_work_item_with_options,
};
use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const WORK_ITEM_ID: &str = "WI-MCP-CROSS-CHECKOUT-CLOSEOUT";

struct TestTempDir(std::path::PathBuf);

impl TestTempDir {
    fn new(prefix: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}-{}-{nonce}", std::process::id()));
        fs::create_dir(&path).expect("create temp directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestTempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "1.0.0-mcp-closeout-test".into(),
        protocol_version: PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(b"mcp-closeout-recovery-test-runtime"),
    }
}

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn repository() -> TestTempDir {
    let directory = TestTempDir::new("cockpit-mcp-closeout-source");
    git(directory.path(), &["init", "--quiet"]);
    attach(directory.path()).expect("attach repository");
    git(
        directory.path(),
        &["config", "user.name", "MCP Closeout Test"],
    );
    git(
        directory.path(),
        &["config", "user.email", "mcp-closeout-test@example.invalid"],
    );
    fs::write(directory.path().join("seed.txt"), "baseline\n").expect("write seed");
    git(directory.path(), &["add", "-A"]);
    git(directory.path(), &["commit", "--quiet", "-m", "baseline"]);
    directory
}

fn build_closed_source(source: &Path) {
    let runtime = runtime();
    start_work_item_with_options(
        source,
        WORK_ITEM_ID,
        "recover a verified closeout through MCP",
        "expose read-only planning and explicit recovery over MCP",
        &[".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["the closeout is recoverable without mutating source".into()],
            ..Default::default()
        },
    )
    .expect("start source Work Item");
    let contract = source.join(format!(
        ".ai/work-items/active/{WORK_ITEM_ID}.contract.json"
    ));
    cockpit_repository::preflight_work_item(source, &contract).expect("preflight");
    checkpoint_work_item(source, WORK_ITEM_ID).expect("checkpoint");
    let request = RepositoryVerificationRequest {
        node_id: "mcp-closeout-source-verification".into(),
        program: "true".into(),
        args: Vec::new(),
        scope: vec![".ai/**".into()],
        stage: "task".into(),
        runner: "local".into(),
        runtime_digest: runtime.runtime_digest.to_string(),
        base_commit: None,
        workers: 1,
        work_item_id: Some(WORK_ITEM_ID.into()),
        timeout_seconds: None,
        policy: RepositoryVerificationPolicy::NeverReuse,
    };
    let execution = run_repository_verification(source, &request).expect("verify source");
    record_verification_with_runtime(
        source,
        WORK_ITEM_ID,
        &serde_json::to_value(execution.receipt).expect("verification receipt JSON"),
        &runtime,
        &execution.final_snapshot,
    )
    .expect("record source verification");
    finish_work_item_with_runtime(source, WORK_ITEM_ID, &runtime).expect("finish source");
    archive_work_item_with_runtime(source, WORK_ITEM_ID, &runtime).expect("archive source");
    git(source, &["add", "-A"]);
    git(
        source,
        &["commit", "--quiet", "-m", "archive verified Work Item"],
    );
}

fn call(destination: &Path, source: &Path, name: &str, request_id: u64) -> Value {
    handle_request_for_repo(
        &serde_json::json!({
            "jsonrpc":"2.0",
            "id":request_id,
            "method":"tools/call",
            "params":{
                "name":name,
                "arguments":{
                    "workItemId":WORK_ITEM_ID,
                    "sourceRepo":source.display().to_string()
                }
            }
        }),
        destination,
        &runtime(),
    )
}

#[test]
fn mcp_exposes_read_only_plan_and_explicit_cross_checkout_recovery() {
    let tools = handle_request_for_repo(
        &serde_json::json!({
            "jsonrpc":"2.0",
            "id":1,
            "method":"tools/list",
            "params":{}
        }),
        Path::new("."),
        &runtime(),
    );
    let listed = tools["result"]["tools"].as_array().expect("tools list");
    let plan_schema = listed
        .iter()
        .find(|tool| tool["name"] == "work_item_closeout_recovery_plan")
        .expect("read-only closeout plan tool");
    let recover_schema = listed
        .iter()
        .find(|tool| tool["name"] == "work_item_closeout_recover")
        .expect("explicit closeout recovery tool");
    for schema in [plan_schema, recover_schema] {
        assert_eq!(schema["inputSchema"]["additionalProperties"], false);
        assert_eq!(
            schema["inputSchema"]["properties"]["workItemId"]["type"],
            "string"
        );
        assert_eq!(
            schema["inputSchema"]["properties"]["sourceRepo"]["type"],
            "string"
        );
        assert_eq!(
            schema["inputSchema"]["required"],
            serde_json::json!(["workItemId", "sourceRepo"])
        );
    }

    let source = repository();
    build_closed_source(source.path());
    let destination_parent = TestTempDir::new("cockpit-mcp-closeout-destination");
    let destination = destination_parent.path().join("clone");
    git(
        source.path(),
        &[
            "clone",
            "--quiet",
            "--no-hardlinks",
            source.path().to_str().expect("source path"),
            destination.to_str().expect("destination path"),
        ],
    );
    assert_eq!(repository_id(source.path()), repository_id(&destination));
    close_work_item_with_structured_decision_and_runtime(
        source.path(),
        WORK_ITEM_ID,
        &HumanDecision {
            decision: "approved".into(),
            actor: "test-human".into(),
            authority_source: "MCP closeout integration test".into(),
            reason: "the source Work Item has verified archived evidence".into(),
            evidence_refs: vec![format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")],
            policy_refs: Vec::new(),
            decided_at: "2026-10-02T00:00:00Z".into(),
            resume_condition: None,
        },
        &runtime(),
    )
    .expect("close source Work Item");
    let source_close_path = source
        .path()
        .join(".ai/decisions")
        .join(format!("{WORK_ITEM_ID}.close.json"));
    let source_close_before = fs::read(&source_close_path).expect("source close bytes");
    let source_status_before = {
        let output = Command::new("git")
            .args(["status", "--porcelain=v1", "--untracked-files=all"])
            .current_dir(source.path())
            .output()
            .expect("source status");
        assert!(output.status.success());
        output.stdout
    };

    let plan = call(
        &destination,
        source.path(),
        "work_item_closeout_recovery_plan",
        2,
    );
    assert_eq!(plan["result"]["isError"], false, "{plan}");
    let plan = &plan["result"]["structuredContent"];
    assert_eq!(plan["allowed"], true, "{plan}");
    assert_eq!(plan["workItemId"], WORK_ITEM_ID);
    assert_eq!(plan["providerFacts"], Value::Null);

    let recovery = call(&destination, source.path(), "work_item_closeout_recover", 3);
    assert_eq!(recovery["result"]["isError"], false, "{recovery}");
    let recovery = &recovery["result"]["structuredContent"];
    assert_eq!(recovery["state"], "recovered");
    assert_eq!(recovery["workItemId"], WORK_ITEM_ID);
    assert!(
        destination
            .join(".ai/decisions")
            .join(format!("{WORK_ITEM_ID}.close.json"))
            .is_file(),
        "the explicit MCP recovery call installs the exact close decision"
    );
    assert_eq!(
        fs::read(source_close_path).expect("source close after"),
        source_close_before
    );
    let source_status_after = {
        let output = Command::new("git")
            .args(["status", "--porcelain=v1", "--untracked-files=all"])
            .current_dir(source.path())
            .output()
            .expect("source status after recovery");
        assert!(output.status.success());
        output.stdout
    };
    assert_eq!(source_status_after, source_status_before);
}

use cockpit_core::Digest;
use cockpit_protocol::{
    CONTRACT_AMENDMENT_SCHEMA_VERSION, ContractAmendmentChange, ContractAmendmentOperation,
    ContractAmendmentRequest, PROTOCOL_VERSION, RuntimeContext,
};
use cockpit_repository::{
    WorkItemStartOptions, attach, record_recovery_decision,
    record_work_item_governance_controls_with_runtime, repository_id, start_work_item_with_options,
};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const WORK_ITEM_ID: &str = "WI-CLI-AMEND-PARITY";

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("git command");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn repository() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temporary repository");
    git(root.path(), &["init", "-q", "--initial-branch=main"]);
    git(
        root.path(),
        &["config", "user.email", "acceptance@example.invalid"],
    );
    git(root.path(), &["config", "user.name", "Acceptance"]);
    fs::write(root.path().join("README.md"), "amendment acceptance\n").expect("README");
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-qm", "initial"]);
    attach(root.path()).expect("attach repository");
    start_work_item_with_options(
        root.path(),
        WORK_ITEM_ID,
        "exercise typed Contract amendment adapters",
        "record a plan change identically through CLI and MCP",
        &["README.md".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["both adapters return one immutable receipt".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start Work Item");
    root
}

fn candidate_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ai-cockpit"))
}

fn run_cli(args: &[&str]) -> Output {
    Command::new(candidate_binary())
        .args(args)
        .output()
        .expect("candidate CLI process")
}

fn assert_success(output: &Output, label: &str) {
    assert!(
        output.status.success(),
        "{label} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn candidate_runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        protocol_version: PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(
            &fs::read(candidate_binary()).expect("read candidate executable"),
        ),
    }
}

fn typed_request(root: &Path) -> ContractAmendmentRequest {
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{WORK_ITEM_ID}.contract.json"));
    let contract: Value =
        serde_json::from_slice(&fs::read(contract_path).expect("Contract")).expect("Contract JSON");
    ContractAmendmentRequest {
        schema_version: CONTRACT_AMENDMENT_SCHEMA_VERSION,
        change_id: "plan-change-001".into(),
        expected_contract_digest: cockpit_protocol::digest_json(&contract)
            .expect("Contract digest"),
        reason: "The implementation surfaced a more accurate acceptance plan.".into(),
        changes: vec![ContractAmendmentChange {
            path: "/goal".into(),
            operation: ContractAmendmentOperation::Set,
            value: Some(json!(
                "record and verify the implementation-plan adjustment"
            )),
        }],
    }
}

fn mcp_call(root: &Path, runtime: &RuntimeContext, name: &str, arguments: Value) -> Value {
    cockpit_mcp::handle_request_for_repo(
        &json!({
            "jsonrpc":"2.0",
            "id":1,
            "method":"tools/call",
            "params":{"name":name,"arguments":arguments}
        }),
        root,
        runtime,
    )
}

fn amendment_history_bytes(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let directory = root
        .join(".ai/evidence")
        .join(format!("{WORK_ITEM_ID}.contract-amendments"));
    if !directory.exists() {
        return Vec::new();
    }
    let mut records = fs::read_dir(directory)
        .expect("read amendment history")
        .map(|entry| {
            let path = entry.expect("history entry").path();
            let bytes = fs::read(&path).expect("history bytes");
            (path, bytes)
        })
        .collect::<Vec<_>>();
    records.sort_by(|left, right| left.0.cmp(&right.0));
    records
}

#[test]
fn amend_help_exposes_typed_request_and_preserves_legacy_adapter() {
    let output = run_cli(&["work-item", "amend", "--help"]);
    assert_success(&output, "work-item amend --help");
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(help.contains("--request"), "typed request missing: {help}");
    assert!(help.contains("--input"), "legacy input missing: {help}");
    assert!(help.contains("--reason"), "legacy reason missing: {help}");
}

#[test]
fn typed_cli_and_mcp_share_receipt_and_read_history_without_writes() {
    let root = repository();
    let request = typed_request(root.path());
    let request_value = serde_json::to_value(&request).expect("request JSON");
    let request_directory = tempfile::tempdir().expect("request directory");
    let request_path = request_directory.path().join("typed-amendment.json");
    fs::write(
        &request_path,
        serde_json::to_vec_pretty(&request_value).expect("serialize request"),
    )
    .expect("write request");

    let cli = run_cli(&[
        "work-item",
        "amend",
        "--repo",
        root.path().to_str().expect("repository path"),
        "--id",
        WORK_ITEM_ID,
        "--request",
        request_path.to_str().expect("request path"),
    ]);
    assert_success(&cli, "typed CLI amendment");
    let mut cli_receipt: Value = serde_json::from_slice(&cli.stdout).expect("CLI receipt");
    cli_receipt
        .as_object_mut()
        .expect("CLI receipt object")
        .remove("nextAction");

    let runtime = candidate_runtime();
    let mcp = mcp_call(
        root.path(),
        &runtime,
        "work_item_amend",
        json!({"workItemId":WORK_ITEM_ID,"request":request_value}),
    );
    assert_eq!(
        mcp.pointer("/result/isError"),
        Some(&Value::Bool(false)),
        "{mcp}"
    );
    assert_eq!(mcp.pointer("/result/structuredContent"), Some(&cli_receipt));

    let legacy_input_path = request_directory.path().join("legacy-additive.json");
    fs::write(
        &legacy_input_path,
        br#"{"scopeAppend":["src/legacy-adapter.rs"]}"#,
    )
    .expect("legacy additive input");
    let legacy = run_cli(&[
        "work-item",
        "amend",
        "--repo",
        root.path().to_str().expect("repository path"),
        "--id",
        WORK_ITEM_ID,
        "--input",
        legacy_input_path.to_str().expect("legacy input path"),
        "--reason",
        "Retain compatibility for existing additive callers.",
    ]);
    assert_success(&legacy, "legacy additive CLI amendment");

    let mut invalid_request = request_value.clone();
    invalid_request["unrecognized"] = json!(true);
    let invalid = mcp_call(
        root.path(),
        &runtime,
        "work_item_amend",
        json!({"workItemId":WORK_ITEM_ID,"request":invalid_request}),
    );
    assert_eq!(invalid.pointer("/result/isError"), Some(&Value::Bool(true)));
    assert!(
        invalid["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .contains("unknown field"),
        "{invalid}"
    );

    let stale = ContractAmendmentRequest {
        change_id: "plan-change-stale".into(),
        ..request.clone()
    };
    let stale_value = serde_json::to_value(&stale).expect("stale request JSON");
    let contract_path = root
        .path()
        .join(".ai/work-items/active")
        .join(format!("{WORK_ITEM_ID}.contract.json"));
    let before_conflict = fs::read(&contract_path).expect("Contract before conflict");
    let conflict = mcp_call(
        root.path(),
        &runtime,
        "work_item_amend",
        json!({"workItemId":WORK_ITEM_ID,"request":stale_value}),
    );
    assert_eq!(
        conflict.pointer("/result/isError"),
        Some(&Value::Bool(true))
    );
    assert!(
        conflict["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .contains("contract_digest_conflict"),
        "{conflict}"
    );
    assert_eq!(
        fs::read(&contract_path).expect("Contract after conflict"),
        before_conflict
    );

    let history_before = amendment_history_bytes(root.path());
    let cli_history = run_cli(&[
        "work-item",
        "amendments",
        "--repo",
        root.path().to_str().expect("repository path"),
        "--id",
        WORK_ITEM_ID,
    ]);
    assert_success(&cli_history, "read CLI amendment history");
    let cli_history: Value = serde_json::from_slice(&cli_history.stdout).expect("CLI history");
    assert_eq!(cli_history.as_array().map(Vec::len), Some(2));
    let mcp_history = mcp_call(
        root.path(),
        &runtime,
        "work_item_amendments",
        json!({"workItemId":WORK_ITEM_ID}),
    );
    assert_eq!(
        mcp_history.pointer("/result/structuredContent"),
        Some(&cli_history)
    );
    assert_eq!(amendment_history_bytes(root.path()), history_before);

    let legacy = RuntimeContext {
        runtime_version: "0.2.113".into(),
        protocol_version: PROTOCOL_VERSION,
        runtime_digest: "sha256:c85632062eb5ef8f8b39f6154b765c9e2543fe6b7ca3f54e107a59c174843686"
            .parse()
            .expect("installed predecessor digest"),
    };
    let unsupported = mcp_call(
        root.path(),
        &legacy,
        "work_item_environment_drift",
        json!({"action":"check","workItemId":WORK_ITEM_ID,"generation":1}),
    );
    assert_eq!(
        unsupported.pointer("/result/isError"),
        Some(&Value::Bool(true))
    );
    assert!(
        unsupported["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .contains("unsupported_runtime_capability"),
        "{unsupported}"
    );
}

#[test]
fn inspect_accepts_null_optional_concurrency_boundary_after_amendment() {
    let root = repository();
    let request = serde_json::to_vec_pretty(&typed_request(root.path()))
        .expect("serialize amendment request");
    let request_file = tempfile::NamedTempFile::new().expect("amendment request file");
    fs::write(request_file.path(), request).expect("write amendment request");

    let amended = run_cli(&[
        "work-item",
        "amend",
        "--repo",
        root.path().to_str().expect("repository path"),
        "--id",
        WORK_ITEM_ID,
        "--request",
        request_file.path().to_str().expect("request path"),
    ]);
    assert_success(&amended, "amend Work Item before inspect");

    let contract_path = root
        .path()
        .join(".ai/work-items/active")
        .join(format!("{WORK_ITEM_ID}.contract.json"));
    let contract: Value = serde_json::from_slice(&fs::read(contract_path).expect("read Contract"))
        .expect("Contract JSON");
    assert_eq!(contract["concurrencyBoundary"], Value::Null);

    let inspected = run_cli(&[
        "work-item",
        "inspect",
        "--repo",
        root.path().to_str().expect("repository path"),
        "--id",
        WORK_ITEM_ID,
    ]);
    assert_success(&inspected, "inspect amended Contract with null boundary");
}

fn assert_sensitive_amendment_blocks_cli_verification_before_command_spawn(retry_pending: bool) {
    let root = repository();
    let repository_path = root.path().to_str().expect("repository path");
    let contract_relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");

    let initial_preflight = run_cli(&[
        "preflight",
        "--repo",
        repository_path,
        "--contract",
        &contract_relative,
    ]);
    assert_success(&initial_preflight, "initial preflight");
    let checkpoint = run_cli(&[
        "checkpoint",
        "--repo",
        repository_path,
        "--id",
        WORK_ITEM_ID,
    ]);
    assert_success(&checkpoint, "checkpoint before amendment");

    let contract_path = root.path().join(&contract_relative);
    if retry_pending {
        let summary_path = root
            .path()
            .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
        let predecessor_contract: Value =
            serde_json::from_slice(&fs::read(&contract_path).expect("read Contract"))
                .expect("Contract JSON");
        let predecessor_summary: Value =
            serde_json::from_slice(&fs::read(&summary_path).expect("read Summary"))
                .expect("Summary JSON");
        let runtime = candidate_runtime();
        let retry = json!({
            "schemaVersion": 1,
            "decisionId": "work-item-recovery",
            "decision": "retry",
            "workItemId": WORK_ITEM_ID,
            "repositoryId": repository_id(root.path()).to_string(),
            "predecessorWorkItemId": WORK_ITEM_ID,
            "predecessorContractDigest": cockpit_protocol::digest_json(&predecessor_contract).expect("Contract digest"),
            "predecessorSummaryDigest": cockpit_protocol::digest_json(&predecessor_summary).expect("Summary digest"),
            "runtimeVersion": runtime.runtime_version,
            "runtimeDigest": runtime.runtime_digest,
            "actor": "human:fixture",
            "authoritySource": "test fixture",
            "reason": "retry a stale verification attempt",
            "decidedAt": "2026-10-03T00:00:00Z",
            "resumeCondition": "run replacement verification"
        });
        record_recovery_decision(root.path(), WORK_ITEM_ID, &retry, &runtime)
            .expect("bind retry decision through Runtime");
    }
    let contract: Value = serde_json::from_slice(&fs::read(&contract_path).expect("read Contract"))
        .expect("Contract JSON");
    let request = ContractAmendmentRequest {
        schema_version: CONTRACT_AMENDMENT_SCHEMA_VERSION,
        change_id: "sensitive-scope-change".into(),
        expected_contract_digest: cockpit_protocol::digest_json(&contract)
            .expect("Contract digest"),
        reason: "scope expansion requires human policy review before verification".into(),
        changes: vec![ContractAmendmentChange {
            path: "/scope".into(),
            operation: ContractAmendmentOperation::Replace,
            value: Some(json!(["README.md", "src/**"])),
        }],
    };
    let request_file = tempfile::NamedTempFile::new().expect("amendment request file");
    fs::write(
        request_file.path(),
        serde_json::to_vec_pretty(&request).expect("serialize amendment request"),
    )
    .expect("write amendment request");
    let amended = run_cli(&[
        "work-item",
        "amend",
        "--repo",
        repository_path,
        "--id",
        WORK_ITEM_ID,
        "--request",
        request_file.path().to_str().expect("request path"),
    ]);
    assert_success(&amended, "record sensitive amendment");

    if retry_pending {
        let status = run_cli(&[
            "work-item",
            "status",
            "--repo",
            repository_path,
            "--id",
            WORK_ITEM_ID,
            "--json",
        ]);
        assert_success(&status, "status before retry amendment review preflight");
        let status: Value = serde_json::from_slice(&status.stdout).expect("status JSON");
        assert!(
            status["safeActions"]
                .as_array()
                .expect("safe actions")
                .iter()
                .any(|action| action == "run_preflight"),
            "a pending retry and sensitive amendment must admit request-generating preflight: {status}"
        );
    }

    let preflight = run_cli(&[
        "preflight",
        "--repo",
        repository_path,
        "--contract",
        &contract_relative,
    ]);
    assert_success(
        &preflight,
        "Runtime-bound preflight should produce a pending amendment review request",
    );
    let preflight: Value = serde_json::from_slice(&preflight.stdout).expect("preflight JSON");
    let current_contract: Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("read amended Contract"))
            .expect("amended Contract JSON");
    let current_contract_digest =
        cockpit_protocol::digest_json(&current_contract).expect("amended Contract digest");
    assert_eq!(preflight["reviewState"], "needs_human_confirmation");
    assert_eq!(
        preflight["humanDecisionRequest"]["decisionId"],
        "contract-preflight-review"
    );
    assert_eq!(
        preflight["humanDecisionRequest"]["status"],
        "needs_human_confirmation"
    );
    assert_eq!(
        preflight["humanDecisionRequest"]["recommendedOption"],
        "confirm_review"
    );
    assert_eq!(preflight["actionAdmission"]["workItemId"], WORK_ITEM_ID);
    assert_eq!(
        preflight["actionAdmission"]["repositoryId"],
        current_contract["repositoryId"]
    );
    assert_eq!(
        preflight["actionAdmission"]["sourceDigests"]["contract"],
        current_contract_digest.as_str()
    );
    assert_eq!(
        preflight["actionAdmission"]["sourceDigests"]["repositorySnapshot"],
        preflight["actionAdmission"]["snapshotDigest"]
    );
    assert!(
        preflight["actionAdmission"]["snapshotDigest"]
            .as_str()
            .is_some_and(|digest| digest.starts_with("sha256:")),
        "request should be returned with a current repository snapshot binding"
    );
    let status = run_cli(&[
        "work-item",
        "status",
        "--repo",
        repository_path,
        "--id",
        WORK_ITEM_ID,
        "--json",
    ]);
    assert_success(&status, "status after refreshed preflight");
    let status: Value = serde_json::from_slice(&status.stdout).expect("status JSON");
    assert_eq!(status["humanDecisionRequired"], true);
    assert_eq!(status["humanDecisions"], json!([]));
    if retry_pending {
        let summary: Value = serde_json::from_slice(
            &fs::read(
                root.path()
                    .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json")),
            )
            .expect("read Summary"),
        )
        .expect("Summary JSON");
        assert_eq!(summary["recoveryRetryPending"], true);

        let current_receipt = json!({
            "schemaVersion": 1,
            "decisionId": "contract-preflight-review",
            "decision": "confirm_review",
            "workItemId": WORK_ITEM_ID,
            "repositoryId": repository_id(root.path()),
            "contractDigest": current_contract_digest,
            "preflightDecisionDigest": summary["preflightDecisionDigest"],
            "repositorySnapshotDigest": summary["preflightRepositorySnapshotDigest"],
            "recordedAt": "2026-10-03T00:00:00Z",
            "recordedBy": "human:test-fixture",
            "reason": "fixture only; no human decision is recorded"
        });
        assert_eq!(
            current_receipt["repositorySnapshotDigest"],
            preflight["actionAdmission"]["snapshotDigest"]
        );
        let previous_contract_digest =
            cockpit_protocol::digest_json(&contract).expect("pre-amendment Contract digest");
        for (field, wrong_digest, expected_diagnostic) in [
            (
                "repositoryId",
                Digest::sha256_bytes(b"foreign repository").to_string(),
                "repository identity mismatch",
            ),
            (
                "contractDigest",
                previous_contract_digest.to_string(),
                "Contract digest mismatch",
            ),
            (
                "repositorySnapshotDigest",
                Digest::sha256_bytes(b"stale snapshot").to_string(),
                "snapshot digest mismatch",
            ),
        ] {
            let mut invalid_receipt = current_receipt.clone();
            invalid_receipt[field] = Value::String(wrong_digest);
            let summary_before = fs::read(
                root.path()
                    .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json")),
            )
            .expect("Summary before rejected decision");
            let error = record_work_item_governance_controls_with_runtime(
                root.path(),
                WORK_ITEM_ID,
                &json!({"decisionEvidence": invalid_receipt}),
                &candidate_runtime(),
            )
            .expect_err("foreign or stale review evidence must be rejected");
            assert!(
                error.to_string().contains(expected_diagnostic),
                "{field} should fail for its identity mismatch: {error}"
            );
            assert_eq!(
                fs::read(
                    root.path()
                        .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json")),
                )
                .expect("Summary after rejected decision"),
                summary_before,
                "{field} rejection must not record a human decision"
            );
        }
        assert!(
            !root
                .path()
                .join(format!(
                    ".ai/decisions/{WORK_ITEM_ID}.preflight-review.json"
                ))
                .exists(),
            "invalid review evidence must not create a decision receipt"
        );
    }
    assert!(
        !status["safeActions"]
            .as_array()
            .expect("safe actions")
            .iter()
            .any(|action| action == "run_verification")
    );

    let marker = root.path().join("verification-command-started");
    let mut verify_args = vec![
        "verify".to_owned(),
        "--repo".to_owned(),
        repository_path.to_owned(),
        "--work-item".to_owned(),
        WORK_ITEM_ID.to_owned(),
        "--command".to_owned(),
    ];
    #[cfg(unix)]
    {
        verify_args.push("sh".into());
        verify_args.push("--args=-c".into());
        verify_args.push(format!("--args=printf started > '{}'", marker.display()));
    }
    #[cfg(windows)]
    {
        verify_args.push("cmd".into());
        verify_args.push("--args=/C".into());
        verify_args.push(format!("--args=echo started > \"{}\"", marker.display()));
    }
    let verify_args = verify_args.iter().map(String::as_str).collect::<Vec<_>>();
    let verify = run_cli(&verify_args);
    assert!(
        !verify.status.success(),
        "unreviewed amendment must reject verify"
    );
    let diagnostic = format!(
        "{}{}",
        String::from_utf8_lossy(&verify.stdout),
        String::from_utf8_lossy(&verify.stderr)
    );
    assert!(
        diagnostic.contains("contract_amendment_policy_review_required"),
        "expected the amendment review gate to reject verification, got: {diagnostic}"
    );
    assert!(
        !marker.exists(),
        "verification command must not start before amendment review"
    );
}

#[test]
fn sensitive_amendment_blocks_cli_verification_before_command_spawn() {
    assert_sensitive_amendment_blocks_cli_verification_before_command_spawn(false);
}

#[test]
fn retry_pending_amendment_admits_bound_review_request_without_verification_spawn() {
    assert_sensitive_amendment_blocks_cli_verification_before_command_spawn(true);
}

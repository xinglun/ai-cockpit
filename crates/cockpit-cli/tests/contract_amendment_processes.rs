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

fn assert_mixed_pre_amendment_review_binding_is_rejected(use_mcp: bool) {
    let root = repository();
    let repository_path = root.path().to_str().expect("repository path");
    let contract_relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    let contract_path = root.path().join(&contract_relative);
    let summary_path = root
        .path()
        .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
    let initial = run_cli(&[
        "preflight",
        "--repo",
        repository_path,
        "--contract",
        &contract_relative,
    ]);
    assert_success(&initial, "initial preflight before mixed-binding fixture");
    let checkpoint = run_cli(&[
        "checkpoint",
        "--repo",
        repository_path,
        "--id",
        WORK_ITEM_ID,
    ]);
    assert_success(&checkpoint, "checkpoint before mixed-binding fixture");
    let old_summary: Value = serde_json::from_slice(&fs::read(&summary_path).expect("old Summary"))
        .expect("old Summary JSON");
    let old_contract: Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("old Contract"))
            .expect("old Contract JSON");
    let runtime = candidate_runtime();
    let request = ContractAmendmentRequest {
        schema_version: CONTRACT_AMENDMENT_SCHEMA_VERSION,
        change_id: "mixed-binding-sensitive-amendment".into(),
        expected_contract_digest: cockpit_protocol::digest_json(&old_contract)
            .expect("old Contract digest"),
        reason: "isolated regression: invalidate old preflight before any new review".into(),
        changes: vec![ContractAmendmentChange {
            path: "/scope".into(),
            operation: ContractAmendmentOperation::Replace,
            value: Some(json!(["README.md", "src/**"])),
        }],
    };
    let amended = mcp_call(
        root.path(),
        &runtime,
        "work_item_amend",
        json!({"workItemId": WORK_ITEM_ID, "request": request}),
    );
    assert_eq!(
        amended.pointer("/result/isError"),
        Some(&json!(false)),
        "{amended}"
    );
    let new_contract_bytes = fs::read(&contract_path).expect("new Contract");
    let new_contract: Value =
        serde_json::from_slice(&new_contract_bytes).expect("new Contract JSON");
    let summary_before = fs::read(&summary_path).expect("invalidated Summary");
    let invalidated: Value =
        serde_json::from_slice(&summary_before).expect("invalidated Summary JSON");
    assert_eq!(invalidated["preflightState"], "not_run");
    assert_eq!(
        invalidated["preflightContractDigest"],
        old_summary["preflightContractDigest"]
    );
    assert_eq!(
        invalidated["preflightDecisionDigest"],
        old_summary["preflightDecisionDigest"]
    );
    let history_before = amendment_history_bytes(root.path());
    // This input exists only inside this temporary regression fixture.
    // A current Contract cannot turn the retained old preflight fields into
    // a new identity-bound human review before Runtime generates that review.
    let controls = json!({"decisionEvidence": {
        "schemaVersion": 1,
        "decisionId": "contract-preflight-review",
        "decision": "confirm_review",
        "workItemId": WORK_ITEM_ID,
        "repositoryId": repository_id(root.path()),
        "contractDigest": cockpit_protocol::digest_json(&new_contract).expect("new Contract digest"),
        "preflightDecisionDigest": old_summary["preflightDecisionDigest"],
        "repositorySnapshotDigest": old_summary["preflightRepositorySnapshotDigest"],
        "recordedAt": "2026-10-16T00:00:00Z",
        "recordedBy": "human:isolated-automated-test-fixture",
        "reason": "negative automated fixture only; not an actual human approval"
    }});
    if use_mcp {
        let response = mcp_call(
            root.path(),
            &runtime,
            "work_item_controls",
            json!({"workItemId": WORK_ITEM_ID, "controls": controls}),
        );
        assert_eq!(
            response.pointer("/result/isError"),
            Some(&json!(true)),
            "MCP must reject a new Contract mixed with pre-amendment preflight fields: {response}"
        );
    } else {
        let input = tempfile::NamedTempFile::new().expect("controls input");
        fs::write(
            input.path(),
            serde_json::to_vec(&controls).expect("controls JSON"),
        )
        .expect("write controls input");
        let response = run_cli(&[
            "work-item",
            "controls",
            "--repo",
            repository_path,
            "--id",
            WORK_ITEM_ID,
            "--input",
            input.path().to_str().expect("controls path"),
        ]);
        assert!(
            !response.status.success(),
            "CLI must reject a new Contract mixed with pre-amendment preflight fields; stdout={} stderr={}",
            String::from_utf8_lossy(&response.stdout),
            String::from_utf8_lossy(&response.stderr)
        );
    }
    assert_eq!(
        fs::read(&summary_path).expect("Summary after rejection"),
        summary_before
    );
    assert_eq!(
        fs::read(&contract_path).expect("Contract after rejection"),
        new_contract_bytes
    );
    assert_eq!(amendment_history_bytes(root.path()), history_before);
    assert!(
        !root
            .path()
            .join(format!(
                ".ai/decisions/{WORK_ITEM_ID}.preflight-review.json"
            ))
            .exists(),
        "rejected mixed binding must not publish a decision receipt"
    );
}

#[test]
fn mixed_current_contract_and_pre_amendment_preflight_receipt_is_rejected_by_cli() {
    assert_mixed_pre_amendment_review_binding_is_rejected(false);
}

#[test]
fn mixed_current_contract_and_pre_amendment_preflight_receipt_is_rejected_by_mcp() {
    assert_mixed_pre_amendment_review_binding_is_rejected(true);
}

fn assert_recorded_preflight_review_boundary(
    state: Option<&str>,
    current_contract: bool,
    use_mcp: bool,
    accepted: bool,
) {
    let root = repository();
    let repo = root.path().to_str().expect("repository path");
    let contract_relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    let contract_path = root.path().join(&contract_relative);
    let summary_path = root
        .path()
        .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
    assert_success(
        &run_cli(&[
            "preflight",
            "--repo",
            repo,
            "--contract",
            &contract_relative,
        ]),
        "record fixture preflight",
    );
    let contract_bytes = fs::read(&contract_path).expect("Contract bytes");
    let contract: Value = serde_json::from_slice(&contract_bytes).expect("Contract JSON");
    let mut summary: Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    // Isolated boundary fixtures vary the recorded field under test. The
    // native amendment reproduction above independently covers real history.
    if let Some(state) = state {
        summary["preflightState"] = json!(state);
    } else {
        summary
            .as_object_mut()
            .expect("Summary object")
            .remove("preflightState");
    }
    if !current_contract {
        summary["preflightContractDigest"] = json!(Digest::sha256_bytes(b"older Contract"));
    }
    fs::write(
        &summary_path,
        serde_json::to_vec_pretty(&summary).expect("fixture Summary"),
    )
    .expect("write isolated fixture");
    let before = fs::read(&summary_path).expect("Summary before record");
    let controls = json!({"decisionEvidence": {
        "schemaVersion": 1, "decisionId": "contract-preflight-review", "decision": "confirm_review",
        "workItemId": WORK_ITEM_ID, "repositoryId": repository_id(root.path()),
        "contractDigest": cockpit_protocol::digest_json(&contract).expect("Contract digest"),
        "preflightDecisionDigest": summary["preflightDecisionDigest"],
        "repositorySnapshotDigest": summary["preflightRepositorySnapshotDigest"],
        "recordedAt": "2026-10-16T00:00:00Z", "recordedBy": "human:isolated-test-fixture",
        "reason": "automated boundary fixture; no real human approval"
    }});
    if use_mcp {
        let response = mcp_call(
            root.path(),
            &candidate_runtime(),
            "work_item_controls",
            json!({"workItemId":WORK_ITEM_ID,"controls":controls}),
        );
        assert_eq!(
            response.pointer("/result/isError"),
            Some(&json!(!accepted)),
            "state={state:?}, current_contract={current_contract}: {response}"
        );
    } else {
        let input = tempfile::NamedTempFile::new().expect("controls input");
        fs::write(
            input.path(),
            serde_json::to_vec(&controls).expect("controls JSON"),
        )
        .expect("controls input bytes");
        let response = run_cli(&[
            "work-item",
            "controls",
            "--repo",
            repo,
            "--id",
            WORK_ITEM_ID,
            "--input",
            input.path().to_str().expect("input path"),
        ]);
        assert_eq!(
            response.status.success(),
            accepted,
            "state={state:?}, current_contract={current_contract}: {}{}",
            String::from_utf8_lossy(&response.stdout),
            String::from_utf8_lossy(&response.stderr)
        );
    }
    let receipt_path = root.path().join(format!(
        ".ai/decisions/{WORK_ITEM_ID}.preflight-review.json"
    ));
    assert_eq!(
        fs::read(&contract_path).expect("unchanged Contract"),
        contract_bytes
    );
    if !accepted {
        assert_eq!(
            fs::read(&summary_path).expect("Summary after rejection"),
            before
        );
        assert!(
            !receipt_path.exists(),
            "rejection must not publish a receipt"
        );
    } else {
        let receipt = fs::read(&receipt_path).expect("actual fixture receipt");
        fs::write(
            root.path().join("README.md"),
            "changed source requires fresh execution, same Contract review remains valid\n",
        )
        .expect("source change");
        let status = run_cli(&[
            "work-item",
            "status",
            "--repo",
            repo,
            "--id",
            WORK_ITEM_ID,
            "--json",
        ]);
        assert_success(&status, "status after same-Contract source change");
        let status: Value = serde_json::from_slice(&status.stdout).expect("status JSON");
        assert_eq!(
            status["humanDecisionRequired"], false,
            "same-Contract approval must be retained: {status}"
        );
        assert_eq!(fs::read(&receipt_path).expect("preserved receipt"), receipt);
    }
}

#[test]
fn missing_recorded_preflight_state_rejects_review_without_writes() {
    for mcp in [false, true] {
        assert_recorded_preflight_review_boundary(None, true, mcp, false);
    }
}

#[test]
fn unknown_recorded_preflight_state_rejects_review_without_writes() {
    for mcp in [false, true] {
        assert_recorded_preflight_review_boundary(Some("unknown"), true, mcp, false);
    }
}

#[test]
fn not_run_recorded_preflight_state_rejects_review_without_writes() {
    for mcp in [false, true] {
        assert_recorded_preflight_review_boundary(Some("not_run"), true, mcp, false);
    }
}

#[test]
fn foreign_recorded_preflight_contract_rejects_review_without_writes() {
    for mcp in [false, true] {
        assert_recorded_preflight_review_boundary(Some("yellow"), false, mcp, false);
    }
}

#[test]
fn valid_recorded_preflight_states_preserve_same_contract_approval() {
    for state in ["green", "yellow", "red"] {
        for mcp in [false, true] {
            assert_recorded_preflight_review_boundary(Some(state), true, mcp, true);
        }
    }
}

fn amended_request_fixture(red: bool) -> tempfile::TempDir {
    let root = repository();
    let repo = root.path().to_str().expect("repository path");
    let contract_relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    if !red {
        let project = root.path().join(".ai/project");
        fs::create_dir_all(&project).expect("project directory");
        fs::write(project.join("documentation-policy.json"), serde_json::to_vec(&json!({
            "schemaVersion":1, "repositoryId":repository_id(root.path()), "defaultProjection":"required",
            "requiredModes":["implementation"], "requiredOperations":[], "preserveExistingRegistrations":true
        })).expect("policy JSON")).expect("documentation-first fixture policy");
    }
    assert_success(
        &run_cli(&[
            "preflight",
            "--repo",
            repo,
            "--contract",
            &contract_relative,
        ]),
        "initial request fixture preflight",
    );
    assert_success(
        &run_cli(&["checkpoint", "--repo", repo, "--id", WORK_ITEM_ID]),
        "request fixture checkpoint",
    );
    let contract: Value =
        serde_json::from_slice(&fs::read(root.path().join(&contract_relative)).expect("Contract"))
            .expect("Contract JSON");
    let changes = vec![json!({"path":"/scope","operation":"add","value":"src/**"})];
    let request = json!({"schemaVersion":1,"changeId":"interlocked-request","expectedContractDigest":cockpit_protocol::digest_json(&contract).expect("digest"),"reason":"isolated request generation fixture","changes":changes});
    let result = mcp_call(
        root.path(),
        &candidate_runtime(),
        "work_item_amend",
        json!({"workItemId":WORK_ITEM_ID,"request":request}),
    );
    assert_eq!(
        result.pointer("/result/isError"),
        Some(&json!(false)),
        "{result}"
    );
    if red {
        // An intentionally contradictory retained envelope makes native
        // Outcome red. Request generation must preserve this evidence and
        // its blocker, never reinterpret it as a passed verification.
        fs::write(
            root.path()
                .join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")),
            br#"{"evidenceSchemaVersion":2,"fixture":"contradictory retained verification envelope"}"#,
        )
        .expect("contradictory evidence fixture");
    }
    root
}

fn assert_interlocked_request_generation(red: bool, use_mcp: bool) {
    let root = amended_request_fixture(red);
    let repo = root.path().to_str().expect("repository path");
    let relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    let summary_path = root
        .path()
        .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
    let history = amendment_history_bytes(root.path());
    let evidence_path = root
        .path()
        .join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json"));
    let evidence_before = fs::read(&evidence_path).ok();
    let status = run_cli(&[
        "work-item",
        "status",
        "--repo",
        repo,
        "--id",
        WORK_ITEM_ID,
        "--json",
    ]);
    assert_success(&status, "interlocked status");
    let status: Value = serde_json::from_slice(&status.stdout).expect("status JSON");
    assert!(
        status["safeActions"]
            .as_array()
            .expect("safe actions")
            .contains(&json!("run_preflight")),
        "interlocked request generation must be admitted: {status}"
    );
    if red {
        assert_eq!(status["governanceState"], "red");
    }
    let mut previous_snapshot = None;
    for observation in 0..2 {
        if observation == 1 {
            fs::write(
                root.path().join("README.md"),
                "new current snapshot for repeated request\n",
            )
            .expect("source update");
        }
        let decision = if use_mcp {
            let response = mcp_call(
                root.path(),
                &candidate_runtime(),
                "preflight",
                json!({"contract":relative}),
            );
            assert_eq!(
                response.pointer("/result/isError"),
                Some(&json!(false)),
                "{response}"
            );
            response["result"]["structuredContent"].clone()
        } else {
            let response = run_cli(&["preflight", "--repo", repo, "--contract", &relative]);
            assert_success(&response, "generate interlocked request");
            serde_json::from_slice(&response.stdout).expect("decision JSON")
        };
        assert_eq!(decision["reviewState"], "needs_human_confirmation");
        assert_eq!(
            decision["humanDecisionRequest"]["decisionId"],
            "contract-preflight-review"
        );
        if red {
            assert_eq!(decision["state"], "red");
            assert_eq!(decision["outcome_state"], "blocked");
            assert!(
                !decision["blockers"]
                    .as_array()
                    .expect("blockers")
                    .is_empty()
            );
        }
        let summary: Value = serde_json::from_slice(&fs::read(&summary_path).expect("Summary"))
            .expect("Summary JSON");
        let contract: Value =
            serde_json::from_slice(&fs::read(root.path().join(&relative)).expect("Contract"))
                .expect("Contract JSON");
        assert_eq!(
            summary["preflightContractDigest"],
            json!(cockpit_protocol::digest_json(&contract).expect("Contract digest"))
        );
        if let Some(previous) = previous_snapshot {
            assert_ne!(
                summary["preflightRepositorySnapshotDigest"], previous,
                "repeat must bind the changed current source snapshot"
            );
        }
        previous_snapshot = Some(summary["preflightRepositorySnapshotDigest"].clone());
        assert!(
            summary.get("decisionEvidence").is_none(),
            "request generation never records confirmation"
        );
        assert_eq!(summary["checkpointCount"], 1);
        assert_eq!(amendment_history_bytes(root.path()), history);
        assert_eq!(fs::read(&evidence_path).ok(), evidence_before);
    }
    let after = run_cli(&[
        "work-item",
        "status",
        "--repo",
        repo,
        "--id",
        WORK_ITEM_ID,
        "--json",
    ]);
    assert_success(&after, "pending request status");
    let after: Value = serde_json::from_slice(&after.stdout).expect("status JSON");
    assert_eq!(after["humanDecisionRequired"], true);
    assert!(
        !after["safeActions"]
            .as_array()
            .expect("safe actions")
            .contains(&json!("run_verification"))
    );
    assert!(
        !root
            .path()
            .join(format!(
                ".ai/decisions/{WORK_ITEM_ID}.preflight-review.json"
            ))
            .exists()
    );
}

#[test]
fn documentation_first_interlocked_request_is_generated_by_cli() {
    assert_interlocked_request_generation(false, false);
}

#[test]
fn documentation_first_interlocked_request_is_generated_by_mcp() {
    assert_interlocked_request_generation(false, true);
}

#[test]
fn governance_red_interlocked_request_is_generated_by_cli() {
    assert_interlocked_request_generation(true, false);
}

#[test]
fn governance_red_interlocked_request_is_generated_by_mcp() {
    assert_interlocked_request_generation(true, true);
}

#[test]
fn interlocked_request_rejects_invalid_provenance_without_writes() {
    for case in [
        "marker_missing",
        "marker_hash",
        "invalidated_checks",
        "revalidation_snapshot",
        "before_edit_hash",
        "summary_repo",
        "summary_wi",
        "unjournaled_contract",
        "journal_tampered",
        "journal_pending",
        "not_ready",
        "closed",
        "recovered",
        "archived",
    ] {
        let root = amended_request_fixture(true);
        let repo = root.path().to_str().expect("repository path");
        let relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
        let contract_path = root.path().join(&relative);
        let summary_path = root
            .path()
            .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
        let mut summary: Value = serde_json::from_slice(&fs::read(&summary_path).expect("Summary"))
            .expect("Summary JSON");
        match case {
            "marker_missing" => {
                summary
                    .as_object_mut()
                    .expect("Summary object")
                    .remove("verificationInvalidatedByContractAmendment");
            }
            "marker_hash" => {
                summary["verificationInvalidatedByContractAmendment"]["contractHash"] =
                    json!(Digest::sha256_bytes(b"foreign Contract"))
            }
            "invalidated_checks" => {
                summary["verificationInvalidatedByContractAmendment"]["invalidatedRequiredChecks"] =
                    json!(["fabricated"]);
                summary["checkpointEvidence"]
                    .as_array_mut()
                    .expect("checkpoints")
                    .last_mut()
                    .expect("last checkpoint")["invalidatedRequiredChecks"] = json!(["fabricated"]);
            }
            "revalidation_snapshot" => {
                summary["checkpointEvidence"]
                    .as_array_mut()
                    .expect("checkpoints")
                    .last_mut()
                    .expect("last checkpoint")["repositorySnapshotDigest"] =
                    json!(Digest::sha256_bytes(b"foreign snapshot"))
            }
            "before_edit_hash" => {
                summary["checkpointEvidence"][0]["contractHash"] =
                    json!(Digest::sha256_bytes(b"foreign before-edit"))
            }
            "summary_repo" => {
                summary["repositoryId"] = json!(Digest::sha256_bytes(b"foreign repository"))
            }
            "summary_wi" => summary["workItemId"] = json!("WI-FOREIGN"),
            "unjournaled_contract" => {
                let mut contract: Value =
                    serde_json::from_slice(&fs::read(&contract_path).expect("Contract"))
                        .expect("Contract JSON");
                contract["goal"] = json!("unrecorded Contract change");
                fs::write(
                    &contract_path,
                    serde_json::to_vec(&contract).expect("Contract JSON"),
                )
                .expect("isolated Contract tamper");
            }
            "journal_tampered" => {
                let path = root.path().join(format!(
                    ".ai/evidence/{WORK_ITEM_ID}.contract-amendments/00000001.committed.json"
                ));
                let mut value: Value = serde_json::from_slice(&fs::read(&path).expect("journal"))
                    .expect("journal JSON");
                value["receipt"]["reason"] = json!("tampered");
                fs::write(path, serde_json::to_vec(&value).expect("journal JSON"))
                    .expect("isolated journal tamper");
            }
            "journal_pending" => fs::remove_file(root.path().join(format!(
                ".ai/evidence/{WORK_ITEM_ID}.contract-amendments/00000001.committed.json"
            )))
            .expect("isolated pending transaction"),
            phase => summary["state"] = json!(phase),
        }
        fs::write(
            &summary_path,
            serde_json::to_vec(&summary).expect("Summary JSON"),
        )
        .expect("isolated projection");
        let before_summary = fs::read(&summary_path).expect("Summary before rejection");
        let before_contract = fs::read(&contract_path).expect("Contract before rejection");
        let before_history = amendment_history_bytes(root.path());
        let cli = run_cli(&["preflight", "--repo", repo, "--contract", &relative]);
        assert!(
            !cli.status.success(),
            "{case} must not admit request generation"
        );
        let mcp = mcp_call(
            root.path(),
            &candidate_runtime(),
            "preflight",
            json!({"contract":relative}),
        );
        assert_eq!(
            mcp.pointer("/result/isError"),
            Some(&json!(true)),
            "{case}: {mcp}"
        );
        assert_eq!(
            fs::read(&summary_path).expect("Summary after rejection"),
            before_summary,
            "{case}"
        );
        assert_eq!(
            fs::read(&contract_path).expect("Contract after rejection"),
            before_contract,
            "{case}"
        );
        assert_eq!(
            amendment_history_bytes(root.path()),
            before_history,
            "{case}"
        );
    }
}

#[test]
fn interlocked_request_covers_each_eligible_nonterminal_phase() {
    for phase in ["implementation_active", "checkpointed", "finish_ready"] {
        let root = amended_request_fixture(true);
        let repo = root.path().to_str().expect("repository path");
        let relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
        let path = root
            .path()
            .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
        let mut summary: Value =
            serde_json::from_slice(&fs::read(&path).expect("Summary")).expect("Summary JSON");
        summary["state"] = json!(phase);
        fs::write(&path, serde_json::to_vec(&summary).expect("Summary JSON"))
            .expect("isolated lifecycle projection");
        assert_success(
            &run_cli(&["preflight", "--repo", repo, "--contract", &relative]),
            phase,
        );
    }
}

fn assert_checkpoint_provenance_rejected(case: &str, use_mcp: bool) {
    let root = amended_request_fixture(true);
    let repo = root.path().to_str().expect("repository path");
    let relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    let summary_path = root
        .path()
        .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
    let contract_path = root.path().join(&relative);
    let decision_path = root.path().join(format!(
        ".ai/decisions/{WORK_ITEM_ID}.preflight-review.json"
    ));
    let material_fallback = case.starts_with("count_");
    if material_fallback {
        // A real non-ancestor HEAD makes committed-material projection
        // unavailable. Keep the declared base and the native journal intact.
        git(
            root.path(),
            &["checkout", "--orphan", "material-unavailable"],
        );
        git(root.path(), &["add", "README.md"]);
        git(
            root.path(),
            &["commit", "-qm", "isolated non-ancestor material HEAD"],
        );
    }
    let read_status = || {
        let result = run_cli(&[
            "work-item",
            "status",
            "--repo",
            repo,
            "--id",
            WORK_ITEM_ID,
            "--json",
        ]);
        assert_success(&result, "checkpoint provenance status");
        serde_json::from_slice::<Value>(&result.stdout).expect("status JSON")
    };
    let control = read_status();
    assert!(
        control["safeActions"]
            .as_array()
            .expect("actions")
            .contains(&json!("run_preflight")),
        "untampered native checkpoint must admit review-request generation: {control}"
    );
    if material_fallback {
        assert!(
            control["blockers"]
                .as_array()
                .expect("blockers")
                .contains(&json!("material_review_projection_unavailable")),
            "the real material fallback must be active: {control}"
        );
    }
    let mut summary: Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    match case {
        "missing" | "forged_digest" | "malformed" => {
            let value = match case {
                "forged_digest" => Some(json!(Digest::sha256_bytes(
                    b"same forged checkpoint source"
                ))),
                "malformed" => Some(json!("not-a-contract-digest")),
                _ => None,
            };
            for (stage, field) in [
                ("before_edit", "contractHash"),
                (
                    "contract_amendment_revalidation",
                    "originalBeforeEditContractHash",
                ),
            ] {
                let record = summary["checkpointEvidence"]
                    .as_array_mut()
                    .expect("checkpoints")
                    .iter_mut()
                    .rfind(|r| r["stage"] == stage)
                    .expect("source checkpoint");
                if let Some(value) = &value {
                    assert_ne!(
                        &record[field], value,
                        "the forged source must differ from native history"
                    );
                    record[field] = value.clone();
                } else {
                    record
                        .as_object_mut()
                        .expect("checkpoint object")
                        .remove(field);
                }
            }
        }
        "count_zero" => summary["checkpointCount"] = json!(0),
        "count_two" => summary["checkpointCount"] = json!(2),
        _ => panic!("unknown checkpoint provenance case"),
    }
    fs::write(
        &summary_path,
        serde_json::to_vec(&summary).expect("Summary JSON"),
    )
    .expect("isolated Summary tamper");
    let before_summary = fs::read(&summary_path).expect("before Summary");
    let before_contract = fs::read(&contract_path).expect("before Contract");
    let before_history = amendment_history_bytes(root.path());
    let before_decision = fs::read(&decision_path).ok();
    let status = read_status();
    let (rejected, response) = if use_mcp {
        let response = mcp_call(
            root.path(),
            &candidate_runtime(),
            "preflight",
            json!({"contract":relative}),
        );
        (
            response.pointer("/result/isError") == Some(&json!(true)),
            response.to_string(),
        )
    } else {
        let response = run_cli(&["preflight", "--repo", repo, "--contract", &relative]);
        (
            !response.status.success(),
            String::from_utf8_lossy(&response.stdout).into_owned(),
        )
    };
    assert!(
        rejected,
        "{case} transport={} must reject request generation; status={status}; response={response}",
        if use_mcp { "MCP" } else { "CLI" }
    );
    assert!(
        !status["safeActions"]
            .as_array()
            .expect("actions")
            .contains(&json!("run_preflight")),
        "{case} must not regain preflight through material fallback: {status}"
    );
    assert_eq!(
        fs::read(&summary_path).expect("after Summary"),
        before_summary,
        "{case}"
    );
    assert_eq!(
        fs::read(&contract_path).expect("after Contract"),
        before_contract,
        "{case}"
    );
    assert_eq!(
        amendment_history_bytes(root.path()),
        before_history,
        "{case}"
    );
    assert_eq!(fs::read(&decision_path).ok(), before_decision, "{case}");
}

#[test]
fn checkpoint_source_pair_missing_is_rejected_by_cli() {
    assert_checkpoint_provenance_rejected("missing", false);
}
#[test]
fn checkpoint_source_pair_missing_is_rejected_by_mcp() {
    assert_checkpoint_provenance_rejected("missing", true);
}
#[test]
fn checkpoint_source_pair_forged_digest_is_rejected_by_cli() {
    assert_checkpoint_provenance_rejected("forged_digest", false);
}
#[test]
fn checkpoint_source_pair_forged_digest_is_rejected_by_mcp() {
    assert_checkpoint_provenance_rejected("forged_digest", true);
}
#[test]
fn checkpoint_source_pair_malformed_is_rejected_by_cli() {
    assert_checkpoint_provenance_rejected("malformed", false);
}
#[test]
fn checkpoint_source_pair_malformed_is_rejected_by_mcp() {
    assert_checkpoint_provenance_rejected("malformed", true);
}
#[test]
fn checkpoint_count_zero_material_fallback_is_rejected_by_cli() {
    assert_checkpoint_provenance_rejected("count_zero", false);
}
#[test]
fn checkpoint_count_zero_material_fallback_is_rejected_by_mcp() {
    assert_checkpoint_provenance_rejected("count_zero", true);
}
#[test]
fn checkpoint_count_two_material_fallback_is_rejected_by_cli() {
    assert_checkpoint_provenance_rejected("count_two", false);
}
#[test]
fn checkpoint_count_two_material_fallback_is_rejected_by_mcp() {
    assert_checkpoint_provenance_rejected("count_two", true);
}

#[test]
fn native_activation_keeps_post_checkpoint_amendment_reviewable() {
    let directory = tempfile::tempdir().expect("activation fixture directory");
    let root = directory.path().join("repo");
    fs::create_dir(&root).expect("fixture repository");
    let origin = directory.path().join("origin.git");
    let id = "WI-CLI-ACTIVATION-CONTINUITY";
    git(&root, &["init", "-q", "--initial-branch=main"]);
    git(&root, &["config", "user.name", "Activation fixture"]);
    git(
        &root,
        &["config", "user.email", "activation@example.invalid"],
    );
    fs::write(root.join("README.md"), "isolated activation fixture\n").expect("README");
    git(&root, &["add", "README.md"]);
    git(&root, &["commit", "-qm", "fixture baseline"]);
    git(
        &root,
        &[
            "init",
            "-q",
            "--bare",
            "--initial-branch=main",
            origin.to_str().expect("origin path"),
        ],
    );
    git(
        &root,
        &[
            "remote",
            "add",
            "origin",
            origin.to_str().expect("origin path"),
        ],
    );
    git(&root, &["push", "-q", "-u", "origin", "main"]);
    git(&root, &["remote", "set-head", "origin", "-a"]);
    git(&root, &["checkout", "-q", "-b", "fixture/activation"]);
    let cli = |args: &[&str]| {
        Command::new(candidate_binary())
            .args(args)
            .arg("--repo")
            .arg(&root)
            .output()
            .expect("native fixture command")
    };
    assert_success(&cli(&["attach"]), "attach activation fixture");
    assert_success(
        &cli(&["work-item", "new", "--id", id, "--mode", "implementation"]),
        "create real not_ready scaffold",
    );
    let contract_path = root.join(format!(".ai/work-items/active/{id}.contract.json"));
    let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
    let contract: Value = serde_json::from_slice(&fs::read(&contract_path).expect("scaffold"))
        .expect("scaffold JSON");
    assert_eq!(contract["state"], "not_ready");
    let intent = "Exercise a real native activation boundary";
    let goal = "Keep post-checkpoint amendments reviewable";
    let acceptance = "The activation boundary preserves strict native amendment admission";
    let request_path = directory.path().join("before-activation.json");
    fs::write(
        &request_path,
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": 1,
            "changeId": "before-activation",
            "expectedContractDigest": cockpit_protocol::digest_json(&contract).expect("digest"),
            "reason": "Isolated test plan; no real Work Item or human authority.",
            "changes": [
                {"path":"/intent","operation":"set","value":intent},
                {"path":"/goal","operation":"set","value":goal},
                {"path":"/scope","operation":"replace","value":["README.md"]},
                {"path":"/outOfScope","operation":"replace","value":[]},
                {"path":"/risk","operation":"set","value":"normal"},
                {"path":"/authority","operation":"set","value":"authorized"},
                {"path":"/acceptanceCriteria","operation":"replace","value":[acceptance]},
                {"path":"/verification","operation":"replace","value":["git diff --check"]}
            ]
        }))
        .expect("request JSON"),
    )
    .expect("pre-activation request");
    assert_success(
        &cli(&[
            "work-item",
            "amend",
            "--id",
            id,
            "--request",
            request_path.to_str().expect("request path"),
        ]),
        "record pre-activation amendment",
    );
    assert_success(
        &cli(&[
            "start",
            "--id",
            id,
            "--intent",
            intent,
            "--goal",
            goal,
            "--scope",
            "README.md",
            "--risk",
            "normal",
            "--authority",
            "authorized",
            "--acceptance",
            acceptance,
            "--verification",
            "git diff --check",
            "--prepare",
        ]),
        "activate real scaffold and prepare review",
    );
    let active: Value = serde_json::from_slice(&fs::read(&contract_path).expect("active Contract"))
        .expect("active JSON");
    let summary: Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    let review_path = directory.path().join("fixture-review.json");
    fs::write(
        &review_path,
        serde_json::to_vec_pretty(&json!({"decisionEvidence": {
            "schemaVersion": 1,
            "decisionId": "contract-preflight-review",
            "decision": "confirm_review",
            "workItemId": id,
            "repositoryId": active["repositoryId"],
            "contractDigest": cockpit_protocol::digest_json(&active).expect("active digest"),
            "preflightDecisionDigest": summary["preflightDecisionDigest"],
            "repositorySnapshotDigest": summary["preflightRepositorySnapshotDigest"],
            "recordedAt": "2026-10-17T00:00:00Z",
            "recordedBy": "fixture:activation-test-only",
            "reason": "Synthetic isolated fixture review; not a real human response or history proof."
        }}))
        .expect("fixture review JSON"),
    )
    .expect("fixture review");
    assert_success(
        &cli(&[
            "work-item",
            "controls",
            "--id",
            id,
            "--input",
            review_path.to_str().expect("review path"),
        ]),
        "record fixture-only review",
    );
    assert_success(
        &cli(&[
            "preflight",
            "--contract",
            contract_path.to_str().expect("Contract path"),
        ]),
        "preflight reviewed fixture",
    );
    assert_success(&cli(&["checkpoint", "--id", id]), "fixture checkpoint");
    let post_path = directory.path().join("after-checkpoint.json");
    fs::write(
        &post_path,
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": 1,
            "changeId": "after-checkpoint",
            "expectedContractDigest": cockpit_protocol::digest_json(&active).expect("digest"),
            "reason": "Keep the amended fixture plan reviewable without granting its review.",
            "changes": [{"path":"/goal","operation":"set","value":"Review the current amended fixture plan"}]
        }))
        .expect("post-checkpoint JSON"),
    )
    .expect("post-checkpoint request");
    let amended = cli(&[
        "work-item",
        "amend",
        "--id",
        id,
        "--request",
        post_path.to_str().expect("post request path"),
    ]);
    let status = cli(&["work-item", "status", "--id", id, "--json"]);
    let history = cli(&["work-item", "amendments", "--id", id]);
    assert_success(&history, "read intact amendment journal");
    let history: Vec<Value> = serde_json::from_slice(&history.stdout).expect("history JSON");
    assert_eq!(history.len(), 2);
    assert_eq!(
        history[1]["previousJournalDigest"],
        history[0]["journalDigest"]
    );
    if !amended.status.success() || !status.status.success() {
        let preserved = directory.keep();
        panic!(
            "real activation boundary must keep amendment admission reviewable; fixture preserved at {}; amendment stderr={}; status stderr={}",
            preserved.display(),
            String::from_utf8_lossy(&amended.stderr),
            String::from_utf8_lossy(&status.stderr)
        );
    }
    let status: Value = serde_json::from_slice(&status.stdout).expect("current status JSON");
    assert_eq!(status["humanDecisionRequired"], true, "{status}");
    assert!(
        !status["safeActions"]
            .as_array()
            .expect("safe actions")
            .iter()
            .any(|action| action == "run_verification"),
        "the new Contract still needs its own human review: {status}"
    );
}

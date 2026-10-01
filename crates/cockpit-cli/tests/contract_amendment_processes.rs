use cockpit_core::Digest;
use cockpit_protocol::{
    CONTRACT_AMENDMENT_AUTHORIZATION_SCHEMA_VERSION, CONTRACT_AMENDMENT_SCHEMA_VERSION,
    ContractAmendmentAuthorization, ContractAmendmentChange, ContractAmendmentDecision,
    ContractAmendmentOperation, ContractAmendmentRequest, EvidenceAssurance, PROTOCOL_VERSION,
    RuntimeContext,
};
use cockpit_repository::{
    WorkItemStartOptions, attach, preflight_work_item, start_work_item_with_options,
};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Arc, Barrier};

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
    request_with_goal(
        root,
        "plan-change-001",
        "record and verify the implementation-plan adjustment",
    )
}

fn request_with_goal(root: &Path, change_id: &str, goal: &str) -> ContractAmendmentRequest {
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{WORK_ITEM_ID}.contract.json"));
    let contract: Value =
        serde_json::from_slice(&fs::read(contract_path).expect("Contract")).expect("Contract JSON");
    authorize_request(
        root,
        ContractAmendmentRequest {
            schema_version: CONTRACT_AMENDMENT_SCHEMA_VERSION,
            change_id: change_id.into(),
            expected_contract_digest: cockpit_protocol::digest_json(&contract)
                .expect("Contract digest"),
            reason: "The implementation surfaced a more accurate acceptance plan.".into(),
            changes: vec![ContractAmendmentChange {
                path: "/goal".into(),
                operation: ContractAmendmentOperation::Set,
                value: Some(json!(goal)),
            }],
            authorization: None,
        },
    )
}

fn authorize_request(
    root: &Path,
    mut request: ContractAmendmentRequest,
) -> ContractAmendmentRequest {
    let preview =
        cockpit_repository::check_work_item_contract_amendment(root, WORK_ITEM_ID, &request)
            .expect("read-only amendment preview");
    assert!(
        preview
            .blockers
            .iter()
            .any(|blocker| blocker == "authorization_missing")
    );
    request.authorization = Some(ContractAmendmentAuthorization {
        schema_version: CONTRACT_AMENDMENT_AUTHORIZATION_SCHEMA_VERSION,
        decision_id: "direct-human-decision-test-001".into(),
        decision: ContractAmendmentDecision::AuthorizeChange,
        authorized_by: "human:test-authorizer".into(),
        authority_source: "direct user instruction in process test".into(),
        assurance: EvidenceAssurance::SelfDeclared,
        executed_by: "agent:cockpit-process-test".into(),
        repository_id: preview.repository_id,
        work_item_id: preview.work_item_id,
        contract_digest: preview.contract_digest,
        repository_snapshot_digest: preview.repository_snapshot_digest,
        request_digest: preview.request_digest,
        changed_paths: preview.changed_paths,
    });
    request
}

#[test]
fn typed_amendment_without_bound_human_authorization_is_rejected_without_mutation() {
    let root = repository();
    let mut request = typed_request(root.path());
    request.authorization = None;
    let request = serde_json::to_value(request).expect("request JSON");
    let request_directory = tempfile::tempdir().expect("request directory");
    let request_path = request_directory.path().join("unbound-amendment.json");
    fs::write(
        &request_path,
        serde_json::to_vec_pretty(&request).expect("serialize request"),
    )
    .expect("write request");

    let contract_path = root
        .path()
        .join(".ai/work-items/active")
        .join(format!("{WORK_ITEM_ID}.contract.json"));
    let summary_path = root
        .path()
        .join(".ai/work-items/active")
        .join(format!("{WORK_ITEM_ID}.summary.json"));
    let shared_coordination_root = root.path().join(".git/.ai-cockpit/coordination/v1");
    let contract_before = fs::read(&contract_path).expect("Contract before");
    let summary_before = fs::read(&summary_path).expect("Summary before");
    let history_before = amendment_history_bytes(root.path());
    assert!(!shared_coordination_root.exists());

    let check = run_cli(&[
        "work-item",
        "amend-check",
        "--repo",
        root.path().to_str().expect("repository path"),
        "--id",
        WORK_ITEM_ID,
        "--request",
        request_path.to_str().expect("request path"),
    ]);
    assert_success(&check, "read-only amendment check");
    let check: Value = serde_json::from_slice(&check.stdout).expect("check JSON");
    assert_eq!(check["allowed"], false, "{check}");
    assert!(
        check["blockers"]
            .as_array()
            .is_some_and(|items| { items.iter().any(|item| item == "authorization_missing") })
    );
    assert_eq!(check["changedPaths"], json!(["/goal"]));
    assert_eq!(
        fs::read(&contract_path).expect("Contract after check"),
        contract_before
    );
    assert_eq!(
        fs::read(&summary_path).expect("Summary after check"),
        summary_before
    );
    assert_eq!(amendment_history_bytes(root.path()), history_before);
    assert!(
        !shared_coordination_root.exists(),
        "read-only check wrote common state"
    );

    let output = run_cli(&[
        "work-item",
        "amend",
        "--repo",
        root.path().to_str().expect("repository path"),
        "--id",
        WORK_ITEM_ID,
        "--request",
        request_path.to_str().expect("request path"),
    ]);

    assert!(
        !output.status.success(),
        "a typed amendment without request-bound human authorization must reject"
    );
    assert_eq!(
        fs::read(&contract_path).expect("Contract after"),
        contract_before
    );
    assert_eq!(
        fs::read(&summary_path).expect("Summary after"),
        summary_before
    );
    assert_eq!(amendment_history_bytes(root.path()), history_before);
    assert!(
        !shared_coordination_root.exists(),
        "rejected apply wrote common state"
    );
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

    let contract_path = root
        .path()
        .join(".ai/work-items/active")
        .join(format!("{WORK_ITEM_ID}.contract.json"));
    let summary_path = root
        .path()
        .join(".ai/work-items/active")
        .join(format!("{WORK_ITEM_ID}.summary.json"));
    let contract_before_check = fs::read(&contract_path).expect("Contract before check");
    let summary_before_check = fs::read(&summary_path).expect("Summary before check");
    let history_before_check = amendment_history_bytes(root.path());
    let check = run_cli(&[
        "work-item",
        "amend-check",
        "--repo",
        root.path().to_str().expect("repository path"),
        "--id",
        WORK_ITEM_ID,
        "--request",
        request_path.to_str().expect("request path"),
    ]);
    assert_success(&check, "authorized read-only amendment check");
    let check: Value = serde_json::from_slice(&check.stdout).expect("check JSON");
    assert_eq!(check["allowed"], true, "{check}");
    assert_eq!(
        check["requestDigest"],
        request
            .authorization
            .as_ref()
            .unwrap()
            .request_digest
            .to_string()
    );
    assert_eq!(
        fs::read(&contract_path).expect("Contract after check"),
        contract_before_check
    );
    assert_eq!(
        fs::read(&summary_path).expect("Summary after check"),
        summary_before_check
    );
    assert_eq!(amendment_history_bytes(root.path()), history_before_check);

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

    let before_legacy_contract = fs::read(&contract_path).expect("Contract before legacy input");
    let before_legacy_summary = fs::read(&summary_path).expect("Summary before legacy input");
    let before_legacy_history = amendment_history_bytes(root.path());
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
    assert!(
        !legacy.status.success(),
        "legacy additive path must fail closed"
    );
    assert_eq!(
        fs::read(&contract_path).expect("Contract after legacy input"),
        before_legacy_contract
    );
    assert_eq!(
        fs::read(&summary_path).expect("Summary after legacy input"),
        before_legacy_summary
    );
    assert_eq!(amendment_history_bytes(root.path()), before_legacy_history);

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
    assert_eq!(cli_history.as_array().map(Vec::len), Some(1));
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

#[test]
fn sensitive_amendment_blocks_cli_verification_before_command_spawn() {
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
    let contract: Value = serde_json::from_slice(&fs::read(&contract_path).expect("read Contract"))
        .expect("Contract JSON");
    let request = authorize_request(
        root.path(),
        ContractAmendmentRequest {
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
            authorization: None,
        },
    );
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

    preflight_work_item(root.path(), &contract_path)
        .expect("fresh preflight remains unable to satisfy amendment review");
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
fn linked_worktree_processes_cannot_both_apply_from_the_same_contract_digest() {
    let root = repository();
    let peer_parent = tempfile::tempdir().expect("peer worktree parent");
    let peer_path = peer_parent.path().join("peer");
    let peer_path_string = peer_path.to_str().expect("peer path");
    git(
        root.path(),
        &[
            "worktree",
            "add",
            "-b",
            "linked-amendment-race",
            peer_path_string,
            "HEAD",
        ],
    );
    let root_ai = root.path().join(".ai");
    let peer_ai = peer_path.join(".ai");
    fs::create_dir_all(&peer_ai).expect("peer governance directory");
    for entry in fs::read_dir(&root_ai).expect("root governance entries") {
        let entry = entry.expect("root governance entry");
        if entry.file_type().expect("governance entry type").is_file() {
            fs::copy(entry.path(), peer_ai.join(entry.file_name()))
                .expect("copy repository-bound governance identity");
        }
    }
    assert_eq!(
        cockpit_repository::repository_id(root.path()),
        cockpit_repository::repository_id(&peer_path),
        "linked worktrees must resolve the same repository identity"
    );
    let active = root.path().join(".ai/work-items/active");
    let peer_active = peer_path.join(".ai/work-items/active");
    fs::create_dir_all(&peer_active).expect("peer active Work Items");
    for suffix in ["contract.json", "summary.json"] {
        let filename = format!("{WORK_ITEM_ID}.{suffix}");
        fs::copy(active.join(&filename), peer_active.join(&filename))
            .expect("copy same admitted Work Item state into linked worktree");
    }

    let request_a = request_with_goal(root.path(), "race-change-a", "race result A");
    let request_b = request_with_goal(&peer_path, "race-change-b", "race result B");
    assert_eq!(
        request_a.expected_contract_digest, request_b.expected_contract_digest,
        "both processes must start from the same Contract digest"
    );
    let requests = tempfile::tempdir().expect("request files");
    let request_a_path = requests.path().join("request-a.json");
    let request_b_path = requests.path().join("request-b.json");
    fs::write(
        &request_a_path,
        serde_json::to_vec(&request_a).expect("request A JSON"),
    )
    .expect("write request A");
    fs::write(
        &request_b_path,
        serde_json::to_vec(&request_b).expect("request B JSON"),
    )
    .expect("write request B");

    let barrier = Arc::new(Barrier::new(3));
    let spawn = |repository_path: PathBuf, request_path: PathBuf| {
        let barrier = Arc::clone(&barrier);
        std::thread::spawn(move || {
            barrier.wait();
            Command::new(candidate_binary())
                .args([
                    "work-item",
                    "amend",
                    "--repo",
                    repository_path.to_str().expect("repository path"),
                    "--id",
                    WORK_ITEM_ID,
                    "--request",
                    request_path.to_str().expect("request path"),
                ])
                .output()
                .expect("amendment process")
        })
    };
    let process_a = spawn(root.path().to_path_buf(), request_a_path);
    let process_b = spawn(peer_path.clone(), request_b_path);
    barrier.wait();
    let output_a = process_a.join().expect("process A join");
    let output_b = process_b.join().expect("process B join");
    assert_ne!(
        output_a.status.success(),
        output_b.status.success(),
        "exactly one concurrent amendment should commit; A stdout/stderr={}/{}, B stdout/stderr={}/{}",
        String::from_utf8_lossy(&output_a.stdout),
        String::from_utf8_lossy(&output_a.stderr),
        String::from_utf8_lossy(&output_b.stdout),
        String::from_utf8_lossy(&output_b.stderr),
    );
    let rejected = if output_a.status.success() {
        &output_b
    } else {
        &output_a
    };
    let diagnostic = String::from_utf8_lossy(&rejected.stderr);
    assert!(
        diagnostic.contains("contract_digest_conflict"),
        "stale linked-worktree request should be rejected by shared admission, got: {diagnostic}"
    );
    let receipt_count =
        cockpit_repository::read_work_item_contract_amendments(root.path(), WORK_ITEM_ID)
            .expect("root amendment receipts")
            .len()
            + cockpit_repository::read_work_item_contract_amendments(&peer_path, WORK_ITEM_ID)
                .expect("peer amendment receipts")
                .len();
    assert_eq!(
        receipt_count, 1,
        "only the winning Work Item may append a receipt"
    );
}

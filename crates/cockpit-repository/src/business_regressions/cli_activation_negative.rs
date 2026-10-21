//! Native fixture migration for specified negative branches, with actual CLI/MCP observers.
#![cfg(test)]
use crate as cockpit_repository;
use crate::review_authority::pipeline_tests::TestAuthority;
use cockpit_core::Digest;
use cockpit_protocol::CONTRACT_AMENDMENT_SCHEMA_VERSION;
use cockpit_protocol::ContractAmendmentChange;
use cockpit_protocol::ContractAmendmentOperation;
use cockpit_protocol::ContractAmendmentRequest;
use cockpit_protocol::PROTOCOL_VERSION;
use cockpit_protocol::RuntimeContext;
use cockpit_protocol::review_authority::HumanReviewOperation;
use cockpit_repository::WorkItemStartOptions;
use cockpit_repository::attach;
use cockpit_repository::record_recovery_decision;
use cockpit_repository::record_work_item_governance_controls_with_runtime;
use cockpit_repository::repository_id;
use cockpit_repository::start_work_item_with_options;
use serde_json::Value;
use serde_json::json;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
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
    super::transport_fixture::binary()
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

fn mcp_call(root: &Path, runtime: &RuntimeContext, name: &str, arguments: Value) -> Value {
    assert_eq!(
        runtime,
        &candidate_runtime(),
        "a real MCP process uses its actual Runtime; foreign Runtime cases require a separate native assertion"
    );
    super::transport_fixture::mcp_request(
        root,
        &json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":name,"arguments":arguments}}),
    )
}

fn assert_sensitive_amendment_blocks_cli_verification_before_command_spawn(
    authority: &TestAuthority,
    retry_pending: bool,
) {
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
    authority.approve(
        root.path(),
        WORK_ITEM_ID,
        HumanReviewOperation::PreflightReview,
        &json!({"decision":"confirm_review"}),
    );
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
        "AuthorityUnavailable"
    );
    assert_eq!(
        preflight["humanDecisionRequest"]["recommendedOption"],
        "stop_work"
    );
    assert_ne!(
        crate::current_review_authority_state(root.path(), WORK_ITEM_ID).unwrap(),
        cockpit_protocol::review_authority::CurrentReviewAuthorityState::Active,
        "the prior C approval cannot cover the amended C"
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
    let authority = TestAuthority::install();
    assert_sensitive_amendment_blocks_cli_verification_before_command_spawn(&authority, false);
}

fn activation_checkpoint_fixture(authority: &TestAuthority) -> tempfile::TempDir {
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
    // The V1 bytes above are synthetic historical material, not CURRENT.
    authority.approve(
        &root,
        id,
        HumanReviewOperation::PreflightReview,
        &json!({"decision":"confirm_review"}),
    );
    assert_eq!(
        crate::current_review_authority_state(&root, id).unwrap(),
        cockpit_protocol::review_authority::CurrentReviewAuthorityState::Active
    );

    assert_success(
        &cli(&[
            "preflight",
            "--contract",
            contract_path.to_str().expect("Contract path"),
        ]),
        "preflight reviewed fixture",
    );
    crate::checkpoint_work_item(&root, id).expect("private exact review admits native checkpoint");
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
    directory
}

#[test]
fn current_contract_review_cannot_hide_a_missing_activation_commit() {
    let authority = TestAuthority::install();
    let directory = activation_checkpoint_fixture(&authority);
    let root = directory.path().join("repo");
    let id = "WI-CLI-ACTIVATION-CONTINUITY";
    let contract_path = root.join(format!(".ai/work-items/active/{id}.contract.json"));
    let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
    let cli = |args: &[&str]| {
        Command::new(candidate_binary())
            .args(args)
            .arg("--repo")
            .arg(&root)
            .output()
            .expect("native fixture command")
    };
    assert_success(
        &cli(&[
            "preflight",
            "--contract",
            contract_path.to_str().expect("path"),
        ]),
        "prepare current Contract review",
    );
    let current: Value = serde_json::from_slice(&fs::read(&contract_path).expect("Contract"))
        .expect("Contract JSON");
    let summary: Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    let review_path = directory.path().join("current-fixture-review.json");
    fs::write(&review_path, serde_json::to_vec(&json!({"decisionEvidence": {
        "schemaVersion":1, "decisionId":"contract-preflight-review", "decision":"confirm_review",
        "workItemId":id, "repositoryId":current["repositoryId"],
        "contractDigest":cockpit_protocol::digest_json(&current).expect("digest"),
        "preflightDecisionDigest":summary["preflightDecisionDigest"],
        "repositorySnapshotDigest":summary["preflightRepositorySnapshotDigest"],
        "recordedAt":"2026-10-17T00:00:00Z", "recordedBy":"fixture:current-review-only",
        "reason":"Synthetic current Contract review never authenticates a missing activation phase."
    }})).expect("review JSON")).expect("review source");
    // The V1 bytes above are synthetic historical material, not CURRENT.
    authority.approve(
        &root,
        id,
        HumanReviewOperation::PreflightReview,
        &json!({"decision":"confirm_review"}),
    );
    assert_eq!(
        crate::current_review_authority_state(&root, id).unwrap(),
        cockpit_protocol::review_authority::CurrentReviewAuthorityState::Active
    );

    let before_contract = fs::read(&contract_path).expect("Contract bytes");
    let before_summary = fs::read(&summary_path).expect("Summary bytes");
    fs::remove_file(root.join(format!(
        ".ai/evidence/{id}.activation-transition/committed.json"
    )))
    .expect("simulate isolated partial publication");
    let native_error =
        crate::work_item_status_snapshot_with_runtime(&root, id, &candidate_runtime()).unwrap_err();
    assert!(
        native_error
            .to_string()
            .contains("unresolved activation transition"),
        "{native_error}"
    );
    let result = cli(&["work-item", "status", "--id", id, "--json"]);
    assert!(
        !result.status.success(),
        "current review must not hide missing activation commit: {}",
        String::from_utf8_lossy(&result.stdout)
    );
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("unresolved activation transition"),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let preflight = cli(&[
        "preflight",
        "--contract",
        contract_path.to_str().expect("path"),
    ]);
    assert!(
        !preflight.status.success(),
        "current preflight must reject the same missing activation commit"
    );
    assert!(
        String::from_utf8_lossy(&preflight.stderr).contains("unresolved activation transition"),
        "{}",
        String::from_utf8_lossy(&preflight.stderr)
    );
    assert_eq!(
        fs::read(contract_path).expect("Contract unchanged"),
        before_contract
    );
    assert_eq!(
        fs::read(summary_path).expect("Summary unchanged"),
        before_summary
    );
}

fn legacy_activation_fixture(authority: &TestAuthority) -> tempfile::TempDir {
    let directory = activation_checkpoint_fixture(authority);
    let root = directory.path().join("repo");
    let id = "WI-CLI-ACTIVATION-CONTINUITY";
    // Synthetic legacy bytes for the raw-source binding tests only. This is
    // deliberately separate from the private port's current V2 approval.
    let source: Value = serde_json::from_slice(
        &fs::read(directory.path().join("fixture-review.json")).expect("synthetic review source"),
    )
    .expect("synthetic review JSON");
    fs::write(
        root.join(format!(".ai/decisions/{id}.preflight-review.json")),
        serde_json::to_vec_pretty(&source["decisionEvidence"]).unwrap(),
    )
    .expect("synthetic plain V1 historical material, never current authority");
    // This isolated fixture models the old on-disk format. Removing its new
    // proof does not authenticate the historical transition or any real WI.
    fs::remove_dir_all(root.join(format!(".ai/evidence/{id}.activation-transition")))
        .expect("model a legacy fixture without a forward receipt");
    let contract_path = root.join(format!(".ai/work-items/active/{id}.contract.json"));
    let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
    let mut legacy_summary: Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    legacy_summary
        .as_object_mut()
        .expect("Summary object")
        .remove("activationContinuity");
    fs::write(
        &summary_path,
        serde_json::to_vec(&legacy_summary).expect("legacy Summary JSON"),
    )
    .expect("model the legacy Summary format without a capability marker");
    let contract: Value = serde_json::from_slice(&fs::read(&contract_path).expect("Contract"))
        .expect("Contract JSON");
    let runtime = candidate_runtime();
    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .expect("fixture HEAD");
    let branch = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(&root)
        .output()
        .expect("fixture branch");
    let input = directory.path().join("registration.json");
    fs::write(&input, serde_json::to_vec(&json!({
        "schemaVersion":1, "repositoryId":contract["repositoryId"], "workItemId":id,
        "contractDigest":cockpit_protocol::digest_json(&contract).expect("digest"),
        "worktreePath":root.canonicalize().expect("root").to_string_lossy(),
        "branch":String::from_utf8(branch.stdout).expect("branch UTF8").trim(),
        "head":String::from_utf8(head.stdout).expect("head UTF8").trim(), "generation":1,
        "runtime":{"schemaVersion":1,"runtimeVersion":runtime.runtime_version,
            "runtimeDigest":runtime.runtime_digest,"capability":"observed_environment_drift_v1"},
        "declaration":{"providedOutcomes":[],"consumedOutcomes":[],"resourceClaims":[],
            "integrationResponsibility":{"responsibleWorkItemId":id,"targetBranch":"main",
                "compositionOrder":[],"rationale":"Isolated recovery fixture; no real authority."},
            "compositionVerification":{"compatibilityConstraints":[],"requiredScenarios":[],"reusableNodes":[]}}
    })).expect("registration JSON")).expect("registration input");
    let cli = |args: &[&str]| {
        Command::new(candidate_binary())
            .args(args)
            .arg("--repo")
            .arg(&root)
            .output()
            .expect("fixture CLI")
    };
    assert_success(
        &cli(&[
            "work-item",
            "coordination",
            "register",
            "--input",
            input.to_str().expect("path"),
        ]),
        "register exact legacy fixture owner",
    );
    directory
}

fn recovery_metadata_bytes(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn walk(path: &Path, values: &mut Vec<(PathBuf, Vec<u8>)>) {
        if !path.exists() {
            return;
        }
        for entry in fs::read_dir(path).expect("metadata directory") {
            let entry = entry.expect("metadata entry");
            if entry.file_type().expect("entry type").is_dir() {
                walk(&entry.path(), values);
            } else {
                values.push((
                    entry.path(),
                    fs::read(entry.path()).expect("metadata bytes"),
                ));
            }
        }
    }
    let mut values = Vec::new();
    walk(&root.join(".ai"), &mut values);
    walk(&root.join(".git/.ai-cockpit"), &mut values);
    values.sort_by(|left, right| left.0.cmp(&right.0));
    values
}

#[test]
fn invalid_legacy_recovery_plans_preserve_all_metadata_bytes() {
    let authority = TestAuthority::install();
    for case in [
        "missing_start",
        "missing_checkpoint",
        "forged_checkpoint",
        "pending_amendment",
        "partial_recovery",
        "foreign_owner",
        "foreign_runtime",
        "unsupported_runtime",
        "new_forward_marker",
        "orphan_recovery_marker",
    ] {
        let directory = legacy_activation_fixture(&authority);
        let root = directory.path().join("repo");
        let id = "WI-CLI-ACTIVATION-CONTINUITY";
        let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
        let mut summary: Value = serde_json::from_slice(&fs::read(&summary_path).expect("Summary"))
            .expect("Summary JSON");
        match case {
            "missing_start" => {
                summary["lifecycleFacts"]
                    .as_object_mut()
                    .expect("facts")
                    .remove("start");
            }
            "missing_checkpoint" => {
                summary["checkpointEvidence"] = json!([]);
            }
            "forged_checkpoint" => {
                for record in summary["checkpointEvidence"]
                    .as_array_mut()
                    .expect("checkpoint records")
                {
                    if record["stage"] == "before_edit" {
                        record["contractHash"] = json!(Digest::sha256_bytes(b"forged checkpoint"));
                    }
                    if record["stage"] == "contract_amendment_revalidation" {
                        record["originalBeforeEditContractHash"] =
                            json!(Digest::sha256_bytes(b"forged checkpoint"));
                    }
                }
            }
            "pending_amendment" => {
                fs::remove_file(root.join(format!(
                    ".ai/evidence/{id}.contract-amendments/00000002.committed.json"
                )))
                .expect("model pending fixture");
            }
            "partial_recovery" => {
                let partial = root.join(format!(".ai/evidence/{id}.activation-recovery"));
                fs::create_dir(&partial).expect("partial directory");
                fs::write(partial.join("prepared.json"), b"{}").expect("partial fixture phase");
            }
            "foreign_owner" => {
                git(&root, &["checkout", "-qb", "foreign-fixture-owner"]);
            }
            "new_forward_marker" => {
                summary["activationContinuity"] = json!({"capability":"activation_continuity_recovery_v1","journalRelevant":true});
            }
            "orphan_recovery_marker" => {
                summary["activationRecovery"] = json!({"schemaVersion":1,
                    "requestDigest":Digest::sha256_bytes(b"missing recovery record"),
                    "historicalTransitionAuthentication":"unknown","businessAuthorityGranted":false,
                    "freshContractReviewRequired":true});
            }
            _ => {}
        }
        fs::write(
            &summary_path,
            serde_json::to_vec(&summary).expect("fixture Summary JSON"),
        )
        .expect("fixture mutation");
        let before = recovery_metadata_bytes(&root);
        let mut runtime = candidate_runtime();
        if case == "foreign_runtime" {
            runtime.runtime_digest = Digest::sha256_bytes(b"foreign fixture Runtime");
        }
        if case == "unsupported_runtime" {
            runtime.runtime_version = "0.2.105".into();
        }
        let rejection = if matches!(case, "foreign_runtime" | "unsupported_runtime") {
            crate::plan_activation_recovery(&root, id, &runtime)
                .expect_err("explicit Runtime mismatch must reject")
                .to_string()
        } else {
            let response = mcp_call(
                &root,
                &runtime,
                "work_item_activation_recovery_plan",
                json!({"workItemId":id}),
            );
            assert_eq!(
                response.pointer("/result/isError"),
                Some(&json!(true)),
                "{case}: {response}"
            );
            response["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .to_owned()
        };
        assert!(
            rejection.contains(expected_plan_rejection(case)),
            "{case}: wrong validation branch: {rejection}"
        );
        assert_eq!(
            recovery_metadata_bytes(&root),
            before,
            "{case} must not repair or publish metadata implicitly"
        );
    }
}

#[test]
fn recovery_request_binds_native_environment_and_raw_review_bytes() {
    let authority = TestAuthority::install();
    let directory = legacy_activation_fixture(&authority);
    let root = directory.path().join("repo");
    let id = "WI-CLI-ACTIVATION-CONTINUITY";
    let planned = run_cli(&[
        "work-item",
        "activation-recovery",
        "plan",
        "--id",
        id,
        "--repo",
        root.to_str().expect("root"),
    ]);
    assert_success(&planned, "plan bound metadata request");
    let request: Value = serde_json::from_slice(&planned.stdout).expect("request");
    let observed = cockpit_repository::RepositoryExecutionContext::capture(&root)
        .expect("context")
        .observe_environment_identity()
        .expect("actual environment");
    assert_eq!(
        request["environmentDigest"],
        json!(observed.environment_digest)
    );
    let review = fs::read(root.join(format!(".ai/decisions/{id}.preflight-review.json")))
        .expect("historical review bytes");
    assert_eq!(
        request["rawEvidenceDigests"]["preflight-review"],
        json!(Digest::sha256_bytes(&review))
    );
}

fn fixture_present_recovery_decision(root: &Path, request: &Value) -> Value {
    let id = "WI-CLI-ACTIVATION-CONTINUITY";
    let relative = format!(".ai/evidence/{id}/actual-fixture-response.json");
    let path = root.join(&relative);
    fs::create_dir_all(path.parent().expect("response parent")).expect("response directory");
    let source = json!({"schemaVersion":1,"kind":"human_activation_recovery_response_source",
        "requestDigest":request["requestDigest"],"response":"accept_present_baseline",
        "humanActor":"human:isolated-test-fixture","respondedAt":"2026-10-17T00:00:00Z",
        "provenance":"Synthetic response in an isolated test only; no real human authentication."});
    let bytes = serde_json::to_vec(&source).expect("response JSON");
    fs::write(path, &bytes).expect("fixture response source");
    json!({"schemaVersion":1,"decisionId":"activation-present-baseline-review",
        "response":"accept_present_baseline","requestDigest":request["requestDigest"],
        "humanActor":"human:isolated-test-fixture","recorderActor":"agent:test-fixture",
        "assurance":"self_declared","authoritySourcePath":relative,
        "authoritySourceDigest":Digest::sha256_bytes(&bytes),"respondedAt":"2026-10-17T00:00:00Z",
        "reason":"Accept only the exact isolated present baseline, retaining all historical unknowns."})
}

#[test]
fn stale_or_substituted_recovery_decisions_never_publish_or_finish_metadata() {
    let authority = TestAuthority::install();
    if environment_worker(&authority) {
        return;
    }

    for case in [
        "request",
        "summary",
        "contract",
        "history",
        "head",
        "registration",
        "environment",
        "source",
        "agent_as_human",
        "blank_human",
        "old_review_source",
        "partial_recovery",
        "pending_amendment",
        "unknown_field",
    ] {
        let directory = legacy_activation_fixture(&authority);
        let root = directory.path().join("repo");
        let id = "WI-CLI-ACTIVATION-CONTINUITY";
        let planned = run_cli(&[
            "work-item",
            "activation-recovery",
            "plan",
            "--id",
            id,
            "--repo",
            root.to_str().expect("root"),
        ]);
        assert_success(&planned, "plan current recovery fixture");
        let request: Value = serde_json::from_slice(&planned.stdout).expect("request");
        let mut decision = fixture_present_recovery_decision(&root, &request);
        let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
        let contract_path = root.join(format!(".ai/work-items/active/{id}.contract.json"));
        let source_path = root.join(
            decision["authoritySourcePath"]
                .as_str()
                .expect("source path"),
        );
        if case == "request" {
            // A well-formed metadata decision still has no production source.
            let input = directory.path().join("public-no-authority.json");
            fs::write(&input, serde_json::to_vec(&decision).unwrap()).unwrap();
            let untouched = recovery_metadata_bytes(&root);
            let rejected = run_cli(&[
                "work-item",
                "activation-recovery",
                "record",
                "--repo",
                root.to_str().unwrap(),
                "--id",
                id,
                "--input",
                input.to_str().unwrap(),
            ]);
            assert!(!rejected.status.success());
            assert!(
                String::from_utf8_lossy(&rejected.stderr).contains("authority_unavailable"),
                "{}",
                String::from_utf8_lossy(&rejected.stderr)
            );
            assert_eq!(
                recovery_metadata_bytes(&root),
                untouched,
                "production refusal writes no metadata"
            );
        }
        match case {
            "request" => {
                decision["requestDigest"] = json!(Digest::sha256_bytes(b"another request"));
            }
            "summary" => {
                let mut value: Value =
                    serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
                value["fixtureNote"] = json!("changed after request");
                fs::write(&summary_path, serde_json::to_vec(&value).unwrap()).unwrap();
            }
            "contract" => {
                let mut value: Value =
                    serde_json::from_slice(&fs::read(&contract_path).unwrap()).unwrap();
                value["goal"] = json!("a different current Contract");
                fs::write(&contract_path, serde_json::to_vec(&value).unwrap()).unwrap();
            }
            "history" => {
                let path = root.join(format!(
                    ".ai/evidence/{id}.contract-amendments/00000001.prepared.json"
                ));
                let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                value["prepared"]["request"]["reason"] = json!("changed raw history after request");
                fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
            }
            "head" => {
                git(
                    &root,
                    &[
                        "commit",
                        "--allow-empty",
                        "-qm",
                        "advance isolated fixture HEAD",
                    ],
                );
            }
            "registration" => {
                let input = directory.path().join("registration.json");
                let mut value: Value = serde_json::from_slice(&fs::read(&input).unwrap()).unwrap();
                value["generation"] = json!(2);
                fs::write(&input, serde_json::to_vec(&value).unwrap()).unwrap();
                let registered = run_cli(&[
                    "work-item",
                    "coordination",
                    "register",
                    "--repo",
                    root.to_str().unwrap(),
                    "--input",
                    input.to_str().unwrap(),
                ]);
                assert_success(&registered, "advance actual native fixture registration");
            }
            "source" => {
                fs::write(&source_path, b"{}").unwrap();
            }
            "agent_as_human" | "blank_human" => {
                let mut value: Value =
                    serde_json::from_slice(&fs::read(&source_path).unwrap()).unwrap();
                value["humanActor"] = if case == "blank_human" {
                    json!("human:   ")
                } else {
                    json!("agent:cannot-supply-human-decision")
                };
                let bytes = serde_json::to_vec(&value).unwrap();
                fs::write(&source_path, &bytes).unwrap();
                decision["humanActor"] = value["humanActor"].clone();
                decision["authoritySourceDigest"] = json!(Digest::sha256_bytes(&bytes));
            }
            "old_review_source" => {
                let bytes =
                    fs::read(root.join(format!(".ai/decisions/{id}.preflight-review.json")))
                        .unwrap();
                fs::write(&source_path, &bytes).unwrap();
                decision["authoritySourceDigest"] = json!(Digest::sha256_bytes(&bytes));
            }
            "partial_recovery" => {
                let partial = root.join(format!(".ai/evidence/{id}.activation-recovery"));
                fs::create_dir(&partial).unwrap();
                fs::write(partial.join("prepared.json"), b"{}").unwrap();
            }
            "pending_amendment" => {
                fs::remove_file(root.join(format!(
                    ".ai/evidence/{id}.contract-amendments/00000002.committed.json"
                )))
                .unwrap();
            }
            "unknown_field" => {
                decision["unsupported"] = json!(true);
            }
            _ => {}
        }
        let (before, rejection) = if case == "environment" {
            environment_rejection_with_native_worker(
                &root,
                &decision,
                &request["environmentDigest"],
            )
        } else if case == "unknown_field" {
            let before = recovery_metadata_bytes(&root);
            let response = mcp_call(
                &root,
                &candidate_runtime(),
                "work_item_activation_recovery_record",
                json!({"workItemId":id,"decision":decision}),
            );
            assert_eq!(
                response.pointer("/result/isError"),
                Some(&json!(true)),
                "{case}: {response}"
            );
            let reason = response["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .to_owned();
            assert!(
                reason.contains("unknown field") && reason.contains("unsupported"),
                "actual MCP schema rejection: {reason}"
            );
            (before, reason)
        } else {
            let typed: cockpit_protocol::ActivationRecoveryDecision =
                serde_json::from_value(decision.clone()).unwrap();
            // Deliberately approve the exact current outer request containing
            // this inconsistent business value, so the native validator must
            // reject its own CAS/source/actor condition. The mock source is
            // explicit and scoped to this test only.
            let runtime = candidate_runtime();
            authority.approve(
                &root,
                id,
                HumanReviewOperation::ActivationRecovery,
                &json!({"decision":typed,"runtime":runtime}),
            );
            let before = recovery_metadata_bytes(&root);
            let error = crate::record_activation_recovery(&root, id, &typed, &runtime)
                .expect_err("native inconsistent decision must reject");
            (before, error.to_string())
        };
        assert!(
            rejection.contains(expected_record_rejection(case)),
            "{case}: wrong validation branch: {rejection}"
        );
        assert_eq!(
            recovery_metadata_bytes(&root),
            before,
            "{case}: refusal must preserve every existing phase and byte"
        );
    }
}

fn expected_plan_rejection(case: &str) -> &'static str {
    match case {
        "missing_start"
        | "missing_checkpoint"
        | "forged_checkpoint"
        | "new_forward_marker"
        | "orphan_recovery_marker" => {
            "legacy activation recovery requires exact start/checkpoint/history bindings"
        }
        "pending_amendment" => "unresolved Contract amendment transaction requires recovery",
        "partial_recovery" => {
            "existing or partial continuity evidence cannot be replaced by legacy recovery"
        }
        "foreign_owner" | "foreign_runtime" => {
            "activation recovery requires a current exact owner/Runtime registration without pending drift"
        }
        "unsupported_runtime" => {
            "unsupported_runtime_capability: Runtime 0.2.105 owns lifecycle only"
        }
        _ => panic!("unmapped negative: {case}"),
    }
}
fn expected_record_rejection(case: &str) -> &'static str {
    match case {
        "request" | "summary" | "registration" => {
            "stale activation recovery request; no write performed"
        }
        "contract" => "legacy activation recovery requires exact start/checkpoint/history bindings",
        "history" => "Contract amendment prepared record failed its identity or digest chain",
        "head" | "environment" => {
            "activation recovery requires a current exact owner/Runtime registration without pending drift"
        }
        "source" | "old_review_source" => "invalid newly supplied human response source",
        "agent_as_human" | "blank_human" => {
            "exact newly supplied present-baseline response is required; old review is not recovery authority"
        }
        "partial_recovery" => "unresolved activation recovery transaction",
        "pending_amendment" => "unresolved Contract amendment transaction requires recovery",
        "unknown_field" => "unknown field",
        _ => panic!("unmapped negative: {case}"),
    }
}

// Explicit cfg(test) subprocess of THIS test, never the production executable.
// Its port is always installed by the test, never selected by environment.
// The variable transports only fixture data; public JSON supplies no capability.
fn environment_worker(authority: &TestAuthority) -> bool {
    let Some(input) = std::env::var_os("AI_COCKPIT_TEST_ACTIVATION_ENV_INPUT") else {
        return false;
    };
    use std::io::{BufRead, Write};
    let value: Value = serde_json::from_slice(&fs::read(input).unwrap()).unwrap();
    let root = PathBuf::from(value["root"].as_str().unwrap());
    assert_eq!(std::env::var("CARGO_BUILD_JOBS").unwrap(), "2");
    let actual = crate::RepositoryExecutionContext::capture(&root)
        .unwrap()
        .observe_environment_identity()
        .unwrap();
    assert_ne!(
        json!(actual.environment_digest),
        value["originalEnvironmentDigest"],
        "worker must observe a real environment difference"
    );
    let decision: cockpit_protocol::ActivationRecoveryDecision =
        serde_json::from_value(value["decision"].clone()).unwrap();
    let runtime = candidate_runtime();
    let id = "WI-CLI-ACTIVATION-CONTINUITY";
    authority.approve(
        &root,
        id,
        HumanReviewOperation::ActivationRecovery,
        &json!({"decision":decision,"runtime":runtime}),
    );
    let before = recovery_metadata_bytes(&root);
    println!("NATIVE_ACTIVATION_WORKER_READY");
    std::io::stdout().flush().unwrap();
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line).unwrap();
    assert_eq!(line.trim(), "continue-after-parent-snapshot");
    let error = crate::record_activation_recovery(&root, id, &decision, &runtime)
        .expect_err("actual environment mismatch cannot record")
        .to_string();
    assert!(
        error.contains(expected_record_rejection("environment")),
        "{error}"
    );
    assert_eq!(
        recovery_metadata_bytes(&root),
        before,
        "worker's rejection is read-only after fixture approval"
    );
    println!(
        "NATIVE_ACTIVATION_WORKER_ERROR={}",
        serde_json::to_string(&error).unwrap()
    );
    true
}
fn environment_rejection_with_native_worker(
    root: &Path,
    decision: &Value,
    original: &Value,
) -> (Vec<(PathBuf, Vec<u8>)>, String) {
    use std::io::{BufRead, Read, Write};
    use std::process::Stdio;
    let input = tempfile::NamedTempFile::new().unwrap();
    fs::write(
        input.path(),
        serde_json::to_vec(
            &json!({"root":root,"decision":decision,"originalEnvironmentDigest":original}),
        )
        .unwrap(),
    )
    .unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "business_regressions::cli_activation_negative::stale_or_substituted_recovery_decisions_never_publish_or_finish_metadata", "--nocapture", "--test-threads=1"])
        .env("CARGO_BUILD_JOBS", "2")
        .env("AI_COCKPIT_TEST_ACTIVATION_ENV_INPUT", input.path())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let mut output = std::io::BufReader::new(child.stdout.take().unwrap());
    loop {
        let mut line = String::new();
        if output.read_line(&mut line).unwrap() == 0 {
            let status = child.wait_with_output().unwrap();
            panic!(
                "native test worker did not reach approval fence: {}",
                String::from_utf8_lossy(&status.stderr)
            );
        }
        if line.contains("NATIVE_ACTIVATION_WORKER_READY") {
            break;
        }
    }
    let before = recovery_metadata_bytes(root);
    writeln!(
        child.stdin.take().unwrap(),
        "continue-after-parent-snapshot"
    )
    .unwrap();
    let mut text = String::new();
    output.read_to_string(&mut text).unwrap();
    let status = child.wait_with_output().unwrap();
    assert!(
        status.status.success(),
        "native test worker failed: {text} {}",
        String::from_utf8_lossy(&status.stderr)
    );
    let line = text
        .lines()
        .find_map(|s| {
            s.split_once("NATIVE_ACTIVATION_WORKER_ERROR=")
                .map(|(_, value)| value)
        })
        .expect("actual native rejection result");
    (before, serde_json::from_str(line).unwrap())
}

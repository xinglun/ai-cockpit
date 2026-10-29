use cockpit_protocol::{
    ApprovalMode, EvidenceAssurance, GovernancePolicy, GovernancePolicyDocument, PolicyLayer,
    PolicyRule, VerificationRequirement, VerificationTier,
};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, plan_repository_verification,
    plan_repository_verification_action, run_repository_verification,
};
use cockpit_verification::ProtectedGateClass;
#[cfg(unix)]
use std::sync::Mutex;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);
#[cfg(unix)]
static CARGO_TARGET_ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn canonical_clean_large_rust_source_satisfies_read_only_quality_gate() {
    use cockpit_core::Digest;
    use cockpit_protocol::{RuntimeContext, VerificationStage};
    use cockpit_repository::{
        WorkItemStartOptions, attach, evaluate_contract_quality_gate,
        plan_work_item_material_review, start_work_item_with_options,
    };

    fn write_source(path: &Path, padding: &str, body: &str) {
        use std::io::Write as _;
        let mut file = fs::File::create(path).expect("source file");
        file.write_all(padding.as_bytes()).expect("source padding");
        file.write_all(body.as_bytes()).expect("source body");
    }

    let root: ::std::path::PathBuf = repository("canonical-clean-large-rust");
    fs::create_dir(root.join("crates")).expect("source directory");
    let padding = "// unchanged source padding\n".repeat(10_000);
    assert!(padding.len() > cockpit_git::MAX_CHANGE_TEXT_BYTES);
    let source = root.join("crates/material.rs");
    write_source(&source, &padding, "pub fn count() -> usize { 1 }\n");
    run_git(&root, &["add", "crates/material.rs"]);
    run_git(
        &root,
        &[
            "-c",
            "user.name=AI Cockpit Test",
            "-c",
            "user.email=ai-cockpit@example.invalid",
            "commit",
            "-qm",
            "large source baseline",
        ],
    );
    attach(&root).expect("attach fixture");
    let id = "WI-CANONICAL-CLEAN-MATERIAL";
    start_work_item_with_options(
        &root,
        id,
        "inspect complete committed source",
        "preserve exact material checks",
        &["crates/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["complete source inspection satisfies the gate".into()],
            ..Default::default()
        },
    )
    .expect("start isolated fixture");
    write_source(&source, &padding, "pub fn token_count() -> usize { 2 }\n");
    run_git(&root, &["add", "crates/material.rs"]);
    run_git(
        &root,
        &[
            "-c",
            "user.name=AI Cockpit Test",
            "-c",
            "user.email=ai-cockpit@example.invalid",
            "commit",
            "-qm",
            "small harmless material change",
        ],
    );
    let contract_path = root.join(format!(".ai/work-items/active/{id}.contract.json"));
    let contract_bytes = fs::read(&contract_path).expect("Contract bytes");
    let contract: serde_json::Value = serde_json::from_slice(&contract_bytes).unwrap();
    let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
    let summary_bytes = fs::read(&summary_path).expect("Summary bytes");
    let request = plan_work_item_material_review(&root, id).expect("complete canonical inspection");
    assert_eq!(
        request.contract_digest,
        cockpit_protocol::digest_json(&contract).unwrap()
    );
    assert!(request.raw_unknown_codes.is_empty(), "{request:?}");
    assert!(request.finding_codes.is_empty());
    assert_eq!(request.entries.len(), 1);
    assert_eq!(
        request.entries[0].after_blob_digest,
        Some(Digest::sha256_bytes(&fs::read(&source).unwrap())),
    );
    assert_eq!(
        serde_json::to_value(&request.entries[0]).unwrap()["scannerAssessment"],
        "clean",
    );
    let runtime = RuntimeContext {
        runtime_version: "test-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"canonical-clean-material-fixture"),
    };
    let report = evaluate_contract_quality_gate(
        &root,
        &contract_path,
        VerificationStage::PullRequest,
        "hosted",
        contract["baseRevision"].as_str(),
        &runtime,
    )
    .expect("actual read-only quality gate");
    assert_eq!(fs::read(&contract_path).unwrap(), contract_bytes);
    assert_eq!(fs::read(&summary_path).unwrap(), summary_bytes);
    assert!(report.raw_scanner_unknowns.is_empty(), "{report:?}");
    assert!(report.review_receipt_digest.is_none());
    assert_eq!(report.contract_digest, request.contract_digest);
    assert_eq!(report.state, "passed", "{report:?}");
    assert!(report.unknowns.is_empty(), "{report:?}");
    fs::remove_dir_all(root).expect("cleanup isolated fixture");
}

fn repository(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "cockpit-verification-service-{name}-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("repository");
    fs::write(root.join("tracked.txt"), "before\n").expect("tracked file");
    fs::write(root.join(".gitignore"), ".counter\n").expect("ignore counter");
    run_git(&root, &["init", "-q"]);
    run_git(&root, &["config", "core.autocrlf", "false"]);
    run_git(&root, &["add", "."]);
    run_git(
        &root,
        &[
            "-c",
            "user.name=AI Cockpit Test",
            "-c",
            "user.email=ai-cockpit@example.invalid",
            "commit",
            "-qm",
            "fixture",
        ],
    );
    root
}

fn run_git(root: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .status()
            .expect("git")
            .success()
    );
}

#[cfg(unix)]
fn run_without_inherited_cargo_target(test_name: &str) -> bool {
    if std::env::var("AI_COCKPIT_TEST_TARGET_ISOLATION_CHILD")
        .ok()
        .as_deref()
        == Some(test_name)
    {
        return false;
    }
    let _environment_guard = CARGO_TARGET_ENV_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let isolated_home = tempfile::tempdir().expect("isolated Cargo home");
    fs::create_dir_all(isolated_home.path().join(".cache")).expect("isolated Cargo cache");
    let output = Command::new(std::env::current_exe().expect("test binary"))
        .args(["--exact", test_name, "--nocapture"])
        .env_remove("CARGO_TARGET_DIR")
        // Keep the test's purpose: exercise Runtime's no-inherited-target
        // fallback. The fallback uses HOME/USERPROFILE, so give the child a
        // writable, disposable home instead of relying on the host account.
        // Command-scoped overrides leave the parent test process environment
        // unchanged and the mutex serializes the isolation harnesses.
        .env("HOME", isolated_home.path())
        .env("USERPROFILE", isolated_home.path())
        .env("AI_COCKPIT_TEST_TARGET_ISOLATION_CHILD", test_name)
        .output()
        .expect("isolated test subprocess");
    assert!(
        output.status.success(),
        "isolated {test_name} failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    true
}

fn request(
    program: &str,
    args: Vec<String>,
    policy: RepositoryVerificationPolicy,
) -> RepositoryVerificationRequest {
    RepositoryVerificationRequest {
        node_id: "project-command-0".into(),
        program: program.into(),
        args,
        scope: vec!["**".into()],
        stage: "task".into(),
        runner: "local".into(),
        runtime_digest: format!("sha256:{}", "a".repeat(64)),
        base_commit: None,
        workers: 1,
        work_item_id: None,
        timeout_seconds: None,
        policy,
    }
}

#[cfg(unix)]
fn verification_receipt_diagnostics(receipt: &cockpit_verification::VerificationReceipt) -> String {
    use std::fmt::Write as _;

    let mut output = format!(
        "passed={}, nodesPlanned={}, nodesExecuted={}, processesSpawned={}, elapsedMs={}",
        receipt.passed,
        receipt.nodes_planned,
        receipt.nodes_executed,
        receipt.processes_spawned,
        receipt.elapsed_ms
    );
    for record in &receipt.execution_records {
        let stdout = hex::decode(&record.stdout_hex)
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            .unwrap_or_else(|error| format!("<invalid stdout hex: {error}>"));
        let stderr = hex::decode(&record.stderr_hex)
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            .unwrap_or_else(|error| format!("<invalid stderr hex: {error}>"));
        let _ = writeln!(
            output,
            "\nnodeId={}, spawned={}, passed={}, exitCode={:?}, terminationSignal={:?}, timedOut={}, elapsedMs={}, stdoutTruncated={}, stderrTruncated={}\nstdout:\n{}\nstderr:\n{}",
            record.node_id,
            record.spawned,
            record.passed,
            record.exit_code,
            record.termination_signal,
            record.timed_out,
            record.elapsed_ms,
            record.stdout_truncated,
            record.stderr_truncated,
            stdout,
            stderr
        );
    }
    output
}

fn authorize_timeout(root: &Path, max_timeout_seconds: u64) {
    fs::create_dir_all(root.join(".ai")).expect("AI directory");
    let policy = GovernancePolicyDocument {
        schema_version: 1,
        organization: None,
        project: Some(GovernancePolicy {
            policy_id: "project-verification-timeout".into(),
            layer: PolicyLayer::Project,
            rules: vec![PolicyRule {
                operation: "modify_source".into(),
                approval_mode: ApprovalMode::NoHumanApprovalForLowRisk,
                required_evidence: Vec::new(),
                verification_requirement: Some(VerificationRequirement {
                    schema_version: 1,
                    required_tier: VerificationTier::T0,
                    required_assurance: EvidenceAssurance::SelfDeclared,
                    policy_refs: vec!["project-verification-timeout".into()],
                    stage_refs: vec!["task".into()],
                    gate_refs: Vec::new(),
                    reason: "authorize bounded verification timeout regression".into(),
                    max_timeout_seconds: Some(max_timeout_seconds),
                }),
            }],
        }),
    };
    fs::write(
        root.join(".ai/policy.json"),
        serde_json::to_vec_pretty(&policy).expect("policy JSON"),
    )
    .expect("policy");
}

#[test]
fn cargo_workspace_verification_is_partitioned_with_a_complete_deterministic_manifest() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf();
    let noncanonical_root = root.join(".");
    let request = request(
        "cargo",
        vec!["test".into(), "--locked".into(), "--workspace".into()],
        RepositoryVerificationPolicy::NeverReuse,
    );

    let plan = plan_repository_verification(&noncanonical_root, &request).expect("workspace plan");

    let manifest = plan.coverage_manifest.expect("coverage manifest");
    assert_eq!(manifest.source_args, request.args);
    assert_eq!(manifest.workspace_members.len(), plan.requests.len());
    assert_eq!(
        manifest.workspace_members,
        plan.requests
            .iter()
            .map(|request| request
                .args
                .windows(2)
                .find(|window| window[0] == "--package")
                .expect("package argument")[1]
                .clone())
            .collect::<Vec<_>>()
    );
    assert!(manifest.validate().is_ok());

    let snapshot = cockpit_git::GitRepository::discover(&root)
        .expect("git repository")
        .snapshot()
        .expect("repository snapshot");
    let first_request = plan.requests.first().expect("package verification request");
    let planned_node =
        plan_repository_verification_action(&noncanonical_root, first_request, &snapshot, 0)
            .expect("planned package verification");
    assert_eq!(
        planned_node["identityBinding"]["commandDigest"], manifest.command_digests[0],
        "coverage must bind the exact command identity used by the executor"
    );
}

#[test]
fn repository_timeout_is_bound_into_plan_and_execution_receipts() {
    let root = repository("timeout-receipt");
    authorize_timeout(&root, 1);
    let mut request = request("true", vec![], RepositoryVerificationPolicy::NeverReuse);
    request.timeout_seconds = Some(1);

    let run = run_repository_verification(&root, &request).expect("verify");

    assert_eq!(run.receipt.timeout_seconds, Some(1));
    assert_eq!(run.receipt.execution_records[0].timeout_seconds, 1);
    assert_eq!(
        run.receipt
            .plan_receipt
            .expect("plan receipt")
            .timeout_seconds,
        Some(1)
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn timeout_override_requires_policy_and_cap_is_rejected_before_spawn() {
    let root = repository("timeout-policy");
    let mut unauthorized = request("true", vec![], RepositoryVerificationPolicy::NeverReuse);
    unauthorized.timeout_seconds = Some(1);
    let snapshot = cockpit_git::GitRepository::discover(&root)
        .expect("git repository")
        .snapshot()
        .expect("snapshot");
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_secs() as i64;
    assert!(
        plan_repository_verification_action(&root, &unauthorized, &snapshot, now).is_err(),
        "explicit override without policy must fail before planning/spawn"
    );

    let mut over_cap = request("true", vec![], RepositoryVerificationPolicy::NeverReuse);
    over_cap.timeout_seconds = Some(901);
    authorize_timeout(&root, 900);
    assert!(
        plan_repository_verification_action(&root, &over_cap, &snapshot, now).is_err(),
        "timeout above the Runtime cap must fail before planning/spawn"
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn implicit_cargo_package_test_deadline_is_bounded_without_changing_other_defaults() {
    let root = repository("implicit-package-deadline");
    let snapshot = cockpit_git::GitRepository::discover(&root)
        .expect("git repository")
        .snapshot()
        .expect("snapshot");
    let package_test = request(
        "cargo",
        vec![
            "test".into(),
            "--locked".into(),
            "--package".into(),
            "cockpit-repository".into(),
        ],
        RepositoryVerificationPolicy::NeverReuse,
    );
    let package_plan =
        plan_repository_verification_action(&root, &package_test, &snapshot, 0).expect("plan");
    assert_eq!(package_plan["timeoutSeconds"], 600);

    for args in [
        vec!["test".into(), "--locked".into(), "--workspace".into()],
        vec!["clippy".into(), "--workspace".into()],
    ] {
        let other = request("cargo", args, RepositoryVerificationPolicy::NeverReuse);
        let plan = plan_repository_verification_action(&root, &other, &snapshot, 0).expect("plan");
        assert_eq!(plan["timeoutSeconds"], 300);
    }
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn linked_worktrees_do_not_execute_cargo_with_the_same_implicit_target() {
    use std::os::unix::fs::PermissionsExt;

    if run_without_inherited_cargo_target(
        "linked_worktrees_do_not_execute_cargo_with_the_same_implicit_target",
    ) {
        return;
    }

    let root: ::std::path::PathBuf = repository("cargo-target-isolation");
    let linked: ::std::path::PathBuf = root.with_extension("linked");
    let cargo_path: &::std::path::Path = Path::new("fake-bin/cargo");
    fs::create_dir_all(root.join("fake-bin")).expect("fake bin");
    fs::write(
        root.join(cargo_path),
        "#!/bin/sh\nprintf '%s\\n' \"$CARGO_TARGET_DIR\"\n",
    )
    .expect("fake cargo");
    fs::set_permissions(root.join(cargo_path), fs::Permissions::from_mode(0o755))
        .expect("executable fake cargo");
    run_git(&root, &["add", "fake-bin/cargo"]);
    run_git(
        &root,
        &[
            "-c",
            "user.name=AI Cockpit Test",
            "-c",
            "user.email=ai-cockpit@example.invalid",
            "commit",
            "-qm",
            "fake cargo",
        ],
    );
    run_git(
        &root,
        &[
            "worktree",
            "add",
            "--detach",
            linked.to_str().expect("path"),
        ],
    );

    let target_for = |checkout: &::std::path::Path| {
        let run = run_repository_verification(
            checkout,
            &request(
                checkout.join(cargo_path).to_str().expect("cargo path"),
                vec!["--version".into()],
                RepositoryVerificationPolicy::NeverReuse,
            ),
        )
        .expect("fake cargo execution");
        assert!(run.receipt.passed);
        let bytes = hex::decode(&run.receipt.execution_records[0].stdout_hex).expect("stdout hex");
        PathBuf::from(String::from_utf8(bytes).expect("UTF-8 target").trim())
    };
    let first = target_for(&root);
    let second = target_for(&linked);
    assert!(first.is_absolute());
    assert!(second.is_absolute());
    assert_ne!(
        first, second,
        "linked worktrees must not share Cargo build products"
    );

    run_git(
        &root,
        &[
            "worktree",
            "remove",
            "--force",
            linked.to_str().expect("path"),
        ],
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn linked_worktree_cargo_test_binary_embeds_its_own_manifest_directory() {
    if run_without_inherited_cargo_target(
        "linked_worktree_cargo_test_binary_embeds_its_own_manifest_directory",
    ) {
        return;
    }
    let root = repository("cargo-manifest-path");
    let linked = root.with_extension("linked");
    fs::create_dir_all(root.join("src")).expect("crate source directory");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"manifest-path-fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("manifest");
    fs::write(
        root.join("src/lib.rs"),
        "#[test] fn manifest_path() { println!(\"MANIFEST_PATH={}\", env!(\"CARGO_MANIFEST_DIR\")); }\n",
    )
    .expect("fixture test");
    assert!(
        Command::new("cargo")
            .args(["generate-lockfile", "--offline"])
            .current_dir(&root)
            .status()
            .expect("generate lockfile")
            .success()
    );
    run_git(&root, &["add", "Cargo.toml", "Cargo.lock", "src/lib.rs"]);
    run_git(
        &root,
        &[
            "-c",
            "user.name=AI Cockpit Test",
            "-c",
            "user.email=ai-cockpit@example.invalid",
            "commit",
            "-qm",
            "tiny Cargo fixture",
        ],
    );
    run_git(
        &root,
        &[
            "worktree",
            "add",
            "--detach",
            linked.to_str().expect("path"),
        ],
    );

    for checkout in [&root, &linked] {
        let run = run_repository_verification(
            checkout,
            &request(
                "cargo",
                vec![
                    "test".into(),
                    "--offline".into(),
                    "--locked".into(),
                    "--".into(),
                    "--nocapture".into(),
                ],
                RepositoryVerificationPolicy::NeverReuse,
            ),
        )
        .expect("real Cargo test");
        assert!(
            run.receipt.passed,
            "Cargo verification failed for checkout {}:\n{}",
            checkout.display(),
            verification_receipt_diagnostics(&run.receipt)
        );
        let stdout = String::from_utf8(
            hex::decode(&run.receipt.execution_records[0].stdout_hex).expect("stdout hex"),
        )
        .expect("Cargo stdout");
        let canonical_checkout = fs::canonicalize(checkout).expect("canonical checkout");
        assert!(
            stdout.contains(&format!("MANIFEST_PATH={}", canonical_checkout.display())),
            "test binary must embed its own checkout, output: {stdout}"
        );
    }

    run_git(
        &root,
        &[
            "worktree",
            "remove",
            "--force",
            linked.to_str().expect("path"),
        ],
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn post_command_repository_drift_discards_the_passed_receipt_candidate() {
    let root = repository("drift");
    cockpit_repository::attach(&root).expect("attach");
    let args = vec![
        "-c".into(),
        "from pathlib import Path; Path('tracked.txt').write_text('after\\n')".into(),
    ];
    cockpit_repository::confirm_profile_update(&root, "python3", &args).expect("confirm");

    let run = run_repository_verification(
        &root,
        &request(
            "python3",
            args,
            RepositoryVerificationPolicy::ProfileAuthorized,
        ),
    )
    .expect("verify");

    assert!(run.receipt.passed);
    assert_eq!(run.receipt.processes_spawned, 1);
    assert!(run.receipt.receipt_candidates.is_empty());
    assert_eq!(run.receipt.results[0].receipt_id, None);
    assert_eq!(
        run.receipt.results[0].reason,
        "post_execution_binding_drift"
    );
    assert!(!root.join(".ai/evidence/reuse/index.json").exists());
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn protected_service_commands_execute_on_every_invocation() {
    let root = repository("protected");
    let args = vec![
        "-c".into(),
        "from pathlib import Path; p=Path('.counter'); p.write_text(str((int(p.read_text()) if p.exists() else 0)+1))".into(),
    ];
    let request = request(
        "python3",
        args,
        RepositoryVerificationPolicy::Protected(ProtectedGateClass::Scope),
    );

    let first = run_repository_verification(&root, &request).expect("first");
    let second = run_repository_verification(&root, &request).expect("second");

    assert_eq!(first.receipt.processes_spawned, 1);
    assert_eq!(second.receipt.processes_spawned, 1);
    assert_eq!(first.receipt.protected_nodes_executed, 1);
    assert_eq!(second.receipt.protected_nodes_executed, 1);
    assert_eq!(first.receipt.protected_nodes_skipped, 0);
    assert_eq!(second.receipt.protected_nodes_skipped, 0);
    assert_eq!(
        fs::read_to_string(root.join(".counter")).expect("counter"),
        "2"
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn missing_profile_executes_as_never_reuse_instead_of_blocking_verification() {
    let root = repository("missing-profile");
    let run = run_repository_verification(
        &root,
        &request(
            "true",
            vec![],
            RepositoryVerificationPolicy::ProfileAuthorized,
        ),
    )
    .expect("missing profile must only disable reuse");

    assert!(run.receipt.passed);
    assert_eq!(run.receipt.processes_spawned, 1);
    assert_eq!(run.receipt.nodes_reused, 0);
    assert_eq!(run.receipt.results[0].reason, "reuse_not_configured");
    assert!(run.receipt.receipt_candidates.is_empty());
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn verification_planning_rejects_invalid_stage_and_missing_release_base() {
    let root = repository("invalid-plan-input");
    let snapshot = cockpit_git::GitRepository::discover(&root)
        .expect("git repository")
        .snapshot()
        .expect("snapshot");
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_secs() as i64;

    let mut invalid_stage = request("true", vec![], RepositoryVerificationPolicy::NeverReuse);
    invalid_stage.stage = "not-a-verification-stage".into();
    assert!(
        plan_repository_verification_action(&root, &invalid_stage, &snapshot, now).is_err(),
        "plan-only must reject a stage that execution cannot run"
    );

    let mut missing_release_base =
        request("true", vec![], RepositoryVerificationPolicy::NeverReuse);
    missing_release_base.stage = "release".into();
    assert!(
        plan_repository_verification_action(&root, &missing_release_base, &snapshot, now).is_err(),
        "plan-only must reject a release route without its required base revision"
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn repository_receipt_projects_the_typed_route_stage() {
    let root = repository("route-stage");
    cockpit_repository::attach(&root).expect("attach");
    let mut route_request = request("true", vec![], RepositoryVerificationPolicy::NeverReuse);
    route_request.stage = "pre_ci".into();
    route_request.base_commit = Some(
        cockpit_git::GitRepository::discover(&root)
            .expect("git")
            .snapshot()
            .expect("snapshot")
            .head
            .expect("head"),
    );
    let run = run_repository_verification(&root, &route_request).expect("verify");
    let plan = run.receipt.plan_receipt.expect("route receipt");
    assert_eq!(plan.stage.as_str(), "pre_ci");
    assert_eq!(plan.initial_tier, cockpit_protocol::VerificationTier::T0);
    assert_eq!(
        plan.assurance,
        cockpit_protocol::EvidenceAssurance::SelfDeclared
    );
    assert_eq!(
        run.receipt.repository_id,
        Some(cockpit_repository::repository_id(&root).to_string())
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn never_reuse_execution_preserves_the_requested_executable_path() {
    use std::os::unix::fs::PermissionsExt;

    let root = repository("never-reuse-path");
    let tool = root.join("path-sensitive-tool.sh");
    fs::write(
        &tool,
        "#!/bin/sh\ntouch \"$(dirname \"$0\")/path-was-preserved\"\n",
    )
    .expect("script");
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("executable");
    let run = run_repository_verification(
        &root,
        &request(
            &tool.to_string_lossy(),
            vec![],
            RepositoryVerificationPolicy::NeverReuse,
        ),
    )
    .expect("verify");

    assert!(run.receipt.passed);
    assert!(root.join("path-was-preserved").is_file());
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn invalid_stage_fails_closed_before_reuse_or_identity_access() {
    let root = repository("early-denial-metrics");
    cockpit_repository::attach(&root).expect("attach");
    let mut profile_request = request(
        "true",
        vec![],
        RepositoryVerificationPolicy::ProfileAuthorized,
    );
    profile_request.stage = "invalid-stage".into();
    let mut never_request = profile_request.clone();
    never_request.policy = RepositoryVerificationPolicy::NeverReuse;

    // Stage parsing is an execution-boundary guard.  It must reject before
    // either reuse policy can inspect identity files or create a receipt.
    assert!(run_repository_verification(&root, &profile_request).is_err());
    assert!(run_repository_verification(&root, &never_request).is_err());
    assert!(!root.join(".ai/evidence/reuse/index.json").exists());
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn drift_after_an_initial_fresh_plan_forces_a_real_execution_fallback() {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::{sync::mpsc, time::Duration};

    let root = repository("fresh-drift");
    let tool = root
        .parent()
        .expect("fixture parent")
        .join(format!("cockpit-large-tool-{}", std::process::id()));
    let mut file = fs::File::create(&tool).expect("tool");
    file.write_all(b"#!/bin/sh\ncount=.counter; n=0; test -f $count && n=$(cat $count); echo $((n+1)) > $count\n")
        .expect("script");
    drop(file);
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("executable");
    cockpit_repository::attach(&root).expect("attach");
    let program = tool.to_string_lossy().into_owned();
    cockpit_repository::confirm_profile_update(&root, &program, &[]).expect("confirm");
    let request = request(
        &program,
        vec![],
        RepositoryVerificationPolicy::ProfileAuthorized,
    );
    let first = run_repository_verification(&root, &request).expect("first");
    assert_eq!(first.receipt.processes_spawned, 1);

    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(root.join(".ai/evidence/reuse/index.lock"))
        .expect("store lock");
    lock.lock().expect("hold store lock");
    let (started_tx, started_rx) = mpsc::channel();
    let worker_root = root.clone();
    let worker_request = request.clone();
    let worker = std::thread::spawn(move || {
        started_tx.send(()).expect("signal");
        run_repository_verification(&worker_root, &worker_request)
    });
    started_rx.recv().expect("started");
    std::thread::sleep(Duration::from_secs(1));
    fs::write(root.join("tracked.txt"), "drifted\n").expect("drift");
    lock.unlock().expect("release store lock");
    let second = worker.join().expect("verification thread").expect("second");

    assert_eq!(second.receipt.nodes_reused, 0);
    assert_eq!(second.receipt.processes_spawned, 1);
    assert_eq!(
        second.receipt.results[0].reason,
        "post_planning_binding_drift"
    );
    assert_eq!(
        fs::read_to_string(root.join(".counter"))
            .expect("counter")
            .trim(),
        "2"
    );
    fs::remove_file(tool).expect("cleanup tool");
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn formatting_only_config_change_between_calls_preserves_semantic_reuse() {
    use std::os::unix::fs::PermissionsExt;

    for identity_file in ["cockpit.toml", "project.json"] {
        let root = repository(&format!("between-call-identity-drift-{identity_file}"));
        let tool = root.parent().expect("fixture parent").join(format!(
            "cockpit-between-call-tool-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(
            &tool,
            b"#!/bin/sh\ncount=.counter; n=0; test -f $count && n=$(cat $count); echo $((n+1)) > $count\n",
        )
        .expect("script");
        fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("executable");
        cockpit_repository::attach(&root).expect("attach");
        let program = tool.to_string_lossy().into_owned();
        cockpit_repository::confirm_profile_update(&root, &program, &[]).expect("confirm");
        let request = request(
            &program,
            vec![],
            RepositoryVerificationPolicy::ProfileAuthorized,
        );

        let first = run_repository_verification(&root, &request).expect("first execution");
        assert_eq!(first.receipt.nodes_reused, 0, "{identity_file}");
        assert_eq!(first.receipt.processes_spawned, 1, "{identity_file}");
        assert!(first.receipt.results[0].passed, "{identity_file}");

        let unchanged = run_repository_verification(&root, &request).expect("unchanged reuse");
        assert_eq!(unchanged.receipt.nodes_reused, 1, "{identity_file}");
        assert_eq!(unchanged.receipt.processes_spawned, 0, "{identity_file}");
        assert_eq!(
            fs::read_to_string(root.join(".counter"))
                .expect("counter")
                .trim(),
            "1",
            "{identity_file}"
        );

        let identity_path = root.join(".ai").join(identity_file);
        let mut bytes = fs::read(&identity_path).expect("identity bytes");
        bytes.push(b'\n');
        fs::write(&identity_path, bytes).expect("drift identity bytes");
        let drifted = run_repository_verification(&root, &request).expect("format-only reuse");
        assert_eq!(drifted.receipt.nodes_reused, 1, "{identity_file}");
        assert_eq!(drifted.receipt.processes_spawned, 0, "{identity_file}");
        assert!(drifted.receipt.results[0].passed, "{identity_file}");
        assert_eq!(
            fs::read_to_string(root.join(".counter"))
                .expect("counter")
                .trim(),
            "1",
            "{identity_file}"
        );
        fs::remove_file(tool).expect("cleanup tool");
        fs::remove_dir_all(root).expect("cleanup repository");
    }
}

#[cfg(unix)]
#[test]
fn executable_path_swap_cannot_change_the_bytes_that_are_executed() {
    use std::os::unix::fs::PermissionsExt;
    use std::{sync::mpsc, time::Duration};

    let root = repository("pinned-executable");
    let suffix = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let fixture_parent = root.parent().expect("fixture parent");
    let tool = fixture_parent.join(format!(
        "cockpit-pinned-tool-{}-{suffix}",
        std::process::id()
    ));
    let backup = tool.with_extension("original");
    let malicious = tool.with_extension("replacement");
    let displaced = tool.with_extension("displaced");
    let armed = tool.with_extension("armed");
    let started = tool.with_extension("started");
    let proceed = tool.with_extension("proceed");
    let original = format!(
        "#!/bin/sh\nif test -f '{armed}'; then echo started > '{started}'; while ! test -f '{proceed}'; do sleep 0.01; done; fi\ncount=.counter; n=0; test -f $count && n=$(cat $count); echo $((n+1)) > $count\n",
        armed = armed.display(),
        started = started.display(),
        proceed = proceed.display(),
    );
    let replacement = format!(
        "#!/bin/sh\necho started > '{started}'\necho replacement > .counter\n",
        started = started.display(),
    );
    fs::write(&tool, original).expect("original tool");
    fs::write(&malicious, replacement).expect("replacement tool");
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("original executable");
    fs::set_permissions(&malicious, fs::Permissions::from_mode(0o755))
        .expect("replacement executable");

    cockpit_repository::attach(&root).expect("attach");
    let program = tool.to_string_lossy().into_owned();
    cockpit_repository::confirm_profile_update(&root, &program, &[]).expect("confirm");
    let request = request(
        &program,
        vec![],
        RepositoryVerificationPolicy::ProfileAuthorized,
    );
    let first = run_repository_verification(&root, &request).expect("first");
    assert!(first.receipt.passed);
    assert_eq!(first.receipt.receipt_candidates.len(), 1);

    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(root.join(".ai/evidence/reuse/index.lock"))
        .expect("store lock");
    lock.lock().expect("hold store lock");
    fs::write(&armed, b"armed\n").expect("arm original");
    let receipt_path = fs::read_dir(root.join(".ai/evidence/reuse/receipts"))
        .expect("receipt directory")
        .next()
        .expect("receipt entry")
        .expect("receipt path")
        .path();

    let (started_tx, started_rx) = mpsc::channel();
    let worker_root = root.clone();
    let worker_request = request.clone();
    let worker = std::thread::spawn(move || {
        started_tx.send(()).expect("signal");
        run_repository_verification(&worker_root, &worker_request)
    });
    started_rx.recv().expect("worker started");
    std::thread::sleep(Duration::from_secs(1));
    fs::rename(&tool, &backup).expect("preserve original path");
    fs::rename(&malicious, &tool).expect("replace executable path");
    fs::remove_file(receipt_path).expect("force execution after planning");
    lock.unlock().expect("release store lock");

    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    while !started.exists() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    let started_on_time = started.exists();
    if !started_on_time {
        // A slow host must not strand the spawned command when startup fails.
        fs::write(&proceed, b"proceed\n").expect("release delayed original command");
        let _ = worker.join().expect("join delayed verification thread");
        panic!("the pinned command must start");
    }
    fs::rename(&tool, &displaced).expect("remove replacement");
    fs::rename(&backup, &tool).expect("restore original path");
    fs::write(&proceed, b"proceed\n").expect("release original command");

    let second = worker.join().expect("verification thread").expect("second");
    assert!(second.receipt.passed);
    assert_eq!(second.receipt.processes_spawned, 1);
    assert_eq!(
        fs::read_to_string(root.join(".counter"))
            .expect("counter")
            .trim(),
        "2"
    );

    for path in [tool, displaced, armed, started, proceed] {
        fs::remove_file(path).expect("cleanup fixture");
    }
    fs::remove_dir_all(root).expect("cleanup repository");
}

#[cfg(unix)]
#[test]
fn fresh_plan_with_persistent_executable_drift_executes_and_receipts_new_bytes() {
    use std::os::unix::fs::PermissionsExt;
    use std::{sync::mpsc, time::Duration};

    let root = repository("persistent-executable-drift");
    fs::write(root.join(".gitignore"), ".counter\n.executed\n").expect("ignore execution log");
    run_git(&root, &["add", ".gitignore"]);
    run_git(
        &root,
        &[
            "-c",
            "user.name=AI Cockpit Test",
            "-c",
            "user.email=ai-cockpit@example.invalid",
            "commit",
            "--amend",
            "--no-edit",
            "-q",
        ],
    );
    let suffix = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let fixture_parent = root.parent().expect("fixture parent");
    let tool = fixture_parent.join(format!(
        "cockpit-drifting-tool-{}-{suffix}",
        std::process::id()
    ));
    let original = tool.with_extension("original");
    let replacement = tool.with_extension("replacement");
    fs::write(&tool, "#!/bin/sh\necho A >> .executed\n").expect("original tool");
    fs::write(
        &replacement,
        format!(
            "#!/bin/sh\nif test \"$0\" = '{}'; then echo UNPINNED >> .executed; else echo B >> .executed; fi\n",
            tool.display()
        ),
    )
    .expect("replacement tool");
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("original executable");
    fs::set_permissions(&replacement, fs::Permissions::from_mode(0o755))
        .expect("replacement executable");

    cockpit_repository::attach(&root).expect("attach");
    let program = tool.to_string_lossy().into_owned();
    cockpit_repository::confirm_profile_update(&root, &program, &[]).expect("confirm");
    let request = request(
        &program,
        vec![],
        RepositoryVerificationPolicy::ProfileAuthorized,
    );
    let first = run_repository_verification(&root, &request).expect("first");
    assert!(first.receipt.passed);
    assert_eq!(first.receipt.processes_spawned, 1);

    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(root.join(".ai/evidence/reuse/index.lock"))
        .expect("store lock");
    lock.lock().expect("hold store lock");
    let (started_tx, started_rx) = mpsc::channel();
    let worker_root = root.clone();
    let worker_request = request.clone();
    let worker = std::thread::spawn(move || {
        started_tx.send(()).expect("signal");
        run_repository_verification(&worker_root, &worker_request)
    });
    started_rx.recv().expect("worker started");
    std::thread::sleep(Duration::from_secs(1));
    fs::rename(&tool, &original).expect("preserve original");
    fs::rename(&replacement, &tool).expect("install replacement");
    lock.unlock().expect("release store lock");

    let second = worker.join().expect("verification thread").expect("second");
    assert!(second.receipt.passed);
    assert_eq!(second.receipt.nodes_reused, 0);
    assert_eq!(second.receipt.processes_spawned, 1);
    assert_eq!(
        second.receipt.results[0].reason,
        "post_planning_binding_drift"
    );
    assert_eq!(
        fs::read_to_string(root.join(".executed")).expect("execution log"),
        "A\nB\n"
    );

    let third = run_repository_verification(&root, &request).expect("third");
    assert!(third.receipt.passed);
    assert_eq!(third.receipt.nodes_reused, 1);
    assert_eq!(third.receipt.processes_spawned, 0);

    for path in [tool, original] {
        fs::remove_file(path).expect("cleanup fixture");
    }
    fs::remove_dir_all(root).expect("cleanup repository");
}

mod evidence_stage_admission_tests {
    use super::*;
    use cockpit_core::Digest;
    use cockpit_git::GitRepository;
    use cockpit_protocol::{RuntimeContext, VerificationStage};
    use cockpit_repository::{
        WorkItemStartOptions, amend_work_item_contract_with_runtime,
        archive_work_item_with_runtime, attach, checkpoint_work_item,
        close_work_item_with_decision_and_runtime, finish_work_item_with_runtime,
        preflight_work_item_with_runtime, record_verification_with_runtime,
        record_work_item_governance_controls, repository_id, require_verification_preconditions,
        start_work_item_with_options, validate_work_item_governance_controls_with_runtime,
        work_item_status_snapshot_with_runtime,
    };
    use serde_json::{Value, json};

    const ID: &str = "WI-EVIDENCE-STAGES";

    struct Fixture {
        root: PathBuf,
        runtime: RuntimeContext,
        contract: PathBuf,
        summary: PathBuf,
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).expect("remove isolated stage fixture");
        }
    }

    fn read(path: &Path) -> Value {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    }

    fn fixture(prerequisite: bool, opt_in: bool) -> Fixture {
        fixture_with_command(prerequisite, opt_in, "git diff --check")
    }

    fn fixture_with_command(prerequisite: bool, opt_in: bool, command: &str) -> Fixture {
        let root = fs::canonicalize(repository("evidence-stages")).unwrap();
        // This fixture deliberately scopes "**", including documentation. Supply
        // the real prearchive projection instead of bypassing that independent
        // gate. These are planned paths, not completed lifecycle evidence.
        fs::create_dir_all(root.join("docs/work-items")).unwrap();
        fs::create_dir_all(root.join("docs/reference")).unwrap();
        for suffix in ["", ".zh-CN", ".ja"] {
            fs::write(
                root.join(format!("docs/work-items/{ID}{suffix}.md")),
                format!(
                    "---\nstatus: in_progress\nworkItemId: {ID}\nlastVerifiedBy: {ID}\n---\n\n# Isolated stage fixture\n"
                ),
            )
            .unwrap();
            fs::write(
                root.join(format!("docs/reference/reference-parity{suffix}.md")),
                format!(
                    "# Reference parity\n\n| {ID} | In progress | [Work Item](../work-items/{ID}{suffix}.md) | planned terminal lifecycle: archive `.ai/work-items/archive/{ID}.contract.json`; verification `.ai/evidence/{ID}.verification.json`; close `.ai/decisions/{ID}.close.json` |\n"
                ),
            )
            .unwrap();
        }
        run_git(&root, &["add", "docs"]);
        run_git(
            &root,
            &[
                "-c",
                "user.name=AI Cockpit Test",
                "-c",
                "user.email=ai-cockpit@example.invalid",
                "commit",
                "-qm",
                "stage fixture documentation baseline",
            ],
        );
        attach(&root).unwrap();
        // The service rejects even a lower explicit timeout without a finite
        // policy ceiling. Authorize this small fixture's ten-second command.
        authorize_timeout(&root, 10);
        let runtime = RuntimeContext {
            runtime_version: "stage-fixture".into(),
            protocol_version: 1,
            runtime_digest: Digest::sha256_bytes(b"evidence-stage-fixture"),
        };
        let mut classes = vec!["verification".into(), "adopter".into()];
        if prerequisite {
            classes.push("inputs".into());
        }
        start_work_item_with_options(
            &root,
            ID,
            "allow source validation before final adoption",
            "preserve final evidence gates",
            &["**".into()],
            &WorkItemStartOptions {
                authority: "authorized".into(),
                risk: "low".into(),
                acceptance_criteria: vec!["final adoption remains required".into()],
                required_evidence_classes: classes,
                verification_commands: vec![command.into()],
                ..Default::default()
            },
        )
        .unwrap();
        let contract = root.join(format!(".ai/work-items/active/{ID}.contract.json"));
        let summary = root.join(format!(".ai/work-items/active/{ID}.summary.json"));
        if opt_in {
            let before = read(&contract);
            amend_work_item_contract_with_runtime(
                &root,
                ID,
                &json!({
                    "schemaVersion":1,"changeId":"staged-evidence-fixture",
                    "expectedContractDigest":cockpit_protocol::digest_json(&before).unwrap(),
                    "reason":"explicit final evidence boundary in isolated test fixture",
                    "changes":[{"path":"/governanceProfile","operation":"set","value":{
                        "evidenceClassStages":{"schemaVersion":1,"classes":[
                            {"class":"adopter","requiredFrom":"finish"}
                        ]}
                    }}]
                }),
                "",
                &runtime,
            )
            .unwrap();
        }
        record_work_item_governance_controls(
            &root,
            ID,
            &json!({
                "intentAlignment":{"state":"resolved","evidence":["tracked.txt"]}
            }),
        )
        .unwrap();
        Fixture {
            root,
            runtime,
            contract,
            summary,
        }
    }

    fn enter(fixture: &Fixture) {
        let decision =
            preflight_work_item_with_runtime(&fixture.root, &fixture.contract, &fixture.runtime)
                .unwrap();
        assert_ne!(
            decision.state,
            cockpit_core::DecisionState::Red,
            "{decision:?}"
        );
        let contract = read(&fixture.contract);
        let summary = read(&fixture.summary);
        record_work_item_governance_controls(&fixture.root, ID, &json!({"decisionEvidence":{
            "schemaVersion":1,"decisionId":"contract-preflight-review","decision":"confirm_review",
            "workItemId":ID,"repositoryId":repository_id(&fixture.root).to_string(),
            "contractDigest":cockpit_protocol::digest_json(&contract).unwrap(),
            "preflightDecisionDigest":summary["preflightDecisionDigest"],
            "repositorySnapshotDigest":summary["preflightRepositorySnapshotDigest"],
            "recordedAt":"2026-10-29T00:00:00Z","recordedBy":"human:test-fixture",
            "reason":"isolated test owner explicitly accepts this exact stage policy"
        }})).unwrap();
        checkpoint_work_item(&fixture.root, ID).unwrap();
    }

    fn execute(fixture: &Fixture) -> cockpit_repository::RepositoryVerificationRun {
        execute_args(fixture, &["diff", "--check"])
    }

    fn execute_args(
        fixture: &Fixture,
        args: &[&str],
    ) -> cockpit_repository::RepositoryVerificationRun {
        let snapshot = GitRepository::discover(&fixture.root)
            .unwrap()
            .snapshot()
            .unwrap();
        require_verification_preconditions(&fixture.root, ID, &fixture.runtime, &snapshot).unwrap();
        let route = cockpit_repository::resolve_verification_route(
            &fixture.root,
            ID,
            VerificationStage::Task,
            "local",
            &snapshot,
        )
        .unwrap();
        let mut command = request(
            "git",
            args.iter().map(|arg| (*arg).into()).collect(),
            RepositoryVerificationPolicy::NeverReuse,
        );
        command.work_item_id = Some(ID.into());
        command.runtime_digest = fixture.runtime.runtime_digest.to_string();
        command.timeout_seconds = Some(10);
        command.base_commit = route.base_revision.clone();
        let mut run = run_repository_verification(&fixture.root, &command).unwrap();
        // Like the CLI, bind the actual service execution to the native policy
        // route. The service-only receipt does not claim these WI bindings.
        // Preserve its results, exit status, execution records, and counters.
        let plan = run.receipt.plan_receipt.as_mut().unwrap();
        assert_eq!(plan.initial_tier, route.actual_tier);
        assert_eq!(plan.final_tier, route.actual_tier);
        assert_eq!(plan.assurance, EvidenceAssurance::SelfDeclared);
        plan.assurance = route.actual_assurance;
        plan.work_item_id = Some(route.work_item_id);
        plan.repository_id = Some(repository_id(&fixture.root).to_string());
        plan.repository_snapshot_digest = Some(
            cockpit_repository::snapshot_digest(&run.final_snapshot)
                .unwrap()
                .to_string(),
        );
        plan.base_revision = route.base_revision;
        plan.affected_paths = route.affected_paths;
        plan.dependency_confidence = Some(route.dependency_confidence);
        let policy = route
            .policy_plan
            .expect("fixture declares a timeout policy");
        plan.required_tier = Some(policy.requirement.required_tier);
        plan.required_assurance = Some(policy.requirement.required_assurance);
        plan.policy_refs = policy.requirement.policy_refs;
        run
    }

    fn input_projection(fixture: &Fixture) -> Value {
        let bytes = b"checked build inputs\n";
        fs::write(fixture.root.join(".ai/evidence/stage-input.txt"), bytes).unwrap();
        json!({"evidenceClasses":{
            "schemaVersion":1,
            "contractDigest":cockpit_protocol::digest_json(&read(&fixture.contract)).unwrap(),
            "items":[{"class":"inputs","evidence":[{
                "type":"fixture","path":".ai/evidence/stage-input.txt","locator":"all",
                "verification":"passed","digest":Digest::sha256_bytes(bytes)
            }]}]
        }})
    }

    fn record_inputs(fixture: &Fixture) {
        record_work_item_governance_controls(&fixture.root, ID, &input_projection(fixture))
            .expect("an explicit stage policy must permit complete prerequisites without inventing final evidence");
    }

    #[test]
    fn evidence_stages_native_execution_and_recording_do_not_complete_final_acceptance() {
        let fixture = fixture(false, true);
        // The real WI uses this representation: pending scenarios are
        // unverified; no evidenceClasses member is invented for them.
        assert!(read(&fixture.summary).get("evidenceClasses").is_none());
        enter(&fixture);
        let status =
            work_item_status_snapshot_with_runtime(&fixture.root, ID, &fixture.runtime).unwrap();
        assert!(
            status
                .safe_actions
                .iter()
                .any(|action| action == "run_verification"),
            "{status:?}"
        );
        let result = execute(&fixture);
        assert!(result.receipt.passed, "{:?}", result.receipt);
        assert_eq!(result.receipt.nodes_executed, 1);
        record_verification_with_runtime(
            &fixture.root,
            ID,
            &serde_json::to_value(&result.receipt).unwrap(),
            &fixture.runtime,
            &result.final_snapshot,
        )
        .unwrap();
        let controls = validate_work_item_governance_controls_with_runtime(
            &fixture.root,
            ID,
            &fixture.runtime,
        )
        .unwrap();
        assert_eq!(controls.evidence_classes, "unknown");
        assert!(
            controls
                .unknowns
                .iter()
                .any(|code| code == "evidence_classes_missing")
        );
        let error = finish_work_item_with_runtime(&fixture.root, ID, &fixture.runtime)
            .expect_err("source receipt cannot satisfy adopter evidence");
        assert!(error.to_string().contains("evidence_classes"), "{error}");
        assert_ne!(read(&fixture.summary)["state"], json!("finish_ready"));
        let summary_before = fs::read(&fixture.summary).unwrap();
        let archive_error = archive_work_item_with_runtime(&fixture.root, ID, &fixture.runtime)
            .expect_err("an unfinished Work Item cannot be archived");
        assert!(
            archive_error.to_string().contains("finish_ready"),
            "{archive_error}"
        );
        let close_error = close_work_item_with_decision_and_runtime(
            &fixture.root,
            ID,
            "approved",
            &fixture.runtime,
        )
        .expect_err("a synthetic fixture decision cannot close an unarchived Work Item");
        assert!(
            close_error.to_string().contains(".archive.json"),
            "{close_error}"
        );
        assert_eq!(fs::read(&fixture.summary).unwrap(), summary_before);
        assert!(
            !fixture
                .root
                .join(format!(".ai/decisions/{ID}.close.json"))
                .exists()
        );
        assert!(
            !fixture
                .root
                .join(format!(".ai/work-items/archive/{ID}.archive.json"))
                .exists()
        );
    }

    #[test]
    fn evidence_stages_unlisted_custom_requirement_still_blocks_before_spawn() {
        let fixture = fixture(false, false);
        enter(&fixture);
        let snapshot = GitRepository::discover(&fixture.root)
            .unwrap()
            .snapshot()
            .unwrap();
        let error =
            require_verification_preconditions(&fixture.root, ID, &fixture.runtime, &snapshot)
                .unwrap_err();
        assert!(
            error.to_string().contains("evidence_classes_missing"),
            "{error}"
        );
        assert!(
            !fixture
                .root
                .join(format!(".ai/evidence/{ID}.verification.json"))
                .exists()
        );
    }

    #[test]
    fn evidence_stages_archive_and_close_recheck_final_evidence() {
        let fixture = fixture(false, true);
        let bytes = b"isolated fixture final acceptance\n";
        let final_path = fixture.root.join(".ai/evidence/stage-adopter.txt");
        fs::write(&final_path, bytes).unwrap();
        record_work_item_governance_controls(
            &fixture.root,
            ID,
            &json!({"evidenceClasses":{
                "schemaVersion":1,
                "contractDigest":cockpit_protocol::digest_json(&read(&fixture.contract)).unwrap(),
                "items":[{"class":"adopter","evidence":[{
                    "type":"fixture","path":".ai/evidence/stage-adopter.txt","locator":"all",
                    "verification":"passed","digest":Digest::sha256_bytes(bytes)
                }]}]
            }}),
        )
        .unwrap();
        enter(&fixture);
        let result = execute(&fixture);
        assert!(result.receipt.passed);
        record_verification_with_runtime(
            &fixture.root,
            ID,
            &serde_json::to_value(&result.receipt).unwrap(),
            &fixture.runtime,
            &result.final_snapshot,
        )
        .unwrap();
        finish_work_item_with_runtime(&fixture.root, ID, &fixture.runtime).unwrap();
        assert_eq!(read(&fixture.summary)["state"], "finish_ready");

        fs::remove_file(&final_path).unwrap();
        let error = archive_work_item_with_runtime(&fixture.root, ID, &fixture.runtime)
            .expect_err("finish-only evidence must still exist at the real archive boundary");
        assert!(error.to_string().contains("governance"), "{error}");
        let controls = validate_work_item_governance_controls_with_runtime(
            &fixture.root,
            ID,
            &fixture.runtime,
        )
        .unwrap();
        assert!(
            controls
                .unknowns
                .iter()
                .any(|code| code == "evidence_classes_missing")
        );
        assert!(
            !fixture
                .root
                .join(format!(".ai/work-items/archive/{ID}.archive.json"))
                .exists()
        );

        fs::write(&final_path, bytes).unwrap();
        archive_work_item_with_runtime(&fixture.root, ID, &fixture.runtime).unwrap();
        let archive_path = fixture
            .root
            .join(format!(".ai/work-items/archive/{ID}.archive.json"));
        let archived = fs::read(&archive_path).unwrap();
        // Ordinary close binds cleanup to the exact native archive at HEAD.
        // Commit the generated artifacts without changing their bytes.
        run_git(
            &fixture.root,
            &[
                "add",
                "--",
                &format!(".ai/work-items/archive/{ID}.contract.json"),
                &format!(".ai/work-items/archive/{ID}.archive.json"),
            ],
        );
        run_git(
            &fixture.root,
            &[
                "-c",
                "user.name=AI Cockpit Test",
                "-c",
                "user.email=ai-cockpit@example.invalid",
                "commit",
                "-qm",
                "checkpoint native stage fixture archive",
            ],
        );
        assert_eq!(fs::read(&archive_path).unwrap(), archived);
        fs::remove_file(&final_path).unwrap();
        let error = close_work_item_with_decision_and_runtime(
            &fixture.root,
            ID,
            "approved",
            &fixture.runtime,
        )
        .expect_err("a fixture close decision cannot waive missing final evidence");
        assert!(error.to_string().contains("governance"), "{error}");
        let controls = validate_work_item_governance_controls_with_runtime(
            &fixture.root,
            ID,
            &fixture.runtime,
        )
        .unwrap();
        assert!(
            controls
                .unknowns
                .iter()
                .any(|code| code == "evidence_classes_missing")
        );
        assert_eq!(fs::read(&archive_path).unwrap(), archived);
        assert!(
            !fixture
                .root
                .join(format!(".ai/decisions/{ID}.close.json"))
                .exists()
        );
        fs::write(&final_path, bytes).unwrap();
        close_work_item_with_decision_and_runtime(&fixture.root, ID, "approved", &fixture.runtime)
            .unwrap();
        assert!(
            fixture
                .root
                .join(format!(".ai/decisions/{ID}.close.json"))
                .is_file()
        );
    }

    #[test]
    fn evidence_stages_native_prerequisite_recording_and_late_tamper_remain_bound() {
        let fixture = fixture(true, true);
        record_inputs(&fixture);
        enter(&fixture);
        let result = execute(&fixture);
        assert!(result.receipt.passed, "{:?}", result.receipt);
        assert_eq!(result.receipt.nodes_executed, 1);
        fs::write(
            fixture.root.join(".ai/evidence/stage-input.txt"),
            b"changed after execution\n",
        )
        .unwrap();
        let summary_before = fs::read(&fixture.summary).unwrap();
        let error = record_verification_with_runtime(
            &fixture.root,
            ID,
            &serde_json::to_value(&result.receipt).unwrap(),
            &fixture.runtime,
            &result.final_snapshot,
        )
        .expect_err("changed prerequisite must prevent publication");
        assert!(
            error.to_string().contains("evidence_classes_stale"),
            "{error}"
        );
        assert_eq!(fs::read(&fixture.summary).unwrap(), summary_before);
        assert!(
            !fixture
                .root
                .join(format!(".ai/evidence/{ID}.verification.json"))
                .exists()
        );
    }

    #[test]
    fn evidence_stages_recording_waiver_requires_known_pre_finish_state() {
        for state in [
            json!("implementation_active"),
            json!("checkpointed"),
            json!("finish_ready"),
            json!("finished"),
            json!("archived"),
            json!("closed"),
            json!("recovered"),
            json!("unknown"),
            Value::Null,
        ] {
            let fixture = fixture(true, true);
            let mut summary = read(&fixture.summary);
            summary["state"] = state.clone();
            // Isolated malformed-state fixture, never a real lifecycle receipt.
            fs::write(&fixture.summary, serde_json::to_vec(&summary).unwrap()).unwrap();
            let before = fs::read(&fixture.summary).unwrap();
            let result = record_work_item_governance_controls(
                &fixture.root,
                ID,
                &input_projection(&fixture),
            );
            if matches!(
                state.as_str(),
                Some("implementation_active" | "checkpointed")
            ) {
                result.unwrap();
                assert_eq!(read(&fixture.summary)["state"], state);
                assert_eq!(
                    read(&fixture.summary)["evidenceClasses"]["items"]
                        .as_array()
                        .unwrap()
                        .len(),
                    1
                );
            } else {
                let error =
                    result.expect_err("unknown or completed state must require all classes");
                assert!(
                    error
                        .to_string()
                        .contains("evidence class projection is not complete"),
                    "{state}: {error}"
                );
                assert_eq!(fs::read(&fixture.summary).unwrap(), before);
            }
        }
    }

    #[test]
    fn evidence_stages_terminal_marker_prevents_prerequisite_only_recording() {
        for relative in [
            format!(".ai/work-items/archive/{ID}.archive.json"),
            format!(".ai/decisions/{ID}.close.json"),
        ] {
            let fixture = fixture(true, true);
            let marker = fixture.root.join(relative);
            fs::create_dir_all(marker.parent().unwrap()).unwrap();
            fs::write(&marker, b"{}\n").unwrap();
            let before = fs::read(&fixture.summary).unwrap();
            let error = record_work_item_governance_controls(
                &fixture.root,
                ID,
                &input_projection(&fixture),
            )
            .expect_err(
                "even an invalid terminal marker cannot authorize early evidence recording",
            );
            assert!(
                error
                    .to_string()
                    .contains("evidence class projection is not complete"),
                "{error}"
            );
            assert_eq!(fs::read(&fixture.summary).unwrap(), before);
            assert_eq!(fs::read(marker).unwrap(), b"{}\n");
        }
    }

    #[test]
    fn evidence_stages_provided_pending_final_member_is_not_accepted() {
        for pending in [
            json!({"class":"adopter","evidence":[]}),
            json!({"class":"adopter","evidence":[{
                "type":"fixture","path":".ai/evidence/stage-input.txt","locator":"all",
                "verification":"unverified","digest":Digest::sha256_bytes(b"checked build inputs\n")
            }]}),
        ] {
            let fixture = fixture(true, true);
            let mut input = input_projection(&fixture);
            input["evidenceClasses"]["items"]
                .as_array_mut()
                .unwrap()
                .push(pending);
            let before = fs::read(&fixture.summary).unwrap();
            let error = record_work_item_governance_controls(&fixture.root, ID, &input)
                .expect_err("deferred members must be omitted until evidence is available");
            assert!(
                error
                    .to_string()
                    .contains("evidence class projection is not complete: Missing"),
                "{error}"
            );
            assert_eq!(fs::read(&fixture.summary).unwrap(), before);
            record_inputs(&fixture);
            enter(&fixture);
            let snapshot = GitRepository::discover(&fixture.root)
                .unwrap()
                .snapshot()
                .unwrap();
            require_verification_preconditions(&fixture.root, ID, &fixture.runtime, &snapshot)
                .unwrap();
            let controls = validate_work_item_governance_controls_with_runtime(
                &fixture.root,
                ID,
                &fixture.runtime,
            )
            .unwrap();
            assert!(
                controls
                    .unknowns
                    .iter()
                    .any(|code| code == "evidence_classes_missing")
            );
        }
    }

    #[test]
    fn evidence_stages_failed_source_execution_cannot_publish_or_finish() {
        let fixture = fixture_with_command(
            false,
            true,
            "git rev-parse --verify refs/heads/wi-stage-missing-reference",
        );
        enter(&fixture);
        let result = execute_args(
            &fixture,
            &[
                "rev-parse",
                "--verify",
                "refs/heads/wi-stage-missing-reference",
            ],
        );
        assert!(!result.receipt.passed, "{:?}", result.receipt);
        assert_eq!(result.receipt.nodes_executed, 1);
        assert_eq!(result.receipt.processes_spawned, 1);
        assert_eq!(result.receipt.execution_records.len(), 1);
        let executed = &result.receipt.execution_records[0];
        assert!(executed.spawned && !executed.passed && !executed.timed_out);
        assert!(executed.exit_code.is_some_and(|code| code != 0));
        let before = fs::read(&fixture.summary).unwrap();
        let error = record_verification_with_runtime(
            &fixture.root,
            ID,
            &serde_json::to_value(&result.receipt).unwrap(),
            &fixture.runtime,
            &result.final_snapshot,
        )
        .expect_err("a failed real source command cannot be completion evidence");
        assert!(
            error
                .to_string()
                .contains("failed verification cannot be recorded"),
            "{error}"
        );
        assert_eq!(fs::read(&fixture.summary).unwrap(), before);
        assert!(
            !fixture
                .root
                .join(format!(".ai/evidence/{ID}.verification.json"))
                .exists()
        );
        finish_work_item_with_runtime(&fixture.root, ID, &fixture.runtime)
            .expect_err("failed source execution cannot finish");
        let status =
            work_item_status_snapshot_with_runtime(&fixture.root, ID, &fixture.runtime).unwrap();
        assert!(!status.safe_actions.iter().any(|action| action == "finish"));
        assert_ne!(read(&fixture.summary)["state"], json!("finish_ready"));
    }

    // The actual predecessor rejection and zero-write assertions are mandatory in
    // conformance_evidence_stage_runtime_compatibility (the repository gate manifest).
}

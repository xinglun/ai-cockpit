use cockpit_core::Digest;
use cockpit_protocol::HumanDecision;
use cockpit_protocol::PROTOCOL_VERSION;
use cockpit_protocol::RuntimeContext;
use cockpit_repository::RepositoryVerificationPolicy;
use cockpit_repository::RepositoryVerificationRequest;
use cockpit_repository::WorkItemStartOptions;
use cockpit_repository::attach;
use cockpit_repository::checkpoint_work_item;
use cockpit_repository::close_work_item_with_structured_decision;
use cockpit_repository::preflight_work_item;
use cockpit_repository::record_verification_with_runtime;
use cockpit_repository::repository_id;
use cockpit_repository::run_repository_verification;
use cockpit_repository::start_work_item_with_options;
use cockpit_repository::work_item_status_snapshot_with_runtime;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
const WORK_ITEM_ID: &str = "WI-CROSS-CHECKOUT-CLOSEOUT";

fn runtime() -> RuntimeContext {
    let binary = fs::read(env!("CARGO_BIN_EXE_ai-cockpit")).expect("Runtime binary");
    RuntimeContext {
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        protocol_version: PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(&binary),
    }
}

fn command(root: &Path, program: &str, args: &[&str]) -> Output {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .expect("spawn command");
    assert!(
        output.status.success(),
        "{program} {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn git(root: &Path, args: &[&str]) -> Output {
    command(root, "git", args)
}

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("source repository");
    git(directory.path(), &["init", "--quiet"]);
    attach(directory.path()).expect("attach source repository");
    git(
        directory.path(),
        &["config", "user.name", "AI Cockpit Test"],
    );
    git(
        directory.path(),
        &["config", "user.email", "ai-cockpit-test@example.invalid"],
    );
    git(directory.path(), &["add", "-A"]);
    git(
        directory.path(),
        &["commit", "--quiet", "-m", "fixture repository baseline"],
    );
    directory
}

fn commit(root: &Path, message: &str) {
    git(root, &["add", "-A"]);
    git(root, &["commit", "--quiet", "-m", message]);
}

fn common_directory(root: &Path) -> PathBuf {
    let output = git(root, &["rev-parse", "--git-common-dir"]);
    let value = String::from_utf8(output.stdout).expect("common directory output");
    let path = PathBuf::from(value.trim());
    fs::canonicalize(if path.is_absolute() {
        path
    } else {
        root.join(path)
    })
    .expect("canonical common directory")
}

fn build_closed_source(source: &Path) {
    build_closed_source_with_runtime(source, &runtime());
}

fn build_closed_source_with_runtime(source: &Path, runtime: &RuntimeContext) {
    let options = WorkItemStartOptions {
        authority: "authorized".into(),
        acceptance_criteria: vec!["the exact closeout is recoverable".into()],
        ..WorkItemStartOptions::default()
    };
    start_work_item_with_options(
        source,
        WORK_ITEM_ID,
        "produce a valid archived closeout",
        "preserve its exact evidence in an independent destination checkout",
        &["src/**".into()],
        &options,
    )
    .expect("start source Work Item");
    let contract = source.join(format!(
        ".ai/work-items/active/{WORK_ITEM_ID}.contract.json"
    ));
    preflight_work_item(source, &contract).expect("source preflight");
    checkpoint_work_item(source, WORK_ITEM_ID).expect("source checkpoint");
    let request = RepositoryVerificationRequest {
        node_id: "closeout-source-verification".into(),
        program: "true".into(),
        args: Vec::new(),
        scope: vec!["**".into()],
        stage: "task".into(),
        runner: "local".into(),
        runtime_digest: runtime.runtime_digest.to_string(),
        base_commit: None,
        workers: 1,
        work_item_id: Some(WORK_ITEM_ID.into()),
        timeout_seconds: None,
        policy: RepositoryVerificationPolicy::NeverReuse,
    };
    let execution =
        run_repository_verification(source, &request).expect("execute source verification");
    let receipt = serde_json::to_value(execution.receipt).expect("verification receipt");
    record_verification_with_runtime(
        source,
        WORK_ITEM_ID,
        &receipt,
        runtime,
        &execution.final_snapshot,
    )
    .expect("source verification");
    cockpit_repository::finish_work_item_with_runtime(source, WORK_ITEM_ID, runtime)
        .expect("finish source Work Item");
    cockpit_repository::archive_work_item_with_runtime(source, WORK_ITEM_ID, runtime)
        .expect("archive source Work Item");
}

#[test]
fn recovery_plan_recognizes_a_closed_item_from_an_independent_checkout() {
    let source = repository();
    build_closed_source(source.path());
    commit(source.path(), "archive verified Work Item before closeout");

    let destination = tempfile::tempdir().expect("destination parent");
    let destination_path = destination.path().join("clone");
    git(
        source.path(),
        &[
            "clone",
            "--quiet",
            "--no-hardlinks",
            source.path().to_str().expect("source path"),
            destination_path.to_str().expect("destination path"),
        ],
    );
    assert_eq!(
        repository_id(source.path()),
        repository_id(&destination_path),
        "independent clones retain the same logical repository identity"
    );
    assert_ne!(
        common_directory(source.path()),
        common_directory(&destination_path),
        "the fixture must use independent Git common directories"
    );

    close_work_item_with_structured_decision(
        source.path(),
        WORK_ITEM_ID,
        &HumanDecision {
            decision: "approved".into(),
            actor: "test-human".into(),
            authority_source: "test fixture".into(),
            reason: "the verified source Work Item is closed".into(),
            evidence_refs: vec![format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")],
            policy_refs: Vec::new(),
            decided_at: "2026-10-02T00:00:00Z".into(),
            resume_condition: None,
        },
    )
    .expect("close source Work Item");
    assert_eq!(
        work_item_status_snapshot_with_runtime(source.path(), WORK_ITEM_ID, &runtime())
            .expect("source status")
            .verification,
        "verified",
        "fixture must prove a genuinely current source verification"
    );

    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args([
            "work-item",
            "closeout-recovery-plan",
            "--repo",
            destination_path.to_str().expect("destination path"),
            "--source-repo",
            source.path().to_str().expect("source path"),
            "--id",
            WORK_ITEM_ID,
        ])
        .output()
        .expect("run closeout recovery plan");
    assert!(
        output.status.success(),
        "a valid source closeout should produce a read-only recovery plan; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: Value = serde_json::from_slice(&output.stdout).expect("recovery plan JSON");
    assert_eq!(plan["allowed"], true, "{plan}");
    assert_eq!(plan["workItemId"], WORK_ITEM_ID);
    assert_eq!(
        plan["sourceRepositoryId"],
        repository_id(source.path()).to_string()
    );
    assert_eq!(
        plan["destinationRepositoryId"],
        repository_id(&destination_path).to_string()
    );
}

#[test]
fn recovery_imports_exact_closeout_and_is_idempotent_without_mutating_source() {
    let source = repository();
    build_closed_source(source.path());
    commit(source.path(), "archive verified Work Item before closeout");

    let destination = tempfile::tempdir().expect("destination parent");
    let destination_path = destination.path().join("clone");
    git(
        source.path(),
        &[
            "clone",
            "--quiet",
            "--no-hardlinks",
            source.path().to_str().expect("source path"),
            destination_path.to_str().expect("destination path"),
        ],
    );
    close_work_item_with_structured_decision(
        source.path(),
        WORK_ITEM_ID,
        &HumanDecision {
            decision: "approved".into(),
            actor: "test-human".into(),
            authority_source: "test fixture".into(),
            reason: "the verified source Work Item is closed".into(),
            evidence_refs: vec![format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")],
            policy_refs: Vec::new(),
            decided_at: "2026-10-02T00:00:00Z".into(),
            resume_condition: None,
        },
    )
    .expect("close source Work Item");

    let source_snapshot_before = cockpit_git::GitRepository::discover(source.path())
        .expect("discover source")
        .snapshot()
        .expect("source snapshot before recovery");
    let close_path = source
        .path()
        .join(".ai/decisions")
        .join(format!("{WORK_ITEM_ID}.close.json"));
    let close_bytes = fs::read(&close_path).expect("source close decision");

    let recover = || {
        Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
            .args([
                "work-item",
                "closeout-recover",
                "--repo",
                destination_path.to_str().expect("destination path"),
                "--source-repo",
                source.path().to_str().expect("source path"),
                "--id",
                WORK_ITEM_ID,
            ])
            .output()
            .expect("run closeout recovery")
    };

    let first = recover();
    assert!(
        first.status.success(),
        "valid closeout recovery should succeed; stderr={}",
        String::from_utf8_lossy(&first.stderr)
    );
    let first_receipt: Value = serde_json::from_slice(&first.stdout).expect("first result JSON");
    assert_eq!(first_receipt["state"], "recovered", "{first_receipt}");
    assert_eq!(
        fs::read(
            destination_path
                .join(".ai/decisions")
                .join(format!("{WORK_ITEM_ID}.close.json"))
        )
        .expect("destination close decision"),
        close_bytes,
        "recovery must install the exact source close decision bytes"
    );
    let destination_status =
        work_item_status_snapshot_with_runtime(&destination_path, WORK_ITEM_ID, &runtime())
            .expect("destination closeout status");
    assert_eq!(destination_status.lifecycle_phase, "closed");
    assert_eq!(destination_status.verification, "verified");

    let destination_snapshot_after_first = cockpit_git::GitRepository::discover(&destination_path)
        .expect("discover destination")
        .snapshot()
        .expect("destination snapshot after first recovery");
    let second = recover();
    assert!(
        second.status.success(),
        "replaying identical closeout recovery should succeed; stderr={}",
        String::from_utf8_lossy(&second.stderr)
    );
    let second_receipt: Value = serde_json::from_slice(&second.stdout).expect("second result JSON");
    assert_eq!(second_receipt["state"], "recovered", "{second_receipt}");
    assert_eq!(
        second_receipt["changedPaths"],
        serde_json::json!([]),
        "an identical replay must be a no-op"
    );
    let destination_snapshot_after_second = cockpit_git::GitRepository::discover(&destination_path)
        .expect("discover destination")
        .snapshot()
        .expect("destination snapshot after second recovery");
    assert_eq!(
        cockpit_repository::snapshot_digest(&destination_snapshot_after_first)
            .expect("first destination digest"),
        cockpit_repository::snapshot_digest(&destination_snapshot_after_second)
            .expect("second destination digest"),
        "idempotent replay must not change destination facts"
    );
    let source_snapshot_after = cockpit_git::GitRepository::discover(source.path())
        .expect("discover source")
        .snapshot()
        .expect("source snapshot after recovery");
    assert_eq!(
        cockpit_repository::snapshot_digest(&source_snapshot_before)
            .expect("source digest before recovery"),
        cockpit_repository::snapshot_digest(&source_snapshot_after)
            .expect("source digest after recovery"),
        "recovery must not mutate its source checkout"
    );
}

#[test]
fn cli_required_receipt_loss_revokes_closed_without_promoting_historical_verification() {
    let source = repository();
    let mut producer = runtime();
    producer.runtime_version = "0.2.74".into();
    producer.runtime_digest = Digest::sha256_bytes(b"older producing Runtime");
    build_closed_source_with_runtime(source.path(), &producer);
    commit(source.path(), "archive older producing Runtime evidence");
    let parent = tempfile::tempdir().unwrap();
    let destination = parent.path().join("clone");
    git(
        source.path(),
        &[
            "clone",
            "--quiet",
            "--no-hardlinks",
            source.path().to_str().unwrap(),
            destination.to_str().unwrap(),
        ],
    );
    cockpit_repository::close_work_item_with_structured_decision_and_runtime(
        source.path(),
        WORK_ITEM_ID,
        &HumanDecision {
            decision: "approved".into(),
            actor: "test-human".into(),
            authority_source: "CLI historical fixture".into(),
            reason: "verified by original producer".into(),
            evidence_refs: vec![],
            policy_refs: vec![],
            decided_at: "2026-10-02T00:00:00Z".into(),
            resume_condition: None,
        },
        &producer,
    )
    .unwrap();
    let evidence = format!(".ai/evidence/{WORK_ITEM_ID}.verification.json");
    let original = fs::read(source.path().join(&evidence)).unwrap();
    let recover = || {
        command(
            &destination,
            env!("CARGO_BIN_EXE_ai-cockpit"),
            &[
                "work-item",
                "closeout-recover",
                "--repo",
                destination.to_str().unwrap(),
                "--source-repo",
                source.path().to_str().unwrap(),
                "--id",
                WORK_ITEM_ID,
            ],
        )
    };
    let output = recover();
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["verification"], "not_ready");
    let status = || -> Value {
        let out = command(
            &destination,
            env!("CARGO_BIN_EXE_ai-cockpit"),
            &[
                "work-item",
                "status",
                "--repo",
                destination.to_str().unwrap(),
                "--id",
                WORK_ITEM_ID,
                "--json",
            ],
        );
        serde_json::from_slice(&out.stdout).unwrap()
    };
    let before = status();
    assert_eq!(before["lifecyclePhase"], "closed");
    assert_eq!(before["blocking"], false);
    assert_eq!(before["verification"], "not_ready");
    assert!(
        before["unknowns"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("historical_evidence_not_revalidated"))
    );
    let origin = destination.join(format!(
        ".ai/decisions/{WORK_ITEM_ID}.cross-checkout-closeout-origin.json"
    ));
    let origin_bytes = fs::read(&origin).unwrap();
    fs::remove_file(destination.join(format!(
        ".ai/decisions/{WORK_ITEM_ID}.cross-checkout-closeout-recovery.json"
    )))
    .unwrap();
    let after = status();
    assert_eq!(after["lifecyclePhase"], "archived");
    assert_eq!(after["blocking"], true);
    assert_eq!(after["verification"], "not_ready");
    assert!(
        after["unknowns"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("close_decision_invalid"))
    );
    assert_eq!(fs::read(&origin).unwrap(), origin_bytes);
    recover();
    assert_eq!(status()["lifecyclePhase"], "closed");
    assert_eq!(fs::read(destination.join(&evidence)).unwrap(), original);
    assert_eq!(fs::read(source.path().join(&evidence)).unwrap(), original);
}

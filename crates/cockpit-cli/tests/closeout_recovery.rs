use cockpit_core::Digest;
use cockpit_protocol::{
    HumanDecision, PROTOCOL_VERSION, ResourceFinalizationContext, RuntimeContext,
};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_work_item, attach, checkpoint_work_item, close_work_item_with_structured_decision,
    finish_work_item, plan_cross_checkout_closeout_recovery,
    plan_resource_finalization_with_runtime, preflight_work_item, record_verification_with_runtime,
    recover_cross_checkout_closeout, repository_id, run_repository_verification,
    start_work_item_with_options, work_item_status_snapshot_with_runtime,
};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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
    let runtime = runtime();
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
        &runtime,
        &execution.final_snapshot,
    )
    .expect("source verification");
    finish_work_item(source, WORK_ITEM_ID).expect("finish source Work Item");
    archive_work_item(source, WORK_ITEM_ID).expect("archive source Work Item");
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
fn recovery_preserves_a_closed_historical_projection_without_promoting_verification() {
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
            reason: "the original close remains historical evidence".into(),
            evidence_refs: vec![format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")],
            policy_refs: Vec::new(),
            decided_at: "2026-10-02T00:00:00Z".into(),
            resume_condition: None,
        },
    )
    .expect("close source Work Item");

    let mut historical_runtime = runtime();
    historical_runtime.runtime_version = "0.2.74".into();
    historical_runtime.runtime_digest = Digest::sha256_bytes(b"historical-runtime");
    let source_status =
        work_item_status_snapshot_with_runtime(source.path(), WORK_ITEM_ID, &historical_runtime)
            .expect("historical source status");
    assert_eq!(source_status.lifecycle_phase, "closed");
    assert_eq!(source_status.verification, "not_ready");
    assert!(!source_status.blocking);
    assert!(!source_status.human_decision_required);

    let plan = plan_cross_checkout_closeout_recovery(
        &destination_path,
        source.path(),
        WORK_ITEM_ID,
        &historical_runtime,
    )
    .expect("historical closeout remains transportable without promotion");
    assert_eq!(plan["sourceStatus"]["lifecyclePhase"], "closed");
    assert_eq!(plan["sourceStatus"]["verification"], "not_ready");
    assert_eq!(
        plan["sourceStatus"]["unknowns"],
        serde_json::to_value(&source_status.unknowns).expect("source unknowns JSON")
    );

    let result = recover_cross_checkout_closeout(
        &destination_path,
        source.path(),
        WORK_ITEM_ID,
        &historical_runtime,
    )
    .expect("recover exact historical closeout");
    assert_eq!(result["verification"], "not_ready", "{result}");
    assert_eq!(
        result["sourceStatus"], result["destinationStatus"],
        "recovery must preserve the whole exposed historical projection"
    );
    let destination_status = work_item_status_snapshot_with_runtime(
        &destination_path,
        WORK_ITEM_ID,
        &historical_runtime,
    )
    .expect("destination status");
    assert_eq!(destination_status.lifecycle_phase, "closed");
    assert_eq!(destination_status.verification, "not_ready");
    assert_eq!(destination_status.unknowns, source_status.unknowns);
}

#[test]
fn tampered_historical_close_report_is_rejected_before_destination_writes() {
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

    let close_path = source
        .path()
        .join(".ai/decisions")
        .join(format!("{WORK_ITEM_ID}.close.json"));
    let mut close: Value = serde_json::from_slice(&fs::read(&close_path).expect("read close"))
        .expect("close decision JSON");
    close["finalReport"]["status"] = Value::String("not_verified".into());
    fs::write(
        &close_path,
        serde_json::to_vec_pretty(&close).expect("serialize tampered close decision"),
    )
    .expect("tamper source close report for negative test");

    let error =
        recover_cross_checkout_closeout(&destination_path, source.path(), WORK_ITEM_ID, &runtime())
            .expect_err("tampered historical verification must fail closed");
    assert!(
        error
            .to_string()
            .contains("closeout_recovery_close_report_invalid"),
        "unexpected rejection: {error}"
    );
    assert!(
        !destination_path
            .join(".ai/decisions")
            .join(format!("{WORK_ITEM_ID}.close.json"))
            .exists(),
        "close decision must not be installed on rejection"
    );
    assert!(
        !destination_path
            .join(".ai/decisions")
            .join(format!(
                "{WORK_ITEM_ID}.cross-checkout-closeout-recovery.json"
            ))
            .exists(),
        "recovery receipt must not be written on rejection"
    );
}

#[test]
fn archived_provider_context_binding_requires_its_finalization_receipt() {
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
    let context = ResourceFinalizationContext {
        branch: "codex/provider-bound-closeout".into(),
        worktree: source.path().display().to_string(),
        base_branch: "develop".into(),
        base_remote: "origin".into(),
        provider: "github".into(),
        pull_request: "https://github.com/example/sentinel/pull/123".into(),
    };
    plan_resource_finalization_with_runtime(source.path(), WORK_ITEM_ID, &context, &runtime())
        .expect("append provider context for the historical archive");

    let result = plan_cross_checkout_closeout_recovery(
        &destination_path,
        source.path(),
        WORK_ITEM_ID,
        &runtime(),
    );
    assert!(
        result.is_err(),
        "provider-bound archive without a finalization receipt must not be recoverable"
    );
    assert!(
        !destination_path
            .join(".ai/decisions")
            .join(format!("{WORK_ITEM_ID}.close.json"))
            .exists(),
        "a read-only plan rejection must leave the destination untouched"
    );
}

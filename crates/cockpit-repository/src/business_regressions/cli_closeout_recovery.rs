//! Library-domain regressions migrated from CLI fixtures; no CLI transport authority is supplied.
#![cfg(test)]
use super::approved_boundary_fixtures as boundary;
use crate as cockpit_repository;
use crate::review_authority::pipeline_tests::TestAuthority;
use cockpit_core::Digest;
use cockpit_protocol::HumanDecision;
use cockpit_protocol::PROTOCOL_VERSION;
use cockpit_protocol::ResourceFinalizationContext;
use cockpit_protocol::RuntimeContext;
use cockpit_repository::RepositoryVerificationPolicy;
use cockpit_repository::RepositoryVerificationRequest;
use cockpit_repository::WorkItemStartOptions;
use cockpit_repository::attach;
use cockpit_repository::checkpoint_work_item;
use cockpit_repository::plan_cross_checkout_closeout_recovery;
use cockpit_repository::plan_resource_finalization_with_runtime;
use cockpit_repository::preflight_work_item;
use cockpit_repository::record_verification_with_runtime;
use cockpit_repository::recover_cross_checkout_closeout;
use cockpit_repository::run_repository_verification;
use cockpit_repository::start_work_item_with_options;
use cockpit_repository::work_item_status_snapshot_with_runtime;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::process::Output;
const WORK_ITEM_ID: &str = "WI-CROSS-CHECKOUT-CLOSEOUT";

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        protocol_version: PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(
            b"private native domain fixture runtime; not CLI transport",
        ),
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

fn build_closed_source(authority: &TestAuthority, source: &Path) {
    build_closed_source_with_runtime(authority, source, &runtime());
}

fn build_closed_source_with_runtime(
    _authority: &TestAuthority,
    source: &Path,
    runtime: &RuntimeContext,
) {
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
fn recovery_preserves_a_closed_historical_projection_without_promoting_verification() {
    let authority = TestAuthority::install();
    let source = repository();
    build_closed_source(&authority, source.path());
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
    boundary::close_structured(
        &authority,
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

    boundary::approve_initial_cross_checkout(
        &authority,
        &destination_path,
        source.path(),
        WORK_ITEM_ID,
        &historical_runtime,
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
    let authority = TestAuthority::install();
    let source = repository();
    build_closed_source(&authority, source.path());
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
    boundary::close_structured(
        &authority,
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

    boundary::approve_initial_cross_checkout(
        &authority,
        &destination_path,
        source.path(),
        WORK_ITEM_ID,
        &runtime(),
    );
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
    let authority = TestAuthority::install();
    let source = repository();
    build_closed_source(&authority, source.path());
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
    boundary::close_structured(
        &authority,
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

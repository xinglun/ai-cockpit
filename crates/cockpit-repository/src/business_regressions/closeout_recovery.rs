//! Native domain fixtures use an explicit private test port, never a production source.
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
use cockpit_repository::finish_work_item_with_runtime;
use cockpit_repository::plan_cross_checkout_closeout_recovery;
use cockpit_repository::plan_resource_finalization_with_runtime;
use cockpit_repository::record_resource_finalization;
use cockpit_repository::record_verification_with_runtime;
use cockpit_repository::repository_id;
use cockpit_repository::run_repository_verification;
use cockpit_repository::start_work_item_with_options;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
const WORK_ITEM_ID: &str = "WI-REPOSITORY-CROSS-CHECKOUT-CLOSEOUT";

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "1.0.0-repository-closeout-test".into(),
        protocol_version: PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(b"repository-closeout-recovery-test-runtime"),
    }
}

fn command(root: &Path, program: &str, args: &[&str]) -> std::process::Output {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .expect("run command");
    assert!(
        output.status.success(),
        "{program} {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn git(root: &Path, args: &[&str]) {
    command(root, "git", args);
}

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("source repository");
    git(directory.path(), &["init", "--quiet"]);
    attach(directory.path()).expect("attach source repository");
    git(
        directory.path(),
        &["config", "user.name", "Repository Closeout Test"],
    );
    git(
        directory.path(),
        &[
            "config",
            "user.email",
            "repository-closeout-test@example.invalid",
        ],
    );
    fs::write(directory.path().join("seed.txt"), "baseline\n").expect("write baseline");
    git(directory.path(), &["add", "-A"]);
    git(
        directory.path(),
        &["commit", "--quiet", "-m", "fixture baseline"],
    );
    directory
}

fn build_archived_source(source: &Path) {
    let runtime = runtime();
    start_work_item_with_options(
        source,
        WORK_ITEM_ID,
        "preserve validated closeout across checkouts",
        "reject provider-bound history unless its archived binding and receipt agree",
        &[".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["invalid provider closeout never authorizes recovery".into()],
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
        node_id: "repository-closeout-source-verification".into(),
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
    .expect("record verification");
    finish_work_item_with_runtime(source, WORK_ITEM_ID, &runtime).expect("finish source");
    cockpit_repository::archive_work_item_with_runtime(source, WORK_ITEM_ID, &runtime)
        .expect("archive source");
    git(source, &["add", "-A"]);
    git(
        source,
        &["commit", "--quiet", "-m", "archive verified Work Item"],
    );
}

fn build_provider_bound_closed_source(
    authority: &TestAuthority,
    source: &Path,
    context: &ResourceFinalizationContext,
) -> (Digest, Digest) {
    let runtime = runtime();
    start_work_item_with_options(
        source,
        WORK_ITEM_ID,
        "preserve validated closeout across checkouts",
        "validate the historical provider receipt against the archived Contract bytes",
        &[".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec![
                "raw finalization binding stays distinct from canonical plan binding".into(),
            ],
            ..Default::default()
        },
    )
    .expect("start provider-bound source Work Item");
    plan_resource_finalization_with_runtime(source, WORK_ITEM_ID, context, &runtime)
        .expect("bind provider resource context before verification");
    let contract = source.join(format!(
        ".ai/work-items/active/{WORK_ITEM_ID}.contract.json"
    ));
    cockpit_repository::preflight_work_item(source, &contract).expect("preflight");
    checkpoint_work_item(source, WORK_ITEM_ID).expect("checkpoint");
    let request = RepositoryVerificationRequest {
        node_id: "provider-bound-closeout-source-verification".into(),
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
    .expect("record verification");
    finish_work_item_with_runtime(source, WORK_ITEM_ID, &runtime).expect("finish source");
    cockpit_repository::archive_work_item_with_runtime(source, WORK_ITEM_ID, &runtime)
        .expect("archive source");

    let contract_path = source.join(format!(
        ".ai/work-items/archive/{WORK_ITEM_ID}.contract.json"
    ));
    let contract_bytes = fs::read(&contract_path).expect("archived Contract bytes");
    let contract: Value = serde_json::from_slice(&contract_bytes).expect("archived Contract JSON");
    let raw_contract_digest = Digest::sha256_bytes(&contract_bytes);
    let canonical_contract_digest =
        cockpit_protocol::digest_json(&contract).expect("canonical Contract digest");
    assert_ne!(
        raw_contract_digest, canonical_contract_digest,
        "fixture must distinguish raw-byte and canonical-JSON Contract digests"
    );
    let head = String::from_utf8(command(source, "git", &["rev-parse", "HEAD"]).stdout)
        .expect("git head")
        .trim()
        .to_owned();
    let receipt = serde_json::json!({
        "schemaVersion": 1,
        "receiptId": "provider-bound-closeout-receipt",
        "operationId": "provider-bound-closeout-operation",
        "repositoryId": repository_id(source).to_string(),
        "workItemId": WORK_ITEM_ID,
        "runtimeVersion": runtime.runtime_version,
        "runtimeDigest": runtime.runtime_digest.to_string(),
        "provider": context.provider,
        "pullRequest": {
            "number": 123,
            "url": context.pull_request,
            "headRevision": head,
            "baseBranch": context.base_branch,
            "baseRemote": context.base_remote,
            "baseRevision": contract["baseRevision"],
            "mergeCommit": head
        },
        "branch": {
            "name": context.branch,
            "remote": context.base_remote,
            "headRevision": head
        },
        "worktree": {
            "worktreeId": "provider-bound-closeout-worktree",
            "path": context.worktree,
            "branch": context.branch,
            "headRevision": head
        },
        "before": {"pullRequest": "merged", "branch": "present", "worktree": "clean"},
        "after": {"pullRequest": "merged", "branch": "deleted", "worktree": "removed"},
        "result": {"disposition": "deleted", "failureCodes": [], "unknownCodes": []},
        "actor": "human:test",
        "authoritySource": "repository closeout integration test",
        "reason": "record a merged provider resource without deleting retained resources",
        "timestamp": "2026-10-02T00:00:00Z",
        "contractDigest": raw_contract_digest,
        "contractBaseRevision": contract["baseRevision"],
        "resourceContext": context
    });
    let finalization_input = tempfile::NamedTempFile::new().expect("finalization input");
    fs::write(
        finalization_input.path(),
        serde_json::to_vec_pretty(&receipt).expect("finalization receipt JSON"),
    )
    .expect("write finalization receipt");
    record_resource_finalization(source, WORK_ITEM_ID, finalization_input.path(), &runtime)
        .expect("record raw-byte-bound provider finalization");
    boundary::close_structured_runtime(
        authority,
        source,
        WORK_ITEM_ID,
        &HumanDecision {
            decision: "approved".into(),
            actor: "test-human".into(),
            authority_source: "repository closeout integration test".into(),
            reason: "the source verification and provider merge are recorded".into(),
            evidence_refs: vec![format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")],
            policy_refs: Vec::new(),
            decided_at: "2026-10-02T00:01:00Z".into(),
            resume_condition: None,
        },
        &runtime,
    )
    .expect("close provider-bound source Work Item");
    git(source, &["add", "-A"]);
    git(
        source,
        &["commit", "--quiet", "-m", "close provider-bound Work Item"],
    );
    (raw_contract_digest, canonical_contract_digest)
}

#[test]
fn archived_provider_binding_without_a_matching_finalize_receipt_is_rejected_read_only() {
    let authority = TestAuthority::install();
    let source = repository();
    build_archived_source(source.path());

    let destination_parent = tempfile::tempdir().expect("destination parent");
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

    boundary::close_structured_runtime(
        &authority,
        source.path(),
        WORK_ITEM_ID,
        &HumanDecision {
            decision: "approved".into(),
            actor: "test-human".into(),
            authority_source: "repository closeout integration test".into(),
            reason: "the historical source closeout was verified".into(),
            evidence_refs: vec![format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")],
            policy_refs: Vec::new(),
            decided_at: "2026-10-02T00:00:00Z".into(),
            resume_condition: None,
        },
        &runtime(),
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
        .expect("append archived resource-context binding");

    let result = plan_cross_checkout_closeout_recovery(
        &destination,
        source.path(),
        WORK_ITEM_ID,
        &runtime(),
    );
    assert!(
        result.is_err(),
        "Contract.resourceContext=null must not cause the archived provider binding to be skipped"
    );
    assert!(
        !destination
            .join(".ai/decisions")
            .join(format!("{WORK_ITEM_ID}.close.json"))
            .exists(),
        "a failed read-only plan leaves the destination untouched"
    );
    let source_context: Value = serde_json::from_slice(
        &fs::read(
            source
                .path()
                .join(".ai/decisions")
                .join(format!("{WORK_ITEM_ID}.resource-context.json")),
        )
        .expect("archived resource context binding"),
    )
    .expect("resource context JSON");
    assert_eq!(source_context["workItemId"], WORK_ITEM_ID);
}

#[test]
fn provider_finalization_uses_raw_contract_bytes_while_recovery_plan_keeps_canonical_binding() {
    let authority = TestAuthority::install();
    let source = repository();
    let context = ResourceFinalizationContext {
        branch: "feature/provider-bound-closeout".into(),
        worktree: source
            .path()
            .join("removed-provider-bound-worktree")
            .display()
            .to_string(),
        base_branch: "develop".into(),
        base_remote: "origin".into(),
        provider: "github".into(),
        pull_request: "https://github.com/example/sentinel/pull/123".into(),
    };
    let (raw_contract_digest, canonical_contract_digest) =
        build_provider_bound_closed_source(&authority, source.path(), &context);

    let destination_parent = tempfile::tempdir().expect("destination parent");
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
    let destination_close = destination
        .join(".ai/decisions")
        .join(format!("{WORK_ITEM_ID}.close.json"));
    let destination_close_before = fs::read(&destination_close).expect("copied close receipt");

    let plan = plan_cross_checkout_closeout_recovery(
        &destination,
        source.path(),
        WORK_ITEM_ID,
        &runtime(),
    )
    .expect("raw-byte historical finalization binding is accepted");

    assert_eq!(plan["state"], "ready");
    assert_eq!(plan["allowed"], true);
    assert_eq!(
        plan["contractDigest"],
        canonical_contract_digest.to_string()
    );
    assert_ne!(raw_contract_digest, canonical_contract_digest);
    assert!(
        fs::read(&destination_close).expect("close receipt after read-only plan")
            == destination_close_before,
        "planning remains read-only"
    );
    boundary::approve_initial_cross_checkout(
        &authority,
        &destination,
        source.path(),
        WORK_ITEM_ID,
        &runtime(),
    );
    cockpit_repository::recover_cross_checkout_closeout(
        &destination,
        source.path(),
        WORK_ITEM_ID,
        &runtime(),
    )
    .expect("recover actual provider-bound close");
    let status = cockpit_repository::work_item_status_snapshot_with_runtime(
        &destination,
        WORK_ITEM_ID,
        &runtime(),
    )
    .unwrap();
    assert_eq!(status.lifecycle_phase, "closed");
    assert!(!status.blocking);
    assert_eq!(
        fs::read(&destination_close).unwrap(),
        destination_close_before
    );
    let origin_path = destination.join(format!(
        ".ai/decisions/{WORK_ITEM_ID}.cross-checkout-closeout-origin.json"
    ));
    let origin: Value = serde_json::from_slice(&fs::read(&origin_path).unwrap()).unwrap();
    assert_eq!(
        origin["contractDigest"],
        canonical_contract_digest.to_string()
    );
    assert_eq!(origin["providerFacts"], plan["providerFacts"]);
    assert!(
        origin["fileDigests"]
            .as_array()
            .unwrap()
            .iter()
            .any(|file| file["path"]
                == format!(".ai/work-items/archive/{WORK_ITEM_ID}.contract.json")
                && file["digest"] == raw_contract_digest.to_string())
    );
}

#[test]
fn provider_bound_recovery_rejects_canonical_digest_and_tampered_archive_or_provider_facts() {
    let authority = TestAuthority::install();
    for (mutation, expected_error) in [
        (
            "canonical_contract_digest",
            "resource finalization identity mismatch: contractDigest",
        ),
        (
            "archived_contract_bytes",
            "closeout_recovery_source_not_closed",
        ),
        (
            "archive_manifest_contract_digest",
            "closeout_recovery_source_not_closed",
        ),
        (
            "provider_identity",
            "resource finalization identity mismatch: resourceContext.provider",
        ),
    ] {
        let source = repository();
        let context = ResourceFinalizationContext {
            branch: "feature/provider-bound-closeout".into(),
            worktree: source
                .path()
                .join("removed-provider-bound-worktree")
                .display()
                .to_string(),
            base_branch: "develop".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: "https://github.com/example/sentinel/pull/123".into(),
        };
        let (_, canonical_contract_digest) =
            build_provider_bound_closed_source(&authority, source.path(), &context);

        let archived_contract = source.path().join(format!(
            ".ai/work-items/archive/{WORK_ITEM_ID}.contract.json"
        ));
        let archive_manifest = source.path().join(format!(
            ".ai/work-items/archive/{WORK_ITEM_ID}.archive.json"
        ));
        let finalization_receipt = source
            .path()
            .join(".ai/decisions")
            .join(format!("{WORK_ITEM_ID}.finalize.json"));
        match mutation {
            "canonical_contract_digest" | "provider_identity" => {
                let mut receipt: Value = serde_json::from_slice(
                    &fs::read(&finalization_receipt).expect("finalization receipt"),
                )
                .expect("finalization receipt JSON");
                if mutation == "canonical_contract_digest" {
                    receipt["contractDigest"] = canonical_contract_digest.to_string().into();
                } else {
                    receipt["provider"] = "gitlab".into();
                }
                fs::write(
                    &finalization_receipt,
                    serde_json::to_vec_pretty(&receipt).expect("finalization JSON"),
                )
                .expect("tamper finalization receipt");
            }
            "archived_contract_bytes" => {
                let mut contract_bytes = fs::read(&archived_contract).expect("archived Contract");
                contract_bytes.push(b' ');
                fs::write(&archived_contract, contract_bytes).expect("tamper archived Contract");
            }
            "archive_manifest_contract_digest" => {
                let mut manifest: Value =
                    serde_json::from_slice(&fs::read(&archive_manifest).expect("archive manifest"))
                        .expect("archive manifest JSON");
                manifest["files"]["contractDigest"] =
                    Digest::sha256_bytes(b"wrong archive contract digest")
                        .to_string()
                        .into();
                fs::write(
                    &archive_manifest,
                    serde_json::to_vec_pretty(&manifest).expect("archive manifest JSON"),
                )
                .expect("tamper archive manifest");
            }
            _ => unreachable!("mutation case is listed above"),
        }
        git(source.path(), &["add", "-A"]);
        git(
            source.path(),
            &[
                "commit",
                "--quiet",
                "-m",
                "tamper provider-bound archive fixture",
            ],
        );

        let destination_parent = tempfile::tempdir().expect("destination parent");
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
        let error = plan_cross_checkout_closeout_recovery(
            &destination,
            source.path(),
            WORK_ITEM_ID,
            &runtime(),
        )
        .expect_err("tampered provider-bound history is rejected");
        let error = error.to_string();
        assert!(
            error.contains(expected_error),
            "tampering {mutation} must fail for {expected_error:?}, got {error}"
        );
        assert!(
            !destination
                .join(".ai/decisions")
                .join(format!(
                    "{WORK_ITEM_ID}.cross-checkout-closeout-recovery.json"
                ))
                .exists(),
            "tampering {mutation} must not write a recovery receipt"
        );
    }
}

struct RetainedCloseRecoveryFixture {
    source: tempfile::TempDir,
    _destination_parent: tempfile::TempDir,
    destination: std::path::PathBuf,
    original_files: Vec<(String, Vec<u8>)>,
    transition: cockpit_protocol::ResourceFinalizationTransitionReceipt,
    appended_path: std::path::PathBuf,
}

fn recovered_retained_fixture(authority: &TestAuthority) -> RetainedCloseRecoveryFixture {
    let source = repository();
    let context = ResourceFinalizationContext {
        branch: "feature/recovered-retained-close".into(),
        worktree: source
            .path()
            .join("removed-retained-worktree")
            .display()
            .to_string(),
        base_branch: "develop".into(),
        base_remote: "origin".into(),
        provider: "github".into(),
        pull_request: "https://github.com/example/sentinel/pull/123".into(),
    };
    build_provider_bound_closed_source(authority, source.path(), &context);
    let finalization = format!(".ai/decisions/{WORK_ITEM_ID}.finalize.json");
    let close = format!(".ai/decisions/{WORK_ITEM_ID}.close.json");
    let mut retained: Value =
        serde_json::from_slice(&fs::read(source.path().join(&finalization)).unwrap()).unwrap();
    // Model the supported older Retained-close history in this disposable
    // fixture. Modern close correctly requires deletion; no real history is edited.
    retained["after"]["branch"] = "present".into();
    retained["after"]["worktree"] = "clean".into();
    retained["result"]["disposition"] = "retained".into();
    let retained_digest = cockpit_protocol::digest_json(&retained).unwrap();
    fs::write(
        source.path().join(&finalization),
        serde_json::to_vec_pretty(&retained).unwrap(),
    )
    .unwrap();
    let mut legacy_close: Value =
        serde_json::from_slice(&fs::read(source.path().join(&close)).unwrap()).unwrap();
    legacy_close["resourceFinalizationHeadDigest"] = retained_digest.to_string().into();
    fs::write(
        source.path().join(&close),
        serde_json::to_vec_pretty(&legacy_close).unwrap(),
    )
    .unwrap();
    git(source.path(), &["add", "-A"]);
    git(
        source.path(),
        &["commit", "--quiet", "-m", "legacy Retained close fixture"],
    );
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
    boundary::approve_initial_cross_checkout(
        authority,
        &destination,
        source.path(),
        WORK_ITEM_ID,
        &runtime(),
    );
    cockpit_repository::recover_cross_checkout_closeout(
        &destination,
        source.path(),
        WORK_ITEM_ID,
        &runtime(),
    )
    .expect("recover valid older Retained close");
    let baseline = cockpit_repository::work_item_status_snapshot_with_runtime(
        &destination,
        WORK_ITEM_ID,
        &runtime(),
    )
    .unwrap();
    assert_eq!(baseline.lifecycle_phase, "closed");
    assert!(!baseline.blocking);
    let origin = format!(".ai/decisions/{WORK_ITEM_ID}.cross-checkout-closeout-origin.json");
    let receipt = format!(".ai/decisions/{WORK_ITEM_ID}.cross-checkout-closeout-recovery.json");
    let frozen = [&close, &finalization, &origin, &receipt]
        .into_iter()
        .map(|p| (p.clone(), fs::read(destination.join(p)).unwrap()))
        .collect::<Vec<_>>();
    let previous: cockpit_protocol::ResourceFinalizationReceipt =
        serde_json::from_value(retained).unwrap();
    let mut deleted = previous.clone();
    deleted.receipt_id = "post-close-deleted".into();
    deleted.operation_id = "post-close-cleanup".into();
    deleted.before = previous.after.clone();
    deleted.after.branch = cockpit_protocol::ResourceFinalizationBranchState::Deleted;
    deleted.after.worktree = cockpit_protocol::ResourceFinalizationWorktreeState::Removed;
    deleted.result.disposition = cockpit_protocol::ResourceFinalizationDisposition::Deleted;
    let transition = cockpit_protocol::ResourceFinalizationTransitionReceipt {
        schema_version: 1,
        transition_id: "recovered-post-close-deleted".into(),
        sequence: 1,
        predecessor_receipt_digest: retained_digest,
        governance_append_revision: None,
        receipt: deleted,
    };
    let input = tempfile::NamedTempFile::new().unwrap();
    fs::write(
        input.path(),
        serde_json::to_vec_pretty(&transition).unwrap(),
    )
    .unwrap();
    let appended =
        record_resource_finalization(&destination, WORK_ITEM_ID, input.path(), &runtime())
            .expect("supported Native post-close Deleted transition");
    assert_eq!(appended["state"], "appended");
    let appended_path = destination.join(appended["path"].as_str().unwrap());
    RetainedCloseRecoveryFixture {
        source,
        _destination_parent: parent,
        destination,
        original_files: frozen,
        transition,
        appended_path,
    }
}

fn assert_retained_originals_unchanged(fixture: &RetainedCloseRecoveryFixture) {
    for (path, bytes) in &fixture.original_files {
        assert_eq!(&fs::read(fixture.destination.join(path)).unwrap(), bytes);
    }
}

fn assert_recovered_close_integrity(fixture: &RetainedCloseRecoveryFixture, closed: bool) {
    let status = cockpit_repository::work_item_status_snapshot_with_runtime(
        &fixture.destination,
        WORK_ITEM_ID,
        &runtime(),
    )
    .unwrap();
    assert_eq!(
        status.lifecycle_phase,
        if closed { "closed" } else { "archived" }
    );
    assert_eq!(status.blocking, !closed);
    assert_eq!(
        status.unknowns.contains(&"close_decision_invalid".into()),
        !closed
    );
    assert_retained_originals_unchanged(fixture);
}

#[test]
fn recovered_retained_close_accepts_normal_deleted_transition_without_rewriting_history() {
    let authority = TestAuthority::install();
    let fixture = recovered_retained_fixture(&authority);
    cockpit_repository::verify_resource_finalization(
        &fixture.destination,
        WORK_ITEM_ID,
        &runtime(),
    )
    .expect("the appended cleanup chain verifies");
    assert_recovered_close_integrity(&fixture, true);
}

#[test]
fn recovered_close_rejects_unsupported_finalization_extensions() {
    let authority = TestAuthority::install();
    let fixture = recovered_retained_fixture(&authority);
    assert_recovered_close_integrity(&fixture, true);
    let original = fs::read(&fixture.appended_path).unwrap();
    let transition = serde_json::to_value(&fixture.transition).unwrap();
    let wrong = serde_json::json!(Digest::sha256_bytes(b"unrelated binding"));
    for (pointer, value) in [
        ("/sequence", serde_json::json!(2)),
        ("/predecessorReceiptDigest", wrong.clone()),
        ("/receipt/runtimeDigest", wrong.clone()),
        (
            "/receipt/runtimeVersion",
            serde_json::json!("unrelated-runtime"),
        ),
        ("/receipt/repositoryId", wrong.clone()),
        ("/receipt/workItemId", serde_json::json!("WI-OTHER")),
        ("/receipt/provider", serde_json::json!("other-provider")),
        ("/receipt/contractDigest", wrong),
    ] {
        fs::remove_file(&fixture.appended_path).unwrap();
        let mut changed = transition.clone();
        *changed.pointer_mut(pointer).unwrap() = value;
        let digest = cockpit_protocol::digest_json(&changed).unwrap();
        let path = fixture.destination.join(format!(
            ".ai/decisions/{WORK_ITEM_ID}.finalize.{}.json",
            digest.as_str().trim_start_matches("sha256:")
        ));
        fs::write(&path, serde_json::to_vec_pretty(&changed).unwrap()).unwrap();
        assert_recovered_close_integrity(&fixture, false);
        fs::remove_file(path).unwrap();
        fs::write(&fixture.appended_path, &original).unwrap();
    }
    // A structurally valid Retained successor is not the permitted Deleted cleanup.
    fs::remove_file(&fixture.appended_path).unwrap();
    let mut retained = fixture.transition.clone();
    retained.receipt.after = retained.receipt.before.clone();
    retained.receipt.result.disposition =
        cockpit_protocol::ResourceFinalizationDisposition::Retained;
    let value = serde_json::to_value(&retained).unwrap();
    let digest = cockpit_protocol::digest_json(&value).unwrap();
    let path = fixture.destination.join(format!(
        ".ai/decisions/{WORK_ITEM_ID}.finalize.{}.json",
        digest.as_str().trim_start_matches("sha256:")
    ));
    fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    assert_recovered_close_integrity(&fixture, false);
    fs::remove_file(path).unwrap();
    fs::write(&fixture.appended_path, &original).unwrap();
    // Two valid next members form a fork rather than an alternative latest head.
    let mut fork = transition;
    fork["transitionId"] = "forked-cleanup".into();
    let digest = cockpit_protocol::digest_json(&fork).unwrap();
    let path = fixture.destination.join(format!(
        ".ai/decisions/{WORK_ITEM_ID}.finalize.{}.json",
        digest.as_str().trim_start_matches("sha256:")
    ));
    fs::write(&path, serde_json::to_vec_pretty(&fork).unwrap()).unwrap();
    assert_recovered_close_integrity(&fixture, false);
    fs::remove_file(path).unwrap();
    // Additional applicable history cannot be ignored just because cleanup is valid.
    let path = fixture.destination.join(format!(
        ".ai/decisions/{WORK_ITEM_ID}.ordinary-cleanup.json"
    ));
    fs::write(&path, b"{\"kind\":\"unrelated\"}").unwrap();
    assert_recovered_close_integrity(&fixture, false);
    fs::remove_file(path).unwrap();
    // Valid claimed cleanup does not excuse contradictory actual local resources.
    git(
        &fixture.destination,
        &["branch", "feature/recovered-retained-close"],
    );
    assert_recovered_close_integrity(&fixture, false);
    git(
        &fixture.destination,
        &["branch", "-d", "feature/recovered-retained-close"],
    );
    assert_recovered_close_integrity(&fixture, true);
}

#[test]
fn extended_recovered_close_keeps_terminal_replay_read_only_and_complete_next_source_inventory() {
    let authority = TestAuthority::install();
    let fixture = recovered_retained_fixture(&authority);
    assert_recovered_close_integrity(&fixture, true);
    let receipt = fixture.destination.join(format!(
        ".ai/decisions/{WORK_ITEM_ID}.cross-checkout-closeout-recovery.json"
    ));
    let original_receipt_bytes = fs::read(&receipt).unwrap();
    fs::remove_file(&receipt).unwrap();
    // The original receipt is intentionally missing, so check status separately
    // from the other immutable originals until normal recovery restores it.
    let status = cockpit_repository::work_item_status_snapshot_with_runtime(
        &fixture.destination,
        WORK_ITEM_ID,
        &runtime(),
    )
    .unwrap();
    assert_eq!(status.lifecycle_phase, "archived");
    assert!(status.blocking);
    let executions_before_replay = authority.execution_count();
    let before_replay = super::material_review::segmented_governance_bytes(&fixture.destination);
    cockpit_repository::recover_cross_checkout_closeout(
        &fixture.destination,
        fixture.source.path(),
        WORK_ITEM_ID,
        &runtime(),
    )
    .expect("the completed result may be replayed without reconstructing missing evidence");
    assert_eq!(authority.execution_count(), executions_before_replay);
    assert_eq!(
        super::material_review::segmented_governance_bytes(&fixture.destination),
        before_replay
    );
    assert!(
        !receipt.exists(),
        "terminal replay must not repeat a business write"
    );
    let blocked = cockpit_repository::work_item_status_snapshot_with_runtime(
        &fixture.destination,
        WORK_ITEM_ID,
        &runtime(),
    )
    .unwrap();
    assert_eq!(blocked.lifecycle_phase, "archived");
    assert!(blocked.blocking);
    // Restore only synthetic fixture data from the exact bytes retained before
    // the adversarial deletion. This is not a production repair operation.
    fs::write(&receipt, &original_receipt_bytes).unwrap();
    assert_eq!(
        cockpit_repository::work_item_status_snapshot_with_runtime(
            &fixture.destination,
            WORK_ITEM_ID,
            &runtime()
        )
        .unwrap()
        .lifecycle_phase,
        "closed"
    );
    // Restored bytes retain the original completed fact and original createdAt.
    for (path, bytes) in &fixture.original_files {
        if !path.ends_with(".cross-checkout-closeout-recovery.json") {
            assert_eq!(&fs::read(fixture.destination.join(path)).unwrap(), bytes);
        }
    }
    let parent = tempfile::tempdir().unwrap();
    let later = parent.path().join("clone");
    git(
        fixture.source.path(),
        &[
            "clone",
            "--quiet",
            "--no-hardlinks",
            fixture.source.path().to_str().unwrap(),
            later.to_str().unwrap(),
        ],
    );
    boundary::approve_initial_cross_checkout(
        &authority,
        &later,
        &fixture.destination,
        WORK_ITEM_ID,
        &runtime(),
    );
    let later_installed = cockpit_repository::recover_cross_checkout_closeout(
        &later,
        &fixture.destination,
        WORK_ITEM_ID,
        &runtime(),
    )
    .expect("recover from a validated extended historical source");
    let origin_path = later.join(format!(
        ".ai/decisions/{WORK_ITEM_ID}.cross-checkout-closeout-origin.json"
    ));
    let origin: Value = serde_json::from_slice(&fs::read(&origin_path).unwrap()).unwrap();
    assert_eq!(origin["providerFacts"]["finalizationSequence"], 1);
    let appended_relative = fixture
        .appended_path
        .strip_prefix(&fixture.destination)
        .unwrap();
    assert!(
        origin["fileDigests"]
            .as_array()
            .unwrap()
            .iter()
            .any(|file| file["path"] == appended_relative.to_str().unwrap())
    );
    let copied = later.join(appended_relative);
    assert_eq!(
        fs::read(&copied).unwrap(),
        fs::read(&fixture.appended_path).unwrap()
    );
    let original_copied_bytes = fs::read(&copied).unwrap();
    fs::remove_file(&copied).unwrap();
    let status = cockpit_repository::work_item_status_snapshot_with_runtime(
        &later,
        WORK_ITEM_ID,
        &runtime(),
    )
    .unwrap();
    assert_eq!(status.lifecycle_phase, "archived");
    assert!(status.blocking);
    let later_executions = authority.execution_count();
    let later_before_replay = super::material_review::segmented_governance_bytes(&later);
    cockpit_repository::recover_cross_checkout_closeout(
        &later,
        &fixture.destination,
        WORK_ITEM_ID,
        &runtime(),
    )
    .unwrap();
    assert_eq!(authority.execution_count(), later_executions);
    assert_eq!(
        super::material_review::segmented_governance_bytes(&later),
        later_before_replay
    );
    assert!(
        !copied.exists(),
        "historical replay cannot republish a missing copied artifact"
    );
    let blocked = cockpit_repository::work_item_status_snapshot_with_runtime(
        &later,
        WORK_ITEM_ID,
        &runtime(),
    )
    .unwrap();
    assert_eq!(blocked.lifecycle_phase, "archived");
    assert!(blocked.blocking);
    // Exact retained fixture bytes enable the subsequent integrity checks;
    // an actual repair requires a separately authorized product workflow.
    fs::write(&copied, &original_copied_bytes).unwrap();
    let retry = cockpit_repository::recover_cross_checkout_closeout(
        &later,
        &fixture.destination,
        WORK_ITEM_ID,
        &runtime(),
    )
    .unwrap();
    assert_eq!(
        retry, later_installed,
        "replay returns the same operation result, including its historical change list"
    );
    assert_eq!(authority.execution_count(), later_executions);
    assert_eq!(
        super::material_review::segmented_governance_bytes(&later),
        {
            let mut restored = later_before_replay;
            restored.insert(
                copied.strip_prefix(&later).unwrap().to_path_buf(),
                (b'f', original_copied_bytes),
            );
            restored
        }
    );
    // When the source already included cleanup, the new origin must bind it.
    // Two matching truncated lists cannot disguise its omission.
    let receipt_path = later.join(format!(
        ".ai/decisions/{WORK_ITEM_ID}.cross-checkout-closeout-recovery.json"
    ));
    let origin_bytes = fs::read(&origin_path).unwrap();
    let receipt_bytes = fs::read(&receipt_path).unwrap();
    let mut changed = origin;
    changed["fileDigests"]
        .as_array_mut()
        .unwrap()
        .retain(|file| file["path"] != appended_relative.to_str().unwrap());
    let bytes = serde_json::to_vec_pretty(&changed).unwrap();
    let mut paired: Value = serde_json::from_slice(&receipt_bytes).unwrap();
    paired["fileDigests"] = changed["fileDigests"].clone();
    paired["originBinding"]["digest"] = Digest::sha256_bytes(&bytes).to_string().into();
    fs::write(&origin_path, bytes).unwrap();
    fs::write(&receipt_path, serde_json::to_vec_pretty(&paired).unwrap()).unwrap();
    let status = cockpit_repository::work_item_status_snapshot_with_runtime(
        &later,
        WORK_ITEM_ID,
        &runtime(),
    )
    .unwrap();
    assert_eq!(status.lifecycle_phase, "archived");
    assert!(status.blocking);
    fs::write(&origin_path, origin_bytes).unwrap();
    fs::write(&receipt_path, receipt_bytes).unwrap();
    assert_eq!(
        cockpit_repository::work_item_status_snapshot_with_runtime(
            &later,
            WORK_ITEM_ID,
            &runtime()
        )
        .unwrap()
        .lifecycle_phase,
        "closed"
    );
}

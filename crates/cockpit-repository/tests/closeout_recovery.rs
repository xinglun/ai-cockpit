use cockpit_core::Digest;
use cockpit_protocol::{
    HumanDecision, PROTOCOL_VERSION, ResourceFinalizationContext, RuntimeContext,
};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions, attach,
    checkpoint_work_item, close_work_item_with_structured_decision_and_runtime,
    finish_work_item_with_runtime, plan_cross_checkout_closeout_recovery,
    plan_resource_finalization_with_runtime, record_resource_finalization,
    record_verification_with_runtime, repository_id, run_repository_verification,
    start_work_item_with_options,
};
use serde_json::Value;
use std::{fs, path::Path, process::Command};

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
    close_work_item_with_structured_decision_and_runtime(
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

    close_work_item_with_structured_decision_and_runtime(
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
        build_provider_bound_closed_source(source.path(), &context);

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
}

#[test]
fn provider_bound_recovery_rejects_canonical_digest_and_tampered_archive_or_provider_facts() {
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
            build_provider_bound_closed_source(source.path(), &context);

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

//! Native domain fixtures use an explicit private test port, never a production source.
#![cfg(test)]
use super::approved_boundary_fixtures as boundary;
use crate as cockpit_repository;
use crate::review_authority::pipeline_tests::TestAuthority;
use cockpit_core::Digest;
use cockpit_git::GitRepository;
use cockpit_protocol::AssuranceLevel;
use cockpit_protocol::DelegatedEvidence;
use cockpit_protocol::EvidenceValidity;
use cockpit_protocol::HumanDecision;
use cockpit_protocol::ResourceFinalizationContext;
use cockpit_protocol::RuntimeContext;
use cockpit_protocol::VerificationStage;
use cockpit_protocol::VerificationTier;
use cockpit_repository::ArchivedVerificationRecoveryRequest;
use cockpit_repository::WorkItemStartOptions;
use cockpit_repository::archive_work_item;
use cockpit_repository::attach;
use cockpit_repository::checkpoint_work_item;
use cockpit_repository::finish_work_item;
use cockpit_repository::import_delegated_evidence;
use cockpit_repository::plan_resource_finalization;
use cockpit_repository::preflight_work_item;
use cockpit_repository::record_verification;
use cockpit_repository::record_verification_with_runtime;
use cockpit_repository::record_work_item_governance_controls;
use cockpit_repository::run_repository_verification;
use cockpit_repository::start_work_item;
use cockpit_repository::start_work_item_with_options;
use cockpit_verification::VerificationCoverageManifest;
use cockpit_verification::VerificationPlanReceipt;
use std::fs;
use std::process::Command;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
static NEXT_REPOSITORY_ID: AtomicU64 = AtomicU64::new(0);

fn git_commit(path: &std::path::Path, files: &[&str], message: &str) {
    if !files.is_empty() {
        let mut add = Command::new("git");
        add.args(["add", "--"]).args(files).current_dir(path);
        assert!(add.status().expect("git add fixture").success());
    }

    let mut commit = Command::new("git");
    commit
        .args([
            "-c",
            "user.name=AI Cockpit Test",
            "-c",
            "user.email=ai-cockpit-test@example.invalid",
            "commit",
        ])
        .current_dir(path);
    if files.is_empty() {
        commit.arg("--allow-empty");
    }
    assert!(
        commit
            .args(["-m", message])
            .status()
            .expect("git commit fixture")
            .success()
    );
}

fn repository() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "cockpit-archive-integrity-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&path)
        .status()
        .expect("git init");
    git_commit(&path, &[], "fixture baseline");
    attach(&path).expect("attach");
    path
}

fn prepare_for_verification(authority: &TestAuthority, path: &std::path::Path, work_item_id: &str) {
    plan_resource_finalization(
        path,
        work_item_id,
        &ResourceFinalizationContext {
            branch: format!("feature/{work_item_id}"),
            worktree: path.display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: format!("https://github.com/example/ai-cockpit/pull/{work_item_id}"),
        },
    )
    .expect("finalization plan");
    prepare_without_finalization_plan(authority, path, work_item_id);
}

fn prepare_without_finalization_plan(
    authority: &TestAuthority,
    path: &std::path::Path,
    work_item_id: &str,
) {
    let contract = path
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let decision = preflight_work_item(path, &contract).expect("preflight");
    assert_ne!(
        decision.state,
        cockpit_core::DecisionState::Red,
        "preflight blocked archive-integrity fixture: {decision:#?}"
    );
    authority.approve(
        path,
        work_item_id,
        cockpit_protocol::review_authority::HumanReviewOperation::PreflightReview,
        &serde_json::json!({"decision":"confirm_review"}),
    );
    checkpoint_work_item(path, work_item_id).expect("checkpoint");
}

#[test]
fn no_resource_context_can_finish_archive_and_close_without_provider_evidence() {
    let authority = TestAuthority::install();
    let path = repository();
    let work_item_id = "WI-NO-EXTERNAL-RESOURCE";
    start_work_item(
        &path,
        work_item_id,
        "local object-engineering change",
        "complete without a provider release resource",
        &[".ai/**".into()],
    )
    .expect("start");

    let contract_path = path
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract"))
            .expect("contract JSON");
    assert!(contract.get("resourceContext").is_none());

    let decision = preflight_work_item(&path, &contract_path).expect("preflight");
    assert_ne!(decision.state, cockpit_core::DecisionState::Red);
    checkpoint_work_item(&path, work_item_id).expect("checkpoint");
    record_verification(
        &path,
        work_item_id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.23",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, work_item_id).expect("finish without external resource");
    archive_work_item(&path, work_item_id).expect("archive without external resource");
    boundary::close_legacy(&authority, &path, work_item_id, "approved")
        .expect("close without external resource finalization");

    assert!(
        path.join(".ai/decisions")
            .join(format!("{work_item_id}.close.json"))
            .is_file()
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn archive_defers_later_external_evidence_but_close_remains_blocked() {
    let authority = TestAuthority::install();
    let path = repository();
    let work_item_id = "WI-ARCHIVE-DEFERS-EXTERNAL-EVIDENCE";
    start_work_item_with_options(
        &path,
        work_item_id,
        "archive evidence boundary",
        "allow archival before reviewed provider actions create their required evidence",
        &[".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            required_evidence_classes: vec![
                "verification".into(),
                "external_evidence".into(),
                "delegated:github".into(),
            ],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    prepare_without_finalization_plan(&authority, &path, work_item_id);
    record_verification(
        &path,
        work_item_id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.103",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");

    finish_work_item(&path, work_item_id).expect("finish defers later external evidence");
    archive_work_item(&path, work_item_id)
        .expect("archive defers later external and delegated evidence");

    let error = boundary::close_legacy(&authority, &path, work_item_id, "approved")
        .expect_err("close must still require the declared external evidence");
    assert!(
        error.to_string().contains("required_evidence_missing"),
        "unexpected close error: {error}"
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn archived_source_recovery_preserves_history_and_replaces_only_stale_projection() {
    let authority = TestAuthority::install();
    let path = repository();
    let work_item_id = "WI-ARCHIVED-SOURCE-RECOVERY";
    for (name, contents, message) in [
        ("base.txt", "base\n", "base"),
        ("second.txt", "second\n", "second"),
    ] {
        fs::write(path.join(name), contents).expect("source file");
        assert!(
            Command::new("git")
                .args(["add", name])
                .current_dir(&path)
                .status()
                .expect("git add")
                .success()
        );
        assert!(
            Command::new("git")
                .args([
                    "-c",
                    "user.name=AI Cockpit Test",
                    "-c",
                    "user.email=ai-cockpit-test@example.invalid",
                    "commit",
                    "-m",
                    message,
                ])
                .current_dir(&path)
                .status()
                .expect("git commit")
                .success()
        );
    }
    start_work_item_with_options(
        &path,
        work_item_id,
        "recover stale archived source evidence",
        "prove current integration verification can replace one stale projection",
        &[
            ".ai/**".into(),
            "base.txt".into(),
            "second.txt".into(),
            "performance-measurement.txt".into(),
        ],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            required_evidence_classes: vec!["performance".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    let contract_path = path
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let measurement_path = path.join("performance-measurement.txt");
    fs::write(&measurement_path, b"p50=1ms\np95=2ms\n").expect("measurement");
    git_commit(
        &path,
        &["performance-measurement.txt"],
        "measurement fixture",
    );
    let contract_value: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract")).expect("contract");
    let contract_digest = cockpit_protocol::digest_json(&contract_value).expect("contract digest");
    record_work_item_governance_controls(
        &path,
        work_item_id,
        &serde_json::json!({
            "evidenceClasses": {
                "schemaVersion": 1,
                "contractDigest": contract_digest,
                "items": [{
                    "class": "performance",
                    "evidence": [{
                        "type": "measurement",
                        "path": "performance-measurement.txt",
                        "locator": "p50,p95",
                        "verification": "passed",
                        "digest": Digest::sha256_bytes(&fs::read(&measurement_path).expect("measurement bytes"))
                    }]
                }]
            }
        }),
    )
    .expect("record projection");
    let old_runtime = RuntimeContext {
        runtime_version: "0.2.90".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"old-runtime"),
    };
    let current_runtime = RuntimeContext {
        runtime_version: "0.2.91".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"current-runtime"),
    };
    let preflight = preflight_work_item(&path, &contract_path).expect("preflight");
    assert_ne!(
        preflight.state,
        cockpit_core::DecisionState::Red,
        "preflight blocked archived source recovery: {preflight:#?}"
    );
    checkpoint_work_item(&path, work_item_id).expect("checkpoint");
    let old_snapshot = GitRepository::discover(&path)
        .expect("git repository")
        .snapshot()
        .expect("old snapshot");
    let old_run = run_repository_verification(
        &path,
        &cockpit_repository::RepositoryVerificationRequest {
            node_id: "project-command-0-package-fixture".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec![
                ".ai/**".into(),
                "base.txt".into(),
                "second.txt".into(),
                "performance-measurement.txt".into(),
            ],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: old_runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: None,
            timeout_seconds: None,
            policy: cockpit_repository::RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .expect("old verification");
    record_verification_with_runtime(
        &path,
        work_item_id,
        &serde_json::to_value(old_run.receipt).expect("old receipt JSON"),
        &old_runtime,
        &old_snapshot,
    )
    .expect("record old verification");
    finish_work_item(&path, work_item_id).expect("finish");
    archive_work_item(&path, work_item_id).expect("archive");

    let archive_root = path.join(".ai/work-items/archive");
    let historical_contract = fs::read(archive_root.join(format!("{work_item_id}.contract.json")))
        .expect("historical contract");
    let historical_summary = fs::read(archive_root.join(format!("{work_item_id}.summary.json")))
        .expect("historical summary");
    let historical_outcome = fs::read(archive_root.join(format!("{work_item_id}.outcome.json")))
        .expect("historical outcome");
    let historical_evidence = fs::read(
        path.join(".ai/evidence")
            .join(format!("{work_item_id}.verification.json")),
    )
    .expect("historical verification");
    fs::write(&measurement_path, b"p50=9ms\np95=20ms\n").expect("changed measurement");
    let current_snapshot = GitRepository::discover(&path)
        .expect("git repository")
        .snapshot()
        .expect("current snapshot");
    let current_request = cockpit_repository::RepositoryVerificationRequest {
        node_id: "project-command-0-package-fixture".into(),
        program: "true".into(),
        args: Vec::new(),
        scope: vec![
            ".ai/**".into(),
            "base.txt".into(),
            "second.txt".into(),
            "performance-measurement.txt".into(),
        ],
        stage: "task".into(),
        runner: "local".into(),
        runtime_digest: current_runtime.runtime_digest.to_string(),
        base_commit: None,
        workers: 1,
        work_item_id: None,
        timeout_seconds: None,
        policy: cockpit_repository::RepositoryVerificationPolicy::NeverReuse,
    };
    let mut current_run =
        run_repository_verification(&path, &current_request).expect("current verification");
    let command_digest = current_run
        .receipt
        .execution_records
        .first()
        .expect("current execution record")
        .command_digest
        .clone();
    let coverage_manifest = VerificationCoverageManifest {
        schema_version: 1,
        source_program: "true".into(),
        source_args: Vec::new(),
        planning_program: "test-planner".into(),
        planning_args: Vec::new(),
        planning_processes_spawned: 1,
        metadata_digest: Digest::sha256_bytes(b"metadata").to_string(),
        workspace_members: vec!["fixture".into()],
        node_ids: vec![current_request.node_id.clone()],
        command_digests: vec![command_digest],
    };
    coverage_manifest.validate().expect("coverage manifest");
    let mut plan = VerificationPlanReceipt::new(
        VerificationStage::Task,
        VerificationTier::T0,
        VerificationTier::T0,
        cockpit_protocol::EvidenceAssurance::SelfDeclared,
        vec!["archived_source_recovery_test".into()],
        Vec::new(),
    )
    .expect("plan receipt");
    plan.work_item_id = Some(work_item_id.into());
    plan.repository_id = Some(cockpit_repository::repository_id(&path).to_string());
    plan.repository_snapshot_digest = Some(
        cockpit_repository::snapshot_digest(&current_run.final_snapshot)
            .expect("current snapshot digest")
            .to_string(),
    );
    plan.executed_nodes = vec![current_request.node_id.clone()];
    plan.coverage_manifest = Some(coverage_manifest);
    current_run.receipt.plan_receipt = Some(plan);
    boundary::archived_verification(
        &authority,
        &path,
        work_item_id,
        &ArchivedVerificationRecoveryRequest {
            reason: "reviewed source integration changed the recorded measurement".into(),
            actor: "human:test".into(),
            authority_source: "test contract".into(),
            evidence_refs: vec![format!(".ai/evidence/{work_item_id}.verification.json")],
            policy_refs: Vec::new(),
            verification_receipt: serde_json::to_value(current_run.receipt).expect("receipt JSON"),
        },
        &current_runtime,
        &current_snapshot,
    )
    .expect("record archived recovery");

    assert_eq!(
        historical_contract,
        fs::read(archive_root.join(format!("{work_item_id}.contract.json")))
            .expect("contract after recovery")
    );
    assert_eq!(
        historical_summary,
        fs::read(archive_root.join(format!("{work_item_id}.summary.json")))
            .expect("summary after recovery")
    );
    assert_eq!(
        historical_outcome,
        fs::read(archive_root.join(format!("{work_item_id}.outcome.json")))
            .expect("outcome after recovery")
    );
    assert_eq!(
        historical_evidence,
        fs::read(
            path.join(".ai/evidence")
                .join(format!("{work_item_id}.verification.json"))
        )
        .expect("evidence after recovery")
    );
    let recovery_path = path
        .join(".ai/decisions")
        .join(format!("{work_item_id}.verification-recovery.json"));
    let recovery: serde_json::Value =
        serde_json::from_slice(&fs::read(&recovery_path).expect("recovery receipt"))
            .expect("recovery JSON");
    assert_eq!(recovery["replacedJudgment"], "evidence_class_projection");
    assert_eq!(
        recovery["currentSourceRevision"],
        current_snapshot.head.clone().unwrap()
    );

    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn public_close_rejects_bound_resource_without_finalization_receipt() {
    let authority = TestAuthority::install();
    let path = repository();
    let work_item_id = "WI-CLOSE-REQUIRES-FINALIZATION";
    start_work_item(
        &path,
        work_item_id,
        "enforce resource finalization at close",
        "prevent the public no-runtime API from bypassing cleanup evidence",
        &[".ai/**".into()],
    )
    .expect("start");
    prepare_for_verification(&authority, &path, work_item_id);
    record_verification(
        &path,
        work_item_id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.23",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, work_item_id).expect("finish");
    archive_work_item(&path, work_item_id).expect("archive");

    let error = boundary::close_legacy(&authority, &path, work_item_id, "approved")
        .expect_err("bound resources require finalization before close");
    assert!(error.to_string().contains("resource finalization"));
    assert!(
        !path
            .join(".ai/decisions")
            .join(format!("{work_item_id}.close.json"))
            .exists()
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn close_rejects_tampered_archived_artifacts() {
    let authority = TestAuthority::install();
    let path = repository();
    start_work_item(
        &path,
        "WI-INTEGRITY",
        "integrity",
        "verify",
        &[".ai/**".into()],
    )
    .expect("start");
    prepare_for_verification(&authority, &path, "WI-INTEGRITY");
    record_verification(
        &path,
        "WI-INTEGRITY",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.1.0",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, "WI-INTEGRITY").expect("finish");
    archive_work_item(&path, "WI-INTEGRITY").expect("archive");
    fs::write(
        path.join(".ai/work-items/archive/WI-INTEGRITY.outcome.json"),
        br#"{"tampered":true}"#,
    )
    .expect("tamper");
    let error = boundary::close_legacy(&authority, &path, "WI-INTEGRITY", "approved")
        .expect_err("tampered archive must be rejected");
    assert!(error.to_string().contains("digest does not match manifest"));
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn archive_moves_failed_attempt_variants_and_binds_their_digests() {
    let authority = TestAuthority::install();
    let path = repository();
    let work_item_id = "WI-FAILED-ATTEMPT-VARIANTS";
    start_work_item(
        &path,
        work_item_id,
        "preserve failed lifecycle attempts",
        "move historical blocked projections out of active during archive",
        &[".ai/**".into()],
    )
    .expect("start");
    prepare_for_verification(&authority, &path, work_item_id);
    record_verification(
        &path,
        work_item_id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.23",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, work_item_id).expect("finish");

    let variants = [
        (
            format!("{work_item_id}.outcome.finish-blocked.json"),
            br#"{"state":"blocked","workItemId":"WI-FAILED-ATTEMPT-VARIANTS"}"#.to_vec(),
        ),
        (
            format!("{work_item_id}.events.finish-blocked.jsonl"),
            br#"{"schemaVersion":1,"eventType":"blocked"}
"#
            .to_vec(),
        ),
    ];
    for (name, bytes) in &variants {
        fs::write(path.join(".ai/work-items/active").join(name), bytes).expect("variant");
    }

    archive_work_item(&path, work_item_id).expect("archive");
    let archive = path.join(".ai/work-items/archive");
    let historical = archive.join(format!("{work_item_id}.outcome.finish-blocked.json"));
    assert_eq!(
        fs::read(&historical).expect("archived outcome variant"),
        variants[0].1
    );
    assert_eq!(
        fs::read(archive.join(format!("{work_item_id}.events.finish-blocked.jsonl")))
            .expect("archived events variant"),
        variants[1].1
    );
    for (name, _) in &variants {
        assert!(!path.join(".ai/work-items/active").join(name).exists());
    }
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(archive.join(format!("{work_item_id}.archive.json"))).expect("manifest"),
    )
    .expect("manifest JSON");
    let historical_artifacts = manifest["historicalArtifacts"]
        .as_array()
        .expect("historical artifact manifest");
    assert_eq!(historical_artifacts.len(), variants.len());
    for (name, bytes) in &variants {
        let path_value = format!(".ai/work-items/archive/{name}");
        let item = historical_artifacts
            .iter()
            .find(|item| item["path"] == path_value)
            .expect("variant path bound");
        assert_eq!(
            item["digest"],
            Digest::sha256_bytes(bytes).to_string(),
            "variant digest bound"
        );
    }
    fs::write(
        archive.join(format!("{work_item_id}.outcome.finish-blocked.json")),
        br#"{"tampered":true}"#,
    )
    .expect("tamper historical variant");
    let error = boundary::close_legacy(&authority, &path, work_item_id, "approved")
        .expect_err("tampered historical variant must be rejected");
    assert!(error.to_string().contains("historical artifact digest"));
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn close_persists_a_structured_human_decision_and_recovery_condition() {
    let authority = TestAuthority::install();
    let path = repository();
    start_work_item(
        &path,
        "WI-DECISION",
        "decision",
        "verify",
        &[".ai/**".into()],
    )
    .expect("start");
    prepare_without_finalization_plan(&authority, &path, "WI-DECISION");
    record_verification(
        &path,
        "WI-DECISION",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.1.0",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    let evidence: serde_json::Value = serde_json::from_slice(
        &fs::read(path.join(".ai/evidence/WI-DECISION.verification.json")).expect("evidence"),
    )
    .expect("evidence JSON");
    assert_eq!(evidence["evidenceSchemaVersion"], 2);
    assert_eq!(
        evidence["repositoryId"],
        cockpit_repository::repository_id(&path).to_string()
    );
    finish_work_item(&path, "WI-DECISION").expect("finish");
    archive_work_item(&path, "WI-DECISION").expect("archive");
    boundary::close_structured(
        &authority,
        &path,
        "WI-DECISION",
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "team-policy".into(),
            reason: "bounded change and fresh evidence".into(),
            evidence_refs: vec![".ai/evidence/WI-DECISION.verification.json".into()],
            policy_refs: vec!["team-policy-v1".into()],
            decided_at: "2026-08-21T19:00:00Z".into(),
            resume_condition: Some("rerun verification if the base changes".into()),
        },
    )
    .expect("structured close");
    let decision: serde_json::Value = serde_json::from_slice(
        &fs::read(path.join(".ai/decisions/WI-DECISION.close.json")).expect("decision"),
    )
    .expect("decision JSON");
    assert_eq!(
        decision["repositoryId"],
        cockpit_repository::repository_id(&path).to_string()
    );
    assert_eq!(decision["structuredDecision"]["actor"], "human:owner");
    assert_eq!(
        decision["structuredDecision"]["resumeCondition"],
        "rerun verification if the base changes"
    );
    assert_eq!(decision["finalReport"]["format"], "ai-cockpit.task-outcome");
    assert!(
        !decision["finalReport"]["sections"]["humanDecisions"]
            .as_array()
            .expect("human decision projections")
            .is_empty()
    );
    assert!(decision["finalReportDigest"].as_str().is_some());
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn close_accepts_immutable_archived_evidence_after_a_post_archive_commit() {
    let authority = TestAuthority::install();
    let path = repository();
    start_work_item(
        &path,
        "WI-ARCHIVE-MERGE",
        "archive",
        "close",
        &[".ai/**".into()],
    )
    .expect("start");
    prepare_without_finalization_plan(&authority, &path, "WI-ARCHIVE-MERGE");
    record_verification(
        &path,
        "WI-ARCHIVE-MERGE",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.9",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, "WI-ARCHIVE-MERGE").expect("finish");
    archive_work_item(&path, "WI-ARCHIVE-MERGE").expect("archive");

    fs::write(
        path.join("post-archive-change.txt"),
        b"merged release change\n",
    )
    .expect("post-archive change");
    assert!(
        Command::new("git")
            .args(["add", "post-archive-change.txt"])
            .current_dir(&path)
            .status()
            .expect("git add")
            .success()
    );
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=AI Cockpit Test",
                "-c",
                "user.email=ai-cockpit-test@example.invalid",
                "commit",
                "-m",
                "post-archive merge",
            ])
            .current_dir(&path)
            .status()
            .expect("git commit")
            .success()
    );

    boundary::close_structured(
        &authority,
        &path,
        "WI-ARCHIVE-MERGE",
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "team-policy".into(),
            reason: "archive was reviewed before the merge commit".into(),
            evidence_refs: vec![".ai/evidence/WI-ARCHIVE-MERGE.verification.json".into()],
            policy_refs: Vec::new(),
            decided_at: "2026-08-22T06:00:00Z".into(),
            resume_condition: None,
        },
    )
    .expect("close after merge must use immutable archived evidence");
    assert!(
        path.join(".ai/decisions/WI-ARCHIVE-MERGE.close.json")
            .is_file()
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn organization_policy_requires_a_bound_structured_decision_at_close() {
    let authority = TestAuthority::install();
    let path = repository();
    fs::write(
        path.join(".ai/policy.json"),
        r#"{
          "schemaVersion": 1,
          "organization": {
            "policyId": "org-release-v1",
            "layer": "organization",
            "rules": [{
              "operation": "modify_source",
              "approvalMode": "single_authorized_human",
              "requiredEvidence": ["delegated:github"]
            }]
          }
        }"#,
    )
    .expect("policy");
    start_work_item_with_options(
        &path,
        "WI-POLICY",
        "policy",
        "verify",
        &[".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            required_evidence_classes: vec!["verification".into(), "delegated:github".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    prepare_without_finalization_plan(&authority, &path, "WI-POLICY");
    let checkpoint_binding =
        crate::human_review_binding(&path, "WI-POLICY").expect("native checkpoint binding");
    let checkpoint_summary: serde_json::Value = serde_json::from_slice(
        &fs::read(path.join(".ai/work-items/active/WI-POLICY.summary.json")).unwrap(),
    )
    .unwrap();
    eprintln!(
        "POLICY_BINDING_CHECKPOINT={}",
        serde_json::to_string(&checkpoint_binding).unwrap()
    );
    let raw = br#"{"run":999}"#;
    import_delegated_evidence(
        &path,
        "WI-POLICY",
        &DelegatedEvidence {
            provider: "github".into(),
            subject: "run:999".into(),
            origin: "https://github.com/example/repo/actions/runs/999".into(),
            assurance: AssuranceLevel::ProviderVerified,
            collected_at: "2026-08-21T19:00:00Z".into(),
            digest: cockpit_core::Digest::sha256_bytes(raw),
            validity: EvidenceValidity::Valid,
            raw_evidence_ref: ".ai/evidence/external/github-run-999.json".into(),
        },
        raw,
        &RuntimeContext {
            runtime_version: "0.2.2".into(),
            protocol_version: 1,
            runtime_digest: Digest::sha256_bytes(b"runtime"),
        },
    )
    .expect("delegated evidence");
    let imported_binding = crate::human_review_binding(&path, "WI-POLICY")
        .expect("native binding after delegated evidence");
    assert_eq!(
        imported_binding, checkpoint_binding,
        "delegated evidence cannot replace approval"
    );
    eprintln!(
        "POLICY_BINDING_AFTER_IMPORT={}",
        serde_json::to_string(&imported_binding).unwrap()
    );
    let contract_path = path.join(".ai/work-items/active/WI-POLICY.contract.json");
    let contract = crate::read_contract(&contract_path).unwrap();
    let snapshot = GitRepository::discover(&path).unwrap().snapshot().unwrap();
    let mut before = crate::governance_decision_for_pre_execution_boundary(
        &path, &contract, &snapshot, None, None,
    )
    .unwrap();
    crate::material_review::apply_material_review_gate_to_decision(
        &path,
        &contract,
        &path.join(".ai/work-items/active/WI-POLICY.summary.json"),
        &mut before,
    );
    eprintln!(
        "POLICY_RAW_BEFORE_VERIFICATION={}",
        serde_json::to_string(&before).unwrap()
    );
    eprintln!(
        "POLICY_RAW_BEFORE_VERIFICATION_DIGEST={}",
        cockpit_protocol::digest_json(&serde_json::to_value(&before).unwrap()).unwrap()
    );
    record_verification(
        &path,
        "WI-POLICY",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.1",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    let completion_binding =
        crate::human_review_binding(&path, "WI-POLICY").expect("native completion binding");
    eprintln!(
        "POLICY_BINDING_AFTER_VERIFICATION={}",
        serde_json::to_string(&completion_binding).unwrap()
    );
    assert_eq!(
        completion_binding, checkpoint_binding,
        "recording completion must not replace an unchanged approved C/P/R/context"
    );
    let completion_summary: serde_json::Value = serde_json::from_slice(
        &fs::read(path.join(".ai/work-items/active/WI-POLICY.summary.json")).unwrap(),
    )
    .unwrap();
    for key in [
        "preflightDecisionDigest",
        "preflightContractDigest",
        "preflightRepositorySnapshotDigest",
    ] {
        assert_eq!(
            completion_summary[key], checkpoint_summary[key],
            "completion keeps approved basis: {key}"
        );
    }
    finish_work_item(&path, "WI-POLICY").expect("finish");
    archive_work_item(&path, "WI-POLICY").expect("archive");

    let missing_binding = boundary::close_structured(
        &authority,
        &path,
        "WI-POLICY",
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "team-policy".into(),
            reason: "fresh evidence".into(),
            evidence_refs: vec![".ai/evidence/WI-POLICY.verification.json".into()],
            policy_refs: Vec::new(),
            decided_at: "2026-08-21T19:00:00Z".into(),
            resume_condition: None,
        },
    )
    .expect_err("policy close must bind the policy reference");
    assert!(missing_binding.to_string().contains("policy"));

    boundary::close_structured(
        &authority,
        &path,
        "WI-POLICY",
        &HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "team-policy".into(),
            reason: "fresh evidence".into(),
            evidence_refs: vec![".ai/evidence/WI-POLICY.verification.json".into()],
            policy_refs: vec!["org-release-v1".into()],
            decided_at: "2026-08-21T19:00:00Z".into(),
            resume_condition: None,
        },
    )
    .expect("bound policy close");
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn policy_verification_and_finish_in_one_commit_reconstruct_the_reviewed_start_basis() {
    policy_history_reconstruction(false, false);
}

#[test]
fn historical_recorder_rejects_late_missing_and_unavailable_protected_source() {
    policy_history_reconstruction(true, false);
}

#[test]
fn native_archive_time_validation_rejects_alias_precision_order_and_record_mismatch() {
    policy_history_reconstruction(false, true);
}

fn policy_history_reconstruction(source_faults: bool, archive_time_cases: bool) {
    let authority = TestAuthority::install();
    let path = repository();
    let id = "WI-POLICY-HISTORY";
    fs::write(path.join(".ai/policy.json"), r#"{"schemaVersion":1,"organization":{"policyId":"org-history-v1","layer":"organization","rules":[{"operation":"modify_source","approvalMode":"single_authorized_human","requiredEvidence":[]}]}}"#).unwrap();
    start_work_item_with_options(
        &path,
        id,
        "preserve reviewed start",
        "reconstruct actual completion without issuing authority",
        &[".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            required_evidence_classes: vec!["verification".into()],
            ..Default::default()
        },
    )
    .unwrap();
    prepare_without_finalization_plan(&authority, &path, id);
    let reviewed = crate::human_review_binding(&path, id).unwrap();
    git_commit(
        &path,
        &[".ai"],
        "actual checkpoint and exact protected review before verification",
    );
    let parent = crate::git_text(&path, &["rev-parse", "HEAD"]).unwrap();
    assert!(
        crate::git_text(
            &path,
            &[
                "show",
                &format!("{parent}:.ai/evidence/{id}.verification.json")
            ]
        )
        .is_none(),
        "this parent must force the actual historical verification recorder branch"
    );
    let runtime = RuntimeContext {
        runtime_version: "0.2.1".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"policy-history-native-fixture-runtime"),
    };
    let execution = run_repository_verification(
        &path,
        &crate::RepositoryVerificationRequest {
            node_id: "policy-history-check".into(),
            program: "true".into(),
            args: vec![],
            scope: vec!["**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: Some(id.into()),
            timeout_seconds: None,
            policy: crate::RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .unwrap();
    record_verification_with_runtime(
        &path,
        id,
        &serde_json::to_value(&execution.receipt).unwrap(),
        &runtime,
        &execution.final_snapshot,
    )
    .unwrap();
    assert_eq!(
        crate::human_review_binding(&path, id).unwrap(),
        reviewed,
        "live completion keeps exact reviewed C/P/R/context"
    );
    crate::finish_work_item_with_runtime(&path, id, &runtime).unwrap();
    git_commit(
        &path,
        &[".ai"],
        "actual typed verification and finish together",
    );
    let finish = crate::git_text(&path, &["rev-parse", "HEAD"]).unwrap();
    crate::archive_work_item_with_runtime(&path, id, &runtime).unwrap();
    git_commit(&path, &[".ai"], "actual archive after finish");
    let archive = crate::git_text(&path, &["rev-parse", "HEAD"]).unwrap();
    let before = super::material_review::segmented_governance_bytes(&path);
    crate::resource_lifecycle::validate_layout_finish_semantics_for_test(
        &path, id, &parent, &finish, &archive,
    )
    .expect(
        "same native verification/finish history reconstructs the original approved start basis",
    );
    assert_eq!(
        super::material_review::segmented_governance_bytes(&path),
        before,
        "historical reconstruction leaves its source bytes unchanged"
    );
    if source_faults {
        crate::resource_lifecycle::project_captured_verification_for_test(
            &path, id, &parent, &finish,
        )
        .expect("the direct real historical recorder accepts the actual earlier protected source");
        let evidence: serde_json::Value = serde_json::from_slice(
            &fs::read(path.join(format!(".ai/evidence/{id}.verification.json"))).unwrap(),
        )
        .unwrap();
        let completed_at = evidence["createdAt"].as_str().unwrap();
        for mode in ["late", "missing", "unavailable"] {
            authority.set_historical_preflight_fault(Some(mode), completed_at);
            let error = crate::resource_lifecycle::project_captured_verification_for_test(
                &path, id, &parent, &finish,
            )
            .expect_err("the full historical recorder requires the actual earlier protected event")
            .to_string();
            let expected = if mode == "unavailable" {
                "authority_unavailable"
            } else {
                "captured_preflight_review_source_unverified"
            };
            assert!(
                error.contains(expected),
                "{mode}: wrong rejection branch: {error}"
            );
            assert_eq!(
                super::material_review::segmented_governance_bytes(&path),
                before
            );
        }
        authority.set_historical_preflight_fault(None, completed_at);
        crate::resource_lifecycle::project_captured_verification_for_test(
            &path, id, &parent, &finish,
        )
        .expect("restoring the private historical source restores proof, never CURRENT");
    }
    if archive_time_cases {
        let manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(path.join(format!(".ai/work-items/archive/{id}.archive.json"))).unwrap(),
        )
        .unwrap();
        crate::resource_lifecycle::validate_layout_archive_manifest_for_test(
            &path, id, &archive, &manifest,
        )
        .expect("actual native nanosecond manifest passes unchanged");
        for mode in [
            "offset",
            "milliseconds",
            "same_second_reverse_order",
            "recorded_mismatch",
        ] {
            let mut changed = manifest.clone();
            let expected = if mode == "recorded_mismatch" {
                changed["recordedAt"] = serde_json::json!("2000-01-01T00:00:00Z");
                "ordinary archive lifecycle metadata is invalid"
            } else {
                match mode {
                    "offset" => {
                        changed["createdAt"] = serde_json::json!(
                            manifest["createdAt"]
                                .as_str()
                                .unwrap()
                                .replace('Z', "+00:00")
                        )
                    }
                    "milliseconds" => {
                        let created = chrono::DateTime::parse_from_rfc3339(
                            manifest["createdAt"].as_str().unwrap(),
                        )
                        .unwrap();
                        changed["createdAt"] = serde_json::json!(
                            created.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
                        );
                    }
                    "same_second_reverse_order" => {
                        changed["createdAt"] = serde_json::json!("2026-10-20T00:00:00.900000000Z");
                        changed["occurredAt"] = serde_json::json!("2026-10-20T00:00:00.100000000Z");
                        changed["recordedAt"] = changed["occurredAt"].clone();
                    }
                    _ => unreachable!(),
                }
                "ordinary archive actor/time binding is invalid"
            };
            let error = crate::resource_lifecycle::validate_layout_archive_manifest_for_test(
                &path, id, &archive, &changed,
            )
            .unwrap_err()
            .to_string();
            assert!(error.contains(expected), "{mode}: {error}");
            assert_eq!(
                super::material_review::segmented_governance_bytes(&path),
                before
            );
        }
    }
}

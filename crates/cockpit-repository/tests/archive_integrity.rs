use cockpit_core::Digest;
use cockpit_git::GitRepository;
use cockpit_protocol::{
    AssuranceLevel, ConcurrencyBoundary, DataClassification, DelegatedEvidence,
    EvidencePersistence, EvidenceRetention, EvidenceValidity, HumanDecision,
    ResourceFinalizationContext, RuntimeContext,
};
use cockpit_repository::{
    ActiveArtifactReconciliationReceipt, WorkItemStartOptions, acquire_parallel_slot,
    archive_work_item, attach, checkpoint_work_item, close_work_item_with_decision,
    close_work_item_with_structured_decision, evidence_purge_plan, evidence_state_for_contract,
    export_audit_events, finish_work_item, governance_decision_for_contract,
    implementation_approach, import_delegated_evidence, plan_resource_finalization,
    preflight_work_item, reconcile_active_artifacts, record_verification, release_parallel_slot,
    render_human_outcome, set_evidence_retention_policy, set_work_item_concurrency_boundary,
    set_work_item_intelligence, start_work_item, start_work_item_with_options, status,
};
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_REPOSITORY_ID: AtomicU64 = AtomicU64::new(0);

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
    attach(&path).expect("attach");
    path
}

fn prepare_for_verification(path: &std::path::Path, work_item_id: &str) {
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
    prepare_without_finalization_plan(path, work_item_id);
}

fn prepare_without_finalization_plan(path: &std::path::Path, work_item_id: &str) {
    let contract = path
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let decision = preflight_work_item(path, &contract).expect("preflight");
    assert_ne!(decision.state, cockpit_core::DecisionState::Red);
    checkpoint_work_item(path, work_item_id).expect("checkpoint");
}

#[test]
fn finish_rejects_provisional_resource_context_before_finish_ready() {
    let path = repository();
    let work_item_id = "WI-FINISH-REQUIRES-PLAN";
    start_work_item(
        &path,
        work_item_id,
        "require explicit resource finalization",
        "fail closed before finish creates an unarchivable state",
        &["**".into()],
    )
    .expect("start");
    prepare_without_finalization_plan(&path, work_item_id);
    record_verification(
        &path,
        work_item_id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.23",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");

    let error = finish_work_item(&path, work_item_id)
        .expect_err("provisional resource context must block finish");
    assert!(error.to_string().contains("non-provisional"));
    let summary: serde_json::Value = serde_json::from_slice(
        &fs::read(
            path.join(".ai/work-items/active")
                .join(format!("{work_item_id}.summary.json")),
        )
        .expect("summary"),
    )
    .expect("summary JSON");
    assert_eq!(summary["state"], "checkpointed");
    assert!(
        !path
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.task-report.json"))
            .exists()
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn finalize_plan_replaces_partial_provisional_context_before_finish() {
    let path = repository();
    let work_item_id = "WI-PARTIAL-FINALIZATION-PLAN";
    start_work_item(
        &path,
        work_item_id,
        "bind a reviewed resource",
        "replace a partially observed finalization context",
        &["**".into()],
    )
    .expect("start");

    // A provider/base observation may be available before the reviewed PR URL
    // exists.  It remains provisional until every identity field is bound.
    plan_resource_finalization(
        &path,
        work_item_id,
        &ResourceFinalizationContext {
            branch: format!("feature/{work_item_id}"),
            worktree: path.display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: "unknown".into(),
        },
    )
    .expect("partial context remains provisional");

    let complete = ResourceFinalizationContext {
        branch: format!("feature/{work_item_id}"),
        worktree: path.display().to_string(),
        base_branch: "main".into(),
        base_remote: "origin".into(),
        provider: "github".into(),
        pull_request: "https://github.com/example/ai-cockpit/pull/423".into(),
    };
    plan_resource_finalization(&path, work_item_id, &complete)
        .expect("complete context replaces the provisional observation");

    let contract: serde_json::Value = serde_json::from_slice(
        &fs::read(
            path.join(".ai/work-items/active")
                .join(format!("{work_item_id}.contract.json")),
        )
        .expect("contract"),
    )
    .expect("contract JSON");
    assert_eq!(
        contract["resourceContext"]["pullRequest"],
        complete.pull_request
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn archive_rejects_provisional_resource_context_without_moving_active_bytes() {
    let path = repository();
    let work_item_id = "WI-ARCHIVE-REQUIRES-PLAN";
    start_work_item(
        &path,
        work_item_id,
        "require explicit resource finalization",
        "fail closed before archive moves active bytes",
        &["**".into()],
    )
    .expect("start");

    prepare_for_verification(&path, work_item_id);
    record_verification(
        &path,
        work_item_id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.23",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, work_item_id).expect("finish");

    let contract_path = path
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract"))
            .expect("contract JSON");
    contract["resourceContext"]["baseBranch"] = serde_json::json!("unknown");
    contract["resourceContext"]["baseRemote"] = serde_json::json!("unknown");
    contract["resourceContext"]["provider"] = serde_json::json!("unknown");
    contract["resourceContext"]["pullRequest"] = serde_json::json!("unknown");
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract).expect("contract bytes"),
    )
    .expect("tamper contract to provisional context");

    let active = path.join(".ai/work-items/active");
    let before = ["contract.json", "summary.json", "outcome.json"].map(|suffix| {
        fs::read(active.join(format!("{work_item_id}.{suffix}")))
            .unwrap_or_else(|error| panic!("read active {suffix}: {error}"))
    });

    let error = archive_work_item(&path, work_item_id)
        .expect_err("provisional resource context must block archive");
    assert!(error.to_string().contains("non-provisional"));
    for (suffix, expected) in ["contract.json", "summary.json", "outcome.json"]
        .into_iter()
        .zip(before)
    {
        assert_eq!(
            fs::read(active.join(format!("{work_item_id}.{suffix}"))).expect("active bytes"),
            expected,
            "archive rejection changed active {suffix} bytes"
        );
        assert!(
            !path
                .join(".ai/work-items/archive")
                .join(format!("{work_item_id}.{suffix}"))
                .exists(),
            "archive rejection created {suffix}"
        );
    }
    assert!(
        !path
            .join(".ai/work-items/archive")
            .join(format!("{work_item_id}.archive.json"))
            .exists()
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn finish_rejects_pending_provider_context_without_moving_active_bytes() {
    let path = repository();
    let work_item_id = "WI-FINISH-REJECTS-PENDING";
    start_work_item(
        &path,
        work_item_id,
        "reject pending provider context",
        "keep unfinished Work Items recoverable",
        &["**".into()],
    )
    .expect("start");
    prepare_for_verification(&path, work_item_id);
    record_verification(
        &path,
        work_item_id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.23",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");

    let contract_path = path
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract"))
            .expect("contract JSON");
    contract["resourceContext"]["pullRequest"] =
        serde_json::json!("pending:WI-FINISH-REJECTS-PENDING");
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract).expect("contract bytes"),
    )
    .expect("tamper contract to pending context");

    let error = finish_work_item(&path, work_item_id)
        .expect_err("pending provider context must block finish");
    assert!(error.to_string().contains("non-provisional"));
    assert!(
        path.join(".ai/work-items/active")
            .join(format!("{work_item_id}.summary.json"))
            .exists()
    );
    assert!(
        !path
            .join(".ai/work-items/archive")
            .join(format!("{work_item_id}.archive.json"))
            .exists()
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn finish_rejects_bare_pending_provider_context_without_moving_active_bytes() {
    let path = repository();
    let work_item_id = "WI-FINISH-REJECTS-BARE-PENDING";
    start_work_item(
        &path,
        work_item_id,
        "reject bare pending provider context",
        "keep unfinished Work Items recoverable",
        &["**".into()],
    )
    .expect("start");
    prepare_for_verification(&path, work_item_id);
    record_verification(
        &path,
        work_item_id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.23",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");

    let contract_path = path
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract"))
            .expect("contract JSON");
    contract["resourceContext"]["pullRequest"] = serde_json::json!("pending");
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract).expect("contract bytes"),
    )
    .expect("tamper contract to bare pending context");

    let error = finish_work_item(&path, work_item_id)
        .expect_err("bare pending provider context must block finish");
    assert!(error.to_string().contains("non-provisional"));
    assert!(
        path.join(".ai/work-items/active")
            .join(format!("{work_item_id}.summary.json"))
            .exists()
    );
    assert!(
        !path
            .join(".ai/work-items/archive")
            .join(format!("{work_item_id}.archive.json"))
            .exists()
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn successful_finish_clears_stale_failed_projection_after_recovery() {
    let path = repository();
    let work_item_id = "WI-FINISH-CLEARS-STALE-FAILURE";
    start_work_item(
        &path,
        work_item_id,
        "clear stale finish projection",
        "allow repaired Work Items to pass the CI lifecycle gate",
        &["**".into()],
    )
    .expect("start");
    prepare_for_verification(&path, work_item_id);
    record_verification(
        &path,
        work_item_id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.23",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");

    let summary_path = path
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let mut summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("summary")).expect("summary JSON");
    summary["failedGate"] = serde_json::json!("finish.governance");
    summary["recoveryCondition"] = serde_json::json!("retry after repairing the lifecycle gate");
    summary["outcomeState"] = serde_json::json!("blocked");
    fs::write(
        &summary_path,
        serde_json::to_vec_pretty(&summary).expect("summary bytes"),
    )
    .expect("seed stale failure projection");

    finish_work_item(&path, work_item_id).expect("fresh successful finish");
    let repaired: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("repaired summary"))
            .expect("repaired summary JSON");
    assert_eq!(repaired["state"], "finish_ready");
    assert!(repaired.get("failedGate").is_none());
    assert!(repaired.get("recoveryCondition").is_none());
    assert!(repaired.get("outcomeState").is_none());
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn archive_accepts_explicit_non_provisional_resource_finalization_plan() {
    let path = repository();
    let work_item_id = "WI-ARCHIVE-WITH-PLAN";
    start_work_item(
        &path,
        work_item_id,
        "bind resource finalization",
        "archive only after the reviewed resource is identified",
        &["**".into()],
    )
    .expect("start");

    prepare_for_verification(&path, work_item_id);
    record_verification(
        &path,
        work_item_id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.23",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, work_item_id).expect("finish");
    archive_work_item(&path, work_item_id).expect("archive after finalization plan");

    let contract: serde_json::Value = serde_json::from_slice(
        &fs::read(
            path.join(".ai/work-items/archive")
                .join(format!("{work_item_id}.contract.json")),
        )
        .expect("archived contract"),
    )
    .expect("contract JSON");
    assert_eq!(contract["resourceContext"]["baseBranch"], "main");
    assert_eq!(contract["resourceContext"]["baseRemote"], "origin");
    assert_eq!(contract["resourceContext"]["provider"], "github");
    assert!(
        contract["resourceContext"]["pullRequest"]
            .as_str()
            .expect("pull request")
            .contains(work_item_id)
    );
    assert!(
        !path
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.contract.json"))
            .exists()
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn close_rejects_tampered_archived_artifacts() {
    let path = repository();
    start_work_item(&path, "WI-INTEGRITY", "integrity", "verify", &["**".into()]).expect("start");
    prepare_for_verification(&path, "WI-INTEGRITY");
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
    let error = close_work_item_with_decision(&path, "WI-INTEGRITY", "approved")
        .expect_err("tampered archive must be rejected");
    assert!(error.to_string().contains("digest does not match manifest"));
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn archive_rejects_tampered_verification_even_without_declared_requirement() {
    let path = repository();
    start_work_item(
        &path,
        "WI-EVIDENCE-TAMPER",
        "tamper",
        "verify",
        &["**".into()],
    )
    .expect("start");
    prepare_for_verification(&path, "WI-EVIDENCE-TAMPER");
    record_verification(
        &path,
        "WI-EVIDENCE-TAMPER",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.1.0",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, "WI-EVIDENCE-TAMPER").expect("finish");
    let evidence_path = path.join(".ai/evidence/WI-EVIDENCE-TAMPER.verification.json");
    let mut evidence: serde_json::Value =
        serde_json::from_slice(&fs::read(&evidence_path).expect("evidence"))
            .expect("evidence JSON");
    evidence["passed"] = serde_json::Value::Bool(false);
    fs::write(
        &evidence_path,
        serde_json::to_vec_pretty(&evidence).expect("tampered evidence"),
    )
    .expect("write tampered evidence");
    let error = archive_work_item(&path, "WI-EVIDENCE-TAMPER")
        .expect_err("archive must fail closed on tampered evidence");
    assert!(error.to_string().contains("valid verification evidence"));
    assert!(
        path.join(".ai/work-items/active/WI-EVIDENCE-TAMPER.contract.json")
            .is_file()
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn archive_moves_implementation_approach_and_removes_active_orphan() {
    let path = repository();
    start_work_item(
        &path,
        "WI-APPROACH-ARCHIVE",
        "approach",
        "archive generated approach",
        &["**".into()],
    )
    .expect("start");
    implementation_approach(&path, "WI-APPROACH-ARCHIVE").expect("approach");
    assert!(
        path.join(".ai/work-items/active/WI-APPROACH-ARCHIVE.approach.json")
            .is_file()
    );
    prepare_for_verification(&path, "WI-APPROACH-ARCHIVE");
    record_verification(
        &path,
        "WI-APPROACH-ARCHIVE",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.1.0",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, "WI-APPROACH-ARCHIVE").expect("finish");
    archive_work_item(&path, "WI-APPROACH-ARCHIVE").expect("archive");

    assert!(
        !path
            .join(".ai/work-items/active/WI-APPROACH-ARCHIVE.approach.json")
            .exists()
    );
    let archived = path.join(".ai/work-items/archive/WI-APPROACH-ARCHIVE.approach.json");
    assert!(archived.is_file());
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(path.join(".ai/work-items/archive/WI-APPROACH-ARCHIVE.archive.json"))
            .expect("manifest"),
    )
    .expect("manifest JSON");
    let digest = Digest::sha256_bytes(&fs::read(&archived).expect("archived approach"));
    assert_eq!(manifest["files"]["approachDigest"], digest.to_string());
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn archive_moves_failed_attempt_variants_and_binds_their_digests() {
    let path = repository();
    let work_item_id = "WI-FAILED-ATTEMPT-VARIANTS";
    start_work_item(
        &path,
        work_item_id,
        "preserve failed lifecycle attempts",
        "move historical blocked projections out of active during archive",
        &["**".into()],
    )
    .expect("start");
    prepare_for_verification(&path, work_item_id);
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
    let error = close_work_item_with_decision(&path, work_item_id, "approved")
        .expect_err("tampered historical variant must be rejected");
    assert!(error.to_string().contains("historical artifact digest"));
    fs::remove_dir_all(path).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn archive_rejects_symlinked_failed_attempt_variant() {
    use std::os::unix::fs::symlink;

    let path = repository();
    let work_item_id = "WI-FAILED-ATTEMPT-SYMLINK";
    start_work_item(
        &path,
        work_item_id,
        "reject unsafe historical projection",
        "do not follow a failed-attempt symlink during archive",
        &["**".into()],
    )
    .expect("start");

    let target = path.join("outside-history.json");
    fs::write(&target, br#"{"foreign":true}"#).expect("target");
    let variant = path
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.outcome.finish-blocked.json"));
    symlink(&target, &variant).expect("variant symlink");

    prepare_for_verification(&path, work_item_id);
    record_verification(
        &path,
        work_item_id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.23",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, work_item_id).expect("finish");

    let error = archive_work_item(&path, work_item_id)
        .expect_err("archive must reject symlinked historical projection");
    assert!(error.to_string().contains("regular non-symlink"));
    assert!(
        fs::symlink_metadata(&variant)
            .expect("variant metadata")
            .file_type()
            .is_symlink()
    );
    assert!(
        !path
            .join(".ai/work-items/archive")
            .join(format!("{work_item_id}.archive.json"))
            .exists()
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn status_reports_orphaned_active_variants_without_counting_them_as_work_items() {
    let path = repository();
    let work_item_id = "WI-ORPHAN-VARIANT-STATUS";
    start_work_item(
        &path,
        work_item_id,
        "report active residue",
        "make orphaned failed attempts visible",
        &["**".into()],
    )
    .expect("start");
    prepare_for_verification(&path, work_item_id);
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
    let name = format!("{work_item_id}.outcome.finish-blocked.json");
    fs::write(
        path.join(".ai/work-items/active").join(&name),
        br#"{"state":"blocked"}"#,
    )
    .expect("orphan variant");

    let repository_status = status(&path).expect("status");
    assert_eq!(repository_status.active_work_items, 0);
    assert_eq!(repository_status.active_artifacts, vec![name.clone()]);
    assert_eq!(repository_status.orphaned_active_artifacts, vec![name]);
    assert!(
        repository_status
            .readiness
            .blockers
            .contains(&"orphaned_active_artifacts_present".into())
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn reconcile_moves_variants_for_an_already_archived_work_item() {
    let path = repository();
    let work_item_id = "WI-ORPHAN-VARIANT-RECONCILE";
    start_work_item(
        &path,
        work_item_id,
        "reconcile archived residue",
        "move old failed attempts without rewriting archive truth",
        &["**".into()],
    )
    .expect("start");
    prepare_for_verification(&path, work_item_id);
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
    let name = format!("{work_item_id}.events.historical.jsonl");
    let bytes = br#"{"schemaVersion":1,"eventType":"blocked"}
"#;
    fs::write(path.join(".ai/work-items/active").join(&name), bytes).expect("orphan variant");
    let original_manifest = fs::read(
        path.join(".ai/work-items/archive")
            .join(format!("{work_item_id}.archive.json")),
    )
    .expect("manifest before reconcile");

    let receipt: ActiveArtifactReconciliationReceipt =
        reconcile_active_artifacts(&path, work_item_id).expect("reconcile");
    assert_eq!(receipt.state, "reconciled");
    assert_eq!(receipt.moved_artifacts.len(), 1);
    assert!(!path.join(".ai/work-items/active").join(&name).exists());
    assert_eq!(
        fs::read(path.join(".ai/work-items/archive").join(&name)).expect("archived variant"),
        bytes
    );
    assert_eq!(
        fs::read(
            path.join(".ai/work-items/archive")
                .join(format!("{work_item_id}.archive.json")),
        )
        .expect("manifest after reconcile"),
        original_manifest,
        "archive truth remains immutable"
    );
    let receipt_path = path
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.artifact-reconciliation.json"));
    assert!(receipt_path.is_file());
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn archive_rewrites_generated_outcome_references_to_archive_paths() {
    let path = repository();
    let work_item_id = "WI-ARCHIVE-PROJECTION";
    start_work_item(
        &path,
        work_item_id,
        "archive projection",
        "keep generated Outcome references valid after archive",
        &["**".into()],
    )
    .expect("start");
    prepare_for_verification(&path, work_item_id);
    record_verification(
        &path,
        work_item_id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.15",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, work_item_id).expect("finish");
    archive_work_item(&path, work_item_id).expect("archive");

    let archive = path.join(".ai/work-items/archive");
    let active_reference = format!(".ai/work-items/active/{work_item_id}");
    let archive_reference = format!(".ai/work-items/archive/{work_item_id}");
    for suffix in [
        "outcome.json",
        "summary.json",
        "task-report.json",
        "task-report.md",
        "events.jsonl",
    ] {
        let bytes = fs::read(archive.join(format!("{work_item_id}.{suffix}")))
            .unwrap_or_else(|error| panic!("read archived {suffix}: {error}"));
        let text = String::from_utf8(bytes).expect("archived artifact is UTF-8");
        assert!(
            !text.contains(&active_reference),
            "archived {suffix} still references a removed active artifact"
        );
        if matches!(suffix, "outcome.json" | "task-report.json" | "events.jsonl") {
            assert!(
                text.contains(&archive_reference),
                "archived {suffix} does not expose its archive reference"
            );
        }
    }

    let handoff = render_human_outcome(
        &path,
        &cockpit_repository::outcome_v2(&path, work_item_id).expect("outcome"),
        "en",
    );
    assert!(!handoff.contains(&active_reference));

    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(archive.join(format!("{work_item_id}.archive.json"))).expect("manifest"),
    )
    .expect("manifest JSON");
    for (name, suffix) in [
        ("outcome", "outcome.json"),
        ("summary", "summary.json"),
        ("taskReport", "task-report.json"),
        ("taskReportMarkdown", "task-report.md"),
        ("events", "events.jsonl"),
    ] {
        let bytes =
            fs::read(archive.join(format!("{work_item_id}.{suffix}"))).expect("archived artifact");
        assert_eq!(
            manifest["files"][format!("{name}Digest")],
            Digest::sha256_bytes(&bytes).to_string(),
            "manifest digest for {name}"
        );
    }

    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn archive_moves_parallel_intelligence_sidecar_and_binds_digest() {
    let path = repository();
    start_work_item(
        &path,
        "WI-INTELLIGENCE-ARCHIVE",
        "intelligence",
        "archive",
        &["**".into()],
    )
    .expect("start");
    set_work_item_intelligence(
        &path,
        "WI-INTELLIGENCE-ARCHIVE",
        vec!["WI-DEPENDENCY".into()],
        Vec::new(),
        true,
    )
    .expect("intelligence");
    prepare_for_verification(&path, "WI-INTELLIGENCE-ARCHIVE");
    record_verification(
        &path,
        "WI-INTELLIGENCE-ARCHIVE",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.1.0",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, "WI-INTELLIGENCE-ARCHIVE").expect("finish");
    archive_work_item(&path, "WI-INTELLIGENCE-ARCHIVE").expect("archive");

    let active = path.join(".ai/work-items/active/WI-INTELLIGENCE-ARCHIVE.intelligence.json");
    let archived = path.join(".ai/work-items/archive/WI-INTELLIGENCE-ARCHIVE.intelligence.json");
    assert!(!active.exists());
    assert!(archived.is_file());
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(path.join(".ai/work-items/archive/WI-INTELLIGENCE-ARCHIVE.archive.json"))
            .expect("manifest"),
    )
    .expect("manifest JSON");
    assert_eq!(
        manifest["files"]["intelligenceDigest"],
        Digest::sha256_bytes(&fs::read(&archived).expect("archived intelligence")).to_string()
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn archive_requires_releasing_active_parallel_slot() {
    let path = repository();
    start_work_item(
        &path,
        "WI-LEASE-ARCHIVE",
        "lease",
        "archive",
        &["src/main.rs".into()],
    )
    .expect("start");
    set_work_item_intelligence(&path, "WI-LEASE-ARCHIVE", Vec::new(), Vec::new(), true)
        .expect("intelligence");
    set_work_item_concurrency_boundary(
        &path,
        "WI-LEASE-ARCHIVE",
        ConcurrencyBoundary {
            schema_version: 1,
            implementation_paths: vec!["src/main.rs".into()],
            generated_evidence_paths: vec![
                ".ai/evidence/WI-LEASE-ARCHIVE.verification.json".into(),
            ],
            verification_output_paths: vec!["target/lease-archive".into()],
            serialized_projection_paths: vec![".ai/work-items/active/WI-LEASE-ARCHIVE".into()],
            max_workers: 1,
            reason: "archive lease test".into(),
        },
    )
    .expect("boundary");
    let lease = acquire_parallel_slot(&path, "WI-LEASE-ARCHIVE").expect("lease");
    prepare_for_verification(&path, "WI-LEASE-ARCHIVE");
    record_verification(
        &path,
        "WI-LEASE-ARCHIVE",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.1.0",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, "WI-LEASE-ARCHIVE").expect("finish");
    let error =
        archive_work_item(&path, "WI-LEASE-ARCHIVE").expect_err("active lease blocks archive");
    assert!(error.to_string().contains("parallel slot"));
    assert!(
        path.join(".ai/work-items/active/WI-LEASE-ARCHIVE.contract.json")
            .is_file()
    );
    release_parallel_slot(&path, "WI-LEASE-ARCHIVE", &lease.lease_id).expect("release");
    archive_work_item(&path, "WI-LEASE-ARCHIVE").expect("archive after release");
    fs::remove_dir_all(path).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn archive_rejects_dangling_implementation_approach_symlink() {
    use std::os::unix::fs::symlink;

    let path = repository();
    start_work_item(
        &path,
        "WI-APPROACH-SYMLINK",
        "approach",
        "archive",
        &["**".into()],
    )
    .expect("start");
    symlink(
        "missing-approach.json",
        path.join(".ai/work-items/active/WI-APPROACH-SYMLINK.approach.json"),
    )
    .expect("symlink");
    prepare_for_verification(&path, "WI-APPROACH-SYMLINK");
    record_verification(
        &path,
        "WI-APPROACH-SYMLINK",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.1.0",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, "WI-APPROACH-SYMLINK").expect("finish");
    let error = archive_work_item(&path, "WI-APPROACH-SYMLINK")
        .expect_err("dangling approach symlink must fail closed");
    assert!(error.to_string().contains("Implementation approach"));
    assert!(
        fs::symlink_metadata(path.join(".ai/work-items/active/WI-APPROACH-SYMLINK.approach.json"))
            .is_ok()
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn verification_receipt_cannot_cross_work_items() {
    let path = repository();
    start_work_item(&path, "WI-A", "first", "verify", &["**".into()]).expect("start A");
    start_work_item(&path, "WI-B", "second", "verify", &["**".into()]).expect("start B");
    prepare_for_verification(&path, "WI-A");
    prepare_for_verification(&path, "WI-B");
    let error = record_verification(
        &path,
        "WI-B",
        &serde_json::json!({"passed": true, "workItemId": "WI-A", "nodesPlanned": 1}),
        "0.1.0",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect_err("cross-work-item evidence must be rejected");
    assert!(error.to_string().contains("another work item"));
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn close_persists_a_structured_human_decision_and_recovery_condition() {
    let path = repository();
    start_work_item(&path, "WI-DECISION", "decision", "verify", &["**".into()]).expect("start");
    prepare_for_verification(&path, "WI-DECISION");
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
    close_work_item_with_structured_decision(
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
    let path = repository();
    start_work_item(
        &path,
        "WI-ARCHIVE-MERGE",
        "archive",
        "close",
        &["**".into()],
    )
    .expect("start");
    prepare_for_verification(&path, "WI-ARCHIVE-MERGE");
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

    close_work_item_with_structured_decision(
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
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            required_evidence_classes: vec!["verification".into(), "delegated:github".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    prepare_for_verification(&path, "WI-POLICY");
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
    record_verification(
        &path,
        "WI-POLICY",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.1",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(&path, "WI-POLICY").expect("finish");
    archive_work_item(&path, "WI-POLICY").expect("archive");

    let missing_binding = close_work_item_with_structured_decision(
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

    close_work_item_with_structured_decision(
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
fn preflight_exposes_policy_authority_and_evidence_gaps() {
    let path = repository();
    fs::write(
        path.join(".ai/policy.json"),
        r#"{
          "schemaVersion": 1,
          "organization": {
            "policyId": "org-production-v1",
            "layer": "organization",
            "rules": [{
              "operation": "modify_source",
              "approvalMode": "single_authorized_human",
              "requiredEvidence": ["hosted_ci"]
            }]
          }
        }"#,
    )
    .expect("policy");
    start_work_item(
        &path,
        "WI-PREFLIGHT-POLICY",
        "policy",
        "verify",
        &["**".into()],
    )
    .expect("start");
    let contract: cockpit_protocol::Contract = serde_json::from_slice(
        &fs::read(path.join(".ai/work-items/active/WI-PREFLIGHT-POLICY.contract.json"))
            .expect("contract"),
    )
    .expect("parse contract");
    let snapshot = GitRepository::discover(&path)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let decision = governance_decision_for_contract(&path, &contract, &snapshot).expect("decision");
    assert_eq!(decision.state, cockpit_core::DecisionState::Yellow);
    assert!(
        decision
            .unknowns
            .contains(&"human_authority_missing".into())
    );
    assert!(
        decision
            .unknowns
            .contains(&"policy_required_evidence_missing".into())
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn digest_only_retention_never_persists_command_output() {
    let path = repository();
    start_work_item(&path, "WI-DIGEST", "digest", "verify", &["**".into()]).expect("start");
    prepare_for_verification(&path, "WI-DIGEST");
    set_evidence_retention_policy(
        &path,
        "WI-DIGEST",
        EvidenceRetention {
            classification: DataClassification::Restricted,
            persistence: EvidencePersistence::DigestOnly,
            retention_days: Some(30),
            expires_at: None,
            disposal_action: "purge_after_expiry".into(),
        },
        &RuntimeContext {
            runtime_version: "0.2.2".into(),
            protocol_version: 1,
            runtime_digest: Digest::sha256_bytes(b"runtime"),
        },
    )
    .expect("retention policy");
    let evidence = record_verification(
        &path,
        "WI-DIGEST",
        &serde_json::json!({
            "passed": true,
            "nodesPlanned": 1,
            "output": "credential=do-not-persist"
        }),
        "0.2.2",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    assert_eq!(evidence["captureMode"], "digest_only");
    assert!(evidence.get("receipt").is_none());
    let bytes = fs::read_to_string(path.join(".ai/evidence/WI-DIGEST.verification.json"))
        .expect("stored evidence");
    assert!(!bytes.contains("do-not-persist"));
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn no_persistence_fails_closed_instead_of_claiming_completion() {
    let path = repository();
    start_work_item(
        &path,
        "WI-NOPERSIST",
        "no persist",
        "verify",
        &["**".into()],
    )
    .expect("start");
    prepare_for_verification(&path, "WI-NOPERSIST");
    set_evidence_retention_policy(
        &path,
        "WI-NOPERSIST",
        EvidenceRetention {
            classification: DataClassification::SecretProhibited,
            persistence: EvidencePersistence::NoPersistence,
            retention_days: Some(1),
            expires_at: None,
            disposal_action: "external_owner".into(),
        },
        &RuntimeContext {
            runtime_version: "0.2.2".into(),
            protocol_version: 1,
            runtime_digest: Digest::sha256_bytes(b"runtime"),
        },
    )
    .expect("retention policy");
    let error = record_verification(
        &path,
        "WI-NOPERSIST",
        &serde_json::json!({"passed": true}),
        "0.2.2",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect_err("completion evidence must not be silently discarded");
    assert!(error.to_string().contains("no_persistence"));
    assert!(
        !path
            .join(".ai/evidence/WI-NOPERSIST.verification.json")
            .exists()
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn purge_plan_is_deterministic_and_never_deletes_evidence() {
    let path = repository();
    start_work_item(&path, "WI-EXPIRE", "expiry", "verify", &["**".into()]).expect("start");
    prepare_for_verification(&path, "WI-EXPIRE");
    set_evidence_retention_policy(
        &path,
        "WI-EXPIRE",
        EvidenceRetention {
            classification: DataClassification::Confidential,
            persistence: EvidencePersistence::DigestOnly,
            retention_days: None,
            expires_at: Some("0".into()),
            disposal_action: "purge_after_review".into(),
        },
        &RuntimeContext {
            runtime_version: "0.2.2".into(),
            protocol_version: 1,
            runtime_digest: Digest::sha256_bytes(b"runtime"),
        },
    )
    .expect("retention policy");
    record_verification(
        &path,
        "WI-EXPIRE",
        &serde_json::json!({"passed": true}),
        "0.2.2",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    let first = evidence_purge_plan(&path, 1_700_000_000).expect("purge plan");
    let second = evidence_purge_plan(&path, 1_700_000_000).expect("purge plan");
    assert_eq!(first, second);
    assert_eq!(
        first[0].disposition,
        cockpit_protocol::EvidenceDisposition::PurgePlanned
    );
    assert!(
        path.join(".ai/evidence/WI-EXPIRE.verification.json")
            .is_file()
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn audit_export_is_deterministic_and_marks_external_retention_boundary() {
    let path = repository();
    start_work_item(&path, "WI-AUDIT", "audit", "export", &["**".into()]).expect("start");
    prepare_for_verification(&path, "WI-AUDIT");
    record_verification(
        &path,
        "WI-AUDIT",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.2",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    let runtime = RuntimeContext {
        runtime_version: "0.2.2".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"runtime"),
    };
    let first = export_audit_events(&path, &runtime).expect("audit export");
    let second = export_audit_events(&path, &runtime).expect("audit export");
    assert_eq!(first, second);
    assert!(first.external_retention_required);
    assert_eq!(first.events.len(), 1);
    assert_eq!(first.events[0].runtime_version, "0.2.2");
    assert_eq!(first.events[0].work_item_id.as_deref(), Some("WI-AUDIT"));
    assert!(first.events[0].event_id.starts_with("sha256:"));
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn delegated_evidence_import_binds_raw_digest_and_work_item() {
    let path = repository();
    start_work_item(
        &path,
        "WI-EXTERNAL",
        "external",
        "bind evidence",
        &["**".into()],
    )
    .expect("start");
    let raw = br#"{"provider":"github","run":123}"#;
    let evidence = DelegatedEvidence {
        provider: "github".into(),
        subject: "run:123".into(),
        origin: "https://github.com/example/repo/actions/runs/123".into(),
        assurance: AssuranceLevel::ProviderVerified,
        collected_at: "2026-08-21T19:00:00Z".into(),
        digest: cockpit_core::Digest::sha256_bytes(raw),
        validity: EvidenceValidity::Valid,
        raw_evidence_ref: ".ai/evidence/external/github-run-123.json".into(),
    };
    let runtime = RuntimeContext {
        runtime_version: "0.2.2".into(),
        protocol_version: 1,
        runtime_digest: cockpit_core::Digest::sha256_bytes(b"runtime"),
    };
    let receipt =
        import_delegated_evidence(&path, "WI-EXTERNAL", &evidence, raw, &runtime).expect("import");
    assert_eq!(
        receipt.repository_id,
        cockpit_repository::repository_id(&path).to_string()
    );
    assert_eq!(receipt.work_item_id, "WI-EXTERNAL");
    assert!(
        path.join(".ai/evidence/external/github-run-123.json")
            .is_file()
    );
    assert_eq!(
        fs::read_dir(path.join(".ai/evidence/external"))
            .expect("external evidence")
            .count(),
        2,
        "raw evidence and its binding receipt are both archived"
    );

    let mismatch = DelegatedEvidence {
        digest: cockpit_core::Digest::sha256_bytes(b"different"),
        ..evidence.clone()
    };
    let error = import_delegated_evidence(&path, "WI-EXTERNAL", &mismatch, raw, &runtime)
        .expect_err("digest mismatch must fail closed");
    assert!(error.to_string().contains("digest"));

    let unsafe_ref = DelegatedEvidence {
        raw_evidence_ref: ".ai/evidence/external/../escape.json".into(),
        ..evidence
    };
    let error = import_delegated_evidence(&path, "WI-EXTERNAL", &unsafe_ref, raw, &runtime)
        .expect_err("path escape must fail closed");
    assert!(error.to_string().contains("external"));
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn valid_delegated_evidence_satisfies_a_provider_specific_contract_requirement() {
    let path = repository();
    start_work_item_with_options(
        &path,
        "WI-DELEGATED-REQUIRED",
        "external",
        "require provider evidence",
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            required_evidence_classes: vec!["delegated:github".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
    let raw = br#"{"run":456}"#;
    import_delegated_evidence(
        &path,
        "WI-DELEGATED-REQUIRED",
        &DelegatedEvidence {
            provider: "github".into(),
            subject: "run:456".into(),
            origin: "https://github.com/example/repo/actions/runs/456".into(),
            assurance: AssuranceLevel::ProviderVerified,
            collected_at: "2026-08-21T19:00:00Z".into(),
            digest: cockpit_core::Digest::sha256_bytes(raw),
            validity: EvidenceValidity::Valid,
            raw_evidence_ref: ".ai/evidence/external/github-run-456.json".into(),
        },
        raw,
        &RuntimeContext {
            runtime_version: "0.2.2".into(),
            protocol_version: 1,
            runtime_digest: cockpit_core::Digest::sha256_bytes(b"runtime"),
        },
    )
    .expect("import");
    let contract: cockpit_protocol::Contract = serde_json::from_slice(
        &fs::read(path.join(".ai/work-items/active/WI-DELEGATED-REQUIRED.contract.json"))
            .expect("contract"),
    )
    .expect("parse contract");
    let snapshot = GitRepository::discover(&path)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    assert_eq!(
        evidence_state_for_contract(&path, &contract, &snapshot).expect("evidence state"),
        cockpit_core::EvidenceState::Complete
    );
    fs::remove_dir_all(path).expect("cleanup");
}

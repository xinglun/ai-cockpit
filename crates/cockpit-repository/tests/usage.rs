use cockpit_core::Digest;
use cockpit_protocol::{
    RuntimeContext, UsageAssurance, UsageCoverage, UsageReceipt, UsageRecordRequest,
    UsageSourceKind, UsageUnit,
};
use cockpit_repository::{
    OutcomeRenderView, WorkItemStartOptions, archive_work_item, attach, checkpoint_work_item,
    close_work_item_with_decision, finish_work_item, outcome_render_input, preflight_work_item,
    query_work_item_usage, read_work_item_usage, read_work_item_usage_receipts,
    record_verification, record_work_item_usage, render_human_outcome_with_view,
    start_work_item_with_options, work_item_status_snapshot_with_runtime,
};
use std::{fs, process::Command, sync::Arc};

fn repository() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("tempdir");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(root.path())
            .status()
            .expect("git init")
            .success()
    );
    attach(root.path()).expect("attach");
    fs::write(
        root.path().join(".ai/evidence/source-usage.json"),
        b"{\"source\":\"caller-claim\"}\n",
    )
    .expect("source evidence");
    root
}

fn start(root: &std::path::Path, id: &str) {
    start_work_item_with_options(
        root,
        id,
        "usage fixture",
        "record explicit usage",
        &[".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..Default::default()
        },
    )
    .expect("start work item");
}

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "1.0.1-test".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"usage test runtime"),
    }
}

fn request(root: &std::path::Path, id: &str, event: &str) -> UsageRecordRequest {
    let bytes = b"{\"source\":\"caller-claim\"}\n";
    UsageRecordRequest {
        schema_version: 1,
        repository_id: cockpit_repository::repository_id(root).to_string(),
        work_item_id: id.into(),
        source_event_id: event.into(),
        source_kind: UsageSourceKind::ProviderReported,
        actor: None,
        configured_model: Some("configured-model".into()),
        reported_model: Some("reported-model".into()),
        role: "implementer".into(),
        phase: "implementation".into(),
        unit: UsageUnit::Turn,
        input_tokens: Some(10),
        output_tokens: Some(5),
        cached_input_tokens: Some(3),
        reasoning_tokens: Some(2),
        source_observed_at: None,
        evidence_ref: ".ai/evidence/source-usage.json".into(),
        evidence_digest: Digest::sha256_bytes(bytes),
    }
}

fn finish_with_usage(root: &std::path::Path, id: &str) -> String {
    start(root, id);
    record_work_item_usage(root, &request(root, id, "turn-before-finish"), &runtime())
        .expect("record before finish");
    let contract = root.join(format!(".ai/work-items/active/{id}.contract.json"));
    preflight_work_item(root, &contract).expect("preflight");
    checkpoint_work_item(root, id).expect("checkpoint");
    record_verification(
        root,
        id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "1.0.1-test",
        &Digest::sha256_bytes(b"usage test runtime"),
    )
    .expect("verification");
    finish_work_item(root, id).expect("finish");
    read_work_item_usage(root, id, None)
        .expect("finish usage")
        .receipt_refs[0]
        .path
        .clone()
}

#[test]
fn received_at_tampering_is_rejected_before_cutoff_filtering() {
    let root = repository();
    let id = "WI-USAGE-TIME-TAMPER";
    start(root.path(), id);
    record_work_item_usage(root.path(), &request(root.path(), id, "turn-1"), &runtime())
        .expect("record usage");
    let reference = read_work_item_usage(root.path(), id, None)
        .expect("query original")
        .receipt_refs[0]
        .path
        .clone();
    let path = root.path().join(reference);
    let mut receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("receipt bytes")).expect("receipt JSON");
    receipt["receivedAt"] = "2999-01-01T00:00:00Z".into();
    fs::write(
        &path,
        serde_json::to_vec_pretty(&receipt).expect("tampered JSON"),
    )
    .expect("tamper receivedAt");
    let error = read_work_item_usage(root.path(), id, None)
        .expect_err("receivedAt is bound to the receipt identity before cutoff");
    assert!(
        error.to_string().contains("usage receipt binding"),
        "{error}"
    );
}

#[test]
fn removed_finish_receipt_is_invalid_evidence_not_absent_usage() {
    let root = repository();
    let id = "WI-USAGE-REMOVED-FINISH";
    let reference = finish_with_usage(root.path(), id);
    fs::remove_file(root.path().join(&reference)).expect("remove bound receipt");
    let query_error = read_work_item_usage(root.path(), id, None)
        .expect_err("missing frozen receipt is not no_usage_receipts");
    assert!(
        query_error.to_string().contains("frozen usage"),
        "{query_error}"
    );
    let archive_error = archive_work_item(root.path(), id)
        .expect_err("archive cannot preserve a report with missing bound usage");
    assert!(
        archive_error.to_string().contains("frozen usage"),
        "{archive_error}"
    );
}

#[test]
fn rebound_future_receipt_cannot_escape_the_frozen_finish_cutoff() {
    let root = repository();
    let id = "WI-USAGE-FUTURE-REBOUND";
    let reference = finish_with_usage(root.path(), id);
    let path = root.path().join(reference);
    let mut receipt: UsageReceipt =
        serde_json::from_slice(&fs::read(&path).expect("receipt bytes")).expect("receipt");
    receipt.received_at = "2999-01-01T00:00:00Z".into();
    receipt.receipt_id = Digest::sha256_bytes(
        &serde_json::to_vec(&(
            receipt.schema_version,
            &receipt.request,
            &receipt.received_at,
            &receipt.source_observed_at,
            receipt.model_assurance,
            receipt.token_assurance,
        ))
        .expect("receipt binding"),
    );
    fs::write(
        &path,
        serde_json::to_vec_pretty(&receipt).expect("receipt JSON"),
    )
    .expect("rebind altered receipt");
    let error = read_work_item_usage(root.path(), id, None)
        .expect_err("future receipt cannot disappear behind an old cutoff");
    assert!(error.to_string().contains("frozen usage"), "{error}");
    let error = archive_work_item(root.path(), id)
        .expect_err("archive must reject a shifted finish receipt");
    assert!(error.to_string().contains("frozen usage"), "{error}");
}

#[test]
fn removed_archived_receipt_blocks_close_without_rewriting_archive() {
    let root = repository();
    let id = "WI-USAGE-REMOVED-ARCHIVE";
    let reference = finish_with_usage(root.path(), id);
    archive_work_item(root.path(), id).expect("archive intact usage");
    let archive_path = root
        .path()
        .join(format!(".ai/work-items/archive/{id}.task-report.json"));
    let archive_bytes = fs::read(&archive_path).expect("frozen report");
    fs::remove_file(root.path().join(&reference)).expect("remove receipt after archive");
    let error = close_work_item_with_decision(root.path(), id, "approved")
        .expect_err("close cannot turn missing bound usage into empty totals");
    assert!(error.to_string().contains("frozen usage"), "{error}");
    assert_eq!(
        fs::read(&archive_path).expect("archive bytes"),
        archive_bytes
    );
}

fn closed_with_late_usage(root: &std::path::Path, id: &str) -> (String, Vec<u8>) {
    let finish_reference = finish_with_usage(root, id);
    archive_work_item(root, id).expect("archive finish report");
    record_work_item_usage(root, &request(root, id, "turn-after-archive"), &runtime())
        .expect("record late usage before close");
    close_work_item_with_decision(root, id, "approved").expect("close with late usage");
    let close_path = root.join(format!(".ai/decisions/{id}.close.json"));
    let close_bytes = fs::read(&close_path).expect("frozen close bytes");
    let refs = read_work_item_usage(root, id, None)
        .expect("normal closed usage is readable")
        .receipt_refs;
    assert_eq!(refs.len(), 2);
    let late = refs
        .into_iter()
        .find(|reference| reference.path != finish_reference)
        .expect("late receipt reference");
    (late.path, close_bytes)
}

#[test]
fn closed_outcome_uses_the_validated_final_usage_snapshot() {
    let root = repository();
    let id = "WI-USAGE-CLOSED-OUTCOME";
    let (_, close_bytes) = closed_with_late_usage(root.path(), id);
    let close: serde_json::Value = serde_json::from_slice(&close_bytes).expect("close JSON");
    let input = outcome_render_input(root.path(), id).expect("public Outcome input");
    let usage = input
        .outcome
        .task_outcome_report
        .as_ref()
        .and_then(|report| report.usage.as_ref())
        .expect("public Outcome usage");
    assert_eq!(usage.receipt_refs.len(), 2);
    assert_eq!(usage.totals.input_tokens, Some(20));
    assert_eq!(
        usage.cutoff,
        close["finalReport"]["usage"]["cutoff"]
            .as_str()
            .expect("close cutoff")
    );
    for (language, label, record_label) in [
        ("en", "Usage", "records: 2"),
        ("zh", "用量", "记录数: 2"),
        ("ja", "使用量", "記録数: 2"),
    ] {
        for view in [OutcomeRenderView::Summary, OutcomeRenderView::Full] {
            let handoff = render_human_outcome_with_view(&input, language, view);
            assert!(handoff.contains(label), "{language} {view:?}: {handoff}");
            assert!(handoff.contains(record_label), "{handoff}");
            assert!(handoff.contains("20"), "{handoff}");
        }
    }
}

#[test]
fn removed_late_receipt_is_invalid_against_frozen_close_report() {
    let root = repository();
    let id = "WI-USAGE-REMOVED-CLOSE";
    let (reference, close_bytes) = closed_with_late_usage(root.path(), id);
    fs::remove_file(root.path().join(reference)).expect("remove late receipt");
    let error = read_work_item_usage(root.path(), id, None)
        .expect_err("close report still binds the missing late receipt");
    assert!(error.to_string().contains("frozen usage"), "{error}");
    assert_eq!(
        fs::read(root.path().join(format!(".ai/decisions/{id}.close.json")))
            .expect("close remains readable"),
        close_bytes
    );
}

#[test]
fn rebound_future_late_receipt_cannot_escape_the_close_cutoff() {
    let root = repository();
    let id = "WI-USAGE-FUTURE-CLOSE";
    let (reference, _) = closed_with_late_usage(root.path(), id);
    let path = root.path().join(reference);
    let mut receipt: UsageReceipt =
        serde_json::from_slice(&fs::read(&path).expect("late receipt bytes")).expect("receipt");
    receipt.received_at = "2999-01-01T00:00:00Z".into();
    receipt.receipt_id = Digest::sha256_bytes(
        &serde_json::to_vec(&(
            receipt.schema_version,
            &receipt.request,
            &receipt.received_at,
            &receipt.source_observed_at,
            receipt.model_assurance,
            receipt.token_assurance,
        ))
        .expect("receipt binding"),
    );
    fs::write(
        &path,
        serde_json::to_vec_pretty(&receipt).expect("receipt JSON"),
    )
    .expect("rebind late receipt");
    let error = read_work_item_usage(root.path(), id, None)
        .expect_err("shifted late receipt cannot disappear before close cutoff");
    assert!(error.to_string().contains("frozen usage"), "{error}");
}

#[test]
fn close_usage_query_rejects_wrong_final_report_digest_and_repository() {
    let root = repository();
    let id = "WI-USAGE-CLOSE-BINDING";
    let (_, original) = closed_with_late_usage(root.path(), id);
    let path = root.path().join(format!(".ai/decisions/{id}.close.json"));
    let mut close: serde_json::Value = serde_json::from_slice(&original).expect("close JSON");
    close["finalReportDigest"] = Digest::sha256_bytes(b"wrong close report")
        .to_string()
        .into();
    fs::write(
        &path,
        serde_json::to_vec_pretty(&close).expect("close JSON"),
    )
    .expect("alter close report digest");
    let error = read_work_item_usage(root.path(), id, None)
        .expect_err("close report digest must match its final report");
    assert!(error.to_string().contains("frozen usage"), "{error}");

    close = serde_json::from_slice(&original).expect("original close JSON");
    close["repositoryId"] = Digest::sha256_bytes(b"other repository").to_string().into();
    fs::write(
        &path,
        serde_json::to_vec_pretty(&close).expect("close JSON"),
    )
    .expect("alter close repository");
    let error = read_work_item_usage(root.path(), id, None)
        .expect_err("close decision must belong to the queried repository");
    assert!(error.to_string().contains("frozen usage"), "{error}");
}

#[test]
fn outcome_does_not_project_closed_when_final_usage_digest_is_invalid() {
    let root = repository();
    let id = "WI-USAGE-OUTCOME-CLOSE-BINDING";
    let (_, original) = closed_with_late_usage(root.path(), id);
    let path = root.path().join(format!(".ai/decisions/{id}.close.json"));
    let mut close: serde_json::Value = serde_json::from_slice(&original).expect("close JSON");
    close["finalReportDigest"] = Digest::sha256_bytes(b"wrong close report")
        .to_string()
        .into();
    fs::write(
        &path,
        serde_json::to_vec_pretty(&close).expect("close JSON"),
    )
    .expect("damage final usage binding");
    read_work_item_usage(root.path(), id, None).expect_err("direct query remains strict");
    let input = outcome_render_input(root.path(), id).expect("read-only outcome projection");
    assert_ne!(input.lifecycle_status, "closed");
    assert!(input.archived_unclosed);
    assert_ne!(
        input.outcome.decision_state,
        Some(cockpit_core::DecisionState::Green)
    );
    let usage = input
        .outcome
        .task_outcome_report
        .as_ref()
        .and_then(|report| report.usage.as_ref())
        .expect("usage projection");
    assert!(
        usage
            .unknown_reasons
            .contains(&"frozen_usage_invalid".into())
    );
    let json = serde_json::to_value(&input.outcome).expect("JSON projection");
    assert_eq!(
        json["taskOutcomeReport"]["usage"]["unknownReasons"][0],
        "frozen_usage_invalid"
    );
    for language in ["en", "zh", "ja"] {
        for view in [OutcomeRenderView::Summary, OutcomeRenderView::Full] {
            let rendered = render_human_outcome_with_view(&input, language, view);
            assert!(!rendered.contains("🟢"), "{language} {view:?}: {rendered}");
            assert!(
                rendered.contains("frozen_usage_invalid"),
                "{language} {view:?}: {rendered}"
            );
        }
    }
}

#[test]
fn close_cannot_downgrade_frozen_usage_by_dropping_final_report_fields() {
    let root = repository();
    let id = "WI-USAGE-CLOSE-DOWNGRADE";
    let (late_reference, close_bytes) = closed_with_late_usage(root.path(), id);
    let path = root.path().join(format!(".ai/decisions/{id}.close.json"));
    let mut close: serde_json::Value = serde_json::from_slice(&close_bytes).expect("close JSON");
    close
        .as_object_mut()
        .expect("close object")
        .remove("finalReport");
    close
        .as_object_mut()
        .expect("close object")
        .remove("usageCutoff");
    fs::write(
        &path,
        serde_json::to_vec_pretty(&close).expect("close JSON"),
    )
    .expect("remove close fields");
    fs::remove_file(root.path().join(late_reference)).expect("remove late receipt");
    let error = read_work_item_usage(root.path(), id, None)
        .expect_err("current close cannot become legacy by deleting final usage fields");
    assert!(error.to_string().contains("frozen usage"), "{error}");
}

#[test]
fn close_with_usage_claim_cannot_become_historical_by_dropping_cutoff() {
    let root = repository();
    let id = "WI-USAGE-CLOSE-CUTOFF-DOWNGRADE";
    let (_, close_bytes) = closed_with_late_usage(root.path(), id);
    let path = root.path().join(format!(".ai/decisions/{id}.close.json"));
    let mut close: serde_json::Value = serde_json::from_slice(&close_bytes).expect("close JSON");
    close["finalReport"]
        .as_object_mut()
        .expect("final report")
        .remove("usage");
    close["finalReportDigest"] = cockpit_protocol::digest_json(&close["finalReport"])
        .expect("report digest")
        .to_string()
        .into();
    close
        .as_object_mut()
        .expect("close object")
        .remove("usageCutoff");
    fs::write(
        &path,
        serde_json::to_vec_pretty(&close).expect("close JSON"),
    )
    .expect("remove close usage fields");
    let error = read_work_item_usage(root.path(), id, None)
        .expect_err("usage-backed close cannot masquerade as a historical close");
    assert!(error.to_string().contains("frozen usage"), "{error}");
    let status = work_item_status_snapshot_with_runtime(root.path(), id, &runtime())
        .expect("read-only status reports invalid frozen usage");
    assert_ne!(status.lifecycle_phase, "closed");
    assert!(status.unknowns.contains(&"frozen_usage_invalid".into()));
}

#[test]
fn legacy_closed_report_without_usage_fields_remains_readable() {
    let root = repository();
    let id = "WI-USAGE-LEGACY-CLOSE";
    start(root.path(), id);
    let contract = root
        .path()
        .join(format!(".ai/work-items/active/{id}.contract.json"));
    preflight_work_item(root.path(), &contract).expect("preflight");
    checkpoint_work_item(root.path(), id).expect("checkpoint");
    record_verification(
        root.path(),
        id,
        &serde_json::json!({"passed":true,"nodesPlanned":1}),
        "1.0.1-test",
        &Digest::sha256_bytes(b"usage test runtime"),
    )
    .expect("verification");
    finish_work_item(root.path(), id).expect("finish");
    let report_path = root
        .path()
        .join(format!(".ai/work-items/active/{id}.task-report.json"));
    let mut report: serde_json::Value =
        serde_json::from_slice(&fs::read(&report_path).expect("report")).expect("report JSON");
    report
        .as_object_mut()
        .expect("report object")
        .remove("usage");
    fs::write(
        &report_path,
        serde_json::to_vec_pretty(&report).expect("report JSON"),
    )
    .expect("make pre-usage report shape");
    archive_work_item(root.path(), id).expect("archive legacy shape");
    let archived_report_path = root
        .path()
        .join(format!(".ai/work-items/archive/{id}.task-report.json"));
    let archived_historical_bytes =
        fs::read(&archived_report_path).expect("archived historical report bytes");
    close_work_item_with_decision(root.path(), id, "approved").expect("close");
    let path = root.path().join(format!(".ai/decisions/{id}.close.json"));
    let mut close: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("close")).expect("close JSON");
    close["finalReport"]
        .as_object_mut()
        .expect("historical final report")
        .remove("usage");
    close["finalReportDigest"] = cockpit_protocol::digest_json(&close["finalReport"])
        .expect("historical final report digest")
        .to_string()
        .into();
    close
        .as_object_mut()
        .expect("close object")
        .remove("usageCutoff");
    fs::write(
        &path,
        serde_json::to_vec_pretty(&close).expect("close JSON"),
    )
    .expect("make pre-usage close shape");
    let summary = read_work_item_usage(root.path(), id, None)
        .expect("historical absence is represented as unknown");
    assert_eq!(summary.coverage, UsageCoverage::Unknown);
    assert_eq!(summary.unknown_reasons, ["no_usage_receipts"]);
    assert_eq!(summary.totals.input_tokens, None);
    assert_eq!(
        fs::read(&archived_report_path).expect("unchanged archived historical report"),
        archived_historical_bytes
    );
    close["finalReportDigest"] = Digest::sha256_bytes(b"wrong historical report")
        .to_string()
        .into();
    fs::write(
        &path,
        serde_json::to_vec_pretty(&close).expect("close JSON"),
    )
    .expect("tamper historical digest");
    let error = read_work_item_usage(root.path(), id, None)
        .expect_err("historical close still binds its final report digest");
    assert!(error.to_string().contains("frozen usage"), "{error}");
}

#[cfg(windows)]
#[test]
fn windows_usage_directory_probe_reads_regular_child_handle() {
    let root = repository();
    let id = "WI-USAGE-WINDOWS-PROBE";
    start(root.path(), id);
    record_work_item_usage(root.path(), &request(root.path(), id, "turn-1"), &runtime())
        .expect("record creates the real usage lock and directory");
    assert_eq!(
        read_work_item_usage(root.path(), id, None)
            .expect("read appended receipt")
            .receipt_refs
            .len(),
        1
    );
}

#[test]
fn usage_directory_without_lock_is_incomplete_and_read_only() {
    let root = repository();
    let id = "WI-USAGE-INCOMPLETE-DIRECTORY";
    fs::create_dir_all(root.path().join(".ai/evidence/usage"))
        .expect("create incomplete usage directory");
    let error = read_work_item_usage(root.path(), id, None)
        .expect_err("a usage directory without its lock is incomplete");
    assert!(error.to_string().contains("evidence_missing"), "{error}");
    assert!(!root.path().join(".ai/locks/usage.lock").exists());
}

#[cfg(unix)]
#[test]
fn usage_directory_probe_rejects_symlink_without_following_target() {
    let root = repository();
    let outside = tempfile::tempdir().expect("outside directory");
    std::os::unix::fs::symlink(outside.path(), root.path().join(".ai/evidence/usage"))
        .expect("usage directory symlink");
    let error = read_work_item_usage(root.path(), "WI-USAGE-LINK", None)
        .expect_err("usage reader must not traverse symlinked directory");
    assert!(error.to_string().contains("usage"), "{error}");
    assert!(outside.path().is_dir());
}

#[test]
fn usage_is_idempotent_bound_and_does_not_double_count_subsets() {
    let root = repository();
    start(root.path(), "WI-USAGE");
    let first_request = request(root.path(), "WI-USAGE", "turn-1");
    let first = record_work_item_usage(root.path(), &first_request, &runtime()).expect("record");
    let replay =
        record_work_item_usage(root.path(), &first_request, &runtime()).expect("idempotent replay");
    assert_eq!(first, replay);
    assert_eq!(first.model_assurance, UsageAssurance::CallerClaim);
    assert_eq!(first.token_assurance, UsageAssurance::CallerClaim);
    assert!(first.source_observed_at.is_none());
    assert!(!first.received_at.is_empty());
    let mut conflicting = first_request.clone();
    conflicting.output_tokens = Some(6);
    assert!(record_work_item_usage(root.path(), &conflicting, &runtime()).is_err());
    let mut overlapping_source = first_request.clone();
    overlapping_source.source_kind = UsageSourceKind::HostReported;
    assert!(record_work_item_usage(root.path(), &overlapping_source, &runtime()).is_err());
    let mut overlapping_unit = first_request.clone();
    overlapping_unit.source_event_id = "invocation-2".into();
    overlapping_unit.unit = UsageUnit::Invocation;
    assert!(record_work_item_usage(root.path(), &overlapping_unit, &runtime()).is_err());
    let mut wrong_work_item = first_request.clone();
    wrong_work_item.work_item_id = "WI-OTHER".into();
    start(root.path(), "WI-OTHER");
    let cross_work_item = record_work_item_usage(root.path(), &wrong_work_item, &runtime())
        .expect_err("one source event cannot be bound to another Work Item");
    assert!(
        cross_work_item
            .to_string()
            .contains("different content, source kind, or Work Item"),
        "{cross_work_item}"
    );
    let summary = read_work_item_usage(root.path(), "WI-USAGE", None).expect("query");
    assert_eq!(summary.coverage, UsageCoverage::Partial);
    assert_eq!(summary.receipt_refs.len(), 1);
    assert_eq!(summary.totals.input_tokens, Some(10));
    assert_eq!(summary.totals.output_tokens, Some(5));
    assert_eq!(summary.totals.cached_input_tokens, Some(3));
    assert_eq!(summary.totals.reasoning_tokens, Some(2));
    assert_eq!(
        summary.subtotals[0].reported_model.as_deref(),
        Some("reported-model")
    );
    assert_eq!(summary.subtotals[0].configured_models, ["configured-model"]);
    assert_eq!(
        summary.subtotals[0].source_kinds,
        [UsageSourceKind::ProviderReported]
    );
    assert_eq!(
        summary.subtotals[0].model_assurance,
        UsageAssurance::CallerClaim
    );
    assert_eq!(
        summary.subtotals[0].token_assurance,
        UsageAssurance::CallerClaim
    );
    let metadata = read_work_item_usage_receipts(root.path(), "WI-USAGE", None)
        .expect("read receipt metadata without source payload");
    assert_eq!(metadata, vec![first]);
    assert_eq!(
        query_work_item_usage(root.path(), "WI-USAGE", None, Some("reported-model"))
            .expect("actual model filter")
            .receipt_refs
            .len(),
        1
    );
    assert!(
        query_work_item_usage(root.path(), "WI-USAGE", None, Some("configured-model"))
            .expect("configured model is not actual model")
            .receipt_refs
            .is_empty()
    );
}

#[test]
fn unknown_counts_remain_null_and_source_time_needs_trusted_provenance() {
    let root = repository();
    start(root.path(), "WI-UNKNOWN");
    let store_path = root.path().join(".ai/evidence/usage");
    assert!(!store_path.exists());
    let empty = read_work_item_usage(root.path(), "WI-UNKNOWN", None).expect("empty query");
    assert!(
        !store_path.exists(),
        "a read-only query must not create receipts"
    );
    assert_eq!(empty.coverage, UsageCoverage::Unknown);
    assert_eq!(empty.unknown_reasons, ["no_usage_receipts"]);
    assert_eq!(empty.totals.input_tokens, None);
    let mut request = request(root.path(), "WI-UNKNOWN", "turn-1");
    request.source_observed_at = Some("2026-10-07T08:00:00+09:00".into());
    assert!(record_work_item_usage(root.path(), &request, &runtime()).is_err());
    request.source_observed_at = None;
    request.input_tokens = None;
    request.cached_input_tokens = None;
    record_work_item_usage(root.path(), &request, &runtime()).expect("record unknown count");
    let summary = read_work_item_usage(root.path(), "WI-UNKNOWN", None).expect("query");
    assert_eq!(summary.totals.input_tokens, None);
    assert_eq!(summary.totals.output_tokens, Some(5));
}

#[test]
fn concurrent_appends_are_atomic_and_overflow_rejects() {
    let root = repository();
    start(root.path(), "WI-PARALLEL-USAGE");
    let path = Arc::new(root.path().to_path_buf());
    let handles = (0..8)
        .map(|index| {
            let path = Arc::clone(&path);
            std::thread::spawn(move || {
                let request = request(&path, "WI-PARALLEL-USAGE", &format!("turn-{index}"));
                record_work_item_usage(&path, &request, &runtime()).expect("concurrent record")
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        handle.join().expect("thread");
    }
    let summary = read_work_item_usage(&path, "WI-PARALLEL-USAGE", None).expect("query");
    assert_eq!(summary.receipt_refs.len(), 8);
    assert_eq!(summary.totals.input_tokens, Some(80));
    let mut overflow = request(&path, "WI-PARALLEL-USAGE", "turn-overflow");
    overflow.input_tokens = Some(u64::MAX);
    overflow.cached_input_tokens = Some(0);
    assert!(record_work_item_usage(&path, &overflow, &runtime()).is_err());
    assert_eq!(
        read_work_item_usage(&path, "WI-PARALLEL-USAGE", None)
            .expect("query")
            .receipt_refs
            .len(),
        8
    );
}

#[test]
fn archived_unclosed_usage_updates_only_the_distinct_close_report() {
    let root = repository();
    let id = "WI-USAGE-CLOSE";
    start(root.path(), id);
    record_work_item_usage(
        root.path(),
        &request(root.path(), id, "turn-before-finish"),
        &runtime(),
    )
    .expect("record before finish");
    let contract = root
        .path()
        .join(format!(".ai/work-items/active/{id}.contract.json"));
    preflight_work_item(root.path(), &contract).expect("preflight");
    checkpoint_work_item(root.path(), id).expect("checkpoint");
    record_verification(
        root.path(),
        id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "1.0.1-test",
        &Digest::sha256_bytes(b"usage test runtime"),
    )
    .expect("verification");
    finish_work_item(root.path(), id).expect("finish");
    let active_path = root
        .path()
        .join(format!(".ai/work-items/active/{id}.task-report.json"));
    let finish_bytes = fs::read(&active_path).expect("finish report bytes");
    let finish: serde_json::Value = serde_json::from_slice(&finish_bytes).expect("finish report");
    assert_eq!(
        finish["usage"]["receiptRefs"]
            .as_array()
            .expect("refs")
            .len(),
        1,
        "{}",
        finish["usage"]
    );
    record_work_item_usage(
        root.path(),
        &request(root.path(), id, "turn-after-finish"),
        &runtime(),
    )
    .expect("record after finish before archive");
    archive_work_item(root.path(), id).expect("archive");
    let archive_path = root
        .path()
        .join(format!(".ai/work-items/archive/{id}.task-report.json"));
    assert_eq!(
        fs::read(&archive_path).expect("archive report"),
        finish_bytes
    );
    record_work_item_usage(
        root.path(),
        &request(root.path(), id, "turn-after-archive"),
        &runtime(),
    )
    .expect("record after archive before close");
    assert_eq!(
        fs::read(&archive_path).expect("frozen archive report"),
        finish_bytes
    );
    close_work_item_with_decision(root.path(), id, "approved").expect("close");
    let close: serde_json::Value = serde_json::from_slice(
        &fs::read(root.path().join(format!(".ai/decisions/{id}.close.json"))).expect("close"),
    )
    .expect("close JSON");
    assert_eq!(
        close["finalReport"]["usage"]["receiptRefs"]
            .as_array()
            .expect("final refs")
            .len(),
        3
    );
    assert_ne!(
        close["finalReport"]["usage"]["cutoff"],
        finish["usage"]["cutoff"]
    );
    assert_eq!(
        fs::read(&archive_path).expect("frozen archive report"),
        finish_bytes
    );
    assert!(
        record_work_item_usage(
            root.path(),
            &request(root.path(), id, "turn-after-close"),
            &runtime()
        )
        .is_err()
    );
}

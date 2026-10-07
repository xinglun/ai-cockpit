use cockpit_core::Digest;
use cockpit_protocol::{
    RuntimeContext, UsageAssurance, UsageCoverage, UsageRecordRequest, UsageSourceKind, UsageUnit,
};
use cockpit_repository::{
    WorkItemStartOptions, archive_work_item, attach, checkpoint_work_item,
    close_work_item_with_decision, finish_work_item, preflight_work_item, query_work_item_usage,
    read_work_item_usage, read_work_item_usage_receipts, record_verification,
    record_work_item_usage, start_work_item_with_options,
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

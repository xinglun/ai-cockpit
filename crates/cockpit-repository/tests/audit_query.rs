use cockpit_core::Digest;
use cockpit_protocol::{
    AssuranceLevel, AuditQueryFilters, DelegatedEvidence, EvidenceValidity, RuntimeContext,
    UsageRecordRequest, UsageSourceKind, UsageUnit,
};
use cockpit_repository::{
    WorkItemStartOptions, attach, export_audit_events, export_audit_events_filtered,
    import_delegated_evidence, query_audit_events, record_work_item_usage,
    start_work_item_with_options, start_work_item_with_options_and_runtime,
};
use std::{fs, process::Command};

fn repository_file_bytes(root: &std::path::Path) -> std::collections::BTreeMap<String, Digest> {
    fn visit(
        root: &std::path::Path,
        directory: &std::path::Path,
        files: &mut std::collections::BTreeMap<String, Digest>,
    ) {
        for entry in fs::read_dir(directory).expect("repository directory") {
            let path = entry.expect("repository entry").path();
            if path.is_dir() {
                visit(root, &path, files);
            } else if path.is_file() {
                files.insert(
                    path.strip_prefix(root)
                        .expect("relative path")
                        .to_string_lossy()
                        .into(),
                    Digest::sha256_bytes(&fs::read(&path).expect("file bytes")),
                );
            }
        }
    }
    let mut files = std::collections::BTreeMap::new();
    visit(root, &root.join(".ai"), &mut files);
    files
}

fn repository() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("repository");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(root.path())
            .status()
            .expect("git init")
            .success()
    );
    attach(root.path()).expect("attach");
    start_work_item_with_options(
        root.path(),
        "WI-AUDIT-QUERY",
        "audit",
        "query",
        &[".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..Default::default()
        },
    )
    .expect("start");
    fs::write(root.path().join(".ai/evidence/audit-source.json"), b"{}\n").expect("source");
    root
}

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "1.0.1-test".into(),
        protocol_version: cockpit_protocol::PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(b"audit query test runtime"),
    }
}

fn record(root: &std::path::Path, event: &str, model: &str, count: Option<u64>) {
    record_work_item_usage(
        root,
        &UsageRecordRequest {
            schema_version: 1,
            repository_id: cockpit_repository::repository_id(root).to_string(),
            work_item_id: "WI-AUDIT-QUERY".into(),
            source_event_id: event.into(),
            source_kind: UsageSourceKind::AgentDeclared,
            actor: Some("agent:test".into()),
            configured_model: Some("configured-model".into()),
            reported_model: Some(model.into()),
            role: "implementer".into(),
            phase: "implementation".into(),
            unit: UsageUnit::Turn,
            input_tokens: count,
            output_tokens: None,
            cached_input_tokens: None,
            reasoning_tokens: None,
            source_observed_at: None,
            evidence_ref: ".ai/evidence/audit-source.json".into(),
            evidence_digest: Digest::sha256_bytes(b"{}\n"),
        },
        &runtime(),
    )
    .expect("record");
}

fn delegated(root: &std::path::Path, number: u8) -> String {
    let raw = format!("{{\"run\":{number}}}").into_bytes();
    let reference = format!(".ai/evidence/external/run-{number}.json");
    import_delegated_evidence(
        root,
        "WI-AUDIT-QUERY",
        &DelegatedEvidence {
            provider: "github".into(),
            subject: format!("run:{number}"),
            origin: format!("https://github.com/example/repo/actions/runs/{number}"),
            assurance: AssuranceLevel::ProviderVerified,
            collected_at: "2026-10-07T00:00:00Z".into(),
            digest: Digest::sha256_bytes(&raw),
            validity: EvidenceValidity::Valid,
            raw_evidence_ref: reference.clone(),
        },
        &raw,
        &runtime(),
    )
    .expect("import delegated evidence");
    reference
}

#[test]
fn delegated_cursor_binds_actual_raw_bytes_on_every_page() {
    let root = repository();
    let first_raw = delegated(root.path(), 1);
    let second_raw = delegated(root.path(), 2);
    let filters = AuditQueryFilters {
        event_type: Some("external_evidence_bound".into()),
        limit: Some(1),
        ..Default::default()
    };
    let first = query_audit_events(root.path(), &runtime(), &filters).expect("first page");
    assert_eq!(first.returned_count, 1);
    assert!(
        first.items[0]
            .evidence_refs
            .iter()
            .all(|reference| reference.digest.is_some())
    );
    assert!(
        first.items[0]
            .evidence_refs
            .iter()
            .any(|reference| { reference.path == first_raw || reference.path == second_raw })
    );
    let cursor = first.next_cursor.expect("second page cursor");
    fs::write(root.path().join(&second_raw), b"{\"changed\":true}").expect("change raw source");
    let error = query_audit_events(
        root.path(),
        &runtime(),
        &AuditQueryFilters {
            cursor: Some(cursor),
            ..filters
        },
    )
    .expect_err("changed raw bytes invalidate source snapshot");
    assert!(error.to_string().contains("stale_cursor"), "{error}");
}

#[test]
fn unreadable_external_directory_does_not_report_complete_query() {
    let root = repository();
    delegated(root.path(), 1);
    let external = root.path().join(".ai/evidence/external");
    fs::rename(&external, root.path().join(".ai/evidence/external.saved"))
        .expect("move source directory");
    fs::write(&external, b"not a directory").expect("block external directory read");
    let error = query_audit_events(root.path(), &runtime(), &AuditQueryFilters::default())
        .expect_err("unreadable external source must not be reported complete");
    assert!(error.to_string().contains("external"), "{error}");
}

#[test]
fn usage_query_filters_runtime_received_time_and_preserves_unknown_counts() {
    let root = repository();
    record(root.path(), "turn-1", "reported-a", None);
    record(root.path(), "turn-2", "reported-b", Some(9));
    let before_query = repository_file_bytes(root.path());
    let all = query_audit_events(root.path(), &runtime(), &AuditQueryFilters::default())
        .expect("read-only audit query");
    let usage: Vec<_> = all
        .items
        .iter()
        .filter(|item| item.event_type == "usage_recorded")
        .collect();
    assert_eq!(usage.len(), 2);
    assert!(usage.iter().all(|item| item.recorded_at.is_some()));
    assert!(usage.iter().all(|item| item.source_observed_at.is_none()));
    assert_eq!(
        usage
            .iter()
            .find(|item| item.reported_model.as_deref() == Some("reported-a"))
            .expect("a")
            .token_counts
            .input_tokens,
        None
    );
    assert!(
        usage
            .iter()
            .all(|item| item.configured_model.as_deref() == Some("configured-model"))
    );
    assert!(
        usage
            .iter()
            .all(|item| item.actor_provenance == "caller_claim")
    );
    assert!(
        usage
            .iter()
            .all(|item| item.evidence_refs.iter().any(|reference| {
                reference.path == ".ai/evidence/audit-source.json"
                    && reference.digest.as_ref() == Some(&Digest::sha256_bytes(b"{}\n"))
            }))
    );
    let only_b = query_audit_events(
        root.path(),
        &runtime(),
        &AuditQueryFilters {
            reported_model: Some("reported-b".into()),
            ..Default::default()
        },
    )
    .expect("reported model filter");
    assert_eq!(only_b.items.len(), 1);
    assert_eq!(only_b.items[0].token_counts.input_tokens, Some(9));
    assert!(
        query_audit_events(
            root.path(),
            &runtime(),
            &AuditQueryFilters {
                reported_model: Some("configured-model".into()),
                ..Default::default()
            }
        )
        .expect("no configured fallback")
        .items
        .is_empty()
    );
    assert_eq!(repository_file_bytes(root.path()), before_query);
}

#[test]
fn audit_cursor_binds_source_snapshot_and_page_totals() {
    let root = repository();
    record(root.path(), "turn-1", "reported-a", Some(4));
    record(root.path(), "turn-2", "reported-a", Some(5));
    let filters = AuditQueryFilters {
        event_type: Some("usage_recorded".into()),
        limit: Some(1),
        ..Default::default()
    };
    let first = query_audit_events(root.path(), &runtime(), &filters).expect("first page");
    assert_eq!(first.returned_count, 1);
    assert!(first.truncated);
    assert_eq!(
        first.page_usage_totals.input_tokens,
        first.items[0].token_counts.input_tokens
    );
    assert_eq!(first.page_usage_subtotals.len(), 1);
    assert_eq!(first.page_usage_subtotals[0].record_count, 1);
    assert_eq!(
        first.page_usage_subtotals[0].counts.input_tokens,
        first.items[0].token_counts.input_tokens
    );
    let cursor = first.next_cursor.clone().expect("next cursor");
    let second = query_audit_events(
        root.path(),
        &runtime(),
        &AuditQueryFilters {
            cursor: Some(cursor.clone()),
            ..filters.clone()
        },
    )
    .expect("second page");
    assert_eq!(second.returned_count, 1);
    assert_ne!(first.items[0].event_id, second.items[0].event_id);
    record(root.path(), "turn-3", "reported-a", Some(6));
    let error = query_audit_events(
        root.path(),
        &runtime(),
        &AuditQueryFilters {
            cursor: Some(cursor),
            ..filters
        },
    )
    .expect_err("source mutation invalidates cursor");
    assert!(error.to_string().contains("stale_cursor"), "{error}");
}

#[test]
fn audit_cursor_rejects_changed_referenced_source_bytes() {
    let root = repository();
    record(root.path(), "turn-1", "reported-a", Some(4));
    record(root.path(), "turn-2", "reported-a", Some(5));
    let filters = AuditQueryFilters {
        event_type: Some("usage_recorded".into()),
        limit: Some(1),
        ..Default::default()
    };
    let first = query_audit_events(root.path(), &runtime(), &filters).expect("first page");
    let cursor = first.next_cursor.expect("cursor");
    fs::write(
        root.path().join(".ai/evidence/audit-source.json"),
        b"{\"changed\":true}\n",
    )
    .expect("change referenced source");
    let error = query_audit_events(
        root.path(),
        &runtime(),
        &AuditQueryFilters {
            cursor: Some(cursor),
            ..filters
        },
    )
    .expect_err("changed source invalidates the snapshot");
    assert!(error.to_string().contains("stale_cursor"), "{error}");
}

#[test]
fn lifecycle_query_uses_only_persisted_successful_transition_facts() {
    let root = repository();
    let page = query_audit_events(
        root.path(),
        &runtime(),
        &AuditQueryFilters {
            work_item_id: Some("WI-AUDIT-QUERY".into()),
            event_type: Some("work_item_started".into()),
            ..Default::default()
        },
    )
    .expect("started event");
    assert_eq!(page.items.len(), 1);
    let started = &page.items[0];
    assert!(
        started
            .recorded_at
            .as_deref()
            .is_some_and(|value| value.ends_with('Z'))
    );
    assert_eq!(started.occurred_at, started.recorded_at);
    assert_eq!(started.actor, None);
    assert_eq!(started.actor_provenance, "unknown");
    assert!(
        query_audit_events(
            root.path(),
            &runtime(),
            &AuditQueryFilters {
                event_type: Some("work_item_closed".into()),
                ..Default::default()
            }
        )
        .expect("unreached terminal event")
        .items
        .is_empty()
    );
}

#[test]
fn failed_finish_report_write_leaves_no_successful_finish_fact() {
    let root = repository();
    let id = "WI-AUDIT-QUERY";
    let contract = root
        .path()
        .join(format!(".ai/work-items/active/{id}.contract.json"));
    cockpit_repository::preflight_work_item(root.path(), &contract).expect("preflight");
    cockpit_repository::checkpoint_work_item(root.path(), id).expect("checkpoint");
    cockpit_repository::record_verification(
        root.path(),
        id,
        &serde_json::json!({"passed":true,"nodesPlanned":1}),
        "1.0.1-test",
        &Digest::sha256_bytes(b"test runtime"),
    )
    .expect("verification");
    let report_path = root
        .path()
        .join(format!(".ai/work-items/active/{id}.task-report.json"));
    let existing = b"existing report must remain intact";
    fs::write(&report_path, existing).expect("plant existing report");
    let markdown_path = root
        .path()
        .join(format!(".ai/work-items/active/{id}.task-report.md"));
    fs::create_dir(&markdown_path).expect("plant nonreplaceable output conflict");
    fs::write(
        markdown_path.join("existing"),
        b"existing markdown directory",
    )
    .expect("plant nested file");
    cockpit_repository::finish_work_item(root.path(), id)
        .expect_err("existing report prevents successful finish");
    assert_eq!(fs::read(&report_path).expect("existing report"), existing);
    assert_eq!(
        fs::read(markdown_path.join("existing")).expect("existing markdown"),
        b"existing markdown directory"
    );
    let summary: serde_json::Value = serde_json::from_slice(
        &fs::read(
            root.path()
                .join(format!(".ai/work-items/active/{id}.summary.json")),
        )
        .expect("summary"),
    )
    .expect("summary JSON");
    assert_ne!(summary["state"], "finish_ready");
    assert!(summary["lifecycleFacts"].get("finish").is_none());
    let query = query_audit_events(
        root.path(),
        &runtime(),
        &AuditQueryFilters {
            event_type: Some("work_item_finished".into()),
            ..Default::default()
        },
    )
    .expect("read-only audit query");
    assert!(query.items.is_empty());
}

#[test]
fn runtime_bound_start_event_keeps_its_producer_identity() {
    let root = tempfile::tempdir().expect("repository");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(root.path())
            .status()
            .expect("git init")
            .success()
    );
    attach(root.path()).expect("attach");
    start_work_item_with_options_and_runtime(
        root.path(),
        "WI-BOUND-START",
        "audit",
        "start",
        &[".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..Default::default()
        },
        &runtime(),
    )
    .expect("bound start");
    let page = query_audit_events(
        root.path(),
        &runtime(),
        &AuditQueryFilters {
            event_type: Some("work_item_started".into()),
            ..Default::default()
        },
    )
    .expect("start query");
    assert_eq!(page.items.len(), 1);
    assert_eq!(
        page.items[0].event_runtime_version.as_deref(),
        Some("1.0.1-test")
    );
    assert_eq!(
        page.items[0].event_runtime_digest.as_ref(),
        Some(&runtime().runtime_digest)
    );
}

#[test]
fn terminal_lifecycle_events_come_from_each_successful_boundary() {
    let root = repository();
    let id = "WI-AUDIT-QUERY";
    record(root.path(), "turn-1", "reported-a", Some(4));
    let contract = root
        .path()
        .join(format!(".ai/work-items/active/{id}.contract.json"));
    cockpit_repository::preflight_work_item(root.path(), &contract).expect("preflight");
    cockpit_repository::checkpoint_work_item(root.path(), id).expect("checkpoint");
    cockpit_repository::record_verification(
        root.path(),
        id,
        &serde_json::json!({"passed":true,"nodesPlanned":1}),
        "1.0.1-test",
        &Digest::sha256_bytes(b"test runtime"),
    )
    .expect("verification");
    cockpit_repository::finish_work_item(root.path(), id).expect("finish");
    let finished = query_audit_events(
        root.path(),
        &runtime(),
        &AuditQueryFilters {
            event_type: Some("work_item_finished".into()),
            ..Default::default()
        },
    )
    .expect("finish event");
    assert_eq!(finished.items.len(), 1);
    assert!(finished.items[0].recorded_at.is_some());
    cockpit_repository::archive_work_item(root.path(), id).expect("archive");
    let archived = query_audit_events(
        root.path(),
        &runtime(),
        &AuditQueryFilters {
            event_type: Some("work_item_archived".into()),
            ..Default::default()
        },
    )
    .expect("archive event");
    assert_eq!(archived.items.len(), 1);
    cockpit_repository::close_work_item_with_decision(root.path(), id, "approved").expect("close");
    let closed = query_audit_events(
        root.path(),
        &runtime(),
        &AuditQueryFilters {
            event_type: Some("work_item_closed".into()),
            ..Default::default()
        },
    )
    .expect("close event");
    assert_eq!(closed.items.len(), 1);
    assert!(closed.items[0].recorded_at.is_some());
    assert_eq!(closed.items[0].actor.as_deref(), Some("legacy-cli"));
    assert_eq!(closed.items[0].actor_provenance, "structured_decision");
    assert!(closed.items[0].wall_elapsed_ms.is_some());
    let exported = export_audit_events_filtered(
        root.path(),
        &runtime(),
        &AuditQueryFilters {
            event_type: Some("work_item_closed".into()),
            ..Default::default()
        },
    )
    .expect("filtered lifecycle export");
    assert_eq!(exported.items, closed.items);
    assert!(
        closed.items[0]
            .evidence_refs
            .iter()
            .all(|reference| reference.digest.is_some())
    );
    let started = query_audit_events(
        root.path(),
        &runtime(),
        &AuditQueryFilters {
            event_type: Some("work_item_started".into()),
            ..Default::default()
        },
    )
    .expect("start event");
    let times = [
        &started.items[0],
        &finished.items[0],
        &archived.items[0],
        &closed.items[0],
    ]
    .map(|item| {
        item.recorded_at
            .as_deref()
            .expect("recorded transition time")
    });
    assert!(times.iter().all(|time| time.contains('.')), "{times:?}");
    assert!(times.windows(2).all(|pair| pair[0] <= pair[1]), "{times:?}");
    let summary: serde_json::Value = serde_json::from_slice(
        &fs::read(
            root.path()
                .join(format!(".ai/work-items/archive/{id}.summary.json")),
        )
        .expect("archived summary"),
    )
    .expect("summary JSON");
    let archive: serde_json::Value = serde_json::from_slice(
        &fs::read(
            root.path()
                .join(format!(".ai/work-items/archive/{id}.archive.json")),
        )
        .expect("archive manifest"),
    )
    .expect("archive JSON");
    let close: serde_json::Value = serde_json::from_slice(
        &fs::read(root.path().join(format!(".ai/decisions/{id}.close.json")))
            .expect("close decision"),
    )
    .expect("close JSON");
    for source in [
        &summary["lifecycleFacts"]["start"],
        &summary["lifecycleFacts"]["finish"],
        &archive,
        &close,
    ] {
        assert!(
            source["recordedAt"]
                .as_str()
                .is_some_and(|time| time.contains('.')),
            "lifecycle source must persist subsecond UTC: {source}"
        );
    }
}

#[test]
fn tokyo_display_changes_only_labels_not_utc_facts_or_snapshot() {
    let root = repository();
    record(root.path(), "turn-1", "reported-a", Some(4));
    let utc = query_audit_events(
        root.path(),
        &runtime(),
        &AuditQueryFilters {
            event_type: Some("usage_recorded".into()),
            ..Default::default()
        },
    )
    .expect("UTC view");
    let tokyo = query_audit_events(
        root.path(),
        &runtime(),
        &AuditQueryFilters {
            event_type: Some("usage_recorded".into()),
            display_timezone: Some("Asia/Tokyo".into()),
            ..Default::default()
        },
    )
    .expect("IANA Tokyo view");
    assert_eq!(utc.source_snapshot_digest, tokyo.source_snapshot_digest);
    assert_eq!(utc.items[0].recorded_at, tokyo.items[0].recorded_at);
    assert_eq!(tokyo.items[0].display_timezone, "Asia/Tokyo");
    assert!(
        tokyo.items[0]
            .display_time
            .as_deref()
            .expect("display time")
            .ends_with("+09:00")
    );
}

#[test]
fn filtered_export_reuses_query_page_and_legacy_export_remains_schema_one() {
    let root = repository();
    record(root.path(), "turn-1", "reported-a", Some(4));
    let filters = AuditQueryFilters {
        event_type: Some("usage_recorded".into()),
        limit: Some(1),
        ..Default::default()
    };
    let query = query_audit_events(root.path(), &runtime(), &filters).expect("query");
    let exported =
        export_audit_events_filtered(root.path(), &runtime(), &filters).expect("filtered export");
    assert_eq!(exported.items, query.items);
    assert_eq!(exported.coverage, query.coverage);
    assert_eq!(
        exported.source_snapshot_digest,
        query.source_snapshot_digest
    );
    let legacy = export_audit_events(root.path(), &runtime()).expect("legacy export");
    assert_eq!(legacy.schema_version, 1);
    assert_eq!(
        serde_json::to_value(&legacy).expect("legacy JSON")["externalRetentionRequired"],
        true
    );
}

#[test]
fn recorded_at_window_is_inclusive_from_and_exclusive_to() {
    let root = repository();
    record(root.path(), "turn-1", "reported-a", Some(4));
    record(root.path(), "turn-2", "reported-a", Some(5));
    let base = AuditQueryFilters {
        event_type: Some("usage_recorded".into()),
        ..Default::default()
    };
    let all = query_audit_events(root.path(), &runtime(), &base).expect("all usage");
    assert_eq!(all.items.len(), 2);
    let first = all.items[0].recorded_at.clone().expect("first recordedAt");
    let second = all.items[1].recorded_at.clone().expect("second recordedAt");
    assert!(first < second);
    let window = query_audit_events(
        root.path(),
        &runtime(),
        &AuditQueryFilters {
            from: Some(first.clone()),
            to: Some(second.clone()),
            ..base.clone()
        },
    )
    .expect("half-open window");
    assert_eq!(window.items.len(), 1);
    assert_eq!(window.items[0].event_id, all.items[0].event_id);
    let later = query_audit_events(
        root.path(),
        &runtime(),
        &AuditQueryFilters {
            from: Some(second),
            actor: Some("agent:test".into()),
            ..base
        },
    )
    .expect("inclusive lower bound and exact actor");
    assert_eq!(later.items.len(), 1);
    assert_eq!(later.items[0].event_id, all.items[1].event_id);
    let normalized = query_audit_events(
        root.path(),
        &runtime(),
        &AuditQueryFilters {
            from: Some("2026-10-07T09:00:00+09:00".into()),
            ..AuditQueryFilters::default()
        },
    )
    .expect("normalize equivalent offset to UTC");
    assert_eq!(
        normalized.filters.from.as_deref(),
        Some("2026-10-07T00:00:00Z")
    );
}

#[test]
fn historical_event_without_recorded_at_is_a_coverage_gap_not_time_inference() {
    let root = repository();
    let id = "WI-AUDIT-QUERY";
    let contract = root
        .path()
        .join(format!(".ai/work-items/active/{id}.contract.json"));
    cockpit_repository::preflight_work_item(root.path(), &contract).expect("preflight");
    cockpit_repository::checkpoint_work_item(root.path(), id).expect("checkpoint");
    cockpit_repository::record_verification(
        root.path(),
        id,
        &serde_json::json!({"passed":true,"nodesPlanned":1}),
        "1.0.1-test",
        &Digest::sha256_bytes(b"test runtime"),
    )
    .expect("verification");
    let base = AuditQueryFilters {
        event_type: Some("verification_recorded".into()),
        ..Default::default()
    };
    let all = query_audit_events(root.path(), &runtime(), &base).expect("historical audit event");
    assert_eq!(all.items.len(), 1);
    assert_eq!(all.items[0].recorded_at, None);
    assert!(all.items[0].occurred_at.is_some());
    assert_eq!(all.coverage.unknown_count, Some(1));
    assert_eq!(all.coverage.unknown_reasons, ["recorded_at_missing"]);
    let bounded = query_audit_events(
        root.path(),
        &runtime(),
        &AuditQueryFilters {
            from: Some("2000-01-01T00:00:00Z".into()),
            ..base
        },
    )
    .expect("time filter does not use createdAt");
    assert!(bounded.items.is_empty());
    assert_eq!(bounded.coverage.unknown_count, Some(1));
}

#[test]
fn partially_removed_close_clock_fact_is_invalid_not_historical() {
    let root = repository();
    let id = "WI-AUDIT-QUERY";
    record(root.path(), "turn-1", "reported-a", Some(4));
    let contract = root
        .path()
        .join(format!(".ai/work-items/active/{id}.contract.json"));
    cockpit_repository::preflight_work_item(root.path(), &contract).expect("preflight");
    cockpit_repository::checkpoint_work_item(root.path(), id).expect("checkpoint");
    cockpit_repository::record_verification(
        root.path(),
        id,
        &serde_json::json!({"passed":true,"nodesPlanned":1}),
        "1.0.1-test",
        &Digest::sha256_bytes(b"test runtime"),
    )
    .expect("verification");
    cockpit_repository::finish_work_item(root.path(), id).expect("finish");
    cockpit_repository::archive_work_item(root.path(), id).expect("archive");
    cockpit_repository::close_work_item_with_decision(root.path(), id, "approved").expect("close");
    let path = root.path().join(format!(".ai/decisions/{id}.close.json"));
    let mut close: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("close bytes")).expect("close JSON");
    close
        .as_object_mut()
        .expect("close object")
        .remove("recordedAt");
    fs::write(
        &path,
        serde_json::to_vec_pretty(&close).expect("close JSON"),
    )
    .expect("remove recordedAt");
    let error = query_audit_events(root.path(), &runtime(), &AuditQueryFilters::default())
        .expect_err("partial Runtime time fact must fail closed");
    assert!(error.to_string().contains("lifecycle"), "{error}");
}

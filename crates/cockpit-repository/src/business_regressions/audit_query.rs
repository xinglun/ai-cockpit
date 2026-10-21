//! Native domain fixtures use an explicit private test port, never a production source.
#![cfg(test)]
use super::approved_boundary_fixtures as boundary;
use crate as cockpit_repository;
use crate::review_authority::pipeline_tests::TestAuthority;
use cockpit_core::Digest;
use cockpit_protocol::AuditQueryFilters;
use cockpit_protocol::RuntimeContext;
use cockpit_protocol::UsageRecordRequest;
use cockpit_protocol::UsageSourceKind;
use cockpit_protocol::UsageUnit;
use cockpit_repository::WorkItemStartOptions;
use cockpit_repository::attach;
use cockpit_repository::export_audit_events_filtered;
use cockpit_repository::query_audit_events;
use cockpit_repository::record_work_item_usage;
use cockpit_repository::start_work_item_with_options;
use std::fs;
use std::process::Command;
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
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=AI Cockpit Test",
                "-c",
                "user.email=ai-cockpit-test@example.invalid",
                "commit",
                "--allow-empty",
                "-m",
                "fixture baseline",
            ])
            .current_dir(root.path())
            .status()
            .expect("git baseline commit")
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

#[test]
fn terminal_lifecycle_events_come_from_each_successful_boundary() {
    let authority = TestAuthority::install();
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
    boundary::close_legacy(&authority, root.path(), id, "approved").expect("close");
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
fn partially_removed_close_clock_fact_is_invalid_not_historical() {
    let authority = TestAuthority::install();
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
    boundary::close_legacy(&authority, root.path(), id, "approved").expect("close");
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

use cockpit_core::Digest;
use cockpit_protocol::ResourceFinalizationContext;
use cockpit_repository::{
    WorkItemStartOptions, archive_work_item, checkpoint_work_item, finish_work_item, outcome_v2,
    plan_resource_finalization, preflight_work_item, record_verification,
    start_work_item_with_options,
};
use std::{fs, process::Command};

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("tempdir");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    cockpit_repository::attach(directory.path()).expect("attach");
    directory
}

fn ready(directory: &tempfile::TempDir, id: &str) {
    start_work_item_with_options(
        directory.path(),
        id,
        "generate a typed report",
        "show evidence-bound outcome sections",
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..Default::default()
        },
    )
    .expect("start");
    plan_resource_finalization(
        directory.path(),
        id,
        &ResourceFinalizationContext {
            branch: format!("feature/{id}"),
            worktree: directory.path().display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: format!("https://github.com/example/ai-cockpit/pull/{id}"),
        },
    )
    .expect("finalization plan");
    let contract = directory
        .path()
        .join(format!(".ai/work-items/active/{id}.contract.json"));
    preflight_work_item(directory.path(), &contract).expect("preflight");
    checkpoint_work_item(directory.path(), id).expect("checkpoint");
    record_verification(
        directory.path(),
        id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.10",
        &Digest::sha256_bytes(b"wi136-runtime"),
    )
    .expect("verify");
}

#[test]
fn report_is_typed_evidence_bound_and_serializable() {
    let directory = repository();
    ready(&directory, "WI-136-REPORT");
    let outcome = outcome_v2(directory.path(), "WI-136-REPORT").expect("outcome");
    let report = outcome.task_outcome_report.clone().expect("task report");
    assert_eq!(report.format, "ai-cockpit.task-outcome");
    assert_eq!(report.schema_version, 1);
    assert_eq!(report.bindings.repository_id, outcome.repository_id);
    assert_eq!(report.bindings.work_item_id, outcome.work_item_id);
    assert!(!report.sections.outcome_summary.is_empty());
    assert!(!report.sections.warnings.is_empty());
    assert!(
        report
            .sections
            .outcome_summary
            .iter()
            .all(|claim| !claim.evidence_refs.is_empty() || claim.inference)
    );
    let encoded = serde_json::to_value(&outcome).expect("encode outcome");
    assert!(encoded.get("taskOutcomeReport").is_some());
}

#[test]
fn finish_writes_event_stream_and_archive_binds_it() {
    let directory = repository();
    ready(&directory, "WI-136-EVENTS");
    finish_work_item(directory.path(), "WI-136-EVENTS").expect("finish");
    let active_events = directory
        .path()
        .join(".ai/work-items/active/WI-136-EVENTS.events.jsonl");
    assert!(active_events.is_file());
    assert!(
        directory
            .path()
            .join(".ai/work-items/active/WI-136-EVENTS.task-report.json")
            .is_file()
    );
    assert!(
        directory
            .path()
            .join(".ai/work-items/active/WI-136-EVENTS.task-report.md")
            .is_file()
    );
    let text = fs::read_to_string(&active_events).expect("events");
    assert!(text.contains("\"eventType\":\"completed\""));
    archive_work_item(directory.path(), "WI-136-EVENTS").expect("archive");
    let archived_events = directory
        .path()
        .join(".ai/work-items/archive/WI-136-EVENTS.events.jsonl");
    assert!(archived_events.is_file());
    assert!(
        directory
            .path()
            .join(".ai/work-items/archive/WI-136-EVENTS.task-report.json")
            .is_file()
    );
    assert!(
        directory
            .path()
            .join(".ai/work-items/archive/WI-136-EVENTS.task-report.md")
            .is_file()
    );
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/archive/WI-136-EVENTS.archive.json"),
        )
        .expect("manifest"),
    )
    .expect("manifest JSON");
    assert!(manifest["files"]["eventsDigest"].is_string());
}

#[test]
fn malformed_or_foreign_event_stream_fails_archive_closed() {
    let directory = repository();
    ready(&directory, "WI-136-TAMPER");
    finish_work_item(directory.path(), "WI-136-TAMPER").expect("finish");
    let path = directory
        .path()
        .join(".ai/work-items/active/WI-136-TAMPER.events.jsonl");
    let mut line: serde_json::Value = serde_json::from_str(
        fs::read_to_string(&path)
            .expect("events")
            .lines()
            .next()
            .expect("event"),
    )
    .expect("event JSON");
    line["repositoryId"] = serde_json::Value::String("sha256:foreign".into());
    fs::write(&path, serde_json::to_vec(&line).expect("event JSON")).expect("tamper");
    assert!(archive_work_item(directory.path(), "WI-136-TAMPER").is_err());
}

#[test]
fn archived_report_tamper_is_red_and_not_reprojected_as_verified() {
    let directory = repository();
    ready(&directory, "WI-136-REPORT-TAMPER");
    finish_work_item(directory.path(), "WI-136-REPORT-TAMPER").expect("finish");
    archive_work_item(directory.path(), "WI-136-REPORT-TAMPER").expect("archive");
    let report_path = directory
        .path()
        .join(".ai/work-items/archive/WI-136-REPORT-TAMPER.task-report.json");
    let mut report: serde_json::Value =
        serde_json::from_slice(&fs::read(&report_path).expect("task report"))
            .expect("task report JSON");
    report["sections"]["warnings"] = serde_json::json!([
        {"text":"tampered", "evidenceRefs":[], "inference":true}
    ]);
    fs::write(
        &report_path,
        serde_json::to_vec_pretty(&report).expect("report JSON"),
    )
    .expect("tamper report");
    let outcome = outcome_v2(directory.path(), "WI-136-REPORT-TAMPER").expect("outcome");
    assert_eq!(outcome.state, cockpit_protocol::OutcomeState::Unknown);
    assert_eq!(
        outcome.decision_state,
        Some(cockpit_core::DecisionState::Red)
    );
    assert!(
        outcome
            .unknowns
            .iter()
            .any(|unknown| unknown == "outcome_report_invalid")
    );
}

use cockpit_core::Digest;
use cockpit_protocol::{ResourceFinalizationContext, TaskOutcomeEvent};
use cockpit_repository::{
    WorkItemStartOptions, archive_work_item, checkpoint_work_item, finish_work_item,
    plan_resource_finalization, preflight_work_item, record_verification,
    start_work_item_with_options,
};
use serde_json::Value;
use std::{fs, path::Path, process::Command};

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
        "generate a typed event report",
        "show event-family evidence",
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
        "0.2.52",
        &Digest::sha256_bytes(b"wi457-runtime"),
    )
    .expect("verify");
}

fn event(
    repository_id: &str,
    work_item_id: &str,
    event_id: &str,
    event_type: &str,
    fingerprint: Option<&str>,
    related_event_ids: Vec<String>,
    correction_of: Option<&str>,
) -> TaskOutcomeEvent {
    TaskOutcomeEvent {
        schema_version: 1,
        event_id: event_id.into(),
        repository_id: repository_id.into(),
        work_item_id: work_item_id.into(),
        event_type: event_type.into(),
        timestamp: format!("2026-09-01T00:00:00Z-{event_id}"),
        detail: format!("detail for {event_type}"),
        evidence_refs: vec![".ai/evidence/test.json".into()],
        related_event_ids,
        correction_of: correction_of.map(str::to_owned),
        finding_fingerprint: fingerprint.map(str::to_owned),
    }
}

fn append_events(path: &Path, events: &[TaskOutcomeEvent]) {
    let mut text = fs::read_to_string(path).expect("event stream");
    for event in events {
        text.push_str(&serde_json::to_string(event).expect("event JSON"));
        text.push('\n');
    }
    fs::write(path, text).expect("append events");
}

fn event_path(directory: &tempfile::TempDir, id: &str) -> std::path::PathBuf {
    directory
        .path()
        .join(format!(".ai/work-items/active/{id}.events.jsonl"))
}

#[test]
fn accepts_reference_event_families_and_append_only_corrections() {
    let directory = repository();
    let id = "WI-457-EVENT-FAMILIES";
    ready(&directory, id);
    finish_work_item(directory.path(), id).expect("finish");
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let path = event_path(&directory, id);
    let mut events = Vec::new();
    for (index, event_type) in [
        "finding",
        "risk",
        "warning",
        "confirmation",
        "stop",
        "resume",
        "resolution",
        "risk-accepted",
        "check-pass-after-fix",
        "prevention",
        "cancelled",
    ]
    .into_iter()
    .enumerate()
    {
        let fingerprint = matches!(event_type, "finding" | "risk")
            .then_some(format!("sha256:{:064x}", index + 1));
        events.push(event(
            &repository_id,
            id,
            &format!("event-{index}"),
            event_type,
            fingerprint.as_deref(),
            Vec::new(),
            None,
        ));
    }
    events.push(event(
        &repository_id,
        id,
        "event-corrected",
        "event_corrected",
        Some("sha256:0000000000000000000000000000000000000000000000000000000000000001"),
        vec!["event-0".into()],
        Some("event-0"),
    ));
    events.push(event(
        &repository_id,
        id,
        "event-superseded",
        "event_superseded",
        Some("sha256:0000000000000000000000000000000000000000000000000000000000000001"),
        vec!["event-corrected".into()],
        Some("event-corrected"),
    ));
    append_events(&path, &events);
    archive_work_item(directory.path(), id).expect("reference event families accepted");
}

#[test]
fn finding_fingerprint_is_required_and_duplicate_fingerprints_fail_closed() {
    let directory = repository();
    let id = "WI-457-EVENT-FINGERPRINT";
    ready(&directory, id);
    finish_work_item(directory.path(), id).expect("finish");
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let path = event_path(&directory, id);
    append_events(
        &path,
        &[event(
            &repository_id,
            id,
            "missing-fingerprint",
            "finding",
            None,
            Vec::new(),
            None,
        )],
    );
    assert!(archive_work_item(directory.path(), id).is_err());

    let directory = repository();
    let id = "WI-457-EVENT-DUPLICATE";
    ready(&directory, id);
    finish_work_item(directory.path(), id).expect("finish");
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let path = event_path(&directory, id);
    let fingerprint = "sha256:1111111111111111111111111111111111111111111111111111111111111111";
    append_events(
        &path,
        &[
            event(
                &repository_id,
                id,
                "finding-1",
                "finding",
                Some(fingerprint),
                vec![],
                None,
            ),
            event(
                &repository_id,
                id,
                "finding-2",
                "finding",
                Some(fingerprint),
                vec![],
                None,
            ),
        ],
    );
    assert!(archive_work_item(directory.path(), id).is_err());
}

#[test]
fn malformed_event_json_remains_rejected() {
    let directory = repository();
    let id = "WI-457-EVENT-MALFORMED";
    ready(&directory, id);
    finish_work_item(directory.path(), id).expect("finish");
    let path = event_path(&directory, id);
    let mut text = fs::read_to_string(&path).expect("events");
    text.push_str("{not-json}\n");
    fs::write(&path, text).expect("tamper");
    assert!(archive_work_item(directory.path(), id).is_err());
}

#[test]
fn correction_and_supersession_events_require_a_prior_event() {
    let directory = repository();
    let id = "WI-457-CORRECTION-BINDING";
    ready(&directory, id);
    finish_work_item(directory.path(), id).expect("finish");
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let path = event_path(&directory, id);
    append_events(
        &path,
        &[event(
            &repository_id,
            id,
            "unbound-correction",
            "event_corrected",
            None,
            Vec::new(),
            None,
        )],
    );
    assert!(archive_work_item(directory.path(), id).is_err());
}

#[test]
fn event_relationships_must_reference_prior_events() {
    let directory = repository();
    let id = "WI-457-RELATION-ORDER";
    ready(&directory, id);
    finish_work_item(directory.path(), id).expect("finish");
    let repository_id = cockpit_repository::repository_id(directory.path()).to_string();
    let path = event_path(&directory, id);
    append_events(
        &path,
        &[event(
            &repository_id,
            id,
            "self-reference",
            "warning",
            None,
            vec!["self-reference".into()],
            None,
        )],
    );
    assert!(archive_work_item(directory.path(), id).is_err());
}

#[test]
fn finish_projects_residual_risk_as_a_stable_finding_fingerprint() {
    let directory = repository();
    let id = "WI-457-GENERATED-FINGERPRINT";
    ready(&directory, id);
    finish_work_item(directory.path(), id).expect("finish");
    let path = event_path(&directory, id);
    let events = fs::read_to_string(path)
        .expect("events")
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("event JSON"))
        .collect::<Vec<_>>();
    let risks = events
        .iter()
        .filter(|event| event["eventType"] == "risk")
        .collect::<Vec<_>>();
    assert_eq!(risks.len(), 1);
    let fingerprint = risks[0]["findingFingerprint"]
        .as_str()
        .expect("risk fingerprint");
    assert!(fingerprint.starts_with("sha256:") && fingerprint.len() == 71);
    archive_work_item(directory.path(), id).expect("archive generated events");
}

#[test]
fn event_json_keeps_finding_fingerprint_camel_case() {
    let value = serde_json::to_value(event(
        "sha256:repo",
        "WI-457",
        "finding-1",
        "finding",
        Some("sha256:1"),
        Vec::new(),
        None,
    ))
    .expect("event value");
    assert_eq!(
        value["findingFingerprint"],
        Value::String("sha256:1".into())
    );
}

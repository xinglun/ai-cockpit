use cockpit_core::{DecisionState, Digest};
use cockpit_protocol::{
    HumanBenefitReport, OutcomeClaim, OutcomeReportBindings, OutcomeReportSections, OutcomeState,
    OutcomeV2, ResourceFinalizationContext, TaskOutcomeReport,
};
use cockpit_repository::{
    WorkItemStartOptions, archive_work_item, checkpoint_work_item, finish_work_item, outcome_v2,
    plan_resource_finalization, preflight_work_item, record_verification, render_human_outcome,
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

struct RenderFixture {
    state: OutcomeState,
    decision_state: DecisionState,
    historical_status: Option<String>,
    sections: OutcomeReportSections,
    unknowns: Vec<String>,
    evidence_refs: Vec<String>,
}

fn render_fixture(directory: &tempfile::TempDir, id: &str, fixture: RenderFixture) -> String {
    let RenderFixture {
        state,
        decision_state,
        historical_status,
        sections,
        unknowns,
        evidence_refs,
    } = fixture;
    let report = TaskOutcomeReport {
        format: "ai-cockpit.task-outcome".into(),
        schema_version: 1,
        work_item_id: id.into(),
        status: state.clone(),
        human_status_color: decision_state.clone(),
        bindings: OutcomeReportBindings {
            repository_id: cockpit_repository::repository_id(directory.path()).to_string(),
            work_item_id: id.into(),
            evidence_refs: evidence_refs.clone(),
            repository_snapshot_digest: None,
        },
        sections,
        failed_gate: None,
        recovery_condition: None,
    };
    let outcome = OutcomeV2 {
        schema_version: 2,
        repository_id: cockpit_repository::repository_id(directory.path()).to_string(),
        work_item_id: id.into(),
        state: state.clone(),
        decision_state: Some(decision_state),
        summary: "fixture summary".into(),
        acceptance_results: vec!["fixture acceptance".into()],
        unknowns: unknowns.clone(),
        evidence_refs: evidence_refs.clone(),
        human_benefit_report: HumanBenefitReport {
            state: OutcomeState::Unknown,
            user_visible_changes: Vec::new(),
            affected_users: Vec::new(),
            unknowns,
            evidence_refs,
        },
        task_outcome_report: Some(report),
        failed_gate: None,
        recovery_condition: None,
        recovery_decision: None,
        historical_status,
    };
    render_human_outcome(directory.path(), &outcome, "en")
}

#[test]
fn human_renderer_separates_verification_lifecycle_and_human_decision() {
    let directory = repository();
    let id = "WI-OUTCOME-TRUST";
    fs::write(
        directory
            .path()
            .join(format!(".ai/work-items/active/{id}.summary.json")),
        serde_json::to_vec_pretty(&serde_json::json!({"state":"implementation_active"}))
            .expect("summary"),
    )
    .expect("summary write");
    let text = render_fixture(
        &directory,
        id,
        RenderFixture {
            state: OutcomeState::Verified,
            decision_state: DecisionState::Green,
            historical_status: None,
            sections: OutcomeReportSections::default(),
            unknowns: vec![],
            evidence_refs: vec![".ai/evidence/WI-OUTCOME-TRUST.verification.json".into()],
        },
    );
    assert!(text.contains("Outcome: 🟢 Declared verification passed — WI-OUTCOME-TRUST"));
    assert!(text.contains("Verification: Declared verification passed"));
    assert!(text.contains("Lifecycle: Implementation active"));
    assert!(text.contains("Human decision: Not recorded"));
    assert!(text.contains("Governance signal: Green; not a human approval"));
    assert!(!text.contains("Outcome: 🟢 Success"));

    let decision_path = directory
        .path()
        .join(format!(".ai/decisions/{id}.close.json"));
    fs::write(
        &decision_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "workItemId": id,
            "repositoryId": cockpit_repository::repository_id(directory.path()).to_string(),
            "state": "closed",
            "decisionState": "confirmed",
            "humanDecision": "approved",
            "structuredDecision": {
                "decision": "approved",
                "actor": "human:owner",
                "authoritySource": "explicit-user-authorization",
                "reason": "reviewed the evidence",
                "evidenceRefs": [".ai/evidence/WI-OUTCOME-TRUST.verification.json"],
                "policyRefs": ["project-policy"],
                "decidedAt": "2026-09-08T00:00:00Z",
                "resumeCondition": "rerun after base changes"
            }
        }))
        .expect("decision"),
    )
    .expect("decision write");
    let text = render_fixture(
        &directory,
        id,
        RenderFixture {
            state: OutcomeState::Verified,
            decision_state: DecisionState::Green,
            historical_status: None,
            sections: OutcomeReportSections::default(),
            unknowns: vec![],
            evidence_refs: vec![".ai/evidence/WI-OUTCOME-TRUST.verification.json".into()],
        },
    );
    assert!(text.contains("Human decision: Recorded: approved"));
    assert!(text.contains("Authority source: explicit-user-authorization"));
    assert!(text.contains("Assurance level: Unknown"));
    assert!(!text.contains("Human decision: Not recorded"));

    let mut decision_record: serde_json::Value =
        serde_json::from_slice(&fs::read(&decision_path).expect("decision record"))
            .expect("decision JSON");
    decision_record["assurance"] = serde_json::json!("repository_verified");
    fs::write(
        &decision_path,
        serde_json::to_vec_pretty(&decision_record).expect("decision with assurance"),
    )
    .expect("decision with assurance write");
    let text = render_fixture(
        &directory,
        id,
        RenderFixture {
            state: OutcomeState::Verified,
            decision_state: DecisionState::Green,
            historical_status: None,
            sections: OutcomeReportSections::default(),
            unknowns: vec![],
            evidence_refs: vec![".ai/evidence/WI-OUTCOME-TRUST.verification.json".into()],
        },
    );
    assert!(text.contains("Assurance level: repository_verified"));
}

#[test]
fn human_renderer_does_not_infer_risk_absence_or_test_strength_from_empty_fields() {
    let directory = repository();
    let id = "WI-OUTCOME-RISK-BOUNDARY";
    let evidence = ".ai/evidence/WI-OUTCOME-RISK-BOUNDARY.verification.json".to_owned();
    let text = render_fixture(
        &directory,
        id,
        RenderFixture {
            state: OutcomeState::Verified,
            decision_state: DecisionState::Green,
            historical_status: None,
            sections: OutcomeReportSections::default(),
            unknowns: vec![],
            evidence_refs: vec![evidence.clone()],
        },
    );
    assert!(text.contains("Risk findings: Not recorded"));
    assert!(!text.contains("No risks"));
    assert!(!text.contains("Tests were not weakened"));

    let mut sections = OutcomeReportSections::default();
    sections.risks.push(OutcomeClaim {
        text: "No test weakening rule was triggered.".into(),
        evidence_refs: vec![evidence],
        inference: false,
    });
    let text = render_fixture(
        &directory,
        id,
        RenderFixture {
            state: OutcomeState::Verified,
            decision_state: DecisionState::Green,
            historical_status: None,
            sections,
            unknowns: vec![],
            evidence_refs: vec![".ai/evidence/WI-OUTCOME-RISK-BOUNDARY.verification.json".into()],
        },
    );
    assert!(
        text.contains(
            "Test-weakening scan: no trigger was recorded for the referenced check scope"
        )
    );
    assert!(text.contains("does not prove that tests were not weakened"));
    assert!(!text.contains("Tests were not weakened."));
}

#[test]
fn human_renderer_preserves_historical_and_superseded_distinctions() {
    let directory = repository();
    let historical = render_fixture(
        &directory,
        "WI-HISTORICAL",
        RenderFixture {
            state: OutcomeState::Unknown,
            decision_state: DecisionState::Yellow,
            historical_status: Some("runtime_historical".into()),
            sections: OutcomeReportSections::default(),
            unknowns: vec!["historical_evidence_not_revalidated".into()],
            evidence_refs: vec![".ai/evidence/historical.json".into()],
        },
    );
    assert!(historical.contains("Historical verification evidence was not revalidated"));
    assert!(!historical.contains("Repair the missing evidence"));
    assert!(historical.contains("not a current failure"));

    let superseded = render_fixture(
        &directory,
        "WI-SUPERSEDED",
        RenderFixture {
            state: OutcomeState::Unknown,
            decision_state: DecisionState::Yellow,
            historical_status: Some("superseded".into()),
            sections: OutcomeReportSections::default(),
            unknowns: vec!["historical_evidence_not_current".into()],
            evidence_refs: vec![".ai/evidence/superseded.json".into()],
        },
    );
    assert!(superseded.contains("Superseded historical item"));
    assert!(superseded.contains("successor owns follow-up work"));
    assert!(!superseded.contains("Repair the missing evidence"));
}

#[test]
fn human_renderer_keeps_partial_missing_and_stale_verification_distinct() {
    let directory = repository();
    let partial = render_fixture(
        &directory,
        "WI-PARTIAL",
        RenderFixture {
            state: OutcomeState::Partial,
            decision_state: DecisionState::Yellow,
            historical_status: None,
            sections: OutcomeReportSections::default(),
            unknowns: vec!["verification_scope_incomplete".into()],
            evidence_refs: vec![".ai/evidence/partial.json".into()],
        },
    );
    assert!(partial.contains("Outcome: 🟡 Partial verification — WI-PARTIAL"));
    assert!(partial.contains("verification_scope_incomplete"));

    let missing = render_fixture(
        &directory,
        "WI-MISSING",
        RenderFixture {
            state: OutcomeState::NotReady,
            decision_state: DecisionState::Yellow,
            historical_status: None,
            sections: OutcomeReportSections::default(),
            unknowns: vec!["required_evidence_missing".into()],
            evidence_refs: vec![],
        },
    );
    assert!(missing.contains("Outcome: 🟡 Verification not ready — WI-MISSING"));
    assert!(missing.contains("required_evidence_missing"));
    assert!(!missing.contains("No risks"));

    let stale = render_fixture(
        &directory,
        "WI-STALE",
        RenderFixture {
            state: OutcomeState::Unknown,
            decision_state: DecisionState::Red,
            historical_status: None,
            sections: OutcomeReportSections::default(),
            unknowns: vec!["evidence_snapshot_stale".into(), "identity_mismatch".into()],
            evidence_refs: vec![".ai/evidence/stale.json".into()],
        },
    );
    assert!(stale.contains("Outcome: 🔴 Verification status unknown — WI-STALE"));
    assert!(stale.contains("evidence_snapshot_stale"));
    assert!(stale.contains("identity_mismatch"));
    assert!(stale.contains("Verification evidence is invalid or does not match"));
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

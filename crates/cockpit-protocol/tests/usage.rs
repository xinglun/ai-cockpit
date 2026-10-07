use cockpit_core::{DecisionState, Digest};
use cockpit_protocol::{
    OutcomeReportBindings, OutcomeReportSections, OutcomeState, TaskOutcomeReport,
    UsageRecordRequest, UsageSourceKind, UsageUnit,
};

fn request() -> UsageRecordRequest {
    UsageRecordRequest {
        schema_version: 1,
        repository_id: Digest::sha256_bytes(b"repository").to_string(),
        work_item_id: "WI-USAGE".into(),
        source_event_id: "turn-1".into(),
        source_kind: UsageSourceKind::AgentDeclared,
        actor: None,
        configured_model: Some("configured-model".into()),
        reported_model: Some("reported-model".into()),
        role: "implementer".into(),
        phase: "implementation".into(),
        unit: UsageUnit::Turn,
        input_tokens: None,
        output_tokens: Some(9),
        cached_input_tokens: None,
        reasoning_tokens: Some(4),
        source_observed_at: None,
        evidence_ref: ".ai/evidence/source.json".into(),
        evidence_digest: Digest::sha256_bytes(b"evidence"),
    }
}

#[test]
fn usage_request_keeps_unknown_counts_null_and_models_separate() {
    let request = request();
    request.validate().expect("valid request");
    let value = serde_json::to_value(&request).expect("serialize usage");
    assert!(value["inputTokens"].is_null());
    assert_eq!(value["reportedModel"], "reported-model");
    assert_eq!(value["configuredModel"], "configured-model");
    assert_eq!(value["sourceKind"], "agent-declared");
    assert_eq!(
        serde_json::from_value::<UsageRecordRequest>(value).expect("decode request"),
        request
    );
}

#[test]
fn usage_request_rejects_invalid_subsets_and_identity() {
    let mut request = request();
    request.cached_input_tokens = Some(1);
    assert!(request.validate().is_err(), "unknown input is not zero");
    request.cached_input_tokens = None;
    request.reasoning_tokens = Some(10);
    assert!(request.validate().is_err(), "reasoning exceeds output");
    request.reasoning_tokens = Some(4);
    request.work_item_id = "../other".into();
    assert!(
        request.validate().is_err(),
        "path-like Work Item IDs reject"
    );
}

#[test]
fn historical_report_without_usage_remains_byte_compatible() {
    let report = TaskOutcomeReport {
        format: "ai-cockpit.task-outcome".into(),
        schema_version: 1,
        work_item_id: "WI-HISTORICAL".into(),
        status: OutcomeState::Unknown,
        human_status_color: DecisionState::Yellow,
        bindings: OutcomeReportBindings {
            repository_id: Digest::sha256_bytes(b"repository").to_string(),
            work_item_id: "WI-HISTORICAL".into(),
            evidence_refs: Vec::new(),
            repository_snapshot_digest: None,
        },
        sections: OutcomeReportSections::default(),
        usage: None,
        release: None,
        failed_gate: None,
        recovery_condition: None,
    };
    let bytes = serde_json::to_vec(&report).expect("legacy report bytes");
    assert!(!String::from_utf8_lossy(&bytes).contains("\"usage\""));
    let decoded: TaskOutcomeReport = serde_json::from_slice(&bytes).expect("legacy decode");
    assert_eq!(decoded.usage, None);
    assert_eq!(
        serde_json::to_vec(&decoded).expect("legacy re-encode"),
        bytes
    );
}

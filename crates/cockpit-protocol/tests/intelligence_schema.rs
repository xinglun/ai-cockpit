use cockpit_core::{DecisionState, Digest};
use cockpit_protocol::{
    AdopterCapabilityState, CapabilityConfidence, CapabilityOwnership, CapabilityTruth,
    CapabilityTruthRegistry, FactOrigin, HumanBenefitReport, ImplementationApproach, OutcomeState,
    OutcomeV2, TaskOutcomeReport, TraceableDerivation, TraceableFact, TruthState,
    WorkItemCompatibility, WorkItemIntelligence, WorkItemStatusIndex, WorkItemStatusIndexEntry,
};

#[test]
fn v2_records_round_trip_with_explicit_unknowns_and_provenance() {
    let digest = Digest::sha256_bytes(b"snapshot");
    let approach = ImplementationApproach {
        schema_version: 2,
        repository_id: "sha256:repo".into(),
        work_item_id: "WI-75".into(),
        repository_snapshot_digest: digest.clone(),
        facts: vec![TraceableFact {
            key: "language".into(),
            value: serde_json::json!("Rust"),
            origin: FactOrigin::Observed,
            evidence_refs: vec!["repository-snapshot".into()],
            confidence: "high".into(),
        }],
        derivations: vec![TraceableDerivation {
            key: "verificationCapability".into(),
            value: serde_json::json!("cargo test"),
            rule: "observer.quality_commands_from_detected_build_system".into(),
            input_fact_keys: vec!["language".into()],
            evidence_refs: vec!["repository-snapshot".into()],
            confidence: "medium".into(),
        }],
        unknowns: vec!["intent".into()],
        evidence_refs: vec!["repository-snapshot".into()],
    };
    let bytes = serde_json::to_vec(&approach).expect("encode");
    let decoded: ImplementationApproach = serde_json::from_slice(&bytes).expect("decode");
    assert_eq!(decoded, approach);

    let outcome = OutcomeV2 {
        schema_version: 2,
        repository_id: "sha256:repo".into(),
        work_item_id: "WI-75".into(),
        state: OutcomeState::Verified,
        decision_state: Some(DecisionState::Green),
        summary: "verification passed".into(),
        acceptance_results: vec!["tests pass".into()],
        unknowns: vec!["user_visible_benefit_not_declared".into()],
        evidence_refs: vec![".ai/evidence/WI-75.verification.json".into()],
        human_benefit_report: HumanBenefitReport {
            state: OutcomeState::Unknown,
            user_visible_changes: Vec::new(),
            affected_users: Vec::new(),
            unknowns: vec!["user_visible_benefit_not_declared".into()],
            evidence_refs: vec![".ai/evidence/WI-75.verification.json".into()],
        },
        task_outcome_report: None,
        failed_gate: None,
        recovery_condition: None,
        recovery_decision: None,
        historical_status: None,
    };
    let value = serde_json::to_value(&outcome).expect("encode");
    assert_eq!(value["humanBenefitReport"]["state"], "unknown");
    assert_eq!(value["unknowns"][0], "user_visible_benefit_not_declared");
}

#[test]
fn capability_and_parallel_records_are_strict_and_repository_bound() {
    let digest = Digest::sha256_bytes(b"snapshot");
    let registry = CapabilityTruthRegistry {
        schema_version: 1,
        repository_id: "sha256:repo".into(),
        snapshot_digest: digest,
        runtime_version: "0.2.28".into(),
        runtime_digest: Digest::sha256_bytes(b"runtime"),
        capabilities: vec![CapabilityTruth {
            capability: "build:cargo".into(),
            state: TruthState::Observed,
            confidence: CapabilityConfidence::High,
            source: FactOrigin::Observed,
            evidence_refs: vec!["repository-snapshot".into()],
            verification: None,
            unknowns: Vec::new(),
        }],
        adopter_capabilities: Vec::new(),
        exclusions: Vec::new(),
        unknowns: Vec::new(),
    };
    let value = serde_json::to_value(&registry).expect("encode");
    assert_eq!(value["repositoryId"], "sha256:repo");
    assert_eq!(value["capabilities"][0]["state"], "observed");

    let intelligence = WorkItemIntelligence {
        schema_version: 1,
        repository_id: "sha256:repo".into(),
        work_item_id: "WI-75".into(),
        depends_on: vec!["WI-74".into()],
        conflicts_with: vec!["WI-77".into()],
        parallelizable: false,
        unknowns: vec!["scope_overlap_not_checked".into()],
    };
    let compatibility = WorkItemCompatibility {
        repository_id: "sha256:repo".into(),
        work_item_id: "WI-75".into(),
        compatible: false,
        dependencies_satisfied: false,
        conflicts: vec!["WI-77".into()],
        reasons: vec!["dependency_active:WI-74".into()],
    };
    let encoded = serde_json::to_vec(&(intelligence, compatibility)).expect("encode");
    assert!(!encoded.is_empty());
}

#[test]
fn capability_registry_accepts_runtime_bound_adopter_truth_and_exclusions() {
    let value = serde_json::json!({
        "schemaVersion": 1,
        "repositoryId": "sha256:repo",
        "snapshotDigest": cockpit_core::Digest::sha256_bytes(b"snapshot"),
        "runtimeVersion": "0.2.28",
        "runtimeDigest": cockpit_core::Digest::sha256_bytes(b"runtime"),
        "capabilities": [],
        "adopterCapabilities": [{
            "id": "work_item_status_interface",
            "state": "runtime_supported",
            "ownership": "runtime",
            "adopterFacing": true,
            "evidenceRefs": ["runtime:sha256:runtime"],
            "unknowns": []
        }],
        "exclusions": [{
            "id": "hosted_ci",
            "ownership": "external_provider",
            "reason": "Hosted execution is external evidence."
        }],
        "unknowns": []
    });

    let registry: CapabilityTruthRegistry =
        serde_json::from_value(value).expect("runtime-bound adopter capability registry");
    assert_eq!(registry.runtime_version, "0.2.28");
    assert_eq!(
        registry.adopter_capabilities[0].state,
        AdopterCapabilityState::RuntimeSupported
    );
    assert_eq!(
        registry.exclusions[0].ownership,
        CapabilityOwnership::ExternalProvider
    );
}

#[test]
fn adopter_capability_state_vocabulary_keeps_truth_levels_distinct() {
    let states = [
        (
            "runtime_supported",
            AdopterCapabilityState::RuntimeSupported,
        ),
        ("observed", AdopterCapabilityState::Observed),
        (
            "profile_confirmed",
            AdopterCapabilityState::ProfileConfirmed,
        ),
        ("adopter_accepted", AdopterCapabilityState::AdopterAccepted),
        ("external", AdopterCapabilityState::External),
        ("unknown", AdopterCapabilityState::Unknown),
    ];

    for (encoded, expected) in states {
        let decoded: AdopterCapabilityState =
            serde_json::from_value(serde_json::json!(encoded)).expect("known truth state");
        assert_eq!(decoded, expected);
    }
}

#[test]
fn all_work_item_status_index_is_strict_and_digest_bound() {
    let status_digest = Digest::sha256_bytes(b"status");
    let index = WorkItemStatusIndex {
        schema_version: 1,
        repository_id: "sha256:repo".into(),
        snapshot_digest: Digest::sha256_bytes(b"snapshot"),
        counts: [("green".into(), 1), ("yellow".into(), 0)]
            .into_iter()
            .collect(),
        items: vec![WorkItemStatusIndexEntry {
            work_item_id: "WI-222".into(),
            governance_state: "green".into(),
            status_digest,
            status: None,
            unknowns: Vec::new(),
            diagnostics: Vec::new(),
        }],
        unknowns: Vec::new(),
        diagnostics: vec!["work_items_aggregated:1".into()],
        index_digest: Digest::sha256_bytes(b"index"),
    };

    let value = serde_json::to_value(&index).expect("encode status index");
    assert_eq!(value["repositoryId"], "sha256:repo");
    assert_eq!(value["items"][0]["workItemId"], "WI-222");
    assert_eq!(value["counts"]["green"], 1);
    let decoded: WorkItemStatusIndex = serde_json::from_value(value).expect("strict status index");
    assert_eq!(decoded, index);
}

#[test]
fn task_outcome_report_rejects_unknown_fields_and_keeps_claim_provenance() {
    let report = serde_json::json!({
        "format": "ai-cockpit.task-outcome",
        "schemaVersion": 1,
        "workItemId": "WI-136",
        "status": "verified",
        "humanStatusColor": "green",
        "bindings": {
            "repositoryId": "sha256:repo",
            "workItemId": "WI-136",
            "evidenceRefs": [".ai/evidence/WI-136.verification.json"]
        },
        "sections": {
            "outcomeSummary": [{
                "text": "verification passed",
                "evidenceRefs": [".ai/evidence/WI-136.verification.json"],
                "inference": false
            }],
            "taskOverview": [], "deliveredChanges": [], "findings": [],
            "risks": [], "warnings": [], "limitations": [],
            "nonRiskExplanations": [], "forbiddenClaims": [],
            "interventions": [], "forcedStops": [], "resolutions": [],
            "recurrencePrevention": [], "avoidedImpact": [],
            "residualRisks": [], "humanDecisions": [], "evidence": []
        }
    });
    let decoded: TaskOutcomeReport = serde_json::from_value(report).expect("report");
    assert!(!decoded.sections.outcome_summary[0].evidence_refs.is_empty());
    let mut unknown = serde_json::to_value(&decoded).expect("encode");
    unknown["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<TaskOutcomeReport>(unknown).is_err());
}

use cockpit_core::Digest;
use cockpit_protocol::{
    CapabilityConfidence, CapabilityTruth, CapabilityTruthRegistry, FactOrigin, HumanBenefitReport,
    ImplementationApproach, OutcomeState, OutcomeV2, TraceableDerivation, TraceableFact,
    TruthState, WorkItemCompatibility, WorkItemIntelligence,
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
        capabilities: vec![CapabilityTruth {
            capability: "build:cargo".into(),
            state: TruthState::Observed,
            confidence: CapabilityConfidence::High,
            source: FactOrigin::Observed,
            evidence_refs: vec!["repository-snapshot".into()],
            verification: None,
            unknowns: Vec::new(),
        }],
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

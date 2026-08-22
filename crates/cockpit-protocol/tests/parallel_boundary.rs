use cockpit_protocol::{
    ConcurrencyBoundary, Contract, PARALLEL_SLOT_LEASE_SCHEMA_VERSION, ParallelSlotLease,
};

fn boundary() -> ConcurrencyBoundary {
    ConcurrencyBoundary {
        schema_version: 1,
        implementation_paths: vec!["crates/cockpit-repository/**".into()],
        generated_evidence_paths: vec![".ai/evidence/**".into()],
        verification_output_paths: vec!["target/**".into()],
        serialized_projection_paths: vec![".ai/work-items/**".into()],
        max_workers: 2,
        reason: "repository and generated state are isolated by path".into(),
    }
}

#[test]
fn boundary_is_additive_and_strict() {
    let value = serde_json::to_value(boundary()).expect("encode");
    assert_eq!(value["schemaVersion"], 1);
    assert_eq!(value["maxWorkers"], 2);
    let decoded: ConcurrencyBoundary = serde_json::from_value(value).expect("decode");
    assert_eq!(decoded, boundary());
    let unknown = serde_json::json!({
        "schemaVersion": 1,
        "implementationPaths": ["src/**"],
        "generatedEvidencePaths": [],
        "verificationOutputPaths": [],
        "serializedProjectionPaths": [],
        "maxWorkers": 1,
        "reason": "bounded",
        "unexpected": true
    });
    assert!(serde_json::from_value::<ConcurrencyBoundary>(unknown).is_err());
}

#[test]
fn legacy_contract_without_boundary_remains_readable() {
    let contract = serde_json::json!({
        "protocolVersion": 1,
        "repositoryId": "sha256:repository",
        "workItemId": "WI-legacy",
        "intent": "intent",
        "goal": "goal",
        "scope": ["src/**"],
        "outOfScope": [],
        "risk": "normal",
        "authority": "authorized",
        "acceptanceCriteria": [],
        "requiredEvidenceClasses": [],
        "baseRevision": "HEAD",
        "projectProfileDigest": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "repositorySnapshotDigest": "sha256:1111111111111111111111111111111111111111111111111111111111111111"
    });
    let decoded: Contract = serde_json::from_value(contract).expect("legacy contract");
    assert!(decoded.concurrency_boundary.is_none());
}

#[test]
fn slot_lease_round_trip_is_repository_bound() {
    let lease = ParallelSlotLease {
        schema_version: PARALLEL_SLOT_LEASE_SCHEMA_VERSION,
        repository_id: "sha256:repository".into(),
        work_item_id: "WI-123".into(),
        slot_id: 0,
        lease_id: "lease-1".into(),
        max_workers: 2,
        acquired_at: "2026-08-22T00:00:00Z".into(),
    };
    let bytes = serde_json::to_vec(&lease).expect("encode");
    assert_eq!(
        serde_json::from_slice::<ParallelSlotLease>(&bytes).unwrap(),
        lease
    );
}

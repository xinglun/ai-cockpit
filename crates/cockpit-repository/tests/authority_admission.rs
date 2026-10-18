//! P2b wiring tests. No production source or permission is installed.
use cockpit_core::Digest;
use serde_json::json;
use std::{fs, path::Path};
use tempfile::{TempDir, tempdir};

const WI: &str = "WI-AUTHORITY-FIXTURE";

fn fixture(required: bool) -> TempDir {
    let dir = tempdir().unwrap();
    let active = dir.path().join(".ai/work-items/active");
    fs::create_dir_all(&active).unwrap();
    let digest = Digest::sha256_bytes(b"isolated authority wiring fixture");
    let mut contract = json!({
        "protocolVersion":1,"repositoryId":cockpit_repository::repository_id(dir.path()),
        "workItemId":WI,"intent":"test only","goal":"test only",
        "scope":["src/**"],"outOfScope":[],"risk":"normal","authority":"authorized",
        "acceptanceCriteria":["test only"],"requiredEvidenceClasses":["verification"],
        "sources":[],"verification":[],"baseRevision":"a".repeat(40),
        "projectProfileDigest":digest,"repositorySnapshotDigest":digest
    });
    if required {
        contract["governanceProfile"] = json!({"goalAuthority":{"required":true}});
    }
    fs::write(
        active.join(format!("{WI}.contract.json")),
        serde_json::to_vec(&contract).unwrap(),
    )
    .unwrap();
    fs::write(
        active.join(format!("{WI}.summary.json")),
        b"{\"state\":\"checkpointed\",\"checkpointCount\":1}",
    )
    .unwrap();
    dir
}

fn summary(root: &Path) -> Vec<u8> {
    fs::read(root.join(format!(".ai/work-items/active/{WI}.summary.json"))).unwrap()
}

#[test]
fn b01_required_no_runtime_controls_reject_before_any_summary_write() {
    let dir = fixture(true);
    let before = summary(dir.path());
    let result = cockpit_repository::record_work_item_governance_controls(
        dir.path(),
        WI,
        &json!({"intentAlignment":{"state":"unknown"}}),
    );
    assert!(
        result.is_err(),
        "required declaration must not authorize a controls write"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("goal_authority_source_unsupported")
    );
    assert_eq!(summary(dir.path()), before);
    assert!(!dir.path().join(".ai/locks").exists());
}

#[test]
fn b13_unconfigured_legacy_controls_keep_existing_behavior() {
    let dir = fixture(false);
    let result = cockpit_repository::record_work_item_governance_controls(
        dir.path(),
        WI,
        &json!({"intentAlignment":{"state":"unknown"}}),
    )
    .unwrap();
    assert_eq!(result["intentAlignment"]["state"], "unknown");
    assert_eq!(result["state"], "checkpointed");
}

fn runtime() -> cockpit_protocol::RuntimeContext {
    cockpit_protocol::RuntimeContext {
        runtime_version: "1.0.1".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"test runtime is not actor authentication"),
    }
}

#[test]
fn b01_b11_public_entry_inventory_rejects_before_other_validation_or_effects() {
    use cockpit_repository as repo;
    type Probe = fn(&Path) -> Result<(), repo::ObserverError>;
    let probes: &[(&str, Probe)] = &[
        ("controls-with-runtime", |p| {
            repo::record_work_item_governance_controls_with_runtime(p, WI, &json!({}), &runtime())
                .map(|_| ())
        }),
        ("controls-validation", |p| {
            repo::validate_work_item_governance_controls(p, WI).map(|_| ())
        }),
        ("action-admission", |p| {
            repo::require_current_action_admission(p, WI, "record_governance_controls", &runtime())
                .map(|_| ())
        }),
        ("checkpoint", |p| {
            repo::checkpoint_work_item(p, WI).map(|_| ())
        }),
        ("direct-revalidation", |p| {
            repo::revalidate_contract_amendment(p, WI, "fixture").map(|_| ())
        }),
        ("preflight", |p| {
            repo::preflight_work_item(
                p,
                &p.join(format!(".ai/work-items/active/{WI}.contract.json")),
            )
            .map(|_| ())
        }),
        ("finish", |p| repo::finish_work_item(p, WI).map(|_| ())),
        ("archive", |p| repo::archive_work_item(p, WI).map(|_| ())),
        ("close", |p| repo::close_work_item(p, WI).map(|_| ())),
        ("reconcile", |p| {
            repo::reconcile_active_artifacts(p, WI).map(|_| ())
        }),
        ("outcome", |p| repo::outcome_v2(p, WI).map(|_| ())),
        ("status", |p| repo::status(p).map(|_| ())),
        ("capability-business-query", |p| {
            repo::capability_truth_registry(p).map(|_| ())
        }),
        ("execution-context", |p| {
            repo::RepositoryExecutionContext::capture(p).map(|_| ())
        }),
        ("approach", |p| {
            repo::implementation_approach(p, WI).map(|_| ())
        }),
        ("approach-read-only", |p| {
            repo::implementation_approach_read_only(p, WI).map(|_| ())
        }),
        ("usage-read", |p| {
            repo::read_work_item_usage(p, WI, None).map(|_| ())
        }),
        ("knowledge-generate", |p| {
            repo::generate_knowledge(p).map(|_| ())
        }),
        ("scaffold", |p| {
            repo::scaffold_work_item(p, WI, "implementation").map(|_| ())
        }),
        ("amendment", |p| {
            repo::apply_work_item_contract_amendment(
                p,
                WI,
                &cockpit_protocol::ContractAmendmentRequest {
                    schema_version: 1,
                    change_id: "fixture-only".into(),
                    expected_contract_digest: Digest::sha256_bytes(b"not a permit"),
                    reason: "test only".into(),
                    changes: vec![],
                },
                &runtime(),
            )
            .map(|_| ())
        }),
    ];
    let mut missing = vec![];
    for (name, probe) in probes {
        let dir = fixture(true);
        let before = summary(dir.path());
        let error = probe(dir.path())
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();
        if !error.contains("goal_authority_source_unsupported") {
            missing.push(format!("{name}: missing authority rejection"));
        }
        if summary(dir.path()) != before || dir.path().join(".ai/locks").exists() {
            missing.push(format!("{name}: changed protected fixture"));
        }
    }
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}

#[test]
fn b01_material_plan_rejects_before_scanning_business_material() {
    let dir = fixture(true);
    let before = summary(dir.path());
    let result = cockpit_repository::plan_work_item_material_review(dir.path(), WI);
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("goal_authority_source_unsupported")
    );
    assert_eq!(summary(dir.path()), before);
}

use cockpit_core::{DecisionState, Digest};
use cockpit_protocol::RuntimeContext;
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions, attach,
    checkpoint_work_item, finish_work_item_with_runtime, outcome_v2_with_runtime,
    preflight_work_item_with_runtime, record_verification_with_runtime,
    run_repository_verification, start_work_item_with_options,
};
use serde_json::Value;
use std::{fs, process::Command};

type EvidenceMutation = (&'static str, fn(&mut Value));

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
    attach(directory.path()).expect("attach");
    directory
}

fn runtime(label: &str) -> RuntimeContext {
    RuntimeContext {
        runtime_version: "test-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(label.as_bytes()),
    }
}

fn start(directory: &tempfile::TempDir, id: &str) {
    start_work_item_with_options(
        directory.path(),
        id,
        "evidence assurance",
        "validate typed verification evidence",
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
}

fn record_typed(directory: &tempfile::TempDir, id: &str, current: &RuntimeContext) -> Value {
    let contract = directory
        .path()
        .join(format!(".ai/work-items/active/{id}.contract.json"));
    preflight_work_item_with_runtime(directory.path(), &contract, current).expect("preflight");
    checkpoint_work_item(directory.path(), id).expect("checkpoint");
    let run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "project-command-0".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec!["**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: current.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .expect("verify");
    let mut raw = serde_json::to_value(&run.receipt).expect("receipt JSON");
    raw["runtimeVersion"] = current.runtime_version.clone().into();
    raw["runtimeDigest"] = current.runtime_digest.to_string().into();
    record_verification_with_runtime(directory.path(), id, &raw, current, &run.final_snapshot)
        .expect("record typed evidence")
}

#[test]
fn strict_evidence_rejects_unknown_envelope_and_nested_fields() {
    let cases: [EvidenceMutation; 5] = [
        ("unknown-envelope", |evidence: &mut Value| {
            evidence["unexpected"] = true.into();
        }),
        ("unknown-receipt", |evidence: &mut Value| {
            evidence["receipt"]["unexpected"] = true.into();
        }),
        ("unknown-result", |evidence: &mut Value| {
            let result = evidence["receipt"]["results"]
                .as_array_mut()
                .and_then(|results| results.first_mut())
                .expect("verification should produce a result");
            result["unexpected"] = true.into();
        }),
        ("missing-nested-repository", |evidence: &mut Value| {
            evidence["receipt"]
                .as_object_mut()
                .expect("receipt object")
                .remove("repositoryId");
        }),
        ("malformed-receipt", |evidence: &mut Value| {
            evidence["receipt"] = Value::String("not-a-receipt".into());
        }),
    ];
    for (name, mutate) in cases {
        let directory = repository();
        let current = runtime(name);
        start(&directory, "WI-110-STRICT");
        record_typed(&directory, "WI-110-STRICT", &current);
        let path = directory
            .path()
            .join(".ai/evidence/WI-110-STRICT.verification.json");
        let mut evidence: Value =
            serde_json::from_slice(&fs::read(&path).expect("evidence")).expect("evidence JSON");
        mutate(&mut evidence);
        fs::write(&path, serde_json::to_vec_pretty(&evidence).expect("JSON"))
            .expect("tamper evidence");
        let outcome =
            outcome_v2_with_runtime(directory.path(), "WI-110-STRICT", &current).expect("outcome");
        assert_eq!(outcome.decision_state, Some(DecisionState::Red), "{name}");
        assert_ne!(
            outcome.state,
            cockpit_protocol::OutcomeState::Verified,
            "{name}"
        );
    }
}

#[test]
fn current_runtime_lifecycle_rejects_foreign_runtime_evidence() {
    let directory = repository();
    let current = runtime("current");
    let foreign = runtime("foreign");
    start(&directory, "WI-110-RUNTIME");
    record_typed(&directory, "WI-110-RUNTIME", &current);
    let error = finish_work_item_with_runtime(directory.path(), "WI-110-RUNTIME", &foreign)
        .expect_err("foreign runtime evidence must not finish");
    assert!(error.to_string().contains("valid current receipt"));
    let outcome = outcome_v2_with_runtime(directory.path(), "WI-110-RUNTIME", &foreign)
        .expect("foreign-runtime outcome");
    assert_eq!(outcome.decision_state, Some(DecisionState::Red));
    assert_ne!(outcome.state, cockpit_protocol::OutcomeState::Verified);
}

#[test]
fn legacy_evidence_is_projected_as_historical_yellow_without_rewriting_bytes() {
    let directory = repository();
    let current = runtime("legacy-reader");
    start(&directory, "WI-110-LEGACY");
    let mut evidence = record_typed(&directory, "WI-110-LEGACY", &current);
    evidence
        .as_object_mut()
        .expect("evidence object")
        .remove("evidenceSchemaVersion");
    evidence
        .as_object_mut()
        .expect("evidence object")
        .remove("repositoryId");
    let path = directory
        .path()
        .join(".ai/evidence/WI-110-LEGACY.verification.json");
    let legacy_bytes = serde_json::to_vec_pretty(&evidence).expect("legacy JSON");
    fs::write(&path, &legacy_bytes).expect("write legacy evidence");
    let outcome = outcome_v2_with_runtime(directory.path(), "WI-110-LEGACY", &current)
        .expect("legacy outcome");
    assert_eq!(outcome.state, cockpit_protocol::OutcomeState::NotReady);
    assert_eq!(outcome.decision_state, Some(DecisionState::Yellow));
    assert!(
        outcome
            .unknowns
            .contains(&"legacy_evidence_historical".into())
    );
    assert_eq!(fs::read(&path).expect("legacy bytes"), legacy_bytes);
}

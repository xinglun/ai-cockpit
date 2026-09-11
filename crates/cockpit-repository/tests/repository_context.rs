use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use cockpit_core::Digest;
use cockpit_protocol::RuntimeContext;
use cockpit_repository::{
    ObservationConsistency, ObservationPhase, RepositoryExecutionContext, RuntimeSession, attach,
    governance_decision_for_observation_context, scaffold_work_item,
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn repository(name: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "cockpit-repository-context-{name}-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("repository");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .status()
            .expect("git init")
            .success()
    );
    root
}

#[test]
fn parallel_repository_contexts_do_not_share_scaffold_state() {
    let left = repository("left");
    let right = repository("right");
    std::thread::scope(|scope| {
        let left_handle = scope.spawn(|| {
            let profile = attach(&left).expect("attach left");
            let scaffold = scaffold_work_item(&left, "WI-LEFT", "code").expect("scaffold left");
            fs::write(left.join("left.txt"), "left\n").expect("left fact");
            (profile, scaffold)
        });
        let right_handle = scope.spawn(|| {
            let profile = attach(&right).expect("attach right");
            let scaffold = scaffold_work_item(&right, "WI-RIGHT", "docs").expect("scaffold right");
            fs::write(right.join("right.txt"), "right\n").expect("right fact");
            (profile, scaffold)
        });
        let (left_profile, left_scaffold) = left_handle.join().expect("left thread");
        let (right_profile, right_scaffold) = right_handle.join().expect("right thread");
        assert_ne!(left_profile.repository_id, right_profile.repository_id);
        assert_eq!(
            left_scaffold.known_facts.repository_id,
            left_profile.repository_id
        );
        assert_eq!(
            right_scaffold.known_facts.repository_id,
            right_profile.repository_id
        );
        assert_ne!(
            left_scaffold.known_facts.repository_snapshot_digest,
            right_scaffold.known_facts.repository_snapshot_digest
        );
        assert!(
            left.join(".ai/work-items/active/WI-LEFT.contract.json")
                .is_file()
        );
        assert!(
            !left
                .join(".ai/work-items/active/WI-RIGHT.contract.json")
                .exists()
        );
        assert!(
            right
                .join(".ai/work-items/active/WI-RIGHT.contract.json")
                .is_file()
        );
        assert!(
            !right
                .join(".ai/work-items/active/WI-LEFT.contract.json")
                .exists()
        );
    });
    fs::remove_dir_all(left).expect("cleanup left");
    fs::remove_dir_all(right).expect("cleanup right");
}

#[test]
fn observation_phase_context_binds_facts_and_rejects_source_mutation() {
    let root = repository("phase-context");
    fs::write(root.join("src.rs"), "fn value() -> u8 { 1 }\n").expect("source");
    attach(&root).expect("attach");
    let runtime = RuntimeContext {
        runtime_version: "0.2.87".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"phase-runtime"),
    };
    let contract_digest = Digest::sha256_bytes(b"contract");
    let context = RepositoryExecutionContext::capture(&root).expect("capture");
    let phase = context
        .observe_phase(
            ObservationPhase::BeforeGovernance,
            Some(&runtime),
            Some(contract_digest.clone()),
        )
        .expect("stable observation phase");

    assert_eq!(phase.phase(), ObservationPhase::BeforeGovernance);
    assert_eq!(phase.repository_id(), context.repository_id());
    assert_eq!(phase.runtime(), Some(&runtime));
    assert_eq!(phase.contract_digest(), Some(&contract_digest));
    assert_eq!(phase.consistency(), ObservationConsistency::Stable);
    assert_eq!(phase.observation(), context.observe().expect("observation"));
    assert!(!phase.snapshot_digest().as_str().is_empty());
    assert!(!phase.configuration_digest().as_str().is_empty());

    fs::write(root.join("src.rs"), "fn value() -> u8 { 2 }\n").expect("mutate source");
    let error = phase
        .validate_current()
        .expect_err("stale phase must fail closed");
    assert!(error.to_string().contains("observation phase"));
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn observation_phase_context_detects_governance_configuration_and_identity_changes() {
    let root = repository("phase-config");
    fs::write(root.join("src.rs"), "fn value() -> u8 { 1 }\n").expect("source");
    attach(&root).expect("attach");
    let context = RepositoryExecutionContext::capture(&root).expect("capture");
    let phase = context
        .observe_phase(ObservationPhase::BeforeGovernance, None, None)
        .expect("stable observation phase");

    fs::write(root.join(".ai/policy.json"), "{\"schemaVersion\":1}\n").expect("policy");
    let error = phase
        .validate_current()
        .expect_err("configuration mutation must fail closed");
    assert!(error.to_string().contains("configuration"));

    let config_path = root.join(".ai/cockpit.toml");
    let mut config = fs::read_to_string(&config_path).expect("config");
    config = config.replace("repository_id = \"", "repository_id = \"sha256:");
    fs::write(&config_path, config).expect("identity");
    let identity_error = phase
        .validate_current()
        .expect_err("identity mutation must fail closed");
    assert!(identity_error.to_string().contains("identity"));
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn observation_phase_requires_a_fresh_context_for_each_lifecycle_phase() {
    let root = repository("phase-boundary");
    fs::write(root.join("src.rs"), "fn value() -> u8 { 1 }\n").expect("source");
    attach(&root).expect("attach");
    let context = RepositoryExecutionContext::capture(&root).expect("capture");
    let before = context
        .observe_phase(ObservationPhase::BeforeGovernance, None, None)
        .expect("before phase");
    let error = before
        .require_phase(ObservationPhase::AfterExecution)
        .expect_err("phase reuse must fail closed");
    assert!(error.to_string().contains("phase"));

    fs::write(root.join("src.rs"), "fn value() -> u8 { 2 }\n").expect("execution mutation");
    let after_context = RepositoryExecutionContext::capture(&root).expect("refresh");
    let after = after_context
        .observe_phase(ObservationPhase::AfterExecution, None, None)
        .expect("after phase");
    after
        .require_phase(ObservationPhase::AfterExecution)
        .expect("fresh after phase");
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn observation_phase_rejects_a_symlinked_contract_before_canonicalizing_it() {
    use std::os::unix::fs::symlink;

    let root = repository("symlink-contract");
    attach(&root).expect("attach");
    scaffold_work_item(&root, "WI-SYMLINK-CONTRACT", "code").expect("scaffold");
    let contract = root.join(".ai/work-items/active/WI-SYMLINK-CONTRACT.contract.json");
    let target = root.join(".ai/work-items/active/WI-SYMLINK-CONTRACT.contract.target.json");
    fs::rename(&contract, &target).expect("move contract target");
    symlink(&target, &contract).expect("symlink contract");

    let context = RepositoryExecutionContext::capture(&root).expect("capture");
    let error = context
        .observe_phase_with_contract(ObservationPhase::BeforeGovernance, None, &contract)
        .expect_err("symlinked Contract must fail closed");
    assert!(error.to_string().contains("regular non-symlink"), "{error}");
}

#[test]
fn governance_decision_consumes_the_validated_observation_context() {
    let root = repository("governance-context");
    attach(&root).expect("attach");
    scaffold_work_item(&root, "WI-GOV-CONTEXT", "code").expect("scaffold");
    fs::write(root.join("src.rs"), "fn value() -> u8 { 1 }\n").expect("source");
    let contract_path = root.join(".ai/work-items/active/WI-GOV-CONTEXT.contract.json");
    let contract: cockpit_protocol::Contract =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract")).expect("parse");
    let repository_context = RepositoryExecutionContext::capture(&root).expect("capture");
    let observation = repository_context
        .observe_phase_with_contract(ObservationPhase::BeforeGovernance, None, &contract_path)
        .expect("observation");

    governance_decision_for_observation_context(&observation, &contract)
        .expect("decision uses stable context");

    let mut mismatched_contract = contract.clone();
    mismatched_contract.goal = "different Contract facts".into();
    let mismatch_error =
        governance_decision_for_observation_context(&observation, &mismatched_contract)
            .expect_err("governance must use the Contract bound to the observation context");
    assert!(
        mismatch_error
            .to_string()
            .contains("captured observation identity")
    );

    let mut contract_value: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract bytes")).expect("json");
    contract_value["goal"] = serde_json::Value::String("changed after capture".into());
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract_value).expect("contract json"),
    )
    .expect("contract mutation");
    let contract_error = governance_decision_for_observation_context(&observation, &contract)
        .expect_err("stale Contract must not reach governance");
    assert!(contract_error.to_string().contains("Contract identity"));

    fs::write(root.join(".ai/policy.json"), "{\"schemaVersion\":1}\n").expect("policy");
    let error = governance_decision_for_observation_context(&observation, &contract)
        .expect_err("stale context must not reach governance");
    assert!(error.to_string().contains("observation phase"));
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn observation_rejects_symlinked_contract_and_active_governance_file() {
    use std::os::unix::fs::symlink;

    let root = repository("symlink-observation");
    attach(&root).expect("attach");
    scaffold_work_item(&root, "WI-SYMLINK-OBSERVATION", "code").expect("scaffold");

    let contract = root.join(".ai/work-items/active/WI-SYMLINK-OBSERVATION.contract.json");
    let contract_target =
        root.join(".ai/work-items/active/WI-SYMLINK-OBSERVATION.contract.target.json");
    fs::rename(&contract, &contract_target).expect("move contract target");
    symlink(&contract_target, &contract).expect("contract symlink");
    let context = RepositoryExecutionContext::capture(&root).expect("capture");
    let contract_error = context
        .observe_phase_with_contract(ObservationPhase::BeforeGovernance, None, &contract)
        .expect_err("observation must reject a symlinked Contract");
    assert!(
        contract_error.to_string().contains("regular non-symlink"),
        "{contract_error}"
    );
    fs::remove_file(&contract).expect("remove contract symlink");
    fs::rename(contract_target, &contract).expect("restore contract");

    let config = root.join(".ai/cockpit.toml");
    let config_target = root.join(".ai/cockpit.target.toml");
    fs::rename(&config, &config_target).expect("move config target");
    symlink(&config_target, &config).expect("config symlink");
    let context = RepositoryExecutionContext::capture(&root).expect("capture after restore");
    let config_error = context
        .observe_phase(ObservationPhase::BeforeGovernance, None, None)
        .expect_err("observation must reject a symlinked active governance file");
    assert!(
        config_error.to_string().contains("symlink")
            || config_error.to_string().contains("regular non-symlink"),
        "{config_error}"
    );
    fs::remove_file(&config).expect("remove config symlink");
    fs::rename(config_target, config).expect("restore config");
}

#[test]
fn execution_context_captures_one_snapshot_and_memoizes_observation() {
    let root = repository("memoized");
    fs::write(root.join("src.rs"), "fn value() -> u8 { 1 }\n").expect("source");
    attach(&root).expect("attach");
    let context = RepositoryExecutionContext::capture(&root).expect("capture");
    assert_eq!(context.snapshot().git_calls, 4);
    let original_tree = context.snapshot().tree_digest.clone();
    let first = context.observe().expect("observe");
    let second = context.observe().expect("observe again");
    assert!(std::ptr::eq(first, second));
    assert_eq!(first, second);

    fs::write(root.join("src.rs"), "fn value() -> u8 { 2 }\n").expect("change");
    // The request-scoped context remains bound to its original snapshot. A
    // caller that wants current facts must explicitly capture a new one.
    assert!(!context.snapshot().changed_paths.is_empty());
    assert_eq!(context.snapshot().tree_digest, original_tree);
    let fresh = RepositoryExecutionContext::capture(&root).expect("fresh context");
    assert_ne!(context.snapshot().diff_digest, fresh.snapshot().diff_digest);
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn runtime_session_reuses_only_explicit_repository_bindings() {
    let left = repository("session-left");
    let right = repository("session-right");
    attach(&left).expect("left attach");
    attach(&right).expect("right attach");
    let session = RuntimeSession::new();
    let left_first = session.bind(&left).expect("left bind");
    let left_second = session.bind(&left).expect("left bind again");
    let right_bound = session.bind(&right).expect("right bind");
    assert!(std::sync::Arc::ptr_eq(&left_first, &left_second));
    assert_ne!(left_first.repository_id(), right_bound.repository_id());
    assert_eq!(session.active_repositories().expect("active").len(), 2);
    assert!(session.unbind(&left).expect("unbind left"));
    assert_eq!(session.active_repositories().expect("active").len(), 1);
    fs::remove_dir_all(left).expect("cleanup left");
    fs::remove_dir_all(right).expect("cleanup right");
}

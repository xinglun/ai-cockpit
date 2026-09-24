use cockpit_core::Digest;
use cockpit_git::GitRepository;
use cockpit_protocol::{
    COLLABORATION_CAPABILITY, CollaborationDeclaration, ConsumedOutcome, CoordinationEvent,
    CoordinationEventKind, CoordinationIntent, CoordinationRequest, CoordinationRequestState,
    IntegrationResponsibility, OutcomeStage, ProvidedOutcome, RuntimeCapabilityBinding,
    WorktreeRegistration,
};
use cockpit_repository::{
    CoordinationError, CoordinationStore, acknowledge_pause, admit_collaboration_action,
    collaboration_projection, refresh_dependency_state, report_impact, request_safe_pause,
    resume_and_re_evaluate,
};
use std::fs;
use std::path::Path;
use std::process::Command;

fn digest(label: &str) -> Digest {
    Digest::sha256_bytes(label.as_bytes())
}

fn run(root: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .expect("git command")
            .success()
    );
}

fn repository() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("tempdir");
    run(root.path(), &["init", "-q"]);
    run(
        root.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    run(root.path(), &["config", "user.name", "Test"]);
    fs::write(root.path().join("README.md"), "initial\n").expect("write");
    run(root.path(), &["add", "."]);
    run(root.path(), &["commit", "-qm", "initial"]);
    root
}

fn runtime() -> RuntimeCapabilityBinding {
    RuntimeCapabilityBinding {
        schema_version: 1,
        runtime_version: "0.2.113".into(),
        runtime_digest: digest("candidate-runtime"),
        capability: COLLABORATION_CAPABILITY.into(),
    }
}

fn store(root: &Path) -> CoordinationStore {
    CoordinationStore::open(&GitRepository::discover(root).unwrap(), runtime()).unwrap()
}

fn declaration(
    provided: &[(&str, OutcomeStage)],
    consumed: &[(&str, &str, OutcomeStage)],
) -> CollaborationDeclaration {
    CollaborationDeclaration {
        provided_outcomes: provided
            .iter()
            .map(|(outcome_id, stage)| ProvidedOutcome {
                outcome_id: (*outcome_id).into(),
                interface_contract: format!("{outcome_id}-interface"),
                behavior_contract: "stable behavior".into(),
                published_head: "0123456789012345678901234567890123456789".into(),
                stage: *stage,
                evidence_refs: vec!["target/outcome.json".into()],
            })
            .collect(),
        consumed_outcomes: consumed
            .iter()
            .map(|(provider, outcome_id, minimum_stage)| ConsumedOutcome {
                provider_work_item_id: (*provider).into(),
                outcome_id: (*outcome_id).into(),
                minimum_stage: *minimum_stage,
                verification_required: true,
            })
            .collect(),
        resource_claims: Vec::new(),
        integration_responsibility: IntegrationResponsibility {
            responsible_work_item_id: "WI-INTEGRATION".into(),
            target_branch: "main".into(),
            composition_order: Vec::new(),
            rationale: "test".into(),
        },
        composition_verification: Default::default(),
    }
}

fn registration(
    root: &Path,
    work_item_id: &str,
    generation: u64,
    declaration: CollaborationDeclaration,
) -> WorktreeRegistration {
    WorktreeRegistration {
        schema_version: 1,
        repository_id: digest("repository"),
        work_item_id: work_item_id.into(),
        contract_digest: digest(&format!("contract-{work_item_id}")),
        worktree_path: root.to_string_lossy().into_owned(),
        branch: format!("codex/{work_item_id}"),
        head: "0123456789012345678901234567890123456789".into(),
        generation,
        declaration,
        runtime: runtime(),
    }
}

fn impact(work_item_id: &str, generation: u64, id: &str) -> CoordinationEvent {
    CoordinationEvent {
        schema_version: 1,
        event_id: id.into(),
        repository_id: digest("repository"),
        work_item_id: work_item_id.into(),
        generation,
        kind: CoordinationEventKind::Impact,
        source: "provider-outcome-changed".into(),
        evidence_refs: vec!["target/impact.json".into()],
    }
}

fn register_provider_and_consumer(store: &CoordinationStore, root: &Path, stage: OutcomeStage) {
    store
        .register(registration(
            root,
            "WI-PROVIDER",
            1,
            declaration(&[("api", stage)], &[]),
        ))
        .unwrap();
    store
        .register(registration(
            root,
            "WI-CONSUMER",
            1,
            declaration(&[], &[("WI-PROVIDER", "api", stage)]),
        ))
        .unwrap();
}

#[test]
fn composable_head_satisfies_dependency_without_provider_closure() {
    let root = repository();
    let store = store(root.path());
    register_provider_and_consumer(&store, root.path(), OutcomeStage::ComposableHead);
    let admission = admit_collaboration_action(&store, "WI-CONSUMER", 1).unwrap();
    assert!(admission.allowed);
    assert!(admission.blockers.is_empty());
}

#[test]
fn impact_blocks_only_affected_consumers_and_unrelated_work_continues() {
    let root = repository();
    let store = store(root.path());
    register_provider_and_consumer(&store, root.path(), OutcomeStage::ComposableHead);
    store
        .register(registration(
            root.path(),
            "WI-UNRELATED",
            1,
            declaration(&[("other", OutcomeStage::InterfaceStable)], &[]),
        ))
        .unwrap();
    report_impact(&store, impact("WI-PROVIDER", 1, "impact-1")).unwrap();
    let consumer = admit_collaboration_action(&store, "WI-CONSUMER", 1).unwrap();
    let unrelated = admit_collaboration_action(&store, "WI-UNRELATED", 1).unwrap();
    assert!(!consumer.allowed);
    assert!(consumer.affected);
    assert!(unrelated.allowed);
    assert!(!unrelated.affected);
}

#[test]
fn dependency_cycle_is_diagnosed_without_blocking_unrelated_work() {
    let root = repository();
    let store = store(root.path());
    store
        .register(registration(
            root.path(),
            "WI-A",
            1,
            declaration(
                &[("a", OutcomeStage::ComposableHead)],
                &[("WI-B", "b", OutcomeStage::ComposableHead)],
            ),
        ))
        .unwrap();
    store
        .register(registration(
            root.path(),
            "WI-B",
            1,
            declaration(
                &[("b", OutcomeStage::ComposableHead)],
                &[("WI-A", "a", OutcomeStage::ComposableHead)],
            ),
        ))
        .unwrap();
    store
        .register(registration(root.path(), "WI-C", 1, declaration(&[], &[])))
        .unwrap();
    let projection = collaboration_projection(&store).unwrap();
    assert!(
        projection
            .cycles
            .iter()
            .any(|cycle| cycle.contains(&"WI-A".into()))
    );
    assert!(
        !admit_collaboration_action(&store, "WI-A", 1)
            .unwrap()
            .allowed
    );
    assert!(
        admit_collaboration_action(&store, "WI-C", 1)
            .unwrap()
            .allowed
    );
}

#[test]
fn pause_request_is_explicit_and_resume_refreshes_before_admission() {
    let root = repository();
    let store = store(root.path());
    register_provider_and_consumer(&store, root.path(), OutcomeStage::ComposableHead);
    let request = CoordinationRequest {
        schema_version: 1,
        request_id: "pause-1".into(),
        repository_id: digest("repository"),
        target_work_item_id: "WI-CONSUMER".into(),
        target_generation: 1,
        intent: CoordinationIntent::RequestSafePause,
        state: CoordinationRequestState::Requested,
        reason: "provider changed".into(),
    };
    request_safe_pause(&store, request.clone()).unwrap();
    acknowledge_pause(&store, "pause-1", CoordinationRequestState::Acknowledged).unwrap();
    acknowledge_pause(&store, "pause-1", CoordinationRequestState::SafelyPaused).unwrap();
    let admission = resume_and_re_evaluate(&store, "WI-CONSUMER", 1).unwrap();
    assert!(admission.allowed);
    assert_eq!(admission.refreshed_events, 0);
    let projection = refresh_dependency_state(&store, "WI-CONSUMER", 1).unwrap();
    assert_eq!(
        projection.requests[0].state,
        CoordinationRequestState::Resumed
    );
}

#[test]
fn fixed_runtime_is_rejected_before_any_coordination_directory_is_written() {
    let root = repository();
    let old = RuntimeCapabilityBinding {
        schema_version: 1,
        runtime_version: "0.2.105".into(),
        runtime_digest: digest("fixed-runtime"),
        capability: "lifecycle_v1".into(),
    };
    let result = CoordinationStore::open(&GitRepository::discover(root.path()).unwrap(), old);
    assert!(matches!(
        result,
        Err(CoordinationError::Runtime(
            cockpit_protocol::RuntimeCapabilityError::UnsupportedCapability
        )) | Err(CoordinationError::Runtime(
            cockpit_protocol::RuntimeCapabilityError::FixedLifecycleRuntime(_)
        ))
    ));
    assert!(!root.path().join(".git/.ai-cockpit").exists());
}

#[test]
fn unavailable_expired_and_duplicate_pause_requests_stay_distinct() {
    let root = repository();
    let store = store(root.path());
    register_provider_and_consumer(&store, root.path(), OutcomeStage::ComposableHead);
    let request = CoordinationRequest {
        schema_version: 1,
        request_id: "pause-unavailable".into(),
        repository_id: digest("repository"),
        target_work_item_id: "WI-CONSUMER".into(),
        target_generation: 1,
        intent: CoordinationIntent::RequestSafePause,
        state: CoordinationRequestState::Requested,
        reason: "pause at boundary".into(),
    };
    assert_eq!(
        request_safe_pause(&store, request.clone()).unwrap(),
        request
    );
    assert_eq!(
        request_safe_pause(&store, request.clone()).unwrap(),
        request
    );
    acknowledge_pause(
        &store,
        "pause-unavailable",
        CoordinationRequestState::Unavailable,
    )
    .unwrap();

    let expired = CoordinationRequest {
        request_id: "pause-expired".into(),
        ..request
    };
    request_safe_pause(&store, expired).unwrap();
    acknowledge_pause(&store, "pause-expired", CoordinationRequestState::Expired).unwrap();
    let projection = collaboration_projection(&store).unwrap();
    let states = projection
        .requests
        .iter()
        .map(|request| request.state)
        .collect::<Vec<_>>();
    assert!(states.contains(&CoordinationRequestState::Unavailable));
    assert!(states.contains(&CoordinationRequestState::Expired));
}

#[test]
fn old_pause_request_cannot_control_a_new_execution_generation() {
    let root = repository();
    let store = store(root.path());
    register_provider_and_consumer(&store, root.path(), OutcomeStage::ComposableHead);
    let request = CoordinationRequest {
        schema_version: 1,
        request_id: "pause-old-generation".into(),
        repository_id: digest("repository"),
        target_work_item_id: "WI-CONSUMER".into(),
        target_generation: 1,
        intent: CoordinationIntent::RequestSafePause,
        state: CoordinationRequestState::Requested,
        reason: "old run".into(),
    };
    request_safe_pause(&store, request).unwrap();
    store
        .register(registration(
            root.path(),
            "WI-CONSUMER",
            2,
            declaration(&[], &[("WI-PROVIDER", "api", OutcomeStage::ComposableHead)]),
        ))
        .unwrap();
    let result = acknowledge_pause(
        &store,
        "pause-old-generation",
        CoordinationRequestState::Acknowledged,
    );
    assert!(matches!(
        result,
        Err(CoordinationError::StaleGeneration { .. })
    ));
}

use cockpit_core::Digest;
use cockpit_git::GitRepository;
use cockpit_protocol::RuntimeContext;
use cockpit_protocol::{
    COLLABORATION_CAPABILITY, CollaborationDeclaration, CompositionBinding, ConsumedOutcome,
    CoordinationEvent, CoordinationEventKind, CoordinationIntent, CoordinationRequest,
    CoordinationRequestState, IntegrationResponsibility, OutcomeStage, ProvidedOutcome,
    RuntimeCapabilityBinding, WorktreeRegistration,
};
use cockpit_repository::{
    CollaborationAction, CollaborationActionKind, CoordinationError, CoordinationStore,
    WorkItemStartOptions, acknowledge_pause, admit_collaboration_action, attach,
    collaboration_outcome_projection, collaboration_projection, recover_impact,
    refresh_dependency_state, report_impact, repository_id, request_safe_pause,
    resume_and_re_evaluate, run_admitted_composition, start_work_item_with_options,
};
use cockpit_verification::{
    CompositionCommand, CompositionIdentity, CompositionInput, CompositionPrecondition,
    composition_commands_digest,
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
    fs::create_dir_all(root.path().join("target")).expect("target directory");
    fs::write(root.path().join("target/outcome.json"), "{}\n").expect("outcome evidence");
    fs::write(root.path().join("target/impact.json"), "{}\n").expect("impact evidence");
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

fn contract_digest(root: &Path, work_item_id: &str) -> Digest {
    if !root.join(".ai/cockpit.toml").exists() {
        attach(root).expect("attach repository");
    }
    let path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    if !path.exists() {
        start_work_item_with_options(
            root,
            work_item_id,
            "collaboration test",
            "bind dependency admission to observed facts",
            &[".ai/**".into(), "README.md".into()],
            &WorkItemStartOptions {
                authority: "authorized".into(),
                out_of_scope: vec!["target/**".into()],
                acceptance_criteria: vec!["admission remains bounded".into()],
                ..WorkItemStartOptions::default()
            },
        )
        .expect("start test Work Item");
    }
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(path).expect("Contract bytes")).expect("Contract JSON");
    cockpit_protocol::digest_json(&value).expect("Contract digest")
}

fn declaration(
    root: &Path,
    provided: &[(&str, OutcomeStage)],
    consumed: &[(&str, &str, OutcomeStage)],
) -> CollaborationDeclaration {
    let head = GitRepository::discover(root)
        .expect("discover")
        .topology()
        .expect("topology")
        .head
        .expect("head");
    CollaborationDeclaration {
        provided_outcomes: provided
            .iter()
            .map(|(outcome_id, stage)| ProvidedOutcome {
                outcome_id: (*outcome_id).into(),
                interface_contract: format!("{outcome_id}-interface"),
                behavior_contract: "stable behavior".into(),
                published_head: head.clone(),
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
    let contract_digest = contract_digest(root, work_item_id);
    let topology = GitRepository::discover(root)
        .expect("discover")
        .topology()
        .expect("topology");
    WorktreeRegistration {
        schema_version: 1,
        repository_id: repository_id(root),
        work_item_id: work_item_id.into(),
        contract_digest,
        worktree_path: topology.repository_root.to_string_lossy().into_owned(),
        branch: topology.branch.expect("branch"),
        head: topology.head.expect("head"),
        generation,
        declaration,
        runtime: runtime(),
    }
}

fn impact(root: &Path, work_item_id: &str, generation: u64, id: &str) -> CoordinationEvent {
    CoordinationEvent {
        schema_version: 1,
        event_id: id.into(),
        repository_id: repository_id(root),
        work_item_id: work_item_id.into(),
        generation,
        kind: CoordinationEventKind::Impact,
        source: "provider-outcome-changed".into(),
        evidence_digests: Default::default(),
        outcome_ids: Vec::new(),
        evidence_refs: vec!["target/impact.json".into()],
    }
}

fn composition_action(work_item_id: &str) -> CollaborationAction {
    CollaborationAction {
        kind: CollaborationActionKind::Composition,
        consumer_work_item_id: work_item_id.into(),
        outcome_ids: Vec::new(),
    }
}

fn composition_input(root: &Path, marker: &Path) -> CompositionInput {
    let topology = GitRepository::discover(root)
        .expect("discover")
        .topology()
        .expect("topology");
    let head = topology.head.expect("head");
    let identity = CompositionIdentity {
        source_digest: digest("source"),
        dependency_digest: digest("dependency"),
        interface_digest: digest("interface"),
        configuration_digest: digest("configuration"),
        toolchain_digest: digest("toolchain"),
        lockfile_digest: digest("lockfile"),
        generated_input_digest: digest("generated"),
        environment_digest: digest("environment"),
        verifier_digest: digest("verifier"),
        command_digest: digest("command"),
    };
    CompositionInput {
        repository_root: root.to_path_buf(),
        state_dir: root.join("target/composition-state"),
        binding: CompositionBinding {
            schema_version: 1,
            repository_id: repository_id(root),
            binding_id: "pause-test".into(),
            target_branch: topology.branch.expect("branch"),
            target_sha: head.clone(),
            participant_work_items: vec!["WI-CONSUMER".into()],
            participant_heads: vec![head],
            contract_digests: vec![contract_digest(root, "WI-CONSUMER")],
            verifier: runtime(),
        },
        identity,
        commands: vec![CompositionCommand {
            node_id: "marker".into(),
            program: "sh".into(),
            args: vec!["-c".into(), format!("touch {}", marker.display())],
        }],
        preconditions: vec![CompositionPrecondition::satisfied("identity-bound")],
        timeout_seconds: 1,
    }
}

fn runtime_context() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "0.2.113".into(),
        protocol_version: 1,
        runtime_digest: digest("candidate-runtime"),
    }
}

#[test]
fn shared_outcome_projects_composition_cleanup_and_actual_reuse() {
    let root = repository();
    let store = store(root.path());
    store
        .register(registration(
            root.path(),
            "WI-CONSUMER",
            1,
            declaration(root.path(), &[], &[]),
        ))
        .expect("register consumer");
    let marker = root.path().join("composition-marker");
    let mut input = composition_input(root.path(), &marker);
    input.identity.command_digest = composition_commands_digest(&input.commands);

    let first = run_admitted_composition(&store, "WI-CONSUMER", 1, input.clone())
        .expect("first composition");
    assert!(first.passed);
    let first_projection =
        collaboration_outcome_projection(root.path(), "WI-CONSUMER", &runtime_context());
    assert_eq!(first_projection.composition_state, "passed");
    assert_eq!(first_projection.target_merge_state, "passed");
    assert_eq!(first_projection.cleanup_state, "cleaned");
    assert!(first_projection.reusable_checks.is_empty());
    assert!(!first_projection.human_decision_required);

    let second =
        run_admitted_composition(&store, "WI-CONSUMER", 1, input).expect("second composition");
    assert!(second.passed);
    assert_eq!(second.processes_spawned, 0);
    let second_projection =
        collaboration_outcome_projection(root.path(), "WI-CONSUMER", &runtime_context());
    assert_eq!(second_projection.reusable_checks, vec!["marker"]);
    assert!(!second_projection.human_decision_required);
}

fn register_provider_and_consumer(store: &CoordinationStore, root: &Path, stage: OutcomeStage) {
    store
        .register(registration(
            root,
            "WI-PROVIDER",
            1,
            declaration(root, &[("api", stage)], &[]),
        ))
        .unwrap();
    store
        .register(registration(
            root,
            "WI-CONSUMER",
            1,
            declaration(root, &[], &[("WI-PROVIDER", "api", stage)]),
        ))
        .unwrap();
}

#[test]
fn composable_head_satisfies_dependency_without_provider_closure() {
    let root = repository();
    let store = store(root.path());
    register_provider_and_consumer(&store, root.path(), OutcomeStage::ComposableHead);
    let admission =
        admit_collaboration_action(&store, "WI-CONSUMER", 1, composition_action("WI-CONSUMER"))
            .unwrap();
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
            declaration(
                root.path(),
                &[("other", OutcomeStage::InterfaceStable)],
                &[],
            ),
        ))
        .unwrap();
    store
        .publish_event(CoordinationEvent {
            kind: CoordinationEventKind::OutcomePublished,
            event_id: "published-1".into(),
            repository_id: repository_id(root.path()),
            work_item_id: "WI-PROVIDER".into(),
            generation: 1,
            source: "provider-outcome".into(),
            ..impact(root.path(), "WI-PROVIDER", 1, "published-template")
        })
        .unwrap();
    assert!(
        admit_collaboration_action(&store, "WI-CONSUMER", 1, composition_action("WI-CONSUMER"),)
            .unwrap()
            .allowed
    );
    report_impact(&store, impact(root.path(), "WI-PROVIDER", 1, "impact-1")).unwrap();
    let consumer =
        admit_collaboration_action(&store, "WI-CONSUMER", 1, composition_action("WI-CONSUMER"))
            .unwrap();
    let unrelated = admit_collaboration_action(
        &store,
        "WI-UNRELATED",
        1,
        composition_action("WI-UNRELATED"),
    )
    .unwrap();
    assert!(!consumer.allowed);
    assert!(consumer.affected);
    assert!(unrelated.allowed);
    assert!(!unrelated.affected);
}

#[test]
fn recovered_impact_is_consumed_for_the_matching_consumer_generation() {
    let root = repository();
    let store = store(root.path());
    register_provider_and_consumer(&store, root.path(), OutcomeStage::ComposableHead);
    report_impact(
        &store,
        impact(root.path(), "WI-PROVIDER", 1, "impact-recovery"),
    )
    .unwrap();
    assert!(
        !admit_collaboration_action(&store, "WI-CONSUMER", 1, composition_action("WI-CONSUMER"),)
            .unwrap()
            .allowed
    );

    recover_impact(&store, "impact-recovery", "WI-CONSUMER", 1).unwrap();
    let admission =
        admit_collaboration_action(&store, "WI-CONSUMER", 1, composition_action("WI-CONSUMER"))
            .unwrap();
    assert!(admission.allowed);
    assert!(!admission.affected);
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
                root.path(),
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
                root.path(),
                &[("b", OutcomeStage::ComposableHead)],
                &[("WI-A", "a", OutcomeStage::ComposableHead)],
            ),
        ))
        .unwrap();
    store
        .register(registration(
            root.path(),
            "WI-C",
            1,
            declaration(root.path(), &[], &[]),
        ))
        .unwrap();
    let projection = collaboration_projection(&store).unwrap();
    assert!(
        projection
            .cycles
            .iter()
            .any(|cycle| cycle.contains(&"WI-A".into()))
    );
    assert!(
        !admit_collaboration_action(&store, "WI-A", 1, composition_action("WI-A"))
            .unwrap()
            .allowed
    );
    assert!(
        admit_collaboration_action(&store, "WI-C", 1, composition_action("WI-C"))
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
        repository_id: repository_id(root.path()),
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
fn safely_paused_composition_is_rejected_before_spawn_and_unrelated_work_continues() {
    let root = repository();
    let store = store(root.path());
    register_provider_and_consumer(&store, root.path(), OutcomeStage::ComposableHead);
    store
        .register(registration(
            root.path(),
            "WI-UNRELATED",
            1,
            declaration(root.path(), &[], &[]),
        ))
        .unwrap();
    let request = CoordinationRequest {
        schema_version: 1,
        request_id: "pause-before-spawn".into(),
        repository_id: repository_id(root.path()),
        target_work_item_id: "WI-CONSUMER".into(),
        target_generation: 1,
        intent: CoordinationIntent::RequestSafePause,
        state: CoordinationRequestState::Requested,
        reason: "pause before composition boundary".into(),
    };
    request_safe_pause(&store, request).unwrap();
    acknowledge_pause(
        &store,
        "pause-before-spawn",
        CoordinationRequestState::Acknowledged,
    )
    .unwrap();
    acknowledge_pause(
        &store,
        "pause-before-spawn",
        CoordinationRequestState::SafelyPaused,
    )
    .unwrap();

    let marker = root.path().join("target/paused-marker");
    let blocked = run_admitted_composition(
        &store,
        "WI-CONSUMER",
        1,
        composition_input(root.path(), &marker),
    )
    .expect_err("SafelyPaused must stop composition before spawn");
    assert!(blocked.to_string().contains("coordination_safely_paused"));
    assert!(!marker.exists());
    assert!(
        admit_collaboration_action(
            &store,
            "WI-UNRELATED",
            1,
            composition_action("WI-UNRELATED")
        )
        .unwrap()
        .allowed
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
        repository_id: repository_id(root.path()),
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
        repository_id: repository_id(root.path()),
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
            declaration(
                root.path(),
                &[],
                &[("WI-PROVIDER", "api", OutcomeStage::ComposableHead)],
            ),
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

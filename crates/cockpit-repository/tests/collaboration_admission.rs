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
    checkpoint_work_item, collaboration_outcome_projection, collaboration_projection,
    preflight_work_item, publish_outcome, record_verification, recover_impact,
    refresh_dependency_state, report_impact, repository_id, request_safe_pause,
    resume_and_re_evaluate, run_admitted_composition, start_work_item_with_options,
};
use cockpit_verification::{
    CompositionCommand, CompositionIdentity, CompositionInput, CompositionPrecondition,
    VerificationCommand, VerificationReusePolicy, composition_commands_digest, execute_bounded,
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
    run(root.path(), &["branch", "-M", "main"]);
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

fn declare_required_checks(root: &Path, work_item_id: &str, checks: &[String]) {
    contract_digest(root, work_item_id);
    let path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("Contract bytes")).expect("Contract JSON");
    contract["verification"] = serde_json::Value::Array(
        checks
            .iter()
            .map(|check| serde_json::json!({"check": check, "required": true}))
            .collect(),
    );
    fs::write(
        path,
        serde_json::to_vec_pretty(&contract).expect("serialize typed Contract checks"),
    )
    .expect("write typed Contract checks");
}

fn declare_required_check_coverage(
    root: &Path,
    work_item_id: &str,
    check: &str,
    scenarios: &[&str],
    constraints: &[&str],
) {
    contract_digest(root, work_item_id);
    let path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("Contract bytes")).expect("Contract JSON");
    contract["verification"] = serde_json::json!([{
        "check": check,
        "required": true,
        "coversScenarios": scenarios,
        "coversConstraints": constraints,
    }]);
    fs::write(
        path,
        serde_json::to_vec_pretty(&contract).expect("serialize coverage-bound check"),
    )
    .expect("write coverage-bound Contract check");
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
                verification_required: false,
            })
            .collect(),
        resource_claims: Vec::new(),
        integration_responsibility: IntegrationResponsibility {
            responsible_work_item_id: "WI-CONSUMER".into(),
            target_branch: "main".into(),
            composition_order: Vec::new(),
            rationale: "test".into(),
        },
        composition_verification: Default::default(),
    }
}

fn record_typed_verification(root: &Path, work_item_id: &str) {
    let evidence_path = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.verification.json"));
    if evidence_path.exists() {
        fs::remove_file(&evidence_path).expect("remove invalid placeholder evidence");
    }
    let contract = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    preflight_work_item(root, &contract).expect("preflight for verification evidence");
    checkpoint_work_item(root, work_item_id).expect("checkpoint for verification evidence");
    let receipt = execute_bounded(
        vec![VerificationCommand::new(
            "collaboration-evidence",
            "sh",
            vec!["-c".into(), "true".into()],
            VerificationReusePolicy::NeverReuse,
        )],
        1,
    )
    .expect("execute evidence check");
    let receipt = serde_json::to_value(receipt).expect("serialize typed receipt");
    record_verification(
        root,
        work_item_id,
        &receipt,
        "0.2.113",
        &digest("candidate-runtime"),
    )
    .expect("record typed verification evidence");
}

fn publish_verification_outcome(
    store: &CoordinationStore,
    root: &Path,
    work_item_id: &str,
    generation: u64,
    outcome_id: &str,
) {
    store
        .publish_event(CoordinationEvent {
            schema_version: 1,
            event_id: format!("verification-{work_item_id}-{generation}-{outcome_id}"),
            repository_id: repository_id(root),
            work_item_id: work_item_id.into(),
            generation,
            kind: CoordinationEventKind::OutcomePublished,
            source: "verified-outcome-publication".into(),
            evidence_refs: vec![format!(".ai/evidence/{work_item_id}.verification.json")],
            evidence_digests: Default::default(),
            outcome_ids: vec![outcome_id.into()],
        })
        .expect("publish generation-bound verification outcome");
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
        evidence_refs: vec!["target/impact.json".into()],
        evidence_digests: Default::default(),
        outcome_ids: Vec::new(),
    }
}

fn composition_action(work_item_id: &str) -> CollaborationAction {
    CollaborationAction {
        kind: CollaborationActionKind::Composition,
        consumer_work_item_id: work_item_id.into(),
        outcomes: Vec::new(),
    }
}

fn outcome_action(work_item_id: &str, outcomes: &[(&str, &str)]) -> CollaborationAction {
    CollaborationAction {
        kind: CollaborationActionKind::Composition,
        consumer_work_item_id: work_item_id.into(),
        outcomes: outcomes
            .iter()
            .map(
                |(provider_work_item_id, outcome_id)| cockpit_protocol::ProviderOutcomeKey {
                    provider_work_item_id: (*provider_work_item_id).into(),
                    outcome_id: (*outcome_id).into(),
                },
            )
            .collect(),
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
            program: "touch".into(),
            args: vec![marker.to_string_lossy().into_owned()],
            depends_on: Vec::new(),
            environment: Default::default(),
            input_paths: vec!["README.md".into()],
            covered_scenarios: Vec::new(),
            covered_constraints: Vec::new(),
        }],
        reusable_node_ids: Vec::new(),
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
    let marker = root.path().join("composition-marker");
    declare_required_checks(root.path(), "WI-CONSUMER", &["true".into()]);
    let mut consumer_declaration = declaration(root.path(), &[], &[]);
    consumer_declaration.composition_verification.reusable_nodes = vec!["marker".into()];
    store
        .register(registration(
            root.path(),
            "WI-CONSUMER",
            1,
            consumer_declaration,
        ))
        .expect("register consumer");
    let mut input = composition_input(root.path(), &marker);
    input.commands[0].program = "true".into();
    input.commands[0].args.clear();
    input.identity.command_digest = composition_commands_digest(&input.commands);

    let first = run_admitted_composition(&store, "WI-CONSUMER", 1, input.clone())
        .expect("first composition");
    assert!(first.passed);
    let first_projection =
        collaboration_outcome_projection(root.path(), "WI-CONSUMER", &runtime_context());
    assert_eq!(first_projection.composition_state, "passed");
    assert_eq!(first_projection.target_merge_state, "merged");
    assert_eq!(first_projection.composition_applicability, "current");
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

#[test]
fn admitted_composition_rejects_invalid_command_dependencies_before_launch() {
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
    let marker = root.path().join("invalid-dependency-marker");
    let mut input = composition_input(root.path(), &marker);
    input.commands[0].depends_on = vec!["missing-upstream".into()];
    input.identity.command_digest = composition_commands_digest(&input.commands);

    let result = run_admitted_composition(&store, "WI-CONSUMER", 1, input);

    assert!(
        result.is_err(),
        "invalid dependency graph must be rejected by admission"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("composition_dependency_order_invalid")
    );
    assert!(!marker.exists(), "invalid graph must not launch the check");
}

#[test]
fn missing_one_of_two_required_scenarios_blocks_composition() {
    let root = repository();
    let store = store(root.path());
    let mut work = declaration(root.path(), &[], &[]);
    work.composition_verification.required_scenarios =
        vec!["api-contract".into(), "docs-contract".into()];
    store
        .register(registration(root.path(), "WI-CONSUMER", 1, work))
        .expect("register consumer");

    let mut input = composition_input(root.path(), &root.path().join("unused-marker"));
    input.commands[0].covered_scenarios = vec!["api-contract".into()];
    input.identity.command_digest = composition_commands_digest(&input.commands);
    let result = run_admitted_composition(&store, "WI-CONSUMER", 1, input);

    assert!(
        result.is_err(),
        "one missing required scenario must fail closed"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("required_scenario_uncovered:docs-contract")
    );
}

#[test]
fn caller_labels_cannot_forge_required_contract_checks() {
    let root = repository();
    let store = store(root.path());
    let marker = root.path().join("forged-check-marker");
    declare_required_checks(
        root.path(),
        "WI-CONSUMER",
        &[format!("touch {}", marker.display()), "true".into()],
    );
    let mut work = declaration(root.path(), &[], &[]);
    work.composition_verification.required_scenarios = vec!["api-contract".into()];
    work.composition_verification.compatibility_constraints = vec!["stable-api".into()];
    store
        .register(registration(root.path(), "WI-CONSUMER", 1, work))
        .expect("register consumer");

    let mut input = composition_input(root.path(), &marker);
    input.commands[0].program = "sh".into();
    input.commands[0].args = vec!["-c".into(), format!("touch {}", marker.display())];
    input.commands[0].covered_scenarios = vec!["api-contract".into()];
    input.commands[0].covered_constraints = vec!["stable-api".into()];
    input.identity.command_digest = composition_commands_digest(&input.commands);

    let result = run_admitted_composition(&store, "WI-CONSUMER", 1, input);

    assert!(
        result.is_err(),
        "caller-supplied labels and a shell command must not replace registered Contract checks"
    );
    assert!(
        !marker.exists(),
        "forged check must be rejected before spawn"
    );
}

#[test]
fn caller_coverage_labels_cannot_attach_scenarios_to_an_unrelated_required_check() {
    let root = repository();
    let store = store(root.path());
    let marker = root.path().join("forged-coverage-marker");
    declare_required_checks(
        root.path(),
        "WI-CONSUMER",
        &[format!("touch {}", marker.display()), "true".into()],
    );
    let mut work = declaration(root.path(), &[], &[]);
    work.composition_verification.required_scenarios = vec!["api-compat".into()];
    work.composition_verification.compatibility_constraints = vec!["stable-api".into()];
    store
        .register(registration(root.path(), "WI-CONSUMER", 1, work))
        .expect("register consumer");

    let mut input = composition_input(root.path(), &marker);
    input.commands[0].program = "touch".into();
    input.commands[0].args = vec![marker.to_string_lossy().into_owned()];
    let mut unrelated_required_check = input.commands[0].clone();
    unrelated_required_check.node_id = "unrelated-required-check".into();
    unrelated_required_check.program = "true".into();
    unrelated_required_check.args.clear();
    unrelated_required_check.covered_scenarios = vec!["api-compat".into()];
    unrelated_required_check.covered_constraints = vec!["stable-api".into()];
    input.commands.push(unrelated_required_check);
    input.identity.command_digest = composition_commands_digest(&input.commands);

    let result = run_admitted_composition(&store, "WI-CONSUMER", 1, input);

    assert!(
        result.is_err(),
        "a matching required command must not inherit caller-invented scenario or constraint coverage"
    );
    assert!(
        !marker.exists(),
        "unbound coverage labels must block before the required command starts"
    );
}

#[test]
fn contract_bound_check_scenario_and_constraint_coverage_is_admitted() {
    let root = repository();
    let store = store(root.path());
    declare_required_check_coverage(
        root.path(),
        "WI-CONSUMER",
        "true",
        &["api-compat"],
        &["stable-api"],
    );
    let mut work = declaration(root.path(), &[], &[]);
    work.composition_verification.required_scenarios = vec!["api-compat".into()];
    work.composition_verification.compatibility_constraints = vec!["stable-api".into()];
    store
        .register(registration(root.path(), "WI-CONSUMER", 1, work))
        .expect("register consumer");

    let mut input = composition_input(root.path(), &root.path().join("unused-marker"));
    input.commands[0].program = "true".into();
    input.commands[0].args.clear();
    input.commands[0].covered_scenarios = vec!["api-compat".into()];
    input.commands[0].covered_constraints = vec!["stable-api".into()];
    input.identity.command_digest = composition_commands_digest(&input.commands);

    let result = run_admitted_composition(&store, "WI-CONSUMER", 1, input);

    assert!(
        result.is_ok(),
        "a digest-bound Contract check may cover exactly its declared scenarios and constraints: {result:?}"
    );
}

#[test]
fn composition_rejects_a_missing_required_contract_check_before_spawn() {
    let root = repository();
    let store = store(root.path());
    let marker = root.path().join("partial-check-marker");
    declare_required_checks(
        root.path(),
        "WI-CONSUMER",
        &[format!("touch {}", marker.display()), "true".into()],
    );
    store
        .register(registration(
            root.path(),
            "WI-CONSUMER",
            1,
            declaration(root.path(), &[], &[]),
        ))
        .expect("register consumer");

    let mut input = composition_input(root.path(), &marker);
    input.identity.command_digest = composition_commands_digest(&input.commands);
    let result = run_admitted_composition(&store, "WI-CONSUMER", 1, input);

    assert!(
        result.is_err(),
        "one exact command cannot cover two required Contract checks"
    );
    assert!(
        !marker.exists(),
        "partial check set must be rejected before spawn"
    );
}

#[test]
fn admitted_composition_rejects_a_different_clone_even_with_matching_repository_id() {
    let root = repository();
    let clone = tempfile::tempdir().expect("separate clone directory");
    let marker = clone.path().join("must-not-run");
    declare_required_checks(
        root.path(),
        "WI-CONSUMER",
        &[format!("touch {}", marker.display())],
    );
    let declaration = declaration(root.path(), &[], &[]);
    let store = store(root.path());
    store
        .register(registration(
            root.path(),
            "WI-CONSUMER",
            1,
            declaration.clone(),
        ))
        .expect("register integration owner");
    run(
        root.path(),
        &[
            "clone",
            "-q",
            "--no-hardlinks",
            root.path().to_str().unwrap(),
            clone.path().to_str().unwrap(),
        ],
    );
    // `git clone` creates the directory; place the copied identity after it.
    fs::create_dir_all(clone.path().join(".ai")).expect("clone identity directory");
    fs::copy(
        root.path().join(".ai/cockpit.toml"),
        clone.path().join(".ai/cockpit.toml"),
    )
    .expect("copy the same repository identity to the separate clone");
    let stale_target = GitRepository::discover(clone.path())
        .expect("discover clone")
        .topology()
        .expect("clone topology")
        .head
        .expect("clone head");

    fs::write(root.path().join("advanced-target.txt"), "advanced\n")
        .expect("advance integration target");
    run(root.path(), &["add", "advanced-target.txt"]);
    run(
        root.path(),
        &["commit", "-qm", "advance integration target"],
    );
    store
        .register(registration(root.path(), "WI-CONSUMER", 2, declaration))
        .expect("refresh registered target head");
    run(
        root.path(),
        &[
            "-C",
            clone.path().to_str().unwrap(),
            "fetch",
            "-q",
            root.path().to_str().unwrap(),
            "main",
        ],
    );

    let mut input = composition_input(root.path(), &marker);
    input.repository_root = clone.path().to_path_buf();
    input.binding.target_sha = stale_target;
    input.commands[0].program = "touch".into();
    input.commands[0].args = vec![marker.to_string_lossy().into_owned()];
    input.identity.command_digest = composition_commands_digest(&input.commands);

    let result = run_admitted_composition(&store, "WI-CONSUMER", 2, input);

    assert!(
        matches!(&result, Err(error) if error.to_string().contains("composition_repository_common_directory_mismatch")),
        "composition must bind the execution repository to the registered coordination store: {result:?}"
    );
    assert!(
        !marker.exists(),
        "a separate clone must be rejected before a composition command is spawned"
    );
}

#[test]
fn non_integration_owner_cannot_start_composition() {
    let root = repository();
    let store = store(root.path());
    let marker = root.path().join("non-owner-marker");
    declare_required_checks(
        root.path(),
        "WI-CONSUMER",
        &[format!("touch {}", marker.display())],
    );
    let mut owner_declaration = declaration(root.path(), &[], &[]);
    owner_declaration
        .integration_responsibility
        .responsible_work_item_id = "WI-INTEGRATION".into();
    store
        .register(registration(
            root.path(),
            "WI-INTEGRATION",
            1,
            owner_declaration,
        ))
        .expect("register integration owner");
    let mut consumer_declaration = declaration(root.path(), &[], &[]);
    consumer_declaration
        .integration_responsibility
        .responsible_work_item_id = "WI-INTEGRATION".into();
    store
        .register(registration(
            root.path(),
            "WI-CONSUMER",
            1,
            consumer_declaration,
        ))
        .expect("register non-owner consumer");

    let mut input = composition_input(root.path(), &marker);
    input.identity.command_digest = composition_commands_digest(&input.commands);
    let result = run_admitted_composition(&store, "WI-CONSUMER", 1, input);

    assert!(
        result.is_err(),
        "only the declared integration owner may compose"
    );
    assert!(
        !marker.exists(),
        "non-owner composition must be rejected before spawn"
    );
}

#[test]
fn composition_rejects_duplicate_extra_and_unresolvable_check_identities() {
    let duplicate_root = repository();
    let duplicate_store = store(duplicate_root.path());
    let duplicate_marker = duplicate_root.path().join("duplicate-check-marker");
    declare_required_checks(
        duplicate_root.path(),
        "WI-CONSUMER",
        &[
            format!("touch {}", duplicate_marker.display()),
            "true".into(),
        ],
    );
    duplicate_store
        .register(registration(
            duplicate_root.path(),
            "WI-CONSUMER",
            1,
            declaration(duplicate_root.path(), &[], &[]),
        ))
        .expect("register duplicate-check consumer");
    let mut duplicate_input = composition_input(duplicate_root.path(), &duplicate_marker);
    let mut duplicate = duplicate_input.commands[0].clone();
    duplicate.node_id = "duplicate-marker".into();
    duplicate_input.commands.push(duplicate);
    duplicate_input.identity.command_digest =
        composition_commands_digest(&duplicate_input.commands);
    let duplicate_result =
        run_admitted_composition(&duplicate_store, "WI-CONSUMER", 1, duplicate_input);
    assert!(
        duplicate_result.is_err(),
        "duplicate command identities must block"
    );
    assert!(
        !duplicate_marker.exists(),
        "duplicate command set must not spawn"
    );

    let extra_root = repository();
    let extra_store = store(extra_root.path());
    let extra_marker = extra_root.path().join("extra-check-marker");
    declare_required_checks(
        extra_root.path(),
        "WI-CONSUMER",
        &[format!("touch {}", extra_marker.display())],
    );
    extra_store
        .register(registration(
            extra_root.path(),
            "WI-CONSUMER",
            1,
            declaration(extra_root.path(), &[], &[]),
        ))
        .expect("register extra-check consumer");
    let mut extra_input = composition_input(extra_root.path(), &extra_marker);
    extra_input.commands.push(CompositionCommand {
        node_id: "undeclared-extra".into(),
        program: "true".into(),
        args: Vec::new(),
        depends_on: Vec::new(),
        environment: Default::default(),
        input_paths: Vec::new(),
        covered_scenarios: Vec::new(),
        covered_constraints: Vec::new(),
    });
    extra_input.identity.command_digest = composition_commands_digest(&extra_input.commands);
    let extra_result = run_admitted_composition(&extra_store, "WI-CONSUMER", 1, extra_input);
    assert!(
        extra_result.is_err(),
        "extra commands must not exceed required checks"
    );
    assert!(!extra_marker.exists(), "extra command set must not spawn");

    let ambiguous_root = repository();
    let ambiguous_store = store(ambiguous_root.path());
    let ambiguous_marker = ambiguous_root.path().join("ambiguous-check-marker");
    declare_required_checks(
        ambiguous_root.path(),
        "WI-CONSUMER",
        &[format!("sh -c \"touch {}\"", ambiguous_marker.display())],
    );
    ambiguous_store
        .register(registration(
            ambiguous_root.path(),
            "WI-CONSUMER",
            1,
            declaration(ambiguous_root.path(), &[], &[]),
        ))
        .expect("register ambiguous-check consumer");
    let mut ambiguous_input = composition_input(ambiguous_root.path(), &ambiguous_marker);
    ambiguous_input.commands[0].program = "sh".into();
    ambiguous_input.commands[0].args =
        vec!["-c".into(), format!("touch {}", ambiguous_marker.display())];
    ambiguous_input.identity.command_digest =
        composition_commands_digest(&ambiguous_input.commands);
    let ambiguous_result =
        run_admitted_composition(&ambiguous_store, "WI-CONSUMER", 1, ambiguous_input);
    assert!(
        ambiguous_result.is_err(),
        "quoted Contract checks are ambiguous"
    );
    assert!(
        !ambiguous_marker.exists(),
        "unresolvable check must not spawn"
    );
}

#[test]
fn composition_requires_the_dependency_closure_in_declared_order() {
    let root = repository();
    let store = store(root.path());
    let provider = registration(
        root.path(),
        "WI-PROVIDER",
        1,
        declaration(root.path(), &[("api", OutcomeStage::ComposableHead)], &[]),
    );
    let mut consumer_declaration = declaration(
        root.path(),
        &[],
        &[("WI-PROVIDER", "api", OutcomeStage::ComposableHead)],
    );
    consumer_declaration
        .integration_responsibility
        .composition_order = vec!["WI-PROVIDER".into(), "WI-CONSUMER".into()];
    let consumer = registration(root.path(), "WI-CONSUMER", 1, consumer_declaration);
    store.register(provider.clone()).expect("register provider");
    store.register(consumer.clone()).expect("register consumer");

    let mut input = composition_input(root.path(), &root.path().join("unused-marker"));
    input.identity.command_digest = composition_commands_digest(&input.commands);
    let missing = run_admitted_composition(&store, "WI-CONSUMER", 1, input.clone())
        .expect_err("provider dependency must be represented");
    assert!(
        missing
            .to_string()
            .contains("composition_dependency_closure_incomplete")
    );

    input.binding.participant_work_items = vec!["WI-CONSUMER".into(), "WI-PROVIDER".into()];
    input.binding.participant_heads = vec![consumer.head, provider.head];
    input.binding.contract_digests = vec![consumer.contract_digest, provider.contract_digest];
    input.identity.command_digest = composition_commands_digest(&input.commands);
    let wrong_order = run_admitted_composition(&store, "WI-CONSUMER", 1, input)
        .expect_err("declared composition order must be honored");
    assert!(
        wrong_order
            .to_string()
            .contains("composition_order_mismatch")
    );
}

#[test]
fn feature_worktree_can_compose_against_the_declared_main_target() {
    let root = repository();
    let main_head = GitRepository::discover(root.path())
        .unwrap()
        .topology()
        .unwrap()
        .head
        .unwrap();
    run(root.path(), &["checkout", "-qb", "feature/consumer"]);
    fs::write(root.path().join("feature.txt"), "feature\n").expect("feature file");
    run(root.path(), &["add", "."]);
    run(root.path(), &["commit", "-qm", "feature"]);
    let feature_head = GitRepository::discover(root.path())
        .unwrap()
        .topology()
        .unwrap()
        .head
        .unwrap();
    let marker = root.path().join("feature-composition-marker");
    declare_required_checks(
        root.path(),
        "WI-CONSUMER",
        &[format!("touch {}", marker.display())],
    );
    let store = store(root.path());
    store
        .register(registration(
            root.path(),
            "WI-CONSUMER",
            1,
            declaration(root.path(), &[], &[]),
        ))
        .expect("register feature worktree");

    let mut input = composition_input(root.path(), &marker);
    input.binding.target_branch = "main".into();
    input.binding.target_sha = main_head;
    input.binding.participant_heads = vec![feature_head];
    input.identity.command_digest = composition_commands_digest(&input.commands);
    let attempt = run_admitted_composition(&store, "WI-CONSUMER", 1, input)
        .expect("feature worktree may target main");

    assert!(attempt.passed);
    assert_eq!(attempt.binding.target_branch, "main");
    let projection =
        collaboration_outcome_projection(root.path(), "WI-CONSUMER", &runtime_context());
    assert_eq!(projection.composition_state, "passed");
    assert_eq!(projection.target_merge_state, "not_merged");
    assert_eq!(projection.composition_applicability, "current");

    run(root.path(), &["checkout", "-q", "main"]);
    fs::write(root.path().join("main-advance.txt"), "advanced\n").expect("advance main");
    run(root.path(), &["add", "main-advance.txt"]);
    run(root.path(), &["commit", "-qm", "advance main"]);
    run(root.path(), &["checkout", "-q", "feature/consumer"]);
    let stale_projection =
        collaboration_outcome_projection(root.path(), "WI-CONSUMER", &runtime_context());
    assert_eq!(stale_projection.composition_state, "passed");
    assert_eq!(stale_projection.composition_applicability, "stale");
    assert_eq!(stale_projection.target_merge_state, "not_merged");
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
            outcome_ids: Vec::new(),
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
fn report_impact_rejects_outcome_publication_events() {
    let root = repository();
    let store = store(root.path());
    store
        .register(registration(
            root.path(),
            "WI-PROVIDER",
            1,
            declaration(root.path(), &[("api", OutcomeStage::ComposableHead)], &[]),
        ))
        .expect("register provider");

    let mut event = impact(root.path(), "WI-PROVIDER", 1, "misrouted-publication");
    event.kind = CoordinationEventKind::OutcomePublished;
    event.evidence_refs = vec!["target/outcome.json".into()];
    event.outcome_ids = vec!["api".into()];
    let result = report_impact(&store, event);

    assert!(
        matches!(&result, Err(CoordinationError::RecoveryRequired(message)) if message.contains("publish_outcome")),
        "OutcomePublished must go through the identity- and evidence-validating publisher: {result:?}"
    );
}

#[test]
fn verification_dependency_rejects_empty_json_then_accepts_a_bound_runtime_receipt() {
    let root = repository();
    let store = store(root.path());
    let mut provider = declaration(root.path(), &[("api", OutcomeStage::ComposableHead)], &[]);
    provider.provided_outcomes[0].evidence_refs =
        vec![".ai/evidence/WI-PROVIDER.verification.json".into()];
    let mut consumer = declaration(
        root.path(),
        &[],
        &[("WI-PROVIDER", "api", OutcomeStage::ComposableHead)],
    );
    consumer.consumed_outcomes[0].verification_required = true;
    let evidence_path = root
        .path()
        .join(".ai/evidence/WI-PROVIDER.verification.json");
    fs::create_dir_all(evidence_path.parent().unwrap()).expect("evidence directory");
    fs::write(&evidence_path, "{}\n").expect("untyped placeholder evidence");
    store
        .register(registration(root.path(), "WI-PROVIDER", 1, provider))
        .expect("register provider");
    store
        .register(registration(root.path(), "WI-CONSUMER", 1, consumer))
        .expect("register consumer");

    let empty_evidence =
        admit_collaboration_action(&store, "WI-CONSUMER", 1, composition_action("WI-CONSUMER"))
            .expect("admission");
    assert!(!empty_evidence.allowed);
    assert!(
        empty_evidence
            .blockers
            .iter()
            .any(|blocker| blocker == "dependency_evidence_missing:WI-PROVIDER:api")
    );

    record_typed_verification(root.path(), "WI-PROVIDER");
    publish_verification_outcome(&store, root.path(), "WI-PROVIDER", 1, "api");
    let bound_evidence =
        admit_collaboration_action(&store, "WI-CONSUMER", 1, composition_action("WI-CONSUMER"))
            .expect("admission after a real Runtime receipt");
    assert!(
        bound_evidence.allowed,
        "valid bound receipt should satisfy dependency"
    );

    let mut changed_bytes = fs::read(&evidence_path).expect("read published verification evidence");
    changed_bytes.extend_from_slice(b" \n");
    fs::write(&evidence_path, changed_bytes).expect("mutate published evidence bytes");
    let mutated_evidence =
        admit_collaboration_action(&store, "WI-CONSUMER", 1, composition_action("WI-CONSUMER"))
            .expect("admission after evidence mutation");
    assert!(
        mutated_evidence
            .blockers
            .iter()
            .any(|blocker| blocker == "dependency_evidence_missing:WI-PROVIDER:api"),
        "a publication must not survive a byte-level evidence change: {:?}",
        mutated_evidence.blockers
    );
}

#[test]
fn outcome_publication_rejects_missing_verification_receipt_before_appending_event() {
    let root = repository();
    let store = store(root.path());
    let mut provider = declaration(root.path(), &[("api", OutcomeStage::ComposableHead)], &[]);
    provider.provided_outcomes[0].evidence_refs =
        vec![".ai/evidence/WI-PROVIDER.verification.json".into()];
    let mut consumer = declaration(
        root.path(),
        &[],
        &[("WI-PROVIDER", "api", OutcomeStage::ComposableHead)],
    );
    consumer.consumed_outcomes[0].verification_required = true;
    let evidence_path = root
        .path()
        .join(".ai/evidence/WI-PROVIDER.verification.json");
    fs::create_dir_all(evidence_path.parent().unwrap()).expect("evidence directory");
    fs::write(&evidence_path, "{}\n").expect("untyped evidence");
    store
        .register(registration(root.path(), "WI-PROVIDER", 1, provider))
        .expect("register provider");
    store
        .register(registration(root.path(), "WI-CONSUMER", 1, consumer))
        .expect("register consumer");

    let result = publish_outcome(&store, "WI-PROVIDER", 1, "api");

    assert!(
        matches!(&result, Err(CoordinationError::RecoveryRequired(message)) if message.contains("verification receipt")),
        "untyped evidence must not be published to a verification-required consumer: {result:?}"
    );
    assert!(
        store.inspect().unwrap().events.is_empty(),
        "rejected publication must not append an event"
    );
}

#[cfg(unix)]
#[test]
fn outcome_publication_rejects_parent_symlink_escape() {
    use std::os::unix::fs::symlink;

    let root = repository();
    let store = store(root.path());
    let outside = tempfile::tempdir().expect("outside evidence directory");
    fs::write(outside.path().join("outcome.json"), "outside evidence\n")
        .expect("write outside evidence");

    let evidence_dir = root.path().join(".ai/evidence");
    fs::create_dir_all(&evidence_dir).expect("registered evidence directory");
    fs::write(evidence_dir.join("outcome.json"), "registered evidence\n")
        .expect("write initial in-tree evidence");
    let mut provider = declaration(root.path(), &[("api", OutcomeStage::ComposableHead)], &[]);
    provider.provided_outcomes[0].evidence_refs = vec![".ai/evidence/outcome.json".into()];
    store
        .register(registration(root.path(), "WI-PROVIDER", 1, provider))
        .expect("register provider against in-tree evidence");

    let evidence_backup = root.path().join(".ai/evidence-before-symlink");
    fs::rename(&evidence_dir, &evidence_backup).expect("preserve original evidence directory");
    symlink(outside.path(), &evidence_dir).expect("link evidence parent outside worktree");

    let result = publish_outcome(&store, "WI-PROVIDER", 1, "api");

    assert!(
        result.is_err(),
        "publication must reject evidence reached through a parent symlink: {result:?}"
    );
    assert!(
        store.inspect().unwrap().events.is_empty(),
        "rejected outside evidence must not append an OutcomePublished event"
    );

    fs::remove_file(&evidence_dir).expect("remove parent symlink");
    fs::rename(&evidence_backup, &evidence_dir).expect("restore original evidence directory");
    fs::remove_file(evidence_dir.join("outcome.json")).expect("remove in-tree evidence leaf");
    symlink(
        outside.path().join("outcome.json"),
        evidence_dir.join("outcome.json"),
    )
    .expect("link evidence leaf outside worktree");
    let final_symlink_result = publish_outcome(&store, "WI-PROVIDER", 1, "api");
    assert!(
        final_symlink_result.is_err(),
        "publication must continue rejecting a final evidence symlink"
    );
    assert!(
        store.inspect().unwrap().events.is_empty(),
        "rejected final symlink must not append an OutcomePublished event"
    );
}

#[test]
fn outcome_publication_rejects_malformed_and_identity_mismatched_receipts() {
    let root = repository();
    let store = store(root.path());
    let mut provider = declaration(root.path(), &[("api", OutcomeStage::ComposableHead)], &[]);
    provider.provided_outcomes[0].evidence_refs =
        vec![".ai/evidence/WI-PROVIDER.verification.json".into()];
    let mut consumer = declaration(
        root.path(),
        &[],
        &[("WI-PROVIDER", "api", OutcomeStage::ComposableHead)],
    );
    consumer.consumed_outcomes[0].verification_required = true;
    let evidence_path = root
        .path()
        .join(".ai/evidence/WI-PROVIDER.verification.json");
    let provider_registration = registration(root.path(), "WI-PROVIDER", 1, provider);
    let consumer_registration = registration(root.path(), "WI-CONSUMER", 1, consumer);
    fs::create_dir_all(evidence_path.parent().unwrap()).expect("evidence directory");
    fs::write(&evidence_path, b"{\"protocolVersion\":").expect("malformed receipt");
    let registered_evidence = Path::new(&provider_registration.worktree_path)
        .join(".ai/evidence/WI-PROVIDER.verification.json");
    assert!(
        registered_evidence.is_file(),
        "{}",
        registered_evidence.display()
    );
    store
        .register(provider_registration)
        .expect("register provider");
    store
        .register(consumer_registration)
        .expect("register consumer");

    assert!(
        publish_outcome(&store, "WI-PROVIDER", 1, "api").is_err(),
        "malformed evidence must not be published"
    );
    assert!(store.inspect().unwrap().events.is_empty());

    record_typed_verification(root.path(), "WI-PROVIDER");
    let valid_bytes = fs::read(&evidence_path).expect("valid verification receipt");
    let valid: serde_json::Value =
        serde_json::from_slice(&valid_bytes).expect("valid verification JSON");
    let invalid_fields = vec![
        ("workItemId", serde_json::json!("WI-OTHER")),
        (
            "repositoryId",
            serde_json::json!(digest("other-repository").to_string()),
        ),
        (
            "repositorySnapshotDigest",
            serde_json::json!(digest("other-head").to_string()),
        ),
        (
            "contractDigest",
            serde_json::json!(digest("other-contract").to_string()),
        ),
        ("runtimeVersion", serde_json::json!("0.2.105")),
        (
            "runtimeDigest",
            serde_json::json!(digest("other-runtime").to_string()),
        ),
        ("passed", serde_json::json!(false)),
        ("captureMode", serde_json::json!("legacy_untyped")),
        ("receipt", serde_json::Value::Null),
    ];
    for (field, replacement) in invalid_fields {
        let mut mismatched = valid.clone();
        mismatched[field] = replacement;
        fs::write(
            &evidence_path,
            serde_json::to_vec(&mismatched).expect("serialize mismatched receipt"),
        )
        .expect("write mismatched receipt");
        assert!(
            publish_outcome(&store, "WI-PROVIDER", 1, "api").is_err(),
            "receipt field {field} must be bound before publication"
        );
        assert!(
            store.inspect().unwrap().events.is_empty(),
            "rejected field {field} must leave no OutcomePublished event"
        );
    }
    fs::write(&evidence_path, valid_bytes).expect("restore valid evidence");
    publish_outcome(&store, "WI-PROVIDER", 1, "api").expect("publish valid bound receipt");
    assert_eq!(store.inspect().unwrap().events.len(), 1);
}

#[test]
fn verification_dependency_requires_current_generation_publication_binding() {
    let root = repository();
    let store = store(root.path());
    let mut provider = declaration(root.path(), &[("api", OutcomeStage::ComposableHead)], &[]);
    provider.provided_outcomes[0].evidence_refs =
        vec![".ai/evidence/WI-PROVIDER.verification.json".into()];
    let mut consumer = declaration(
        root.path(),
        &[],
        &[("WI-PROVIDER", "api", OutcomeStage::ComposableHead)],
    );
    consumer.consumed_outcomes[0].verification_required = true;
    let evidence_path = root
        .path()
        .join(".ai/evidence/WI-PROVIDER.verification.json");
    fs::create_dir_all(evidence_path.parent().unwrap()).expect("evidence directory");
    fs::write(&evidence_path, "{}\n").expect("initial placeholder evidence");
    store
        .register(registration(
            root.path(),
            "WI-PROVIDER",
            1,
            provider.clone(),
        ))
        .expect("register first provider generation");
    store
        .register(registration(root.path(), "WI-CONSUMER", 1, consumer))
        .expect("register consumer");

    record_typed_verification(root.path(), "WI-PROVIDER");
    publish_verification_outcome(&store, root.path(), "WI-PROVIDER", 1, "api");
    let first_generation =
        admit_collaboration_action(&store, "WI-CONSUMER", 1, composition_action("WI-CONSUMER"))
            .unwrap();
    assert!(
        first_generation.allowed,
        "current receipt publication should satisfy evidence"
    );

    store
        .register(registration(root.path(), "WI-PROVIDER", 2, provider))
        .expect("advance provider registration generation");
    let next_generation =
        admit_collaboration_action(&store, "WI-CONSUMER", 1, composition_action("WI-CONSUMER"))
            .unwrap();

    assert!(
        next_generation
            .blockers
            .iter()
            .any(|blocker| blocker == "dependency_evidence_missing:WI-PROVIDER:api"),
        "generation-1 publication must not make the old receipt current for generation 2: {:?}",
        next_generation.blockers
    );
}

#[cfg(unix)]
#[test]
fn dependency_inspection_rejects_published_evidence_through_parent_symlink() {
    use std::os::unix::fs::symlink;

    let root = repository();
    let store = store(root.path());
    let mut provider = declaration(root.path(), &[("api", OutcomeStage::ComposableHead)], &[]);
    provider.provided_outcomes[0].evidence_refs =
        vec![".ai/evidence/WI-PROVIDER.verification.json".into()];
    let mut consumer = declaration(
        root.path(),
        &[],
        &[("WI-PROVIDER", "api", OutcomeStage::ComposableHead)],
    );
    consumer.consumed_outcomes[0].verification_required = true;
    let evidence_dir = root.path().join(".ai/evidence");
    let evidence_path = evidence_dir.join("WI-PROVIDER.verification.json");
    fs::create_dir_all(&evidence_dir).expect("evidence directory");
    fs::write(&evidence_path, "placeholder\n").expect("initial receipt placeholder");
    store
        .register(registration(root.path(), "WI-PROVIDER", 1, provider))
        .expect("register provider");
    store
        .register(registration(root.path(), "WI-CONSUMER", 1, consumer))
        .expect("register consumer");

    record_typed_verification(root.path(), "WI-PROVIDER");
    publish_verification_outcome(&store, root.path(), "WI-PROVIDER", 1, "api");
    let before =
        admit_collaboration_action(&store, "WI-CONSUMER", 1, composition_action("WI-CONSUMER"))
            .expect("inspect valid published evidence");
    assert!(
        before.allowed,
        "valid current receipt should admit: {before:?}"
    );

    let outside = tempfile::tempdir().expect("outside evidence directory");
    let receipt_bytes = fs::read(&evidence_path).expect("read current Runtime receipt");
    fs::write(
        outside.path().join("WI-PROVIDER.verification.json"),
        receipt_bytes,
    )
    .expect("copy receipt outside registered worktree");
    let evidence_backup = root.path().join(".ai/evidence-before-symlink");
    fs::rename(&evidence_dir, &evidence_backup).expect("preserve original evidence directory");
    symlink(outside.path(), &evidence_dir).expect("link evidence parent outside worktree");

    let after =
        admit_collaboration_action(&store, "WI-CONSUMER", 1, composition_action("WI-CONSUMER"))
            .expect("inspect after evidence parent substitution");
    assert!(
        !after.allowed
            && (after
                .blockers
                .iter()
                .any(|blocker| blocker == "dependency_evidence_missing:WI-PROVIDER:api")
                || (after
                    .blockers
                    .iter()
                    .any(|blocker| blocker == "dependency_missing:WI-PROVIDER:api")
                    && after.unknowns.iter().any(
                        |unknown| unknown.contains("evidence parent is not safely contained")
                    ))),
        "outside evidence must remain blocked and the safely detected parent escape must be visible: {after:?}"
    );
}

#[test]
fn ordinary_single_work_item_verification_remains_serial_without_coordination_state() {
    let root = repository();
    let _contract = contract_digest(root.path(), "WI-SERIAL-ONLY");
    let contract_path = root
        .path()
        .join(".ai/work-items/active/WI-SERIAL-ONLY.contract.json");
    let preflight = preflight_work_item(root.path(), &contract_path).expect("preflight");
    assert_ne!(preflight.state, cockpit_core::DecisionState::Red);
    checkpoint_work_item(root.path(), "WI-SERIAL-ONLY").expect("checkpoint");
    let receipt = execute_bounded(
        vec![VerificationCommand::new(
            "serial-gate",
            "sh",
            vec!["-c".into(), "true".into()],
            VerificationReusePolicy::NeverReuse,
        )],
        1,
    )
    .expect("serial verification execution");
    assert!(receipt.passed);
    assert_eq!(receipt.processes_spawned, 1);
    assert_eq!(receipt.max_concurrent_processes, 1);
    let receipt = serde_json::to_value(receipt).expect("serialize receipt");
    record_verification(
        root.path(),
        "WI-SERIAL-ONLY",
        &receipt,
        "0.2.113",
        &digest("candidate-runtime"),
    )
    .expect("record ordinary verification");
    assert!(
        !GitRepository::discover(root.path())
            .unwrap()
            .topology()
            .unwrap()
            .common_dir
            .join(".ai-cockpit/coordination/v1")
            .exists(),
        "ordinary serial verification must not require collaboration storage"
    );
}

#[test]
fn merged_target_stage_requires_actual_target_ancestry() {
    let root = repository();
    let main_head = GitRepository::discover(root.path())
        .unwrap()
        .topology()
        .unwrap()
        .head
        .unwrap();
    run(root.path(), &["checkout", "-qb", "feature/provider"]);
    fs::write(root.path().join("provider.txt"), "not merged\n").expect("provider file");
    run(root.path(), &["add", "."]);
    run(root.path(), &["commit", "-qm", "provider head"]);
    let provider_head = GitRepository::discover(root.path())
        .unwrap()
        .topology()
        .unwrap()
        .head
        .unwrap();
    let store = store(root.path());
    store
        .register(registration(
            root.path(),
            "WI-PROVIDER",
            1,
            declaration(root.path(), &[("api", OutcomeStage::MergedTarget)], &[]),
        ))
        .expect("register provider");
    let consumer_declaration = declaration(
        root.path(),
        &[],
        &[("WI-PROVIDER", "api", OutcomeStage::MergedTarget)],
    );
    store
        .register(registration(
            root.path(),
            "WI-CONSUMER",
            1,
            consumer_declaration,
        ))
        .expect("register consumer");
    assert_ne!(provider_head, main_head);

    let admission =
        admit_collaboration_action(&store, "WI-CONSUMER", 1, composition_action("WI-CONSUMER"))
            .expect("admission");
    assert!(!admission.allowed);
    assert!(
        admission
            .blockers
            .iter()
            .any(|blocker| blocker == "outcome_merge_fact_missing:WI-PROVIDER:api")
    );

    run(root.path(), &["checkout", "-q", "main"]);
    run(
        root.path(),
        &[
            "merge",
            "--no-ff",
            "-m",
            "merge provider for MergedTarget acceptance",
            "feature/provider",
        ],
    );
    run(root.path(), &["checkout", "-q", "feature/provider"]);
    let merged =
        admit_collaboration_action(&store, "WI-CONSUMER", 1, composition_action("WI-CONSUMER"))
            .expect("admission after actual target merge");
    assert!(
        merged.allowed,
        "a provider head in the actual main ancestry satisfies MergedTarget: {:?}",
        merged.blockers
    );
}

#[test]
fn selected_outcome_admission_ignores_unselected_outcome_merge_blocker() {
    let root = repository();
    let main_head = GitRepository::discover(root.path())
        .unwrap()
        .topology()
        .unwrap()
        .head
        .unwrap();
    run(root.path(), &["checkout", "-qb", "feature/provider"]);
    fs::write(root.path().join("provider.txt"), "provider change\n").expect("provider file");
    run(root.path(), &["add", "provider.txt"]);
    run(root.path(), &["commit", "-qm", "provider change"]);
    let provider_head = GitRepository::discover(root.path())
        .unwrap()
        .topology()
        .unwrap()
        .head
        .unwrap();
    assert_ne!(provider_head, main_head);

    let store = store(root.path());
    store
        .register(registration(
            root.path(),
            "WI-PROVIDER",
            1,
            declaration(
                root.path(),
                &[
                    ("api", OutcomeStage::ComposableHead),
                    ("docs", OutcomeStage::MergedTarget),
                ],
                &[],
            ),
        ))
        .unwrap();
    store
        .register(registration(
            root.path(),
            "WI-CONSUMER",
            1,
            declaration(
                root.path(),
                &[],
                &[
                    ("WI-PROVIDER", "api", OutcomeStage::ComposableHead),
                    ("WI-PROVIDER", "docs", OutcomeStage::ComposableHead),
                ],
            ),
        ))
        .unwrap();

    let api = admit_collaboration_action(
        &store,
        "WI-CONSUMER",
        1,
        outcome_action("WI-CONSUMER", &[("WI-PROVIDER", "api")]),
    )
    .unwrap();

    assert!(
        api.allowed,
        "unselected docs merge fact must not block api: {:?}",
        api.blockers
    );
    assert!(
        !api.blockers
            .iter()
            .any(|blocker| blocker.starts_with("outcome_merge_fact_")),
        "only the selected api outcome may contribute dependency blockers"
    );
}

#[test]
fn same_work_item_admission_filters_impact_by_consumed_outcome() {
    let root = repository();
    let store = store(root.path());
    store
        .register(registration(
            root.path(),
            "WI-PROVIDER",
            1,
            declaration(
                root.path(),
                &[
                    ("api", OutcomeStage::ComposableHead),
                    ("docs", OutcomeStage::ComposableHead),
                ],
                &[],
            ),
        ))
        .unwrap();
    store
        .register(registration(
            root.path(),
            "WI-CONSUMER",
            1,
            declaration(
                root.path(),
                &[],
                &[
                    ("WI-PROVIDER", "api", OutcomeStage::ComposableHead),
                    ("WI-PROVIDER", "docs", OutcomeStage::ComposableHead),
                ],
            ),
        ))
        .unwrap();
    let mut api_changed = impact(root.path(), "WI-PROVIDER", 1, "impact-api");
    api_changed.outcome_ids = vec!["api".into()];
    report_impact(&store, api_changed).unwrap();

    let api = admit_collaboration_action(
        &store,
        "WI-CONSUMER",
        1,
        outcome_action("WI-CONSUMER", &[("WI-PROVIDER", "api")]),
    )
    .unwrap();
    let docs = admit_collaboration_action(
        &store,
        "WI-CONSUMER",
        1,
        outcome_action("WI-CONSUMER", &[("WI-PROVIDER", "docs")]),
    )
    .unwrap();

    assert!(!api.allowed, "the impacted outcome remains blocked");
    assert!(api.affected);
    assert!(
        docs.allowed,
        "an unaffected outcome in the same WI can proceed"
    );
    assert!(!docs.affected);
}

#[test]
fn action_dependency_selection_is_provider_scoped() {
    let root = repository();
    let store = store(root.path());
    for provider in ["WI-PROVIDER-A", "WI-PROVIDER-B"] {
        store
            .register(registration(
                root.path(),
                provider,
                1,
                declaration(root.path(), &[("api", OutcomeStage::ComposableHead)], &[]),
            ))
            .expect("register provider with shared outcome ID");
    }
    store
        .register(registration(
            root.path(),
            "WI-CONSUMER",
            1,
            declaration(
                root.path(),
                &[],
                &[
                    ("WI-PROVIDER-A", "api", OutcomeStage::ComposableHead),
                    ("WI-PROVIDER-B", "api", OutcomeStage::ComposableHead),
                ],
            ),
        ))
        .expect("register consumer of both provider/outcome pairs");

    let mut provider_a_impact = impact(root.path(), "WI-PROVIDER-A", 1, "impact-provider-a-api");
    provider_a_impact.outcome_ids = vec!["api".into()];
    report_impact(&store, provider_a_impact).expect("publish provider A impact");

    // Before provider identity is carried, this bare ID accidentally selects
    // both providers. The desired call selects only Provider B's `api`.
    let provider_b_only = admit_collaboration_action(
        &store,
        "WI-CONSUMER",
        1,
        outcome_action("WI-CONSUMER", &[("WI-PROVIDER-B", "api")]),
    )
    .expect("admit action selecting provider B only");
    assert!(
        provider_b_only.allowed,
        "Provider A's impact must not cross-block Provider B's identical outcome ID: {:?}",
        provider_b_only.blockers
    );
    assert!(
        !provider_b_only
            .blockers
            .iter()
            .any(|blocker| blocker == "dependency_impact:WI-PROVIDER-A:api"),
        "a provider-B-only action must not consume provider A's invalidation"
    );

    let provider_a_only = admit_collaboration_action(
        &store,
        "WI-CONSUMER",
        1,
        outcome_action("WI-CONSUMER", &[("WI-PROVIDER-A", "api")]),
    )
    .expect("admit action selecting provider A only");
    assert!(
        provider_a_only
            .blockers
            .iter()
            .any(|blocker| blocker == "dependency_impact:WI-PROVIDER-A:api"),
        "selecting Provider A must retain Provider A's impact blocker: {:?}",
        provider_a_only.blockers
    );
}

#[test]
fn impact_propagates_through_three_dependency_levels_only() {
    let root = repository();
    let store = store(root.path());
    let head = GitRepository::discover(root.path())
        .unwrap()
        .topology()
        .unwrap()
        .head
        .unwrap();
    store
        .register(registration(
            root.path(),
            "WI-A",
            1,
            declaration(root.path(), &[("base", OutcomeStage::ComposableHead)], &[]),
        ))
        .unwrap();
    store
        .register(registration(
            root.path(),
            "WI-B",
            1,
            declaration(
                root.path(),
                &[("build", OutcomeStage::ComposableHead)],
                &[("WI-A", "base", OutcomeStage::ComposableHead)],
            ),
        ))
        .unwrap();
    store
        .register(registration(
            root.path(),
            "WI-C",
            1,
            declaration(
                root.path(),
                &[("package", OutcomeStage::ComposableHead)],
                &[("WI-B", "build", OutcomeStage::ComposableHead)],
            ),
        ))
        .unwrap();
    store
        .register(registration(
            root.path(),
            "WI-D",
            1,
            declaration(
                root.path(),
                &[],
                &[("WI-C", "package", OutcomeStage::ComposableHead)],
            ),
        ))
        .unwrap();
    store
        .register(registration(
            root.path(),
            "WI-UNRELATED",
            1,
            declaration(root.path(), &[], &[]),
        ))
        .unwrap();

    let mut invalidation = impact(root.path(), "WI-A", 1, "impact-three-levels");
    invalidation.outcome_ids = vec!["base".into()];
    report_impact(&store, invalidation).unwrap();

    let projection = collaboration_projection(&store).unwrap();
    for work_item_id in ["WI-B", "WI-C", "WI-D"] {
        assert!(
            projection
                .affected_work_items
                .iter()
                .any(|id| id == work_item_id),
            "transitive consumer {work_item_id} must be marked affected: {projection:?}"
        );
        assert!(
            !admit_collaboration_action(&store, work_item_id, 1, composition_action(work_item_id))
                .unwrap()
                .allowed,
            "transitive consumer {work_item_id} must be blocked"
        );
    }
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
    assert_eq!(head.len(), 40, "fixture starts from a real committed head");
}

#[test]
fn removing_provider_output_preserves_transitive_invalidation() {
    let root = repository();
    let store = store(root.path());
    store
        .register(registration(
            root.path(),
            "WI-A",
            1,
            declaration(root.path(), &[("base", OutcomeStage::ComposableHead)], &[]),
        ))
        .unwrap();
    store
        .register(registration(
            root.path(),
            "WI-B",
            1,
            declaration(
                root.path(),
                &[("build", OutcomeStage::ComposableHead)],
                &[("WI-A", "base", OutcomeStage::ComposableHead)],
            ),
        ))
        .unwrap();
    store
        .register(registration(
            root.path(),
            "WI-C",
            1,
            declaration(
                root.path(),
                &[("package", OutcomeStage::ComposableHead)],
                &[("WI-B", "build", OutcomeStage::ComposableHead)],
            ),
        ))
        .unwrap();
    store
        .register(registration(
            root.path(),
            "WI-D",
            1,
            declaration(
                root.path(),
                &[],
                &[("WI-C", "package", OutcomeStage::ComposableHead)],
            ),
        ))
        .unwrap();
    store
        .register(registration(
            root.path(),
            "WI-UNRELATED",
            1,
            declaration(root.path(), &[], &[]),
        ))
        .unwrap();

    store
        .register(registration(
            root.path(),
            "WI-A",
            2,
            declaration(root.path(), &[], &[]),
        ))
        .expect("new provider generation may remove a published output");

    let projection = collaboration_projection(&store).unwrap();
    let invalidation = projection
        .events
        .iter()
        .find(|event| event.event_id == "auto-impact-WI-A-2")
        .expect("provider identity change persists an invalidation event");
    assert_eq!(
        invalidation.outcome_ids,
        vec!["base"],
        "the event must retain the removed output identity"
    );
    for work_item_id in ["WI-B", "WI-C", "WI-D"] {
        assert!(
            !admit_collaboration_action(&store, work_item_id, 1, composition_action(work_item_id))
                .unwrap()
                .allowed,
            "removed output must invalidate transitive consumer {work_item_id}"
        );
    }
    assert!(
        admit_collaboration_action(
            &store,
            "WI-UNRELATED",
            1,
            composition_action("WI-UNRELATED")
        )
        .unwrap()
        .allowed,
        "unrelated work remains admissible"
    );

    store
        .register(registration(
            root.path(),
            "WI-A",
            3,
            declaration(root.path(), &[], &[]),
        ))
        .expect("provider may advance again after removing its output");
    let later_projection = collaboration_projection(&store).unwrap();
    let historical_invalidation = later_projection
        .events
        .iter()
        .find(|event| event.event_id == "auto-impact-WI-A-2")
        .expect("removed-output invalidation remains readable after another generation");
    assert_eq!(
        historical_invalidation.outcome_ids,
        vec!["base"],
        "advancing the provider must not erase the removed output from its historical invalidation"
    );
    for work_item_id in ["WI-B", "WI-C", "WI-D"] {
        assert!(
            !admit_collaboration_action(&store, work_item_id, 1, composition_action(work_item_id))
                .unwrap()
                .allowed,
            "historical removed-output invalidation must continue through {work_item_id}"
        );
    }
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
fn historical_impact_can_be_recovered_after_provider_generation_advances() {
    let root = repository();
    let store = store(root.path());
    register_provider_and_consumer(&store, root.path(), OutcomeStage::ComposableHead);
    let mut event = impact(
        root.path(),
        "WI-PROVIDER",
        1,
        "impact-before-provider-advance",
    );
    event.outcome_ids = vec!["api".into()];
    report_impact(&store, event).unwrap();
    assert!(
        !admit_collaboration_action(
            &store,
            "WI-CONSUMER",
            1,
            outcome_action("WI-CONSUMER", &[("WI-PROVIDER", "api")]),
        )
        .unwrap()
        .allowed
    );

    let advanced = registration(
        root.path(),
        "WI-PROVIDER",
        2,
        declaration(root.path(), &[("api", OutcomeStage::ComposableHead)], &[]),
    );
    store
        .register(advanced)
        .expect("provider generation can advance without changing outcome identity");

    let recovery = recover_impact(&store, "impact-before-provider-advance", "WI-CONSUMER", 1)
        .expect("historical provider event remains recoverable");
    assert_eq!(recovery.provider_generation, 1);
    assert_eq!(recovery.current_provider_generation, Some(2));

    let inspection = store.inspect().unwrap();
    assert!(
        inspection
            .events
            .iter()
            .any(|event| event.event_id == "impact-before-provider-advance")
    );
    assert!(
        admit_collaboration_action(
            &store,
            "WI-CONSUMER",
            1,
            outcome_action("WI-CONSUMER", &[("WI-PROVIDER", "api")]),
        )
        .unwrap()
        .allowed
    );
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
    let paused_projection =
        collaboration_outcome_projection(root.path(), "WI-CONSUMER", &runtime_context());
    assert_eq!(paused_projection.state, "blocked");
    assert!(
        paused_projection
            .blockers
            .iter()
            .any(|blocker| blocker.starts_with("coordination_safely_paused:"))
    );

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

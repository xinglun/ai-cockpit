use cockpit_core::Digest;
use cockpit_protocol::{COLLABORATION_CAPABILITY, CompositionBinding, RuntimeCapabilityBinding};
use cockpit_verification::{
    CompositionCommand, CompositionIdentity, CompositionInput, CompositionPrecondition,
    ReuseDecisionKind, classify_reuse, composition_commands_digest, run_composition,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn tempdir(label: &str) -> TempDir {
    let sequence = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "cockpit-composition-{label}-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("temp directory");
    TempDir(path)
}

fn digest(label: &str) -> Digest {
    Digest::sha256_bytes(label.as_bytes())
}

fn run(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("git command");
    assert!(output.status.success(), "git {:?} failed", args);
    String::from_utf8(output.stdout)
        .expect("utf8")
        .trim()
        .into()
}

fn repository() -> TempDir {
    let root = tempdir("repository");
    run(root.path(), &["init", "-q"]);
    run(
        root.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    run(root.path(), &["config", "user.name", "Test"]);
    fs::write(root.path().join("README.md"), "initial\n").expect("write");
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

fn identity(label: &str) -> CompositionIdentity {
    CompositionIdentity {
        source_digest: digest(&format!("source-{label}")),
        dependency_digest: digest(&format!("dependency-{label}")),
        interface_digest: digest(&format!("interface-{label}")),
        configuration_digest: digest(&format!("configuration-{label}")),
        toolchain_digest: digest(&format!("toolchain-{label}")),
        lockfile_digest: digest(&format!("lockfile-{label}")),
        generated_input_digest: digest(&format!("generated-{label}")),
        environment_digest: digest(&format!("environment-{label}")),
        verifier_digest: digest(&format!("verifier-{label}")),
        command_digest: digest("placeholder-command-digest"),
    }
}

fn binding(target_sha: &str, participant_heads: Vec<String>) -> CompositionBinding {
    CompositionBinding {
        schema_version: 1,
        repository_id: digest("repository"),
        binding_id: "composition-test".into(),
        target_branch: "main".into(),
        target_sha: target_sha.into(),
        participant_work_items: vec!["WI-PROVIDER".into(), "WI-CONSUMER".into()],
        participant_heads,
        contract_digests: vec![digest("provider-contract"), digest("consumer-contract")],
        verifier: runtime(),
    }
}

fn command(node_id: &str, program: &str, args: &[&str]) -> CompositionCommand {
    CompositionCommand {
        node_id: node_id.into(),
        program: program.into(),
        args: args.iter().map(|arg| (*arg).into()).collect(),
        depends_on: Vec::new(),
        environment: Default::default(),
        input_paths: vec!["README.md".into()],
        covered_scenarios: Vec::new(),
        covered_constraints: Vec::new(),
    }
}

#[test]
fn composition_commands_accept_explicit_upstream_dependencies() {
    let command: CompositionCommand = serde_json::from_value(serde_json::json!({
        "nodeId": "consumer",
        "program": "sh",
        "args": ["-c", "true"],
        "dependsOn": ["provider"]
    }))
    .expect("command dependency protocol");

    assert_eq!(command.node_id, "consumer");
    assert_eq!(
        serde_json::to_value(command)
            .expect("serialize command")
            .get("dependsOn")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(1)
    );
}

#[test]
fn composition_rejects_dependencies_that_are_not_prior_nodes_before_spawn() {
    let root = repository();
    let head = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let mut consumer = command("consumer", "sh", &["-c", "true"]);
    consumer.depends_on = vec!["provider".into()];
    let attempt = run_composition(input(
        root.path(),
        state.path(),
        binding(&head, vec![head.clone(), head.clone()]),
        vec![consumer, command("provider", "sh", &["-c", "true"])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ))
    .expect("invalid composition is recorded");

    assert!(!attempt.passed);
    assert_eq!(attempt.processes_spawned, 0);
    assert_eq!(
        attempt.failure.as_deref(),
        Some("invalid_composition_command_graph")
    );
    assert!(attempt.isolated_worktree.is_empty());
}

fn input(
    root: &Path,
    state_dir: &Path,
    binding: CompositionBinding,
    commands: Vec<CompositionCommand>,
    preconditions: Vec<CompositionPrecondition>,
) -> CompositionInput {
    let mut identity = identity("stable");
    identity.command_digest = composition_commands_digest(&commands);
    CompositionInput {
        repository_root: root.to_path_buf(),
        state_dir: state_dir.to_path_buf(),
        binding,
        identity,
        reusable_node_ids: commands
            .iter()
            .map(|command| command.node_id.clone())
            .collect(),
        commands,
        preconditions,
        timeout_seconds: 1,
    }
}

#[test]
fn exact_composition_uses_real_linked_worktree_and_finds_interface_error() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    run(root.path(), &["checkout", "-qb", "provider"]);
    fs::write(root.path().join("provider-api.txt"), "api\n").expect("provider file");
    run(root.path(), &["add", "."]);
    run(root.path(), &["commit", "-qm", "provider"]);
    let provider_head = run(root.path(), &["rev-parse", "HEAD"]);
    run(root.path(), &["checkout", "-q", "-b", "consumer", &base]);
    fs::write(root.path().join("consumer.txt"), "consumer\n").expect("consumer file");
    run(root.path(), &["add", "."]);
    run(root.path(), &["commit", "-qm", "consumer"]);
    let consumer_head = run(root.path(), &["rev-parse", "HEAD"]);
    run(root.path(), &["checkout", "-q", "main"]);
    let state = tempdir("state");
    let missing_interface = "test -f required-interface.txt";
    let attempt = run_composition(input(
        root.path(),
        state.path(),
        binding(&base, vec![provider_head, consumer_head]),
        vec![command("interface-check", "sh", &["-c", missing_interface])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ))
    .expect("composition attempt");
    assert!(!attempt.passed);
    assert!(attempt.isolated_worktree.ends_with("composition"));
    assert_eq!(attempt.binding.target_sha, base);
    assert_eq!(attempt.binding.participant_work_items.len(), 2);
    assert_eq!(attempt.execution_records.len(), 1);
    assert!(attempt.execution_records[0].spawned);
    assert!(
        attempt
            .failure
            .as_deref()
            .unwrap_or_default()
            .contains("exit")
    );
}

#[test]
fn failed_precondition_spawns_zero_expensive_processes_and_persists_attempt() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let sentinel = state.path().join("should-not-exist");
    let attempt = run_composition(input(
        root.path(),
        state.path(),
        binding(&base, vec![base.clone(), base.clone()]),
        vec![command(
            "expensive",
            "sh",
            &["-c", &format!("touch {}", sentinel.display())],
        )],
        vec![CompositionPrecondition::unsatisfied(
            "dependency-ready",
            "dependency impact is unresolved",
        )],
    ))
    .expect("precondition attempt");
    assert!(!attempt.passed);
    assert_eq!(attempt.processes_spawned, 0);
    assert!(attempt.execution_records.is_empty());
    assert!(!sentinel.exists());
    assert!(
        state
            .path()
            .join(format!("{}.json", attempt.attempt_id))
            .exists()
    );
}

#[test]
fn failed_attempts_are_append_only_and_exact_identity_controls_reuse() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command("failure", "sh", &["-c", "exit 17"])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let first = run_composition(composition.clone()).expect("first attempt");
    std::thread::sleep(std::time::Duration::from_millis(1));
    let second = run_composition(composition.clone()).expect("retry attempt");
    assert_ne!(first.attempt_id, second.attempt_id);
    assert!(
        state
            .path()
            .join(format!("{}.json", first.attempt_id))
            .exists()
    );
    assert!(
        state
            .path()
            .join(format!("{}.json", second.attempt_id))
            .exists()
    );
    assert_eq!(
        classify_reuse(&first, &composition).kind,
        ReuseDecisionKind::Unknown
    );

    let mut changed = composition;
    changed.identity.interface_digest = digest("changed-interface");
    assert_eq!(
        classify_reuse(&first, &changed).kind,
        ReuseDecisionKind::Unknown
    );
}

#[test]
fn successful_exact_identity_allows_reuse_but_command_change_reexecutes() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command("cheap", "sh", &["-c", "true"])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let passed = run_composition(composition.clone()).expect("successful attempt");
    assert!(passed.passed);
    assert_eq!(
        classify_reuse(&passed, &composition).kind,
        ReuseDecisionKind::Unknown
    );

    let mut changed = composition;
    changed.commands = vec![command("cheap", "sh", &["-c", "false"])];
    changed.identity.command_digest = composition_commands_digest(&changed.commands);
    assert_eq!(
        classify_reuse(&passed, &changed).kind,
        ReuseDecisionKind::Unknown
    );
}

#[test]
fn repeated_exact_composition_reuses_without_spawning_a_process() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command("cheap", "sh", &["-c", "true"])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );

    let first = run_composition(composition.clone()).expect("first attempt");
    let second = run_composition(composition).expect("second attempt");

    assert!(first.passed);
    assert!(second.passed);
    assert_eq!(second.processes_spawned, 0);
    assert_eq!(second.execution_records.len(), 1);
    assert!(!second.execution_records[0].spawned);
    assert!(second.execution_records[0].reused);
    assert_eq!(
        second.execution_records[0]
            .predecessor_attempt_id
            .as_deref(),
        Some(first.attempt_id.as_str())
    );
}

#[test]
fn node_without_observable_inputs_executes_again_instead_of_reusing() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let mut unverifiable = command("unverifiable", "sh", &["-c", "true"]);
    unverifiable.input_paths.clear();
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![unverifiable],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let first = run_composition(composition.clone()).expect("first attempt");
    let second = run_composition(composition).expect("second attempt");

    assert!(first.passed && second.passed);
    assert_eq!(second.processes_spawned, 1);
    assert!(!second.execution_records[0].reused);
    assert_eq!(second.reuse_decision.kind, ReuseDecisionKind::Unknown);
}

#[test]
fn changed_command_only_reexecutes_the_affected_node() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let first_input = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![
            command("stable", "sh", &["-c", "true"]),
            command("changed", "sh", &["-c", "true"]),
        ],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let first = run_composition(first_input.clone()).expect("first attempt");
    assert!(first.passed);

    let second_input = input(
        root.path(),
        state.path(),
        first_input.binding,
        vec![
            command("stable", "sh", &["-c", "true"]),
            command("changed", "sh", &["-c", "false"]),
        ],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let second = run_composition(second_input).expect("second attempt");

    assert!(!second.passed);
    assert_eq!(second.processes_spawned, 1);
    assert!(!second.execution_records[0].spawned);
    assert!(second.execution_records[0].reused);
    assert!(second.execution_records[1].spawned);
    assert!(!second.execution_records[1].passed);
}

#[test]
fn changed_source_file_only_reexecutes_nodes_that_observe_that_file() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    run(root.path(), &["checkout", "-qb", "provider"]);
    fs::write(root.path().join("api.txt"), "api-v1\n").expect("api input");
    fs::write(root.path().join("docs.txt"), "docs-stable\n").expect("docs input");
    run(root.path(), &["add", "."]);
    run(root.path(), &["commit", "-qm", "provider-v1"]);
    let first_head = run(root.path(), &["rev-parse", "HEAD"]);
    run(root.path(), &["checkout", "-q", "main"]);

    let state = tempdir("state");
    let mut api = command("api", "sh", &["-c", "true"]);
    api.input_paths = vec!["api.txt".into()];
    let mut docs = command("docs", "sh", &["-c", "true"]);
    docs.input_paths = vec!["docs.txt".into()];
    let first_input = input(
        root.path(),
        state.path(),
        binding(&base, vec![first_head.clone(), base.clone()]),
        vec![api.clone(), docs.clone()],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let first = run_composition(first_input).expect("first attempt");
    assert!(first.passed);

    run(root.path(), &["checkout", "-q", "provider"]);
    fs::write(root.path().join("api.txt"), "api-v2\n").expect("updated api input");
    run(root.path(), &["add", "api.txt"]);
    run(root.path(), &["commit", "-qm", "provider-api-v2"]);
    let second_head = run(root.path(), &["rev-parse", "HEAD"]);
    run(root.path(), &["checkout", "-q", "main"]);

    let second = run_composition(input(
        root.path(),
        state.path(),
        binding(&base, vec![second_head, base.clone()]),
        vec![api, docs],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ))
    .expect("second attempt");

    assert!(second.passed);
    assert_eq!(second.processes_spawned, 1);
    let api_record = second
        .execution_records
        .iter()
        .find(|record| record.node_id == "api")
        .unwrap();
    let docs_record = second
        .execution_records
        .iter()
        .find(|record| record.node_id == "docs")
        .unwrap();
    assert!(api_record.spawned);
    assert!(!api_record.reused);
    assert!(!docs_record.spawned);
    assert!(docs_record.reused);
}

#[test]
fn changed_upstream_receipt_reexecutes_transitive_dependents_only() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    run(root.path(), &["checkout", "-qb", "provider"]);
    fs::write(root.path().join("api.txt"), "api-v1\n").expect("api input");
    fs::write(root.path().join("docs.txt"), "docs-stable\n").expect("docs input");
    run(root.path(), &["add", "."]);
    run(root.path(), &["commit", "-qm", "provider-v1"]);
    let first_head = run(root.path(), &["rev-parse", "HEAD"]);
    run(root.path(), &["checkout", "-q", "main"]);

    let state = tempdir("state");
    let mut source = command("source", "sh", &["-c", "cat api.txt"]);
    source.input_paths = vec!["api.txt".into()];
    let mut consumer = command("consumer", "sh", &["-c", "true"]);
    consumer.depends_on = vec!["source".into()];
    let mut transitive = command("transitive", "sh", &["-c", "true"]);
    transitive.depends_on = vec!["consumer".into()];
    let mut independent = command("independent", "sh", &["-c", "true"]);
    independent.input_paths = vec!["docs.txt".into()];
    let commands = vec![
        source.clone(),
        consumer.clone(),
        transitive.clone(),
        independent.clone(),
    ];
    let first = run_composition(input(
        root.path(),
        state.path(),
        binding(&base, vec![first_head.clone(), base.clone()]),
        commands.clone(),
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ))
    .expect("first attempt");
    assert!(first.passed);

    run(root.path(), &["checkout", "-q", "provider"]);
    fs::write(root.path().join("api.txt"), "api-v2\n").expect("updated api input");
    run(root.path(), &["add", "api.txt"]);
    run(root.path(), &["commit", "-qm", "provider-api-v2"]);
    let second_head = run(root.path(), &["rev-parse", "HEAD"]);
    run(root.path(), &["checkout", "-q", "main"]);

    let second = run_composition(input(
        root.path(),
        state.path(),
        binding(&base, vec![second_head, base.clone()]),
        commands,
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ))
    .expect("second attempt");

    assert!(second.passed);
    assert_eq!(second.processes_spawned, 3);
    for node_id in ["source", "consumer", "transitive"] {
        let record = second
            .execution_records
            .iter()
            .find(|record| record.node_id == node_id)
            .unwrap();
        assert!(record.spawned, "{node_id} must be re-executed");
        assert!(
            !record.reused,
            "{node_id} cannot reuse a stale dependency receipt"
        );
    }
    let independent = second
        .execution_records
        .iter()
        .find(|record| record.node_id == "independent")
        .unwrap();
    assert!(!independent.spawned);
    assert!(independent.reused);
}

#[test]
fn actual_command_environment_change_invalidates_reuse_even_when_json_identity_is_stale() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let mut first_command = command(
        "environment-check",
        "sh",
        &["-c", "test \"$COMPOSITION_FLAVOR\" = one"],
    );
    first_command
        .environment
        .insert("COMPOSITION_FLAVOR".into(), "one".into());
    let first_input = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![first_command],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let first = run_composition(first_input.clone()).expect("first attempt");
    assert!(first.passed, "first attempt: {first:?}");

    let mut second_input = first_input;
    second_input.commands[0]
        .environment
        .insert("COMPOSITION_FLAVOR".into(), "two".into());
    // Deliberately keep every caller-supplied identity digest unchanged.
    let second = run_composition(second_input).expect("second attempt");

    assert!(!second.passed);
    assert_eq!(second.processes_spawned, 1);
    assert!(second.execution_records[0].spawned);
}

#[test]
fn inherited_environment_change_invalidates_reuse_across_real_processes() {
    let child_mode = std::env::var_os("COMPOSITION_ENV_CHILD").is_some();
    if child_mode {
        let root = PathBuf::from(std::env::var_os("COMPOSITION_ENV_ROOT").expect("root path"));
        let state = PathBuf::from(std::env::var_os("COMPOSITION_ENV_STATE").expect("state path"));
        let base = run(&root, &["rev-parse", "refs/heads/main"]);
        let check = command(
            "inherited-environment-check",
            "sh",
            &["-c", "test -n \"$COMPOSITION_EXTERNAL_FLAVOR\""],
        );
        let attempt = run_composition(input(
            &root,
            &state,
            binding(&base, vec![base.clone(), base.clone()]),
            vec![check],
            vec![CompositionPrecondition::satisfied("identity-bound")],
        ))
        .expect("child composition attempt");
        assert!(attempt.passed, "child composition failed: {attempt:?}");
        return;
    }

    let root = repository();
    let state = tempdir("inherited-environment-state");
    for flavor in ["one", "two"] {
        let child = Command::new(std::env::current_exe().expect("integration-test executable"))
            .args([
                "--exact",
                "inherited_environment_change_invalidates_reuse_across_real_processes",
            ])
            .env("COMPOSITION_ENV_CHILD", "1")
            .env("COMPOSITION_ENV_ROOT", root.path())
            .env("COMPOSITION_ENV_STATE", state.path())
            .env("COMPOSITION_EXTERNAL_FLAVOR", flavor)
            .output()
            .expect("spawn separate test process");
        assert!(
            child.status.success(),
            "child process for flavor {flavor} failed: {}",
            String::from_utf8_lossy(&child.stderr)
        );
    }

    let mut attempts = fs::read_dir(state.path())
        .expect("composition state directory")
        .map(|entry| {
            let path = entry.expect("attempt entry").path();
            serde_json::from_slice::<serde_json::Value>(&fs::read(path).expect("attempt bytes"))
                .expect("attempt JSON")
        })
        .collect::<Vec<_>>();
    attempts.sort_by_key(|attempt| {
        attempt["recordedAtUnixNanos"]
            .as_u64()
            .expect("recorded timestamp")
    });
    assert_eq!(attempts.len(), 2, "both processes must persist an attempt");
    assert_eq!(attempts[0]["processesSpawned"], 1);
    assert_eq!(
        attempts[1]["processesSpawned"], 1,
        "changing only inherited process environment must execute the node again"
    );
    assert_eq!(attempts[1]["executionRecords"][0]["reused"], false);
}

#[cfg(unix)]
#[test]
fn replacing_the_same_toolchain_executable_invalidates_reuse() {
    use std::os::unix::fs::PermissionsExt;

    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let executable = root.path().join("verify-tool");
    fs::write(&executable, "#!/bin/sh\nexit 0\n").expect("write initial verifier");
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755))
        .expect("make initial verifier executable");
    let mut check = command("toolchain-check", executable.to_str().unwrap(), &[]);
    check.input_paths = vec!["README.md".into()];
    let first_input = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![check],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let first = run_composition(first_input.clone()).expect("first attempt");
    assert!(first.passed);

    fs::write(&executable, "#!/bin/sh\nexit 19\n").expect("replace verifier at same path");
    let second = run_composition(first_input).expect("second attempt");
    assert!(!second.passed);
    assert_eq!(second.processes_spawned, 1);
    assert_eq!(second.execution_records[0].exit_code, Some(19));
}

#[cfg(unix)]
#[test]
fn executable_resolved_from_command_path_override_invalidates_reuse() {
    use std::os::unix::fs::PermissionsExt;

    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let path_override = tempdir("command-path");
    let executable = path_override.path().join("sh");
    fs::write(&executable, "#!/bin/sh\nexit 0\n").expect("write initial command executable");
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755))
        .expect("make command executable");
    let mut check = command("path-toolchain-check", "sh", &[]);
    check.environment.insert(
        "PATH".into(),
        path_override.path().to_string_lossy().into_owned(),
    );
    let first_input = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![check],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let first = run_composition(first_input.clone()).expect("first attempt");
    assert!(first.passed, "first attempt: {first:?}");

    fs::write(&executable, "#!/bin/sh\nexit 19\n").expect("replace command executable");
    let second = run_composition(first_input).expect("second attempt");

    assert!(!second.passed);
    assert_eq!(second.processes_spawned, 1);
    assert_eq!(second.execution_records[0].exit_code, Some(19));
}

#[test]
fn empty_required_checks_fail_closed_without_a_worktree_or_process() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let attempt = run_composition(input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        Vec::new(),
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ))
    .expect("empty composition attempt");

    assert!(!attempt.passed);
    assert_eq!(attempt.processes_spawned, 0);
    assert_eq!(attempt.failure.as_deref(), Some("required_checks_empty"));
    assert!(
        attempt
            .cleanup
            .as_ref()
            .is_some_and(|cleanup| !cleanup.attempted)
    );
}

#[test]
fn timed_out_node_is_durable_and_reports_cleanup() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("state");
    let attempt = run_composition(input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command("timeout", "sh", &["-c", "sleep 2"])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ))
    .expect("timeout composition attempt");

    assert!(!attempt.passed);
    assert_eq!(attempt.processes_spawned, 1);
    assert!(attempt.execution_records[0].timed_out);
    assert!(
        attempt
            .cleanup
            .as_ref()
            .is_some_and(|cleanup| cleanup.attempted)
    );
    assert!(
        state
            .path()
            .join(format!("{}.json", attempt.attempt_id))
            .exists()
    );
}

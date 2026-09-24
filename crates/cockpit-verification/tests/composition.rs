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
    }
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
        ReuseDecisionKind::Execute
    );

    let mut changed = composition;
    changed.identity.interface_digest = digest("changed-interface");
    assert_eq!(
        classify_reuse(&first, &changed).kind,
        ReuseDecisionKind::Execute
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
        ReuseDecisionKind::Reuse
    );

    let mut changed = composition;
    changed.commands = vec![command("cheap", "sh", &["-c", "false"])];
    changed.identity.command_digest = composition_commands_digest(&changed.commands);
    assert_eq!(
        classify_reuse(&passed, &changed).kind,
        ReuseDecisionKind::Execute
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

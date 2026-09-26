use cockpit_core::Digest;
use cockpit_protocol::{COLLABORATION_CAPABILITY, CompositionBinding, RuntimeCapabilityBinding};
use cockpit_verification::{
    CompositionCommand, CompositionError, CompositionIdentity, CompositionInput,
    CompositionPrecondition, ReuseDecisionKind, classify_reuse, composition_commands_digest,
    run_composition,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

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
        vec![command("cheap", "true", &[])],
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
        vec![command("cheap", "true", &[])],
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
fn failed_attempt_cannot_become_reusable_by_tampering_with_its_pass_flag() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("tampered-attempt-state");
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command("must-fail", "false", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );

    let failed = run_composition(composition.clone()).expect("failed attempt is recorded");
    assert!(!failed.passed);
    let attempt_path = state.path().join(format!("{}.json", failed.attempt_id));
    let mut persisted: serde_json::Value =
        serde_json::from_slice(&fs::read(&attempt_path).expect("persisted attempt bytes"))
            .expect("persisted attempt JSON");
    persisted["executionRecords"][0]["passed"] = serde_json::Value::Bool(true);
    fs::write(
        &attempt_path,
        serde_json::to_vec_pretty(&persisted).expect("serialize tampered attempt"),
    )
    .expect("tamper only the pass flag");

    let retry = run_composition(composition).expect("retry executes the required node");

    assert!(!retry.passed);
    assert_eq!(retry.processes_spawned, 1);
    assert!(retry.execution_records[0].spawned);
    assert!(!retry.execution_records[0].reused);
}

#[cfg(unix)]
#[test]
fn inherited_path_cannot_substitute_a_composition_verifier() {
    use std::os::unix::fs::PermissionsExt;

    if std::env::var_os("COMPOSITION_HOSTILE_PATH_CHILD").is_some() {
        let root = PathBuf::from(std::env::var_os("COMPOSITION_HOSTILE_PATH_ROOT").expect("root"));
        let state =
            PathBuf::from(std::env::var_os("COMPOSITION_HOSTILE_PATH_STATE").expect("state"));
        let base = run(&root, &["rev-parse", "refs/heads/main"]);
        let marker =
            PathBuf::from(std::env::var_os("COMPOSITION_FAKE_CARGO_MARKER").expect("marker path"));
        let attempt = run_composition(input(
            &root,
            &state,
            binding(&base, vec![base.clone(), base.clone()]),
            vec![command("required-cargo", "cargo", &["--version"])],
            vec![CompositionPrecondition::satisfied("identity-bound")],
        ))
        .expect("composition records the verifier result");
        assert!(
            !marker.exists(),
            "an inherited PATH entry must not substitute the Contract-required verifier"
        );
        assert!(attempt.passed, "the Runtime-bound cargo verifier must pass");
        assert_eq!(attempt.processes_spawned, 1);

        let mut unbound_override = input(
            &root,
            &state,
            binding(&base, vec![base.clone(), base.clone()]),
            vec![command("override-cargo", "cargo", &["--version"])],
            vec![CompositionPrecondition::satisfied("identity-bound")],
        );
        unbound_override.commands[0].environment.insert(
            "RUSTUP_TOOLCHAIN".into(),
            "missing-runtime-toolchain".into(),
        );
        let rejected = run_composition(unbound_override).expect("reject unbound override");
        assert!(!rejected.passed);
        assert_eq!(rejected.processes_spawned, 0);
        assert!(
            !marker.exists(),
            "an unbound toolchain overlay must be rejected before process start"
        );
        return;
    }

    let root = repository();
    let state = tempdir("hostile-path-state");
    let fake_bin = tempdir("hostile-path-bin");
    let marker = state.path().join("fake-cargo-ran");
    let fake_cargo = fake_bin.path().join("cargo");
    fs::write(
        &fake_cargo,
        "#!/bin/sh\nprintf forged > \"$COMPOSITION_FAKE_CARGO_MARKER\"\nexit 0\n",
    )
    .expect("write fake cargo");
    fs::set_permissions(&fake_cargo, fs::Permissions::from_mode(0o755))
        .expect("make fake cargo executable");

    let mut paths = vec![fake_bin.path().to_path_buf()];
    if let Some(system_path) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&system_path));
    }
    let inherited_path = std::env::join_paths(paths).expect("hostile PATH");
    let child = Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "inherited_path_cannot_substitute_a_composition_verifier",
        ])
        .env("COMPOSITION_HOSTILE_PATH_CHILD", "1")
        .env("COMPOSITION_HOSTILE_PATH_ROOT", root.path())
        .env("COMPOSITION_HOSTILE_PATH_STATE", state.path())
        .env("COMPOSITION_FAKE_CARGO_MARKER", &marker)
        .env("PATH", inherited_path)
        .env("RUSTUP_TOOLCHAIN", "missing-runtime-toolchain")
        .env("RUSTFLAGS", "--runtime-must-not-inherit-this")
        .output()
        .expect("run isolated child with hostile inherited PATH");

    assert!(
        child.status.success(),
        "hostile PATH child failed; stdout: {}; stderr: {}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
    assert!(!marker.exists(), "fake cargo must never run");
}

#[cfg(unix)]
#[test]
fn read_set_under_a_parent_symlink_is_not_reusable() {
    use std::os::unix::fs::symlink;

    let root = repository();
    let external = tempdir("read-set-symlink-target");
    fs::write(external.path().join("input.txt"), "external bytes\n").expect("external input");
    symlink(external.path(), root.path().join("linked"))
        .expect("symlink parent into external directory");
    run(root.path(), &["add", "linked"]);
    run(root.path(), &["commit", "-qm", "add linked input"]);
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("read-set-symlink-state");
    let mut external_read = command("external-read", "cat", &["linked/input.txt"]);
    external_read.input_paths = vec!["linked/input.txt".into()];
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![external_read],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );

    let first = run_composition(composition.clone()).expect("first composition");
    let second = run_composition(composition).expect("second composition");

    assert!(first.passed);
    assert!(second.passed);
    assert_eq!(second.processes_spawned, 1);
    assert!(second.execution_records[0].spawned);
    assert!(!second.execution_records[0].reused);
}

#[cfg(unix)]
#[test]
fn persistence_error_after_worktree_creation_cleans_the_temporary_worktree() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("composition-write-failure-state");
    let backup = state.path().with_file_name(format!(
        "{}-backup",
        state.path().file_name().unwrap().to_string_lossy()
    ));
    let worktrees_before = run(root.path(), &["worktree", "list", "--porcelain"]);
    let mut sabotage = command(
        "sabotage-state-store",
        "sh",
        &[
            "-c",
            "mv \"$COMPOSITION_STATE_DIR\" \"$COMPOSITION_STATE_BACKUP\" && touch \"$COMPOSITION_STATE_DIR\"",
        ],
    );
    sabotage.environment.insert(
        "COMPOSITION_STATE_DIR".into(),
        state.path().to_string_lossy().into_owned(),
    );
    sabotage.environment.insert(
        "COMPOSITION_STATE_BACKUP".into(),
        backup.to_string_lossy().into_owned(),
    );

    let result = run_composition(input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![sabotage],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ));
    let worktrees_after = run(root.path(), &["worktree", "list", "--porcelain"]);
    let root_identity = fs::canonicalize(root.path()).expect("canonical test repository");
    let leaked_worktrees = worktrees_after
        .lines()
        .filter_map(|line| line.strip_prefix("worktree "))
        .filter_map(|worktree| fs::canonicalize(worktree).ok())
        .filter(|worktree| worktree != &root_identity)
        .collect::<Vec<_>>();

    for worktree in &leaked_worktrees {
        assert_eq!(
            worktree.file_name().and_then(|name| name.to_str()),
            Some("composition"),
            "only the uniquely named test composition worktree may be cleaned"
        );
        let parent = worktree.parent().expect("composition parent");
        let temp_root = fs::canonicalize(std::env::temp_dir()).expect("canonical temp root");
        assert_eq!(
            parent.parent(),
            Some(temp_root.as_path()),
            "composition cleanup is limited to a direct child of the temp root"
        );
        assert!(
            parent
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("ai-cockpit-composition-")),
            "composition parent must use the Runtime's unique prefix"
        );
        let _ = Command::new("git")
            .args(["worktree", "remove", "--force"])
            .arg(worktree)
            .current_dir(root.path())
            .status();
        fs::remove_dir_all(parent).expect("remove only the isolated composition parent");
    }
    if state.path().is_file() {
        fs::remove_file(state.path()).expect("remove sabotaging state file");
    }
    if backup.exists() {
        fs::remove_dir_all(&backup).expect("remove isolated state backup");
    }

    assert!(result.is_err(), "persistence failure should be preserved");
    assert!(
        leaked_worktrees.is_empty(),
        "temporary worktree leaked after a state persistence failure: {leaked_worktrees:?}"
    );
    assert_eq!(worktrees_before, worktrees_after);
}

#[test]
fn retry_does_not_remove_a_worktree_owned_by_a_live_process() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("live-owner-state");
    let mut composition = input(
        root.path(),
        state.path(),
        binding(&base, vec![base.clone(), base.clone()]),
        vec![command("slow-owner", "sh", &["-c", "sleep 2"])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    composition.timeout_seconds = 10;
    let root_path = root.path().to_path_buf();
    let state_path = state.path().to_path_buf();
    let worker = std::thread::spawn(move || run_composition(composition.clone()));

    let deadline = Instant::now() + Duration::from_secs(8);
    let (attempt_path, worktree_path) = loop {
        let attempt = fs::read_dir(&state_path)
            .expect("state directory")
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "json")
            })
            .find_map(|path| {
                let value: serde_json::Value =
                    serde_json::from_slice(&fs::read(&path).ok()?).ok()?;
                let worktree = value["isolatedWorktree"].as_str()?.to_owned();
                (!worktree.is_empty()).then_some((path, PathBuf::from(worktree)))
            });
        if let Some((attempt_path, worktree)) = attempt {
            let registered = run(&root_path, &["worktree", "list", "--porcelain"])
                .lines()
                .any(|line| {
                    line.strip_prefix("worktree ").is_some_and(|path| {
                        fs::canonicalize(path).ok() == fs::canonicalize(&worktree).ok()
                    })
                });
            if registered {
                break (attempt_path, worktree);
            }
        }
        assert!(
            Instant::now() < deadline,
            "live attempt never registered a worktree"
        );
        std::thread::sleep(Duration::from_millis(10));
    };

    let retry = input(
        root.path(),
        state.path(),
        binding(&base, vec![base.clone(), base.clone()]),
        vec![command("retry", "true", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let error = run_composition(retry).expect_err("a live owner must block recovery");
    assert!(matches!(
        error,
        CompositionError::ActiveAttempt { owner_pid, .. } if owner_pid == std::process::id()
    ));
    assert!(worktree_path.is_dir(), "live owner's worktree was removed");
    assert!(attempt_path.is_file(), "live owner's attempt was replaced");

    let finished = worker
        .join()
        .expect("owner thread")
        .expect("composition result");
    assert!(finished.passed, "owner attempt failed: {finished:?}");
    assert!(
        !worktree_path.exists(),
        "owner worktree should clean after completion"
    );
}

#[cfg(unix)]
#[test]
fn retry_reconciles_an_interrupted_owner_after_process_exit() {
    if std::env::var_os("COMPOSITION_CRASH_CHILD").is_some() {
        let root = PathBuf::from(std::env::var_os("COMPOSITION_CRASH_ROOT").expect("root"));
        let state = PathBuf::from(std::env::var_os("COMPOSITION_CRASH_STATE").expect("state"));
        let base = run(&root, &["rev-parse", "refs/heads/main"]);
        let state_arg = state.to_string_lossy().into_owned();
        let _ = run_composition(input(
            &root,
            &state,
            binding(&base, vec![base.clone(), base.clone()]),
            vec![command(
                "terminate-owner",
                "sh",
                &[
                    "-c",
                    "while ! /usr/bin/grep -Eq '\"activeProcessGroupId\"[[:space:]]*:[[:space:]]*[0-9]+' \"$1\"/*.json; do /bin/sleep 0.01; done; /usr/bin/python3 -c 'import os,sys,time; error=None\ntry:\n held=open(os.path.join(os.getcwd(),\"README.md\"),\"rb\"); os.setsid(); os.chdir(\"/\")\nexcept Exception as exc:\n error=repr(exc); os.chdir(\"/\")\nopen(sys.argv[1]+\".error\",\"w\").write(error or \"none\"); open(sys.argv[1],\"w\").write(str(os.getpid())); time.sleep(60)' \"$1/escaped.pid\" >/dev/null 2>&1 & while [ ! -s \"$1/escaped.pid\" ]; do /bin/sleep 0.01; done; kill -9 \"$PPID\"",
                    "sh",
                    &state_arg,
                ],
            )],
            vec![CompositionPrecondition::satisfied("identity-bound")],
        ));
        panic!("composition owner should have been terminated by its child command");
    }

    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("crash-recovery-state");
    let mut child = Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "retry_reconciles_an_interrupted_owner_after_process_exit",
        ])
        .env("COMPOSITION_CRASH_CHILD", "1")
        .env("COMPOSITION_CRASH_ROOT", root.path())
        .env("COMPOSITION_CRASH_STATE", state.path())
        .spawn()
        .expect("spawn killable composition owner");
    let owner_pid = child.id();

    let deadline = Instant::now() + Duration::from_secs(8);
    let status = loop {
        if let Some(status) = child.try_wait().expect("check owner process") {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "composition owner did not terminate"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(
        !status.success(),
        "the child command should terminate its owner"
    );

    let attempt_path = fs::read_dir(state.path())
        .expect("state directory")
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .expect("durable interrupted attempt");
    let mut interrupted: serde_json::Value =
        serde_json::from_slice(&fs::read(&attempt_path).expect("attempt bytes"))
            .expect("attempt JSON");
    assert_eq!(interrupted["ownerPid"], owner_pid);
    assert_eq!(
        interrupted["activeExecutionNode"], "terminate-owner",
        "interrupted verifier identity must remain durable"
    );
    assert!(
        interrupted["activeProcessGroupId"].as_u64().is_some(),
        "interrupted verifier process group must remain durable"
    );
    let worktree = PathBuf::from(
        interrupted["isolatedWorktree"]
            .as_str()
            .expect("durable worktree path"),
    );
    assert!(!worktree.as_os_str().is_empty());
    assert!(
        worktree.is_dir(),
        "interrupted worktree should remain for recovery"
    );

    let observed_attempt = interrupted.clone();
    let interrupted_object = interrupted
        .as_object_mut()
        .expect("interrupted attempt object");
    interrupted_object.remove("processObservationSchemaVersion");
    interrupted_object.remove("activeExecutionNode");
    interrupted_object.remove("activeProcessGroupId");
    fs::write(
        &attempt_path,
        serde_json::to_vec_pretty(&interrupted).expect("serialize legacy attempt"),
    )
    .expect("simulate pre-observer interrupted attempt");
    let legacy_retry = run_composition(input(
        root.path(),
        state.path(),
        binding(&base, vec![base.clone(), base.clone()]),
        Vec::new(),
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ));
    assert!(
        matches!(
            legacy_retry,
            Err(CompositionError::UnknownAttemptOwner { .. })
        ),
        "legacy interrupted attempts without process observations must fail closed"
    );
    assert!(
        worktree.is_dir(),
        "unknown legacy process state must preserve its worktree"
    );
    fs::write(
        &attempt_path,
        serde_json::to_vec_pretty(&observed_attempt).expect("restore observed attempt"),
    )
    .expect("restore process observations");

    let escaped_process_id = fs::read_to_string(state.path().join("escaped.pid"))
        .expect("detached verifier descendant pid")
        .trim()
        .parse::<u32>()
        .expect("detached verifier descendant process id");
    assert_eq!(
        fs::read_to_string(state.path().join("escaped.pid.error"))
            .expect("detached verifier setup result"),
        "none",
        "detached verifier must open a worktree file and leave its working directory"
    );
    struct DetachedProcessGuard(u32);
    impl Drop for DetachedProcessGuard {
        fn drop(&mut self) {
            unsafe {
                libc::kill(self.0 as libc::pid_t, libc::SIGKILL);
            }
        }
    }
    let _escaped_process_guard = DetachedProcessGuard(escaped_process_id);
    let blocked = run_composition(input(
        root.path(),
        state.path(),
        binding(&base, vec![base.clone(), base.clone()]),
        Vec::new(),
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ));
    assert!(
        matches!(
            blocked,
            Err(CompositionError::ActiveVerifierDescendant { process_id, .. })
                if process_id == escaped_process_id
        ),
        "a verifier that escaped its original process group must still protect the worktree: {blocked:?}"
    );
    assert!(
        worktree.is_dir(),
        "live detached verifier lost its worktree"
    );
    unsafe {
        libc::kill(escaped_process_id as libc::pid_t, libc::SIGKILL);
    }
    let escaped_deadline = Instant::now() + Duration::from_secs(5);
    while unsafe { libc::kill(escaped_process_id as libc::pid_t, 0) } == 0 {
        assert!(
            Instant::now() < escaped_deadline,
            "detached verifier did not exit after termination"
        );
        std::thread::sleep(Duration::from_millis(10));
    }

    let retry = run_composition(input(
        root.path(),
        state.path(),
        binding(&base, vec![base.clone(), base.clone()]),
        vec![command("recovered", "true", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    ))
    .expect("retry reconciles the dead owner");

    assert!(retry.passed, "recovered attempt failed: {retry:?}");
    assert_eq!(retry.processes_spawned, 1);
    interrupted = serde_json::from_slice(&fs::read(&attempt_path).expect("reconciled attempt"))
        .expect("reconciled attempt JSON");
    assert_eq!(interrupted["failure"], "interrupted_owner_terminated");
    assert_eq!(interrupted["cleanup"]["removed"], true);
    assert!(
        !worktree.exists(),
        "abandoned worktree must be removed on retry"
    );
    assert!(
        !run(root.path(), &["worktree", "list", "--porcelain"])
            .lines()
            .any(|line| line.strip_prefix("worktree ").is_some_and(|path| {
                fs::canonicalize(path).ok() == fs::canonicalize(&worktree).ok()
            }))
    );
}

#[cfg(unix)]
#[test]
fn retry_preserves_worktree_while_orphan_verifier_process_group_is_alive() {
    if std::env::var_os("COMPOSITION_ORPHAN_CHILD").is_some() {
        let root = PathBuf::from(std::env::var_os("COMPOSITION_ORPHAN_ROOT").expect("root"));
        let state = PathBuf::from(std::env::var_os("COMPOSITION_ORPHAN_STATE").expect("state"));
        let pid_file =
            PathBuf::from(std::env::var_os("COMPOSITION_ORPHAN_PID_FILE").expect("pid file"));
        let base = run(&root, &["rev-parse", "refs/heads/main"]);
        let state_arg = state.to_string_lossy().into_owned();
        let pid_file_arg = pid_file.to_string_lossy().into_owned();
        let script = "printf '%s\\n' \"$$\" > \"$2\"; while ! /usr/bin/grep -Eq '\"activeProcessGroupId\"[[:space:]]*:[[:space:]]*[0-9]+' \"$1\"/*.json; do /bin/sleep 0.01; done; kill -9 \"$PPID\"; exec /bin/sleep 60";
        let _ = run_composition(input(
            &root,
            &state,
            binding(&base, vec![base.clone(), base.clone()]),
            vec![command(
                "orphan-verifier",
                "sh",
                &["-c", script, "sh", &state_arg, &pid_file_arg],
            )],
            vec![CompositionPrecondition::satisfied("identity-bound")],
        ));
        panic!("the verifier command should terminate its composition owner");
    }

    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("orphan-verifier-state");
    let pid_file = state.path().join("orphan-verifier.pid");
    let mut child = Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "retry_preserves_worktree_while_orphan_verifier_process_group_is_alive",
        ])
        .env("COMPOSITION_ORPHAN_CHILD", "1")
        .env("COMPOSITION_ORPHAN_ROOT", root.path())
        .env("COMPOSITION_ORPHAN_STATE", state.path())
        .env("COMPOSITION_ORPHAN_PID_FILE", &pid_file)
        .spawn()
        .expect("spawn killable composition owner");
    let owner_pid = child.id();
    let deadline = Instant::now() + Duration::from_secs(20);
    let status = loop {
        if let Some(status) = child.try_wait().expect("check owner process") {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "composition owner did not terminate"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(
        !status.success(),
        "verifier command should terminate its owner"
    );

    let attempt_path = fs::read_dir(state.path())
        .expect("state directory")
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .expect("durable interrupted attempt");
    let interrupted: serde_json::Value =
        serde_json::from_slice(&fs::read(&attempt_path).expect("attempt bytes"))
            .expect("attempt JSON");
    assert_eq!(interrupted["ownerPid"], owner_pid);
    let process_group_id = interrupted["activeProcessGroupId"]
        .as_u64()
        .expect("durable active verifier process group") as u32;
    assert_eq!(
        fs::read_to_string(&pid_file)
            .expect("verifier pid file")
            .trim(),
        process_group_id.to_string()
    );
    let worktree = PathBuf::from(
        interrupted["isolatedWorktree"]
            .as_str()
            .expect("durable worktree path"),
    );
    assert!(
        worktree.is_dir(),
        "interrupted worktree must remain recoverable"
    );

    struct ProcessGroupGuard(u32);
    impl Drop for ProcessGroupGuard {
        fn drop(&mut self) {
            unsafe {
                libc::kill(-(self.0 as libc::pid_t), libc::SIGKILL);
            }
        }
    }
    let process_group_guard = ProcessGroupGuard(process_group_id);
    let retry_input = input(
        root.path(),
        state.path(),
        binding(&base, vec![base.clone(), base.clone()]),
        Vec::new(),
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let blocked = run_composition(retry_input.clone());
    assert!(matches!(
        blocked,
        Err(CompositionError::ActiveVerifierProcessGroup {
            process_group_id: active_group,
            ..
        }) if active_group == process_group_id
    ));
    assert!(
        worktree.is_dir(),
        "live verifier group worktree must be preserved"
    );

    unsafe {
        libc::kill(-(process_group_id as libc::pid_t), libc::SIGKILL);
    }
    let group_deadline = Instant::now() + Duration::from_secs(5);
    while unsafe { libc::kill(-(process_group_id as libc::pid_t), 0) } == 0 {
        assert!(
            Instant::now() < group_deadline,
            "verifier process group did not exit"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    drop(process_group_guard);

    let _ = run_composition(retry_input).expect("retry reconciles after verifier exit");
    let reconciled: serde_json::Value =
        serde_json::from_slice(&fs::read(&attempt_path).expect("reconciled attempt"))
            .expect("reconciled attempt JSON");
    assert_eq!(reconciled["cleanup"]["removed"], true);
    assert!(
        !worktree.exists(),
        "dead verifier worktree should be cleaned on retry"
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
            command("stable", "true", &[]),
            command("changed", "true", &[]),
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
            command("stable", "true", &[]),
            command("changed", "false", &[]),
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
    let mut api = command("api", "true", &[]);
    api.input_paths = vec!["api.txt".into()];
    let mut docs = command("docs", "true", &[]);
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
    let mut source = command("source", "cat", &["api.txt"]);
    source.input_paths = vec!["api.txt".into()];
    let mut consumer = command("consumer", "true", &[]);
    consumer.depends_on = vec!["source".into()];
    let mut transitive = command("transitive", "true", &[]);
    transitive.depends_on = vec!["consumer".into()];
    let mut independent = command("independent", "true", &[]);
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
fn inherited_environment_does_not_enter_runtime_child_or_invalidate_reuse() {
    let child_mode = std::env::var_os("COMPOSITION_ENV_CHILD").is_some();
    if child_mode {
        let root = PathBuf::from(std::env::var_os("COMPOSITION_ENV_ROOT").expect("root path"));
        let state = PathBuf::from(std::env::var_os("COMPOSITION_ENV_STATE").expect("state path"));
        let base = run(&root, &["rev-parse", "refs/heads/main"]);
        let check = command("inherited-environment-check", "env", &[]);
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
                "inherited_environment_does_not_enter_runtime_child_or_invalidate_reuse",
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
        attempts[1]["processesSpawned"], 0,
        "an unrelated inherited variable is absent from the controlled child environment"
    );
    assert_eq!(attempts[1]["executionRecords"][0]["reused"], true);
    assert!(
        !attempts[1]["executionRecords"][0]["stdout"]
            .as_str()
            .unwrap_or_default()
            .contains("COMPOSITION_EXTERNAL_FLAVOR")
    );
}

#[cfg(unix)]
#[test]
fn untrusted_absolute_executable_fails_before_spawn() {
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
    let attempt = run_composition(first_input).expect("attempt rejects untrusted executable");
    assert!(!attempt.passed);
    assert_eq!(attempt.processes_spawned, 0);
    assert!(attempt.execution_records.is_empty());
    assert_eq!(
        attempt.failure.as_deref(),
        Some("composition_executable_unbound")
    );
}

#[cfg(unix)]
#[test]
fn unbound_command_path_override_fails_before_spawn() {
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
    let attempt = run_composition(first_input).expect("attempt rejects PATH overlay");
    assert!(!attempt.passed);
    assert_eq!(attempt.processes_spawned, 0);
    assert!(attempt.execution_records.is_empty());
    assert_eq!(
        attempt.failure.as_deref(),
        Some("unbound_runtime_environment_override")
    );
}

#[test]
fn unbounded_external_reads_execute_again_but_independent_node_reuses() {
    let root = repository();
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let state = tempdir("unbounded-read-state");
    let external = tempdir("unbounded-read-source");
    let source = external.path().join("source.txt");
    fs::write(&source, "version-one\n").expect("write external source");
    let mut check = command("external-read", "cat", &[source.to_str().unwrap()]);
    // This is deliberately not the file read by `cat`; a caller-supplied path
    // list cannot make an unbounded command read-set complete.
    check.input_paths = vec!["README.md".into()];
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![check, command("independent", "true", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let original_json = serde_json::to_vec(&composition).expect("serialize composition input");

    let first = run_composition(composition.clone()).expect("first attempt");
    fs::write(&source, "version-two\n").expect("change external source without editing input");
    assert_eq!(
        serde_json::to_vec(&composition).expect("serialize unchanged composition input"),
        original_json
    );
    let second = run_composition(composition).expect("second attempt");

    assert!(first.passed && second.passed);
    assert_eq!(first.processes_spawned, 2);
    assert_eq!(second.processes_spawned, 1);
    assert!(!second.execution_records[0].reused);
    assert!(second.execution_records[1].reused);
    assert!(second.execution_records[1].node_id == "independent");
}

#[cfg(unix)]
#[test]
fn relative_executable_uses_isolated_worktree_bytes_and_changes_identity() {
    use std::os::unix::fs::PermissionsExt;

    let root = repository();
    let state = tempdir("relative-executable-state");
    let executable = root.path().join("tools/check.sh");
    fs::create_dir_all(executable.parent().expect("tools parent")).expect("create tools directory");
    fs::write(&executable, "#!/bin/sh\nexit 0\n").expect("write initial executable");
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).expect("make executable");
    run(root.path(), &["add", "tools/check.sh"]);
    run(root.path(), &["commit", "-qm", "add relative executable"]);
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let check = command("relative-check", "./tools/check.sh", &[]);
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![check],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );

    let first = run_composition(composition.clone()).expect("first attempt");
    assert!(first.passed, "first attempt: {first:?}");
    assert_ne!(
        first.identity.toolchain_digest,
        CompositionIdentity::default().toolchain_digest,
        "the Runtime must hash the executable resolved from the isolated worktree"
    );

    fs::write(&executable, "#!/bin/sh\nexit 19\n").expect("replace committed executable bytes");
    run(root.path(), &["add", "tools/check.sh"]);
    run(
        root.path(),
        &["commit", "-qm", "change relative executable"],
    );
    let next_head = run(root.path(), &["rev-parse", "HEAD"]);
    let mut second_input = composition;
    second_input.binding.target_sha = next_head.clone();
    second_input.binding.participant_heads = vec![next_head.clone(), next_head];
    let second = run_composition(second_input).expect("second attempt");

    assert!(!second.passed);
    assert_eq!(second.processes_spawned, 1);
    assert_eq!(second.execution_records[0].exit_code, Some(19));
    assert_ne!(
        second.identity.toolchain_digest,
        first.identity.toolchain_digest
    );
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

#[cfg(unix)]
#[test]
fn installed_rust_toolchain_change_with_stale_json_reexecutes_reusable_node() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let home = tempdir("rustup-home");
    let proxy_dir = home.path().join(".cargo/bin");
    fs::create_dir_all(&proxy_dir).expect("create proxy directory");
    let rustup = proxy_dir.join("rustup");
    fs::write(&rustup, "#!/bin/sh\nexit 0\n").expect("write rustup proxy");
    fs::set_permissions(&rustup, fs::Permissions::from_mode(0o755))
        .expect("make rustup proxy executable");
    symlink("rustup", proxy_dir.join("cargo")).expect("create cargo proxy");

    let toolchain = home.path().join(".rustup/toolchains/test-channel");
    let toolchain_bin = toolchain.join("bin");
    fs::create_dir_all(&toolchain_bin).expect("create fake toolchain");
    fs::write(toolchain_bin.join("cargo"), b"cargo-v1").expect("write cargo identity");
    fs::write(toolchain_bin.join("rustc"), b"rustc-v1").expect("write rustc identity");
    let rustlib = toolchain.join("lib/rustlib");
    fs::create_dir_all(&rustlib).expect("create rustlib metadata");
    fs::write(rustlib.join("components"), b"rustc\nrust-std\n")
        .expect("write installed components");
    fs::write(rustlib.join("manifest-test-channel"), b"toolchain-v1")
        .expect("write toolchain manifest");
    let rustup_home = home.path().join(".rustup");
    fs::create_dir_all(&rustup_home).expect("create rustup home");
    fs::write(
        rustup_home.join("settings.toml"),
        "default_toolchain = \"test-channel\"\n[overrides]\n",
    )
    .expect("write rustup settings");

    let root = repository();
    let state = tempdir("rustup-state");
    let base = run(root.path(), &["rev-parse", "HEAD"]);
    let composition = input(
        root.path(),
        state.path(),
        binding(&base.clone(), vec![base.clone(), base]),
        vec![command("toolchain-change", "true", &[])],
        vec![CompositionPrecondition::satisfied("identity-bound")],
    );
    let unchanged_input = serde_json::to_vec(&composition).expect("serialize stable input");
    let input_path = state.path().join("composition.input");
    fs::write(&input_path, unchanged_input).expect("write stable input JSON");

    let output = Command::new(std::env::current_exe().expect("current test executable"))
        .args(["--exact", "installed_toolchain_change_child", "--nocapture"])
        .env("HOME", home.path())
        .env("COCKPIT_TOOLCHAIN_TEST_INPUT", &input_path)
        .env("COCKPIT_TOOLCHAIN_TEST_INSTALL", &toolchain)
        .output()
        .expect("run isolated child test process");
    assert!(
        output.status.success(),
        "isolated toolchain test failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(unix)]
#[test]
fn installed_toolchain_change_child() {
    let Ok(input_path) = std::env::var("COCKPIT_TOOLCHAIN_TEST_INPUT") else {
        return;
    };
    let toolchain = PathBuf::from(
        std::env::var("COCKPIT_TOOLCHAIN_TEST_INSTALL").expect("toolchain install path"),
    );
    let composition: CompositionInput =
        serde_json::from_slice(&fs::read(input_path).expect("read unchanged composition JSON"))
            .expect("decode composition JSON");

    let first = run_composition(composition.clone()).expect("first toolchain attempt");
    assert!(first.passed, "first attempt: {first:?}");
    assert_eq!(first.processes_spawned, 1);
    fs::write(toolchain.join("bin/rustc"), b"rustc-v2")
        .expect("update installed rustc without changing composition JSON");

    let second = run_composition(composition).expect("second toolchain attempt");
    assert!(second.passed, "second attempt: {second:?}");
    assert_eq!(second.processes_spawned, 1, "changed toolchain must rerun");
    assert!(!second.execution_records[0].reused);
    assert_ne!(
        first.identity.toolchain_digest,
        second.identity.toolchain_digest
    );
}

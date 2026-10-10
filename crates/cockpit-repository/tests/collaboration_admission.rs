use cockpit_core::Digest;
use cockpit_git::GitRepository;
use cockpit_protocol::RuntimeContext;
use cockpit_protocol::{
    CollaborationDeclaration, CompositionBinding, ConsumedOutcome, CoordinationEvent,
    CoordinationEventKind, CoordinationIntent, CoordinationRequest, CoordinationRequestState,
    ENVIRONMENT_DRIFT_CAPABILITY, IntegrationResponsibility, OutcomeStage, ProvidedOutcome,
    RuntimeCapabilityBinding, WorktreeRegistration,
};
use cockpit_repository::{
    CollaborationAction, CollaborationActionKind, CoordinationError, CoordinationStore,
    WorkItemStartOptions, acknowledge_pause, admit_collaboration_action, attach,
    checkpoint_work_item, collaboration_outcome_projection, collaboration_projection,
    preflight_work_item, publish_outcome, record_verification, recover_impact,
    refresh_dependency_state, report_impact, repository_id, request_safe_pause,
    resume_and_re_evaluate, start_work_item_with_options,
};
use cockpit_verification::{
    CompositionCommand, CompositionIdentity, CompositionInput, CompositionPrecondition,
    VerificationCommand, VerificationReusePolicy, composition_commands_digest, execute_bounded,
};
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
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
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        runtime_digest: runtime_digest(),
        capability: ENVIRONMENT_DRIFT_CAPABILITY.into(),
    }
}

fn runtime_digest() -> Digest {
    let executable = std::env::current_exe().expect("controlled test helper path");
    Digest::sha256_bytes(&fs::read(executable).expect("controlled test helper bytes"))
}

fn run_admitted_composition(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    input: CompositionInput,
) -> Result<cockpit_verification::CompositionAttempt, cockpit_repository::CollaborationExecutionError>
{
    run_admitted_composition_with_supervisor_mode(store, work_item_id, generation, input, "run")
}

fn run_admitted_composition_with_supervisor_mode(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    input: CompositionInput,
    mode: &str,
) -> Result<cockpit_verification::CompositionAttempt, cockpit_repository::CollaborationExecutionError>
{
    let executable = std::env::current_exe().expect("controlled test helper path");
    let mut extra_environment = vec![(
        "AI_COCKPIT_COMPOSITION_SUPERVISOR_TEST_HELPER".into(),
        mode.into(),
    )];
    if mode == "revoke-generation-before-ready" {
        extra_environment.push((
            "AI_COCKPIT_COMPOSITION_REVOKE_ROOT".into(),
            input.repository_root.to_string_lossy().into_owned(),
        ));
        extra_environment.push((
            "AI_COCKPIT_COMPOSITION_REVOKE_WORK_ITEM".into(),
            work_item_id.into(),
        ));
    }
    run_admitted_composition_with_supervisor_environment(
        store,
        work_item_id,
        generation,
        input,
        &executable,
        &extra_environment,
    )
}

fn run_admitted_composition_with_supervisor_environment(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    input: CompositionInput,
    executable: &Path,
    extra_environment: &[(String, String)],
) -> Result<cockpit_verification::CompositionAttempt, cockpit_repository::CollaborationExecutionError>
{
    cockpit_repository::run_admitted_composition_with_supervisor_executable(
        store,
        work_item_id,
        generation,
        input,
        executable,
        &[
            "--exact".into(),
            "composition_supervisor_test_helper_entry".into(),
            "--nocapture".into(),
        ],
        extra_environment,
    )
}

#[cfg(target_os = "linux")]
struct WorktreeRemoveFault {
    repository_root: PathBuf,
    wrapper: PathBuf,
    supervisor_environment: Vec<(String, String)>,
    target_worktree_path_file: PathBuf,
    injected_worktree_path_file: PathBuf,
}

#[cfg(target_os = "linux")]
struct PreserveTempDirOnPanic(Option<tempfile::TempDir>);

#[cfg(target_os = "linux")]
impl PreserveTempDirOnPanic {
    fn new(tempdir: tempfile::TempDir) -> Self {
        Self(Some(tempdir))
    }

    fn path(&self) -> &Path {
        self.0.as_ref().expect("fixture owner retained").path()
    }
}

#[cfg(target_os = "linux")]
impl Drop for PreserveTempDirOnPanic {
    fn drop(&mut self) {
        if std::thread::panicking()
            && let Some(tempdir) = self.0.take()
        {
            let path = tempdir.keep();
            eprintln!("preserved_test_fixture_owner={}", path.display());
        }
    }
}

#[cfg(target_os = "linux")]
fn persist_composition_attempt_snapshot(
    repository_root: &Path,
    name: &str,
    attempt: &cockpit_verification::CompositionAttempt,
) -> PathBuf {
    let directory = repository_root
        .join("target")
        .join("collaboration-admission-attempts");
    fs::create_dir_all(&directory).expect("create composition attempt snapshot directory");
    let path = directory.join(format!("{name}.json"));
    fs::write(
        &path,
        serde_json::to_vec_pretty(attempt).expect("serialize complete composition attempt"),
    )
    .expect("persist complete composition attempt snapshot");
    path
}

#[cfg(target_os = "linux")]
fn fail_first_worktree_remove_environment(root: &Path) -> WorktreeRemoveFault {
    use std::os::unix::fs::PermissionsExt;

    let actual_git = std::env::split_paths(&std::env::var_os("PATH").expect("test PATH"))
        .map(|directory| directory.join("git"))
        .find(|path| path.is_file())
        .expect("resolve actual git executable");
    let wrapper_dir = root.join("test-bin");
    fs::create_dir_all(&wrapper_dir).expect("create Git wrapper directory");
    let wrapper = wrapper_dir.join("git");
    let target_worktree_path_file = root.join(".injected-worktree-remove-target");
    let failure_marker = root.join(".injected-worktree-remove-failure-used");
    let script = format!(
        "#!/bin/sh\nif [ \"$1\" = -C ] && [ \"$3\" = worktree ] && [ \"$4\" = add ] && [ \"$5\" = --detach ] && [ ! -e '{}' ]; then\n  printf '%s\\n' \"$6\" > '{}' || exit 98\nfi\nif [ \"$1\" = -C ] && [ \"$3\" = worktree ] && [ \"$4\" = remove ] && [ -e '{}' ] && [ \"$6\" = \"$(cat '{}')\" ] && [ ! -e '{}' ]; then\n  printf '%s\\n' \"$6\" > '{}' || exit 98\n  echo 'injected one-time worktree cleanup refusal' >&2\n  exit 1\nfi\nexec '{}' \"$@\"\n",
        target_worktree_path_file.display(),
        target_worktree_path_file.display(),
        target_worktree_path_file.display(),
        target_worktree_path_file.display(),
        failure_marker.display(),
        failure_marker.display(),
        actual_git.display(),
    );
    fs::write(&wrapper, script).expect("write Git wrapper");
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755))
        .expect("make Git wrapper executable");
    let mut path_entries = vec![wrapper_dir];
    path_entries.extend(std::env::split_paths(
        &std::env::var_os("PATH").expect("test PATH"),
    ));
    let path = std::env::join_paths(path_entries)
        .expect("compose controlled supervisor PATH")
        .into_string()
        .expect("supervisor PATH UTF-8");
    WorktreeRemoveFault {
        repository_root: root.to_path_buf(),
        wrapper,
        supervisor_environment: vec![
            (
                "AI_COCKPIT_COMPOSITION_SUPERVISOR_TEST_HELPER".into(),
                "run".into(),
            ),
            ("PATH".into(), path),
        ],
        target_worktree_path_file,
        injected_worktree_path_file: failure_marker,
    }
}

#[cfg(target_os = "linux")]
fn wrapped_worktree_add(
    wrapper: &Path,
    repository: &Path,
    worktree: &Path,
    target_sha: &str,
) -> std::process::Output {
    Command::new(wrapper)
        .arg("-C")
        .arg(repository)
        .args(["worktree", "add", "--detach"])
        .arg(worktree)
        .arg(target_sha)
        .output()
        .expect("run controlled Git worktree add")
}

#[cfg(target_os = "linux")]
fn wrapped_worktree_remove(
    wrapper: &Path,
    repository: &Path,
    worktree: &Path,
) -> std::process::Output {
    Command::new(wrapper)
        .arg("-C")
        .arg(repository)
        .args(["worktree", "remove", "--force"])
        .arg(worktree)
        .output()
        .expect("run controlled Git worktree remove")
}

#[cfg(target_os = "linux")]
#[test]
fn worktree_remove_injection_is_consumed_once_for_old_worktree() {
    let root = PreserveTempDirOnPanic::new(repository());
    let fault = fail_first_worktree_remove_environment(root.path());
    let wrapper = &fault.wrapper;
    let head = GitRepository::discover(root.path())
        .expect("discover repository")
        .topology()
        .expect("repository topology")
        .head
        .expect("repository HEAD");
    let worktree_parent = root.path().join("worktrees");
    fs::create_dir_all(&worktree_parent).expect("worktree parent");
    let old_worktree = worktree_parent.join("old-worktree");
    let fresh_worktree = worktree_parent.join("fresh-worktree");

    let old_add = wrapped_worktree_add(wrapper, root.path(), &old_worktree, &head);
    assert!(
        old_add.status.success(),
        "old worktree add: {}",
        String::from_utf8_lossy(&old_add.stderr)
    );
    assert_eq!(
        fs::read_to_string(&fault.target_worktree_path_file)
            .expect("old add captures target path")
            .trim(),
        old_worktree.to_string_lossy()
    );
    let old_remove = wrapped_worktree_remove(wrapper, root.path(), &old_worktree);
    assert!(!old_remove.status.success());
    assert!(
        String::from_utf8_lossy(&old_remove.stderr)
            .contains("injected one-time worktree cleanup refusal")
    );
    assert_eq!(
        fs::read_to_string(&fault.injected_worktree_path_file)
            .expect("consumed injection marker")
            .trim(),
        old_worktree.to_string_lossy()
    );

    let fresh_add = wrapped_worktree_add(wrapper, root.path(), &fresh_worktree, &head);
    assert!(
        fresh_add.status.success(),
        "fresh worktree add: {}",
        String::from_utf8_lossy(&fresh_add.stderr)
    );
    let fresh_remove = wrapped_worktree_remove(wrapper, root.path(), &fresh_worktree);
    assert!(
        fresh_remove.status.success(),
        "fresh worktree must not receive the old worktree's injection: {}",
        String::from_utf8_lossy(&fresh_remove.stderr)
    );
    assert!(old_worktree.is_dir(), "the old worktree remains untouched");
    assert!(!fresh_worktree.exists(), "the fresh worktree was cleaned");

    run(
        root.path(),
        &[
            "worktree",
            "remove",
            "--force",
            old_worktree.to_str().expect("UTF-8 worktree path"),
        ],
    );
}

#[cfg(target_os = "linux")]
#[test]
fn unconsumed_worktree_remove_injection_does_not_move_to_fresh_worktree() {
    let root = PreserveTempDirOnPanic::new(repository());
    let fault = fail_first_worktree_remove_environment(root.path());
    let wrapper = &fault.wrapper;
    let head = GitRepository::discover(root.path())
        .expect("discover repository")
        .topology()
        .expect("repository topology")
        .head
        .expect("repository HEAD");
    let worktree_parent = root.path().join("worktrees");
    fs::create_dir_all(&worktree_parent).expect("worktree parent");
    let old_worktree = worktree_parent.join("old-worktree");
    let fresh_worktree = worktree_parent.join("fresh-worktree");

    let old_add = wrapped_worktree_add(wrapper, root.path(), &old_worktree, &head);
    assert!(
        old_add.status.success(),
        "old worktree add: {}",
        String::from_utf8_lossy(&old_add.stderr)
    );
    let fresh_add = wrapped_worktree_add(wrapper, root.path(), &fresh_worktree, &head);
    assert!(
        fresh_add.status.success(),
        "fresh worktree add: {}",
        String::from_utf8_lossy(&fresh_add.stderr)
    );
    assert_eq!(
        fs::read_to_string(&fault.target_worktree_path_file)
            .expect("old add captures target path")
            .trim(),
        old_worktree.to_string_lossy(),
        "adding a fresh attempt cannot retarget the reserved injection"
    );

    // Model an old attempt deferred before it reaches Git cleanup: the one-shot
    // hook must remain reserved for that exact path, not migrate to this retry.
    let fresh_remove = wrapped_worktree_remove(wrapper, root.path(), &fresh_worktree);
    assert!(
        fresh_remove.status.success(),
        "an unconsumed old-worktree injection must not reject the fresh worktree: {}",
        String::from_utf8_lossy(&fresh_remove.stderr)
    );
    assert!(old_worktree.is_dir(), "the old worktree remains untouched");
    assert!(!fresh_worktree.exists(), "the fresh worktree was cleaned");
    assert!(
        !fault.injected_worktree_path_file.exists(),
        "the fresh worktree must not consume the old worktree's injection"
    );

    run(
        root.path(),
        &[
            "worktree",
            "remove",
            "--force",
            old_worktree.to_str().expect("UTF-8 worktree path"),
        ],
    );
}

#[cfg(target_os = "linux")]
fn fail_first_proc_cwd_observation_environment(
    root: &Path,
    peer_pid: u32,
) -> Vec<(String, String)> {
    let source = root.join("observer-fault.c");
    let library = root.join("observer-fault.so");
    let counter = root.join("observer-fault-count");
    fs::write(
        &source,
        r#"#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/file.h>
#include <sys/syscall.h>
#include <unistd.h>

ssize_t readlink(const char *path, char *buffer, size_t size) {
    const char *marker = getenv("AI_COCKPIT_OBSERVER_FAULT_COUNTER");
    const char *peer = getenv("AI_COCKPIT_OBSERVER_FAULT_PID");
    char expected[80];
    int expected_size = peer ? snprintf(expected, sizeof(expected), "/proc/%s/cwd", peer) : -1;
    if (marker && peer && expected_size > 0 &&
        (size_t)expected_size < sizeof(expected) && strcmp(path, expected) == 0) {
        int fd = open(marker, O_RDWR | O_CREAT, 0600);
        if (fd >= 0) {
            if (flock(fd, LOCK_EX) == 0) {
                char current = '0';
                (void)read(fd, &current, 1);
                if (current < '0' || current > '9') current = '0';
                int count = current - '0';
                (void)lseek(fd, 0, SEEK_SET);
                char next = (char)('0' + count + 1);
                (void)write(fd, &next, 1);
                (void)flock(fd, LOCK_UN);
                (void)close(fd);
                if (count < 1) { errno = EIO; return -1; }
            } else (void)close(fd);
        }
    }
    return syscall(SYS_readlink, path, buffer, size);
}
"#,
    )
    .expect("write controlled procfs fault shim");
    let compile = Command::new("cc")
        .args(["-shared", "-fPIC", "-o"])
        .arg(&library)
        .arg(&source)
        .output()
        .expect("compile controlled procfs fault shim");
    assert!(
        compile.status.success(),
        "C shim: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    vec![
        (
            "AI_COCKPIT_COMPOSITION_SUPERVISOR_TEST_HELPER".into(),
            "run".into(),
        ),
        ("LD_PRELOAD".into(), library.to_string_lossy().into_owned()),
        (
            "AI_COCKPIT_OBSERVER_FAULT_COUNTER".into(),
            counter.to_string_lossy().into_owned(),
        ),
        ("AI_COCKPIT_OBSERVER_FAULT_PID".into(), peer_pid.to_string()),
    ]
}

#[cfg(target_os = "linux")]
fn defer_successful_composition_cleanup(
    store: &CoordinationStore,
    input: CompositionInput,
    fault: &WorktreeRemoveFault,
) -> (
    cockpit_verification::CompositionAttempt,
    PathBuf,
    PathBuf,
    PathBuf,
) {
    let executable = std::env::current_exe().expect("controlled test helper path");
    let first = run_admitted_composition_with_supervisor_environment(
        store,
        "WI-CONSUMER",
        1,
        input.clone(),
        &executable,
        &fault.supervisor_environment,
    )
    .expect("first formal composition");
    persist_composition_attempt_snapshot(&fault.repository_root, "old-attempt", &first);
    let (attempt_path, attempt) = fs::read_dir(store.root().join("compositions"))
        .expect("composition attempts")
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if !path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                return None;
            }
            let bytes = fs::read(&path).ok()?;
            let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
            (value["attemptId"].as_str() == Some(first.attempt_id.as_str()))
                .then_some((path, value))
        })
        .next()
        .expect("persisted first attempt matches its exact ID");
    let diagnostic = format!(
        "first attempt diagnostics: fixture_owner={}\nfirst={first:#?}\npersisted={attempt:#}",
        fault.repository_root.display()
    );
    assert_eq!(
        first.execution_outcome,
        cockpit_verification::CompositionExecutionOutcome::Passed,
        "first execution must pass; {diagnostic}"
    );
    assert!(first.execution_evidence_complete, "{diagnostic}");
    assert_eq!(
        first.cleanup_disposition,
        cockpit_verification::CompositionCleanupDisposition::Deferred,
        "first attempt must be explicitly deferred; {diagnostic}"
    );
    assert!(
        !first.passed,
        "Deferred is not a passing terminal; {diagnostic}"
    );
    assert!(
        !first.is_coherent_successful_terminal(),
        "Deferred cannot be reused as a terminal result; {diagnostic}"
    );
    assert!(
        first
            .supervisor_receipt
            .as_ref()
            .is_some_and(|receipt| receipt.attempt_id == first.attempt_id),
        "old attempt must retain its exact supervisor termination binding; {diagnostic}"
    );
    assert!(
        first
            .supervisor_receipt
            .as_ref()
            .is_some_and(|receipt| receipt.descendants_reaped_to_echild),
        "old attempt must retain owned descendant termination proof; {diagnostic}"
    );
    assert_eq!(attempt["failure"], "composition_cleanup_deferred");
    assert_eq!(attempt["attemptId"], first.attempt_id);
    assert_eq!(attempt["executionOutcome"], "passed");
    assert_eq!(attempt["cleanupDisposition"], "deferred");
    assert_eq!(attempt["passed"], first.passed);

    let old_worktree = Path::new(
        attempt["isolatedWorktree"]
            .as_str()
            .expect("old worktree path"),
    )
    .to_path_buf();
    assert_eq!(first.isolated_worktree, old_worktree.to_string_lossy());
    assert!(
        old_worktree.is_dir(),
        "deferred worktree must remain on disk; {diagnostic}"
    );
    assert_eq!(
        fs::read_to_string(&fault.target_worktree_path_file)
            .expect("wrapper captured the first formal worktree")
            .trim(),
        old_worktree.to_string_lossy(),
        "injection target must bind to the old worktree; {diagnostic}"
    );
    let cleanup = first.cleanup.as_ref().expect("deferred cleanup evidence");
    assert_eq!(attempt["cleanup"]["attempted"], cleanup.attempted);
    assert_eq!(attempt["cleanup"]["removed"], cleanup.removed);
    assert_eq!(
        attempt["cleanup"]["error"].as_str(),
        cleanup.error.as_deref()
    );
    match cleanup.error.as_deref() {
        Some(error) if error.contains("injected one-time worktree cleanup refusal") => {
            assert!(
                cleanup.attempted,
                "injected Git refusal must be attempted; {diagnostic}"
            );
            assert_eq!(
                fs::read_to_string(&fault.injected_worktree_path_file)
                    .expect("injected old worktree path marker")
                    .trim(),
                old_worktree.to_string_lossy(),
                "injection marker must name the old worktree; {diagnostic}"
            );
        }
        Some(error) if error.starts_with("verifier_process_state_unknown:") => {
            assert!(
                !cleanup.attempted,
                "observer Err must defer before cleanup; {diagnostic}"
            );
            assert!(
                !fault.injected_worktree_path_file.exists(),
                "observer Err must not consume the Git cleanup injection; {diagnostic}"
            );
            assert!(
                first
                    .supervisor_receipt
                    .as_ref()
                    .is_some_and(|receipt| receipt.descendants_reaped_to_echild),
                "observer Err cannot stand in for owned termination proof; {diagnostic}"
            );
        }
        _ => panic!(
            "Deferred must name the injected Git refusal or a pre-cleanup observer Err; {diagnostic}"
        ),
    }
    let old_marker = old_worktree.join(".deferred-tree-marker");
    fs::write(&old_marker, "retain this deferred tree\n").expect("write old tree marker");
    let registered_worktrees = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(&fault.repository_root)
        .output()
        .expect("list deferred worktrees");
    assert!(registered_worktrees.status.success(), "{diagnostic}");
    assert!(
        String::from_utf8_lossy(&registered_worktrees.stdout)
            .lines()
            .any(|line| line == format!("worktree {}", old_worktree.display())),
        "old Deferred worktree registration must remain; {diagnostic}"
    );

    (first, attempt_path, old_worktree, old_marker)
}

#[test]
fn composition_supervisor_test_helper_entry() {
    let Ok(mode) = std::env::var("AI_COCKPIT_COMPOSITION_SUPERVISOR_TEST_HELPER") else {
        return;
    };
    if mode == "exit-before-ready" {
        return;
    }
    if mode == "revoke-generation-before-ready" {
        let root = std::env::var_os("AI_COCKPIT_COMPOSITION_REVOKE_ROOT")
            .expect("revocation test repository root");
        let work_item_id = std::env::var("AI_COCKPIT_COMPOSITION_REVOKE_WORK_ITEM")
            .expect("revocation test Work Item");
        let store = store(Path::new(&root));
        let mut registration = store
            .inspect()
            .expect("inspect registration before revocation")
            .registrations
            .into_iter()
            .find(|registration| registration.work_item_id == work_item_id)
            .expect("revoked Work Item registration");
        registration.generation += 1;
        store
            .register(registration)
            .expect("advance Work Item generation before Ready");
    }
    if mode == "identity-observation-error" {
        let path = std::env::var_os("AI_COCKPIT_SUPERVISOR_IDENTITY_PID_FILE")
            .expect("identity observer test PID file");
        fs::write(path, std::process::id().to_string()).expect("write supervisor PID marker");
    }
    // libtest's serial PrettyFormatter leaves this test's `... ` prefix open
    // on stdout. The parent protocol reader is line framed, so terminate that
    // prefix before the Ready JSON is written to the same stdout stream.
    let mut stdout = std::io::stdout().lock();
    stdout
        .write_all(b"\n")
        .expect("terminate supervisor helper test prefix");
    stdout.flush().expect("flush supervisor helper test prefix");
    drop(stdout);
    cockpit_repository::run_composition_supervisor_stdio()
        .expect("controlled composition supervisor protocol");
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
        env!("CARGO_PKG_VERSION"),
        &runtime_digest(),
    )
    .expect("record typed verification evidence");
}

fn publish_verification_outcome(
    store: &CoordinationStore,
    _root: &Path,
    work_item_id: &str,
    generation: u64,
    outcome_id: &str,
) {
    publish_outcome(store, work_item_id, generation, outcome_id)
        .expect("publish generation-bound verification outcome");
}

fn registration(
    root: &Path,
    work_item_id: &str,
    generation: u64,
    mut declaration: CollaborationDeclaration,
) -> WorktreeRegistration {
    let contract_digest = contract_digest(root, work_item_id);
    let topology = GitRepository::discover(root)
        .expect("discover")
        .topology()
        .expect("topology");
    if declaration
        .integration_responsibility
        .responsible_work_item_id
        == "WI-CONSUMER"
        && work_item_id != "WI-CONSUMER"
    {
        declaration
            .integration_responsibility
            .responsible_work_item_id = work_item_id.into();
    }
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
        environment: None,
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
        environment_change: None,
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
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        protocol_version: 1,
        runtime_digest: runtime_digest(),
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
    assert!(first.passed, "first composition failed: {first:?}");
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
    assert_eq!(second.processes_spawned, if cfg!(unix) { 0 } else { 1 });
    let second_projection =
        collaboration_outcome_projection(root.path(), "WI-CONSUMER", &runtime_context());
    assert_eq!(
        second_projection.reusable_checks,
        if cfg!(unix) {
            vec!["marker"]
        } else {
            Vec::new()
        }
    );
    assert!(!second_projection.human_decision_required);
}

#[test]
fn supervisor_exit_before_ready_records_unknown_without_spawning_verifier() {
    let root = repository();
    let store = store(root.path());
    let marker = root.path().join("must-not-exist");
    declare_required_checks(
        root.path(),
        "WI-CONSUMER",
        &[format!("touch {}", marker.display())],
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
    input.commands[0].program = "touch".into();
    input.commands[0].args = vec![marker.to_string_lossy().into_owned()];
    input.identity.command_digest = composition_commands_digest(&input.commands);

    let error = run_admitted_composition_with_supervisor_mode(
        &store,
        "WI-CONSUMER",
        1,
        input,
        "exit-before-ready",
    )
    .expect_err("a supervisor that exits before Ready must block verifier execution");
    assert!(error.to_string().contains("supervisor"), "{error:?}");
    assert!(
        !marker.exists(),
        "verifier command ran before supervisor release"
    );

    let attempts = fs::read_dir(store.root().join("compositions"))
        .expect("composition attempts")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    assert_eq!(attempts.len(), 1);
    let attempt: cockpit_verification::CompositionAttempt =
        serde_json::from_slice(&fs::read(&attempts[0]).expect("attempt evidence"))
            .expect("attempt JSON");
    assert!(!attempt.passed);
    assert_eq!(attempt.processes_spawned, 0);
    assert!(attempt.execution_records.is_empty());
    assert_eq!(
        attempt.execution_outcome,
        cockpit_verification::CompositionExecutionOutcome::Unknown
    );
    let projection =
        collaboration_outcome_projection(root.path(), "WI-CONSUMER", &runtime_context());
    assert_eq!(projection.composition_state, "unknown");
    assert_eq!(
        projection.execution_outcome,
        cockpit_verification::CompositionExecutionOutcome::Unknown
    );
    assert!(!projection.execution_evidence_complete);
}

#[cfg(target_os = "linux")]
#[test]
fn supervisor_identity_observation_error_reaps_before_return() {
    const INNER: &str = "AI_COCKPIT_SUPERVISOR_IDENTITY_TEST_INNER";
    if std::env::var_os(INNER).is_some() {
        let root = repository();
        let store = store(root.path());
        let marker = root.path().join("identity-observation-must-not-run");
        declare_required_checks(
            root.path(),
            "WI-CONSUMER",
            &[format!("touch {}", marker.display())],
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
        input.commands[0].program = "touch".into();
        input.commands[0].args = vec![marker.to_string_lossy().into_owned()];
        input.identity.command_digest = composition_commands_digest(&input.commands);

        let result = run_admitted_composition_with_supervisor_mode(
            &store,
            "WI-CONSUMER",
            1,
            input,
            "identity-observation-error",
        );
        let pid_file = std::env::var_os("AI_COCKPIT_SUPERVISOR_IDENTITY_PID_FILE")
            .expect("supervisor PID file path");
        let supervisor_pid = fs::read_to_string(pid_file)
            .expect("supervisor PID marker")
            .trim()
            .parse::<libc::pid_t>()
            .expect("supervisor PID");
        let error = result.expect_err("identity observer error must block verifier execution");

        let mut status = 0;
        let waited = unsafe { libc::waitpid(supervisor_pid, &mut status, libc::WNOHANG) };
        let reaped_before_return =
            waited == -1 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ECHILD);
        if !reaped_before_return && waited == 0 {
            unsafe {
                libc::kill(supervisor_pid, libc::SIGKILL);
                libc::waitpid(supervisor_pid, &mut status, 0);
            }
        }
        assert!(
            reaped_before_return,
            "supervisor {supervisor_pid} must be reaped before returning the observer error"
        );
        assert!(error.to_string().contains("Permission denied"), "{error:?}");
        assert!(
            !marker.exists(),
            "verifier command ran after supervisor identity observation failed"
        );

        let attempts = fs::read_dir(store.root().join("compositions"))
            .expect("composition attempts")
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "json")
            })
            .collect::<Vec<_>>();
        assert_eq!(attempts.len(), 1);
        let attempt: cockpit_verification::CompositionAttempt =
            serde_json::from_slice(&fs::read(&attempts[0]).expect("attempt evidence"))
                .expect("attempt JSON");
        assert!(!attempt.passed);
        assert_eq!(attempt.processes_spawned, 0);
        assert!(
            attempt
                .failure
                .as_deref()
                .is_some_and(|failure| failure.contains("Permission denied")),
            "observer error must remain in durable attempt: {attempt:?}"
        );
        return;
    }

    let fixture = tempfile::tempdir().expect("fault shim directory");
    let source = fixture.path().join("identity-observer-fault.c");
    let library = fixture.path().join("identity-observer-fault.so");
    let pid_file = fixture.path().join("supervisor.pid");
    fs::write(
        &source,
        r#"#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <unistd.h>

static int injected_failure = 0;

static pid_t marked_supervisor_pid(void) {
    const char *path = getenv("AI_COCKPIT_SUPERVISOR_IDENTITY_PID_FILE");
    if (!path) return -1;
    int fd = (int)syscall(SYS_openat, AT_FDCWD, path, O_RDONLY, 0);
    if (fd < 0) return -1;
    char value[32] = {0};
    ssize_t size = syscall(SYS_read, fd, value, sizeof(value) - 1);
    syscall(SYS_close, fd);
    return size > 0 ? (pid_t)strtol(value, NULL, 10) : -1;
}

static int injected_openat(int dirfd, const char *path, int flags, mode_t mode) {
    pid_t target = marked_supervisor_pid();
    char expected[80];
    int size = target > 0 ? snprintf(expected, sizeof(expected), "/proc/%ld/stat", (long)target) : -1;
    if (target > 0 && getpid() != target && size > 0 &&
        (size_t)size < sizeof(expected) && strcmp(path, expected) == 0 &&
        __sync_bool_compare_and_swap(&injected_failure, 0, 1)) {
        errno = EACCES;
        return -1;
    }
    return (int)syscall(SYS_openat, dirfd, path, flags, mode);
}

int openat(int dirfd, const char *path, int flags, ...) {
    mode_t mode = 0;
    if (flags & O_CREAT) { va_list args; va_start(args, flags); mode = va_arg(args, mode_t); va_end(args); }
    return injected_openat(dirfd, path, flags, mode);
}
int openat64(int dirfd, const char *path, int flags, ...) {
    mode_t mode = 0;
    if (flags & O_CREAT) { va_list args; va_start(args, flags); mode = va_arg(args, mode_t); va_end(args); }
    return injected_openat(dirfd, path, flags, mode);
}
int open(const char *path, int flags, ...) {
    mode_t mode = 0;
    if (flags & O_CREAT) { va_list args; va_start(args, flags); mode = va_arg(args, mode_t); va_end(args); }
    return injected_openat(AT_FDCWD, path, flags, mode);
}
int open64(const char *path, int flags, ...) {
    mode_t mode = 0;
    if (flags & O_CREAT) { va_list args; va_start(args, flags); mode = va_arg(args, mode_t); va_end(args); }
    return injected_openat(AT_FDCWD, path, flags, mode);
}
"#,
    )
    .expect("write controlled identity-observer fault shim");
    let compile = Command::new("cc")
        .args(["-shared", "-fPIC", "-o"])
        .arg(&library)
        .arg(&source)
        .output()
        .expect("compile controlled identity-observer fault shim");
    assert!(
        compile.status.success(),
        "C shim: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let output = Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "supervisor_identity_observation_error_reaps_before_return",
            "--nocapture",
        ])
        .env(INNER, "1")
        .env("AI_COCKPIT_SUPERVISOR_IDENTITY_PID_FILE", &pid_file)
        .env("LD_PRELOAD", &library)
        .output()
        .expect("run isolated identity-observer fault test");
    assert!(
        output.status.success(),
        "isolated fault test failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn legacy_incomplete_composition_attempts_project_unknown_execution() {
    let root = repository();
    let store = store(root.path());
    let marker = root.path().join("legacy-composition-marker");
    declare_required_checks(root.path(), "WI-CONSUMER", &["true".into()]);
    store
        .register(registration(
            root.path(),
            "WI-CONSUMER",
            1,
            declaration(root.path(), &[], &[]),
        ))
        .expect("register consumer");
    let mut input = composition_input(root.path(), &marker);
    input.commands[0].program = "true".into();
    input.commands[0].args.clear();
    input.commands[0].input_paths.clear();
    input.identity.command_digest = composition_commands_digest(&input.commands);
    let attempt =
        run_admitted_composition(&store, "WI-CONSUMER", 1, input).expect("create source attempt");
    let attempt_path = fs::read_dir(store.root().join("compositions"))
        .expect("composition attempts")
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
                && fs::read(path)
                    .ok()
                    .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
                    .is_some_and(|value| value["attemptId"] == attempt.attempt_id)
        })
        .expect("persisted composition attempt");

    for failure in [
        "in_progress",
        "verifier_process_state_unknown:cannot inspect process identity",
    ] {
        let mut legacy: serde_json::Value =
            serde_json::from_slice(&fs::read(&attempt_path).expect("attempt bytes"))
                .expect("attempt JSON");
        legacy["schemaVersion"] = serde_json::json!(2);
        legacy["passed"] = serde_json::json!(false);
        legacy["failure"] = serde_json::json!(failure);
        for field in [
            "executionOutcome",
            "executionEvidenceComplete",
            "supervisorReceipt",
            "cleanupDisposition",
        ] {
            legacy
                .as_object_mut()
                .expect("attempt object")
                .remove(field);
        }
        fs::write(
            &attempt_path,
            serde_json::to_vec_pretty(&legacy).expect("serialize legacy attempt"),
        )
        .expect("write legacy attempt");

        let projection =
            collaboration_outcome_projection(root.path(), "WI-CONSUMER", &runtime_context());
        assert_eq!(
            projection.execution_outcome,
            cockpit_verification::CompositionExecutionOutcome::Unknown,
            "legacy failure {failure:?} must not be promoted to an execution failure"
        );
        assert!(!projection.execution_evidence_complete);
        assert_eq!(
            projection.composition_state,
            if failure == "in_progress" {
                "in_progress"
            } else {
                "unknown"
            }
        );
    }
}

#[test]
fn revoked_admission_after_registration_persists_receipt_and_allows_a_later_retry() {
    let root = repository();
    let store = store(root.path());
    let marker = root.path().join("revoked-composition-must-not-run");
    declare_required_checks(
        root.path(),
        "WI-CONSUMER",
        &[format!("touch {}", marker.display())],
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
    input.commands[0].program = "touch".into();
    input.commands[0].args = vec![marker.to_string_lossy().into_owned()];
    input.identity.command_digest = composition_commands_digest(&input.commands);

    let error = run_admitted_composition_with_supervisor_mode(
        &store,
        "WI-CONSUMER",
        1,
        input.clone(),
        "revoke-generation-before-ready",
    )
    .expect_err("revoked generation must abort before verifier execution");
    assert!(error.to_string().contains("WI-CONSUMER"), "{error:?}");
    assert!(
        !marker.exists(),
        "verifier spawned after admission revocation"
    );

    let attempt_path = fs::read_dir(store.root().join("compositions"))
        .expect("composition attempt records")
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .expect("durable aborted attempt");
    let attempt: cockpit_verification::CompositionAttempt =
        serde_json::from_slice(&fs::read(&attempt_path).expect("aborted attempt bytes"))
            .expect("aborted attempt JSON");
    assert_eq!(attempt.processes_spawned, 0);
    assert!(attempt.execution_records.is_empty());
    assert!(
        attempt
            .failure
            .as_deref()
            .is_some_and(|failure| failure.starts_with("supervisor_aborted_before_release:")),
        "revocation failure must remain auditable: {attempt:?}"
    );
    assert_eq!(
        attempt.execution_outcome,
        cockpit_verification::CompositionExecutionOutcome::Unknown
    );
    let receipt = attempt
        .supervisor_receipt
        .as_ref()
        .expect("aborted attempt keeps its registered supervisor receipt");
    let registration_dir = store.root().join("compositions/supervisors");
    let registered_receipt: cockpit_verification::CompositionSupervisorReceipt =
        fs::read_dir(&registration_dir)
            .expect("supervisor registration directory")
            .flatten()
            .map(|entry| entry.path())
            .find(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "json")
            })
            .map(|path| {
                serde_json::from_slice(&fs::read(path).expect("registration bytes"))
                    .expect("registration JSON")
            })
            .expect("durable registration");
    assert_eq!(receipt, &registered_receipt);

    let retry = run_admitted_composition(&store, "WI-CONSUMER", 2, input)
        .expect("a later admitted attempt reconciles the aborted registration");
    assert!(retry.passed, "later retry failed: {retry:?}");
    assert_ne!(retry.attempt_id, attempt.attempt_id);
}

#[cfg(target_os = "linux")]
#[test]
fn deferred_execution_projection_keeps_execution_and_cleanup_distinct() {
    let root = PreserveTempDirOnPanic::new(repository());
    let store = store(root.path());
    let marker = root.path().join("deferred-projection-marker");
    declare_required_checks(root.path(), "WI-CONSUMER", &["true".into()]);
    store
        .register(registration(
            root.path(),
            "WI-CONSUMER",
            1,
            declaration(root.path(), &[], &[]),
        ))
        .expect("register consumer");
    let mut input = composition_input(root.path(), &marker);
    input.commands[0].program = "true".into();
    input.commands[0].args.clear();
    input.commands[0].input_paths.clear();
    input.identity.command_digest = composition_commands_digest(&input.commands);
    let fault = fail_first_worktree_remove_environment(root.path());
    let (attempt, _, _, _) = defer_successful_composition_cleanup(&store, input, &fault);

    assert_eq!(
        attempt.execution_outcome,
        cockpit_verification::CompositionExecutionOutcome::Passed
    );
    let projection =
        collaboration_outcome_projection(root.path(), "WI-CONSUMER", &runtime_context());
    assert_eq!(projection.composition_state, "unknown");
    assert_eq!(projection.cleanup_state, "deferred");
    assert_eq!(
        projection.execution_outcome,
        cockpit_verification::CompositionExecutionOutcome::Passed
    );
    assert!(projection.execution_evidence_complete);
    assert_eq!(
        projection.cleanup_disposition,
        cockpit_verification::CompositionCleanupDisposition::Deferred
    );
    assert!(projection.reusable_checks.is_empty());
}

#[cfg(target_os = "linux")]
#[test]
fn deferred_cleanup_retry_uses_a_fresh_attempt_and_preserves_the_old_tree() {
    let root = PreserveTempDirOnPanic::new(repository());
    let store = store(root.path());
    let marker = root.path().join("deferred-retry-marker");
    declare_required_checks(root.path(), "WI-CONSUMER", &["true".into()]);
    store
        .register(registration(
            root.path(),
            "WI-CONSUMER",
            1,
            declaration(root.path(), &[], &[]),
        ))
        .expect("register consumer");
    let mut input = composition_input(root.path(), &marker);
    input.commands[0].program = "true".into();
    input.commands[0].args.clear();
    input.commands[0].input_paths.clear();
    input.identity.command_digest = composition_commands_digest(&input.commands);
    let fault = fail_first_worktree_remove_environment(root.path());
    let (first, old_attempt_path, old_worktree, old_marker) =
        defer_successful_composition_cleanup(&store, input.clone(), &fault);

    let executable = std::env::current_exe().expect("controlled test helper path");
    let second = run_admitted_composition_with_supervisor_environment(
        &store,
        "WI-CONSUMER",
        1,
        input,
        &executable,
        &fault.supervisor_environment,
    )
    .expect("safe retry creates a fresh formal attempt");
    let fresh_attempt_path =
        persist_composition_attempt_snapshot(root.path(), "fresh-attempt", &second);
    let diagnostic = format!(
        "fixture_owner={}\nold_attempt_snapshot={}\nfresh_attempt_snapshot={}\nold={first:#?}\nfresh={second:#?}",
        root.path().display(),
        old_attempt_path.display(),
        fresh_attempt_path.display(),
    );
    assert_ne!(
        first.attempt_id, second.attempt_id,
        "fresh attempt identity; {diagnostic}"
    );
    assert_ne!(
        first
            .supervisor_receipt
            .as_ref()
            .map(|receipt| &receipt.run_nonce),
        second
            .supervisor_receipt
            .as_ref()
            .map(|receipt| &receipt.run_nonce),
        "fresh nonce; {diagnostic}"
    );
    assert_eq!(
        second.identity, first.identity,
        "same bound source identity; {diagnostic}"
    );
    assert_eq!(
        second
            .supervisor_receipt
            .as_ref()
            .map(|receipt| receipt.attempt_id.as_str()),
        Some(second.attempt_id.as_str()),
        "fresh receipt must bind the fresh attempt; {diagnostic}"
    );
    assert_eq!(
        second.processes_spawned, 1,
        "no prior node result is reused; {diagnostic}"
    );
    assert_eq!(
        second.execution_records.len(),
        1,
        "one fresh command record; {diagnostic}"
    );
    assert!(
        second.execution_records[0].spawned,
        "fresh attempt spawned the command; {diagnostic}"
    );
    assert!(
        !second.execution_records[0].reused,
        "predecessor result was not reused; {diagnostic}"
    );
    assert!(
        second.execution_records[0].passed,
        "fresh command passed; {diagnostic}"
    );
    assert_eq!(
        second.execution_records[0].exit_code,
        Some(0),
        "fresh command exit; {diagnostic}"
    );
    assert_eq!(
        second.execution_outcome,
        cockpit_verification::CompositionExecutionOutcome::Passed,
        "execution evidence remains complete independent of cleanup; {diagnostic}"
    );
    assert!(
        second.execution_evidence_complete,
        "fresh execution evidence must be complete; {diagnostic}"
    );
    assert_eq!(
        second.reuse_decision.kind,
        cockpit_verification::ReuseDecisionKind::Execute,
        "deferred retry must execute; {diagnostic}"
    );
    assert_eq!(
        second.reuse_decision.predecessor_attempt_id.as_deref(),
        Some(first.attempt_id.as_str()),
        "fresh execution binds the old attempt as predecessor without reusing results; {diagnostic}"
    );
    assert_ne!(
        second.isolated_worktree, first.isolated_worktree,
        "fresh attempt must use a distinct worktree; {diagnostic}"
    );
    assert!(
        old_marker.is_file(),
        "deferred old tree must remain untouched; {diagnostic}"
    );
    let worktrees = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(root.path())
        .output()
        .expect("list worktrees");
    assert!(worktrees.status.success(), "{diagnostic}");
    let worktrees = String::from_utf8(worktrees.stdout).expect("worktree list UTF-8");
    assert!(
        worktrees
            .lines()
            .any(|line| line == format!("worktree {}", old_worktree.display())),
        "old deferred worktree registration must remain; {diagnostic}"
    );
    let old_cleanup_was_injected = fs::read_to_string(&fault.injected_worktree_path_file)
        .ok()
        .map(|path| path.trim().to_owned());
    if let Some(path) = old_cleanup_was_injected.as_deref() {
        assert_eq!(
            path,
            old_worktree.to_string_lossy(),
            "only the old attempt may consume the injection; {diagnostic}"
        );
    }
    let fresh_worktree = PathBuf::from(&second.isolated_worktree);
    let fresh_cleanup = second.cleanup.as_ref().expect("fresh cleanup record");
    match second.cleanup_disposition {
        cockpit_verification::CompositionCleanupDisposition::Cleaned => {
            assert!(second.passed, "cleaned fresh attempt passes; {diagnostic}");
            assert!(
                fresh_cleanup.attempted && fresh_cleanup.removed,
                "fresh cleanup record must prove cleanup; {diagnostic}"
            );
            assert!(
                fresh_cleanup.error.is_none(),
                "cleaned attempt has no cleanup error; {diagnostic}"
            );
            assert!(
                !fresh_worktree.exists(),
                "fresh worktree was cleaned; {diagnostic}"
            );
        }
        cockpit_verification::CompositionCleanupDisposition::Deferred => {
            assert!(
                !second.passed,
                "Deferred fresh attempt cannot pass; {diagnostic}"
            );
            assert!(
                !fresh_cleanup.attempted && !fresh_cleanup.removed,
                "observer Err must stop before cleanup; {diagnostic}"
            );
            assert!(
                fresh_cleanup
                    .error
                    .as_deref()
                    .is_some_and(|error| error.starts_with("verifier_process_state_unknown:")),
                "fresh Deferred must report the actual observer Err, never the old injection; {diagnostic}"
            );
            assert!(
                fresh_worktree.is_dir(),
                "fresh Deferred tree remains on disk; {diagnostic}"
            );
            assert!(
                worktrees
                    .lines()
                    .any(|line| line == format!("worktree {}", fresh_worktree.display())),
                "fresh Deferred worktree remains registered; {diagnostic}"
            );
            assert!(
                !second.is_coherent_successful_terminal(),
                "fresh Deferred result cannot be terminal or reusable; {diagnostic}"
            );
            let projection =
                collaboration_outcome_projection(root.path(), "WI-CONSUMER", &runtime_context());
            assert_eq!(
                projection.composition_state, "unknown",
                "Deferred fresh attempt is not terminal; {diagnostic}"
            );
            assert_eq!(
                projection.cleanup_state, "deferred",
                "cleanup stays Deferred; {diagnostic}"
            );
            assert_eq!(
                projection.execution_outcome,
                cockpit_verification::CompositionExecutionOutcome::Passed,
                "execution remains separately observable; {diagnostic}"
            );
            assert!(
                projection.execution_evidence_complete,
                "complete execution evidence is retained; {diagnostic}"
            );
            assert!(
                projection.reusable_checks.is_empty(),
                "Deferred attempt cannot contribute reusable checks; {diagnostic}"
            );
        }
        other => panic!("fresh attempt has unexpected cleanup disposition {other:?}; {diagnostic}"),
    }
}

#[cfg(target_os = "linux")]
#[test]
fn external_observer_error_allows_only_a_fresh_safe_formal_retry() {
    use std::os::unix::fs::MetadataExt;

    struct ObserverPeer(std::process::Child);
    impl Drop for ObserverPeer {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    let root = PreserveTempDirOnPanic::new(repository());
    let store = store(root.path());
    let marker = root.path().join("observer-error-marker");
    declare_required_checks(root.path(), "WI-CONSUMER", &["true".into()]);
    store
        .register(registration(
            root.path(),
            "WI-CONSUMER",
            1,
            declaration(root.path(), &[], &[]),
        ))
        .expect("register consumer");
    let mut input = composition_input(root.path(), &marker);
    input.commands[0].program = "true".into();
    input.commands[0].args.clear();
    input.commands[0].input_paths.clear();
    input.identity.command_digest = composition_commands_digest(&input.commands);
    let mut peer = ObserverPeer(
        Command::new("sleep")
            .arg("30")
            .spawn()
            .expect("spawn bounded same-UID observer peer"),
    );
    assert!(peer.0.try_wait().expect("observer peer status").is_none());
    assert_eq!(
        fs::metadata(format!("/proc/{}", peer.0.id()))
            .expect("observer peer proc metadata")
            .uid(),
        unsafe { libc::getuid() },
    );
    let environment = fail_first_proc_cwd_observation_environment(root.path(), peer.0.id());
    let executable = std::env::current_exe().expect("controlled test helper path");
    let first = run_admitted_composition_with_supervisor_environment(
        &store,
        "WI-CONSUMER",
        1,
        input.clone(),
        &executable,
        &environment,
    )
    .expect("first formal result remains representable");
    let old_attempt_path = persist_composition_attempt_snapshot(root.path(), "old-attempt", &first);
    let first_diagnostic = format!(
        "fixture_owner={}\nold_attempt_snapshot={}\nold={first:#?}",
        root.path().display(),
        old_attempt_path.display(),
    );
    assert!(
        !first.passed,
        "Deferred old attempt cannot pass; {first_diagnostic}"
    );
    assert_eq!(first.processes_spawned, 1, "{first_diagnostic}");
    assert_eq!(first.execution_records.len(), 1, "{first_diagnostic}");
    assert!(first.execution_records[0].spawned, "{first_diagnostic}");
    assert!(!first.execution_records[0].reused, "{first_diagnostic}");
    assert_eq!(
        first.execution_records[0].exit_code,
        Some(0),
        "{first_diagnostic}"
    );
    assert!(first.execution_records[0].passed, "{first_diagnostic}");
    assert_eq!(
        first.execution_outcome,
        cockpit_verification::CompositionExecutionOutcome::Passed,
        "observer failure must preserve the completed execution outcome; {first_diagnostic}"
    );
    assert!(first.execution_evidence_complete, "{first_diagnostic}");
    assert_eq!(
        first.cleanup_disposition,
        cockpit_verification::CompositionCleanupDisposition::Deferred,
        "{first_diagnostic}"
    );
    assert_eq!(
        first.failure.as_deref(),
        Some("composition_cleanup_deferred"),
        "{first_diagnostic}"
    );
    assert!(
        first
            .supervisor_receipt
            .as_ref()
            .is_some_and(|receipt| receipt.attempt_id == first.attempt_id
                && receipt.descendants_reaped_to_echild),
        "old Deferred attempt retains exact receipt and owned termination proof; {first_diagnostic}"
    );
    assert!(
        !first.is_coherent_successful_terminal(),
        "Deferred old attempt is not terminal; {first_diagnostic}"
    );
    let cleanup = first.cleanup.as_ref().expect("deferred cleanup evidence");
    assert!(
        !cleanup.attempted && !cleanup.removed,
        "no deletion was attempted after observer error; {first_diagnostic}"
    );
    assert!(
        cleanup
            .error
            .as_deref()
            .is_some_and(|error| error.starts_with("verifier_process_state_unknown:")),
        "Deferred cause must be the actual observer Err; {first_diagnostic}"
    );
    assert!(
        first
            .supervisor_receipt
            .as_ref()
            .is_some_and(|receipt| receipt.descendants_reaped_to_echild)
    );
    let old_worktree = PathBuf::from(&first.isolated_worktree);
    assert!(old_worktree.is_dir(), "{first_diagnostic}");
    let old_marker = old_worktree.join(".observer-error-preserve");
    fs::write(&old_marker, "old deferred tree\n").expect("mark old worktree");
    let projection =
        collaboration_outcome_projection(root.path(), "WI-CONSUMER", &runtime_context());
    assert_eq!(projection.composition_state, "unknown");
    assert_eq!(projection.cleanup_state, "deferred");
    assert_eq!(
        projection.execution_outcome,
        cockpit_verification::CompositionExecutionOutcome::Passed
    );
    assert!(projection.reusable_checks.is_empty(), "{first_diagnostic}");
    assert_eq!(
        fs::read_to_string(root.path().join("observer-fault-count"))
            .expect("first observer fault count"),
        "1"
    );

    let second = run_admitted_composition_with_supervisor_environment(
        &store,
        "WI-CONSUMER",
        1,
        input,
        &executable,
        &environment,
    )
    .expect("second formal result remains representable");
    let fresh_attempt_path =
        persist_composition_attempt_snapshot(root.path(), "fresh-attempt", &second);
    let diagnostic = format!(
        "fixture_owner={}\nold_attempt_snapshot={}\nfresh_attempt_snapshot={}\nold={first:#?}\nfresh={second:#?}",
        root.path().display(),
        old_attempt_path.display(),
        fresh_attempt_path.display(),
    );
    assert_eq!(
        second.execution_outcome,
        cockpit_verification::CompositionExecutionOutcome::Passed,
        "fresh execution result remains separate from cleanup: {diagnostic}"
    );
    assert!(second.execution_evidence_complete, "{diagnostic}");
    assert_eq!(second.processes_spawned, 1, "{diagnostic}");
    assert_eq!(second.execution_records.len(), 1, "{diagnostic}");
    assert!(second.execution_records[0].spawned, "{diagnostic}");
    assert!(second.execution_records[0].passed, "{diagnostic}");
    assert_eq!(
        second.execution_records[0].exit_code,
        Some(0),
        "{diagnostic}"
    );
    assert!(!second.execution_records[0].reused, "{diagnostic}");
    assert_ne!(second.attempt_id, first.attempt_id, "{diagnostic}");
    assert_eq!(
        second.identity, first.identity,
        "same bound source identity on fresh attempt; {diagnostic}"
    );
    assert_eq!(
        second.reuse_decision.kind,
        cockpit_verification::ReuseDecisionKind::Execute,
        "fresh attempt must execute; {diagnostic}"
    );
    assert_eq!(
        second.reuse_decision.predecessor_attempt_id.as_deref(),
        Some(first.attempt_id.as_str()),
        "fresh attempt binds predecessor without reusing its result; {diagnostic}"
    );
    assert_ne!(
        second.isolated_worktree, first.isolated_worktree,
        "fresh retry worktree; {diagnostic}"
    );
    assert_eq!(
        second
            .supervisor_receipt
            .as_ref()
            .map(|receipt| receipt.attempt_id.as_str()),
        Some(second.attempt_id.as_str()),
        "fresh supervisor receipt identity; {diagnostic}"
    );
    assert_ne!(
        second
            .supervisor_receipt
            .as_ref()
            .map(|receipt| &receipt.run_nonce),
        first
            .supervisor_receipt
            .as_ref()
            .map(|receipt| &receipt.run_nonce)
    );
    assert!(
        old_marker.is_file(),
        "old tree marker remains; {diagnostic}"
    );
    let worktrees = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(root.path())
        .output()
        .expect("inspect retained worktree registration");
    assert!(worktrees.status.success(), "{diagnostic}");
    let worktree_list = String::from_utf8_lossy(&worktrees.stdout);
    assert!(
        worktree_list
            .lines()
            .any(|line| line == format!("worktree {}", old_worktree.display())),
        "old worktree registration retained; {diagnostic}"
    );
    let fresh_worktree = PathBuf::from(&second.isolated_worktree);
    let fresh_cleanup = second.cleanup.as_ref().expect("fresh cleanup record");
    match second.cleanup_disposition {
        cockpit_verification::CompositionCleanupDisposition::Cleaned => {
            assert!(second.passed, "cleaned fresh attempt passes; {diagnostic}");
            assert!(
                fresh_cleanup.attempted && fresh_cleanup.removed,
                "fresh cleanup must prove removal; {diagnostic}"
            );
            assert!(
                fresh_cleanup.error.is_none(),
                "clean fresh cleanup has no error; {diagnostic}"
            );
            assert!(
                !fresh_worktree.exists(),
                "fresh worktree removed; {diagnostic}"
            );
        }
        cockpit_verification::CompositionCleanupDisposition::Deferred => {
            assert!(
                !second.passed,
                "Deferred fresh attempt cannot pass; {diagnostic}"
            );
            assert!(
                !fresh_cleanup.attempted && !fresh_cleanup.removed,
                "observer Err must stop before deletion; {diagnostic}"
            );
            assert!(
                fresh_cleanup
                    .error
                    .as_deref()
                    .is_some_and(|error| error.starts_with("verifier_process_state_unknown:")),
                "fresh Deferred must name actual observer Err; {diagnostic}"
            );
            assert!(
                fresh_worktree.is_dir(),
                "fresh Deferred tree retained; {diagnostic}"
            );
            assert!(
                worktree_list
                    .lines()
                    .any(|line| line == format!("worktree {}", fresh_worktree.display())),
                "fresh Deferred worktree registration retained; {diagnostic}"
            );
            assert!(
                !second.is_coherent_successful_terminal(),
                "Deferred attempt is not terminal or reusable; {diagnostic}"
            );
            let projection =
                collaboration_outcome_projection(root.path(), "WI-CONSUMER", &runtime_context());
            assert_eq!(
                projection.composition_state, "unknown",
                "Deferred attempt cannot be terminal; {diagnostic}"
            );
            assert_eq!(
                projection.cleanup_state, "deferred",
                "Deferred cleanup remains visible; {diagnostic}"
            );
            assert_eq!(
                projection.execution_outcome,
                cockpit_verification::CompositionExecutionOutcome::Passed,
                "execution remains separately visible; {diagnostic}"
            );
            assert!(
                projection.execution_evidence_complete,
                "complete execution evidence retained; {diagnostic}"
            );
            assert!(
                projection.reusable_checks.is_empty(),
                "Deferred checks are not reusable; {diagnostic}"
            );
        }
        other => panic!("fresh attempt has unexpected cleanup disposition {other:?}; {diagnostic}"),
    }
    assert!(
        fs::read_to_string(root.path().join("observer-fault-count"))
            .expect("observer counter")
            .parse::<u32>()
            .expect("numeric counter")
            >= 1,
        "the fault shim ran at least once; {diagnostic}"
    );
    run(
        root.path(),
        &[
            "worktree",
            "remove",
            "--force",
            old_worktree.to_str().expect("UTF-8 worktree path"),
        ],
    );
}

#[cfg(target_os = "linux")]
#[test]
fn deferred_cleanup_retry_blocks_unbound_command_effects_before_verifier_spawn() {
    let root = PreserveTempDirOnPanic::new(repository());
    let store = store(root.path());
    let marker = root.path().join("unsafe-deferred-retry-marker");
    declare_required_checks(root.path(), "WI-CONSUMER", &["true".into()]);
    store
        .register(registration(
            root.path(),
            "WI-CONSUMER",
            1,
            declaration(root.path(), &[], &[]),
        ))
        .expect("register consumer");
    let mut input = composition_input(root.path(), &marker);
    input.commands[0].program = "true".into();
    input.commands[0].args.clear();
    input.commands[0].input_paths.clear();
    input.identity.command_digest = composition_commands_digest(&input.commands);
    let fault = fail_first_worktree_remove_environment(root.path());
    let (_first, attempt_path, _old_worktree, old_marker) =
        defer_successful_composition_cleanup(&store, input.clone(), &fault);

    let mut unsafe_input = input;
    unsafe_input.commands[0]
        .input_paths
        .push("README.md".into());
    unsafe_input.identity.command_digest = composition_commands_digest(&unsafe_input.commands);
    let executable = std::env::current_exe().expect("controlled test helper path");
    let blocked = run_admitted_composition_with_supervisor_environment(
        &store,
        "WI-CONSUMER",
        1,
        unsafe_input,
        &executable,
        &fault.supervisor_environment,
    )
    .expect("unsafe repeat is recorded as a blocked formal attempt");
    assert!(!blocked.passed);
    assert_eq!(blocked.processes_spawned, 0);
    assert!(blocked.execution_records.is_empty());
    assert!(
        blocked
            .failure
            .as_deref()
            .is_some_and(|failure| failure.starts_with("unsafe_deferred_cleanup_retry:"))
    );
    assert!(
        old_marker.is_file(),
        "unsafe retry must preserve the old tree"
    );
    let old_attempt: serde_json::Value =
        serde_json::from_slice(&fs::read(&attempt_path).expect("old attempt evidence"))
            .expect("old attempt JSON");
    assert_eq!(old_attempt["failure"], "composition_cleanup_deferred");
}

#[test]
fn shared_outcome_does_not_trust_a_tampered_composition_pass_flag() {
    let root = repository();
    let store = store(root.path());
    let marker = root.path().join("tampered-composition-marker");
    declare_required_checks(root.path(), "WI-CONSUMER", &["false".into()]);
    store
        .register(registration(
            root.path(),
            "WI-CONSUMER",
            1,
            declaration(root.path(), &[], &[]),
        ))
        .expect("register consumer");
    let mut input = composition_input(root.path(), &marker);
    input.commands[0].program = "false".into();
    input.commands[0].args.clear();
    input.identity.command_digest = composition_commands_digest(&input.commands);
    let attempt =
        run_admitted_composition(&store, "WI-CONSUMER", 1, input).expect("failed composition");
    assert!(!attempt.passed);

    let attempt_path = fs::read_dir(store.root().join("compositions"))
        .expect("composition records")
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .expect("persisted attempt");
    let mut tampered: serde_json::Value =
        serde_json::from_slice(&fs::read(&attempt_path).expect("attempt bytes"))
            .expect("attempt JSON");
    tampered["passed"] = serde_json::json!(true);
    fs::write(
        &attempt_path,
        serde_json::to_vec_pretty(&tampered).expect("serialize tampered attempt"),
    )
    .expect("persist tampering");

    let projection =
        collaboration_outcome_projection(root.path(), "WI-CONSUMER", &runtime_context());

    assert_ne!(
        projection.composition_state, "passed",
        "a contradictory terminal record must not project as a successful composition"
    );
    assert!(
        projection.reusable_checks.is_empty(),
        "incoherent attempts must not project reusable checks"
    );
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
    let marker = root.path().join("missing-scenario-marker");
    let mut work = declaration(root.path(), &[], &[]);
    work.composition_verification.required_scenarios =
        vec!["api-contract".into(), "docs-contract".into()];
    store
        .register(registration(root.path(), "WI-CONSUMER", 1, work))
        .expect("register consumer");

    let mut input = composition_input(root.path(), &marker);
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
    assert!(
        !marker.exists(),
        "missing scenario coverage must reject before spawning the required check"
    );
}

#[test]
fn missing_one_of_two_required_checks_blocks_before_spawn() {
    let root = repository();
    let store = store(root.path());
    let marker = root.path().join("missing-required-check-marker");
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
    let error = run_admitted_composition(&store, "WI-CONSUMER", 1, input)
        .expect_err("omitting a required check must fail closed")
        .to_string();

    assert!(
        error.contains("required_check_set_incomplete:expected=2:actual=1"),
        "the exact missing required-check count must be actionable: {error}"
    );
    assert!(
        !marker.exists(),
        "incomplete required-check coverage must reject before spawning any check"
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
fn required_check_environment_overlay_is_rejected_when_unbound_by_contract() {
    let root = repository();
    let store = store(root.path());
    declare_required_checks(
        root.path(),
        "WI-CONSUMER",
        &["printenv COCKPIT_REQUIRED_CHECK_RESULT".into()],
    );
    store
        .register(registration(
            root.path(),
            "WI-CONSUMER",
            1,
            declaration(root.path(), &[], &[]),
        ))
        .expect("register consumer");

    let mut input = composition_input(root.path(), &root.path().join("unused-marker"));
    input.commands[0].program = "printenv".into();
    input.commands[0].args = vec!["COCKPIT_REQUIRED_CHECK_RESULT".into()];
    input.commands[0]
        .environment
        .insert("COCKPIT_REQUIRED_CHECK_RESULT".into(), "forged-pass".into());
    input.identity.command_digest = composition_commands_digest(&input.commands);

    let error = run_admitted_composition(&store, "WI-CONSUMER", 1, input)
        .expect_err("an undeclared environment overlay must not redefine a required check");

    assert!(
        error
            .to_string()
            .contains("required_check_environment_unbound"),
        "the rejection should identify the unbound environment: {error}"
    );
    assert!(
        !store.root().join("compositions").exists(),
        "an unbound environment must be rejected before an attempt or check process starts"
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

    let admission =
        admit_collaboration_action(&store, "WI-CONSUMER", 1, composition_action("WI-CONSUMER"))
            .expect("query non-owner composition admission");
    assert!(
        !admission.allowed,
        "the admission projection must reject a non-owner before execution"
    );
    assert!(
        admission
            .blockers
            .iter()
            .any(|blocker| blocker.starts_with("composition_integration_owner_mismatch:"))
    );

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
    input.binding.target_sha = main_head.clone();
    input.binding.participant_heads = vec![feature_head.clone()];
    input.identity.command_digest = composition_commands_digest(&input.commands);
    let attempt = run_admitted_composition(&store, "WI-CONSUMER", 1, input)
        .expect("feature worktree may target main");

    assert_eq!(
        attempt.execution_outcome,
        cockpit_verification::CompositionExecutionOutcome::Passed,
        "feature-to-main execution failed: {attempt:?}"
    );
    assert!(attempt.execution_evidence_complete);
    assert_eq!(attempt.processes_spawned, 1);
    assert_eq!(attempt.execution_records.len(), 1);
    assert_eq!(attempt.execution_records[0].node_id, "marker");
    assert!(attempt.execution_records[0].spawned);
    assert!(!attempt.execution_records[0].reused);
    assert!(attempt.execution_records[0].passed);
    assert_eq!(attempt.execution_records[0].exit_code, Some(0));
    assert!(marker.is_file());
    assert!(attempt.text_conflicts.is_empty());
    assert!(attempt.preconditions.iter().all(|item| item.satisfied));
    assert_eq!(attempt.binding.target_branch, "main");
    assert_eq!(attempt.binding.target_sha, main_head);
    assert_eq!(attempt.binding.participant_heads, vec![feature_head]);
    let composition_state = match attempt.cleanup_disposition {
        cockpit_verification::CompositionCleanupDisposition::Cleaned => {
            assert!(
                attempt.passed,
                "feature-to-main composition failed: {attempt:?}"
            );
            assert!(attempt.is_coherent_successful_terminal());
            "passed"
        }
        cockpit_verification::CompositionCleanupDisposition::Deferred => {
            assert_eq!(attempt.schema_version, 3);
            assert_eq!(attempt.process_observation_schema_version, 1);
            assert!(!attempt.passed);
            assert_eq!(
                attempt.failure.as_deref(),
                Some("composition_cleanup_deferred")
            );
            assert!(!attempt.owned_tree_termination_unknown);
            assert!(attempt.owner_termination_signal.is_none());
            assert!(attempt.active_execution_node.is_none());
            assert!(attempt.active_process_group_id.is_none());
            assert!(attempt.active_process_group_identity.is_none());
            let receipt = attempt
                .supervisor_receipt
                .as_ref()
                .expect("deferred cleanup requires a bound supervisor receipt");
            assert_eq!(receipt.schema_version, 1);
            assert_eq!(
                receipt.backend,
                cockpit_verification::CompositionSupervisorBackend::LinuxSubreaper
            );
            assert_eq!(receipt.attempt_id, attempt.attempt_id);
            assert!(!receipt.run_nonce.is_empty());
            assert_eq!(receipt.generation, 1);
            assert_eq!(receipt.repository_id, attempt.binding.repository_id);
            assert_eq!(receipt.target_sha, attempt.binding.target_sha);
            assert_eq!(
                receipt.runtime_version,
                attempt.binding.verifier.runtime_version
            );
            assert_eq!(
                receipt.runtime_digest,
                attempt.binding.verifier.runtime_digest
            );
            assert_eq!(receipt.command_plan_digest, attempt.identity.command_digest);
            assert!(receipt.owner.process_id > 0);
            assert!(receipt.owner.start_time_ticks.is_some());
            assert!(receipt.owner.process_group_id.is_some());
            assert!(receipt.owner.session_id.is_some());
            assert!(receipt.supervisor.start_time_ticks.is_some());
            assert!(receipt.supervisor.process_group_id.is_some());
            assert!(receipt.supervisor.session_id.is_some());
            assert_eq!(attempt.owner_pid, Some(receipt.supervisor.process_id));
            assert!(
                receipt
                    .linux_boot_id
                    .as_deref()
                    .is_some_and(|id| !id.is_empty())
            );
            assert!(receipt.descendants_reaped_to_echild);
            let cleanup = attempt.cleanup.as_ref().expect("deferred cleanup evidence");
            assert!(!cleanup.attempted);
            assert!(!cleanup.removed);
            let error = cleanup.error.as_deref().expect("external observer error");
            assert!(
                error.starts_with("verifier_process_state_unknown:cannot inspect process ")
                    && [" working directory", " file descriptors", " open files"]
                        .iter()
                        .any(|suffix| error.ends_with(suffix)),
                "unexpected observer error: {error}"
            );
            let worktree = Path::new(&attempt.isolated_worktree);
            assert!(worktree.is_dir(), "deferred worktree must remain on disk");
            let worktrees = Command::new("git")
                .args(["worktree", "list", "--porcelain"])
                .current_dir(root.path())
                .output()
                .expect("inspect retained worktree registration");
            assert!(worktrees.status.success());
            assert!(
                String::from_utf8_lossy(&worktrees.stdout)
                    .lines()
                    .any(|line| line == format!("worktree {}", worktree.display())),
                "deferred worktree registration must remain"
            );
            "unknown"
        }
        other => panic!("unexpected feature-to-main cleanup disposition: {other:?}"),
    };
    let projection =
        collaboration_outcome_projection(root.path(), "WI-CONSUMER", &runtime_context());
    assert_eq!(projection.composition_state, composition_state);
    assert_eq!(
        projection.execution_outcome,
        cockpit_verification::CompositionExecutionOutcome::Passed
    );
    assert!(projection.execution_evidence_complete);
    assert_eq!(projection.cleanup_disposition, attempt.cleanup_disposition);
    assert_eq!(
        projection.cleanup_state,
        if composition_state == "unknown" {
            "deferred"
        } else {
            "cleaned"
        }
    );
    if composition_state == "unknown" {
        assert!(projection.reusable_checks.is_empty());
    }
    assert_eq!(projection.target_merge_state, "not_merged");
    assert_eq!(projection.composition_applicability, "current");

    run(root.path(), &["checkout", "-q", "main"]);
    fs::write(root.path().join("main-advance.txt"), "advanced\n").expect("advance main");
    run(root.path(), &["add", "main-advance.txt"]);
    run(root.path(), &["commit", "-qm", "advance main"]);
    run(root.path(), &["checkout", "-q", "feature/consumer"]);
    let stale_projection =
        collaboration_outcome_projection(root.path(), "WI-CONSUMER", &runtime_context());
    assert_eq!(stale_projection.composition_state, composition_state);
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
fn observed_environment_drift_is_persisted_and_denies_composition_before_spawn() {
    let root = repository();
    let store = store(root.path());
    register_provider_and_consumer(&store, root.path(), OutcomeStage::ComposableHead);
    let marker = root.path().join("unexpected-process-started");
    fs::write(
        root.path().join("Cargo.toml"),
        "[package]\nname = \"collaboration-fixture\"\nversion = \"0.9.0\"\n",
    )
    .expect("change current dependency input after registration");

    let queried =
        admit_collaboration_action(&store, "WI-CONSUMER", 1, composition_action("WI-CONSUMER"))
            .expect("read-only admission query");
    assert!(
        !queried.allowed,
        "unrecorded drift must fail closed in queries"
    );
    assert!(
        queried
            .unknowns
            .iter()
            .any(|unknown| { unknown == "environment_drift_unrecorded:WI-PROVIDER" })
    );
    assert!(
        store
            .inspect()
            .expect("inspect before explicit refresh")
            .events
            .is_empty(),
        "a query must not persist the event it reports"
    );

    let result = run_admitted_composition(
        &store,
        "WI-CONSUMER",
        1,
        composition_input(root.path(), &marker),
    );

    assert!(
        matches!(
            result,
            Err(cockpit_repository::CollaborationExecutionError::Blocked { .. })
        ),
        "stale provider environment must block before composition: {result:?}"
    );
    assert!(
        !marker.exists(),
        "no verification process may start on stale input"
    );
    let events = store.inspect().expect("read shared drift event").events;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].source, "runtime-observed-environment-drift");
    assert_eq!(events[0].outcome_ids, ["api"]);
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
    publish_outcome(&store, "WI-PROVIDER", 1, "api").unwrap();
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

    publish_outcome(&store, "WI-PROVIDER", 1, "api")
        .expect("the current generation must be able to republish its new evidence bytes");
    let publications = store.inspect().expect("inspect append-only publications");
    assert_eq!(
        publications.events.len(),
        2,
        "republishing must preserve the old event and append a new exact binding"
    );
    let rebound_evidence =
        admit_collaboration_action(&store, "WI-CONSUMER", 1, composition_action("WI-CONSUMER"))
            .expect("admission after republishing current evidence");
    assert!(
        rebound_evidence.allowed,
        "the newest exact evidence binding must satisfy the dependency: {:?}",
        rebound_evidence.blockers
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
        env!("CARGO_PKG_VERSION"),
        &runtime_digest(),
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
fn legacy_impact_without_outcome_ids_still_invalidates_transitive_consumers() {
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

    report_impact(&store, impact(root.path(), "WI-A", 1, "legacy-impact-WI-A")).unwrap();
    let legacy_event_path = store.root().join("events/legacy-impact-WI-A.json");
    let mut legacy_event: serde_json::Value =
        serde_json::from_slice(&fs::read(&legacy_event_path).expect("legacy event bytes"))
            .expect("legacy event JSON");
    assert_eq!(legacy_event["outcomeIds"], serde_json::json!([]));
    legacy_event
        .as_object_mut()
        .expect("legacy event object")
        .remove("outcomeIds");
    fs::write(
        &legacy_event_path,
        serde_json::to_vec_pretty(&legacy_event).expect("serialize old event shape"),
    )
    .expect("write old event shape without outcomeIds");

    store
        .register(registration(
            root.path(),
            "WI-A",
            2,
            declaration(root.path(), &[], &[]),
        ))
        .expect("provider generation removes the old output");
    recover_impact(&store, "auto-impact-WI-A-2", "WI-B", 1)
        .expect("isolate the pre-existing legacy event from the new explicit event");

    for work_item_id in ["WI-C", "WI-D"] {
        assert!(
            !admit_collaboration_action(&store, work_item_id, 1, composition_action(work_item_id))
                .unwrap()
                .allowed,
            "legacy all-outcomes impact must invalidate transitive consumer {work_item_id} after the provider removes its output"
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
        "a legacy provider impact must not block unrelated work"
    );
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
fn tampered_recovery_identity_cannot_suppress_invalidation() {
    let root = repository();
    let store = store(root.path());
    register_provider_and_consumer(&store, root.path(), OutcomeStage::ComposableHead);
    let mut event = impact(root.path(), "WI-PROVIDER", 1, "impact-recovery-validation");
    event.outcome_ids = vec!["api".into()];
    report_impact(&store, event).expect("publish invalidation");
    recover_impact(&store, "impact-recovery-validation", "WI-CONSUMER", 1)
        .expect("record valid recovery");

    let recovery_path = store
        .root()
        .join("recoveries/recovery-impact-recovery-validation-WI-CONSUMER-1.json");
    let valid: serde_json::Value =
        serde_json::from_slice(&fs::read(&recovery_path).expect("recovery bytes"))
            .expect("recovery JSON");
    for (field, replacement) in [
        ("schemaVersion", serde_json::json!(99)),
        (
            "repositoryId",
            serde_json::json!(format!("sha256:{}", "a".repeat(64))),
        ),
        ("consumptionId", serde_json::json!("recovery-copied")),
        ("eventId", serde_json::json!("other-impact")),
        ("providerWorkItemId", serde_json::json!("WI-OTHER")),
        ("providerGeneration", serde_json::json!(2)),
        ("currentProviderGeneration", serde_json::json!(Some(2))),
        ("currentProviderHead", serde_json::json!("foreign-head")),
        (
            "currentProviderContractDigest",
            serde_json::json!(format!("sha256:{}", "b".repeat(64))),
        ),
        ("consumerWorkItemId", serde_json::json!("WI-OTHER")),
        ("consumerGeneration", serde_json::json!(2)),
    ] {
        let mut tampered = valid.clone();
        tampered[field] = replacement;
        fs::write(
            &recovery_path,
            serde_json::to_vec_pretty(&tampered).expect("serialize tampered recovery"),
        )
        .expect("write tampered recovery");

        let inspection = store.inspect().expect("inspect tampered recovery");
        assert!(
            !inspection.unknowns.is_empty(),
            "tampered {field} must be reported as unknown"
        );
        assert!(
            inspection.recoveries.is_empty(),
            "tampered {field} must not enter the trusted projection"
        );
        let admission = admit_collaboration_action(
            &store,
            "WI-CONSUMER",
            1,
            outcome_action("WI-CONSUMER", &[("WI-PROVIDER", "api")]),
        )
        .expect("admission remains queryable");
        assert!(
            !admission.allowed,
            "tampered {field} must not suppress the provider invalidation"
        );
        assert!(
            admission
                .blockers
                .iter()
                .any(|blocker| blocker.starts_with("dependency_impact:")),
            "tampered {field} must preserve the dependency impact blocker"
        );
        fs::write(
            &recovery_path,
            serde_json::to_vec_pretty(&valid).expect("serialize valid recovery"),
        )
        .expect("restore original recovery for next case");
        assert!(
            admit_collaboration_action(
                &store,
                "WI-CONSUMER",
                1,
                outcome_action("WI-CONSUMER", &[("WI-PROVIDER", "api")]),
            )
            .expect("valid recovery admission")
            .allowed,
            "restoring valid {field} binding must consume the event"
        );
    }
}

#[test]
fn tampered_historical_provider_snapshot_cannot_suppress_invalidation() {
    let root = repository();
    let store = store(root.path());
    register_provider_and_consumer(&store, root.path(), OutcomeStage::ComposableHead);
    let mut event = impact(root.path(), "WI-PROVIDER", 1, "impact-historical-snapshot");
    event.outcome_ids = vec!["api".into()];
    report_impact(&store, event).expect("publish invalidation");

    store
        .register(registration(
            root.path(),
            "WI-PROVIDER",
            2,
            declaration(root.path(), &[("api", OutcomeStage::ComposableHead)], &[]),
        ))
        .expect("advance provider generation");
    recover_impact(&store, "impact-historical-snapshot", "WI-CONSUMER", 1)
        .expect("record recovery against the advanced provider");

    let recovery_path = store
        .root()
        .join("recoveries/recovery-impact-historical-snapshot-WI-CONSUMER-1.json");
    let mut tampered: serde_json::Value =
        serde_json::from_slice(&fs::read(&recovery_path).expect("recovery bytes"))
            .expect("recovery JSON");
    assert_eq!(tampered["currentProviderGeneration"], 2);
    tampered["currentProviderGeneration"] = serde_json::json!(1);
    tampered["currentProviderHead"] = serde_json::json!("forged-historical-head");
    tampered["currentProviderContractDigest"] =
        serde_json::json!(digest("forged-historical-contract").to_string());
    fs::write(
        &recovery_path,
        serde_json::to_vec_pretty(&tampered).expect("serialize tampered recovery"),
    )
    .expect("persist tampered historical snapshot");

    let inspection = store.inspect().expect("inspect tampered recovery");
    assert!(
        !inspection.unknowns.is_empty(),
        "an unverifiable historical provider snapshot must be unknown"
    );
    assert!(inspection.recoveries.is_empty());
    let admission = admit_collaboration_action(
        &store,
        "WI-CONSUMER",
        1,
        outcome_action("WI-CONSUMER", &[("WI-PROVIDER", "api")]),
    )
    .expect("query admission after tampering");
    assert!(!admission.allowed);
    assert!(
        admission
            .blockers
            .iter()
            .any(|blocker| blocker.starts_with("dependency_impact:"))
    );
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

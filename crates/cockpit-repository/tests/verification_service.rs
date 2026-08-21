use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, run_repository_verification,
};
use cockpit_verification::ProtectedGateClass;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn repository(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "cockpit-verification-service-{name}-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("repository");
    fs::write(root.join("tracked.txt"), "before\n").expect("tracked file");
    fs::write(root.join(".gitignore"), ".counter\n").expect("ignore counter");
    run_git(&root, &["init", "-q"]);
    run_git(&root, &["add", "."]);
    run_git(
        &root,
        &[
            "-c",
            "user.name=AI Cockpit Test",
            "-c",
            "user.email=ai-cockpit@example.invalid",
            "commit",
            "-qm",
            "fixture",
        ],
    );
    root
}

fn run_git(root: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .status()
            .expect("git")
            .success()
    );
}

fn request(
    program: &str,
    args: Vec<String>,
    policy: RepositoryVerificationPolicy,
) -> RepositoryVerificationRequest {
    RepositoryVerificationRequest {
        node_id: "project-command-0".into(),
        program: program.into(),
        args,
        scope: vec!["**".into()],
        stage: "task".into(),
        runner: "local".into(),
        runtime_digest: format!("sha256:{}", "a".repeat(64)),
        base_commit: None,
        workers: 1,
        policy,
    }
}

#[cfg(unix)]
#[test]
fn post_command_repository_drift_discards_the_passed_receipt_candidate() {
    let root = repository("drift");
    cockpit_repository::attach(&root).expect("attach");
    let args = vec![
        "-c".into(),
        "from pathlib import Path; Path('tracked.txt').write_text('after\\n')".into(),
    ];
    cockpit_repository::confirm_profile_update(&root, "python3", &args).expect("confirm");

    let run = run_repository_verification(
        &root,
        &request(
            "python3",
            args,
            RepositoryVerificationPolicy::ProfileAuthorized,
        ),
    )
    .expect("verify");

    assert!(run.receipt.passed);
    assert_eq!(run.receipt.processes_spawned, 1);
    assert!(run.receipt.receipt_candidates.is_empty());
    assert_eq!(run.receipt.results[0].receipt_id, None);
    assert_eq!(
        run.receipt.results[0].reason,
        "post_execution_binding_drift"
    );
    assert!(!root.join(".ai/evidence/reuse/index.json").exists());
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn protected_service_commands_execute_on_every_invocation() {
    let root = repository("protected");
    let args = vec![
        "-c".into(),
        "from pathlib import Path; p=Path('.counter'); p.write_text(str((int(p.read_text()) if p.exists() else 0)+1))".into(),
    ];
    let request = request(
        "python3",
        args,
        RepositoryVerificationPolicy::Protected(ProtectedGateClass::Scope),
    );

    let first = run_repository_verification(&root, &request).expect("first");
    let second = run_repository_verification(&root, &request).expect("second");

    assert_eq!(first.receipt.processes_spawned, 1);
    assert_eq!(second.receipt.processes_spawned, 1);
    assert_eq!(first.receipt.protected_nodes_executed, 1);
    assert_eq!(second.receipt.protected_nodes_executed, 1);
    assert_eq!(first.receipt.protected_nodes_skipped, 0);
    assert_eq!(second.receipt.protected_nodes_skipped, 0);
    assert_eq!(
        fs::read_to_string(root.join(".counter")).expect("counter"),
        "2"
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn missing_profile_executes_as_never_reuse_instead_of_blocking_verification() {
    let root = repository("missing-profile");
    let run = run_repository_verification(
        &root,
        &request(
            "true",
            vec![],
            RepositoryVerificationPolicy::ProfileAuthorized,
        ),
    )
    .expect("missing profile must only disable reuse");

    assert!(run.receipt.passed);
    assert_eq!(run.receipt.processes_spawned, 1);
    assert_eq!(run.receipt.nodes_reused, 0);
    assert_eq!(run.receipt.results[0].reason, "reuse_not_configured");
    assert!(run.receipt.receipt_candidates.is_empty());
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn never_reuse_execution_preserves_the_requested_executable_path() {
    use std::os::unix::fs::PermissionsExt;

    let root = repository("never-reuse-path");
    let tool = root.join("path-sensitive-tool.sh");
    fs::write(
        &tool,
        "#!/bin/sh\ntouch \"$(dirname \"$0\")/path-was-preserved\"\n",
    )
    .expect("script");
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("executable");
    let run = run_repository_verification(
        &root,
        &request(
            &tool.to_string_lossy(),
            vec![],
            RepositoryVerificationPolicy::NeverReuse,
        ),
    )
    .expect("verify");

    assert!(run.receipt.passed);
    assert!(root.join("path-was-preserved").is_file());
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn early_reuse_denial_does_not_report_identity_files_that_were_never_read() {
    let root = repository("early-denial-metrics");
    cockpit_repository::attach(&root).expect("attach");
    let mut profile_request = request(
        "true",
        vec![],
        RepositoryVerificationPolicy::ProfileAuthorized,
    );
    profile_request.stage = "invalid-stage".into();
    let mut never_request = profile_request.clone();
    never_request.policy = RepositoryVerificationPolicy::NeverReuse;

    let denied = run_repository_verification(&root, &profile_request).expect("denied reuse");
    let never = run_repository_verification(&root, &never_request).expect("never reuse");

    assert_eq!(denied.receipt.files_read, never.receipt.files_read);
    assert_eq!(denied.receipt.files_hashed, never.receipt.files_hashed);
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn drift_after_an_initial_fresh_plan_forces_a_real_execution_fallback() {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::{sync::mpsc, time::Duration};

    let root = repository("fresh-drift");
    let tool = root
        .parent()
        .expect("fixture parent")
        .join(format!("cockpit-large-tool-{}", std::process::id()));
    let mut file = fs::File::create(&tool).expect("tool");
    file.write_all(b"#!/bin/sh\ncount=.counter; n=0; test -f $count && n=$(cat $count); echo $((n+1)) > $count\n")
        .expect("script");
    drop(file);
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("executable");
    cockpit_repository::attach(&root).expect("attach");
    let program = tool.to_string_lossy().into_owned();
    cockpit_repository::confirm_profile_update(&root, &program, &[]).expect("confirm");
    let request = request(
        &program,
        vec![],
        RepositoryVerificationPolicy::ProfileAuthorized,
    );
    let first = run_repository_verification(&root, &request).expect("first");
    assert_eq!(first.receipt.processes_spawned, 1);

    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(root.join(".ai/evidence/reuse/index.lock"))
        .expect("store lock");
    lock.lock().expect("hold store lock");
    let (started_tx, started_rx) = mpsc::channel();
    let worker_root = root.clone();
    let worker_request = request.clone();
    let worker = std::thread::spawn(move || {
        started_tx.send(()).expect("signal");
        run_repository_verification(&worker_root, &worker_request)
    });
    started_rx.recv().expect("started");
    std::thread::sleep(Duration::from_secs(1));
    fs::write(root.join("tracked.txt"), "drifted\n").expect("drift");
    lock.unlock().expect("release store lock");
    let second = worker.join().expect("verification thread").expect("second");

    assert_eq!(second.receipt.nodes_reused, 0);
    assert_eq!(second.receipt.processes_spawned, 1);
    assert_eq!(
        second.receipt.results[0].reason,
        "post_planning_binding_drift"
    );
    assert_eq!(
        fs::read_to_string(root.join(".counter"))
            .expect("counter")
            .trim(),
        "2"
    );
    fs::remove_file(tool).expect("cleanup tool");
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn profile_and_config_byte_drift_after_authorization_force_execution() {
    use std::os::unix::fs::PermissionsExt;
    use std::{sync::mpsc, time::Duration};

    for identity_file in ["cockpit.toml", "project.json"] {
        let root = repository(&format!("identity-drift-{identity_file}"));
        let tool = root.parent().expect("fixture parent").join(format!(
            "cockpit-identity-tool-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(
            &tool,
            b"#!/bin/sh\ncount=.counter; n=0; test -f $count && n=$(cat $count); echo $((n+1)) > $count\n",
        )
        .expect("script");
        fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("executable");
        cockpit_repository::attach(&root).expect("attach");
        let program = tool.to_string_lossy().into_owned();
        cockpit_repository::confirm_profile_update(&root, &program, &[]).expect("confirm");
        let request = request(
            &program,
            vec![],
            RepositoryVerificationPolicy::ProfileAuthorized,
        );
        let first = run_repository_verification(&root, &request).expect("first");
        assert_eq!(first.receipt.processes_spawned, 1);

        let lock = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(root.join(".ai/evidence/reuse/index.lock"))
            .expect("store lock");
        lock.lock().expect("hold store lock");
        let (started_tx, started_rx) = mpsc::channel();
        let worker_root = root.clone();
        let worker_request = request.clone();
        let worker = std::thread::spawn(move || {
            started_tx.send(()).expect("signal");
            run_repository_verification(&worker_root, &worker_request)
        });
        started_rx.recv().expect("started");
        std::thread::sleep(Duration::from_secs(1));
        let identity_path = root.join(".ai").join(identity_file);
        let mut bytes = fs::read(&identity_path).expect("identity bytes");
        bytes.push(b'\n');
        fs::write(&identity_path, bytes).expect("drift identity bytes");
        lock.unlock().expect("release store lock");

        let second = worker.join().expect("verification thread").expect("second");
        assert_eq!(second.receipt.nodes_reused, 0, "{identity_file}");
        assert_eq!(second.receipt.processes_spawned, 1, "{identity_file}");
        assert_eq!(
            second.receipt.results[0].reason, "post_planning_binding_drift",
            "{identity_file}"
        );
        assert_eq!(
            fs::read_to_string(root.join(".counter"))
                .expect("counter")
                .trim(),
            "2",
            "{identity_file}"
        );
        fs::remove_file(tool).expect("cleanup tool");
        fs::remove_dir_all(root).expect("cleanup");
    }
}

#[cfg(unix)]
#[test]
fn executable_path_swap_cannot_change_the_bytes_that_are_executed() {
    use std::os::unix::fs::PermissionsExt;
    use std::{sync::mpsc, time::Duration};

    let root = repository("pinned-executable");
    let suffix = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let fixture_parent = root.parent().expect("fixture parent");
    let tool = fixture_parent.join(format!(
        "cockpit-pinned-tool-{}-{suffix}",
        std::process::id()
    ));
    let backup = tool.with_extension("original");
    let malicious = tool.with_extension("replacement");
    let displaced = tool.with_extension("displaced");
    let armed = tool.with_extension("armed");
    let started = tool.with_extension("started");
    let proceed = tool.with_extension("proceed");
    let original = format!(
        "#!/bin/sh\nif test -f '{armed}'; then echo started > '{started}'; while ! test -f '{proceed}'; do sleep 0.01; done; fi\ncount=.counter; n=0; test -f $count && n=$(cat $count); echo $((n+1)) > $count\n",
        armed = armed.display(),
        started = started.display(),
        proceed = proceed.display(),
    );
    let replacement = format!(
        "#!/bin/sh\necho started > '{started}'\necho replacement > .counter\n",
        started = started.display(),
    );
    fs::write(&tool, original).expect("original tool");
    fs::write(&malicious, replacement).expect("replacement tool");
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("original executable");
    fs::set_permissions(&malicious, fs::Permissions::from_mode(0o755))
        .expect("replacement executable");

    cockpit_repository::attach(&root).expect("attach");
    let program = tool.to_string_lossy().into_owned();
    cockpit_repository::confirm_profile_update(&root, &program, &[]).expect("confirm");
    let request = request(
        &program,
        vec![],
        RepositoryVerificationPolicy::ProfileAuthorized,
    );
    let first = run_repository_verification(&root, &request).expect("first");
    assert!(first.receipt.passed);
    assert_eq!(first.receipt.receipt_candidates.len(), 1);

    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(root.join(".ai/evidence/reuse/index.lock"))
        .expect("store lock");
    lock.lock().expect("hold store lock");
    fs::write(&armed, b"armed\n").expect("arm original");
    let receipt_path = fs::read_dir(root.join(".ai/evidence/reuse/receipts"))
        .expect("receipt directory")
        .next()
        .expect("receipt entry")
        .expect("receipt path")
        .path();

    let (started_tx, started_rx) = mpsc::channel();
    let worker_root = root.clone();
    let worker_request = request.clone();
    let worker = std::thread::spawn(move || {
        started_tx.send(()).expect("signal");
        run_repository_verification(&worker_root, &worker_request)
    });
    started_rx.recv().expect("worker started");
    std::thread::sleep(Duration::from_secs(1));
    fs::rename(&tool, &backup).expect("preserve original path");
    fs::rename(&malicious, &tool).expect("replace executable path");
    fs::remove_file(receipt_path).expect("force execution after planning");
    lock.unlock().expect("release store lock");

    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while !started.exists() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(started.exists(), "the pinned command must start");
    fs::rename(&tool, &displaced).expect("remove replacement");
    fs::rename(&backup, &tool).expect("restore original path");
    fs::write(&proceed, b"proceed\n").expect("release original command");

    let second = worker.join().expect("verification thread").expect("second");
    assert!(second.receipt.passed);
    assert_eq!(second.receipt.processes_spawned, 1);
    assert_eq!(
        fs::read_to_string(root.join(".counter"))
            .expect("counter")
            .trim(),
        "2"
    );

    for path in [tool, displaced, armed, started, proceed] {
        fs::remove_file(path).expect("cleanup fixture");
    }
    fs::remove_dir_all(root).expect("cleanup repository");
}

#[cfg(unix)]
#[test]
fn fresh_plan_with_persistent_executable_drift_executes_and_receipts_new_bytes() {
    use std::os::unix::fs::PermissionsExt;
    use std::{sync::mpsc, time::Duration};

    let root = repository("persistent-executable-drift");
    fs::write(root.join(".gitignore"), ".counter\n.executed\n").expect("ignore execution log");
    run_git(&root, &["add", ".gitignore"]);
    run_git(
        &root,
        &[
            "-c",
            "user.name=AI Cockpit Test",
            "-c",
            "user.email=ai-cockpit@example.invalid",
            "commit",
            "--amend",
            "--no-edit",
            "-q",
        ],
    );
    let suffix = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let fixture_parent = root.parent().expect("fixture parent");
    let tool = fixture_parent.join(format!(
        "cockpit-drifting-tool-{}-{suffix}",
        std::process::id()
    ));
    let original = tool.with_extension("original");
    let replacement = tool.with_extension("replacement");
    fs::write(&tool, "#!/bin/sh\necho A >> .executed\n").expect("original tool");
    fs::write(
        &replacement,
        format!(
            "#!/bin/sh\nif test \"$0\" = '{}'; then echo UNPINNED >> .executed; else echo B >> .executed; fi\n",
            tool.display()
        ),
    )
    .expect("replacement tool");
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("original executable");
    fs::set_permissions(&replacement, fs::Permissions::from_mode(0o755))
        .expect("replacement executable");

    cockpit_repository::attach(&root).expect("attach");
    let program = tool.to_string_lossy().into_owned();
    cockpit_repository::confirm_profile_update(&root, &program, &[]).expect("confirm");
    let request = request(
        &program,
        vec![],
        RepositoryVerificationPolicy::ProfileAuthorized,
    );
    let first = run_repository_verification(&root, &request).expect("first");
    assert!(first.receipt.passed);
    assert_eq!(first.receipt.processes_spawned, 1);

    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(root.join(".ai/evidence/reuse/index.lock"))
        .expect("store lock");
    lock.lock().expect("hold store lock");
    let (started_tx, started_rx) = mpsc::channel();
    let worker_root = root.clone();
    let worker_request = request.clone();
    let worker = std::thread::spawn(move || {
        started_tx.send(()).expect("signal");
        run_repository_verification(&worker_root, &worker_request)
    });
    started_rx.recv().expect("worker started");
    std::thread::sleep(Duration::from_secs(1));
    fs::rename(&tool, &original).expect("preserve original");
    fs::rename(&replacement, &tool).expect("install replacement");
    lock.unlock().expect("release store lock");

    let second = worker.join().expect("verification thread").expect("second");
    assert!(second.receipt.passed);
    assert_eq!(second.receipt.nodes_reused, 0);
    assert_eq!(second.receipt.processes_spawned, 1);
    assert_eq!(
        second.receipt.results[0].reason,
        "post_planning_binding_drift"
    );
    assert_eq!(
        fs::read_to_string(root.join(".executed")).expect("execution log"),
        "A\nB\n"
    );

    let third = run_repository_verification(&root, &request).expect("third");
    assert!(third.receipt.passed);
    assert_eq!(third.receipt.nodes_reused, 1);
    assert_eq!(third.receipt.processes_spawned, 0);

    for path in [tool, original] {
        fs::remove_file(path).expect("cleanup fixture");
    }
    fs::remove_dir_all(root).expect("cleanup repository");
}

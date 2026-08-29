use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, VerificationContextInput,
    VerificationReuseAssessment, assess_verification_reuse, run_repository_verification,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_REPOSITORY_ID: AtomicU64 = AtomicU64::new(0);

fn digest(fill: char) -> String {
    format!("sha256:{}", fill.to_string().repeat(64))
}

fn repository(name: &str, commit: bool) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "cockpit-verification-context-{name}-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(root.join("src")).expect("repository directories");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"context-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("manifest");
    fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").expect("source");
    run(&root, &["init", "-q"]);
    if commit {
        run(&root, &["add", "Cargo.toml", "src/lib.rs"]);
        run(
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
    }
    root
}

fn run(root: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .status()
        .expect("run git");
    assert!(status.success(), "git {args:?}");
}

fn input() -> VerificationContextInput {
    VerificationContextInput {
        program: "cargo".into(),
        args: vec!["test".into(), "--workspace".into()],
        command_digest: digest('a'),
        scope: vec!["src/**".into()],
        stage: "task".into(),
        runner: "local".into(),
        runtime_digest: digest('b'),
        base_commit: None,
    }
}

fn successful_command() -> (&'static str, Vec<String>) {
    if cfg!(windows) {
        ("cmd.exe", vec!["/C".into(), "exit".into(), "0".into()])
    } else {
        ("true", Vec::new())
    }
}

fn authorized_context(
    root: &Path,
    snapshot: &cockpit_git::RepositorySnapshot,
    input: &VerificationContextInput,
) -> cockpit_evidence::EvidenceContext {
    match assess_verification_reuse(root, snapshot, input).expect("assess reuse") {
        VerificationReuseAssessment::Authorized(authorization) => authorization.context,
        VerificationReuseAssessment::Denied { reason } => {
            panic!("expected authorized context, denied: {reason}")
        }
    }
}

#[test]
fn calibrated_verified_command_derives_one_stable_profile_bound_context() {
    let root = repository("authorized", true);
    cockpit_repository::attach(&root).expect("attach");
    let profile = cockpit_repository::confirm_profile_update(
        &root,
        "cargo",
        &["test".into(), "--workspace".into()],
    )
    .expect("confirm profile");
    let snapshot = cockpit_git::GitRepository::discover(&root)
        .expect("git")
        .snapshot()
        .expect("snapshot");

    let first = authorized_context(&root, &snapshot, &input());
    let second = authorized_context(&root, &snapshot, &input());

    assert_eq!(first, second);
    assert_eq!(first.diff.base_commit, snapshot.head.clone().unwrap());
    assert_eq!(first.diff.head_commit, snapshot.head.clone().unwrap());
    assert_eq!(
        first.diff.changed_paths_digest,
        "sha256:4f53cda18c2baa0c0354bb5f9a3ecbe5ed12ab4d8e11ba873c2f11161202b945"
    );
    assert_eq!(first.command_digest, digest('a'));
    assert_eq!(
        first.profile_digest,
        profile.profile_digest.unwrap().to_string()
    );
    assert_eq!(first.stage, "task");
    assert_eq!(first.runner, "local");
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn source_runtime_and_scope_changes_invalidate_their_context_dimensions() {
    let root = repository("invalidation", true);
    cockpit_repository::attach(&root).expect("attach");
    cockpit_repository::confirm_profile_update(
        &root,
        "cargo",
        &["test".into(), "--workspace".into()],
    )
    .expect("confirm profile");
    let git = cockpit_git::GitRepository::discover(&root).expect("git");
    let clean = git.snapshot().expect("clean snapshot");
    let baseline = authorized_context(&root, &clean, &input());

    fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 2 }\n").expect("change source");
    let changed = git.snapshot().expect("changed snapshot");
    let changed_context = authorized_context(&root, &changed, &input());
    assert_ne!(baseline.content_digest, changed_context.content_digest);
    assert_ne!(
        baseline.diff.changed_paths_digest,
        changed_context.diff.changed_paths_digest
    );

    let mut runtime_input = input();
    runtime_input.runtime_digest = digest('c');
    let runtime_context = authorized_context(&root, &changed, &runtime_input);
    assert_ne!(
        changed_context.environment_digest,
        runtime_context.environment_digest
    );

    let mut scope_input = input();
    scope_input.scope = vec!["tests/**".into()];
    let scope_context = authorized_context(&root, &changed, &scope_input);
    assert_ne!(changed_context.scope_digest, scope_context.scope_digest);
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn governance_only_receipts_do_not_invalidate_source_content_identity() {
    let root = repository("governance-only", true);
    cockpit_repository::attach(&root).expect("attach");
    cockpit_repository::confirm_profile_update(
        &root,
        "cargo",
        &["test".into(), "--workspace".into()],
    )
    .expect("confirm profile");
    let git = cockpit_git::GitRepository::discover(&root).expect("git");
    let clean = git.snapshot().expect("snapshot");
    let baseline = authorized_context(&root, &clean, &input());

    fs::create_dir_all(root.join(".ai/notes")).expect("notes directory");
    fs::write(root.join(".ai/notes/observation.json"), b"{}\n").expect("receipt");
    let governance_changed = git.snapshot().expect("governance snapshot");
    let after = authorized_context(&root, &governance_changed, &input());

    assert_eq!(baseline.content_digest, after.content_digest);
    assert_eq!(
        baseline.diff.changed_paths_digest,
        after.diff.changed_paths_digest
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn profile_authorized_verification_reuses_exact_receipt_without_source_changes() {
    let root = repository("hot-reuse", true);
    cockpit_repository::attach(&root).expect("attach");
    let (program, args) = successful_command();
    cockpit_repository::confirm_profile_update(&root, program, &args).expect("confirm profile");
    let request = RepositoryVerificationRequest {
        node_id: "project-command-0".into(),
        program: program.into(),
        args,
        scope: vec!["**".into()],
        stage: "task".into(),
        runner: "local".into(),
        runtime_digest: digest('b'),
        base_commit: None,
        workers: 1,
        policy: RepositoryVerificationPolicy::ProfileAuthorized,
    };

    let first = run_repository_verification(&root, &request).expect("first verification");
    assert_eq!(first.receipt.nodes_executed, 1);
    assert_eq!(first.receipt.nodes_reused, 0);

    let second = run_repository_verification(&root, &request).expect("reused verification");
    assert_eq!(second.receipt.nodes_executed, 0);
    assert_eq!(second.receipt.nodes_reused, 1);
    assert_eq!(second.receipt.execution_elapsed_ms, 0);

    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn uncalibrated_unverified_headless_and_tampered_profiles_deny_reuse() {
    let uncalibrated = repository("uncalibrated", true);
    cockpit_repository::attach(&uncalibrated).expect("attach");
    let snapshot = cockpit_git::GitRepository::discover(&uncalibrated)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    assert!(matches!(
        assess_verification_reuse(&uncalibrated, &snapshot, &input()).expect("assess"),
        VerificationReuseAssessment::Denied { reason } if reason == "profile_not_calibrated"
    ));

    cockpit_repository::confirm_profile_update(
        &uncalibrated,
        "cargo",
        &["test".into(), "--workspace".into()],
    )
    .expect("confirm profile");
    let mut other = input();
    other.args = vec!["check".into()];
    assert!(matches!(
        assess_verification_reuse(&uncalibrated, &snapshot, &other).expect("assess"),
        VerificationReuseAssessment::Denied { reason } if reason == "command_not_profile_verified"
    ));

    let project_path = uncalibrated.join(".ai/project.json");
    let mut project: serde_json::Value =
        serde_json::from_slice(&fs::read(&project_path).expect("profile")).expect("profile JSON");
    project["profileDigest"] = serde_json::json!(digest('f'));
    fs::write(
        &project_path,
        serde_json::to_vec_pretty(&project).expect("encode profile"),
    )
    .expect("tamper profile");
    assert!(matches!(
        assess_verification_reuse(&uncalibrated, &snapshot, &input()).expect("assess"),
        VerificationReuseAssessment::Denied { reason } if reason == "profile_digest_mismatch"
    ));

    let headless = repository("headless", false);
    cockpit_repository::attach(&headless).expect("attach headless");
    let headless_snapshot = cockpit_git::GitRepository::discover(&headless)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    assert!(matches!(
        assess_verification_reuse(&headless, &headless_snapshot, &input()).expect("assess"),
        VerificationReuseAssessment::Denied { reason } if reason == "source_revision_unknown"
    ));

    fs::remove_dir_all(uncalibrated).expect("cleanup uncalibrated");
    fs::remove_dir_all(headless).expect("cleanup headless");
}

#[test]
fn oversized_governance_identity_files_fail_closed_with_bounded_reads() {
    for name in ["cockpit.toml", "project.json"] {
        let root = repository(&format!("oversized-{name}"), true);
        cockpit_repository::attach(&root).expect("attach");
        cockpit_repository::confirm_profile_update(
            &root,
            "cargo",
            &["test".into(), "--workspace".into()],
        )
        .expect("confirm profile");
        fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(root.join(".ai").join(name))
            .expect("identity file")
            .set_len(2 * 1024 * 1024)
            .expect("oversized identity file");
        let snapshot = cockpit_git::GitRepository::discover(&root)
            .expect("git")
            .snapshot()
            .expect("snapshot");

        assert!(
            assess_verification_reuse(&root, &snapshot, &input()).is_err(),
            "{name} must be rejected before an unbounded parse"
        );
        fs::remove_dir_all(root).expect("cleanup");
    }
}

#[test]
fn pr_base_revision_and_resolved_executable_bytes_are_independent_bindings() {
    let root = repository("base-toolchain", true);
    let tool = root
        .parent()
        .expect("fixture parent")
        .join(format!("cockpit-context-tool-{}", std::process::id()));
    fs::write(&tool, "tool-v1").expect("tool fixture");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("executable tool");
    }
    cockpit_repository::attach(&root).expect("attach");
    let program = tool.to_string_lossy().into_owned();
    cockpit_repository::confirm_profile_update(&root, &program, &[]).expect("confirm tool");
    let snapshot = cockpit_git::GitRepository::discover(&root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let mut request = input();
    request.program = program;
    request.args.clear();
    request.stage = "pr".into();
    request.base_commit = Some("4".repeat(40));
    let first = authorized_context(&root, &snapshot, &request);

    request.base_commit = Some("5".repeat(40));
    let moved_base = authorized_context(&root, &snapshot, &request);
    assert_ne!(first.diff.base_commit, moved_base.diff.base_commit);

    fs::write(&tool, "tool-v2").expect("mutate tool fixture");
    let changed_tool = authorized_context(&root, &snapshot, &request);
    assert_ne!(moved_base.toolchain_digest, changed_tool.toolchain_digest);

    fs::remove_file(tool).expect("cleanup tool");
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(windows)]
#[test]
fn batch_commands_fail_closed_instead_of_reusing_unbound_cmd_runtime() {
    let root = repository("batch-runtime", true);
    let tool = root.join("verified-tool.cmd");
    fs::write(&tool, "@echo off\r\nexit /b 0\r\n").expect("batch fixture");
    cockpit_repository::attach(&root).expect("attach");
    let program = tool.to_string_lossy().into_owned();
    cockpit_repository::confirm_profile_update(&root, &program, &[]).expect("confirm batch");
    let snapshot = cockpit_git::GitRepository::discover(&root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let mut request = input();
    request.program = program;
    request.args.clear();

    assert!(matches!(
        assess_verification_reuse(&root, &snapshot, &request).expect("assess batch"),
        VerificationReuseAssessment::Denied { reason } if reason == "toolchain_identity_unknown"
    ));

    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(windows)]
#[test]
fn reparse_point_executable_fails_closed_instead_of_reusing_a_movable_target() {
    use std::os::windows::fs::symlink_file;

    let root = repository("reparse-executable", true);
    let target = root.join("target.exe");
    let tool = root.join("verified-tool.exe");
    fs::copy(std::env::current_exe().expect("test executable"), &target).expect("target copy");
    symlink_file(&target, &tool).expect("executable symlink");
    cockpit_repository::attach(&root).expect("attach");
    let program = tool.to_string_lossy().into_owned();
    cockpit_repository::confirm_profile_update(&root, &program, &[]).expect("confirm tool");
    let snapshot = cockpit_git::GitRepository::discover(&root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let mut request = input();
    request.program = program;
    request.args.clear();

    assert!(matches!(
        assess_verification_reuse(&root, &snapshot, &request).expect("assess reparse executable"),
        VerificationReuseAssessment::Denied { reason } if reason == "toolchain_identity_unknown"
    ));
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn shebang_interpreter_bytes_are_part_of_toolchain_identity() {
    use std::os::unix::fs::PermissionsExt;

    let root = repository("shebang-interpreter", true);
    let fixture_parent = root.parent().expect("fixture parent");
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let interpreter = fixture_parent.join(format!(
        "cockpit-context-interpreter-{}-{sequence}",
        std::process::id()
    ));
    let tool = fixture_parent.join(format!(
        "cockpit-context-script-{}-{sequence}",
        std::process::id()
    ));
    fs::copy("/bin/sh", &interpreter).expect("copy interpreter");
    fs::set_permissions(&interpreter, fs::Permissions::from_mode(0o755))
        .expect("interpreter executable");
    fs::write(
        &tool,
        format!("#!{}\nexit 0\n", interpreter.to_string_lossy()),
    )
    .expect("script");
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("script executable");

    cockpit_repository::attach(&root).expect("attach");
    let program = tool.to_string_lossy().into_owned();
    cockpit_repository::confirm_profile_update(&root, &program, &[]).expect("confirm tool");
    let snapshot = cockpit_git::GitRepository::discover(&root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let mut request = input();
    request.program = program;
    request.args.clear();
    let first = authorized_context(&root, &snapshot, &request);

    let mut interpreter_bytes = fs::read(&interpreter).expect("interpreter bytes");
    interpreter_bytes.push(0);
    fs::write(&interpreter, interpreter_bytes).expect("mutate interpreter");
    let changed = authorized_context(&root, &snapshot, &request);
    assert_ne!(first.toolchain_digest, changed.toolchain_digest);

    fs::remove_file(tool).expect("cleanup tool");
    fs::remove_file(interpreter).expect("cleanup interpreter");
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn nested_script_interpreter_fails_closed_instead_of_authorizing_reuse() {
    use std::os::unix::fs::PermissionsExt;

    let root = repository("nested-interpreter", true);
    let nested = root.join("nested-interpreter.sh");
    let tool = root.join("verified-tool");
    fs::write(&nested, "#!/bin/sh\nexec /bin/sh \"$@\"\n").expect("nested interpreter");
    fs::write(&tool, format!("#!{}\nexit 0\n", nested.to_string_lossy())).expect("verified tool");
    fs::set_permissions(&nested, fs::Permissions::from_mode(0o755)).expect("nested executable");
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("tool executable");
    cockpit_repository::attach(&root).expect("attach");
    let program = tool.to_string_lossy().into_owned();
    cockpit_repository::confirm_profile_update(&root, &program, &[]).expect("confirm tool");
    let snapshot = cockpit_git::GitRepository::discover(&root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let mut request = input();
    request.program = program;
    request.args.clear();

    assert!(matches!(
        assess_verification_reuse(&root, &snapshot, &request).expect("assess nested interpreter"),
        VerificationReuseAssessment::Denied { reason } if reason == "toolchain_identity_unknown"
    ));

    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn unknown_profile_command_and_config_fields_deny_reuse() {
    let root = repository("unknown-identity-fields", true);
    cockpit_repository::attach(&root).expect("attach");
    cockpit_repository::confirm_profile_update(
        &root,
        "cargo",
        &["test".into(), "--workspace".into()],
    )
    .expect("confirm profile");
    let snapshot = cockpit_git::GitRepository::discover(&root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let profile_path = root.join(".ai/project.json");
    let original_profile = fs::read(&profile_path).expect("profile");

    for location in ["top", "command"] {
        let mut profile: serde_json::Value =
            serde_json::from_slice(&original_profile).expect("profile JSON");
        if location == "top" {
            profile["futurePolicy"] = serde_json::json!("strict");
        } else {
            profile["tests"][0]["futurePolicy"] = serde_json::json!("strict");
        }
        fs::write(
            &profile_path,
            serde_json::to_vec_pretty(&profile).expect("profile bytes"),
        )
        .expect("mutate profile");
        assert!(matches!(
            assess_verification_reuse(&root, &snapshot, &input()).expect("assessment"),
            VerificationReuseAssessment::Denied { reason } if reason == "profile_untrusted"
        ));
    }

    fs::write(&profile_path, original_profile).expect("restore profile");
    let config_path = root.join(".ai/cockpit.toml");
    let mut config = fs::read_to_string(&config_path).expect("config");
    config.push_str("future_policy = \"strict\"\n");
    fs::write(&config_path, config).expect("mutate config");
    assert!(assess_verification_reuse(&root, &snapshot, &input()).is_err());

    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn ambiguous_env_shebang_options_deny_reuse() {
    use std::os::unix::fs::PermissionsExt;

    let root = repository("ambiguous-env-shebang", true);
    let tool = root
        .parent()
        .expect("fixture parent")
        .join(format!("cockpit-ambiguous-env-tool-{}", std::process::id()));
    fs::write(&tool, "#!/usr/bin/env -u sh /bin/true\n").expect("script");
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("executable");
    cockpit_repository::attach(&root).expect("attach");
    let program = tool.to_string_lossy().into_owned();
    cockpit_repository::confirm_profile_update(&root, &program, &[]).expect("confirm");
    let snapshot = cockpit_git::GitRepository::discover(&root)
        .expect("git")
        .snapshot()
        .expect("snapshot");
    let mut request = input();
    request.program = program;
    request.args.clear();

    assert!(matches!(
        assess_verification_reuse(&root, &snapshot, &request).expect("assessment"),
        VerificationReuseAssessment::Denied { reason } if reason == "toolchain_identity_unknown"
    ));
    fs::remove_file(tool).expect("cleanup tool");
    fs::remove_dir_all(root).expect("cleanup");
}

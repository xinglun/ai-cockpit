use cockpit_core::Digest;
use cockpit_git::RepositorySnapshot;
use cockpit_protocol::{ProjectProfile, QualityCommand};
use cockpit_repository::{EvolutionClass, RepositoryObservation, classify_evolution};
use std::path::PathBuf;

fn observation() -> RepositoryObservation {
    RepositoryObservation {
        snapshot_digest: Digest::sha256_bytes(b"snapshot"),
        dependency_fingerprint: Digest::sha256_bytes(b"dependencies"),
        languages: vec![],
        build_systems: vec![],
        test_roots: vec!["tests/**".into()],
        quality_commands: vec![QualityCommand {
            program: "cargo".into(),
            args: vec!["test".into()],
            state: "verified".into(),
            execution_recipe: None,
        }],
        ci_surfaces: vec![],
        critical_domains: vec![],
        files_read: 0,
        cache_hit: false,
    }
}

fn snapshot(paths: &[&str]) -> RepositorySnapshot {
    RepositorySnapshot {
        root: PathBuf::from("/tmp/repo"),
        git_root: PathBuf::from("/tmp/repo"),
        head: Some("0123456789abcdef0123456789abcdef01234567".into()),
        changed_paths: paths.iter().map(|path| (*path).into()).collect(),
        change_evidence: vec![],
        git_calls: 2,
        tree_digest: "sha256:tree".into(),
        diff_digest: "sha256:diff".into(),
        dependency_fingerprint: "sha256:dependencies".into(),
        files_read: 0,
        files_hashed: 0,
        bytes_read: 0,
        bytes_hashed: 0,
        source_tree_digest: None,
    }
}

#[test]
fn new_test_in_existing_root_is_l1_auto_absorb() {
    let profile = ProjectProfile {
        profile_version: 1,
        repository_id: "repo".into(),
        tests: observation().quality_commands.clone(),
        build_systems: vec!["cargo".into()],
    };
    let events = classify_evolution(
        &profile,
        &observation(),
        &snapshot(&["tests/payment/refund.rs"]),
    );
    assert!(
        events
            .iter()
            .any(|event| event.class == EvolutionClass::L1 && event.action == "auto_absorb")
    );
}

#[test]
fn new_framework_is_l2_and_needs_confirmation() {
    let profile = ProjectProfile {
        profile_version: 1,
        repository_id: "repo".into(),
        tests: observation().quality_commands.clone(),
        build_systems: vec!["cargo".into()],
    };
    let events = classify_evolution(
        &profile,
        &observation(),
        &snapshot(&["playwright.config.ts"]),
    );
    assert!(
        events
            .iter()
            .any(|event| event.class == EvolutionClass::L2 && event.action == "needs_confirmation")
    );
}

#[cfg(unix)]
fn c18_recipe_fixture() -> tempfile::TempDir {
    use std::{fs, process::Command};
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("README.md"), "owned recipe fixture\n").unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["add", "README.md"],
        vec![
            "-c",
            "user.name=Recipe Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "fixture",
        ],
    ] {
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(root.path())
                .args(args)
                .status()
                .unwrap()
                .success()
        );
    }
    cockpit_repository::attach(root.path()).unwrap();
    root
}

#[cfg(unix)]
fn c18_recipe(value: &str) -> cockpit_protocol::VerificationExecutionRecipe {
    use cockpit_protocol::{VerificationEnvironmentVariable as Env, VerificationExecutionRecipe};
    VerificationExecutionRecipe {
        schema_version: 1,
        clear_environment: true,
        environment: vec![
            Env {
                name: "PATH".into(),
                value: std::env::var("PATH").unwrap(),
            },
            Env {
                name: "C18_RECIPE_ONLY".into(),
                value: value.into(),
            },
        ],
    }
}

#[cfg(unix)]
fn c18_recipe_command() -> (String, Vec<String>) {
    (std::fs::canonicalize("/bin/sh").unwrap().to_string_lossy().into_owned(),
     vec!["-c".into(), "test \"${C18_RECIPE_ONLY-}\" = present && test \"${WI1113_RUN_OUT+x}\" != x && printf c18-cleared".into()])
}

#[test]
fn c18_legacy_quality_wire_and_profile_digest_keep_absent_recipe_unchanged() {
    let q = QualityCommand {
        program: "cargo".into(),
        args: vec!["test".into()],
        state: "verified".into(),
        execution_recipe: None,
    };
    assert_eq!(
        serde_json::to_value(&q).unwrap(),
        serde_json::json!({"program":"cargo","args":["test"],"state":"verified"})
    );
    assert!(serde_json::from_value::<cockpit_protocol::VerificationExecutionRecipe>(serde_json::json!({"schemaVersion":1,"clearEnvironment":true,"environment":[],"trusted":true})).is_err());
}

#[cfg(unix)]
#[test]
fn c18_native_recipe_confirmation_binds_actual_recipe_and_preserves_original_commands() {
    let root = c18_recipe_fixture();
    let (program, args) = c18_recipe_command();
    let before: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.path().join(".ai/project.json")).unwrap())
            .unwrap();
    let result = cockpit_repository::confirm_profile_update_with_execution_recipe(
        root.path(),
        &program,
        &args,
        Some(c18_recipe("present")),
    );
    assert!(
        result.is_ok(),
        "TARGET: native recipe confirmation missing: {result:?}"
    );
    let profile = result.unwrap();
    let selected = profile
        .tests
        .iter()
        .filter(|q| q.program == program && q.args == args && q.state == "verified")
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].execution_recipe, Some(c18_recipe("present")));
    assert_eq!(
        profile.profile_version,
        before["profileVersion"].as_u64().unwrap() + 1
    );
    assert!(
        root.path()
            .join(format!(
                ".ai/decisions/profile-v{}.json",
                profile.profile_version
            ))
            .is_file()
    );
    for old in before["tests"].as_array().unwrap() {
        assert!(
            serde_json::to_value(&profile.tests)
                .unwrap()
                .as_array()
                .unwrap()
                .contains(old)
        );
    }
}

#[cfg(unix)]
#[test]
fn c18_recipe_is_real_child_environment_and_exact_repeat_can_reuse() {
    use cockpit_repository::{RepositoryVerificationPolicy, RepositoryVerificationRequest};
    let root = c18_recipe_fixture();
    let (program, args) = c18_recipe_command();
    let result = cockpit_repository::confirm_profile_update_with_execution_recipe(
        root.path(),
        &program,
        &args,
        Some(c18_recipe("present")),
    );
    assert!(
        result.is_ok(),
        "TARGET: execution recipe API unavailable: {result:?}"
    );
    let request = RepositoryVerificationRequest {
        node_id: "recipe-node".into(),
        program,
        args,
        scope: vec!["**".into()],
        stage: "task".into(),
        runner: "local".into(),
        runtime_digest: Digest::sha256_bytes(b"fixture-runtime").to_string(),
        base_commit: None,
        workers: 1,
        work_item_id: None,
        timeout_seconds: Some(5),
        policy: RepositoryVerificationPolicy::ProfileAuthorized,
    };
    let first = cockpit_repository::run_repository_verification(root.path(), &request).unwrap();
    assert!(
        first.receipt.passed,
        "actual cleared child must execute successfully"
    );
    assert_eq!(first.receipt.nodes_executed, 1);
    assert_eq!(first.receipt.nodes_reused, 0);
    assert_eq!(
        first.receipt.execution_records[0].stdout_hex,
        "6331382d636c6561726564"
    );
    let second = cockpit_repository::run_repository_verification(root.path(), &request).unwrap();
    assert!(second.receipt.passed);
    assert_eq!(second.receipt.nodes_reused, 1);
    assert_eq!(second.receipt.processes_spawned, 0);
}

#[cfg(unix)]
#[test]
fn c18_invalid_recipe_rejection_precedes_profile_or_decision_writes() {
    use cockpit_protocol::VerificationEnvironmentVariable as Env;
    let root = c18_recipe_fixture();
    let (program, args) = c18_recipe_command();
    let path = root.path().join(".ai/project.json");
    let before = std::fs::read(&path).unwrap();
    let mut cases = Vec::new();
    let mut q = c18_recipe("present");
    q.schema_version = 2;
    cases.push((q, "execution_recipe_schema"));
    let mut q = c18_recipe("present");
    q.clear_environment = false;
    cases.push((q, "execution_recipe_requires_clear_environment"));
    let mut q = c18_recipe("present");
    q.environment.push(Env {
        name: "PATH".into(),
        value: "different".into(),
    });
    cases.push((q, "execution_recipe_duplicate_key"));
    let mut q = c18_recipe("present");
    q.environment.push(Env {
        name: "BAD=KEY".into(),
        value: "x".into(),
    });
    cases.push((q, "execution_recipe_key"));
    let mut q = c18_recipe("present");
    q.environment[0].value = "x\0y".into();
    cases.push((q, "execution_recipe_value"));
    let mut q = c18_recipe("present");
    q.environment[1].value = "x".repeat(65537);
    cases.push((q, "execution_recipe_budget"));
    for (recipe, reason) in cases {
        let error = cockpit_repository::confirm_profile_update_with_execution_recipe(
            root.path(),
            &program,
            &args,
            Some(recipe),
        )
        .unwrap_err()
        .to_string();
        assert!(
            error.contains(reason),
            "TARGET: expected {reason}, got {error}"
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert!(!root.path().join(".ai/decisions/profile-v2.json").exists());
    }
}

#[cfg(unix)]
fn c18_request(
    program: String,
    args: Vec<String>,
) -> cockpit_repository::RepositoryVerificationRequest {
    cockpit_repository::RepositoryVerificationRequest {
        node_id: "recipe-negative-node".into(),
        program,
        args,
        scope: vec!["**".into()],
        stage: "task".into(),
        runner: "local".into(),
        runtime_digest: Digest::sha256_bytes(b"fixture-runtime").to_string(),
        base_commit: None,
        workers: 1,
        work_item_id: None,
        timeout_seconds: Some(5),
        policy: cockpit_repository::RepositoryVerificationPolicy::ProfileAuthorized,
    }
}

#[cfg(unix)]
#[test]
fn c18_recipe_path_and_env_shebang_resolve_only_from_the_child_environment() {
    use std::os::unix::fs::PermissionsExt;
    let root = c18_recipe_fixture();
    let bin = root.path().join("recipe-bin");
    std::fs::create_dir(&bin).unwrap();
    std::fs::copy(
        std::fs::canonicalize("/bin/sh").unwrap(),
        bin.join("c18-interpreter"),
    )
    .unwrap();
    let tool = bin.join("c18-tool");
    std::fs::write(&tool, "#!/usr/bin/env c18-interpreter\ntest \"${HOME+x}\" != x && test \"${C18_RECIPE_ONLY-}\" = present && printf recipe-path\n").unwrap();
    std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(0o755)).unwrap();
    let mut recipe = c18_recipe("present");
    recipe.environment[0].value = bin.to_string_lossy().into_owned();
    cockpit_repository::confirm_profile_update_with_execution_recipe(
        root.path(),
        "c18-tool",
        &[],
        Some(recipe),
    )
    .unwrap();
    let request = c18_request("c18-tool".into(), vec![]);
    let first = cockpit_repository::run_repository_verification(root.path(), &request).unwrap();
    assert!(first.receipt.passed);
    assert_eq!(first.receipt.processes_spawned, 1);
    assert_eq!(
        first.receipt.execution_records[0].stdout_hex,
        "7265636970652d70617468"
    );
    let second = cockpit_repository::run_repository_verification(root.path(), &request).unwrap();
    assert!(second.receipt.passed);
    assert_eq!(second.receipt.nodes_reused, 1);
    assert_eq!(second.receipt.processes_spawned, 0);
}

#[cfg(unix)]
#[test]
fn c18_recipe_failure_does_not_reuse_a_previous_success_and_policies_stay_protected() {
    use cockpit_repository::{RepositoryVerificationPolicy, run_repository_verification};
    let root = c18_recipe_fixture();
    let (program, args) = c18_recipe_command();
    cockpit_repository::confirm_profile_update_with_execution_recipe(
        root.path(),
        &program,
        &args,
        Some(c18_recipe("present")),
    )
    .unwrap();
    let mut request = c18_request(program, args);
    let first = run_repository_verification(root.path(), &request).unwrap();
    assert!(first.receipt.passed);
    assert_eq!(
        run_repository_verification(root.path(), &request)
            .unwrap()
            .receipt
            .nodes_reused,
        1
    );
    for policy in [
        RepositoryVerificationPolicy::NeverReuse,
        RepositoryVerificationPolicy::Protected(cockpit_verification::ProtectedGateClass::Scope),
    ] {
        request.policy = policy;
        for _ in 0..2 {
            let run = run_repository_verification(root.path(), &request).unwrap();
            assert!(run.receipt.passed);
            assert_eq!(run.receipt.nodes_reused, 0);
            assert_eq!(run.receipt.processes_spawned, 1);
            assert!(run.receipt.receipt_candidates.is_empty());
            if matches!(request.policy, RepositoryVerificationPolicy::Protected(_)) {
                assert_eq!(run.receipt.protected_nodes_executed, 1);
            }
        }
    }
    request.policy = RepositoryVerificationPolicy::ProfileAuthorized;
    cockpit_repository::confirm_profile_update_with_execution_recipe(
        root.path(),
        &request.program,
        &request.args,
        Some(c18_recipe("missing")),
    )
    .unwrap();
    for _ in 0..2 {
        let run = run_repository_verification(root.path(), &request).unwrap();
        assert!(!run.receipt.passed);
        assert_eq!(run.receipt.nodes_reused, 0);
        assert_eq!(run.receipt.processes_spawned, 1);
        assert_ne!(
            run.receipt.execution_records[0].command_digest,
            first.receipt.execution_records[0].command_digest
        );
    }
}

#[cfg(unix)]
#[test]
fn c18_corrupt_recipe_fails_before_spawn_instead_of_falling_back_to_ambient_environment() {
    let root = c18_recipe_fixture();
    let program = std::fs::canonicalize("/bin/sh")
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let args = vec![
        "-c".into(),
        "printf x >> .ai/c18-child-runs; test \"${C18_RECIPE_ONLY-}\" = present".into(),
    ];
    cockpit_repository::confirm_profile_update_with_execution_recipe(
        root.path(),
        &program,
        &args,
        Some(c18_recipe("present")),
    )
    .unwrap();
    let request = c18_request(program, args);
    assert!(
        cockpit_repository::run_repository_verification(root.path(), &request)
            .unwrap()
            .receipt
            .passed
    );
    let marker = root.path().join(".ai/c18-child-runs");
    assert_eq!(std::fs::read(&marker).unwrap(), b"x");
    let path = root.path().join(".ai/project.json");
    let original: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    for (mutation, reason) in [
        ("digest", "execution_recipe_profile_digest_mismatch"),
        ("schema", "execution_recipe_schema"),
        ("clear", "execution_recipe_requires_clear_environment"),
        ("duplicate", "execution_recipe_duplicate_key"),
        ("null", "execution_recipe_command_ambiguous_or_unconfirmed"),
    ] {
        let mut value = original.clone();
        let command = value["tests"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|command| {
                command["program"] == request.program
                    && command["args"] == serde_json::json!(request.args)
            })
            .unwrap();
        match mutation {
            "digest" => {
                command["executionRecipe"]["environment"][1]["value"] =
                    serde_json::json!("tampered")
            }
            "schema" => command["executionRecipe"]["schemaVersion"] = serde_json::json!(2),
            "clear" => command["executionRecipe"]["clearEnvironment"] = serde_json::json!(false),
            "duplicate" => {
                let env = command["executionRecipe"]["environment"]
                    .as_array_mut()
                    .unwrap();
                env.push(env[0].clone());
            }
            "null" => command["executionRecipe"] = serde_json::Value::Null,
            _ => unreachable!(),
        }
        if mutation != "digest" {
            let profile: cockpit_repository::AttachedProfile =
                serde_json::from_value(value.clone()).unwrap();
            value["profileDigest"] = serde_json::to_value(
                cockpit_protocol::digest_json(&ProjectProfile {
                    profile_version: profile.profile_version,
                    repository_id: profile.repository_id,
                    tests: profile.tests,
                    build_systems: profile.build_systems,
                })
                .unwrap(),
            )
            .unwrap();
        }
        std::fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
        let error = cockpit_repository::run_repository_verification(root.path(), &request)
            .unwrap_err()
            .to_string();
        assert!(error.contains(reason), "expected {reason}, got {error}");
        assert_eq!(
            std::fs::read(&marker).unwrap(),
            b"x",
            "invalid recipe must not spawn"
        );
    }

    // The duplicate is visible only after JSON unescapes the key. A Value
    // alone silently retains the second (valid) recipe and its original digest.
    let escaped_duplicate = serde_json::to_string(&original).unwrap().replace(
        "\"executionRecipe\":",
        "\"\\u0065xecutionRecipe\":null,\"\\u0065xecutionRecipe\":",
    );
    assert!(!escaped_duplicate.contains("executionRecipe"));
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&escaped_duplicate).unwrap(),
        original
    );
    std::fs::write(&path, escaped_duplicate.as_bytes()).unwrap();
    let error = cockpit_repository::run_repository_verification(root.path(), &request)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("execution_recipe_profile_untrusted"),
        "{error}"
    );
    assert_eq!(std::fs::read(&path).unwrap(), escaped_duplicate.as_bytes());
    assert_eq!(
        std::fs::read(&marker).unwrap(),
        b"x",
        "escaped duplicate must not spawn"
    );
}

#[cfg(unix)]
#[test]
fn c18_legacy_malformed_profile_still_executes_without_reuse() {
    let root = c18_recipe_fixture();
    std::fs::write(
        root.path().join(".ai/project.json"),
        "legacy malformed profile",
    )
    .unwrap();
    let request = c18_request(
        std::fs::canonicalize("/bin/sh")
            .unwrap()
            .to_string_lossy()
            .into_owned(),
        vec!["-c".into(), "printf legacy-compatible".into()],
    );
    let run = cockpit_repository::run_repository_verification(root.path(), &request).unwrap();
    assert!(run.receipt.passed);
    assert_eq!(run.receipt.nodes_reused, 0);
    assert_eq!(run.receipt.processes_spawned, 1);
    assert_eq!(
        run.receipt.execution_records[0].stdout_hex,
        "6c65676163792d636f6d70617469626c65"
    );
}

#[cfg(unix)]
#[test]
fn c18_recipe_rustup_proxy_is_explicitly_unsupported_without_changing_legacy_execution() {
    use std::os::unix::fs::symlink;
    let root = c18_recipe_fixture();
    let bin = root.path().join("proxy-bin");
    std::fs::create_dir(&bin).unwrap();
    std::fs::copy(
        std::fs::canonicalize("/bin/sh").unwrap(),
        bin.join("rustup"),
    )
    .unwrap();
    symlink("rustup", bin.join("cargo")).unwrap();
    let program = bin.join("cargo").to_string_lossy().into_owned();
    let args = vec!["-c".into(), "printf legacy-proxy".into()];
    let request = c18_request(program.clone(), args.clone());
    cockpit_repository::confirm_profile_update(root.path(), &program, &args).unwrap();
    let legacy = cockpit_repository::run_repository_verification(root.path(), &request).unwrap();
    assert!(legacy.receipt.passed);
    assert_eq!(
        legacy.receipt.execution_records[0].stdout_hex,
        "6c65676163792d70726f7879"
    );
    let mut recipe = c18_recipe("present");
    recipe.environment[0].value = bin.to_string_lossy().into_owned();
    cockpit_repository::confirm_profile_update_with_execution_recipe(
        root.path(),
        &program,
        &args,
        Some(recipe),
    )
    .unwrap();
    let error = cockpit_repository::run_repository_verification(root.path(), &request)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("execution_recipe_rustup_proxy_unsupported"),
        "{error}"
    );
}

#[cfg(unix)]
#[test]
fn c18_late_recipe_removal_fails_closed_after_actual_execution() {
    let root = c18_recipe_fixture();
    let (program, args) = c18_recipe_command();
    cockpit_repository::confirm_profile_update_with_execution_recipe(
        root.path(),
        &program,
        &args,
        Some(c18_recipe("present")),
    )
    .unwrap();
    let mut request = c18_request(program, args);
    assert!(
        cockpit_repository::run_repository_verification(root.path(), &request)
            .unwrap()
            .receipt
            .passed
    );
    request.policy = cockpit_repository::RepositoryVerificationPolicy::NeverReuse;
    let path = root.path().join(".ai/project.json");
    let error = cockpit_repository::run_repository_verification_with_process_observer(
        root.path(),
        &request,
        move |_, _, started| {
            if started {
                std::fs::write(&path, "legacy-shaped metadata loss")
                    .map_err(|error| error.to_string())?;
            }
            Ok(())
        },
    )
    .unwrap_err()
    .to_string();
    assert!(
        error.contains("execution_recipe_profile_changed"),
        "{error}"
    );
}

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

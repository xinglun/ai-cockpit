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
fn local_profile_fixture() -> tempfile::TempDir {
    use cockpit_protocol::{
        ApprovalMode, EvidenceAssurance, GovernancePolicy, GovernancePolicyDocument, PolicyLayer,
        PolicyRule, VerificationRequirement, VerificationTier,
    };
    use std::{fs, process::Command};
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("README.md"), "owned profile fixture\n").unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["add", "README.md"],
        vec![
            "-c",
            "user.name=Profile Fixture",
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
    let policy = GovernancePolicyDocument {
        schema_version: 1,
        organization: None,
        project: Some(GovernancePolicy {
            policy_id: "c18-fixture-five-second-verification".into(),
            layer: PolicyLayer::Project,
            rules: vec![PolicyRule {
                operation: "modify_source".into(),
                approval_mode: ApprovalMode::NoHumanApprovalForLowRisk,
                required_evidence: Vec::new(),
                verification_requirement: Some(VerificationRequirement {
                    schema_version: 1,
                    required_tier: VerificationTier::T0,
                    required_assurance: EvidenceAssurance::SelfDeclared,
                    policy_refs: vec!["c18-fixture-five-second-verification".into()],
                    stage_refs: vec!["task".into()],
                    gate_refs: Vec::new(),
                    reason: "authorize only the owned C18 fixture's five-second verification"
                        .into(),
                    max_timeout_seconds: Some(5),
                }),
            }],
        }),
    };
    fs::write(
        root.path().join(".ai/policy.json"),
        serde_json::to_vec_pretty(&policy).unwrap(),
    )
    .unwrap();
    root
}

#[cfg(unix)]
fn local_profile_request(
    program: String,
    args: Vec<String>,
) -> cockpit_repository::RepositoryVerificationRequest {
    cockpit_repository::RepositoryVerificationRequest {
        node_id: "profile-compatibility-node".into(),
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
fn legacy_malformed_profile_still_executes_without_reuse() {
    let root = local_profile_fixture();
    std::fs::write(
        root.path().join(".ai/project.json"),
        "legacy malformed profile",
    )
    .unwrap();
    let request = local_profile_request(
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

#[test]
fn ordinary_quality_command_wire_has_no_execution_recipe_extension() {
    let wire = serde_json::json!({"program":"cargo","args":["test"],"state":"verified"});
    let command: QualityCommand = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(serde_json::to_value(command).unwrap(), wire);
    let mut retired = wire;
    retired["executionRecipe"] =
        serde_json::json!({"schemaVersion":1,"clearEnvironment":true,"environment":[]});
    let error = serde_json::from_value::<QualityCommand>(retired).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("unknown field `executionRecipe`")
    );
}

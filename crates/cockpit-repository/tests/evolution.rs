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
        git_calls: 2,
        tree_digest: "sha256:tree".into(),
        diff_digest: "sha256:diff".into(),
        dependency_fingerprint: "sha256:dependencies".into(),
        files_read: 0,
        files_hashed: 0,
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

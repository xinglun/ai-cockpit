use cockpit_core::Digest;
use cockpit_git::GitRepository;
use cockpit_protocol::{
    ProjectCapabilityDeclaration, ProjectProfilePolicy, ProjectSuccessCriteriaDeclaration,
};
use cockpit_repository::{
    attach, capability_truth_registry, project_governance_projection, repository_id,
    snapshot_digest,
};
use std::{fs, process::Command};

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("tempdir");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    attach(directory.path()).expect("attach");
    directory
}

fn snapshot(root: &std::path::Path) -> Digest {
    let git = GitRepository::discover(root).expect("discover");
    let snapshot = git.snapshot().expect("snapshot");
    snapshot_digest(&snapshot).expect("snapshot digest")
}

fn write_declarations(root: &std::path::Path, digest: Option<Digest>) {
    let project = root.join(".ai/project");
    fs::create_dir_all(&project).expect("project directory");
    let repository_id = repository_id(root).to_string();
    let capabilities = ProjectCapabilityDeclaration {
        schema_version: 1,
        repository_id: repository_id.clone(),
        repository_snapshot_digest: digest.clone(),
        capabilities: vec!["ai_governance".into(), "documentation".into()],
        non_capabilities: vec!["physical_operation".into()],
        critical_domains: vec!["release".into()],
        operation_mappings: [
            (
                "repository_governance.modify".into(),
                vec!["ai_governance".into()],
            ),
            ("documentation.modify".into(), vec!["documentation".into()]),
        ]
        .into_iter()
        .collect(),
    };
    let criteria = ProjectSuccessCriteriaDeclaration {
        schema_version: 1,
        repository_id: repository_id.clone(),
        repository_snapshot_digest: digest.clone(),
        work_item_id: "WI-276".into(),
        criteria: vec![cockpit_protocol::ProjectSuccessCriterion {
            id: "SC-1".into(),
            statement: "Projection remains repository-bound.".into(),
            evidence_hints: vec!["tests/project_governance.rs".into()],
        }],
    };
    let policy = ProjectProfilePolicy {
        schema_version: 1,
        repository_id,
        repository_snapshot_digest: digest,
        approved_boundaries: cockpit_protocol::ProjectBoundaries {
            production_roots: vec!["crates/**".into()],
            feature_roots: vec!["crates/**".into()],
            test_roots: vec!["tests/**".into()],
            generated_paths: vec!["target/**".into()],
            critical_paths: vec![".ai/**".into()],
        },
        critical_domains: vec!["release".into()],
        review_requirements: vec!["quality".into()],
        unknowns: Vec::new(),
    };
    for (name, value) in [
        (
            "capabilities.json",
            serde_json::to_value(capabilities).expect("capabilities"),
        ),
        (
            "success_criteria.json",
            serde_json::to_value(criteria).expect("criteria"),
        ),
        (
            "profile-policy.json",
            serde_json::to_value(policy).expect("policy"),
        ),
    ] {
        fs::write(
            project.join(name),
            serde_json::to_vec_pretty(&value).expect("encode declaration"),
        )
        .expect("write declaration");
    }
}

#[test]
fn projection_is_valid_identity_bound_and_read_only() {
    let directory = repository();
    let digest = snapshot(directory.path());
    write_declarations(directory.path(), Some(digest.clone()));
    let before = fs::read_dir(directory.path().join(".ai/project"))
        .expect("before")
        .map(|entry| {
            let entry = entry.expect("entry");
            (entry.file_name(), fs::read(entry.path()).expect("bytes"))
        })
        .collect::<Vec<_>>();

    let git = GitRepository::discover(directory.path()).expect("discover");
    let current = git.snapshot().expect("snapshot");
    let projection = project_governance_projection(directory.path(), &current).expect("projection");
    assert_eq!(
        projection.repository_id,
        repository_id(directory.path()).to_string()
    );
    assert_eq!(
        projection.snapshot_digest,
        snapshot_digest(&current).expect("digest")
    );
    assert!(projection.capabilities_digest.is_some());
    assert!(projection.success_criteria_digest.is_some());
    assert_eq!(
        projection
            .success_criteria
            .as_ref()
            .expect("visible criteria")
            .criteria[0]
            .id,
        "SC-1"
    );
    assert!(projection.profile_policy_digest.is_some());
    assert!(projection.unknowns.is_empty());
    let registry = capability_truth_registry(directory.path()).expect("registry");
    assert_eq!(registry.project_governance, Some(projection));

    let after = fs::read_dir(directory.path().join(".ai/project"))
        .expect("after")
        .map(|entry| {
            let entry = entry.expect("entry");
            (entry.file_name(), fs::read(entry.path()).expect("bytes"))
        })
        .collect::<Vec<_>>();
    assert_eq!(before, after, "projection must not write declarations");
}

#[test]
fn projection_reports_missing_malformed_foreign_and_stale_inputs() {
    let directory = repository();
    let digest = snapshot(directory.path());
    write_declarations(directory.path(), Some(digest));
    fs::write(
        directory.path().join(".ai/project/capabilities.json"),
        br#"{"schemaVersion":1,"repositoryId":"sha256:foreign","repositorySnapshotDigest":"sha256:stale","capabilities":[],"nonCapabilities":[],"criticalDomains":[],"operationMappings":{}}"#,
    )
    .expect("foreign declaration");
    fs::write(
        directory.path().join(".ai/project/success_criteria.json"),
        br#"{"schemaVersion":1,"repositoryId":"sha256:repo","repositorySnapshotDigest":"sha256:stale","workItemId":"WI","criteria":[{"id":"x","statement":"x"}],"unexpected":true}"#,
    )
    .expect("malformed declaration");
    let project_policy = directory.path().join(".ai/project/profile-policy.json");
    let mut policy: serde_json::Value =
        serde_json::from_slice(&fs::read(&project_policy).expect("policy")).expect("policy json");
    policy["repositorySnapshotDigest"] = serde_json::json!(Digest::sha256_bytes(b"stale"));
    fs::write(
        &project_policy,
        serde_json::to_vec_pretty(&policy).expect("policy"),
    )
    .expect("stale policy");

    let git = GitRepository::discover(directory.path()).expect("discover");
    let current = git.snapshot().expect("snapshot");
    let projection = project_governance_projection(directory.path(), &current).expect("projection");
    for code in [
        "project_capabilities_repository_mismatch",
        "project_success_criteria_invalid",
        "project_profile_policy_stale",
    ] {
        assert!(projection.unknowns.contains(&code.into()), "missing {code}");
    }
    assert!(projection.capabilities_digest.is_none());
    assert!(projection.success_criteria_digest.is_none());
    assert!(projection.profile_policy_digest.is_none());
}

#[cfg(unix)]
#[test]
fn projection_rejects_symlinked_declaration() {
    use std::os::unix::fs::symlink;
    let directory = repository();
    let digest = snapshot(directory.path());
    write_declarations(directory.path(), Some(digest));
    let target = directory.path().join("outside-capabilities.json");
    fs::write(
        &target,
        fs::read(directory.path().join(".ai/project/capabilities.json")).expect("bytes"),
    )
    .expect("target");
    let path = directory.path().join(".ai/project/capabilities.json");
    fs::remove_file(&path).expect("remove");
    symlink(&target, &path).expect("symlink");
    let git = GitRepository::discover(directory.path()).expect("discover");
    let current = git.snapshot().expect("snapshot");
    let projection = project_governance_projection(directory.path(), &current).expect("projection");
    assert!(
        projection
            .unknowns
            .contains(&"project_capabilities_symlink".into())
    );
    assert!(projection.capabilities_digest.is_none());
}

#[test]
fn projection_isolated_between_repositories() {
    let left = repository();
    let right = repository();
    write_declarations(left.path(), Some(snapshot(left.path())));
    write_declarations(right.path(), Some(snapshot(right.path())));
    let left_snapshot = GitRepository::discover(left.path())
        .expect("left discover")
        .snapshot()
        .expect("left snapshot");
    let right_snapshot = GitRepository::discover(right.path())
        .expect("right discover")
        .snapshot()
        .expect("right snapshot");
    let left_projection =
        project_governance_projection(left.path(), &left_snapshot).expect("left projection");
    let right_projection =
        project_governance_projection(right.path(), &right_snapshot).expect("right projection");
    assert_ne!(
        left_projection.repository_id,
        right_projection.repository_id
    );
    assert_ne!(
        left_projection.capabilities_digest,
        right_projection.capabilities_digest
    );
    assert!(left_projection.unknowns.is_empty());
    assert!(right_projection.unknowns.is_empty());
}

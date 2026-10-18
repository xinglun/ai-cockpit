//! P2b wiring tests. No production source or permission is installed.
use cockpit_core::Digest;
use serde_json::json;
use std::{fs, path::Path};
use tempfile::{TempDir, tempdir};

const WI: &str = "WI-AUTHORITY-FIXTURE";

fn fixture(required: bool) -> TempDir {
    let dir = tempdir().unwrap();
    let active = dir.path().join(".ai/work-items/active");
    fs::create_dir_all(&active).unwrap();
    let digest = Digest::sha256_bytes(b"isolated authority wiring fixture");
    let mut contract = json!({
        "protocolVersion":1,"repositoryId":cockpit_repository::repository_id(dir.path()),
        "workItemId":WI,"intent":"test only","goal":"test only",
        "scope":["src/**"],"outOfScope":[],"risk":"normal","authority":"authorized",
        "acceptanceCriteria":["test only"],"requiredEvidenceClasses":["verification"],
        "sources":[],"verification":[],"baseRevision":"a".repeat(40),
        "projectProfileDigest":digest,"repositorySnapshotDigest":digest
    });
    if required {
        contract["governanceProfile"] = json!({"goalAuthority":{"required":true}});
    }
    fs::write(
        active.join(format!("{WI}.contract.json")),
        serde_json::to_vec(&contract).unwrap(),
    )
    .unwrap();
    fs::write(
        active.join(format!("{WI}.summary.json")),
        b"{\"state\":\"checkpointed\",\"checkpointCount\":1}",
    )
    .unwrap();
    dir
}

fn summary(root: &Path) -> Vec<u8> {
    fs::read(root.join(format!(".ai/work-items/active/{WI}.summary.json"))).unwrap()
}

#[test]
fn b01_required_no_runtime_controls_reject_before_any_summary_write() {
    let dir = fixture(true);
    let before = summary(dir.path());
    let result = cockpit_repository::record_work_item_governance_controls(
        dir.path(),
        WI,
        &json!({"intentAlignment":{"state":"unknown"}}),
    );
    assert!(
        result.is_err(),
        "required declaration must not authorize a controls write"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("goal_authority_source_unsupported")
    );
    assert_eq!(summary(dir.path()), before);
    assert!(!dir.path().join(".ai/locks").exists());
}

#[test]
fn b13_unconfigured_legacy_controls_keep_existing_behavior() {
    let dir = fixture(false);
    let result = cockpit_repository::record_work_item_governance_controls(
        dir.path(),
        WI,
        &json!({"intentAlignment":{"state":"unknown"}}),
    )
    .unwrap();
    assert_eq!(result["intentAlignment"]["state"], "unknown");
    assert_eq!(result["state"], "checkpointed");
}

fn runtime() -> cockpit_protocol::RuntimeContext {
    cockpit_protocol::RuntimeContext {
        runtime_version: "1.0.1".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"test runtime is not actor authentication"),
    }
}

#[test]
fn b01_b11_public_entry_inventory_rejects_before_other_validation_or_effects() {
    use cockpit_repository as repo;
    type Probe = fn(&Path) -> Result<(), repo::ObserverError>;
    let probes: &[(&str, Probe)] = &[
        ("controls-with-runtime", |p| {
            repo::record_work_item_governance_controls_with_runtime(p, WI, &json!({}), &runtime())
                .map(|_| ())
        }),
        ("controls-validation", |p| {
            repo::validate_work_item_governance_controls(p, WI).map(|_| ())
        }),
        ("action-admission", |p| {
            repo::require_current_action_admission(p, WI, "record_governance_controls", &runtime())
                .map(|_| ())
        }),
        ("checkpoint", |p| {
            repo::checkpoint_work_item(p, WI).map(|_| ())
        }),
        ("direct-revalidation", |p| {
            repo::revalidate_contract_amendment(p, WI, "fixture").map(|_| ())
        }),
        ("preflight", |p| {
            repo::preflight_work_item(
                p,
                &p.join(format!(".ai/work-items/active/{WI}.contract.json")),
            )
            .map(|_| ())
        }),
        ("finish", |p| repo::finish_work_item(p, WI).map(|_| ())),
        ("archive", |p| repo::archive_work_item(p, WI).map(|_| ())),
        ("close", |p| repo::close_work_item(p, WI).map(|_| ())),
        ("reconcile", |p| {
            repo::reconcile_active_artifacts(p, WI).map(|_| ())
        }),
        ("outcome", |p| repo::outcome_v2(p, WI).map(|_| ())),
        ("status", |p| repo::status(p).map(|_| ())),
        ("capability-business-query", |p| {
            repo::capability_truth_registry(p).map(|_| ())
        }),
        ("execution-context", |p| {
            repo::RepositoryExecutionContext::capture(p).map(|_| ())
        }),
        ("approach", |p| {
            repo::implementation_approach(p, WI).map(|_| ())
        }),
        ("approach-read-only", |p| {
            repo::implementation_approach_read_only(p, WI).map(|_| ())
        }),
        ("usage-read", |p| {
            repo::read_work_item_usage(p, WI, None).map(|_| ())
        }),
        ("knowledge-generate", |p| {
            repo::generate_knowledge(p).map(|_| ())
        }),
        ("scaffold", |p| {
            repo::scaffold_work_item(p, WI, "implementation").map(|_| ())
        }),
        ("amendment", |p| {
            repo::apply_work_item_contract_amendment(
                p,
                WI,
                &cockpit_protocol::ContractAmendmentRequest {
                    schema_version: 1,
                    change_id: "fixture-only".into(),
                    expected_contract_digest: Digest::sha256_bytes(b"not a permit"),
                    reason: "test only".into(),
                    changes: vec![],
                },
                &runtime(),
            )
            .map(|_| ())
        }),
    ];
    let mut missing = vec![];
    for (name, probe) in probes {
        let dir = fixture(true);
        let before = summary(dir.path());
        let error = probe(dir.path())
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();
        if !error.contains("goal_authority_source_unsupported") {
            missing.push(format!("{name}: missing authority rejection"));
        }
        if summary(dir.path()) != before || dir.path().join(".ai/locks").exists() {
            missing.push(format!("{name}: changed protected fixture"));
        }
    }
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}

#[test]
fn b01_material_plan_rejects_before_scanning_business_material() {
    let dir = fixture(true);
    let before = summary(dir.path());
    let result = cockpit_repository::plan_work_item_material_review(dir.path(), WI);
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("goal_authority_source_unsupported")
    );
    assert_eq!(summary(dir.path()), before);
}

fn init_git(root: &Path) {
    assert!(
        std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(root)
            .status()
            .unwrap()
            .success()
    );
}

fn collaboration_runtime() -> cockpit_protocol::RuntimeCapabilityBinding {
    cockpit_protocol::RuntimeCapabilityBinding {
        schema_version: 1,
        runtime_version: "1.0.1".into(),
        runtime_digest: Digest::sha256_bytes(b"fixture identity is not authority"),
        capability: cockpit_protocol::COLLABORATION_CAPABILITY.into(),
    }
}

#[test]
fn b01_direct_store_open_rejects_before_creating_shared_directories() {
    let dir = fixture(true);
    init_git(dir.path());
    let git = cockpit_git::GitRepository::discover(dir.path()).unwrap();
    let error =
        cockpit_repository::CoordinationStore::open(&git, collaboration_runtime()).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("goal_authority_source_unsupported"),
        "{error}"
    );
    assert!(!dir.path().join(".git/.ai-cockpit").exists());
}

#[test]
fn b01_retained_store_rechecks_before_business_reads_or_recovery() {
    let dir = fixture(false);
    init_git(dir.path());
    let git = cockpit_git::GitRepository::discover(dir.path()).unwrap();
    let store = cockpit_repository::CoordinationStore::open(&git, collaboration_runtime()).unwrap();
    let path = dir
        .path()
        .join(format!(".ai/work-items/active/{WI}.contract.json"));
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["governanceProfile"] = json!({"goalAuthority":{"required":true}});
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(
        store
            .inspect()
            .unwrap_err()
            .to_string()
            .contains("goal_authority_source_unsupported")
    );
    assert!(
        store
            .recover()
            .unwrap_err()
            .to_string()
            .contains("goal_authority_source_unsupported")
    );
    assert!(!store.root().join(".lock").exists());
}

#[test]
fn b01_retained_context_rechecks_even_after_observation_is_cached() {
    let dir = tempdir().unwrap();
    init_git(dir.path());
    cockpit_repository::attach(dir.path()).unwrap();
    let context = cockpit_repository::RepositoryExecutionContext::capture(dir.path()).unwrap();
    context.observe().unwrap();
    let active = dir.path().join(".ai/work-items/active");
    fs::create_dir_all(&active).unwrap();
    fs::write(
        active.join(format!("{WI}.contract.json")),
        br#"{"governanceProfile":{"goalAuthority":{"required":true}}}"#,
    )
    .unwrap();
    let error = context.observe().unwrap_err();
    assert!(
        error
            .to_string()
            .contains("goal_authority_source_unsupported"),
        "{error}"
    );
    assert!(
        context
            .observe_phase(
                cockpit_repository::ObservationPhase::BeforeGovernance,
                None,
                None
            )
            .unwrap_err()
            .to_string()
            .contains("goal_authority_source_unsupported")
    );
}

#[test]
fn b04_declarations_validate_shape_and_reference_precision_without_enabling_grants() {
    let cases = [
        json!({"required":false,"unexpected":true}),
        json!({"required":false,"reference":{"grantId":"a"}}),
        json!({"required":false,"reference":{"grantId":"a","subjectRef":"s","versionResolution":"current","deliverableRefs":["part"],"notBefore":"2030-01-01T00:00:00.0000000000001Z","deadline":{"mode":"inherit_source"}}}),
        json!({"required":false,"reference":{"grantId":"a","subjectRef":"s","versionResolution":"current","deliverableRefs":["part"],"deadline":{"mode":"cap_at","expiresAt":"2030-01-01T00:00:01.0000000000001Z"}}}),
    ];
    for declaration in cases {
        let dir = fixture(false);
        let path = dir
            .path()
            .join(format!(".ai/work-items/active/{WI}.contract.json"));
        let mut contract: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        contract["governanceProfile"] = json!({"goalAuthority":declaration});
        fs::write(&path, serde_json::to_vec(&contract).unwrap()).unwrap();
        assert!(
            cockpit_repository::require_goal_authority_access(dir.path(), Some(WI))
                .unwrap_err()
                .to_string()
                .contains("goal_authority_declaration_invalid")
        );
    }
}

#[test]
fn b04_duplicate_locator_keys_cannot_erase_a_required_request() {
    for bytes in [
        br#"{"governanceProfile":{"goalAuthority":{"required":true,"required":false}}}"#.as_slice(),
        br#"{"governanceProfile":{"goalAuthority":{"required":true}},"governanceProfile":{}}"#
            .as_slice(),
    ] {
        let dir = fixture(false);
        fs::write(
            dir.path()
                .join(format!(".ai/work-items/active/{WI}.contract.json")),
            bytes,
        )
        .unwrap();
        assert!(
            cockpit_repository::require_goal_authority_access(dir.path(), None)
                .unwrap_err()
                .to_string()
                .contains("duplicate")
        );
    }
}

#[test]
fn review_linked_checkout_must_not_create_required_common_root_coordination() {
    let dir = fixture(true);
    init_git(dir.path());
    assert!(
        std::process::Command::new("git")
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--allow-empty",
                "-qm",
                "fixture"
            ])
            .current_dir(dir.path())
            .status()
            .unwrap()
            .success()
    );
    let owner = tempdir().unwrap();
    let linked = owner.path().join("linked");
    assert!(
        std::process::Command::new("git")
            .args(["worktree", "add", "-q", "-b", "linked-fixture"])
            .arg(&linked)
            .current_dir(dir.path())
            .status()
            .unwrap()
            .success()
    );
    let git = cockpit_git::GitRepository::discover(&linked).unwrap();
    let error =
        cockpit_repository::CoordinationStore::open(&git, collaboration_runtime()).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("goal_authority_source_unsupported"),
        "{error}"
    );
    assert!(!dir.path().join(".git/.ai-cockpit").exists());
}

#[test]
fn review_direct_node_plans_cannot_turn_authority_denial_into_reuse_miss() {
    for policy in [
        cockpit_repository::RepositoryVerificationPolicy::NeverReuse,
        cockpit_repository::RepositoryVerificationPolicy::ProfileAuthorized,
    ] {
        let dir = fixture(true);
        init_git(dir.path());
        let snapshot = cockpit_git::GitRepository::discover(dir.path())
            .unwrap()
            .snapshot()
            .unwrap();
        let request = cockpit_repository::RepositoryVerificationRequest {
            node_id: "fixture-node".into(),
            program: "git".into(),
            args: vec!["--version".into()],
            scope: vec![],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: "fixture".into(),
            base_commit: None,
            workers: 1,
            work_item_id: None,
            timeout_seconds: None,
            policy,
        };
        let error = cockpit_repository::plan_repository_verification_action(
            dir.path(),
            &request,
            &snapshot,
            0,
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("goal_authority_source_unsupported"),
            "{error}"
        );
    }
}

#[test]
fn review_public_input_reader_checks_the_source_repository_independently() {
    let source = fixture(true);
    let path = source.path().join("business-input.json");
    fs::write(&path, br#"{"unique":"protected source bytes"}"#).unwrap();
    let error = cockpit_repository::read_legacy_closeout_source_input(&path).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("goal_authority_source_unsupported"),
        "{error}"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn review_boolean_recovery_query_does_not_open_the_decision_directory() {
    use std::{
        io::Read,
        os::unix::{
            ffi::OsStrExt,
            io::{AsRawFd, FromRawFd},
        },
    };
    let dir = fixture(true);
    let decisions = dir.path().join(".ai/decisions");
    fs::create_dir(&decisions).unwrap();
    let raw = unsafe { libc::inotify_init1(libc::IN_NONBLOCK | libc::IN_CLOEXEC) };
    assert!(raw >= 0);
    let mut watcher = unsafe { fs::File::from_raw_fd(raw) };
    let path = std::ffi::CString::new(decisions.as_os_str().as_bytes()).unwrap();
    assert!(
        unsafe { libc::inotify_add_watch(watcher.as_raw_fd(), path.as_ptr(), libc::IN_OPEN) } >= 0
    );
    assert!(
        !cockpit_repository::selected_successor_lineage_recovery_resolves_pending_close(
            dir.path(),
            WI,
            "fixture"
        )
    );
    let mut events = [0u8; 1024];
    let result = watcher.read(&mut events);
    assert!(
        matches!(result,Err(ref e) if e.kind()==std::io::ErrorKind::WouldBlock),
        "protected decision directory was opened: {result:?}"
    );
}

#[test]
fn review_subdirectory_or_alias_root_cannot_skip_the_authority_locator() {
    let dir = fixture(true);
    let child = dir.path().join("source-subdirectory");
    fs::create_dir(&child).unwrap();
    let error = cockpit_repository::require_goal_authority_access(&child, None).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("goal_authority_source_unsupported"),
        "{error}"
    );
    #[cfg(unix)]
    {
        let alias_owner = tempdir().unwrap();
        let alias = alias_owner.path().join("alias");
        std::os::unix::fs::symlink(&child, &alias).unwrap();
        let error = cockpit_repository::require_goal_authority_access(&alias, None).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("goal_authority_source_unsupported"),
            "{error}"
        );
    }
}

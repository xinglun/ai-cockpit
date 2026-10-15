use cockpit_repository::{WorkItemStartOptions, attach, start_work_item_with_options, status};
use std::{fs, path::Path, process::Command};

const ID: &str = "WI-LEGACY-SOURCE";

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    git(directory.path(), &["init", "--quiet"]);
    attach(directory.path()).unwrap();
    fs::write(directory.path().join("seed.txt"), "baseline\n").unwrap();
    git(directory.path(), &["add", "-A"]);
    git(
        directory.path(),
        &[
            "-c",
            "user.name=Legacy source test",
            "-c",
            "user.email=legacy@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "baseline",
        ],
    );
    directory
}

#[test]
fn malformed_requirement_remains_pending_when_archive_directory_is_missing() {
    // The production change under test is enumeration from canonical proof
    // names before parsing; removing that union must make this assertion fail.
    let directory = repository();
    let root = directory.path();
    let archive = root.join(".ai/work-items/archive");
    if archive.exists() {
        fs::remove_dir_all(&archive).unwrap();
    }
    fs::create_dir_all(root.join(".ai/decisions")).unwrap();
    fs::write(
        root.join(format!(
            ".ai/decisions/{ID}.legacy-closeout-source-requirement.json"
        )),
        b"{broken",
    )
    .unwrap();
    let observed = status(root).unwrap();
    assert!(
        observed
            .readiness
            .unclosed_archived_work_items
            .contains(&ID.to_string()),
        "surviving malformed canonical requirement must retain the WI: {:?}",
        observed.readiness
    );
}

#[test]
fn foreign_registration_cannot_release_untrusted_archived_scope() {
    let directory = repository();
    let root = directory.path();
    fs::create_dir_all(root.join(".ai/decisions")).unwrap();
    fs::write(
        root.join(format!(
            ".ai/decisions/{ID}.legacy-closeout-source-registration.json"
        )),
        br#"{"repositoryId":"foreign","workItemId":"other"}"#,
    )
    .unwrap();
    let result = start_work_item_with_options(
        root,
        "WI-OVERLAPPING",
        "bounded source work",
        "keep lost scope blocked",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["scope remains blocked".into()],
            ..Default::default()
        },
    );
    let error =
        result.expect_err("foreign content must not hide a canonical registration obligation");
    assert!(
        error
            .to_string()
            .contains("archived_work_item_scope_untrusted"),
        "{error}"
    );
}

mod extension {
    use cockpit_core::Digest;
    use cockpit_protocol::{
        HumanDecision, PROTOCOL_VERSION, ResourceFinalizationContext, RuntimeContext,
    };
    use cockpit_repository::{
        RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions, attach,
        checkpoint_work_item, close_work_item_with_structured_decision_and_runtime,
        finish_work_item_with_runtime, plan_resource_finalization_with_runtime,
        record_resource_finalization, record_verification_with_runtime, repository_id,
        run_repository_verification, start_work_item_with_options,
    };
    use serde_json::Value;
    use std::{fs, path::Path, process::Command};

    const WORK_ITEM_ID: &str = "WI-REPOSITORY-CROSS-CHECKOUT-CLOSEOUT";

    fn runtime() -> RuntimeContext {
        RuntimeContext {
            runtime_version: "1.0.0-repository-closeout-test".into(),
            protocol_version: PROTOCOL_VERSION,
            runtime_digest: Digest::sha256_bytes(b"repository-closeout-recovery-test-runtime"),
        }
    }

    fn command(root: &Path, program: &str, args: &[&str]) -> std::process::Output {
        let output = Command::new(program)
            .args(args)
            .current_dir(root)
            .output()
            .expect("run command");
        assert!(
            output.status.success(),
            "{program} {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }

    fn git(root: &Path, args: &[&str]) {
        command(root, "git", args);
    }

    fn repository() -> tempfile::TempDir {
        let directory = tempfile::tempdir().expect("source repository");
        git(directory.path(), &["init", "--quiet"]);
        attach(directory.path()).expect("attach source repository");
        git(
            directory.path(),
            &["config", "user.name", "Repository Closeout Test"],
        );
        git(
            directory.path(),
            &[
                "config",
                "user.email",
                "repository-closeout-test@example.invalid",
            ],
        );
        fs::write(directory.path().join("seed.txt"), "baseline\n").expect("write baseline");
        git(directory.path(), &["add", "-A"]);
        git(
            directory.path(),
            &["commit", "--quiet", "-m", "fixture baseline"],
        );
        directory
    }

    fn build_provider_bound_closed_source(
        source: &Path,
        context: &ResourceFinalizationContext,
    ) -> (Digest, Digest) {
        let runtime = runtime();
        start_work_item_with_options(
            source,
            WORK_ITEM_ID,
            "preserve validated closeout across checkouts",
            "validate the historical provider receipt against the archived Contract bytes",
            &[".ai/**".into()],
            &WorkItemStartOptions {
                authority: "authorized".into(),
                acceptance_criteria: vec![
                    "raw finalization binding stays distinct from canonical plan binding".into(),
                ],
                ..Default::default()
            },
        )
        .expect("start provider-bound source Work Item");
        plan_resource_finalization_with_runtime(source, WORK_ITEM_ID, context, &runtime)
            .expect("bind provider resource context before verification");
        let contract = source.join(format!(
            ".ai/work-items/active/{WORK_ITEM_ID}.contract.json"
        ));
        cockpit_repository::preflight_work_item(source, &contract).expect("preflight");
        checkpoint_work_item(source, WORK_ITEM_ID).expect("checkpoint");
        let request = RepositoryVerificationRequest {
            node_id: "provider-bound-closeout-source-verification".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec![".ai/**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: Some(WORK_ITEM_ID.into()),
            timeout_seconds: None,
            policy: RepositoryVerificationPolicy::NeverReuse,
        };
        let execution = run_repository_verification(source, &request).expect("verify source");
        record_verification_with_runtime(
            source,
            WORK_ITEM_ID,
            &serde_json::to_value(execution.receipt).expect("verification receipt JSON"),
            &runtime,
            &execution.final_snapshot,
        )
        .expect("record verification");
        finish_work_item_with_runtime(source, WORK_ITEM_ID, &runtime).expect("finish source");
        cockpit_repository::archive_work_item_with_runtime(source, WORK_ITEM_ID, &runtime)
            .expect("archive source");

        let contract_path = source.join(format!(
            ".ai/work-items/archive/{WORK_ITEM_ID}.contract.json"
        ));
        let contract_bytes = fs::read(&contract_path).expect("archived Contract bytes");
        let contract: Value =
            serde_json::from_slice(&contract_bytes).expect("archived Contract JSON");
        let raw_contract_digest = Digest::sha256_bytes(&contract_bytes);
        let canonical_contract_digest =
            cockpit_protocol::digest_json(&contract).expect("canonical Contract digest");
        assert_ne!(
            raw_contract_digest, canonical_contract_digest,
            "fixture must distinguish raw-byte and canonical-JSON Contract digests"
        );
        let head = String::from_utf8(command(source, "git", &["rev-parse", "HEAD"]).stdout)
            .expect("git head")
            .trim()
            .to_owned();
        let receipt = serde_json::json!({
            "schemaVersion": 1,
            "receiptId": "provider-bound-closeout-receipt",
            "operationId": "provider-bound-closeout-operation",
            "repositoryId": repository_id(source).to_string(),
            "workItemId": WORK_ITEM_ID,
            "runtimeVersion": runtime.runtime_version,
            "runtimeDigest": runtime.runtime_digest.to_string(),
            "provider": context.provider,
            "pullRequest": {
                "number": 123,
                "url": context.pull_request,
                "headRevision": head,
                "baseBranch": context.base_branch,
                "baseRemote": context.base_remote,
                "baseRevision": contract["baseRevision"],
                "mergeCommit": head
            },
            "branch": {
                "name": context.branch,
                "remote": context.base_remote,
                "headRevision": head
            },
            "worktree": {
                "worktreeId": "provider-bound-closeout-worktree",
                "path": context.worktree,
                "branch": context.branch,
                "headRevision": head
            },
            "before": {"pullRequest": "merged", "branch": "present", "worktree": "clean"},
            "after": {"pullRequest": "merged", "branch": "deleted", "worktree": "removed"},
            "result": {"disposition": "deleted", "failureCodes": [], "unknownCodes": []},
            "actor": "human:test",
            "authoritySource": "repository closeout integration test",
            "reason": "record a merged provider resource without deleting retained resources",
            "timestamp": "2026-10-02T00:00:00Z",
            "contractDigest": raw_contract_digest,
            "contractBaseRevision": contract["baseRevision"],
            "resourceContext": context
        });
        let finalization_input = tempfile::NamedTempFile::new().expect("finalization input");
        fs::write(
            finalization_input.path(),
            serde_json::to_vec_pretty(&receipt).expect("finalization receipt JSON"),
        )
        .expect("write finalization receipt");
        record_resource_finalization(source, WORK_ITEM_ID, finalization_input.path(), &runtime)
            .expect("record raw-byte-bound provider finalization");
        close_work_item_with_structured_decision_and_runtime(
            source,
            WORK_ITEM_ID,
            &HumanDecision {
                decision: "approved".into(),
                actor: "test-human".into(),
                authority_source: "repository closeout integration test".into(),
                reason: "the source verification and provider merge are recorded".into(),
                evidence_refs: vec![format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")],
                policy_refs: Vec::new(),
                decided_at: "2026-10-02T00:01:00Z".into(),
                resume_condition: None,
            },
            &runtime,
        )
        .expect("close provider-bound source Work Item");
        git(source, &["add", "-A"]);
        git(
            source,
            &["commit", "--quiet", "-m", "close provider-bound Work Item"],
        );
        (raw_contract_digest, canonical_contract_digest)
    }

    fn registered_retained_fixture() -> RegisteredRetainedFixture {
        let source = repository();
        let context = ResourceFinalizationContext {
            branch: "feature/recovered-retained-close".into(),
            worktree: source
                .path()
                .join("removed-retained-worktree")
                .display()
                .to_string(),
            base_branch: "develop".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: "https://github.com/example/sentinel/pull/123".into(),
        };
        build_provider_bound_closed_source(source.path(), &context);
        let finalization = format!(".ai/decisions/{WORK_ITEM_ID}.finalize.json");
        let close = format!(".ai/decisions/{WORK_ITEM_ID}.close.json");
        let mut retained: Value =
            serde_json::from_slice(&fs::read(source.path().join(&finalization)).unwrap()).unwrap();
        // Model the supported older Retained-close history in this disposable
        // fixture. Modern close correctly requires deletion; no real history is edited.
        retained["after"]["branch"] = "present".into();
        retained["after"]["worktree"] = "clean".into();
        retained["result"]["disposition"] = "retained".into();
        let retained_digest = cockpit_protocol::digest_json(&retained).unwrap();
        fs::write(
            source.path().join(&finalization),
            serde_json::to_vec_pretty(&retained).unwrap(),
        )
        .unwrap();
        let mut legacy_close: Value =
            serde_json::from_slice(&fs::read(source.path().join(&close)).unwrap()).unwrap();
        legacy_close["resourceFinalizationHeadDigest"] = retained_digest.to_string().into();
        fs::write(
            source.path().join(&close),
            serde_json::to_vec_pretty(&legacy_close).unwrap(),
        )
        .unwrap();
        git(source.path(), &["add", "-A"]);
        git(
            source.path(),
            &["commit", "--quiet", "-m", "legacy Retained close fixture"],
        );
        let parent = tempfile::tempdir().unwrap();
        let destination = parent.path().join("clone");
        git(
            source.path(),
            &[
                "clone",
                "--quiet",
                "--no-hardlinks",
                source.path().to_str().unwrap(),
                destination.to_str().unwrap(),
            ],
        );
        cockpit_repository::recover_cross_checkout_closeout(
            &destination,
            source.path(),
            WORK_ITEM_ID,
            &runtime(),
        )
        .expect("recover valid older Retained close");
        let baseline = cockpit_repository::work_item_status_snapshot_with_runtime(
            &destination,
            WORK_ITEM_ID,
            &runtime(),
        )
        .unwrap();
        assert_eq!(baseline.lifecycle_phase, "closed");
        assert!(!baseline.blocking);
        let origin = format!(".ai/decisions/{WORK_ITEM_ID}.cross-checkout-closeout-origin.json");
        let receipt = format!(".ai/decisions/{WORK_ITEM_ID}.cross-checkout-closeout-recovery.json");
        let _frozen = [&close, &finalization, &origin, &receipt]
            .into_iter()
            .map(|p| (p.clone(), fs::read(destination.join(p)).unwrap()))
            .collect::<Vec<_>>();
        let previous: cockpit_protocol::ResourceFinalizationReceipt =
            serde_json::from_value(retained).unwrap();
        let mut deleted = previous.clone();
        deleted.receipt_id = "post-close-deleted".into();
        deleted.operation_id = "post-close-cleanup".into();
        deleted.before = previous.after.clone();
        deleted.after.branch = cockpit_protocol::ResourceFinalizationBranchState::Deleted;
        deleted.after.worktree = cockpit_protocol::ResourceFinalizationWorktreeState::Removed;
        deleted.result.disposition = cockpit_protocol::ResourceFinalizationDisposition::Deleted;
        let transition = cockpit_protocol::ResourceFinalizationTransitionReceipt {
            schema_version: 1,
            transition_id: "recovered-post-close-deleted".into(),
            sequence: 1,
            predecessor_receipt_digest: retained_digest,
            governance_append_revision: None,
            receipt: deleted,
        };

        let recovery_path = destination.join(&receipt);
        let mut legacy: Value = serde_json::from_slice(&fs::read(&recovery_path).unwrap()).unwrap();
        legacy["schemaVersion"] = 1.into();
        legacy.as_object_mut().unwrap().remove("originBinding");
        fs::write(&recovery_path, serde_json::to_vec_pretty(&legacy).unwrap()).unwrap();
        fs::remove_file(destination.join(&origin)).unwrap();
        git(
            &destination,
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "add",
                "-A",
            ],
        );
        git(
            &destination,
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--quiet",
                "-m",
                "complete old-v1 fixture",
            ],
        );
        let pinned_owner = tempfile::tempdir().unwrap();
        let pinned = pinned_owner.path().join("source");
        git(
            &destination,
            &[
                "clone",
                "--quiet",
                "--no-hardlinks",
                destination.to_str().unwrap(),
                pinned.to_str().unwrap(),
            ],
        );
        let revision = String::from_utf8(command(&pinned, "git", &["rev-parse", "HEAD"]).stdout)
            .unwrap()
            .trim()
            .to_owned();
        let registrar = RuntimeContext {
            runtime_version: "current-registrar-fixture".into(),
            protocol_version: PROTOCOL_VERSION,
            runtime_digest: Digest::sha256_bytes(b"different current registrar"),
        };
        let request = serde_json::from_value(serde_json::json!({"schemaVersion":1,"operation":"register","sourceRevision":revision,"actor":"test-data:registrar","authoritySource":"isolated fixture","reason":"preserve complete old-v1 history","sourceAccepted":true})).unwrap();
        let root_plan = cockpit_repository::plan_legacy_closeout_source(
            &destination,
            Some(&pinned),
            WORK_ITEM_ID,
            &request,
            &registrar,
        )
        .unwrap();
        cockpit_repository::record_legacy_closeout_source(
            &destination,
            WORK_ITEM_ID,
            &root_plan,
            &registrar,
        )
        .unwrap();
        RegisteredRetainedFixture {
            _owners: (source, parent, pinned_owner),
            destination,
            transition,
            registrar,
        }
    }

    struct RegisteredRetainedFixture {
        _owners: (tempfile::TempDir, tempfile::TempDir, tempfile::TempDir),
        destination: std::path::PathBuf,
        transition: cockpit_protocol::ResourceFinalizationTransitionReceipt,
        registrar: RuntimeContext,
    }

    #[test]
    fn registered_root_refuses_unprepared_raw_transition_without_writing() {
        let fixture = registered_retained_fixture();
        let raw = serde_json::to_vec_pretty(&fixture.transition).unwrap();
        let input = tempfile::NamedTempFile::new().unwrap();
        fs::write(input.path(), &raw).unwrap();
        let error = record_resource_finalization(&fixture.destination, WORK_ITEM_ID, input.path(), &fixture.registrar)
        .expect_err("registered old-v1 root requires independently retained extension plan before publication");
        assert!(
            error.to_string().contains("extension_plan_required"),
            "{error}"
        );
        let canonical = cockpit_protocol::digest_json(&fixture.transition)
            .unwrap()
            .to_string();
        assert!(
            !fixture
                .destination
                .join(format!(
                    ".ai/decisions/{WORK_ITEM_ID}.finalize.{}.json",
                    &canonical[7..]
                ))
                .exists()
        );
    }

    fn extension_plan(
        fixture: &RegisteredRetainedFixture,
    ) -> cockpit_protocol::LegacyCloseoutSourcePlan {
        let request = serde_json::from_value(serde_json::json!({"schemaVersion":1,"operation":"extend","transitionBytes":String::from_utf8(serde_json::to_vec_pretty(&fixture.transition).unwrap()).unwrap(),"actor":"test-data:extension-registrar","authoritySource":"isolated fixture","reason":"bind exactly the supported Retained to Deleted transition"})).expect("typed extension request supported");
        cockpit_repository::plan_legacy_closeout_source(
            &fixture.destination,
            None,
            WORK_ITEM_ID,
            &request,
            &fixture.registrar,
        )
        .expect("read-only exact extension plan")
    }

    #[test]
    fn prepared_extension_uses_one_writer_and_replays_original_before_next_sequence() {
        let fixture = registered_retained_fixture();
        let plan = extension_plan(&fixture);
        let path = fixture.destination.join(format!(
            ".ai/decisions/{WORK_ITEM_ID}.legacy-closeout-source-extension.json"
        ));
        assert!(!path.exists(), "plan must be read-only");
        let input = tempfile::NamedTempFile::new().unwrap();
        fs::write(input.path(), serde_json::to_vec_pretty(&plan).unwrap()).unwrap();
        let result = record_resource_finalization(
            &fixture.destination,
            WORK_ITEM_ID,
            input.path(),
            &fixture.registrar,
        )
        .expect("normal finalize uses shared prepared extension writer");
        let original =
            fs::read(fixture.destination.join(result["path"].as_str().unwrap())).unwrap();
        assert_eq!(
            original,
            serde_json::to_vec_pretty(&fixture.transition).unwrap()
        );
        let after = cockpit_repository::work_item_status_snapshot_with_runtime(
            &fixture.destination,
            WORK_ITEM_ID,
            &fixture.registrar,
        )
        .unwrap();
        assert_eq!(after.lifecycle_phase, "closed", "{after:?}");
        assert_ne!(after.verification, "verified");
        let replay = cockpit_repository::record_legacy_closeout_source(
            &fixture.destination,
            WORK_ITEM_ID,
            &plan,
            &fixture.registrar,
        )
        .expect("explicit record replays exact original");
        assert_eq!(replay, result);
        fs::write(input.path(), &original).unwrap();
        let raw_replay = record_resource_finalization(
            &fixture.destination,
            WORK_ITEM_ID,
            input.path(),
            &fixture.registrar,
        )
        .expect("exact committed raw retry is recognized before sequence + 1");
        assert_eq!(raw_replay, result);
        assert_eq!(
            fs::read(fixture.destination.join(result["path"].as_str().unwrap())).unwrap(),
            original
        );
    }

    #[test]
    fn committed_raw_retry_rejects_canonical_equal_but_changed_raw_bytes() {
        let fixture = registered_retained_fixture();
        let plan = extension_plan(&fixture);
        cockpit_repository::record_legacy_closeout_source(
            &fixture.destination,
            WORK_ITEM_ID,
            &plan,
            &fixture.registrar,
        )
        .unwrap();
        let input = tempfile::NamedTempFile::new().unwrap();
        fs::write(
            input.path(),
            serde_json::to_vec(&fixture.transition).unwrap(),
        )
        .unwrap();
        let error = record_resource_finalization(
            &fixture.destination,
            WORK_ITEM_ID,
            input.path(),
            &fixture.registrar,
        )
        .expect_err("canonical equality cannot replace exact original raw retry binding");
        assert!(
            error.to_string().contains("published_transition_changed"),
            "{error}"
        );
    }

    #[test]
    fn partial_extension_requires_independent_original_and_resumes_both_survivors() {
        for transition_only in [false, true] {
            let fixture = registered_retained_fixture();
            let original = extension_plan(&fixture);
            let extension: cockpit_protocol::LegacyCloseoutSourceExtension =
                serde_json::from_str(original.extension_bytes.as_ref().unwrap()).unwrap();
            let extension_path = fixture.destination.join(format!(
                ".ai/decisions/{WORK_ITEM_ID}.legacy-closeout-source-extension.json"
            ));
            if transition_only {
                fs::write(
                    fixture.destination.join(&extension.transition.path),
                    extension.transition_bytes.as_bytes(),
                )
                .unwrap();
            } else {
                fs::write(
                    &extension_path,
                    original.extension_bytes.as_ref().unwrap().as_bytes(),
                )
                .unwrap();
            }
            let pending = cockpit_repository::work_item_status_snapshot_with_runtime(
                &fixture.destination,
                WORK_ITEM_ID,
                &fixture.registrar,
            )
            .unwrap();
            assert_ne!(pending.lifecycle_phase, "closed");
            assert!(pending.blocking);
            let missing = serde_json::from_value(
                serde_json::json!({"schemaVersion":1,"operation":"resume_extension"}),
            )
            .unwrap();
            assert!(
                cockpit_repository::plan_legacy_closeout_source(
                    &fixture.destination,
                    None,
                    WORK_ITEM_ID,
                    &missing,
                    &fixture.registrar
                )
                .is_err()
            );
            let fresh = serde_json::from_value(serde_json::json!({"schemaVersion":1,"operation":"extend","transitionBytes":extension.transition_bytes,"actor":"new","authoritySource":"new","reason":"must not launder survivor"})).unwrap();
            assert!(
                cockpit_repository::plan_legacy_closeout_source(
                    &fixture.destination,
                    None,
                    WORK_ITEM_ID,
                    &fresh,
                    &fixture.registrar
                )
                .is_err()
            );
            let request = serde_json::from_value(serde_json::json!({"schemaVersion":1,"operation":"resume_extension","originalExtensionBytes":original.extension_bytes,"originalExtensionRawDigest":original.extension_raw_digest})).unwrap();
            let resumed = cockpit_repository::plan_legacy_closeout_source(&fixture.destination, None, WORK_ITEM_ID, &request, &fixture.registrar).expect("precise independently retained partial binding remains recoverable without closed admission");
            let result = cockpit_repository::record_legacy_closeout_source(
                &fixture.destination,
                WORK_ITEM_ID,
                &resumed,
                &fixture.registrar,
            )
            .unwrap();
            assert_eq!(result, extension.result);
            assert_eq!(
                fs::read(&extension_path).unwrap(),
                original.extension_bytes.as_ref().unwrap().as_bytes()
            );
            assert_eq!(
                fs::read(fixture.destination.join(&extension.transition.path)).unwrap(),
                extension.transition_bytes.as_bytes()
            );
            let status = cockpit_repository::work_item_status_snapshot_with_runtime(
                &fixture.destination,
                WORK_ITEM_ID,
                &fixture.registrar,
            )
            .unwrap();
            assert_eq!(status.lifecycle_phase, "closed");
        }
    }
}

mod cross_wi {
    use cockpit_core::Digest;
    use cockpit_protocol::{HumanDecision, ResourceFinalizationContext, RuntimeContext};
    use cockpit_repository::{
        ArchivedContractRevalidationRequest, RepositoryVerificationPolicy,
        RepositoryVerificationRequest, WorkItemStartOptions, archive_work_item, attach,
        checkpoint_work_item, close_work_item_with_structured_decision_and_runtime,
        finish_work_item, plan_resource_finalization, preflight_work_item,
        record_resource_finalization, record_verification_with_runtime,
        revalidate_archived_work_item_with_runtime, run_repository_verification,
        start_work_item_with_options, status,
    };
    use serde_json::json;
    use std::fs;
    use std::process::Command;

    fn commit_empty_baseline(path: &std::path::Path) {
        assert!(
            Command::new("git")
                .args([
                    "-c",
                    "user.name=AI Cockpit test fixture",
                    "-c",
                    "user.email=ai-cockpit-test@example.invalid",
                    "commit",
                    "--allow-empty",
                    "-qm",
                    "test fixture baseline",
                ])
                .current_dir(path)
                .status()
                .unwrap()
                .success()
        );
    }

    const ID: &str = "WI-ARCHIVED-AMENDMENT";
    const SUCCESSOR: &str = "WI-ARCHIVED-AMENDMENT-REVALIDATION";

    fn runtime() -> RuntimeContext {
        RuntimeContext {
            runtime_version: "0.2.75".into(),
            protocol_version: 1,
            runtime_digest: Digest::sha256_bytes(b"wi583-runtime"),
        }
    }

    fn upgraded_runtime() -> RuntimeContext {
        RuntimeContext {
            runtime_version: "0.2.76".into(),
            protocol_version: 1,
            runtime_digest: Digest::sha256_bytes(b"wi589-runtime"),
        }
    }

    fn commit(path: &std::path::Path, message: &str) {
        assert!(
            Command::new("git")
                .args(["add", "."])
                .current_dir(path)
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Command::new("git")
                .args([
                    "-c",
                    "user.email=test@example.invalid",
                    "-c",
                    "user.name=test",
                    "commit",
                    "-qm",
                    message
                ])
                .current_dir(path)
                .status()
                .unwrap()
                .success()
        );
    }

    fn archived_amended_repository() -> tempfile::TempDir {
        let directory = tempfile::tempdir().unwrap();
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .current_dir(directory.path())
                .status()
                .unwrap()
                .success()
        );
        commit_empty_baseline(directory.path());
        attach(directory.path()).unwrap();
        start_work_item_with_options(
            directory.path(),
            ID,
            "preserve an amended archived Work Item",
            "revalidate an archived Contract without rewriting evidence",
            &["src/**".into()],
            &WorkItemStartOptions {
                authority: "authorized".into(),
                acceptance_criteria: vec!["historical bytes remain auditable".into()],
                ..WorkItemStartOptions::default()
            },
        )
        .unwrap();
        let contract_path = directory
            .path()
            .join(format!(".ai/work-items/active/{ID}.contract.json"));
        preflight_work_item(directory.path(), &contract_path).unwrap();
        checkpoint_work_item(directory.path(), ID).unwrap();
        let run = run_repository_verification(
            directory.path(),
            &RepositoryVerificationRequest {
                node_id: "archived-amendment-check".into(),
                program: "true".into(),
                args: Vec::new(),
                scope: vec!["src/**".into()],
                stage: "task".into(),
                runner: "local".into(),
                runtime_digest: runtime().runtime_digest.to_string(),
                base_commit: None,
                workers: 1,
                work_item_id: None,
                timeout_seconds: None,
                policy: RepositoryVerificationPolicy::NeverReuse,
            },
        )
        .unwrap();
        let receipt = serde_json::to_value(&run.receipt).unwrap();
        record_verification_with_runtime(
            directory.path(),
            ID,
            &receipt,
            &runtime(),
            &run.final_snapshot,
        )
        .unwrap();
        finish_work_item(directory.path(), ID).unwrap();
        archive_work_item(directory.path(), ID).unwrap();

        // This models a reviewed repository repair: the current archived Contract
        // and its manifest are updated together, while the original verification
        // evidence bytes remain untouched and still carry the old Contract digest.
        let archive = directory.path().join(".ai/work-items/archive");
        let contract_path = archive.join(format!("{ID}.contract.json"));
        let mut contract: serde_json::Value =
            serde_json::from_slice(&fs::read(&contract_path).unwrap()).unwrap();
        contract["goal"] = json!("revalidate the reviewed archived Contract amendment");
        let contract_bytes = serde_json::to_vec_pretty(&contract).unwrap();
        fs::write(&contract_path, &contract_bytes).unwrap();
        let manifest_path = archive.join(format!("{ID}.archive.json"));
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        manifest["files"]["contractDigest"] =
            json!(Digest::sha256_bytes(&contract_bytes).to_string());
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        commit(directory.path(), "model reviewed archived Contract repair");
        directory
    }

    fn retained_successor_finalization(
        root: &std::path::Path,
        work_item_id: &str,
        runtime: &RuntimeContext,
    ) {
        let contract_path = root.join(format!(
            ".ai/work-items/archive/{work_item_id}.contract.json"
        ));
        let contract: serde_json::Value =
            serde_json::from_slice(&fs::read(&contract_path).unwrap()).unwrap();
        let context = contract["resourceContext"].clone();
        let repository_id = status(root).unwrap().repository_id;
        let receipt = json!({
            "schemaVersion": 1,
            "receiptId": format!("receipt-{work_item_id}"),
            "operationId": format!("operation-{work_item_id}"),
            "repositoryId": repository_id,
            "workItemId": work_item_id,
            "runtimeVersion": runtime.runtime_version,
            "runtimeDigest": runtime.runtime_digest,
            "provider": context["provider"],
            "pullRequest": {
                "number": 1,
                "url": context["pullRequest"],
                "headRevision": "head-successor",
                "baseBranch": context["baseBranch"],
                "baseRemote": context["baseRemote"],
                "baseRevision": contract["baseRevision"],
                "mergeCommit": "merge-successor"
            },
            "branch": {"name": context["branch"], "remote": "origin", "headRevision": "head-successor"},
            "worktree": {"worktreeId": format!("wt-{work_item_id}"), "path": context["worktree"], "branch": context["branch"], "headRevision": "head-successor"},
            "before": {"pullRequest": "merged", "branch": "present", "worktree": "clean"},
            "after": {"pullRequest": "merged", "branch": "present", "worktree": "clean"},
            "result": {"disposition": "retained", "failureCodes": [], "unknownCodes": []},
            "actor": "human:test",
            "authoritySource": "test-policy",
            "reason": "successor shared-worktree resources are retained",
            "timestamp": "2026-09-05T00:00:00Z",
            "contractDigest": Digest::sha256_bytes(&fs::read(&contract_path).unwrap()),
            "resourceContext": context
        });
        let input = tempfile::NamedTempFile::new().unwrap();
        fs::write(input.path(), serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
        record_resource_finalization(root, work_item_id, input.path(), runtime).unwrap();
    }

    fn terminal_chain() -> tempfile::TempDir {
        let directory = archived_amended_repository();
        // The predecessor finalization is intentionally produced by an older
        // Runtime. The successor and both close operations use the upgraded
        // Runtime, matching the cross-version adopter failure that motivated
        // this regression.
        let runtime = upgraded_runtime();
        revalidate_archived_work_item_with_runtime(
            directory.path(),
            ID,
            &ArchivedContractRevalidationRequest {
                successor_work_item_id: SUCCESSOR.into(),
                reason: "reviewed Contract repair requires fresh successor verification".into(),
                actor: "human:repository-owner".into(),
                authority_source: "explicit-user-authorized-revalidation".into(),
                resume_condition: "complete successor lifecycle before predecessor close".into(),
                evidence_refs: vec![format!(".ai/evidence/{ID}.verification.json")],
                policy_refs: vec!["docs/reference/agent-workflow.md".into()],
            },
            &runtime,
        )
        .unwrap();
        start_work_item_with_options(
            directory.path(),
            SUCCESSOR,
            "revalidate the amended archived Contract",
            "produce current evidence without rewriting historical evidence",
            &["src/**".into()],
            &WorkItemStartOptions {
                authority: "authorized".into(),
                acceptance_criteria: vec!["current Contract is freshly verified".into()],
                ..WorkItemStartOptions::default()
            },
        )
        .unwrap();
        let context = ResourceFinalizationContext {
            branch: "feature/archived-amendment-successor".into(),
            worktree: directory.path().display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "local".into(),
            pull_request: "local://archived-amendment-successor".into(),
        };
        plan_resource_finalization(directory.path(), SUCCESSOR, &context).unwrap();
        let active_contract = directory
            .path()
            .join(format!(".ai/work-items/active/{SUCCESSOR}.contract.json"));
        preflight_work_item(directory.path(), &active_contract).unwrap();
        checkpoint_work_item(directory.path(), SUCCESSOR).unwrap();
        let run = run_repository_verification(
            directory.path(),
            &RepositoryVerificationRequest {
                node_id: "successor-check".into(),
                program: "true".into(),
                args: Vec::new(),
                scope: vec!["src/**".into()],
                stage: "task".into(),
                runner: "local".into(),
                runtime_digest: runtime.runtime_digest.to_string(),
                base_commit: None,
                workers: 1,
                work_item_id: None,
                timeout_seconds: None,
                policy: RepositoryVerificationPolicy::NeverReuse,
            },
        )
        .unwrap();
        record_verification_with_runtime(
            directory.path(),
            SUCCESSOR,
            &serde_json::to_value(&run.receipt).unwrap(),
            &runtime,
            &run.final_snapshot,
        )
        .unwrap();
        finish_work_item(directory.path(), SUCCESSOR).unwrap();
        archive_work_item(directory.path(), SUCCESSOR).unwrap();
        retained_successor_finalization(directory.path(), SUCCESSOR, &runtime);
        close_work_item_with_structured_decision_and_runtime(
            directory.path(),
            SUCCESSOR,
            &HumanDecision {
                decision: "approved".into(),
                actor: "human:repository-owner".into(),
                authority_source: "explicit-user-authorized-revalidation".into(),
                reason: "successor evidence is current and complete".into(),
                evidence_refs: vec![format!(".ai/evidence/{SUCCESSOR}.verification.json")],
                policy_refs: vec!["docs/reference/agent-workflow.md".into()],
                decided_at: "2026-09-05T00:00:00Z".into(),
                resume_condition: None,
            },
            &runtime,
        )
        .unwrap();
        close_work_item_with_structured_decision_and_runtime(
            directory.path(),
            ID,
            &HumanDecision {
                decision: "approved".into(),
                actor: "human:repository-owner".into(),
                authority_source: "explicit-user-authorized-revalidation".into(),
                reason: "successor revalidated the amended Contract".into(),
                evidence_refs: vec![format!(".ai/evidence/{SUCCESSOR}.verification.json")],
                policy_refs: vec!["docs/reference/agent-workflow.md".into()],
                decided_at: "2026-09-05T00:00:00Z".into(),
                resume_condition: None,
            },
            &runtime,
        )
        .unwrap();
        commit(directory.path(), "complete terminal chain source");
        directory
    }

    fn cloned(source: &std::path::Path, dest: &std::path::Path) {
        let out = Command::new("git")
            .args([
                "clone",
                "--quiet",
                "--no-hardlinks",
                source.to_str().unwrap(),
                dest.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    struct ChainPair {
        _owners: (tempfile::TempDir, tempfile::TempDir, tempfile::TempDir),
        destination: std::path::PathBuf,
        pinned: std::path::PathBuf,
        registrar: RuntimeContext,
    }
    fn chain_pair() -> ChainPair {
        let source = terminal_chain();
        let dest_owner = tempfile::tempdir().unwrap();
        let destination = dest_owner.path().join("destination");
        cloned(source.path(), &destination);
        cockpit_repository::recover_cross_checkout_closeout(
            &destination,
            source.path(),
            ID,
            &upgraded_runtime(),
        )
        .unwrap();
        let path = destination.join(format!(
            ".ai/decisions/{ID}.cross-checkout-closeout-recovery.json"
        ));
        let mut old: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        old["schemaVersion"] = json!(1);
        old.as_object_mut().unwrap().remove("originBinding");
        fs::write(&path, serde_json::to_vec_pretty(&old).unwrap()).unwrap();
        fs::remove_file(destination.join(format!(
            ".ai/decisions/{ID}.cross-checkout-closeout-origin.json"
        )))
        .unwrap();
        commit(
            &destination,
            "complete old-v1 source with transitive terminal dependency",
        );
        let pinned_owner = tempfile::tempdir().unwrap();
        let pinned = pinned_owner.path().join("pinned");
        cloned(&destination, &pinned);
        let registrar = RuntimeContext {
            runtime_version: "present-day-registrar".into(),
            protocol_version: 1,
            runtime_digest: Digest::sha256_bytes(b"present-day-registrar"),
        };
        ChainPair {
            _owners: (source, dest_owner, pinned_owner),
            destination,
            pinned,
            registrar,
        }
    }
    fn registration_plan(pair: &ChainPair) -> cockpit_protocol::LegacyCloseoutSourcePlan {
        let out = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&pair.pinned)
            .output()
            .unwrap();
        assert!(out.status.success());
        let revision = String::from_utf8(out.stdout).unwrap().trim().to_owned();
        let request=serde_json::from_value(json!({"schemaVersion":1,"operation":"register","sourceRevision":revision,"actor":"test-data:registrar","authoritySource":"isolated closure fixture","reason":"bind all applicable dependency bytes","sourceAccepted":true})).unwrap();
        cockpit_repository::plan_legacy_closeout_source(
            &pair.destination,
            Some(&pair.pinned),
            ID,
            &request,
            &pair.registrar,
        )
        .unwrap()
    }
    #[test]
    fn semantically_valid_successor_raw_change_revokes_registered_predecessor() {
        let pair = chain_pair();
        let plan = registration_plan(&pair);
        cockpit_repository::record_legacy_closeout_source(
            &pair.destination,
            ID,
            &plan,
            &pair.registrar,
        )
        .unwrap();
        let path = pair
            .destination
            .join(format!(".ai/decisions/{SUCCESSOR}.close.json"));
        let mut close: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert!(close["structuredDecision"].is_object());
        close["structuredDecision"]["reason"] =
            json!("another nonempty, semantically valid historical reason");
        fs::write(&path, serde_json::to_vec_pretty(&close).unwrap()).unwrap();
        let successor = cockpit_repository::work_item_status_snapshot_with_runtime(
            &pair.destination,
            SUCCESSOR,
            &pair.registrar,
        )
        .unwrap();
        assert_eq!(
            successor.lifecycle_phase, "closed",
            "successor remains semantically closed: {successor:?}"
        );
        let predecessor = cockpit_repository::work_item_status_snapshot_with_runtime(
            &pair.destination,
            ID,
            &pair.registrar,
        )
        .unwrap();
        assert!(
            predecessor.blocking && predecessor.lifecycle_phase != "closed",
            "root registration must bind raw successor payload, not merely accept valid successor semantics: {predecessor:?}"
        );
    }

    #[test]
    fn initial_registration_rejects_successor_canonical_proof_obligation_damage() {
        let pair = chain_pair();
        for root in [&pair.destination, &pair.pinned] {
            fs::write(
                root.join(format!(
                    ".ai/decisions/{SUCCESSOR}.legacy-closeout-source-requirement.json"
                )),
                b"{broken",
            )
            .unwrap();
        }
        commit(
            &pair.pinned,
            "preserve damaged dependency obligation fixture",
        );
        let out = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&pair.pinned)
            .output()
            .unwrap();
        let revision = String::from_utf8(out.stdout).unwrap().trim().to_owned();
        let request=serde_json::from_value(json!({"schemaVersion":1,"operation":"register","sourceRevision":revision,"actor":"test-data:registrar","authoritySource":"isolated closure fixture","reason":"must conjoin dependency registration validity","sourceAccepted":true})).unwrap();
        let error = cockpit_repository::plan_legacy_closeout_source(
            &pair.destination,
            Some(&pair.pinned),
            ID,
            &request,
            &pair.registrar,
        )
        .expect_err("lower historical validation cannot waive dependency canonical obligations");
        assert!(
            error.to_string().contains("dependency_integrity_invalid"),
            "{error}"
        );
    }
}

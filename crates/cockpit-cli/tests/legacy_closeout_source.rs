use std::process::Command;

#[test]
fn explicit_source_plan_and_record_are_discoverable() {
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["work-item", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(
        help.contains("legacy-closeout-source-plan"),
        "explicit read-only source plan is missing: {help}"
    );
    assert!(
        help.contains("legacy-closeout-source-record"),
        "explicit source record is missing: {help}"
    );
}

// The production change under test is the explicit root-registration API.
// Existing historical factory APIs create all evidence before the new command
// is invoked; a missing command must be an assertion RED, not a compile error.
mod complete_source {
    use super::*;
    use cockpit_core::Digest;
    use cockpit_protocol::{HumanDecision, PROTOCOL_VERSION, RuntimeContext};
    use cockpit_repository::{
        RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
        archive_work_item_with_runtime, attach, checkpoint_work_item,
        close_work_item_with_structured_decision, finish_work_item_with_runtime,
        preflight_work_item, record_verification_with_runtime, recover_cross_checkout_closeout,
        run_repository_verification, start_work_item_with_options,
        work_item_status_snapshot_with_runtime,
    };
    use serde_json::{Value, json};
    use std::{
        collections::BTreeMap,
        fs,
        path::{Path, PathBuf},
    };
    const ID: &str = "WI-LEGACY-ROOT";

    fn git(root: &Path, args: &[&str]) -> Vec<u8> {
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
        output.stdout
    }
    fn commit(root: &Path, message: &str) {
        git(root, &["add", "-A"]);
        git(
            root,
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-qm",
                message,
            ],
        );
    }
    fn clone(source: &Path, destination: &Path) {
        git(
            source,
            &[
                "clone",
                "-q",
                "--no-hardlinks",
                "--config",
                "core.autocrlf=false",
                source.to_str().unwrap(),
                destination.to_str().unwrap(),
            ],
        );
    }
    fn old_runtime() -> RuntimeContext {
        RuntimeContext {
            runtime_version: "legacy-producer-1.0.0".into(),
            protocol_version: PROTOCOL_VERSION,
            runtime_digest: Digest::sha256_bytes(b"distinct historical producer fixture"),
        }
    }
    fn current_runtime() -> RuntimeContext {
        RuntimeContext {
            runtime_version: env!("CARGO_PKG_VERSION").into(),
            protocol_version: PROTOCOL_VERSION,
            runtime_digest: Digest::sha256_bytes(
                &fs::read(env!("CARGO_BIN_EXE_ai-cockpit")).unwrap(),
            ),
        }
    }
    fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        fn collect(root: &Path, directory: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
            for entry in fs::read_dir(directory).unwrap() {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() {
                    collect(root, &entry.path(), files);
                } else if entry.file_type().unwrap().is_file()
                    && !entry.path().to_string_lossy().ends_with(".lock")
                {
                    files.insert(
                        entry.path().strip_prefix(root).unwrap().into(),
                        fs::read(entry.path()).unwrap(),
                    );
                }
            }
        }
        let mut files = BTreeMap::new();
        collect(root, &root.join(".ai"), &mut files);
        files
    }
    fn legacy_pair() -> (tempfile::TempDir, PathBuf, tempfile::TempDir, PathBuf) {
        let origin = tempfile::tempdir().unwrap();
        let root = origin.path();
        git(root, &["init", "-q"]);
        git(root, &["config", "core.autocrlf", "false"]);
        attach(root).unwrap();
        fs::write(root.join("README.md"), "legacy fixture baseline\n").unwrap();
        commit(root, "baseline");
        let historical = old_runtime();
        start_work_item_with_options(
            root,
            ID,
            "preserve a complete historical closeout",
            "register a present observation without reauthenticating its producer",
            &["src/**".into()],
            &WorkItemStartOptions {
                authority: "authorized".into(),
                acceptance_criteria: vec!["complete immutable historical evidence".into()],
                ..Default::default()
            },
        )
        .unwrap();
        preflight_work_item(
            root,
            &root.join(format!(".ai/work-items/active/{ID}.contract.json")),
        )
        .unwrap();
        checkpoint_work_item(root, ID).unwrap();
        let execution = run_repository_verification(
            root,
            &RepositoryVerificationRequest {
                node_id: "historical-root-fixture".into(),
                program: "true".into(),
                args: vec![],
                scope: vec!["**".into()],
                stage: "task".into(),
                runner: "local".into(),
                runtime_digest: historical.runtime_digest.to_string(),
                base_commit: None,
                workers: 1,
                work_item_id: Some(ID.into()),
                timeout_seconds: None,
                policy: RepositoryVerificationPolicy::NeverReuse,
            },
        )
        .unwrap();
        record_verification_with_runtime(
            root,
            ID,
            &serde_json::to_value(execution.receipt).unwrap(),
            &historical,
            &execution.final_snapshot,
        )
        .unwrap();
        finish_work_item_with_runtime(root, ID, &historical).unwrap();
        archive_work_item_with_runtime(root, ID, &historical).unwrap();
        commit(root, "archive old producer evidence");
        let destination = tempfile::tempdir().unwrap();
        let dest = destination.path().join("clone");
        clone(root, &dest);
        close_work_item_with_structured_decision(
            root,
            ID,
            &HumanDecision {
                decision: "approved".into(),
                actor: "test-data:historical-human".into(),
                authority_source: "isolated fixture".into(),
                reason: "historical close test data".into(),
                evidence_refs: vec![format!(".ai/evidence/{ID}.verification.json")],
                policy_refs: vec![],
                decided_at: "2026-10-01T00:00:00Z".into(),
                resume_condition: None,
            },
        )
        .unwrap();
        recover_cross_checkout_closeout(&dest, root, ID, &historical).unwrap();
        // TEST FIXTURE ONLY: represent an old-v1 producer without modifying any
        // real repository evidence. Keep every payload and producer field.
        let receipt = dest.join(format!(
            ".ai/decisions/{ID}.cross-checkout-closeout-recovery.json"
        ));
        let mut v: Value = serde_json::from_slice(&fs::read(&receipt).unwrap()).unwrap();
        v["schemaVersion"] = json!(1);
        v.as_object_mut().unwrap().remove("originBinding");
        fs::write(receipt, serde_json::to_vec_pretty(&v).unwrap()).unwrap();
        fs::remove_file(dest.join(format!(
            ".ai/decisions/{ID}.cross-checkout-closeout-origin.json"
        )))
        .unwrap();
        commit(&dest, "complete legacy-v1 fixture");
        let independent = tempfile::tempdir().unwrap();
        let source = independent.path().join("source");
        clone(&dest, &source);
        let status = work_item_status_snapshot_with_runtime(&dest, ID, &current_runtime()).unwrap();
        assert_eq!(status.lifecycle_phase, "closed");
        assert_ne!(status.verification, "verified");
        (destination, dest, independent, source)
    }

    #[test]
    fn registers_complete_old_source_without_rewriting_or_revalidating_history() {
        let (_destination, dest, source_owner, source) = legacy_pair();
        let source_revision = String::from_utf8(git(&source, &["rev-parse", "HEAD"]))
            .unwrap()
            .trim()
            .to_owned();
        let before = files(&dest);
        let source_before = files(&source);
        let inputs = tempfile::tempdir().unwrap();
        let request = inputs.path().join("request.json");
        fs::write(&request,serde_json::to_vec_pretty(&json!({"schemaVersion":1,"operation":"register","sourceRevision":source_revision,"actor":"test-data:current-registrar","authoritySource":"isolated current attestation fixture","reason":"accept this complete pinned source now without authenticating the legacy producer","sourceAccepted":true})).unwrap()).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
            .args(["work-item", "legacy-closeout-source-plan", "--repo"])
            .arg(&dest)
            .arg("--source-repo")
            .arg(&source)
            .args(["--id", ID, "--input"])
            .arg(&request)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "complete old-v1 source must produce a read-only plan: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let plan: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(plan["state"], "prepared");
        assert_eq!(files(&dest), before, "plan must not write destination");
        assert_eq!(files(&source), source_before, "plan must not write source");
        let plan_path = inputs.path().join("original-plan.json");
        fs::write(&plan_path, &output.stdout).unwrap();
        let record = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
            .args(["work-item", "legacy-closeout-source-record", "--repo"])
            .arg(&dest)
            .args(["--id", ID, "--input"])
            .arg(&plan_path)
            .output()
            .unwrap();
        assert!(
            record.status.success(),
            "{}",
            String::from_utf8_lossy(&record.stderr)
        );
        let result: Value = serde_json::from_slice(&record.stdout).unwrap();
        assert_eq!(result["state"], "committed");
        for (path, bytes) in before {
            assert_eq!(
                fs::read(dest.join(path)).unwrap(),
                bytes,
                "historical bytes must remain unchanged"
            );
        }
        let registration: Value = serde_json::from_slice(
            &fs::read(dest.join(format!(
                ".ai/decisions/{ID}.legacy-closeout-source-registration.json"
            )))
            .unwrap(),
        )
        .unwrap();
        assert_eq!(registration["assurance"], "explicit_attestation");
        assert_eq!(registration["actor"], "test-data:current-registrar");
        assert_eq!(
            registration["runtime"],
            serde_json::to_value(current_runtime()).unwrap()
        );
        drop(source_owner);
        let status = work_item_status_snapshot_with_runtime(&dest, ID, &current_runtime()).unwrap();
        assert_eq!(status.lifecycle_phase, "closed");
        assert_ne!(
            status.verification, "verified",
            "registration does not execute historical checks under the current Runtime"
        );
    }

    fn original_plan(dest: &Path, source: &Path) -> cockpit_protocol::LegacyCloseoutSourcePlan {
        let revision = String::from_utf8(git(source, &["rev-parse", "HEAD"]))
            .unwrap()
            .trim()
            .to_owned();
        let request:cockpit_protocol::LegacyCloseoutSourceRequest=serde_json::from_value(json!({"schemaVersion":1,"operation":"register","sourceRevision":revision,"actor":"test-data:registrar","authoritySource":"isolated fixture","reason":"accept exact source","sourceAccepted":true})).unwrap();
        cockpit_repository::plan_legacy_closeout_source(
            dest,
            Some(source),
            ID,
            &request,
            &current_runtime(),
        )
        .unwrap()
    }

    #[test]
    fn forged_resume_cannot_create_a_first_root_after_source_disappears() {
        let (_destination, dest, source_owner, source) = legacy_pair();
        let mut plan = original_plan(&dest, &source);
        drop(source_owner);
        // An input author can recompute self-checking JSON digests. They cannot
        // turn a first registration into resume to skip the full pinned source.
        plan.operation = cockpit_protocol::LegacyCloseoutSourceOperation::Resume;
        let mut value = serde_json::to_value(&plan).unwrap();
        value.as_object_mut().unwrap().remove("planDigest");
        plan.plan_digest = cockpit_protocol::digest_json(&value).unwrap();
        let result =
            cockpit_repository::record_legacy_closeout_source(&dest, ID, &plan, &current_runtime());
        assert!(
            result.is_err(),
            "resume without a surviving obligation must not publish a first root"
        );
        assert!(
            !dest
                .join(format!(
                    ".ai/decisions/{ID}.legacy-closeout-source-requirement.json"
                ))
                .exists()
        );
        assert!(
            !dest
                .join(format!(
                    ".ai/decisions/{ID}.legacy-closeout-source-registration.json"
                ))
                .exists()
        );
    }

    #[test]
    fn malformed_extension_remains_an_obligation_after_valid_root_registration() {
        let (_destination, dest, _source_owner, source) = legacy_pair();
        let plan = original_plan(&dest, &source);
        cockpit_repository::record_legacy_closeout_source(&dest, ID, &plan, &current_runtime())
            .unwrap();
        fs::write(
            dest.join(format!(
                ".ai/decisions/{ID}.legacy-closeout-source-extension.json"
            )),
            b"{broken",
        )
        .unwrap();
        let status = work_item_status_snapshot_with_runtime(&dest, ID, &current_runtime()).unwrap();
        assert_ne!(
            status.lifecycle_phase, "closed",
            "a valid root does not authorize a malformed extension"
        );
        assert!(status.blocking, "surviving malformed extension must block");
    }

    #[test]
    fn lost_archive_keeps_structured_status_and_index_blocking() {
        let (_destination, dest, _source_owner, source) = legacy_pair();
        let plan = original_plan(&dest, &source);
        cockpit_repository::record_legacy_closeout_source(&dest, ID, &plan, &current_runtime())
            .unwrap();
        fs::remove_dir_all(dest.join(".ai/work-items/archive")).unwrap();
        let status = work_item_status_snapshot_with_runtime(&dest, ID, &current_runtime())
            .expect("canonical obligations must retain a structured status without the Contract");
        assert!(status.blocking);
        assert_ne!(status.lifecycle_phase, "closed");
        assert!(
            status
                .blockers
                .contains(&"legacy_closeout_source_integrity_invalid".into())
        );
        let index =
            cockpit_repository::work_item_status_index_with_runtime(&dest, &current_runtime())
                .unwrap();
        let value = serde_json::to_value(index).unwrap();
        assert!(
            value["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["workItemId"] == ID),
            "aggregate index must union canonical proof names: {value}"
        );
    }

    #[test]
    fn delegated_observation_and_raw_dependencies_are_bound_and_loss_revokes_closed() {
        let (_destination, dest, _source_owner, source) = legacy_pair();
        let raw = b"provider observation fixture\n";
        let raw_ref = format!(".ai/evidence/external/{ID}.provider.raw");
        let evidence = cockpit_protocol::DelegatedEvidence {
            provider: "fixture".into(),
            subject: "complete legacy dependency".into(),
            origin: "isolated fixture API".into(),
            assurance: cockpit_protocol::EvidenceAssurance::SelfDeclared,
            collected_at: "2026-10-01T00:00:00Z".into(),
            digest: Digest::sha256_bytes(raw),
            validity: cockpit_protocol::EvidenceValidity::Valid,
            raw_evidence_ref: raw_ref.clone(),
        };
        cockpit_repository::import_delegated_evidence(&dest, ID, &evidence, raw, &old_runtime())
            .unwrap();
        let observation = cockpit_repository::import_delegated_evidence_observation(
            &dest,
            ID,
            &evidence,
            raw,
            &old_runtime(),
        )
        .unwrap();
        commit(&dest, "retain full delegated and observation dependencies");
        git(&source, &["pull", "--ff-only", "--quiet"]);
        let plan = original_plan(&dest, &source);
        let registration: Value = serde_json::from_str(&plan.registration_bytes).unwrap();
        let bindings = registration["dependencyBindings"].as_array().unwrap();
        assert!(
            bindings.iter().any(|entry| entry["path"] == raw_ref),
            "raw bytes must be an explicit dependency"
        );
        assert!(
            bindings.iter().any(|entry| entry["path"]
                .as_str()
                .unwrap()
                .contains(&observation.receipt.observation_digest.to_string()[7..])),
            "exact observation selector must be retained"
        );
        cockpit_repository::record_legacy_closeout_source(&dest, ID, &plan, &current_runtime())
            .unwrap();
        fs::remove_file(dest.join(raw_ref)).unwrap();
        let status = work_item_status_snapshot_with_runtime(&dest, ID, &current_runtime()).unwrap();
        assert_ne!(status.lifecycle_phase, "closed");
        assert!(status.blocking);
    }

    #[test]
    fn malformed_delegated_digest_returns_error_without_panicking() {
        let (_destination, dest, _source_owner, source) = legacy_pair();
        fs::create_dir_all(dest.join(".ai/evidence/external")).unwrap();
        let bad = json!({"schemaVersion":1,"repositoryId":cockpit_repository::repository_id(&dest).to_string(),"workItemId":ID,"runtimeVersion":"historical","runtimeDigest":Digest::sha256_bytes(b"historical"),"boundAt":"2026-10-01T00:00:00Z","evidence":{"provider":"fixture","subject":"bad digest","origin":"isolated fixture","assurance":"self_declared","collectedAt":"2026-10-01T00:00:00Z","digest":"x","validity":"valid","rawEvidenceRef":format!(".ai/evidence/external/{ID}.raw")}});
        fs::write(
            dest.join(format!(".ai/evidence/external/{ID}.bad.delegated.json")),
            serde_json::to_vec_pretty(&bad).unwrap(),
        )
        .unwrap();
        commit(&dest, "malformed dependency fixture");
        git(&source, &["pull", "--ff-only", "--quiet"]);
        let revision = String::from_utf8(git(&source, &["rev-parse", "HEAD"]))
            .unwrap()
            .trim()
            .to_owned();
        let request:cockpit_protocol::LegacyCloseoutSourceRequest=serde_json::from_value(json!({"schemaVersion":1,"operation":"register","sourceRevision":revision,"actor":"fixture","authoritySource":"fixture","reason":"malformed input must be an error","sourceAccepted":true})).unwrap();
        let result = std::panic::catch_unwind(|| {
            cockpit_repository::plan_legacy_closeout_source(
                &dest,
                Some(&source),
                ID,
                &request,
                &current_runtime(),
            )
        });
        assert!(result.is_ok(), "malformed untrusted digests must not panic");
        assert!(result.unwrap().is_err());
    }

    #[test]
    fn partial_root_resume_preserves_first_exact_bytes_with_independent_binding() {
        for removed in ["requirement", "registration"] {
            let (_destination, dest, source_owner, source) = legacy_pair();
            let original = original_plan(&dest, &source);
            cockpit_repository::record_legacy_closeout_source(
                &dest,
                ID,
                &original,
                &current_runtime(),
            )
            .unwrap();
            let path = dest.join(format!(
                ".ai/decisions/{ID}.legacy-closeout-source-{removed}.json"
            ));
            fs::remove_file(&path).unwrap();
            drop(source_owner);
            let request:cockpit_protocol::LegacyCloseoutSourceRequest=serde_json::from_value(json!({"schemaVersion":1,"operation":"resume","originalRegistrationBytes":original.registration_bytes,"originalRequirementRawDigest":original.requirement_raw_digest})).unwrap();
            let resume = cockpit_repository::plan_legacy_closeout_source(
                &dest,
                None,
                ID,
                &request,
                &current_runtime(),
            )
            .unwrap();
            let result = cockpit_repository::record_legacy_closeout_source(
                &dest,
                ID,
                &resume,
                &current_runtime(),
            )
            .unwrap();
            assert_eq!(result["state"], "committed");
            assert_eq!(
                fs::read(dest.join(format!(
                    ".ai/decisions/{ID}.legacy-closeout-source-requirement.json"
                )))
                .unwrap(),
                original.requirement_bytes.as_bytes()
            );
            assert_eq!(
                fs::read(dest.join(format!(
                    ".ai/decisions/{ID}.legacy-closeout-source-registration.json"
                )))
                .unwrap(),
                original.registration_bytes.as_bytes()
            );
        }
    }

    #[test]
    fn damaged_survivor_cannot_become_original_through_a_fresh_plan() {
        let (_destination, dest, _source_owner, source) = legacy_pair();
        let original = original_plan(&dest, &source);
        cockpit_repository::record_legacy_closeout_source(&dest, ID, &original, &current_runtime())
            .unwrap();
        fs::remove_file(dest.join(format!(
            ".ai/decisions/{ID}.legacy-closeout-source-registration.json"
        )))
        .unwrap();
        let path = dest.join(format!(
            ".ai/decisions/{ID}.legacy-closeout-source-requirement.json"
        ));
        let mut requirement: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let mut changed: Value =
            serde_json::from_str(requirement["registrationBytes"].as_str().unwrap()).unwrap();
        changed["actor"] = json!("changed actor");
        let changed_bytes = serde_json::to_vec_pretty(&changed).unwrap();
        requirement["registrationBytes"] = String::from_utf8(changed_bytes.clone()).unwrap().into();
        requirement["registrationRawDigest"] =
            Digest::sha256_bytes(&changed_bytes).to_string().into();
        let changed_requirement = serde_json::to_vec_pretty(&requirement).unwrap();
        fs::write(&path, &changed_requirement).unwrap();
        let request:cockpit_protocol::LegacyCloseoutSourceRequest=serde_json::from_value(json!({"schemaVersion":1,"operation":"resume","originalRegistrationBytes":original.registration_bytes,"originalRequirementRawDigest":original.requirement_raw_digest})).unwrap();
        assert!(
            cockpit_repository::plan_legacy_closeout_source(
                &dest,
                None,
                ID,
                &request,
                &current_runtime()
            )
            .is_err()
        );
        let no_original: cockpit_protocol::LegacyCloseoutSourceRequest =
            serde_json::from_value(json!({"schemaVersion":1,"operation":"resume"})).unwrap();
        assert!(
            cockpit_repository::plan_legacy_closeout_source(
                &dest,
                None,
                ID,
                &no_original,
                &current_runtime()
            )
            .is_err()
        );
        assert_eq!(fs::read(path).unwrap(), changed_requirement);
    }
    #[test]
    fn complete_source_rejects_missing_legacy_identity_header_without_rewriting_history() {
        let (_destination, dest, _source_owner, source) = legacy_pair();
        let receipt = format!(".ai/decisions/{ID}.cross-checkout-closeout-recovery.json");
        for root in [&dest, &source] {
            let path = root.join(&receipt);
            let mut legacy: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            legacy.as_object_mut().unwrap().remove("runtimeDigest");
            fs::write(path, serde_json::to_vec_pretty(&legacy).unwrap()).unwrap();
        }
        commit(&source, "preserve incomplete legacy identity fixture");
        // New registration is strict; unregistered historical compatibility is
        // independently preserved rather than silently changing old evidence.
        assert_eq!(
            work_item_status_snapshot_with_runtime(&dest, ID, &current_runtime())
                .unwrap()
                .lifecycle_phase,
            "closed"
        );
        let revision = String::from_utf8(git(&source, &["rev-parse", "HEAD"]))
            .unwrap()
            .trim()
            .to_owned();
        let request=serde_json::from_value(json!({"schemaVersion":1,"operation":"register","sourceRevision":revision,"actor":"test-data:registrar","authoritySource":"isolated fixture","reason":"reject incomplete original identity","sourceAccepted":true})).unwrap();
        let result = cockpit_repository::plan_legacy_closeout_source(
            &dest,
            Some(&source),
            ID,
            &request,
            &current_runtime(),
        );
        assert!(
            result.is_err(),
            "a complete pinned source cannot replace missing legacy Runtime identity"
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("legacy_receipt_invalid")
        );
    }
    #[test]
    fn register_request_cannot_silently_ignore_extension_fields() {
        let (_destination, dest, _source_owner, source) = legacy_pair();
        let revision = String::from_utf8(git(&source, &["rev-parse", "HEAD"]))
            .unwrap()
            .trim()
            .to_owned();
        let request=serde_json::from_value(json!({"schemaVersion":1,"operation":"register","sourceRevision":revision,"actor":"test-data:registrar","authoritySource":"isolated fixture","reason":"require one unambiguous operation","sourceAccepted":true,"transitionBytes":"unrelated operation content"})).unwrap();
        let result = cockpit_repository::plan_legacy_closeout_source(
            &dest,
            Some(&source),
            ID,
            &request,
            &current_runtime(),
        );
        assert!(
            result.is_err(),
            "typed operation roles must reject ignored fields"
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("request_role_invalid")
        );
    }
    #[test]
    fn fresh_plan_rejects_identical_content_destination_root_replacement() {
        assert_replacement_rejected("destination_root");
    }
    #[test]
    fn fresh_plan_rejects_identical_content_destination_git_replacement() {
        assert_replacement_rejected("destination_git");
    }
    #[test]
    fn fresh_plan_rejects_identical_content_source_root_replacement() {
        assert_replacement_rejected("source_root");
    }
    fn assert_replacement_rejected(subject: &str) {
        fn copy_tree(source: &Path, destination: &Path) {
            fs::create_dir_all(destination).unwrap();
            for entry in fs::read_dir(source).unwrap() {
                let entry = entry.unwrap();
                let target = destination.join(entry.file_name());
                if entry.file_type().unwrap().is_dir() {
                    copy_tree(&entry.path(), &target);
                } else {
                    fs::copy(entry.path(), target).unwrap();
                }
            }
        }
        {
            let (_destination, dest, _source_owner, source) = legacy_pair();
            let revision = String::from_utf8(git(&source, &["rev-parse", "HEAD"]))
                .unwrap()
                .trim()
                .to_owned();
            let request=serde_json::from_value(json!({"schemaVersion":1,"operation":"register","sourceRevision":revision,"actor":"test-data:registrar","authoritySource":"isolated fixture","reason":"bind actual directories as well as bytes","sourceAccepted":true})).unwrap();
            let runtime = current_runtime();
            let plan = cockpit_repository::plan_legacy_closeout_source(
                &dest,
                Some(&source),
                ID,
                &request,
                &runtime,
            )
            .unwrap();
            let path = match subject {
                "destination_root" => dest.clone(),
                "destination_git" => dest.join(".git"),
                _ => source.clone(),
            };
            let retained = if subject == "source_root" {
                source.parent().unwrap()
            } else {
                dest.parent().unwrap()
            }
            .join(format!("retained-{subject}"));
            fs::rename(&path, &retained).unwrap();
            copy_tree(&retained, &path);
            let result =
                cockpit_repository::record_legacy_closeout_source(&dest, ID, &plan, &runtime);
            assert!(
                result.is_err(),
                "plan must reject physical replacement of {subject}"
            );
            assert!(
                !dest
                    .join(format!(
                        ".ai/decisions/{ID}.legacy-closeout-source-requirement.json"
                    ))
                    .exists()
            );
            assert!(
                !dest
                    .join(format!(
                        ".ai/decisions/{ID}.legacy-closeout-source-registration.json"
                    ))
                    .exists()
            );
        }
    }
    #[test]
    fn concurrent_root_writers_share_one_winner_and_preserve_conflicting_plans() {
        for same_binding in [true, false] {
            let (_destination, dest, _source_owner, source) = legacy_pair();
            let revision = String::from_utf8(git(&source, &["rev-parse", "HEAD"]))
                .unwrap()
                .trim()
                .to_owned();
            let request: cockpit_protocol::LegacyCloseoutSourceRequest = serde_json::from_value(json!({"schemaVersion":1,"operation":"register","sourceRevision":revision,"actor":"test-data:first-registrar","authoritySource":"isolated fixture","reason":"serialize exact root writers","sourceAccepted":true})).unwrap();
            let runtime = current_runtime();
            let first = cockpit_repository::plan_legacy_closeout_source(
                &dest,
                Some(&source),
                ID,
                &request,
                &runtime,
            )
            .unwrap();
            let second = if same_binding {
                first.clone()
            } else {
                let mut other = request;
                other.actor = Some("test-data:conflicting-registrar".into());
                cockpit_repository::plan_legacy_closeout_source(
                    &dest,
                    Some(&source),
                    ID,
                    &other,
                    &runtime,
                )
                .unwrap()
            };
            let barrier = std::sync::Barrier::new(2);
            let results = std::thread::scope(|scope| {
                let handles = [&first, &second]
                    .into_iter()
                    .map(|plan| {
                        let barrier = &barrier;
                        let dest = &dest;
                        let runtime = &runtime;
                        scope.spawn(move || {
                            barrier.wait();
                            cockpit_repository::record_legacy_closeout_source(
                                dest, ID, plan, runtime,
                            )
                        })
                    })
                    .collect::<Vec<_>>();
                handles
                    .into_iter()
                    .map(|handle| handle.join().unwrap())
                    .collect::<Vec<_>>()
            });
            assert_eq!(
                results.iter().filter(|result| result.is_ok()).count(),
                if same_binding { 2 } else { 1 }
            );
            let bytes = fs::read(dest.join(format!(
                ".ai/decisions/{ID}.legacy-closeout-source-registration.json"
            )))
            .unwrap();
            assert!(
                bytes == first.registration_bytes.as_bytes()
                    || bytes == second.registration_bytes.as_bytes()
            );
            let winner = if bytes == first.registration_bytes.as_bytes() {
                &first
            } else {
                &second
            };
            assert_eq!(
                fs::read(dest.join(format!(
                    ".ai/decisions/{ID}.legacy-closeout-source-requirement.json"
                )))
                .unwrap(),
                winner.requirement_bytes.as_bytes()
            );
            assert_eq!(
                work_item_status_snapshot_with_runtime(&dest, ID, &runtime)
                    .unwrap()
                    .lifecycle_phase,
                "closed"
            );
        }
    }
    #[test]
    fn read_only_plan_cannot_return_an_unrecordable_oversized_template() {
        let (_destination, dest, _source_owner, source) = legacy_pair();
        let revision = String::from_utf8(git(&source, &["rev-parse", "HEAD"]))
            .unwrap()
            .trim()
            .to_owned();
        let request = serde_json::from_value(json!({"schemaVersion":1,"operation":"register","sourceRevision":revision,"actor":"a".repeat(5 * 1024 * 1024),"authoritySource":"isolated fixture","reason":"bound the complete frozen plan","sourceAccepted":true})).unwrap();
        let before = files(&dest);
        let result = cockpit_repository::plan_legacy_closeout_source(
            &dest,
            Some(&source),
            ID,
            &request,
            &current_runtime(),
        );
        assert!(
            result.is_err(),
            "the whole serialized plan must fit the input budget"
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("record_budget_exceeded")
        );
        assert_eq!(files(&dest), before);
    }
    #[test]
    fn every_bound_history_file_and_each_root_obligation_remain_required() {
        let (_destination, dest, _source_owner, source) = legacy_pair();
        let plan = original_plan(&dest, &source);
        let runtime = current_runtime();
        cockpit_repository::record_legacy_closeout_source(&dest, ID, &plan, &runtime).unwrap();
        let registration: cockpit_protocol::LegacyCloseoutSourceRegistration =
            serde_json::from_str(&plan.registration_bytes).unwrap();
        let paths = registration
            .inventory
            .iter()
            .map(|file| file.path.clone())
            .chain([
                format!(".ai/decisions/{ID}.legacy-closeout-source-requirement.json"),
                format!(".ai/decisions/{ID}.legacy-closeout-source-registration.json"),
            ]);
        for relative in paths {
            let path = dest.join(&relative);
            let original = fs::read(&path).unwrap();
            for missing in [true, false] {
                if missing {
                    fs::remove_file(&path).unwrap();
                } else {
                    let mut changed = original.clone();
                    changed.push(b'\n');
                    fs::write(&path, changed).unwrap();
                }
                let status = work_item_status_snapshot_with_runtime(&dest, ID, &runtime).unwrap();
                assert_ne!(
                    status.lifecycle_phase, "closed",
                    "{relative} missing={missing}"
                );
                assert!(status.blocking, "{relative} missing={missing}");
                let observed = cockpit_repository::status(&dest).unwrap();
                assert!(
                    observed
                        .readiness
                        .unclosed_archived_work_items
                        .contains(&ID.to_owned()),
                    "{relative} missing={missing}"
                );
                fs::write(&path, &original).unwrap();
                assert_eq!(
                    work_item_status_snapshot_with_runtime(&dest, ID, &runtime)
                        .unwrap()
                        .lifecycle_phase,
                    "closed"
                );
            }
        }
        let manifest_path = dest.join(format!(".ai/work-items/archive/{ID}.archive.json"));
        let original_manifest = fs::read(&manifest_path).unwrap();
        let mut manifest: Value = serde_json::from_slice(&original_manifest).unwrap();
        manifest["closeRequired"] = json!(false);
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert_ne!(
            work_item_status_snapshot_with_runtime(&dest, ID, &runtime)
                .unwrap()
                .lifecycle_phase,
            "closed"
        );
        assert!(
            cockpit_repository::status(&dest)
                .unwrap()
                .readiness
                .unclosed_archived_work_items
                .contains(&ID.to_owned())
        );
        let attempt = start_work_item_with_options(
            &dest,
            "WI-OVERLAPPING-PROOF",
            "do not free scope",
            "reject a damaged registered obligation",
            &["src/**".into()],
            &WorkItemStartOptions {
                authority: "authorized".into(),
                acceptance_criteria: vec!["keep the original registered scope".into()],
                ..Default::default()
            },
        );
        assert!(attempt.is_err());
        fs::write(manifest_path, original_manifest).unwrap();
    }
}

#[test]
fn legacy_source_input_is_bounded_before_json_parsing() {
    let repo = tempfile::tempdir().unwrap();
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(repo.path())
            .status()
            .unwrap()
            .success()
    );
    cockpit_repository::attach(repo.path()).unwrap();
    let input = tempfile::NamedTempFile::new().unwrap();
    input.as_file().set_len(9 * 1024 * 1024).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args([
            "work-item",
            "legacy-closeout-source-plan",
            "--repo",
            repo.path().to_str().unwrap(),
            "--id",
            "WI-BOUND",
            "--input",
            input.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error.contains("bounded read limit"),
        "size must be rejected before parsing: {error}"
    );
}

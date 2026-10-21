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

    use cockpit_core::Digest;
    use cockpit_protocol::HumanDecision;
    use cockpit_protocol::PROTOCOL_VERSION;
    use cockpit_protocol::RuntimeContext;
    use cockpit_repository::RepositoryVerificationPolicy;
    use cockpit_repository::RepositoryVerificationRequest;
    use cockpit_repository::WorkItemStartOptions;
    use cockpit_repository::archive_work_item_with_runtime;
    use cockpit_repository::attach;
    use cockpit_repository::checkpoint_work_item;
    use cockpit_repository::close_work_item_with_structured_decision;
    use cockpit_repository::finish_work_item_with_runtime;
    use cockpit_repository::preflight_work_item;
    use cockpit_repository::record_verification_with_runtime;
    use cockpit_repository::recover_cross_checkout_closeout;
    use cockpit_repository::run_repository_verification;
    use cockpit_repository::start_work_item_with_options;
    use cockpit_repository::work_item_status_snapshot_with_runtime;
    use serde_json::Value;
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
    use std::process::Command;
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

    fn cli_source(
        root: &Path,
        source: Option<&Path>,
        command: &str,
        input: &Value,
    ) -> std::process::Output {
        let file = tempfile::NamedTempFile::new().unwrap();
        fs::write(file.path(), serde_json::to_vec(input).unwrap()).unwrap();
        let mut process = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"));
        process
            .args(["work-item", command, "--repo"])
            .arg(root)
            .args(["--id", ID, "--input"])
            .arg(file.path());
        if let Some(source) = source {
            process.arg("--source-repo").arg(source);
        }
        process.output().unwrap()
    }
    fn mcp_source(root: &Path, command: &str, field: &str, input: &Value) -> Value {
        cockpit_mcp::handle_request_for_repo(
            &json!({"jsonrpc":"2.0","id":41,"method":"tools/call","params":{"name":command,"arguments":{"workItemId":ID,(field):input}}}),
            root,
            &current_runtime(),
        )
    }
    fn cli_success(output: std::process::Output) -> Value {
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }

    #[test]
    fn actual_cli_transport_replay_resume_proof_loss_and_source_drift_match_mcp() {
        let (_destination, dest, source_owner, source) = legacy_pair();
        let revision = String::from_utf8(git(&source, &["rev-parse", "HEAD"]))
            .unwrap()
            .trim()
            .to_owned();
        let request = json!({"schemaVersion":1,"operation":"register","sourceRevision":revision,"actor":"test-data:transport","authoritySource":"isolated CLI/MCP fixture","reason":"bind actual transport behavior","sourceAccepted":true});
        let before = files(&dest);
        let plan = cli_success(cli_source(
            &dest,
            Some(&source),
            "legacy-closeout-source-plan",
            &request,
        ));
        assert_eq!(files(&dest), before, "actual CLI plan must be read-only");
        let committed = cli_success(cli_source(
            &dest,
            None,
            "legacy-closeout-source-record",
            &plan,
        ));
        assert_eq!(committed["state"], "committed");
        let registered = files(&dest);
        drop(source_owner);
        let replay = cli_success(cli_source(
            &dest,
            None,
            "legacy-closeout-source-record",
            &plan,
        ));
        let mcp = mcp_source(
            &dest,
            "work_item_legacy_closeout_source_record",
            "plan",
            &plan,
        );
        assert_eq!(mcp["result"]["isError"], false, "{mcp}");
        assert_eq!(replay, mcp["result"]["structuredContent"]);
        assert_eq!(
            files(&dest),
            registered,
            "both replays preserve all exact bytes offline"
        );
        let registration = dest.join(format!(
            ".ai/decisions/{ID}.legacy-closeout-source-registration.json"
        ));
        let original_bytes = fs::read(&registration).unwrap();
        fs::remove_file(&registration).unwrap();
        assert_ne!(
            work_item_status_snapshot_with_runtime(&dest, ID, &current_runtime())
                .unwrap()
                .lifecycle_phase,
            "closed"
        );
        let resume_request = json!({"schemaVersion":1,"operation":"resume","originalRegistrationBytes":plan["registrationBytes"],"originalRequirementRawDigest":plan["requirementRawDigest"]});
        let partial = files(&dest);
        let resumed = cli_success(cli_source(
            &dest,
            None,
            "legacy-closeout-source-plan",
            &resume_request,
        ));
        assert_eq!(files(&dest), partial);
        let repaired = cli_success(cli_source(
            &dest,
            None,
            "legacy-closeout-source-record",
            &resumed,
        ));
        let mcp = mcp_source(
            &dest,
            "work_item_legacy_closeout_source_record",
            "plan",
            &resumed,
        );
        assert_eq!(mcp["result"]["isError"], false, "{mcp}");
        assert_eq!(repaired["state"], "committed");
        assert_eq!(
            repaired["changedPaths"],
            json!([format!(
                ".ai/decisions/{ID}.legacy-closeout-source-registration.json"
            )])
        );
        assert!(repaired["publicationObservedAt"].is_string());
        let repair_replay = cli_success(cli_source(
            &dest,
            None,
            "legacy-closeout-source-record",
            &resumed,
        ));
        assert_eq!(repair_replay, mcp["result"]["structuredContent"]);
        assert_eq!(repair_replay["changedPaths"], json!([]));
        for field in ["registrationRawDigest", "requirementRawDigest"] {
            assert_eq!(repaired[field], repair_replay[field]);
        }
        assert_eq!(fs::read(registration).unwrap(), original_bytes);
        assert_eq!(files(&dest), registered);
        assert_eq!(
            work_item_status_snapshot_with_runtime(&dest, ID, &current_runtime())
                .unwrap()
                .lifecycle_phase,
            "closed"
        );

        let (_destination, dest, _source_owner, source) = legacy_pair();
        let plan = serde_json::to_value(original_plan(&dest, &source)).unwrap();
        git(
            &source,
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--allow-empty",
                "-qm",
                "advance only disposable source HEAD",
            ],
        );
        let before = files(&dest);
        let cli = cli_source(&dest, None, "legacy-closeout-source-record", &plan);
        assert!(!cli.status.success());
        assert!(String::from_utf8_lossy(&cli.stderr).contains("source_stale"));
        let mcp = mcp_source(
            &dest,
            "work_item_legacy_closeout_source_record",
            "plan",
            &plan,
        );
        assert_eq!(mcp["result"]["isError"], true, "{mcp}");
        assert!(
            mcp["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("source_stale")
        );
        assert_eq!(
            files(&dest),
            before,
            "drift rejection must not publish either root"
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

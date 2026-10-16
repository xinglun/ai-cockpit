use cockpit_core::Digest;
use cockpit_mcp::{handle_request, handle_request_for_repo};
use cockpit_protocol::{PROTOCOL_VERSION, RuntimeContext};
use serde_json::json;
fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "legacy-source-mcp-test".into(),
        protocol_version: PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(b"legacy-source-mcp-test"),
    }
}
#[test]
fn legacy_source_tools_expose_strict_typed_plan_and_record() {
    let response = handle_request(
        &json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}),
        &runtime(),
    );
    let tools = response["result"]["tools"].as_array().unwrap();
    for (name, required) in [
        ("work_item_legacy_closeout_source_plan", "request"),
        ("work_item_legacy_closeout_source_record", "plan"),
    ] {
        let tool = tools
            .iter()
            .find(|tool| tool["name"] == name)
            .expect("explicit legacy source tool is discoverable");
        assert_eq!(tool["inputSchema"]["additionalProperties"], false);
        assert!(
            tool["inputSchema"]["required"]
                .as_array()
                .unwrap()
                .contains(&json!(required))
        );
        assert_eq!(
            tool["inputSchema"]["properties"][required]["additionalProperties"],
            false
        );
    }
}
#[test]
fn wrong_nested_request_is_rejected_before_repository_access() {
    let absent = std::env::temp_dir().join("ai-cockpit-mcp-legacy-source-absent");
    let response = handle_request_for_repo(
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"work_item_legacy_closeout_source_plan","arguments":{"workItemId":"WI-LEGACY-SOURCE","request":{"schemaVersion":1,"operation":"extend","unexpected":true}}}}),
        &absent,
        &runtime(),
    );
    assert_eq!(response["result"]["isError"], true, "{response}");
    let message = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(message.contains("unknown field"), "{message}");
    assert!(!absent.exists());
}

#[test]
fn raw_legacy_rpc_rejects_duplicate_keys_before_repository_access() {
    let raw = br#"{"jsonrpc":"2.0","id":33,"method":"tools/call","params":{"name":"work_item_legacy_closeout_source_plan","arguments":{"workItemId":"WI-LEGACY-SOURCE","request":{"schemaVersion":1,"operation":"register","operation":"resume"}}}}"#;
    for bound in [false, true] {
        let mut output = Vec::new();
        let reader = std::io::Cursor::new(raw);
        if bound {
            cockpit_mcp::serve_with_repo(
                reader,
                &mut output,
                &std::env::temp_dir().join("absent-legacy-rpc-fixture"),
                &runtime(),
            )
            .unwrap();
        } else {
            cockpit_mcp::serve(reader, &mut output, &runtime()).unwrap();
        }
        let response: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(response["result"]["isError"], true);
        assert!(
            response["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("duplicate JSON object key"),
            "{response}"
        );
    }
}

#[test]
fn rpc_line_is_bounded_before_json_parsing() {
    let bytes = vec![b' '; 16 * 1024 * 1024 + 1];
    let mut output = Vec::new();
    let error =
        cockpit_mcp::serve(std::io::Cursor::new(bytes), &mut output, &runtime()).unwrap_err();
    assert!(
        error.to_string().contains("rpc_line_budget_exceeded"),
        "{error}"
    );
    assert!(output.is_empty());
}

mod complete_source_parity {
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
    use std::process::Command;
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
        super::runtime()
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

    fn call(root: &Path, name: &str, arguments: Value) -> Value {
        handle_request_for_repo(
            &json!({"jsonrpc":"2.0","id":17,"method":"tools/call","params":{"name":name,"arguments":arguments}}),
            root,
            &runtime(),
        )
    }
    fn success(response: Value) -> Value {
        assert_eq!(response["result"]["isError"], false, "{response}");
        response["result"]["structuredContent"].clone()
    }
    #[test]
    fn complete_history_plan_record_resume_offline_and_proof_loss_share_repository_semantics() {
        let (_destination, dest, source_owner, source) = legacy_pair();
        let revision = String::from_utf8(git(&source, &["rev-parse", "HEAD"]))
            .unwrap()
            .trim()
            .to_owned();
        let before = files(&dest);
        let unknown = call(
            &dest,
            "work_item_legacy_closeout_source_plan",
            json!({"workItemId":ID,"request":{"schemaVersion":1,"operation":"register","sourceRevision":revision,"sourceAccepted":true,"actor":"test-data:mcp-registrar","authoritySource":"isolated fixture","reason":"a hash alone is not a source"}}),
        );
        assert_eq!(unknown["result"]["isError"], true);
        assert!(
            unknown["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("provenance_unknown")
        );
        assert_eq!(files(&dest), before);
        let plan_value = success(call(
            &dest,
            "work_item_legacy_closeout_source_plan",
            json!({"workItemId":ID,"sourceRepo":source,"request":{"schemaVersion":1,"operation":"register","sourceRevision":revision,"sourceAccepted":true,"actor":"test-data:mcp-registrar","authoritySource":"isolated fixture","reason":"complete source through MCP"}}),
        ));
        assert_eq!(files(&dest), before, "MCP plan must remain read only");
        let plan: cockpit_protocol::LegacyCloseoutSourcePlan =
            serde_json::from_value(plan_value.clone()).unwrap();
        let committed = success(call(
            &dest,
            "work_item_legacy_closeout_source_record",
            json!({"workItemId":ID,"plan":plan_value}),
        ));
        assert_eq!(committed["state"], "committed");
        let replay =
            cockpit_repository::record_legacy_closeout_source(&dest, ID, &plan, &runtime())
                .unwrap();
        let mcp_replay = success(call(
            &dest,
            "work_item_legacy_closeout_source_record",
            json!({"workItemId":ID,"plan":plan}),
        ));
        assert_eq!(replay, mcp_replay);
        assert_eq!(
            replay["registrationRawDigest"],
            committed["registrationRawDigest"]
        );
        drop(source_owner);
        let status = work_item_status_snapshot_with_runtime(&dest, ID, &runtime()).unwrap();
        assert_eq!(status.lifecycle_phase, "closed");
        assert_ne!(status.verification, "verified");
        let registration = dest.join(format!(
            ".ai/decisions/{ID}.legacy-closeout-source-registration.json"
        ));
        let original = fs::read(&registration).unwrap();
        fs::remove_file(&registration).unwrap();
        assert_ne!(
            work_item_status_snapshot_with_runtime(&dest, ID, &runtime())
                .unwrap()
                .lifecycle_phase,
            "closed"
        );
        let resume = success(call(
            &dest,
            "work_item_legacy_closeout_source_plan",
            json!({"workItemId":ID,"request":{"schemaVersion":1,"operation":"resume","originalRegistrationBytes":plan.registration_bytes,"originalRequirementRawDigest":plan.requirement_raw_digest}}),
        ));
        assert_eq!(
            success(call(
                &dest,
                "work_item_legacy_closeout_source_record",
                json!({"workItemId":ID,"plan":resume})
            ))["state"],
            "committed"
        );
        assert_eq!(fs::read(&registration).unwrap(), original);
        let mut damaged = original;
        damaged.push(b'\n');
        fs::write(&registration, &damaged).unwrap();
        let rejected = call(
            &dest,
            "work_item_legacy_closeout_source_record",
            json!({"workItemId":ID,"plan":plan}),
        );
        assert_eq!(rejected["result"]["isError"], true);
        assert!(
            cockpit_repository::record_legacy_closeout_source(&dest, ID, &plan, &runtime())
                .is_err()
        );
        assert_eq!(fs::read(&registration).unwrap(), damaged);
        assert_ne!(
            work_item_status_snapshot_with_runtime(&dest, ID, &runtime())
                .unwrap()
                .lifecycle_phase,
            "closed"
        );
    }
}

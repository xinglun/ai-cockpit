use std::process::Command;

#[test]
fn github_plan_help_explains_exact_attempt_and_non_admission() {
    let out = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["evidence", "github", "plan", "--help"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let help = String::from_utf8(out.stdout).unwrap();
    let help = help.split_whitespace().collect::<Vec<_>>().join(" ");
    for text in [
        "--repo",
        "--work-item",
        "--run-id",
        "--attempt",
        "--job-id",
        "--artifact-id",
        "never accepts evidence",
        "latest attempt",
    ] {
        assert!(help.contains(text), "missing public help {text}: {help}");
    }
    for private in [
        "__verification-supervisor",
        "__composition-supervisor",
        "__portable-source-observe",
        "__portable-native-publish",
    ] {
        assert!(!help.contains(private));
    }
}

#[test]
fn github_plan_rejects_zero_selector_and_fake_authority_before_repository_io() {
    for (flag, value) in [
        ("--run-id", "0"),
        ("--provider-authenticated", "true"),
        ("--endpoint", "https://evil.invalid"),
    ] {
        let mut args = vec![
            "evidence",
            "github",
            "plan",
            "--repo",
            "/nonexistent-plan-fixture",
            "--work-item",
            "WI-TEST",
            "--run-id",
            "1",
            "--attempt",
            "1",
            "--job-id",
            "2",
            "--artifact-id",
            "3",
        ];
        if flag == "--run-id" {
            args[8] = value;
        } else {
            args.extend([flag, value]);
        }
        let out = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
        let error = String::from_utf8(out.stderr).unwrap();
        assert!(error.contains(if flag == "--run-id" {
            "invalid value"
        } else {
            "unexpected argument"
        }));
    }
}

#[test]
fn portable_bundle_help_separates_inspection_from_admission() {
    for (action, required, description) in [
        (
            "plan",
            vec!["--repo", "--input", "--work-item"],
            "without execution or writes",
        ),
        (
            "accept",
            vec![
                "--repo",
                "--input",
                "--work-item",
                "--expected-plan-digest",
                "--source-request",
                "--node",
            ],
            "independent source proof",
        ),
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
            .args(["evidence", "bundle", action, "--help"])
            .output()
            .expect("candidate help");
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let help = String::from_utf8(out.stdout)
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        for flag in required {
            assert!(help.contains(flag), "missing {flag}: {help}");
        }
        assert!(help.contains(description), "missing purpose: {help}");
        for private in [
            "__verification-supervisor",
            "__composition-supervisor",
            "__portable-source-observe",
            "__portable-native-publish",
        ] {
            assert!(!help.contains(private));
        }
    }
}

#[test]
fn portable_bundle_rejects_authority_and_command_injection_before_repository_io() {
    for action in ["plan", "accept"] {
        for flag in [
            "--trusted",
            "--authenticated",
            "--source-verifier",
            "--worker-path",
            "--tool-digest",
            "--url",
            "--command",
        ] {
            let mut args = vec![
                "evidence",
                "bundle",
                action,
                "--repo",
                "/nonexistent-portable-fixture",
                "--input",
                "/nonexistent-input",
            ];
            if action == "accept" {
                args.extend([
                    "--work-item",
                    "WI-TEST",
                    "--expected-plan-digest",
                    "sha256:0000000000000000000000000000000000000000000000000000000000000000",
                ]);
            }
            args.extend([flag, "forged"]);
            let out = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
                .args(args)
                .output()
                .unwrap();
            assert_eq!(out.status.code(), Some(2));
            assert!(out.stdout.is_empty());
            assert!(String::from_utf8_lossy(&out.stderr).contains("unexpected argument"));
        }
    }
    for value in [
        "not-a-digest",
        "sha256:abc",
        "sha256:000000000000000000000000000000000000000000000000000000000000000g",
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
            .args([
                "evidence",
                "bundle",
                "accept",
                "--repo",
                "/nonexistent-portable-fixture",
                "--input",
                "/nonexistent-input",
                "--work-item",
                "WI-TEST",
                "--expected-plan-digest",
                value,
            ])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
        assert!(String::from_utf8_lossy(&out.stderr).contains("invalid value"));
    }
}

// Captured schema-2 bytes are only parsing data, never authentication or a new execution.
fn portable_unverified_fixture() -> &'static str {
    r#"{"schemaVersion":1,"nativeInput":{"schemaVersion":1,"nodeId":"project-command-0","producer":"untrusted_captured_fixture","assurance":"self_declared"},"nativeEvidence":{"captureMode":"full_capture","contractDigest":"sha256:40e7ab0b9c454664c3a004ccb362c969eb5897beabfb25b6c8db78bd1c60b6ea","createdAt":"2026-10-31T05:42:18Z","evidenceSchemaVersion":2,"passed":true,"protocolVersion":1,"receipt":{"costObservation":{"advisoryOnly":true,"confidence":"complete","elapsedMs":44080,"executionElapsedMs":43583,"maxConcurrentProcesses":1,"nodesExecuted":1,"nodesPlanned":1,"nodesReused":0,"passed":true,"planningElapsedMs":0,"processesSpawned":1,"protectedNodesExecuted":0,"repositoryId":"sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b","runtimeDigest":"sha256:6f7897c9b46ec160a1fb193f6767ad3f745c86aea522dc16e81a8ab60003d838","runtimeVersion":"1.0.1","schemaVersion":1,"unknowns":[]},"elapsedMs":44080,"executionElapsedMs":43583,"executionRecords":[{"commandDigest":"sha256:dd3968def0dd92b8c1cabb1d5198ce956a5824b3ee296eb33f492523bd40965f","deadlineMs":1793426194838,"elapsedMs":43578,"exitCode":0,"nodeId":"project-command-0","passed":true,"spawned":true,"stderr":"7461736b2d617574686f72697a656420576f726b204974656d20766572696669636174696f6e20726571756972657320616e206578706c69636974202d2d7461736b2d63616c6c65722066696c650a63616e6469646174652052756e74696d6520707265666c69676874206661696c65642077697468206578697420310a6465636c6172656420766572696669636174696f6e2074696d656f7574206d757374206265206120706f7369746976652066696e69746520696e74656765720a6465636c6172656420766572696669636174696f6e2074696d656f7574206d757374206265206120706f7369746976652066696e69746520696e74656765720a6465636c6172656420766572696669636174696f6e2074696d656f7574206d757374206265206120706f7369746976652066696e69746520696e74656765720a6465636c6172656420766572696669636174696f6e2074696d656f7574206d757374206265206120706f7369746976652066696e69746520696e74656765720a6465636c6172656420766572696669636174696f6e2074696d656f7574206d757374206265206120706f7369746976652066696e69746520696e74656765720a63616e6469646174652052756e74696d65207265706f72747320667265736820766572696669636174696f6e2c206275742069747320666f726d616c2072656365697074206973206e6f74207265757361626c6520286578697374696e675f66726573685f726563656970745f6964656e746974795f6d69736d61746368290a776f726b737061636520726563656970742073796e632072656a65637465643a20666f726d616c20766572696669636174696f6e2072656365697074207265706f7369746f7279206964656e7469747920646966666572732066726f6d2052756e74696d65207374617475730a63616e6469646174652052756e74696d6520646f6573206e6f74206163636570742074686520666f726d616c207265636569707420617320667265736820666f72207468652063757272656e7420436f6e747261637420616e6420736f7572636520736e617073686f740a576f726b204974656d20494420697320756e7361666520666f72207468652063616e6f6e6963616c2065766964656e636520706174680a776f726b737061636520726563656970742073796e632072656a65637465643a2063616e6f6e6963616c20776f726b7370616365207265636569707420746172676574206973206e6f74206120726567756c6172206e6f6e2d73796d6c696e6b2066696c650a63616e6469646174652052756e74696d6520646f6573206e6f74206163636570742074686520666f726d616c207265636569707420617320667265736820666f72207468652063757272656e7420436f6e747261637420616e6420736f7572636520736e617073686f740a63616e6469646174652052756e74696d6520646f6573206e6f74206163636570742074686520666f726d616c207265636569707420617320667265736820666f72207468652063757272656e7420436f6e747261637420616e6420736f7572636520736e617073686f740a63616e6469646174652052756e74696d652061646d697473206e6569746865722072756e5f707265666c69676874206e6f722072756e5f766572696669636174696f6e0a63616e6469646174652052756e74696d65206e6f206c6f6e6765722061646d6974732072756e5f766572696669636174696f6e2061667465722073746174757320726566726573680a63616e6469646174652052756e74696d652072657475726e656420616e20696e76616c696420576f726b204974656d2076616c69646174696f6e207265706f72740a63616e6469646174652052756e74696d652072657475726e656420616e20696e76616c696420576f726b204974656d2076616c69646174696f6e207265706f72740a63616e6469646174652052756e74696d652072657475726e656420616e20696e76616c696420576f726b204974656d2076616c69646174696f6e207265706f72740a63616e6469646174652052756e74696d652072657475726e656420616e20696e76616c696420576f726b204974656d2076616c69646174696f6e207265706f72740a63616e6469646174652052756e74696d652072657475726e656420616e20696e76616c696420576f726b204974656d2076616c69646174696f6e207265706f72740a63616e6469646174652052756e74696d652061646d697473206e6569746865722072756e5f707265666c69676874206e6f722072756e5f766572696669636174696f6e0a686f7374656420657865637574696f6e206f75747075742062696e64696e672072656a65637465643a20457870656374696e672070726f7065727479206e616d6520656e636c6f73656420696e20646f75626c652071756f7465733a206c696e65203120636f6c756d6e20322028636861722031290a686f7374656420657865637574696f6e206f75747075742062696e64696e672072656a65637465643a206475706c6963617465207374646f75742f65766964656e6365204a534f4e206669656c640a686f7374656420657865637574696f6e206f75747075742062696e64696e672072656a65637465643a2052756e74696d65207374646f757420646f6573206e6f742070726f6a65637420746f2074686520666f726d616c20726563656970740a686f7374656420657865637574696f6e206f75747075742062696e64696e672072656a65637465643a2052756e74696d65207374646f757420646f6573206e6f742070726f6a65637420746f2074686520666f726d616c20726563656970740a","stderrTruncated":false,"stdout":"776f726b7370616365207061636b61676520636f766572616765207061737365643a2031207061636b616765730a74696d656f75742072656772657373696f6e207061737365643a20626f756e6465643d312c2064656661756c743d312c20756e72656c617465643d312c20696e76616c69643d352c2052756e74696d652d7265667573616c3d310a70726f64756365722f636f6e73756d657220436172676f20636f6e746578742070726f6f66207061737365643a2031322063617365730a686f737465642052756e74696d6520766572696669636174696f6e206f726368657374726174696f6e2072656772657373696f6e207061737365640a","stdoutTruncated":false,"timedOut":false,"timeoutSeconds":900}],"filesHashed":5,"filesRead":5,"gitCalls":6,"maxConcurrentProcesses":1,"nodesExecuted":1,"nodesPlanned":1,"nodesReused":0,"passed":true,"planReceipt":{"affectedPaths":[],"assurance":"repository_verified","baseRevision":"efc9f309f1500e433240a76fe5dc76db64abb53b","dependencyConfidence":"unknown","escalations":[],"executedNodes":["project-command-0"],"executionElapsedMs":43583,"finalTier":"T0","initialTier":"T0","planningElapsedMs":0,"policyRefs":["wi-1114-bounded-release-verification-v1"],"repositoryId":"sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b","repositorySnapshotDigest":"sha256:3ff877b2050551986531641a9573e9ecf332ea3da524b710f6e9a3ee38cb1150","requiredAssurance":"self_declared","requiredTier":"T0","reusedNodes":[],"savedExecutions":0,"schemaVersion":1,"selectionReasons":["effective:ai-cockpit-project-v1:wi-1114-bounded-release-verification-v1: Enforce the approved maximum of 900 seconds per actual verification command with single worker, finite supervised cleanup and at least 1 GiB free disk. Stage applicability does not authorize publication or production use; exact version, tag and assets require separate future owner approval. Keep all platform, source, evidence and approval requirements; the whole multi-command run has a separately declared finite aggregate budget."],"sharedNodes":[],"stage":"task","timeoutSeconds":900,"workItemId":"WI-1114-release-runtime-convergence"},"planningElapsedMs":0,"processSpawnFailures":0,"processesSpawned":1,"protectedNodesExecuted":0,"protectedNodesSkipped":0,"receiptCandidates":[],"repositoryId":"sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b","rerunStale":0,"rerunUnknown":0,"results":[{"action":"execute","nodeId":"project-command-0","outputDigest":"sha256:3ed74a2373b13cd963aa5dc6b029c636f686b556ba635ac4b49d6f7f3d48634e","outputTruncated":false,"passed":true,"protected":false,"reason":"reuse_not_configured","receiptId":null,"reused":false,"satisfiedBy":"execution","state":"not_applicable","timedOut":false}],"runtimeDigest":"sha256:6f7897c9b46ec160a1fb193f6767ad3f745c86aea522dc16e81a8ab60003d838","runtimeVersion":"1.0.1","timeoutSeconds":900,"workItemId":"WI-1114-release-runtime-convergence"},"receiptDigest":"sha256:c82b48c8f1c733a39619b96d2ab77af6379cb329a03c1ba070546742cb3a6ff2","repositoryId":"sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b","repositorySnapshotDigest":"sha256:3ff877b2050551986531641a9573e9ecf332ea3da524b710f6e9a3ee38cb1150","runtimeDigest":"sha256:6f7897c9b46ec160a1fb193f6767ad3f745c86aea522dc16e81a8ab60003d838","runtimeVersion":"1.0.1","workItemId":"WI-1114-release-runtime-convergence"}}"#
}

fn portable_ai_bytes(
    root: &std::path::Path,
) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
    fn walk(
        base: &std::path::Path,
        dir: &std::path::Path,
        bytes: &mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>,
    ) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let metadata = std::fs::symlink_metadata(&path).unwrap();
            assert!(!metadata.file_type().is_symlink());
            if metadata.is_dir() {
                walk(base, &path, bytes);
            } else {
                bytes.insert(
                    path.strip_prefix(base).unwrap().to_owned(),
                    std::fs::read(&path).unwrap(),
                );
            }
        }
    }
    let mut bytes = std::collections::BTreeMap::new();
    walk(root, &root.join(".ai"), &mut bytes);
    bytes
}

#[test]
fn portable_bundle_real_cli_inspects_cold_data_without_work_item_or_grant() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(root)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "--allow-empty",
                "-qm",
                "portable fixture"
            ])
            .current_dir(root)
            .status()
            .unwrap()
            .success()
    );
    cockpit_repository::attach(root).unwrap();
    let input = directory.path().join("untrusted-bundle.json");
    std::fs::write(&input, portable_unverified_fixture()).unwrap();
    let before = portable_ai_bytes(root);
    let out = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["evidence", "bundle", "plan", "--repo"])
        .arg(root)
        .arg("--input")
        .arg(&input)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let plan: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(plan["schemaVersion"], 1);
    assert_eq!(plan["integrityValid"], true);
    assert_eq!(plan["sourceVerification"], "unverified");
    assert_eq!(plan["mayReplaceExecution"], false);
    assert_eq!(plan["workItemId"], serde_json::Value::Null);
    assert_eq!(plan["writesPerformed"], 0);
    assert_eq!(plan["verificationCommandsExecuted"], 0);
    assert!(!plan["gaps"].as_array().unwrap().is_empty());
    assert_eq!(
        plan["inputDigest"],
        cockpit_core::Digest::sha256_bytes(portable_unverified_fixture().as_bytes()).to_string()
    );
    assert_eq!(
        portable_ai_bytes(root),
        before,
        "inspection must preserve native records"
    );
}

// This Linux-only fixture exercises the real candidate and native supervisor.
// Its self-declared owner is local test data, not this WI's grant or provider proof.
#[cfg(target_os = "linux")]
#[test]
fn native_source_helper_real_candidate_publishes_only_its_bounded_native_execution() {
    use cockpit_protocol::{
        AssuranceLevel, RuntimeContext, TaskAuthorizationCaller, TaskAuthorizationPolicy,
    };
    use cockpit_repository::{
        WorkItemStartOptions, attach, checkpoint_work_item_with_caller, confirm_profile_update,
        preflight_work_item_with_runtime_report, record_work_item_governance_controls_with_runtime,
        start_work_item_with_options_and_runtime,
    };
    use cockpit_verification::{
        VerificationCommand, VerificationReceipt, VerificationReusePolicy, execute_bounded,
    };
    use serde_json::{Value, json};
    use sha2::{Digest as _, Sha256};
    use std::io::Read;
    use std::path::{Path, PathBuf};

    fn read_json(path: &Path) -> Value {
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
    }
    fn bounded_git(root: &Path, args: &[&str]) {
        let result = execute_bounded(
            vec![
                VerificationCommand::new(
                    "fixture-git",
                    "git",
                    args.iter().map(|value| (*value).into()).collect(),
                    VerificationReusePolicy::NeverReuse,
                )
                .with_current_dir(root)
                .with_timeout_seconds(20),
            ],
            1,
        )
        .unwrap();
        assert!(result.passed, "fixture Git failed: {result:?}");
        assert_eq!(result.processes_spawned, 1);
        assert_eq!(result.nodes_reused, 0);
    }

    let directory = tempfile::Builder::new()
        .prefix("native-real-cli-repo-")
        .tempdir()
        .unwrap();
    let private = tempfile::Builder::new()
        .prefix("native-real-cli-control-")
        .tempdir()
        .unwrap();
    let root = directory.path().canonicalize().unwrap();
    let private_root = private.path().canonicalize().unwrap();
    assert!(!private_root.starts_with(&root));
    std::fs::write(root.join("tracked.txt"), b"native publisher fixture\n").unwrap();
    for args in [
        vec!["init", "-q", "--initial-branch=main"],
        vec!["config", "core.autocrlf", "false"],
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "fixture",
        ],
    ] {
        bounded_git(&root, &args);
    }
    attach(&root).unwrap();
    // Profile confirmation must precede start so Contract.profileDigest is current.
    confirm_profile_update(&root, "git", &["--version".into()]).unwrap();
    let candidate = PathBuf::from(env!("CARGO_BIN_EXE_ai-cockpit"))
        .canonicalize()
        .unwrap();
    let candidate_info = std::fs::symlink_metadata(&candidate).unwrap();
    assert!(candidate_info.is_file() && !candidate_info.file_type().is_symlink());
    let mut binary = std::fs::File::open(&candidate).unwrap();
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = binary.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let runtime = RuntimeContext {
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        protocol_version: 1,
        runtime_digest: format!("sha256:{}", hex::encode(hasher.finalize()))
            .parse()
            .unwrap(),
    };
    let caller = TaskAuthorizationCaller {
        schema_version: 1,
        actor: "agent:real-cli-native-fixture".into(),
        authority_source: "fixture:synthetic-local-owner-not-this-work-item-grant".into(),
        assurance: AssuranceLevel::SelfDeclared,
    };
    let wi = "WI-REAL-CLI-NATIVE-HELPER";
    start_work_item_with_options_and_runtime(
        &root,
        wi,
        "exercise actual native source publication",
        "do not promote one node to full verification",
        &["tracked.txt".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec![
                "An actual bounded git child supplies its own native observation".into(),
            ],
            verification_commands: vec![
                "git --version".into(),
                "git --definitely-invalid-fixture-option".into(),
            ],
            task_authorization: Some(TaskAuthorizationPolicy {
                schema_version: 1,
                agent_actor: caller.actor.clone(),
                authority_source: caller.authority_source.clone(),
                allowed_actions: vec![
                    "run_preflight".into(),
                    "checkpoint".into(),
                    "run_verification".into(),
                ],
                closeout_actions: vec![],
                expires_at: Some("2099-01-01T00:00:00Z".into()),
                public_target: None,
            }),
            ..Default::default()
        },
        &runtime,
    )
    .unwrap();
    let contract_path = root.join(format!(".ai/work-items/active/{wi}.contract.json"));
    let summary_path = root.join(format!(".ai/work-items/active/{wi}.summary.json"));
    preflight_work_item_with_runtime_report(&root, &contract_path, &runtime).unwrap();
    let contract = read_json(&contract_path);
    let summary = read_json(&summary_path);
    record_work_item_governance_controls_with_runtime(&root, wi, &json!({"decisionEvidence": {
        "schemaVersion": 1, "decisionId": "contract-preflight-review", "decision": "confirm_review",
        "workItemId": wi, "repositoryId": cockpit_repository::repository_id(&root),
        "contractDigest": cockpit_protocol::digest_json(&contract).unwrap(),
        "preflightDecisionDigest": summary["preflightDecisionDigest"],
        "repositorySnapshotDigest": summary["preflightRepositorySnapshotDigest"],
        "recordedAt": summary["updatedAt"], "recordedBy": "agent:test-fixture-recorder",
        "reason": "Synthetic local test confirmation, not external human or GitHub authentication."
    }}), &runtime).unwrap();
    checkpoint_work_item_with_caller(&root, wi, &runtime, Some(&caller)).unwrap();
    let before_summary = std::fs::read(&summary_path).unwrap();
    let outcome_path = root.join(format!(".ai/work-items/active/{wi}.outcome.json"));
    let before_outcome = match std::fs::read(&outcome_path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => panic!("read fixture Outcome: {error}"),
    };
    let formal_path = root.join(format!(".ai/evidence/{wi}.verification.json"));
    assert!(!formal_path.exists());
    let caller_path = private_root.join("caller.json");
    std::fs::write(&caller_path, serde_json::to_vec(&caller).unwrap()).unwrap();
    let runner = private_root.join("runner");
    std::fs::create_dir(&runner).unwrap();
    let output = private_root.join("github-output.txt");
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)
        .unwrap();
    let helper = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/ci/run_hosted_runtime_verification.sh")
        .canonicalize()
        .unwrap();
    let result = execute_bounded(
        vec![
            VerificationCommand::new(
                "actual-native-source-helper",
                "bash",
                vec![
                    helper.to_string_lossy().into_owned(),
                    "--native-source".into(),
                    candidate.to_string_lossy().into_owned(),
                    root.to_string_lossy().into_owned(),
                    contract_path.to_string_lossy().into_owned(),
                    runner.to_string_lossy().into_owned(),
                    "--node-id".into(),
                    "project-command-0".into(),
                    "--task-caller".into(),
                    caller_path.to_string_lossy().into_owned(),
                ],
                VerificationReusePolicy::NeverReuse,
            )
            .with_current_dir(&root)
            .with_timeout_seconds(90)
            .with_environment(vec![
                ("RUNNER_TEMP".into(), runner.as_os_str().to_owned()),
                ("GITHUB_OUTPUT".into(), output.as_os_str().to_owned()),
            ]),
        ],
        1,
    )
    .unwrap();
    assert!(result.passed, "real native helper: {result:?}");
    assert_eq!(
        (
            result.processes_spawned,
            result.nodes_executed,
            result.nodes_reused
        ),
        (1, 1, 0)
    );
    assert_eq!(result.execution_records.len(), 1);
    let execution = &result.execution_records[0];
    assert!(
        execution.spawned
            && execution.passed
            && !execution.timed_out
            && !execution.stdout_truncated
            && !execution.stderr_truncated
    );
    assert_eq!(execution.exit_code, Some(0));
    assert_eq!(execution.termination_signal, None);
    let stdout = String::from_utf8(hex::decode(&execution.stdout_hex).unwrap()).unwrap();
    assert!(stdout.ends_with('\n'));
    let lines = stdout.lines().collect::<Vec<_>>();
    assert_eq!(
        lines.len(),
        1,
        "exactly one helper machine record: {stdout}"
    );
    let encoded = lines[0]
        .strip_prefix("AI_COCKPIT_NATIVE_SOURCE_V1 ")
        .expect("native source machine prefix");
    assert!(!encoded.is_empty() && encoded.len() <= 8192 && encoded.len() % 2 == 0);
    assert!(
        encoded
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );
    let machine: Value = serde_json::from_slice(&hex::decode(encoded).unwrap()).unwrap();
    let mut keys = machine
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "inputDigest",
            "nativeRoot",
            "publisherNonce",
            "runtimeDigest",
            "schemaVersion"
        ]
    );
    assert_eq!(
        hex::encode(
            serde_json::to_vec(
                &machine
                    .as_object()
                    .unwrap()
                    .iter()
                    .collect::<std::collections::BTreeMap<_, _>>()
            )
            .unwrap()
        ),
        encoded,
        "machine line must be canonical and contain no repeated JSON keys"
    );
    assert_eq!(machine["schemaVersion"], 1);
    let nonce = machine["publisherNonce"].as_str().unwrap();
    assert_eq!(nonce.len(), 64);
    assert!(
        nonce
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );
    assert_eq!(machine["nativeRoot"], machine["inputDigest"]);
    assert_eq!(machine["runtimeDigest"], runtime.runtime_digest.to_string());
    let github_output = std::fs::read_to_string(&output).unwrap();
    let outputs = github_output.lines().collect::<Vec<_>>();
    assert_eq!(outputs.len(), 2);
    assert_eq!(outputs[0], format!("native_record_hex={encoded}"));
    let portable_path = PathBuf::from(outputs[1].strip_prefix("native_portable_path=").unwrap());
    assert_eq!(
        portable_path.file_name().unwrap(),
        "portable-native-evidence.json"
    );
    assert!(portable_path.starts_with(&runner));
    let metadata = std::fs::symlink_metadata(&portable_path).unwrap();
    assert!(metadata.is_file() && !metadata.file_type().is_symlink());
    let bytes = std::fs::read(&portable_path).unwrap();
    let input_digest = cockpit_core::Digest::sha256_bytes(&bytes);
    assert_eq!(machine["inputDigest"], input_digest.to_string());
    let native_path = root.join(format!(
        ".ai/evidence/{wi}.verification-admissions/portable/objects/{}.bin",
        &input_digest.as_str()[7..]
    ));
    assert_eq!(std::fs::read(&native_path).unwrap(), bytes);
    let packet: cockpit_protocol::ci_admission::PortableExecutionEvidence<
        cockpit_protocol::ci_admission::NativeExecutionEnvelope<VerificationReceipt>,
    > = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        packet.native_evidence.runtime_digest,
        runtime.runtime_digest
    );
    assert_eq!(packet.native_evidence.work_item_id, wi);
    assert_eq!(packet.native_input["program"], "git");
    assert_eq!(packet.native_input["args"], json!(["--version"]));
    assert_eq!(packet.native_input["nodeId"], "project-command-0");
    let receipt = &packet.native_evidence.receipt;
    assert!(receipt.passed);
    assert_eq!(
        (
            receipt.nodes_executed,
            receipt.processes_spawned,
            receipt.nodes_reused
        ),
        (1, 1, 0)
    );
    assert_eq!(receipt.execution_records.len(), 1);
    assert_eq!(receipt.results.len(), 1);
    assert!(!receipt.results[0].reused);
    let native_execution = &receipt.execution_records[0];
    assert!(
        native_execution.spawned
            && native_execution.passed
            && !native_execution.timed_out
            && !native_execution.stdout_truncated
            && !native_execution.stderr_truncated
    );
    assert_eq!(native_execution.exit_code, Some(0));
    assert_eq!(native_execution.termination_signal, None);
    assert!(
        String::from_utf8(hex::decode(&native_execution.stdout_hex).unwrap())
            .unwrap()
            .starts_with("git version ")
    );
    let execution_id = packet.native_input["producerExecutionId"].as_str().unwrap();
    execution_id.parse::<cockpit_core::Digest>().unwrap();
    assert!(
        std::fs::read_dir(root.join(".ai/evidence"))
            .unwrap()
            .filter_map(Result::ok)
            .any(|entry| {
                let path = entry.path();
                path.extension()
                    .is_some_and(|extension| extension == "json")
                    && {
                        let observation = read_json(&path);
                        observation["kind"] == "verification_execution_checkpoint"
                            && observation["state"] == "node_execution_completed"
                            && observation["executionId"] == execution_id
                            && observation["receipt"]["executionRecords"][0]
                                == serde_json::to_value(native_execution).unwrap()
                    }
            }),
        "published execution ID must identify the real completed native checkpoint"
    );
    assert_eq!(std::fs::read(&summary_path).unwrap(), before_summary);
    assert!(
        !formal_path.exists(),
        "partial native source publication must not record formal verification"
    );
    let after_outcome = match std::fs::read(&outcome_path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => panic!("read final fixture Outcome: {error}"),
    };
    assert_eq!(after_outcome, before_outcome);
    // No artifact or hosted identity is fabricated: this proves only the local native bridge.
}

//! Legacy target name: generic untrusted bundle inspection and retired CLI
//! regressions. GitHub collection, admission, source workers and publishers
//! are not supported.

use std::process::Command;

fn assert_parser_rejection(args: &[&str], diagnostic: &str) {
    let out = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(args)
        .output()
        .expect("run actual CLI process");
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    let error = String::from_utf8(out.stderr).unwrap();
    assert!(
        error.contains(diagnostic),
        "missing {diagnostic:?}: {error}"
    );
}

#[test]
fn retired_subcommands_are_unknown_to_the_actual_cli_process() {
    for (args, name) in [
        (vec!["evidence", "github", "plan"], "github"),
        (vec!["evidence", "bundle", "accept"], "accept"),
        (
            vec!["__portable-source-observe", "{}"],
            "__portable-source-observe",
        ),
        (
            vec!["__portable-native-publish", "{}"],
            "__portable-native-publish",
        ),
    ] {
        assert_parser_rejection(&args, &format!("unrecognized subcommand '{name}'"));
    }
}

#[test]
fn retired_flags_are_unknown_before_repository_io() {
    for (args, name) in [
        (
            vec![
                "verify",
                "--repo",
                "/nonexistent-retired-fixture",
                "--work-item",
                "WI-TEST",
                "--source-bundle",
                "/nonexistent-input",
            ],
            "--source-bundle",
        ),
        (
            vec![
                "profile",
                "confirm",
                "--repo",
                "/nonexistent-retired-fixture",
                "--program",
                "git",
                "--execution-recipe",
                "/nonexistent-input",
            ],
            "--execution-recipe",
        ),
    ] {
        assert_parser_rejection(&args, &format!("unexpected argument '{name}'"));
    }
}

#[test]
fn portable_bundle_plan_help_describes_untrusted_read_only_inspection() {
    let out = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["evidence", "bundle", "plan", "--help"])
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
    for text in [
        "--repo",
        "--input",
        "--work-item",
        "without execution or writes",
        "unverified",
    ] {
        assert!(help.contains(text), "missing {text}: {help}");
    }
    for retired in [
        "--expected-plan-digest",
        "--source-request",
        "--node",
        "accept",
        "__portable-source-observe",
        "__portable-native-publish",
    ] {
        assert!(!help.contains(retired), "retired {retired}: {help}");
    }
}

#[test]
fn portable_bundle_plan_rejects_authority_and_command_injection_before_repository_io() {
    for flag in [
        "--trusted",
        "--authenticated",
        "--source-verifier",
        "--worker-path",
        "--tool-digest",
        "--url",
        "--command",
        "--source-request",
        "--expected-plan-digest",
        "--node",
    ] {
        assert_parser_rejection(
            &[
                "evidence",
                "bundle",
                "plan",
                "--repo",
                "/nonexistent-portable-fixture",
                "--input",
                "/nonexistent-input",
                flag,
                "forged",
            ],
            &format!("unexpected argument '{flag}'"),
        );
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

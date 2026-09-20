use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

#[allow(dead_code)]
mod common;

static NEXT_REPOSITORY_ID: AtomicU64 = AtomicU64::new(0);

#[test]
fn verify_executes_an_explicit_never_reuse_command_with_bounded_telemetry() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "cockpit-verify-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["verify", "--repo"])
        .arg(&directory)
        .args(["--command", "true"])
        .output()
        .expect("verify");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON");
    assert_eq!(json["nodesPlanned"], 1);
    assert_eq!(json["nodesExecuted"], 1);
    assert_eq!(json["nodesReused"], 0);
    assert_eq!(json["rerunStale"], 0);
    assert_eq!(json["rerunUnknown"], 0);
    assert_eq!(json["protectedNodesExecuted"], 0);
    assert_eq!(json["protectedNodesSkipped"], 0);
    assert!(json["planningElapsedMs"].is_u64());
    assert!(json["executionElapsedMs"].is_u64());
    assert_eq!(json["processesSpawned"], 1);
    assert_eq!(json["processSpawnFailures"], 0);
    assert_eq!(json["results"][0]["nodeId"], "project-command-0");
    assert_eq!(json["results"][0]["protected"], false);
    assert_eq!(json["results"][0]["action"], "execute");
    assert_eq!(json["results"][0]["satisfiedBy"], "execution");
    assert_eq!(json["passed"], true);
    assert_eq!(json["runtimeVersion"], env!("CARGO_PKG_VERSION"));
    assert!(
        json["runtimeDigest"]
            .as_str()
            .is_some_and(|value| value.starts_with("sha256:"))
    );
    assert_eq!(json["planReceipt"]["stage"], "task");
    assert_eq!(json["timeoutSeconds"], 300);
    assert_eq!(json["planReceipt"]["timeoutSeconds"], 300);
    assert_eq!(json["executionRecords"][0]["timeoutSeconds"], 300);
    assert_eq!(json["planReceipt"]["initialTier"], "T0");
    assert_eq!(json["planReceipt"]["assurance"], "self_declared");
    assert_eq!(json["costObservation"]["confidence"], "complete");
    assert_eq!(json["costObservation"]["nodesExecuted"], 1);
    assert!(
        json["repositoryId"]
            .as_str()
            .is_some_and(|value| value.starts_with("sha256:"))
    );
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn verify_cli_accepts_policy_authorized_timeout_and_rejects_the_runtime_cap_before_spawn() {
    let directory = std::env::temp_dir().join(format!(
        "cockpit-verify-timeout-{}-{}",
        std::process::id(),
        NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(directory.join(".ai")).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    fs::write(
        directory.join(".ai/policy.json"),
        r#"{
          "schemaVersion": 1,
          "project": {
            "policyId": "project-timeout-cli",
            "layer": "project",
            "rules": [{
              "operation": "modify_source",
              "approvalMode": "no_human_approval_for_low_risk",
              "requiredEvidence": [],
              "verificationRequirement": {
                "schemaVersion": 1,
                "requiredTier": "T0",
                "requiredAssurance": "self_declared",
                "policyRefs": ["project-timeout-cli"],
                "stageRefs": ["task"],
                "gateRefs": [],
                "reason": "authorize CLI timeout regression",
                "maxTimeoutSeconds": 1
              }
            }]
          }
        }"#,
    )
    .expect("policy");
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let authorized = Command::new(binary)
        .args(["verify", "--repo"])
        .arg(&directory)
        .args(["--command", "true", "--timeout-seconds", "1"])
        .output()
        .expect("authorized verify");
    assert!(
        authorized.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&authorized.stderr)
    );
    let receipt: serde_json::Value = serde_json::from_slice(&authorized.stdout).expect("JSON");
    assert_eq!(receipt["timeoutSeconds"], 1);
    assert_eq!(receipt["planReceipt"]["timeoutSeconds"], 1);
    assert_eq!(receipt["executionRecords"][0]["timeoutSeconds"], 1);

    let rejected = Command::new(binary)
        .args(["verify", "--repo"])
        .arg(&directory)
        .args(["--command", "true", "--timeout-seconds", "901"])
        .output()
        .expect("cap rejection");
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("finite range 1..=900s"));
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn work_item_verification_persists_strict_receipt_without_cli_plan_projection() {
    let directory = std::env::temp_dir().join(format!(
        "cockpit-verify-typed-receipt-{}-{}",
        std::process::id(),
        NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&directory).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let run = |args: &[&str]| {
        let output = Command::new(binary)
            .args(args)
            .args(["--repo"])
            .arg(&directory)
            .output()
            .expect("run ai-cockpit");
        assert!(
            output.status.success(),
            "args={args:?}, stdout={}, stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        output
    };
    run(&[
        "start",
        "--id",
        "WI-CLI-TYPED-RECEIPT",
        "--intent",
        "persist a strict Runtime verification receipt",
        "--goal",
        "keep the CLI planning projection out of persisted receipt identity",
        "--scope",
        "README.md",
        "--authority",
        "authorized",
        "--acceptance",
        "A1: strict typed verification receipt is persisted",
        "--required-evidence",
        "verification",
    ]);
    fs::write(directory.join("README.md"), "typed receipt fixture\n").expect("README");
    run(&[
        "preflight",
        "--contract",
        ".ai/work-items/active/WI-CLI-TYPED-RECEIPT.contract.json",
    ]);
    run(&["checkpoint", "--id", "WI-CLI-TYPED-RECEIPT"]);
    let controls = tempfile::NamedTempFile::new().expect("controls input");
    fs::write(
        controls.path(),
        serde_json::to_vec_pretty(&serde_json::json!({
            "acceptanceEvidence": [{
                "acceptanceId": "A1",
                "evidence": [{
                    "type": "test",
                    "path": "README.md",
                    "locator": "typed receipt fixture",
                    "verification": "passed"
                }]
            }],
            "intentAlignment": {
                "state": "resolved",
                "evidence": ["README.md"]
            }
        }))
        .expect("controls JSON"),
    )
    .expect("write controls");
    run(&[
        "work-item",
        "controls",
        "--id",
        "WI-CLI-TYPED-RECEIPT",
        "--input",
        controls.path().to_str().expect("controls path"),
    ]);

    let verification = run(&[
        "verify",
        "--work-item",
        "WI-CLI-TYPED-RECEIPT",
        "--command",
        "true",
    ]);
    let response: serde_json::Value =
        serde_json::from_slice(&verification.stdout).expect("verification response JSON");
    assert_eq!(response["plannedNodes"].as_array().map(Vec::len), Some(1));
    let evidence: serde_json::Value = serde_json::from_slice(
        &fs::read(directory.join(".ai/evidence/WI-CLI-TYPED-RECEIPT.verification.json"))
            .expect("persisted verification evidence"),
    )
    .expect("persisted verification evidence JSON");
    let receipt = evidence.get("receipt").expect("typed receipt field");
    assert!(receipt.get("plannedNodes").is_none());
    let _: cockpit_verification::VerificationReceipt =
        serde_json::from_value(receipt.clone()).expect("strict typed verification receipt");
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn verify_workspace_route_emits_coverage_manifest_and_execution_records() {
    let directory = std::env::temp_dir().join(format!(
        "cockpit-verify-workspace-{}-{}",
        std::process::id(),
        NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(directory.join("src")).expect("directory");
    fs::write(
        directory.join("Cargo.toml"),
        "[package]\nname = \"verify-workspace-fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("manifest");
    fs::write(directory.join("src/lib.rs"), "pub fn fixture() {}\n").expect("source");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["verify", "--repo"])
        .arg(&directory)
        .output()
        .expect("verify");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON");
    assert_eq!(
        json["planReceipt"]["coverageManifest"]["workspaceMembers"],
        serde_json::json!(["verify-workspace-fixture"])
    );
    assert_eq!(json["executionRecords"].as_array().map(Vec::len), Some(1));
    assert_eq!(json["executionRecords"][0]["exitCode"], 0);
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn verify_plan_only_does_not_spawn_project_commands() {
    let directory = std::env::temp_dir().join(format!(
        "cockpit-verify-plan-only-{}-{}",
        std::process::id(),
        NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(directory.join("src")).expect("directory");
    fs::write(
        directory.join("Cargo.toml"),
        "[package]\nname = \"verify-plan-only-fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("manifest");
    fs::write(directory.join("src/lib.rs"), "pub fn fixture() {}\n").expect("source");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["verify", "--repo"])
        .arg(&directory)
        .args(["--plan-only"])
        .output()
        .expect("plan");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON");
    assert_eq!(json["requests"].as_array().map(Vec::len), Some(1));
    assert_eq!(json["requests"][0]["args"][0], "test");
    assert_eq!(json["requests"][0]["args"][1], "--package");
    assert_eq!(json["coverageManifest"]["planningProcessesSpawned"], 1);
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn work_item_plan_includes_each_declared_verification_command() {
    let directory = std::env::temp_dir().join(format!(
        "cockpit-verify-declared-commands-{}-{}",
        std::process::id(),
        NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(directory.join("src")).expect("directory");
    fs::write(
        directory.join("Cargo.toml"),
        "[package]\nname = \"verify-declared-commands-fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("manifest");
    fs::write(directory.join("src/lib.rs"), "pub fn fixture() {}\n").expect("source");
    fs::write(directory.join("docs-gate.sh"), "#!/bin/sh\nexit 0\n").expect("gate");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    assert!(
        Command::new("git")
            .args(["add", "Cargo.toml", "docs-gate.sh", "src/lib.rs"])
            .current_dir(&directory)
            .status()
            .expect("git add")
            .success()
    );
    assert!(
        Command::new("cargo")
            .arg("generate-lockfile")
            .current_dir(&directory)
            .status()
            .expect("generate lockfile")
            .success()
    );
    assert!(
        Command::new("git")
            .args(["add", "Cargo.lock"])
            .current_dir(&directory)
            .status()
            .expect("add lockfile")
            .success()
    );
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=AI Cockpit Test",
                "-c",
                "user.email=ai-cockpit@example.invalid",
                "commit",
                "-qm",
                "fixture",
            ])
            .current_dir(&directory)
            .status()
            .expect("git commit")
            .success()
    );
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    assert!(
        Command::new(binary)
            .args(["attach", "--repo"])
            .arg(&directory)
            .status()
            .expect("attach")
            .success()
    );
    assert!(
        Command::new(binary)
            .args(["profile", "confirm", "--repo"])
            .arg(&directory)
            .args(["--program", "cargo", "--args", "test,--locked,--workspace",])
            .status()
            .expect("confirm Cargo profile")
            .success()
    );
    assert!(
        Command::new(binary)
            .args(["start", "--repo"])
            .arg(&directory)
            .args([
                "--id",
                "WI-DECLARED-VERIFY",
                "--intent",
                "plan declared verification commands",
                "--goal",
                "avoid silently dropping a required documentation gate",
                "--scope",
                "docs-gate.sh",
                "--authority",
                "authorized",
            ])
            .status()
            .expect("start")
            .success()
    );
    let contract_path = directory.join(".ai/work-items/active/WI-DECLARED-VERIFY.contract.json");
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract"))
            .expect("contract JSON");
    contract["verification"] =
        serde_json::json!(["cargo test --locked --workspace", "bash docs-gate.sh"]);
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract).expect("contract bytes"),
    )
    .expect("update fixture Contract");
    assert!(
        Command::new(binary)
            .args(["preflight", "--repo"])
            .arg(&directory)
            .args(["--contract"])
            .arg(&contract_path)
            .status()
            .expect("preflight")
            .success()
    );
    assert!(
        Command::new(binary)
            .args(["checkpoint", "--repo"])
            .arg(&directory)
            .args(["--id", "WI-DECLARED-VERIFY"])
            .status()
            .expect("checkpoint")
            .success()
    );
    let output = Command::new(binary)
        .args(["verify", "--repo"])
        .arg(&directory)
        .args(["--work-item", "WI-DECLARED-VERIFY", "--plan-only"])
        .output()
        .expect("plan declared verification");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_slice(&output.stdout).expect("plan JSON");
    let requests = plan["requests"].as_array().expect("planned requests");
    assert!(
        requests.iter().any(|request| {
            request["program"] == "cargo"
                && request["args"]
                    .as_array()
                    .is_some_and(|args| args.iter().any(|argument| argument == "--package"))
        }),
        "Cargo workspace declaration should remain package-partitioned: {plan:#}"
    );
    assert!(
        requests.iter().any(|request| {
            request["program"] == "bash" && request["args"] == serde_json::json!(["docs-gate.sh"])
        }),
        "declared shell gate must not be dropped: {plan:#}"
    );
    assert_eq!(plan["processesSpawned"], 0);
    let verification = Command::new(binary)
        .args(["verify", "--repo"])
        .arg(&directory)
        .args(["--work-item", "WI-DECLARED-VERIFY"])
        .output()
        .expect("verify declared commands");
    assert!(
        verification.status.success(),
        "stdout={}, stderr={}",
        String::from_utf8_lossy(&verification.stdout),
        String::from_utf8_lossy(&verification.stderr)
    );
    let receipt: serde_json::Value =
        serde_json::from_slice(&verification.stdout).expect("verification JSON");
    assert_eq!(receipt["nodesPlanned"], 2);
    assert!(
        directory
            .join(".ai/evidence/WI-DECLARED-VERIFY.verification.json")
            .is_file()
    );
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn verify_returns_nonzero_and_structured_receipt_when_command_fails() {
    let directory = std::env::temp_dir().join(format!(
        "cockpit-verify-failure-{}-{}",
        std::process::id(),
        NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&directory).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["verify", "--repo"])
        .arg(&directory)
        .args(["--command", "false"])
        .output()
        .expect("verify");
    assert!(!output.status.success());
    let receipt: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("failed verification must remain machine-readable on stdout");
    assert_eq!(receipt["passed"], false);
    assert_eq!(receipt["results"][0]["passed"], false);
    assert_eq!(receipt["results"][0]["nodeId"], "project-command-0");
    assert!(
        !String::from_utf8_lossy(&output.stderr)
            .contains("failed verification cannot be recorded as completion evidence")
    );
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn verify_rejects_an_unknown_typed_stage() {
    let directory = std::env::temp_dir().join(format!(
        "cockpit-verify-stage-{}-{}",
        std::process::id(),
        NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&directory).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["verify", "--repo"])
        .arg(&directory)
        .args(["--command", "true", "--stage", "ci"])
        .output()
        .expect("verify");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unsupported verification stage"));
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn verify_cli_rejects_missing_intent_before_starting_the_command() {
    let directory = std::env::temp_dir().join(format!(
        "cockpit-verify-route-{}-{}",
        std::process::id(),
        NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&directory).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    for args in [
        vec![
            "attach".into(),
            "--repo".into(),
            directory.display().to_string(),
        ],
        vec![
            "start".into(),
            "--repo".into(),
            directory.display().to_string(),
            "--id".into(),
            "WI-CLI-ROUTE".into(),
            "--intent".into(),
            "route intent".into(),
            "--goal".into(),
            "route goal".into(),
            "--scope".into(),
            "**".into(),
            "--authority".into(),
            "authorized".into(),
            "--acceptance".into(),
            "route is enforced".into(),
        ],
    ] {
        let output = Command::new(binary)
            .args(args.iter().map(String::as_str))
            .output()
            .expect("lifecycle command");
        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    fs::write(
        directory.join(".ai/policy.json"),
        r#"{
          "schemaVersion": 1,
          "organization": {
            "policyId": "cli-route-v1",
            "layer": "organization",
            "rules": [{
              "operation": "modify_source",
              "approvalMode": "no_human_approval_for_low_risk",
              "requiredEvidence": [],
              "verificationRequirement": {
                "schemaVersion": 1,
                "requiredTier": "T0",
                "requiredAssurance": "repository_verified",
                "policyRefs": ["cli-route-v1"],
                "stageRefs": ["task"],
                "gateRefs": [],
                "reason": "route test"
              }
            }]
          }
        }"#,
    )
    .expect("policy");
    let contract_path = directory.join(".ai/work-items/active/WI-CLI-ROUTE.contract.json");
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract")).expect("JSON");
    contract["intent"] = serde_json::json!("");
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract).expect("contract bytes"),
    )
    .expect("contract mutation");
    let output = Command::new(binary)
        .args([
            "verify",
            "--repo",
            directory.to_str().expect("repo path"),
            "--work-item",
            "WI-CLI-ROUTE",
            "--stage",
            "task",
            "--command",
            "true",
        ])
        .output()
        .expect("verify");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("intent/scenario verification route"),
        "{stderr}"
    );
    assert!(
        !directory
            .join(".ai/evidence/WI-CLI-ROUTE.verification.json")
            .exists()
    );
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn verify_preserves_multiple_explicit_command_compatibility() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "cockpit-verify-multiple-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");

    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["verify", "--repo"])
        .arg(&directory)
        .args(["--command", "true", "--command", "true"])
        .output()
        .expect("verify");

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON");
    assert_eq!(json["nodesPlanned"], 2);
    assert_eq!(json["nodesExecuted"], 2);
    assert_eq!(json["processesSpawned"], 2);
    assert_eq!(json["results"].as_array().expect("results").len(), 2);
    fs::remove_dir_all(directory).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn workers_bound_parallel_execution_of_multiple_explicit_commands() {
    use std::os::unix::fs::PermissionsExt;

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "cockpit-verify-parallel-multiple-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    let commands = [directory.join("first.sh"), directory.join("second.sh")];
    let markers = [
        directory.join("first.started"),
        directory.join("second.started"),
    ];
    for (index, command) in commands.iter().enumerate() {
        let other = 1 - index;
        fs::write(
            command,
            format!(
                "#!/bin/sh\ntouch '{}'\nn=0\nwhile ! test -f '{}'; do sleep 0.05; n=$((n+1)); test $n -lt 100 || exit 7; done\nsleep 1\n",
                markers[index].display(),
                markers[other].display()
            ),
        )
        .expect("script");
        fs::set_permissions(command, fs::Permissions::from_mode(0o755)).expect("executable");
    }

    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["verify", "--repo"])
        .arg(&directory)
        .arg("--command")
        .arg(&commands[0])
        .arg("--command")
        .arg(&commands[1])
        .args(["--workers", "2"])
        .output()
        .expect("verify");

    assert!(output.status.success());
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn calibrated_auto_command_reuses_a_persisted_receipt_in_a_second_process() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "cockpit-verify-reuse-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("directory");
    fs::write(
        directory.join("package.json"),
        r#"{"scripts":{"test":"node verify.js"}}"#,
    )
    .expect("package");
    fs::write(
        directory.join("verify.js"),
        "const fs=require('fs'); const p='.verify-count'; const n=fs.existsSync(p)?+fs.readFileSync(p):0; fs.writeFileSync(p,String(n+1));\n",
    )
    .expect("script");
    fs::write(directory.join(".gitignore"), ".verify-count\n").expect("ignore counter");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    Command::new("git")
        .args(["add", "."])
        .current_dir(&directory)
        .status()
        .expect("git add");
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=AI Cockpit Test",
                "-c",
                "user.email=ai-cockpit@example.invalid",
                "commit",
                "-qm",
                "fixture",
            ])
            .current_dir(&directory)
            .status()
            .expect("git commit")
            .success()
    );
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    assert!(
        Command::new(binary)
            .args(["attach", "--repo"])
            .arg(&directory)
            .status()
            .expect("attach")
            .success()
    );
    assert!(
        Command::new(binary)
            .args(["profile", "confirm", "--repo"])
            .arg(&directory)
            .args(["--program", "npm", "--args", "test"])
            .status()
            .expect("confirm profile")
            .success()
    );
    let run = || {
        let output = Command::new(binary)
            .args(["verify", "--repo"])
            .arg(&directory)
            .env("AI_COCKPIT_DEBUG_SPAWN", "1")
            .output()
            .expect("verify");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let value = serde_json::from_slice::<serde_json::Value>(&output.stdout).expect("JSON");
        if value["processSpawnFailures"]
            .as_u64()
            .is_some_and(|count| count > 0)
        {
            eprintln!(
                "verification spawn diagnostics: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        value
    };

    let first = run();

    assert_eq!(first["nodesExecuted"], 1);
    assert_eq!(
        first["processesSpawned"], 1,
        "first verification: {first:#}"
    );
    assert_eq!(first["nodesReused"], 0);
    let planned = Command::new(binary)
        .args(["verify", "--repo"])
        .arg(&directory)
        .args(["--plan-only"])
        .env("AI_COCKPIT_DEBUG_SPAWN", "1")
        .output()
        .expect("plan reusable verification");
    assert!(
        planned.status.success(),
        "{}",
        String::from_utf8_lossy(&planned.stderr)
    );
    let plan: serde_json::Value = serde_json::from_slice(&planned.stdout).expect("plan JSON");
    let second = run();
    assert_eq!(
        plan["plannedNodes"][0]["nodeId"],
        first["results"][0]["nodeId"]
    );
    assert_eq!(
        plan["plannedNodes"][0]["action"], second["results"][0]["action"],
        "plan: {plan:#}; execution: {second:#}"
    );
    assert_eq!(
        plan["plannedNodes"][0]["state"],
        second["results"][0]["state"]
    );
    assert_eq!(
        plan["plannedNodes"][0]["reason"], second["results"][0]["reason"],
        "plan: {plan:#}; execution: {second:#}"
    );
    assert!(
        plan["plannedNodes"][0]["identityBinding"]["commandDigest"]
            .as_str()
            .is_some_and(|value| value.starts_with("sha256:"))
    );
    assert_eq!(plan["processesSpawned"], 0);
    assert_eq!(second["nodesExecuted"], 0);
    assert_eq!(second["processesSpawned"], 0);
    assert_eq!(second["nodesReused"], 1);
    assert_eq!(first["gitCalls"], second["gitCalls"]);
    assert_eq!(first["filesHashed"], second["filesHashed"]);
    assert_eq!(
        second["filesRead"].as_u64(),
        first["filesRead"].as_u64().map(|count| count + 2)
    );
    assert_eq!(
        fs::read_to_string(directory.join(".verify-count")).expect("counter"),
        "1"
    );
    assert_eq!(
        first["results"][0]["receiptId"],
        second["results"][0]["receiptId"]
    );
    assert!(
        Command::new(binary)
            .args(["profile", "confirm", "--repo"])
            .arg(&directory)
            .args(["--program", "npm", "--args", "test"])
            .status()
            .expect("reconfirm profile")
            .success()
    );
    let third = run();
    let fourth = run();
    assert_eq!(third["processesSpawned"], 1);
    assert_eq!(third["nodesReused"], 0);
    assert_eq!(fourth["processesSpawned"], 0);
    assert_eq!(fourth["nodesReused"], 1);
    assert_ne!(
        second["results"][0]["receiptId"],
        third["results"][0]["receiptId"]
    );
    assert_eq!(
        fs::read_to_string(directory.join(".verify-count")).expect("counter"),
        "2"
    );
    let changed_environment = Command::new(binary)
        .args(["verify", "--repo"])
        .arg(&directory)
        .env("AI_COCKPIT_WI33_ENV_MUTATION", "changed")
        .output()
        .expect("verify changed environment");
    assert!(changed_environment.status.success());
    let changed_environment: serde_json::Value =
        serde_json::from_slice(&changed_environment.stdout).expect("JSON");
    assert_eq!(changed_environment["processesSpawned"], 1);
    assert_eq!(changed_environment["nodesReused"], 0);
    assert_eq!(
        fs::read_to_string(directory.join(".verify-count")).expect("counter"),
        "3"
    );
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn work_item_detected_command_uses_dynamic_profile_authorized_reuse() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "cockpit-verify-work-item-reuse-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("directory");
    fs::write(
        directory.join("package.json"),
        r#"{"scripts":{"test":"node verify.js"}}"#,
    )
    .expect("package");
    fs::write(
        directory.join("verify.js"),
        "const fs=require('fs'); const p='.verify-count'; const n=fs.existsSync(p)?+fs.readFileSync(p):0; fs.writeFileSync(p,String(n+1));\n",
    )
    .expect("script");
    fs::write(directory.join(".gitignore"), ".verify-count\n").expect("ignore counter");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    Command::new("git")
        .args(["add", "."])
        .current_dir(&directory)
        .status()
        .expect("git add");
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=AI Cockpit Test",
                "-c",
                "user.email=ai-cockpit@example.invalid",
                "commit",
                "-qm",
                "fixture",
            ])
            .current_dir(&directory)
            .status()
            .expect("git commit")
            .success()
    );
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    for args in [
        vec!["attach", "--repo"],
        vec!["profile", "confirm", "--repo"],
    ] {
        let mut command = Command::new(binary);
        command.args(&args).arg(&directory);
        if args[0] == "profile" {
            command.args(["--program", "npm", "--args", "test"]);
        }
        assert!(
            command
                .current_dir(&directory)
                .output()
                .expect("setup")
                .status
                .success()
        );
    }
    let start = Command::new(binary)
        .args([
            "start",
            "--repo",
            directory.to_str().expect("repo path"),
            "--id",
            "WI-VERIFY-REUSE",
            "--intent",
            "verify reuse",
            "--goal",
            "reuse exact evidence",
            "--scope",
            "package.json",
            "--authority",
            "authorized",
            "--acceptance",
            "unchanged verification reuses exact evidence",
        ])
        .current_dir(&directory)
        .output()
        .expect("start");
    assert!(
        start.status.success(),
        "start stderr: {}",
        String::from_utf8_lossy(&start.stderr)
    );
    let contract_path = directory.join(".ai/work-items/active/WI-VERIFY-REUSE.contract.json");
    assert!(
        Command::new(binary)
            .args(["preflight", "--repo"])
            .arg(&directory)
            .args(["--contract"])
            .arg(&contract_path)
            .current_dir(&directory)
            .output()
            .expect("preflight")
            .status
            .success()
    );
    assert!(
        Command::new(binary)
            .args(["checkpoint", "--repo"])
            .arg(&directory)
            .args(["--id", "WI-VERIFY-REUSE"])
            .current_dir(&directory)
            .output()
            .expect("checkpoint")
            .status
            .success()
    );
    let run = || {
        let output = Command::new(binary)
            .args([
                "verify",
                "--repo",
                directory.to_str().expect("repo path"),
                "--work-item",
                "WI-VERIFY-REUSE",
            ])
            .current_dir(&directory)
            .output()
            .expect("verify");
        assert!(
            output.status.success(),
            "verify stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice::<serde_json::Value>(&output.stdout).expect("JSON")
    };
    let first = run();
    let second = run();
    assert_eq!(first["nodesExecuted"], 1);
    assert_eq!(first["nodesReused"], 0);
    assert_eq!(second["nodesExecuted"], 0);
    assert_eq!(second["nodesReused"], 1);
    assert_eq!(
        fs::read_to_string(directory.join(".verify-count")).expect("counter"),
        "1"
    );
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn verification_evidence_uses_snapshot_after_command_side_effects() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "cockpit-verify-side-effect-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(directory.join("src")).expect("directory");
    fs::write(directory.join(".gitignore"), "target/\n").expect("gitignore");
    fs::write(
        directory.join("Cargo.toml"),
        "[package]\nname = \"side-effect-fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("manifest");
    fs::write(directory.join("src/main.rs"), "fn main() {}\n").expect("source");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    for (key, value) in [
        ("user.email", "test@example.invalid"),
        ("user.name", "Test"),
    ] {
        assert!(
            Command::new("git")
                .args(["config", key, value])
                .current_dir(&directory)
                .status()
                .expect("git config")
                .success()
        );
    }
    assert!(
        Command::new("git")
            .args(["add", "."])
            .current_dir(&directory)
            .status()
            .expect("git add")
            .success()
    );
    assert!(
        Command::new("git")
            .args(["commit", "-qm", "baseline"])
            .current_dir(&directory)
            .status()
            .expect("git commit")
            .success()
    );
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    assert!(
        Command::new(binary)
            .args(["attach", "--repo"])
            .arg(&directory)
            .status()
            .expect("attach")
            .success()
    );
    assert!(
        Command::new(binary)
            .args(["start", "--repo"])
            .arg(&directory)
            .args([
                "--id",
                "WI-SIDE-EFFECT",
                "--intent",
                "verify",
                "--goal",
                "bind after command",
                "--scope",
                "Cargo.lock",
                "--authority",
                "authorized",
                "--required-evidence",
                "verification",
            ])
            .status()
            .expect("start")
            .success()
    );
    common::plan(binary, &directory, "WI-SIDE-EFFECT");
    assert!(
        Command::new(binary)
            .args(["preflight", "--repo"])
            .arg(&directory)
            .args([
                "--contract",
                ".ai/work-items/active/WI-SIDE-EFFECT.contract.json"
            ])
            .status()
            .expect("preflight")
            .success()
    );
    assert!(
        Command::new(binary)
            .args(["checkpoint", "--repo"])
            .arg(&directory)
            .args(["--id", "WI-SIDE-EFFECT"])
            .status()
            .expect("checkpoint")
            .success()
    );
    assert!(
        Command::new(binary)
            .args(["verify", "--repo"])
            .arg(&directory)
            .args([
                "--work-item",
                "WI-SIDE-EFFECT",
                "--command",
                "cargo",
                "--args",
                "check",
            ])
            .status()
            .expect("verify")
            .success()
    );
    let evidence: serde_json::Value = serde_json::from_slice(
        &fs::read(directory.join(".ai/evidence/WI-SIDE-EFFECT.verification.json"))
            .expect("verification evidence"),
    )
    .expect("verification evidence JSON");
    let expected_runtime_digest = cockpit_core::Digest::sha256_bytes(
        &fs::read(binary).expect("read exact executable under test"),
    )
    .to_string();
    assert_eq!(evidence["runtimeVersion"], env!("CARGO_PKG_VERSION"));
    assert_eq!(evidence["runtimeDigest"], expected_runtime_digest);
    let finish = Command::new(binary)
        .args(["finish", "--repo"])
        .arg(&directory)
        .args(["--id", "WI-SIDE-EFFECT"])
        .output()
        .expect("finish");
    assert!(
        finish.status.success(),
        "finish should accept post-command snapshot: {}",
        String::from_utf8_lossy(&finish.stderr)
    );
    assert!(directory.join("Cargo.lock").is_file());
    fs::remove_dir_all(directory).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn multi_command_evidence_uses_one_snapshot_after_all_workers_finish() {
    use std::os::unix::fs::PermissionsExt;

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "cockpit-verify-final-snapshot-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("directory");
    fs::write(directory.join("tracked.txt"), "before\n").expect("tracked");
    let slow = directory.join("slow.sh");
    let fast = directory.join("fast.sh");
    fs::write(
        &slow,
        "#!/bin/sh\nsleep 1\nprintf 'after\\n' > tracked.txt\n",
    )
    .expect("slow");
    fs::write(&fast, "#!/bin/sh\nexit 0\n").expect("fast");
    fs::set_permissions(&slow, fs::Permissions::from_mode(0o755)).expect("slow executable");
    fs::set_permissions(&fast, fs::Permissions::from_mode(0o755)).expect("fast executable");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    Command::new("git")
        .args(["add", "."])
        .current_dir(&directory)
        .status()
        .expect("git add");
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=AI Cockpit Test",
                "-c",
                "user.email=ai-cockpit@example.invalid",
                "commit",
                "-qm",
                "fixture",
            ])
            .current_dir(&directory)
            .status()
            .expect("git commit")
            .success()
    );
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    assert!(
        Command::new(binary)
            .args(["attach", "--repo"])
            .arg(&directory)
            .status()
            .expect("attach")
            .success()
    );
    assert!(
        Command::new(binary)
            .args(["start", "--repo"])
            .arg(&directory)
            .args([
                "--id",
                "WI-FINAL-SNAPSHOT",
                "--intent",
                "verify",
                "--goal",
                "bind final snapshot",
                "--scope",
                "tracked.txt",
                "--authority",
                "authorized",
                "--required-evidence",
                "verification",
            ])
            .status()
            .expect("start")
            .success()
    );
    common::plan(binary, &directory, "WI-FINAL-SNAPSHOT");
    assert!(
        Command::new(binary)
            .args(["preflight", "--repo"])
            .arg(&directory)
            .args([
                "--contract",
                ".ai/work-items/active/WI-FINAL-SNAPSHOT.contract.json"
            ])
            .status()
            .expect("preflight")
            .success()
    );
    assert!(
        Command::new(binary)
            .args(["checkpoint", "--repo"])
            .arg(&directory)
            .args(["--id", "WI-FINAL-SNAPSHOT"])
            .status()
            .expect("checkpoint")
            .success()
    );
    let verify = Command::new(binary)
        .args(["verify", "--repo"])
        .arg(&directory)
        .args(["--work-item", "WI-FINAL-SNAPSHOT", "--command"])
        .arg(&slow)
        .arg("--command")
        .arg(&fast)
        .args(["--workers", "2"])
        .output()
        .expect("verify");
    assert!(
        verify.status.success(),
        "{}",
        String::from_utf8_lossy(&verify.stderr)
    );

    let finish = Command::new(binary)
        .args(["finish", "--repo"])
        .arg(&directory)
        .args(["--id", "WI-FINAL-SNAPSHOT"])
        .output()
        .expect("finish");
    assert!(
        finish.status.success(),
        "evidence must bind the snapshot after the slow worker: {}",
        String::from_utf8_lossy(&finish.stderr)
    );
    fs::remove_dir_all(directory).expect("cleanup");
}

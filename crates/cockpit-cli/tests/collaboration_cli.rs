use cockpit_core::Digest;
use cockpit_git::GitRepository;
use cockpit_protocol::{
    COLLABORATION_CAPABILITY, CollaborationDeclaration, CoordinationEvent, CoordinationEventKind,
    IntegrationResponsibility, OutcomeStage, ProvidedOutcome, RuntimeCapabilityBinding,
    WorktreeRegistration,
};
use serde_json::Value;
use sha2::{Digest as ShaDigest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use cockpit_repository::{
    WorkItemStartOptions, attach, repository_id, start_work_item_with_options,
};

fn run_git(root: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .expect("git command")
            .success()
    );
}

fn repository() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("tempdir");
    run_git(root.path(), &["init", "-q"]);
    run_git(
        root.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    run_git(root.path(), &["config", "user.name", "Test"]);
    fs::write(root.path().join("README.md"), "initial\n").expect("write");
    run_git(root.path(), &["add", "."]);
    run_git(root.path(), &["commit", "-qm", "initial"]);
    root
}

fn candidate_runtime() -> RuntimeCapabilityBinding {
    let executable = PathBuf::from(env!("CARGO_BIN_EXE_ai-cockpit"));
    let bytes = fs::read(executable).expect("read candidate executable");
    let digest = format!("sha256:{:x}", Sha256::digest(bytes))
        .parse::<Digest>()
        .expect("runtime digest");
    RuntimeCapabilityBinding {
        schema_version: 1,
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        runtime_digest: digest,
        capability: COLLABORATION_CAPABILITY.into(),
    }
}

fn registration(root: &Path) -> WorktreeRegistration {
    let topology = GitRepository::discover(root)
        .expect("discover repository")
        .topology()
        .expect("repository topology");
    let contract_path = root.join(".ai/work-items/active/WI-CLI.contract.json");
    let contract: Value = serde_json::from_slice(&fs::read(&contract_path).expect("contract"))
        .expect("contract JSON");
    WorktreeRegistration {
        schema_version: 1,
        repository_id: repository_id(root),
        work_item_id: "WI-CLI".into(),
        contract_digest: cockpit_protocol::digest_json(&contract).expect("contract digest"),
        worktree_path: topology.repository_root.to_string_lossy().into_owned(),
        branch: topology.branch.expect("branch"),
        head: topology.head.expect("head"),
        generation: 1,
        declaration: CollaborationDeclaration {
            integration_responsibility: IntegrationResponsibility {
                responsible_work_item_id: "WI-CLI".into(),
                target_branch: "main".into(),
                composition_order: vec!["WI-CLI".into()],
                rationale: "CLI test".into(),
            },
            ..Default::default()
        },
        runtime: candidate_runtime(),
    }
}

fn invoke(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(args)
        .output()
        .expect("candidate CLI")
}

fn coordination_snapshot(root: &Path) -> Option<BTreeMap<PathBuf, Vec<u8>>> {
    let directory = root.join(".git/.ai-cockpit/coordination");
    if !directory.exists() {
        return None;
    }

    fn collect_files(directory: &Path, current: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(current).expect("read coordination directory") {
            let entry = entry.expect("coordination entry");
            let path = entry.path();
            let kind = entry.file_type().expect("coordination entry type");
            assert!(!kind.is_symlink(), "coordination store contains a symlink");
            if kind.is_dir() {
                collect_files(directory, &path, files);
            } else {
                assert!(kind.is_file(), "unexpected coordination entry: {path:?}");
                files.insert(
                    path.strip_prefix(directory)
                        .expect("entry beneath coordination directory")
                        .to_owned(),
                    fs::read(path).expect("coordination record bytes"),
                );
            }
        }
    }

    let mut files = BTreeMap::new();
    collect_files(&directory, &directory, &mut files);
    Some(files)
}

fn invoke_mcp(root: &Path, name: &str, arguments: Value) -> Value {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": name, "arguments": arguments}
    });
    let mut child = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["mcp", "--repo"])
        .arg(root)
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("candidate MCP stdio server");
    let mut stdin = child.stdin.take().expect("MCP stdin");
    let mut request_bytes = serde_json::to_vec(&request).expect("MCP request JSON");
    request_bytes.push(b'\n');
    stdin.write_all(&request_bytes).expect("write MCP request");
    drop(stdin);

    let output = child.wait_with_output().expect("MCP response");
    assert!(
        output.status.success(),
        "MCP stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let response: Value = serde_json::from_slice(&output.stdout).expect("MCP response JSON");
    assert_ne!(
        response["result"]["isError"], true,
        "MCP response: {response}"
    );
    response
}

#[test]
fn coordination_cli_and_mcp_queries_preserve_store_bytes_across_processes() {
    let root = repository();
    run_git(root.path(), &["checkout", "-qb", "codex/wi-cli"]);
    attach(root.path()).expect("attach repository");
    start_work_item_with_options(
        root.path(),
        "WI-CLI",
        "CLI coordination test",
        "bind registration to observed facts",
        &[".ai/**".into(), "README.md".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["registration is fact-bound".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start Work Item");

    assert_eq!(coordination_snapshot(root.path()), None);
    let output = invoke(&[
        "work-item",
        "coordination",
        "inspect",
        "--repo",
        root.path().to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let first: Value = serde_json::from_slice(&output.stdout).expect("projection JSON");
    assert_eq!(coordination_snapshot(root.path()), None);

    let repo_path = root.path().to_str().unwrap();
    for (label, args) in [
        (
            "CLI status",
            vec![
                "work-item",
                "status",
                "--repo",
                repo_path,
                "--id",
                "WI-CLI",
                "--json",
            ],
        ),
        (
            "CLI Outcome",
            vec![
                "work-item",
                "outcome",
                "--repo",
                repo_path,
                "--id",
                "WI-CLI",
                "--json",
            ],
        ),
    ] {
        let output = invoke(&args);
        assert!(
            output.status.success(),
            "{label}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let _: Value = serde_json::from_slice(&output.stdout).expect("projection JSON");
        assert_eq!(
            coordination_snapshot(root.path()),
            None,
            "{label} created a store"
        );
    }

    for (name, arguments) in [
        (
            "work_item_coordination",
            serde_json::json!({"action":"inspect"}),
        ),
        (
            "work_item_status",
            serde_json::json!({"workItemId":"WI-CLI"}),
        ),
        (
            "work_item_outcome",
            serde_json::json!({"workItemId":"WI-CLI"}),
        ),
    ] {
        let response = invoke_mcp(root.path(), name, arguments);
        assert!(
            !response["result"]["structuredContent"].is_null(),
            "{name} returned no structured projection: {response}"
        );
        assert_eq!(
            coordination_snapshot(root.path()),
            None,
            "{name} created a store"
        );
    }

    let input = root.path().join("registration.json");
    fs::write(
        &input,
        serde_json::to_vec_pretty(&registration(root.path())).unwrap(),
    )
    .unwrap();
    let output = invoke(&[
        "work-item",
        "coordination",
        "register",
        "--repo",
        root.path().to_str().unwrap(),
        "--input",
        input.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let written: Value = serde_json::from_slice(&output.stdout).expect("write result JSON");
    assert!(written.get("projection").is_some());
    let registered_snapshot = coordination_snapshot(root.path())
        .expect("explicit registration creates a durable coordination store");

    let output = invoke(&["work-item", "coordination", "inspect", "--repo", repo_path]);
    assert!(output.status.success());
    let second: Value = serde_json::from_slice(&output.stdout).expect("projection JSON");
    assert_eq!(first["events"], second["events"]);
    assert_eq!(second["registrations"].as_array().unwrap().len(), 1);
    assert_eq!(
        coordination_snapshot(root.path()),
        Some(registered_snapshot.clone())
    );

    for (label, args) in [
        (
            "CLI status",
            vec![
                "work-item",
                "status",
                "--repo",
                repo_path,
                "--id",
                "WI-CLI",
                "--json",
            ],
        ),
        (
            "CLI Outcome",
            vec![
                "work-item",
                "outcome",
                "--repo",
                repo_path,
                "--id",
                "WI-CLI",
                "--json",
            ],
        ),
    ] {
        let output = invoke(&args);
        assert!(
            output.status.success(),
            "{label}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let _: Value = serde_json::from_slice(&output.stdout).expect("projection JSON");
        assert_eq!(
            coordination_snapshot(root.path()),
            Some(registered_snapshot.clone()),
            "{label} rewrote, consumed, or appended to the coordination store"
        );
    }

    for (name, arguments) in [
        (
            "work_item_coordination",
            serde_json::json!({"action":"inspect"}),
        ),
        (
            "work_item_status",
            serde_json::json!({"workItemId":"WI-CLI"}),
        ),
        (
            "work_item_outcome",
            serde_json::json!({"workItemId":"WI-CLI"}),
        ),
    ] {
        let response = invoke_mcp(root.path(), name, arguments);
        assert!(
            !response["result"]["structuredContent"].is_null(),
            "{name} returned no structured projection: {response}"
        );
        assert_eq!(
            coordination_snapshot(root.path()),
            Some(registered_snapshot.clone()),
            "{name} rewrote, consumed, or appended to the coordination store"
        );
    }

    let event = CoordinationEvent {
        schema_version: 1,
        event_id: "impact-WI-CLI-1".into(),
        repository_id: repository_id(root.path()),
        work_item_id: "WI-CLI".into(),
        generation: 1,
        kind: CoordinationEventKind::Impact,
        source: "explicit CLI impact report".into(),
        evidence_refs: Vec::new(),
        evidence_digests: Default::default(),
        outcome_ids: Vec::new(),
    };
    let event_path = root.path().join("impact.json");
    fs::write(&event_path, serde_json::to_vec_pretty(&event).unwrap()).unwrap();
    let output = invoke(&[
        "work-item",
        "coordination",
        "report-impact",
        "--repo",
        repo_path,
        "--input",
        event_path.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let reported_snapshot = coordination_snapshot(root.path()).expect("reported event store");
    assert_ne!(reported_snapshot, registered_snapshot);

    let fresh_process_projection = invoke_mcp(
        root.path(),
        "work_item_coordination",
        serde_json::json!({"action":"inspect"}),
    )["result"]["structuredContent"]
        .clone();
    assert_eq!(
        fresh_process_projection["events"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        fresh_process_projection["events"][0]["eventId"],
        "impact-WI-CLI-1"
    );
    assert_eq!(coordination_snapshot(root.path()), Some(reported_snapshot));
}

#[test]
fn publish_outcome_cli_binds_selected_generation_to_exact_evidence_bytes() {
    let root = repository();
    run_git(root.path(), &["checkout", "-qb", "codex/wi-cli-publish"]);
    attach(root.path()).expect("attach repository");
    start_work_item_with_options(
        root.path(),
        "WI-CLI",
        "CLI outcome publication test",
        "bind a published outcome to observed evidence bytes",
        &[
            ".ai/**".into(),
            "README.md".into(),
            "target/api.json".into(),
        ],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["publication binds exact evidence bytes".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start Work Item");

    let evidence_reference = "target/api.json";
    let evidence_bytes = b"{\"api\":1}\n";
    fs::create_dir_all(root.path().join("target")).expect("target directory");
    fs::write(root.path().join(evidence_reference), evidence_bytes).expect("write evidence");
    let mut registration = registration(root.path());
    registration
        .declaration
        .provided_outcomes
        .push(ProvidedOutcome {
            outcome_id: "api".into(),
            interface_contract: "api-v1".into(),
            behavior_contract: "stable response".into(),
            published_head: registration.head.clone(),
            stage: OutcomeStage::ComposableHead,
            evidence_refs: vec![evidence_reference.into()],
        });
    let input = root.path().join("registration.json");
    fs::write(&input, serde_json::to_vec_pretty(&registration).unwrap()).unwrap();
    let registered = invoke(&[
        "work-item",
        "coordination",
        "register",
        "--repo",
        root.path().to_str().unwrap(),
        "--input",
        input.to_str().unwrap(),
    ]);
    assert!(
        registered.status.success(),
        "{}",
        String::from_utf8_lossy(&registered.stderr)
    );

    let published = invoke(&[
        "work-item",
        "coordination",
        "publish-outcome",
        "--repo",
        root.path().to_str().unwrap(),
        "--id",
        "WI-CLI",
        "--generation",
        "1",
        "--outcome-id",
        "api",
    ]);
    assert!(
        published.status.success(),
        "{}",
        String::from_utf8_lossy(&published.stderr)
    );
    let response: Value = serde_json::from_slice(&published.stdout).expect("publication JSON");
    assert_eq!(response["result"]["kind"], "outcome_published");
    assert_eq!(response["result"]["outcomeIds"], serde_json::json!(["api"]));
    assert_eq!(
        response["result"]["evidenceDigests"][evidence_reference],
        Digest::sha256_bytes(evidence_bytes).to_string()
    );
}

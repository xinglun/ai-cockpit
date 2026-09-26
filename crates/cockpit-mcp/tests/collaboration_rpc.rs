use cockpit_core::Digest;
use cockpit_git::GitRepository;
use cockpit_protocol::{
    COLLABORATION_CAPABILITY, CollaborationDeclaration, IntegrationResponsibility, OutcomeStage,
    PROTOCOL_VERSION, ProvidedOutcome, RuntimeCapabilityBinding, RuntimeContext,
    WorktreeRegistration,
};
use cockpit_repository::{
    WorkItemStartOptions, attach, repository_id, start_work_item_with_options,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "0.2.113".into(),
        protocol_version: PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(b"candidate-mcp"),
    }
}

fn outcome_registration(root: &Path) -> WorktreeRegistration {
    let topology = GitRepository::discover(root)
        .expect("discover repository")
        .topology()
        .expect("repository topology");
    let contract_path = root.join(".ai/work-items/active/WI-MCP.contract.json");
    let contract: Value =
        serde_json::from_slice(&fs::read(contract_path).expect("contract")).expect("contract JSON");
    WorktreeRegistration {
        schema_version: 1,
        repository_id: repository_id(root),
        work_item_id: "WI-MCP".into(),
        contract_digest: cockpit_protocol::digest_json(&contract).expect("contract digest"),
        worktree_path: topology.repository_root.to_string_lossy().into_owned(),
        branch: topology.branch.expect("branch"),
        head: topology.head.clone().expect("head"),
        generation: 1,
        declaration: CollaborationDeclaration {
            provided_outcomes: vec![ProvidedOutcome {
                outcome_id: "api".into(),
                interface_contract: "api-v1".into(),
                behavior_contract: "stable response".into(),
                published_head: topology.head.expect("head"),
                stage: OutcomeStage::ComposableHead,
                evidence_refs: vec!["target/api.json".into()],
            }],
            integration_responsibility: IntegrationResponsibility {
                responsible_work_item_id: "WI-MCP".into(),
                target_branch: "main".into(),
                composition_order: vec!["WI-MCP".into()],
                rationale: "MCP outcome publication test".into(),
            },
            ..Default::default()
        },
        runtime: RuntimeCapabilityBinding {
            schema_version: 1,
            runtime_version: runtime().runtime_version,
            runtime_digest: runtime().runtime_digest,
            capability: COLLABORATION_CAPABILITY.into(),
        },
    }
}

fn call(root: &Path, name: &str, arguments: Value) -> Value {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": name, "arguments": arguments}
    });
    let response = cockpit_mcp::handle_request_for_repo(&request, root, &runtime());
    response["result"]["structuredContent"].clone()
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

#[test]
fn coordination_rpc_exposes_explicit_read_write_operations_and_stable_projection() {
    let root = repository();
    let tools = cockpit_mcp::handle_request(
        &json!({"jsonrpc":"2.0", "id":1, "method":"tools/list"}),
        &runtime(),
    );
    let names = tools["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect::<Vec<_>>();
    assert!(names.contains(&"work_item_coordination"));
    assert!(names.contains(&"work_item_composition"));

    let first = call(
        root.path(),
        "work_item_coordination",
        json!({"action":"inspect"}),
    );
    assert_eq!(coordination_snapshot(root.path()), None);
    let second = call(
        root.path(),
        "work_item_coordination",
        json!({"action":"inspect"}),
    );
    assert_eq!(first, second);
    assert_eq!(coordination_snapshot(root.path()), None);

    let error = cockpit_mcp::handle_request_for_repo(
        &json!({
            "jsonrpc":"2.0", "id":2, "method":"tools/call",
            "params":{"name":"work_item_coordination", "arguments":{"action":"inspect", "event":{}}}
        }),
        root.path(),
        &runtime(),
    );
    assert_eq!(error["result"]["isError"], true);
    assert_eq!(coordination_snapshot(root.path()), None);
}

#[test]
fn coordination_rpc_projections_preserve_exact_persisted_store_bytes() {
    let root = repository();
    run_git(root.path(), &["branch", "-M", "main"]);
    run_git(root.path(), &["checkout", "-qb", "codex/wi-mcp-readonly"]);
    attach(root.path()).expect("attach repository");
    start_work_item_with_options(
        root.path(),
        "WI-MCP",
        "MCP read-only projection test",
        "keep projection queries separate from durable coordination writes",
        &[
            ".ai/**".into(),
            "README.md".into(),
            "target/api.json".into(),
        ],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["read routes preserve coordination bytes".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start Work Item");

    for (name, arguments) in [
        ("work_item_coordination", json!({"action":"inspect"})),
        ("work_item_status", json!({"workItemId":"WI-MCP"})),
        ("work_item_outcome", json!({"workItemId":"WI-MCP"})),
    ] {
        let response = call(root.path(), name, arguments);
        assert!(!response.is_null(), "{name} returned no projection");
        assert_eq!(
            coordination_snapshot(root.path()),
            None,
            "{name} created a store"
        );
    }

    fs::create_dir_all(root.path().join("target")).expect("target directory");
    fs::write(root.path().join("target/api.json"), b"{\"api\":1}\n").expect("outcome evidence");
    let write_result = call(
        root.path(),
        "work_item_coordination",
        json!({
            "action":"register",
            "registration":outcome_registration(root.path())
        }),
    );
    assert_eq!(
        write_result["result"]["workItemId"], "WI-MCP",
        "{write_result}"
    );
    let persisted = coordination_snapshot(root.path())
        .expect("explicit registration creates persisted coordination records");

    for (name, arguments) in [
        ("work_item_coordination", json!({"action":"inspect"})),
        ("work_item_status", json!({"workItemId":"WI-MCP"})),
        ("work_item_outcome", json!({"workItemId":"WI-MCP"})),
    ] {
        let response = call(root.path(), name, arguments);
        assert!(!response.is_null(), "{name} returned no projection");
        assert_eq!(
            coordination_snapshot(root.path()),
            Some(persisted.clone()),
            "{name} rewrote, consumed, or appended to the coordination store"
        );
    }
}

#[test]
fn coordination_schema_binds_provider_consumer_and_resume_identities_per_action() {
    let tools = cockpit_mcp::handle_request(
        &json!({"jsonrpc":"2.0", "id":1, "method":"tools/list"}),
        &runtime(),
    );
    let schema = &tools["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["name"] == "work_item_coordination")
        .expect("coordination tool")["inputSchema"];
    let properties = schema["properties"].as_object().expect("properties");

    assert_eq!(
        properties["providerWorkItemId"]["description"],
        "Provider Work Item whose declared outcome is being published."
    );
    assert_eq!(
        properties["providerGeneration"]["description"],
        "Current registration generation of providerWorkItemId."
    );
    assert_eq!(
        properties["workItemId"]["description"],
        "Work Item being resumed; the legacy publish-outcome alias names its provider."
    );
    assert_eq!(
        properties["consumerWorkItemId"]["description"],
        "Consumer Work Item identity for recovery consumption."
    );
    assert_eq!(
        properties.len(),
        properties
            .keys()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        "serialized MCP property names are unique"
    );
    assert!(schema["oneOf"].as_array().is_some_and(|variants| {
        variants.iter().any(|variant| {
            variant["properties"]["action"]["const"] == "recover"
                && variant["required"]
                    == json!([
                        "action",
                        "eventId",
                        "consumerWorkItemId",
                        "consumerGeneration"
                    ])
        })
    }));
    assert!(schema["oneOf"].as_array().is_some_and(|variants| {
        variants.iter().any(|variant| {
            variant["properties"]["action"]["const"] == "publish-outcome"
                && variant["required"]
                    == json!([
                        "action",
                        "providerWorkItemId",
                        "providerGeneration",
                        "outcomeId"
                    ])
        })
    }));
    assert!(schema["oneOf"].as_array().is_some_and(|variants| {
        variants.iter().any(|variant| {
            variant["properties"]["action"]["const"] == "resume"
                && variant["required"] == json!(["action", "workItemId", "generation"])
        })
    }));
}

#[test]
fn coordination_rejects_mixed_provider_aliases_before_opening_the_store() {
    let root = repository();
    let response = cockpit_mcp::handle_request_for_repo(
        &json!({
            "jsonrpc":"2.0", "id":5, "method":"tools/call",
            "params":{"name":"work_item_coordination", "arguments":{
                "action":"publish-outcome",
                "providerWorkItemId":"WI-PROVIDER",
                "providerGeneration":1,
                "workItemId":"WI-OTHER",
                "generation":1,
                "outcomeId":"api"
            }}
        }),
        root.path(),
        &runtime(),
    );
    assert_eq!(response["result"]["isError"], true);
    assert!(
        response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("identity contract")
    );
    assert!(!root.path().join(".git/.ai-cockpit/coordination").exists());
}

#[test]
fn publish_outcome_mcp_action_uses_the_same_bound_write_path() {
    let root = repository();
    run_git(root.path(), &["branch", "-M", "main"]);
    run_git(root.path(), &["checkout", "-qb", "codex/wi-mcp"]);
    attach(root.path()).expect("attach repository");
    start_work_item_with_options(
        root.path(),
        "WI-MCP",
        "MCP outcome publication test",
        "bind an outcome publication to its selected evidence",
        &[
            ".ai/**".into(),
            "README.md".into(),
            "target/api.json".into(),
        ],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["MCP publication binds exact evidence bytes".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start Work Item");
    fs::create_dir_all(root.path().join("target")).expect("target directory");
    let evidence_bytes = b"{\"api\":1}\n";
    fs::write(root.path().join("target/api.json"), evidence_bytes).expect("write evidence");

    let registered = call(
        root.path(),
        "work_item_coordination",
        json!({"action":"register", "registration":outcome_registration(root.path())}),
    );
    assert_eq!(registered["result"]["workItemId"], "WI-MCP");

    let response = cockpit_mcp::handle_request_for_repo(
        &json!({
            "jsonrpc":"2.0", "id":3, "method":"tools/call",
            "params":{"name":"work_item_coordination", "arguments":{
                "action":"publish-outcome", "providerWorkItemId":"WI-MCP", "providerGeneration":1, "outcomeId":"api"
            }}
        }),
        root.path(),
        &runtime(),
    );
    assert_ne!(response["result"]["isError"], true, "{response}");
    let published = &response["result"]["structuredContent"]["result"];
    assert_eq!(published["kind"], "outcome_published");
    assert_eq!(published["outcomeIds"], json!(["api"]));
    assert_eq!(
        published["evidenceDigests"]["target/api.json"],
        Digest::sha256_bytes(evidence_bytes).to_string()
    );

    let legacy = cockpit_mcp::handle_request_for_repo(
        &json!({
            "jsonrpc":"2.0", "id":4, "method":"tools/call",
            "params":{"name":"work_item_coordination", "arguments":{
                "action":"publish-outcome", "workItemId":"WI-MCP", "generation":1, "outcomeId":"api"
            }}
        }),
        root.path(),
        &runtime(),
    );
    assert_ne!(
        legacy["result"]["isError"], true,
        "legacy provider alias: {legacy}"
    );
}

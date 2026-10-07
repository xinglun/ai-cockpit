use cockpit_core::Digest;
use cockpit_protocol::RuntimeContext;
use serde_json::json;
use std::{fs, path::Path, process::Command};

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        protocol_version: cockpit_protocol::PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(b"material review MCP runtime"),
    }
}

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("run git fixture command");
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn plan_fixture() -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    git(root, &["init", "-q"]);
    fs::write(root.join("README.md"), "baseline\n").unwrap();
    git(root, &["add", "."]);
    git(
        root,
        &[
            "-c",
            "user.name=fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "base",
        ],
    );
    let base = git(root, &["rev-parse", "HEAD"]);
    let contract = json!({
        "protocolVersion": 1,
        "repositoryId": cockpit_repository::repository_id(root).to_string(),
        "workItemId": "WI-MATERIAL-MCP",
        "intent": "review bounded committed material",
        "goal": "retain exact material evidence",
        "scope": ["README.md"],
        "outOfScope": [],
        "risk": "high",
        "authority": "authorized",
        "acceptanceCriteria": ["exact material request"],
        "requiredEvidenceClasses": [],
        "verification": ["true"],
        "baseRevision": base,
        "projectProfileDigest": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "repositorySnapshotDigest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"
    });
    let active = root.join(".ai/work-items/active");
    fs::create_dir_all(&active).unwrap();
    fs::write(
        active.join("WI-MATERIAL-MCP.contract.json"),
        serde_json::to_vec_pretty(&contract).unwrap(),
    )
    .unwrap();
    directory
}

#[test]
fn material_review_plan_tool_uses_shared_read_only_repository_service() {
    let directory = plan_fixture();
    let initialized = cockpit_mcp::handle_request(
        &json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{}}),
        &runtime(),
    );
    assert_eq!(initialized["result"]["serverInfo"]["name"], "ai-cockpit");
    let listed = cockpit_mcp::handle_request(
        &json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}),
        &runtime(),
    );
    let tools = listed["result"]["tools"].as_array().expect("MCP tools");
    assert_eq!(tools.len(), 31);
    let plan_tool = tools
        .iter()
        .find(|tool| tool["name"] == "work_item_material_review_plan")
        .expect("material review plan tool");
    assert_eq!(plan_tool["inputSchema"]["required"], json!(["workItemId"]));

    let response = cockpit_mcp::handle_request_for_repo(
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{
            "name":"work_item_material_review_plan",
            "arguments":{"workItemId":"WI-MATERIAL-MCP"}
        }}),
        directory.path(),
        &runtime(),
    );
    assert_eq!(response["result"]["isError"], false, "{response}");
    let request = &response["result"]["structuredContent"];
    assert_eq!(request["workItemId"], "WI-MATERIAL-MCP");
    assert_eq!(request["reviewEnabled"], false);
    assert_eq!(request["reviewDiagnostic"], "material_review_not_enabled");
    assert_eq!(git(directory.path(), &["status", "--porcelain"]), "?? .ai/");
}

#[test]
fn material_review_record_tool_fails_closed_without_stage_one_opt_in() {
    let directory = plan_fixture();
    let listed = cockpit_mcp::handle_request(
        &json!({"jsonrpc":"2.0","id":4,"method":"tools/list","params":{}}),
        &runtime(),
    );
    let tools = listed["result"]["tools"].as_array().expect("MCP tools");
    let tool = tools
        .iter()
        .find(|tool| tool["name"] == "work_item_material_review_record")
        .expect("material review record tool");
    assert_eq!(
        tool["inputSchema"]["required"],
        json!(["workItemId", "decision"])
    );
    assert_eq!(
        tool["inputSchema"]["properties"]["decision"]["additionalProperties"],
        false
    );
    let response = cockpit_mcp::handle_request_for_repo(
        &json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{
            "name":"work_item_material_review_record",
            "arguments":{
                "workItemId":"WI-MATERIAL-MCP",
                "decision":{
                    "schemaVersion":1,
                    "decision":"accept_permitted_unknowns",
                    "requestDigest":"sha256:0000000000000000000000000000000000000000000000000000000000000000",
                    "reviewerActor":"agent:Raydot",
                    "authoritySource":"user-delegation:ray-approved-WI1068",
                    "assurance":"self_declared",
                    "evidenceRefs":[{"path":"docs/review-evidence.md","digest":"sha256:0000000000000000000000000000000000000000000000000000000000000000"}],
                    "rationale":"Review the exact bounded material request.",
                    "residualRisk":"The scanner remains incomplete for permitted syntax."
                }
            }
        }}),
        directory.path(),
        &runtime(),
    );
    assert_eq!(response["result"]["isError"], true, "{response}");
    let serialized = serde_json::to_string(&response).unwrap();
    assert!(serialized.contains("not enabled"), "{response}");
    assert!(
        !directory
            .path()
            .join(".ai/evidence/material-inspection-review")
            .exists()
    );
}

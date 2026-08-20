use serde_json::{Value, json};
use std::io::{self, BufRead, Write};
use std::path::Path;

const TOOL_NAMES: [&str; 10] = [
    "status",
    "work_item_get",
    "work_item_list",
    "blockers",
    "safe_actions",
    "knowledge_query",
    "evidence_get",
    "repository_observe",
    "preflight",
    "verify",
];

pub fn handle_request(request: &Value) -> Value {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    match request.get("method").and_then(Value::as_str) {
        Some("initialize") => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2025-06-18",
                "capabilities": {"tools": {"listChanged": false}},
                "serverInfo": {"name": "ai-cockpit", "version": "0.1.0"}
            }
        }),
        Some("tools/list") => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {"tools": TOOL_NAMES.iter().map(|name| json!({
                "name": name,
                "description": format!("Read-only or bounded verification surface: {name}"),
                "inputSchema": {"type": "object", "additionalProperties": true}
            })).collect::<Vec<_>>()}
        }),
        Some("tools/call") => {
            let name = request
                .pointer("/params/name")
                .and_then(Value::as_str)
                .unwrap_or("");
            if TOOL_NAMES.contains(&name) {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {"content": [{"type": "text", "text": format!("tool {name} requires repository application service binding")}], "isError": false}
                })
            } else {
                error_response(id, -32602, "unknown tool")
            }
        }
        Some("notifications/initialized") => Value::Null,
        Some(_) | None => error_response(id, -32601, "method not found"),
    }
}

pub fn handle_request_for_repo(request: &Value, repo: &Path) -> Value {
    if request.get("method").and_then(Value::as_str) != Some("tools/call") {
        return handle_request(request);
    }
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let name = request
        .pointer("/params/name")
        .and_then(Value::as_str)
        .unwrap_or("");
    let arguments = request
        .pointer("/params/arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let result: Result<Value, String> = match name {
        "status" => cockpit_repository::status(repo)
            .map_err(|error| error.to_string())
            .and_then(|value| serde_json::to_value(value).map_err(|error| error.to_string())),
        "repository_observe" => cockpit_git::GitRepository::discover(repo)
            .and_then(|git| git.snapshot())
            .map_err(|error| error.to_string())
            .and_then(|snapshot| cockpit_repository::observe(&snapshot.root, &snapshot).map_err(|error| error.to_string()))
            .and_then(|value| serde_json::to_value(value).map_err(|error| error.to_string())),
        "knowledge_query" => cockpit_repository::generate_knowledge(repo)
            .map_err(|error| error.to_string())
            .map(|index| {
                let filter = cockpit_knowledge::Query {
                    topic: arguments.get("topic").and_then(Value::as_str).map(str::to_owned),
                    component: arguments.get("component").and_then(Value::as_str).map(str::to_owned),
                    state: arguments.get("state").and_then(Value::as_str).map(str::to_owned),
                    work_item_id: arguments.get("workItemId").and_then(Value::as_str).map(str::to_owned),
                };
                json!({"matchCount": cockpit_knowledge::query(&index, &filter).len(), "results": cockpit_knowledge::query(&index, &filter)})
            }),
        "blockers" | "safe_actions" => Ok(json!({"items": []})),
        "work_item_list" => Ok(json!({"items": []})),
        "work_item_get" | "evidence_get" => Ok(json!({"state": "read_only_lookup_requires_explicit_id"})),
        "preflight" => preflight_for_repo(repo, &arguments),
        "verify" => verify_for_repo(repo, &arguments),
        _ => return error_response(id, -32602, "unknown tool"),
    };
    match result {
        Ok(value) => {
            json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":serde_json::to_string(&value).unwrap_or_default()}],"structuredContent":value,"isError":false}})
        }
        Err(error) => {
            json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":error.to_string()}],"isError":true}})
        }
    }
}

fn preflight_for_repo(repo: &Path, arguments: &Value) -> Result<Value, String> {
    let contract_path = arguments
        .get("contract")
        .and_then(Value::as_str)
        .ok_or("contract argument is required")?;
    let contract: cockpit_protocol::Contract =
        serde_json::from_slice(&std::fs::read(contract_path).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    cockpit_protocol::validate_protocol_version(contract.protocol_version)
        .map_err(|error| error.to_string())?;
    let git = cockpit_git::GitRepository::discover(repo).map_err(|error| error.to_string())?;
    let snapshot = git.snapshot().map_err(|error| error.to_string())?;
    let decision = cockpit_core::evaluate(cockpit_core::GovernanceInput {
        scope: contract.scope,
        out_of_scope: contract.out_of_scope,
        changed_paths: snapshot.changed_paths,
        action: if contract.risk.contains("destructive") {
            cockpit_core::ActionKind::Destructive
        } else {
            cockpit_core::ActionKind::Write
        },
        authority: if contract.authority == "authorized" {
            cockpit_core::AuthorityState::Authorized
        } else {
            cockpit_core::AuthorityState::Missing
        },
        evidence: if contract.required_evidence_classes.is_empty() {
            cockpit_core::EvidenceState::Complete
        } else {
            cockpit_core::EvidenceState::Missing
        },
        untrusted_material: false,
        test_weakening: false,
        coverage_weakening: false,
    });
    serde_json::to_value(decision).map_err(|error| error.to_string())
}

fn verify_for_repo(repo: &Path, arguments: &Value) -> Result<Value, String> {
    let program = arguments
        .get("command")
        .and_then(Value::as_str)
        .ok_or("command argument is required")?;
    let args = arguments
        .get("args")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let receipt = cockpit_verification::execute_bounded(
        vec![
            cockpit_verification::VerificationCommand::new("mcp-verify", program, args)
                .with_protected(true),
        ],
        2,
    )
    .map_err(|error| error.to_string())?;
    let _ = repo;
    serde_json::to_value(json!({"nodesPlanned":receipt.nodes_planned,"nodesExecuted":receipt.nodes_executed,"nodesReused":receipt.nodes_reused,"processesSpawned":receipt.processes_spawned,"elapsedMs":receipt.elapsed_ms,"passed":receipt.passed})).map_err(|error| error.to_string())
}

fn error_response(id: Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}

pub fn serve<R: BufRead, W: Write>(reader: R, mut writer: W) -> io::Result<()> {
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = serde_json::from_str(&line)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let response = handle_request(&request);
        if !response.is_null() {
            writeln!(writer, "{}", response)?;
            writer.flush()?;
        }
    }
    Ok(())
}

pub fn serve_with_repo<R: BufRead, W: Write>(
    reader: R,
    mut writer: W,
    repo: &Path,
) -> io::Result<()> {
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = serde_json::from_str(&line)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let response = handle_request_for_repo(&request, repo);
        if !response.is_null() {
            writeln!(writer, "{}", response)?;
            writer.flush()?;
        }
    }
    Ok(())
}

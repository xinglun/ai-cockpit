use serde_json::{Value, json};
use sha2::{Digest as ShaDigest, Sha256};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

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
        "repository_observe" => repository_observe(repo),
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
        "blockers" => decision_items(repo, &arguments, "blockers"),
        "safe_actions" => decision_items(repo, &arguments, "safe_actions"),
        "work_item_list" => work_item_list(repo),
        "work_item_get" => work_item_get(repo, &arguments),
        "evidence_get" => evidence_get(repo, &arguments),
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
    let contract_path = repository_path(repo, contract_path)?;
    let contract: cockpit_protocol::Contract =
        serde_json::from_slice(&std::fs::read(contract_path).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    cockpit_protocol::validate_protocol_version(contract.protocol_version)
        .map_err(|error| error.to_string())?;
    let git = cockpit_git::GitRepository::discover(repo).map_err(|error| error.to_string())?;
    let snapshot = git.snapshot().map_err(|error| error.to_string())?;
    let explicit_blockers =
        cockpit_repository::contract_freshness_findings(repo, &contract, &snapshot)
            .map_err(|error| error.to_string())?;
    let decision = cockpit_core::evaluate(cockpit_core::GovernanceInput {
        scope: contract.scope.clone(),
        out_of_scope: contract.out_of_scope.clone(),
        changed_paths: snapshot.changed_paths.clone(),
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
        explicit_blockers,
        explicit_unknowns: vec![],
        outcome_state_override: None,
        authority_override: None,
    });
    serde_json::to_value(decision).map_err(|error| error.to_string())
}

fn verify_for_repo(repo: &Path, arguments: &Value) -> Result<Value, String> {
    let program = arguments
        .get("command")
        .and_then(Value::as_str)
        .ok_or("command argument is required")?;
    let args: Vec<String> = arguments
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
    let allowed = [
        "cargo", "npm", "go", "pytest", "python", "python3", "node", "true",
    ];
    if !allowed.contains(&program) {
        return Err(format!(
            "verification command is not allowlisted: {program}"
        ));
    }
    let root = fs::canonicalize(repo).map_err(|error| error.to_string())?;
    let snapshot = cockpit_git::GitRepository::discover(&root)
        .and_then(|git| git.snapshot())
        .map_err(|error| error.to_string())?;
    let mut graph = cockpit_verification::VerificationGraph::default();
    graph
        .add(cockpit_verification::VerificationNode::new(
            "mcp-verify",
            cockpit_verification::VerificationNodeKind::Protected,
            vec![],
        ))
        .map_err(|error| error.to_string())?;
    let plan = graph.plan().map_err(|error| error.to_string())?;
    let commands = plan
        .into_iter()
        .map(|id| {
            cockpit_verification::VerificationCommand::new(&id, program, args.clone())
                .with_protected(true)
                .with_current_dir(&root)
        })
        .collect();
    let mut receipt =
        cockpit_verification::execute_bounded(commands, 2).map_err(|error| error.to_string())?;
    receipt.git_calls = snapshot.git_calls;
    receipt.files_read += snapshot.files_read;
    receipt.files_hashed += snapshot.files_hashed;
    let output = json!({"nodesPlanned":receipt.nodes_planned,"nodesExecuted":receipt.nodes_executed,"nodesReused":receipt.nodes_reused,"processesSpawned":receipt.processes_spawned,"gitCalls":receipt.git_calls,"filesRead":receipt.files_read,"filesHashed":receipt.files_hashed,"elapsedMs":receipt.elapsed_ms,"passed":receipt.passed});
    if let Some(work_item_id) = arguments.get("workItemId").and_then(Value::as_str) {
        cockpit_repository::record_verification_with_snapshot(
            &root,
            work_item_id,
            &output,
            "0.1.0",
            &cockpit_core::Digest::sha256_bytes(b"ai-cockpit-0.1.0"),
            &snapshot,
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(output)
}

fn repository_observe(repo: &Path) -> Result<Value, String> {
    let git = cockpit_git::GitRepository::discover(repo).map_err(|error| error.to_string())?;
    let snapshot = git.snapshot().map_err(|error| error.to_string())?;
    let observation = cockpit_repository::observe(&snapshot.root, &snapshot)
        .map_err(|error| error.to_string())?;
    let (evolution, profile_update_proposal) = if let Ok(profile_bytes) =
        fs::read(snapshot.root.join(".ai/project.json"))
    {
        let profile: cockpit_protocol::ProjectProfile =
            serde_json::from_slice(&profile_bytes).map_err(|error| error.to_string())?;
        let evolution = cockpit_repository::classify_evolution(&profile, &observation, &snapshot);
        let proposal = cockpit_repository::profile_update_proposal(&profile, &evolution);
        (evolution, proposal)
    } else {
        (Vec::new(), None)
    };
    serde_json::to_value(json!({
        "snapshot": snapshot,
        "observation": observation,
        "evolution": evolution,
        "profileUpdateProposal": profile_update_proposal,
    }))
    .map_err(|error| error.to_string())
}

fn decision_items(repo: &Path, arguments: &Value, field: &str) -> Result<Value, String> {
    let Some(contract) = arguments.get("contract").and_then(Value::as_str) else {
        return Ok(json!({"items": []}));
    };
    let decision = preflight_for_repo(repo, &json!({"contract": contract}))?;
    Ok(json!({"items": decision.get(field).cloned().unwrap_or_else(|| json!([]))}))
}

fn work_item_list(repo: &Path) -> Result<Value, String> {
    let root = fs::canonicalize(repo).map_err(|error| error.to_string())?;
    let mut items = Vec::new();
    for (directory, state) in [
        (".ai/work-items/active", "active"),
        (".ai/work-items/archive", "archived"),
    ] {
        let path = root.join(directory);
        let Ok(entries) = fs::read_dir(&path) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(id) = name.strip_suffix(".contract.json") else {
                continue;
            };
            items.push(json!({"workItemId": id, "state": state}));
        }
    }
    items.sort_by(|left, right| {
        left["workItemId"]
            .as_str()
            .cmp(&right["workItemId"].as_str())
    });
    Ok(json!({"items": items}))
}

fn work_item_get(repo: &Path, arguments: &Value) -> Result<Value, String> {
    let id = arguments
        .get("workItemId")
        .or_else(|| arguments.get("id"))
        .and_then(Value::as_str)
        .ok_or("workItemId argument is required")?;
    validate_id(id)?;
    let root = fs::canonicalize(repo).map_err(|error| error.to_string())?;
    let mut result = serde_json::Map::new();
    for (directory, state) in [
        (".ai/work-items/active", "active"),
        (".ai/work-items/archive", "archived"),
    ] {
        for name in ["contract", "summary", "outcome", "archive"] {
            let path = root.join(directory).join(format!("{id}.{name}.json"));
            if path.is_file() {
                let value: Value =
                    serde_json::from_slice(&fs::read(&path).map_err(|error| error.to_string())?)
                        .map_err(|error| error.to_string())?;
                result.insert(name.into(), value);
                result.insert("state".into(), Value::String(state.into()));
            }
        }
    }
    if result.is_empty() {
        return Err("work item not found".into());
    }
    Ok(Value::Object(result))
}

fn evidence_get(repo: &Path, arguments: &Value) -> Result<Value, String> {
    let requested = arguments
        .get("path")
        .or_else(|| arguments.get("evidencePath"))
        .and_then(Value::as_str)
        .or_else(|| arguments.get("id").and_then(Value::as_str))
        .ok_or("path or id argument is required")?;
    let path = if requested.contains('/') || requested.ends_with(".json") {
        repository_path(repo, requested)?
    } else {
        repository_path(repo, &format!(".ai/evidence/{requested}.json"))?
    };
    let bytes = fs::read(&path).map_err(|error| error.to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let digest = format!("sha256:{}", hex::encode(hasher.finalize()));
    let content = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()));
    Ok(
        json!({"path": path.strip_prefix(fs::canonicalize(repo).map_err(|error| error.to_string())?).unwrap_or(&path), "digest": digest, "content": content}),
    )
}

fn repository_path(repo: &Path, requested: &str) -> Result<PathBuf, String> {
    let root = fs::canonicalize(repo).map_err(|error| error.to_string())?;
    let candidate = if Path::new(requested).is_absolute() {
        PathBuf::from(requested)
    } else {
        root.join(requested)
    };
    let canonical = fs::canonicalize(&candidate).map_err(|error| error.to_string())?;
    if !canonical.starts_with(&root) {
        return Err("path escapes repository root".into());
    }
    Ok(canonical)
}

fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err("invalid work item id".into());
    }
    Ok(())
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

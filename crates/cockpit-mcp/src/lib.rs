use serde_json::{Value, json};
use sha2::{Digest as ShaDigest, Sha256};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

const TOOL_NAMES: [&str; 13] = [
    "status",
    "work_item_get",
    "work_item_outcome",
    "work_item_validate",
    "work_item_list",
    "blockers",
    "safe_actions",
    "knowledge_query",
    "evidence_get",
    "delegated_evidence_list",
    "repository_observe",
    "preflight",
    "verify",
];

pub fn handle_request(request: &Value, runtime: &cockpit_protocol::RuntimeContext) -> Value {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    match request.get("method").and_then(Value::as_str) {
        Some("initialize") => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2025-06-18",
                "capabilities": {"tools": {"listChanged": false}},
                "serverInfo": {
                    "name": "ai-cockpit",
                    "version": &runtime.runtime_version,
                    "runtimeDigest": &runtime.runtime_digest,
                }
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
                error_response(
                    id,
                    -32001,
                    &format!("tool {name} requires an explicit repository binding"),
                )
            } else {
                error_response(id, -32602, "unknown tool")
            }
        }
        Some("notifications/initialized") => Value::Null,
        Some(_) | None => error_response(id, -32601, "method not found"),
    }
}

pub fn handle_request_for_repo(
    request: &Value,
    repo: &Path,
    runtime: &cockpit_protocol::RuntimeContext,
) -> Value {
    if request.get("method").and_then(Value::as_str) != Some("tools/call") {
        return handle_request(request, runtime);
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
        "knowledge_query" => {
            require_compatible(repo, runtime).and_then(|_| {
                cockpit_repository::generate_knowledge(repo)
                    .map_err(|error| error.to_string())
                    .map(|index| {
                        let filter = cockpit_knowledge::Query {
                            topic: arguments.get("topic").and_then(Value::as_str).map(str::to_owned),
                            component: arguments.get("component").and_then(Value::as_str).map(str::to_owned),
                            state: arguments.get("state").and_then(Value::as_str).map(str::to_owned),
                            work_item_id: arguments.get("workItemId").and_then(Value::as_str).map(str::to_owned),
                        };
                        json!({"matchCount": cockpit_knowledge::query(&index, &filter).len(), "results": cockpit_knowledge::query(&index, &filter)})
                    })
            })
        }
        "blockers" => {
            require_compatible(repo, runtime)
                .and_then(|_| decision_items(repo, &arguments, "blockers", runtime))
        }
        "safe_actions" => {
            require_compatible(repo, runtime)
                .and_then(|_| decision_items(repo, &arguments, "safe_actions", runtime))
        }
        "work_item_list" => work_item_list(repo),
        "work_item_get" => work_item_get(repo, &arguments),
        "work_item_outcome" => {
            require_compatible(repo, runtime)
                .and_then(|_| work_item_outcome(repo, &arguments, runtime))
        }
        "work_item_validate" => {
            require_compatible(repo, runtime).and_then(|_| work_item_validate(repo, &arguments))
        }
        "evidence_get" => evidence_get(repo, &arguments),
        "delegated_evidence_list" => {
            require_compatible(repo, runtime).and_then(|_| {
                let work_item_id = arguments
                    .get("workItemId")
                    .and_then(Value::as_str)
                    .ok_or("workItemId argument is required")?;
                cockpit_repository::list_delegated_evidence(repo, work_item_id)
                    .map_err(|error| error.to_string())
                    .and_then(|receipts| {
                        serde_json::to_value(receipts).map_err(|error| error.to_string())
                    })
            })
        }
        "preflight" => {
            require_compatible(repo, runtime)
                .and_then(|_| preflight_for_repo(repo, &arguments, runtime))
        }
        "verify" => verify_for_repo(repo, &arguments, runtime),
        _ => return error_response(id, -32602, "unknown tool"),
    };
    match result {
        Ok(value) => {
            let text = if name == "work_item_outcome" {
                value
                    .get("humanHandoff")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned()
            } else {
                serde_json::to_string(&value).unwrap_or_default()
            };
            json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":text}],"structuredContent":value,"isError":false}})
        }
        Err(error) => {
            json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":error.to_string()}],"isError":true}})
        }
    }
}

fn preflight_for_repo(
    repo: &Path,
    arguments: &Value,
    runtime: &cockpit_protocol::RuntimeContext,
) -> Result<Value, String> {
    let contract_path = arguments
        .get("contract")
        .and_then(Value::as_str)
        .ok_or("contract argument is required")?;
    let contract_path = repository_path(repo, contract_path)?;
    let decision =
        cockpit_repository::preflight_work_item_with_runtime(repo, &contract_path, runtime)
            .map_err(|error| error.to_string())?;
    serde_json::to_value(decision).map_err(|error| error.to_string())
}

fn verify_for_repo(
    repo: &Path,
    arguments: &Value,
    runtime: &cockpit_protocol::RuntimeContext,
) -> Result<Value, String> {
    require_compatible(repo, runtime)?;
    let root = fs::canonicalize(repo).map_err(|error| error.to_string())?;
    if let Some(work_item_id) = arguments.get("workItemId").and_then(Value::as_str) {
        cockpit_repository::require_policy_for_verification(&root, work_item_id)
            .map_err(|error| error.to_string())?;
    }
    let explicit_program = match arguments.get("command") {
        Some(Value::String(program)) => Some(program.as_str()),
        Some(_) => return Err("command argument must be a string".into()),
        None => None,
    };
    let explicit = explicit_program.is_some();
    let supplied_args = match arguments.get("args") {
        Some(Value::Array(items)) if items.iter().all(Value::is_string) => items
            .iter()
            .map(|item| item.as_str().expect("validated string").to_owned())
            .collect(),
        Some(Value::Array(_)) => return Err("every args element must be a string".into()),
        Some(_) => return Err("args argument must be an array".into()),
        None => Vec::new(),
    };
    let allowed = [
        "cargo", "npm", "go", "pytest", "python", "python3", "node", "true",
    ];
    if explicit_program.is_some_and(|program| !allowed.contains(&program)) {
        return Err(format!(
            "verification command is not allowlisted: {}",
            explicit_program.expect("explicit program exists")
        ));
    }
    let (program, args) = if let Some(program) = explicit_program {
        (program.to_owned(), supplied_args)
    } else if root.join("Cargo.toml").is_file() {
        ("cargo".into(), vec!["test".into(), "--workspace".into()])
    } else if root.join("package.json").is_file() {
        ("npm".into(), vec!["test".into()])
    } else {
        return Err("no verified project command detected; provide command".into());
    };
    let run = cockpit_repository::run_repository_verification(
        &root,
        &cockpit_repository::RepositoryVerificationRequest {
            node_id: "project-command-0".into(),
            program,
            args,
            scope: vec!["**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 2,
            policy: if explicit || arguments.get("workItemId").is_some() {
                cockpit_repository::RepositoryVerificationPolicy::NeverReuse
            } else {
                cockpit_repository::RepositoryVerificationPolicy::ProfileAuthorized
            },
        },
    )
    .map_err(|error| error.to_string())?;
    let mut output = serde_json::to_value(&run.receipt).map_err(|error| error.to_string())?;
    output["runtimeVersion"] = Value::String(runtime.runtime_version.clone());
    output["runtimeDigest"] = Value::String(runtime.runtime_digest.to_string());
    if let Some(work_item_id) = arguments.get("workItemId").and_then(Value::as_str) {
        cockpit_repository::record_verification_with_runtime(
            &root,
            work_item_id,
            &output,
            runtime,
            &run.final_snapshot,
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(output)
}

fn require_compatible(
    repo: &Path,
    runtime: &cockpit_protocol::RuntimeContext,
) -> Result<(), String> {
    if !["cockpit.toml", "project.json", "agent-interface.json"]
        .iter()
        .all(|name| repo.join(".ai").join(name).is_file())
    {
        return Ok(());
    }
    let report = cockpit_repository::compatibility_report(repo, runtime)
        .map_err(|error| error.to_string())?;
    if report.state != "COMPATIBLE" {
        return Err(format!(
            "repository compatibility is {}; run ai-cockpit migrate plan --repo <repository> and apply the reviewed migration before continuing",
            report.state
        ));
    }
    Ok(())
}

fn repository_observe(repo: &Path) -> Result<Value, String> {
    let git = cockpit_git::GitRepository::discover(repo).map_err(|error| error.to_string())?;
    let snapshot = git.snapshot().map_err(|error| error.to_string())?;
    let observation = cockpit_repository::observe(&snapshot.root, &snapshot)
        .map_err(|error| error.to_string())?;
    let (evolution, profile_update_proposal) = if let Ok(profile_bytes) =
        fs::read(snapshot.root.join(".ai/project.json"))
    {
        let profile: cockpit_repository::AttachedProfile =
            serde_json::from_slice(&profile_bytes).map_err(|error| error.to_string())?;
        let profile = cockpit_protocol::ProjectProfile {
            profile_version: profile.profile_version,
            repository_id: profile.repository_id,
            tests: profile.tests,
            build_systems: profile.build_systems,
        };
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

fn decision_items(
    repo: &Path,
    arguments: &Value,
    field: &str,
    runtime: &cockpit_protocol::RuntimeContext,
) -> Result<Value, String> {
    let Some(contract) = arguments.get("contract").and_then(Value::as_str) else {
        return Ok(json!({"items": []}));
    };
    let decision = preflight_for_repo(repo, &json!({"contract": contract}), runtime)?;
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

fn work_item_outcome(
    repo: &Path,
    arguments: &Value,
    runtime: &cockpit_protocol::RuntimeContext,
) -> Result<Value, String> {
    let id = arguments
        .get("workItemId")
        .or_else(|| arguments.get("id"))
        .and_then(Value::as_str)
        .ok_or("workItemId argument is required")?;
    validate_id(id)?;
    let outcome = cockpit_repository::outcome_v2_with_runtime(repo, id, runtime)
        .map_err(|error| error.to_string())?;
    let language = requested_language(arguments);
    let handoff = cockpit_repository::render_human_outcome(repo, &outcome, language);
    Ok(json!({
        "workItemId": id,
        "outcome": outcome,
        "humanHandoff": handoff,
        "language": language,
        "contractLanguageBoundary": "Acceptance criteria remain in their original Contract language and are not machine-translated."
    }))
}

fn work_item_validate(repo: &Path, arguments: &Value) -> Result<Value, String> {
    let id = arguments
        .get("workItemId")
        .or_else(|| arguments.get("id"))
        .and_then(Value::as_str)
        .ok_or("workItemId argument is required")?;
    validate_id(id)?;
    let report = cockpit_repository::validate_work_item_governance_controls(repo, id)
        .map_err(|error| error.to_string())?;
    serde_json::to_value(report).map_err(|error| error.to_string())
}

fn requested_language(arguments: &Value) -> &'static str {
    let requested = arguments
        .get("language")
        .and_then(Value::as_str)
        .map(str::to_ascii_lowercase)
        .or_else(|| {
            std::env::var("AI_COCKPIT_LANGUAGE")
                .ok()
                .map(|value| value.to_ascii_lowercase())
        })
        .or_else(|| {
            std::env::var("LANG")
                .ok()
                .map(|value| value.to_ascii_lowercase())
        })
        .unwrap_or_default();
    if requested.starts_with("zh") {
        "zh"
    } else if requested.starts_with("ja") {
        "ja"
    } else {
        "en"
    }
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

pub fn serve<R: BufRead, W: Write>(
    reader: R,
    mut writer: W,
    runtime: &cockpit_protocol::RuntimeContext,
) -> io::Result<()> {
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = serde_json::from_str(&line)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let response = handle_request(&request, runtime);
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
    runtime: &cockpit_protocol::RuntimeContext,
) -> io::Result<()> {
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = serde_json::from_str(&line)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let response = handle_request_for_repo(&request, repo, runtime);
        if !response.is_null() {
            writeln!(writer, "{}", response)?;
            writer.flush()?;
        }
    }
    Ok(())
}

use serde_json::{Value, json};
use sha2::{Digest as ShaDigest, Sha256};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

const TOOL_NAMES: [&str; 20] = [
    "status",
    "work_item_get",
    "work_item_start",
    "work_item_outcome",
    "work_item_status",
    "work_item_validate",
    "work_item_list",
    "blockers",
    "safe_actions",
    "knowledge_query",
    "evidence_get",
    "delegated_evidence_list",
    "repository_observe",
    "capability_show",
    "preflight",
    "work_item_controls",
    "work_item_recover",
    "work_item_recover_selected_lineage",
    "verify",
    "work_item_parallel",
];

fn string_property(description: &str) -> Value {
    json!({
        "type": "string",
        "minLength": 1,
        "description": description,
    })
}

fn object_schema(properties: Value, required: &[&str]) -> Value {
    let mut schema = json!({
        "type": "object",
        "properties": properties,
        "additionalProperties": false,
    });
    if !required.is_empty() {
        schema["required"] = json!(required);
    }
    schema
}

fn one_of_aliases(names: &[&str]) -> Value {
    Value::Array(
        names
            .iter()
            .map(|name| json!({"required": [name]}))
            .collect(),
    )
}

fn mcp_tool_schema(name: &str) -> Value {
    let id_properties = json!({
        "workItemId": string_property("Canonical Work Item identifier."),
        "id": string_property("Deprecated alias for workItemId."),
    });
    match name {
        "status" | "work_item_list" | "repository_observe" | "capability_show" => {
            object_schema(json!({}), &[])
        }
        "work_item_get" => {
            let mut schema = object_schema(id_properties, &[]);
            schema["oneOf"] = one_of_aliases(&["workItemId", "id"]);
            schema
        }
        "work_item_start" => object_schema(
            json!({
                "workItemId": string_property("Canonical Work Item identifier."),
                "intent": string_property("Human-supplied reason for the Work Item; do not infer missing intent."),
                "goal": string_property("Human-supplied bounded outcome."),
                "scope": {
                    "type": "array",
                    "minItems": 1,
                    "items": string_property("Repository-relative path or glob."),
                },
                "outOfScope": {"type":"array", "items":string_property("Explicit excluded path or behavior.")},
                "risk": string_property("Risk classification; defaults to normal."),
                "authority": string_property("Human-supplied authority; defaults to missing and never grants authority by inference."),
                "acceptanceCriteria": {"type":"array", "items":string_property("Human-supplied acceptance criterion.")},
                "requiredEvidenceClasses": {"type":"array", "items":string_property("Evidence class required by the Contract.")},
                "sources": {"type":"array", "items":string_property("Human- or task-provided source reference appended before preflight.")},
            }),
            &["workItemId", "intent", "goal", "scope"],
        ),
        "work_item_outcome" => {
            let mut properties = id_properties;
            properties["language"] = string_property(
                "Presentation language: en, zh, or ja (regional forms are accepted).",
            );
            properties["view"] = json!({
                "type": "string",
                "enum": ["summary", "full"],
                "default": "summary",
                "description": "Human handoff projection; summary is the reader-first default and full retains the complete audit report."
            });
            properties["delivery"] = json!({
                "type": "boolean",
                "default": false,
                "description": "For an archived Work Item, return the versioned full Outcome delivery payload. This proves tool return only; host acceptance or display remains unknown unless the host provides it."
            });
            let mut schema = object_schema(properties, &[]);
            schema["oneOf"] = one_of_aliases(&["workItemId", "id"]);
            schema
        }
        "work_item_status" => {
            let mut properties = id_properties;
            properties["all"] = json!({
                "type": "boolean",
                "description": "When true, return the stable repository-wide Work Item index.",
            });
            let mut schema = object_schema(properties, &[]);
            schema["oneOf"] = json!([
                {"properties": {"all": {"const": true}}, "required": ["all"]},
                {"required": ["workItemId"]},
                {"required": ["id"]}
            ]);
            schema
        }
        "work_item_validate" => {
            let mut schema = object_schema(id_properties, &[]);
            schema["oneOf"] = one_of_aliases(&["workItemId", "id"]);
            schema
        }
        "blockers" | "safe_actions" => object_schema(
            json!({
                "contract": string_property("Repository-relative Contract path."),
            }),
            &[],
        ),
        "knowledge_query" => object_schema(
            json!({
                "topic": string_property("Optional knowledge topic filter."),
                "component": string_property("Optional component filter."),
                "state": string_property("Optional knowledge state filter."),
                "workItemId": string_property("Optional Work Item filter."),
            }),
            &[],
        ),
        "evidence_get" => {
            let mut schema = object_schema(
                json!({
                    "path": string_property("Repository-relative evidence path."),
                    "evidencePath": string_property("Deprecated alias for path."),
                    "id": string_property("Evidence identifier or Work Item evidence stem."),
                }),
                &[],
            );
            schema["oneOf"] = one_of_aliases(&["path", "evidencePath", "id"]);
            schema
        }
        "delegated_evidence_list" => object_schema(
            json!({
                "workItemId": string_property("Work Item whose provider evidence is listed."),
            }),
            &["workItemId"],
        ),
        "preflight" => object_schema(
            json!({
                "contract": string_property("Repository-relative Contract path."),
            }),
            &["contract"],
        ),
        "work_item_controls" => {
            let mut properties = id_properties;
            properties["controls"] = json!({
                "type": "object",
                "description": "Explicit governance-control projection object.",
            });
            properties["input"] = json!({
                "type": "object",
                "description": "Deprecated alias for controls.",
            });
            let mut schema = object_schema(properties, &[]);
            schema["oneOf"] = one_of_aliases(&["workItemId", "id"]);
            schema["allOf"] = json!([
                {"oneOf": [{"required": ["controls"]}, {"required": ["input"]}]}
            ]);
            schema
        }
        "work_item_recover" => {
            let mut properties = id_properties;
            properties["receipt"] = json!({
                "type": "object",
                "description": "Identity-bound recovery decision receipt.",
            });
            properties["input"] = json!({
                "type": "object",
                "description": "Deprecated alias for receipt.",
            });
            let mut schema = object_schema(properties, &[]);
            schema["oneOf"] = one_of_aliases(&["workItemId", "id"]);
            schema["allOf"] = json!([
                {"oneOf": [{"required": ["receipt"]}, {"required": ["input"]}]}
            ]);
            schema
        }
        "work_item_recover_selected_lineage" => {
            let mut properties = id_properties;
            properties["receipt"] = json!({
                "type": "object",
                "description": "Identity-bound append-only multi-hop successor lineage recovery receipt.",
            });
            properties["input"] = json!({
                "type": "object",
                "description": "Deprecated alias for receipt.",
            });
            let mut schema = object_schema(properties, &[]);
            schema["oneOf"] = one_of_aliases(&["workItemId", "id"]);
            schema["allOf"] = json!([
                {"oneOf": [{"required": ["receipt"]}, {"required": ["input"]}]}
            ]);
            schema
        }
        "verify" => object_schema(
            json!({
                "workItemId": string_property("Optional Work Item to bind the verification receipt."),
                "command": string_property("Allowlisted executable; omit to detect Cargo or npm."),
                "args": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Command arguments as a string array.",
                },
                "timeoutSeconds": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 900,
                    "description": "Finite command timeout in seconds. Omission preserves the 300-second Runtime default; explicit values require Contract or repository policy authorization.",
                },
                "planOnly": {
                    "type": "boolean",
                    "description": "Return identity-bound execute/reuse/stale actions without starting project verification commands.",
                },
            }),
            &[],
        ),
        "work_item_parallel" => parallel_tool_schema(),
        _ => object_schema(json!({}), &[]),
    }
}

fn parallel_tool_schema() -> Value {
    let properties = json!({
        "action": {
            "type": "string",
            "enum": ["inspect", "acquire", "release", "list"],
            "default": "inspect",
            "description": "Slot action.",
        },
        "workItemId": string_property("Canonical Work Item identifier."),
        "id": string_property("Deprecated alias for workItemId."),
        "leaseId": string_property("Lease identifier required by release."),
    });
    let item_id = json!({
        "oneOf": [{"required": ["workItemId"]}, {"required": ["id"]}]
    });
    json!({
        "type": "object",
        "properties": properties,
        "additionalProperties": false,
        "oneOf": [
            {"allOf": [item_id.clone()], "not": {"required": ["action"]}},
            {"allOf": [item_id.clone(), {"required": ["action"]}], "properties": {"action": {"const": "inspect"}}},
            {"allOf": [item_id.clone(), {"required": ["action"]}], "properties": {"action": {"const": "acquire"}}},
            {"allOf": [item_id, {"required": ["action", "leaseId"]}], "properties": {"action": {"const": "release"}}},
            {"required": ["action"], "properties": {"action": {"const": "list"}}, "not": {"anyOf": [{"required": ["workItemId"]}, {"required": ["id"]}, {"required": ["leaseId"]}]}}
        ]
    })
}

fn mcp_tool_definitions() -> Vec<Value> {
    let descriptions = [
        ("status", "Read current repository protocol status."),
        ("work_item_get", "Read raw records for one Work Item."),
        (
            "work_item_start",
            "Start a Work Item, append supplied sources, persist preflight, and create one before-edit checkpoint only when no blocker or human-confirmation boundary is present.",
        ),
        (
            "work_item_outcome",
            "Render a localized human handoff and structured OutcomeV2.",
        ),
        (
            "work_item_status",
            "Read one Work Item or the stable repository-wide status index.",
        ),
        (
            "work_item_validate",
            "Validate one Work Item's Contract and governance controls.",
        ),
        ("work_item_list", "List active and archived Work Items."),
        (
            "blockers",
            "Read blockers derived from an optional Contract.",
        ),
        (
            "safe_actions",
            "Read safe recovery actions derived from an optional Contract.",
        ),
        (
            "knowledge_query",
            "Query repository-local derived knowledge.",
        ),
        (
            "evidence_get",
            "Read one repository-bound evidence record and its digest.",
        ),
        (
            "delegated_evidence_list",
            "List repository-bound provider evidence receipts.",
        ),
        (
            "repository_observe",
            "Observe repository facts and profile evolution without governance writes.",
        ),
        (
            "capability_show",
            "Show Runtime- and repository-bound capability truth.",
        ),
        (
            "preflight",
            "Evaluate a repository-relative Contract before implementation.",
        ),
        (
            "work_item_controls",
            "Record explicitly supplied Work Item governance controls.",
        ),
        (
            "work_item_recover",
            "Record an identity-bound retry, successor, or supersede decision.",
        ),
        (
            "work_item_recover_selected_lineage",
            "Record append-only recovery for an already selected multi-hop successor lineage without creating a competing successor.",
        ),
        (
            "verify",
            "Plan or run an allowlisted verification command and optionally bind its receipt.",
        ),
        (
            "work_item_parallel",
            "Inspect or manage repository-local parallel Work Item slots.",
        ),
    ];
    descriptions
        .into_iter()
        .map(|(name, description)| {
            json!({
                "name": name,
                "description": description,
                "inputSchema": mcp_tool_schema(name),
            })
        })
        .collect()
}

fn validate_tool_arguments(name: &str, arguments: &Value) -> Result<(), String> {
    let object = arguments
        .as_object()
        .ok_or_else(|| format!("invalid arguments for {name}: expected a JSON object"))?;

    let allowed = match name {
        "status" | "work_item_list" | "repository_observe" | "capability_show" => &[][..],
        "work_item_get" | "work_item_validate" => &["workItemId", "id"][..],
        "work_item_start" => &[
            "workItemId",
            "intent",
            "goal",
            "scope",
            "outOfScope",
            "risk",
            "authority",
            "acceptanceCriteria",
            "requiredEvidenceClasses",
            "sources",
        ][..],
        "work_item_outcome" => &["workItemId", "id", "language", "view", "delivery"][..],
        "work_item_status" => &["workItemId", "id", "all"][..],
        "blockers" | "safe_actions" | "preflight" => &["contract"][..],
        "knowledge_query" => &["topic", "component", "state", "workItemId"][..],
        "evidence_get" => &["path", "evidencePath", "id"][..],
        "delegated_evidence_list" => &["workItemId"][..],
        "work_item_controls" => &["workItemId", "id", "controls", "input"][..],
        "work_item_recover" => &["workItemId", "id", "receipt", "input"][..],
        "work_item_recover_selected_lineage" => &["workItemId", "id", "receipt", "input"][..],
        "verify" => &[
            "workItemId",
            "command",
            "args",
            "timeoutSeconds",
            "planOnly",
        ][..],
        "work_item_parallel" => &["action", "workItemId", "id", "leaseId"][..],
        _ => return Err(format!("unknown tool: {name}")),
    };
    for key in object.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(format!("invalid arguments for {name}: unknown field {key}"));
        }
    }

    match name {
        "work_item_get" | "work_item_outcome" | "work_item_validate" => {
            require_exactly_one_string(object, &["workItemId", "id"], name)?;
            if name == "work_item_outcome" {
                optional_string(object, "language", name)?;
                if let Some(delivery) = object.get("delivery")
                    && !delivery.is_boolean()
                {
                    return Err(format!(
                        "invalid arguments for {name}: delivery must be a boolean"
                    ));
                }
                if let Some(view) = object.get("view") {
                    let view = view.as_str().ok_or_else(|| {
                        format!("invalid arguments for {name}: view must be a string")
                    })?;
                    if !matches!(view, "summary" | "full") {
                        return Err(format!(
                            "invalid arguments for {name}: view must be summary or full"
                        ));
                    }
                }
            }
        }
        "work_item_start" => {
            require_string(object, "workItemId", name)?;
            require_string(object, "intent", name)?;
            require_string(object, "goal", name)?;
            parse_string_array(object, "scope", name, true, 1)?;
            for field in ["risk", "authority"] {
                optional_string(object, field, name)?;
            }
            for field in [
                "outOfScope",
                "acceptanceCriteria",
                "requiredEvidenceClasses",
                "sources",
            ] {
                parse_string_array(object, field, name, false, 0)?;
            }
        }
        "work_item_status" => {
            if let Some(value) = object.get("all") {
                if !value.is_boolean() {
                    return Err(format!(
                        "invalid arguments for {name}: all must be a boolean"
                    ));
                }
                if value.as_bool() == Some(true)
                    && (object.contains_key("workItemId") || object.contains_key("id"))
                {
                    return Err(
                        "invalid arguments for work_item_status: all=true cannot include a Work Item id"
                            .into(),
                    );
                }
            }
            if object.get("all").and_then(Value::as_bool) != Some(true) {
                require_exactly_one_string(object, &["workItemId", "id"], name)?;
            }
        }
        "blockers" | "safe_actions" => optional_string(object, "contract", name)?,
        "knowledge_query" => {
            for field in ["topic", "component", "state", "workItemId"] {
                optional_string(object, field, name)?;
            }
        }
        "evidence_get" => {
            require_exactly_one_string(object, &["path", "evidencePath", "id"], name)?;
        }
        "delegated_evidence_list" => require_string(object, "workItemId", name)?,
        "preflight" => require_string(object, "contract", name)?,
        "work_item_controls" => {
            require_exactly_one_string(object, &["workItemId", "id"], name)?;
            require_exactly_one_object_alias(object, &["controls", "input"], name)?;
        }
        "work_item_recover" => {
            require_exactly_one_string(object, &["workItemId", "id"], name)?;
            require_exactly_one_object_alias(object, &["receipt", "input"], name)?;
        }
        "work_item_recover_selected_lineage" => {
            require_exactly_one_string(object, &["workItemId", "id"], name)?;
            require_exactly_one_object_alias(object, &["receipt", "input"], name)?;
        }
        "verify" => {
            if let Some(value) = object.get("workItemId")
                && (!value.is_string() || value.as_str().is_some_and(str::is_empty))
            {
                return Err(format!(
                    "invalid arguments for {name}: workItemId must be a non-empty string"
                ));
            }
            if let Some(value) = object.get("command")
                && (!value.is_string() || value.as_str().is_some_and(str::is_empty))
            {
                return Err(format!(
                    "invalid arguments for {name}: command must be a non-empty string"
                ));
            }
            if let Some(value) = object.get("args") {
                let Some(items) = value.as_array() else {
                    return Err(format!(
                        "invalid arguments for {name}: args must be an array of strings"
                    ));
                };
                if items.iter().any(|item| !item.is_string()) {
                    return Err(format!(
                        "invalid arguments for {name}: args must be an array of strings"
                    ));
                }
            }
            if object
                .get("planOnly")
                .is_some_and(|value| !value.is_boolean())
            {
                return Err(format!(
                    "invalid arguments for {name}: planOnly must be a boolean"
                ));
            }
            if let Some(value) = object.get("timeoutSeconds")
                && value.as_u64().is_none_or(|seconds| seconds == 0)
            {
                return Err(format!(
                    "invalid arguments for {name}: timeoutSeconds must be a positive integer"
                ));
            }
        }
        "work_item_parallel" => {
            let action = match object.get("action") {
                None => "inspect",
                Some(Value::String(value)) => value.as_str(),
                Some(_) => {
                    return Err(
                        "invalid arguments for work_item_parallel: action must be a string".into(),
                    );
                }
            };
            if !matches!(action, "inspect" | "acquire" | "release" | "list") {
                return Err(
                    "invalid arguments for work_item_parallel: action must be inspect, acquire, release, or list"
                        .into(),
                );
            }
            match action {
                "inspect" | "acquire" => {
                    require_exactly_one_string(object, &["workItemId", "id"], name)?;
                    if object.contains_key("leaseId") {
                        return Err(
                            "invalid arguments for work_item_parallel: leaseId is only valid for release"
                                .into(),
                        );
                    }
                }
                "release" => {
                    require_exactly_one_string(object, &["workItemId", "id"], name)?;
                    require_string(object, "leaseId", name)?;
                }
                "list" => {
                    if object.keys().any(|key| key != "action") {
                        return Err(
                            "invalid arguments for work_item_parallel: list accepts only action"
                                .into(),
                        );
                    }
                }
                _ => unreachable!(),
            }
        }
        "status" | "work_item_list" | "repository_observe" | "capability_show" => {}
        _ => {}
    }
    Ok(())
}

fn require_string(
    object: &serde_json::Map<String, Value>,
    field: &str,
    tool: &str,
) -> Result<(), String> {
    match object.get(field) {
        Some(Value::String(value)) if !value.is_empty() => Ok(()),
        Some(_) => Err(format!(
            "invalid arguments for {tool}: {field} must be a non-empty string"
        )),
        None => Err(format!("invalid arguments for {tool}: {field} is required")),
    }
}

fn optional_string(
    object: &serde_json::Map<String, Value>,
    field: &str,
    tool: &str,
) -> Result<(), String> {
    if let Some(value) = object.get(field)
        && (!value.is_string() || value.as_str().is_some_and(str::is_empty))
    {
        return Err(format!(
            "invalid arguments for {tool}: {field} must be a non-empty string"
        ));
    }
    Ok(())
}

fn parse_string_array(
    object: &serde_json::Map<String, Value>,
    field: &str,
    tool: &str,
    required: bool,
    minimum_items: usize,
) -> Result<Vec<String>, String> {
    let Some(value) = object.get(field) else {
        return if required {
            Err(format!("invalid arguments for {tool}: {field} is required"))
        } else {
            Ok(Vec::new())
        };
    };
    let Some(items) = value.as_array() else {
        return Err(format!(
            "invalid arguments for {tool}: {field} must be an array of strings"
        ));
    };
    if items.len() < minimum_items
        || items
            .iter()
            .any(|item| item.as_str().is_none_or(|value| value.trim().is_empty()))
    {
        return Err(format!(
            "invalid arguments for {tool}: {field} must contain at least {minimum_items} non-empty strings"
        ));
    }
    Ok(items
        .iter()
        .map(|item| item.as_str().expect("validated string").to_owned())
        .collect())
}

fn require_exactly_one_string(
    object: &serde_json::Map<String, Value>,
    fields: &[&str],
    tool: &str,
) -> Result<(), String> {
    let present = fields
        .iter()
        .filter(|field| object.contains_key(**field))
        .copied()
        .collect::<Vec<_>>();
    if present.len() != 1 {
        return Err(format!(
            "invalid arguments for {tool}: exactly one of {} is required",
            fields.join(", ")
        ));
    }
    require_string(object, present[0], tool)
}

fn require_exactly_one_object_alias(
    object: &serde_json::Map<String, Value>,
    fields: &[&str],
    tool: &str,
) -> Result<(), String> {
    let present = fields
        .iter()
        .filter(|field| object.contains_key(**field))
        .copied()
        .collect::<Vec<_>>();
    if present.len() != 1 {
        return Err(format!(
            "invalid arguments for {tool}: exactly one of {} is required",
            fields.join(", ")
        ));
    }
    if !object[present[0]].is_object() {
        return Err(format!(
            "invalid arguments for {tool}: {} must be a JSON object",
            present[0]
        ));
    }
    Ok(())
}

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
            "result": {"tools": mcp_tool_definitions()}
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
    if !TOOL_NAMES.contains(&name) {
        return error_response(id, -32602, "unknown tool");
    }
    if let Err(error) = validate_tool_arguments(name, &arguments) {
        return tool_error_response(id, &error);
    }
    let result: Result<Value, String> = match name {
        "status" => cockpit_repository::status(repo)
            .map_err(|error| error.to_string())
            .and_then(|value| serde_json::to_value(value).map_err(|error| error.to_string())),
        "repository_observe" => repository_observe(repo),
        "capability_show" => require_compatible(repo, runtime).and_then(|_| {
            cockpit_repository::capability_truth_registry_with_runtime(repo, runtime)
                .map_err(|error| error.to_string())
                .and_then(|registry| {
                    serde_json::to_value(registry).map_err(|error| error.to_string())
                })
        }),
        "knowledge_query" => require_compatible(repo, runtime).and_then(|_| {
            let projection_path = repo.join(".ai/knowledge/index.json");
            let before = fs::read(&projection_path).ok();
            cockpit_repository::generate_knowledge(repo)
                .map_err(|error| error.to_string())
                .map(|index| {
                    let after = fs::read(&projection_path).ok();
                    let materialization = if before.is_none() {
                        "created"
                    } else if before != after {
                        "rebuilt"
                    } else {
                        "reused"
                    };
                    let filter = cockpit_knowledge::Query {
                        topic: arguments
                            .get("topic")
                            .and_then(Value::as_str)
                            .map(str::to_owned),
                        component: arguments
                            .get("component")
                            .and_then(Value::as_str)
                            .map(str::to_owned),
                        state: arguments
                            .get("state")
                            .and_then(Value::as_str)
                            .map(str::to_owned),
                        work_item_id: arguments
                            .get("workItemId")
                            .and_then(Value::as_str)
                            .map(str::to_owned),
                    };
                    let results = cockpit_knowledge::query(&index, &filter);
                    json!({
                        "schemaVersion": 1,
                        "projection": {
                            "path": ".ai/knowledge/index.json",
                            "materialization": materialization,
                            "writeBoundary": "repository-local-derived",
                            "authority": "none",
                            "sourceDigest": index.source_digest
                        },
                        "matchCount": results.len(),
                        "results": results
                    })
                })
        }),
        "blockers" => require_compatible(repo, runtime)
            .and_then(|_| decision_items(repo, &arguments, "blockers", runtime)),
        "safe_actions" => require_compatible(repo, runtime)
            .and_then(|_| decision_items(repo, &arguments, "safe_actions", runtime)),
        "work_item_list" => work_item_list(repo),
        "work_item_get" => work_item_get(repo, &arguments),
        "work_item_start" => require_compatible(repo, runtime)
            .and_then(|_| work_item_start(repo, &arguments, runtime)),
        "work_item_outcome" => require_compatible(repo, runtime)
            .and_then(|_| work_item_outcome(repo, &arguments, runtime)),
        "work_item_status" => require_compatible(repo, runtime)
            .and_then(|_| work_item_status(repo, &arguments, runtime)),
        "work_item_validate" => require_compatible(repo, runtime)
            .and_then(|_| work_item_validate(repo, &arguments, runtime)),
        "evidence_get" => evidence_get(repo, &arguments),
        "delegated_evidence_list" => require_compatible(repo, runtime).and_then(|_| {
            let work_item_id = arguments
                .get("workItemId")
                .and_then(Value::as_str)
                .ok_or("workItemId argument is required")?;
            cockpit_repository::list_delegated_evidence(repo, work_item_id)
                .map_err(|error| error.to_string())
                .and_then(|receipts| {
                    serde_json::to_value(receipts).map_err(|error| error.to_string())
                })
        }),
        "preflight" => require_compatible(repo, runtime)
            .and_then(|_| preflight_for_repo(repo, &arguments, runtime)),
        "work_item_controls" => {
            require_compatible(repo, runtime).and_then(|_| work_item_controls(repo, &arguments))
        }
        "work_item_recover" => require_compatible(repo, runtime)
            .and_then(|_| work_item_recover(repo, &arguments, runtime)),
        "work_item_recover_selected_lineage" => require_compatible(repo, runtime)
            .and_then(|_| work_item_recover_selected_lineage(repo, &arguments, runtime)),
        "verify" => verify_for_repo(repo, &arguments, runtime),
        "work_item_parallel" => {
            require_compatible(repo, runtime).and_then(|_| work_item_parallel(repo, &arguments))
        }
        _ => unreachable!("tool names and dispatch must stay in sync"),
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

fn work_item_parallel(repo: &Path, arguments: &Value) -> Result<Value, String> {
    let action = arguments
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("inspect");
    match action {
        "inspect" => {
            let id = arguments
                .get("workItemId")
                .or_else(|| arguments.get("id"))
                .and_then(Value::as_str)
                .ok_or("workItemId argument is required for inspect")?;
            validate_id(id)?;
            let compatibility = cockpit_repository::work_item_compatibility(repo, id)
                .map_err(|error| error.to_string())?;
            let leases =
                cockpit_repository::list_parallel_slots(repo).map_err(|error| error.to_string())?;
            Ok(json!({
                "workItemId": id,
                "compatibility": compatibility,
                "leases": leases,
            }))
        }
        "acquire" => {
            let id = arguments
                .get("workItemId")
                .or_else(|| arguments.get("id"))
                .and_then(Value::as_str)
                .ok_or("workItemId argument is required for acquire")?;
            validate_id(id)?;
            let lease = cockpit_repository::acquire_parallel_slot(repo, id)
                .map_err(|error| error.to_string())?;
            serde_json::to_value(lease).map_err(|error| error.to_string())
        }
        "release" => {
            let id = arguments
                .get("workItemId")
                .or_else(|| arguments.get("id"))
                .and_then(Value::as_str)
                .ok_or("workItemId argument is required for release")?;
            let lease_id = arguments
                .get("leaseId")
                .and_then(Value::as_str)
                .ok_or("leaseId argument is required for release")?;
            validate_id(id)?;
            let lease = cockpit_repository::release_parallel_slot(repo, id, lease_id)
                .map_err(|error| error.to_string())?;
            serde_json::to_value(lease).map_err(|error| error.to_string())
        }
        "list" => {
            let leases =
                cockpit_repository::list_parallel_slots(repo).map_err(|error| error.to_string())?;
            Ok(json!({"leases": leases}))
        }
        _ => Err("action must be inspect, acquire, release, or list".into()),
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

fn work_item_controls(repo: &Path, arguments: &Value) -> Result<Value, String> {
    let work_item_id = arguments
        .get("workItemId")
        .or_else(|| arguments.get("id"))
        .and_then(Value::as_str)
        .ok_or("workItemId argument is required")?;
    validate_id(work_item_id)?;
    let controls = arguments
        .get("controls")
        .or_else(|| arguments.get("input"))
        .ok_or("controls argument is required")?;
    cockpit_repository::record_work_item_governance_controls(repo, work_item_id, controls)
        .map_err(|error| error.to_string())
}

fn work_item_start(
    repo: &Path,
    arguments: &Value,
    runtime: &cockpit_protocol::RuntimeContext,
) -> Result<Value, String> {
    let object = arguments
        .as_object()
        .ok_or("work_item_start arguments must be an object")?;
    let work_item_id = arguments
        .get("workItemId")
        .and_then(Value::as_str)
        .ok_or("workItemId argument is required")?;
    validate_id(work_item_id)?;
    let scope = parse_string_array(object, "scope", "work_item_start", true, 1)?;
    let out_of_scope = parse_string_array(object, "outOfScope", "work_item_start", false, 0)?;
    let acceptance_criteria =
        parse_string_array(object, "acceptanceCriteria", "work_item_start", false, 0)?;
    let required_evidence_classes = parse_string_array(
        object,
        "requiredEvidenceClasses",
        "work_item_start",
        false,
        0,
    )?;
    let sources = parse_string_array(object, "sources", "work_item_start", false, 0)?;
    let options = cockpit_repository::WorkItemStartOptions {
        out_of_scope,
        risk: arguments
            .get("risk")
            .and_then(Value::as_str)
            .unwrap_or("normal")
            .to_owned(),
        authority: arguments
            .get("authority")
            .and_then(Value::as_str)
            .unwrap_or("missing")
            .to_owned(),
        acceptance_criteria,
        required_evidence_classes,
    };
    let intent = arguments
        .get("intent")
        .and_then(Value::as_str)
        .ok_or("intent argument is required")?;
    let goal = arguments
        .get("goal")
        .and_then(Value::as_str)
        .ok_or("goal argument is required")?;
    cockpit_repository::start_work_item_prepared(
        repo,
        work_item_id,
        intent,
        goal,
        &scope,
        &options,
        &sources,
        runtime,
    )
    .map_err(|error| error.to_string())
}

fn work_item_recover(
    repo: &Path,
    arguments: &Value,
    runtime: &cockpit_protocol::RuntimeContext,
) -> Result<Value, String> {
    let work_item_id = arguments
        .get("workItemId")
        .or_else(|| arguments.get("id"))
        .and_then(Value::as_str)
        .ok_or("workItemId argument is required")?;
    validate_id(work_item_id)?;
    let receipt = arguments
        .get("receipt")
        .or_else(|| arguments.get("input"))
        .ok_or("receipt argument is required")?;
    cockpit_repository::record_recovery_decision(repo, work_item_id, receipt, runtime)
        .map_err(|error| error.to_string())
}

fn work_item_recover_selected_lineage(
    repo: &Path,
    arguments: &Value,
    runtime: &cockpit_protocol::RuntimeContext,
) -> Result<Value, String> {
    let work_item_id = arguments
        .get("workItemId")
        .or_else(|| arguments.get("id"))
        .and_then(Value::as_str)
        .ok_or("workItemId argument is required")?;
    validate_id(work_item_id)?;
    let receipt = arguments
        .get("receipt")
        .or_else(|| arguments.get("input"))
        .ok_or("receipt argument is required")?;
    cockpit_repository::record_selected_successor_lineage_recovery(
        repo,
        work_item_id,
        receipt,
        runtime,
    )
    .map_err(|error| error.to_string())
}

fn verify_for_repo(
    repo: &Path,
    arguments: &Value,
    runtime: &cockpit_protocol::RuntimeContext,
) -> Result<Value, String> {
    require_compatible(repo, runtime)?;
    let root = fs::canonicalize(repo).map_err(|error| error.to_string())?;
    let work_item_id = arguments.get("workItemId").and_then(Value::as_str);
    if let Some(work_item_id) = work_item_id {
        validate_id(work_item_id)?;
    }
    let plan_only = arguments
        .get("planOnly")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let timeout_seconds = arguments
        .get("timeoutSeconds")
        .map(|value| {
            value
                .as_u64()
                .ok_or("timeoutSeconds argument must be a positive integer".to_owned())
        })
        .transpose()?;
    let initial_snapshot = if work_item_id.is_some() || plan_only {
        Some(
            cockpit_git::GitRepository::discover(&root)
                .map_err(|error| error.to_string())?
                .snapshot()
                .map_err(|error| error.to_string())?,
        )
    } else {
        None
    };
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
    let request = cockpit_repository::RepositoryVerificationRequest {
        node_id: "project-command-0".into(),
        program,
        args,
        scope: vec!["**".into()],
        stage: "task".into(),
        runner: "local".into(),
        runtime_digest: runtime.runtime_digest.to_string(),
        base_commit: None,
        workers: 2,
        work_item_id: work_item_id.map(str::to_owned),
        timeout_seconds,
        policy: if explicit {
            cockpit_repository::RepositoryVerificationPolicy::NeverReuse
        } else {
            cockpit_repository::RepositoryVerificationPolicy::ProfileAuthorized
        },
    };
    if let (Some(work_item_id), Some(snapshot)) = (work_item_id, initial_snapshot.as_ref()) {
        let precondition = cockpit_repository::require_policy_for_verification(&root, work_item_id)
            .and_then(|_| {
                cockpit_repository::require_verification_preconditions(
                    &root,
                    work_item_id,
                    runtime,
                    snapshot,
                )
            });
        if let Err(error) = precondition {
            let diagnostic = error.to_string();
            let persistence = cockpit_repository::persist_verification_attempt(
                &root,
                work_item_id,
                std::slice::from_ref(&request),
                snapshot,
                runtime,
                "precondition_rejected",
                Some(("verification_preconditions", &diagnostic)),
                None,
            );
            let persistence_note = match persistence {
                Ok(attempt) => format!(
                    "; verification attempt persisted at {}",
                    attempt["path"].as_str().unwrap_or("unknown path")
                ),
                Err(persistence_error) => {
                    format!("; verification attempt persistence failed: {persistence_error}")
                }
            };
            return Err(format!(
                "verification preconditions rejected: {diagnostic}{persistence_note}"
            ));
        }
    }
    if plan_only {
        let snapshot = initial_snapshot
            .as_ref()
            .expect("plan-only verification captures a repository snapshot");
        let now_epoch_seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| i64::try_from(duration.as_secs()).unwrap_or(i64::MAX))
            .unwrap_or(0);
        let planned = match cockpit_repository::plan_repository_verification_action(
            &root,
            &request,
            snapshot,
            now_epoch_seconds,
        ) {
            Ok(planned) => planned,
            Err(error) => {
                let diagnostic = error.to_string();
                let attempt = if let Some(work_item_id) = work_item_id {
                    cockpit_repository::persist_verification_attempt(
                        &root,
                        work_item_id,
                        std::slice::from_ref(&request),
                        snapshot,
                        runtime,
                        "precondition_rejected",
                        Some(("verification_plan", &diagnostic)),
                        None,
                    )
                } else {
                    Ok(json!({"state":"not_applicable"}))
                };
                let persistence = match attempt {
                    Ok(attempt) => attempt,
                    Err(error) => {
                        json!({"state":"persistence_failed", "diagnostic":error.to_string()})
                    }
                };
                return Err(format!(
                    "verification planning rejected: {diagnostic}; attempt: {}",
                    serde_json::to_string(&persistence).unwrap_or_default()
                ));
            }
        };
        let reused = planned["action"] == "reuse";
        return Ok(json!({
            "state": "planned",
            "workItemId": work_item_id,
            "repositoryId": cockpit_repository::repository_id(&root).to_string(),
            "repositorySnapshotDigest": cockpit_repository::snapshot_digest(snapshot)
                .map_err(|error| error.to_string())?.to_string(),
            "runtimeVersion": runtime.runtime_version,
            "runtimeDigest": runtime.runtime_digest.to_string(),
            "nodesPlanned": 1,
            "nodesToExecute": usize::from(!reused),
            "nodesReused": usize::from(reused),
            "processesSpawned": 0,
            "timeoutSeconds": request.timeout_seconds.unwrap_or(
                cockpit_verification::DEFAULT_EXECUTION_SECONDS,
            ),
            "requests": [{
                "nodeId": request.node_id,
                "program": request.program,
                "args": request.args,
                "dependencies": [],
                "timeoutSeconds": request.timeout_seconds.unwrap_or(
                    cockpit_verification::DEFAULT_EXECUTION_SECONDS,
                ),
            }],
            "plannedNodes": [planned],
        }));
    }
    let run = cockpit_repository::run_repository_verification(&root, &request)
        .map_err(|error| error.to_string())?;
    let mut output = serde_json::to_value(&run.receipt).map_err(|error| error.to_string())?;
    output["runtimeVersion"] = Value::String(runtime.runtime_version.clone());
    output["runtimeDigest"] = Value::String(runtime.runtime_digest.to_string());
    if let Some(work_item_id) = work_item_id {
        output["workItemId"] = Value::String(work_item_id.into());
        output["repositoryId"] =
            Value::String(cockpit_repository::repository_id(&root).to_string());
    }
    if let Some(work_item_id) = work_item_id
        && let Err(error) = cockpit_repository::record_verification_with_runtime(
            &root,
            work_item_id,
            &output,
            runtime,
            &run.final_snapshot,
        )
    {
        let execution_succeeded = output["passed"] == Value::Bool(true);
        let diagnostic = error.to_string();
        let persistence = cockpit_repository::persist_verification_attempt(
            &root,
            work_item_id,
            std::slice::from_ref(&request),
            initial_snapshot
                .as_ref()
                .expect("repository-bound verification snapshot"),
            runtime,
            if execution_succeeded {
                "execution_completed"
            } else {
                "execution_failed"
            },
            Some((
                if execution_succeeded {
                    "verification_recording"
                } else {
                    "verification_execution"
                },
                &diagnostic,
            )),
            Some(&output),
        );
        let persistence_note = match persistence {
            Ok(attempt) => format!(
                "; verification attempt persisted at {}",
                attempt["path"].as_str().unwrap_or("unknown path")
            ),
            Err(persistence_error) => {
                format!("; verification attempt persistence failed: {persistence_error}")
            }
        };
        return Err(format!(
            "record verification evidence: {diagnostic}{persistence_note}"
        ));
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
    let language = requested_language(arguments);
    let delivery_requested = arguments
        .get("delivery")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let view = arguments
        .get("view")
        .and_then(Value::as_str)
        .map(|view| match view {
            "full" => cockpit_repository::OutcomeRenderView::Full,
            _ => cockpit_repository::OutcomeRenderView::Summary,
        })
        .unwrap_or(cockpit_repository::OutcomeRenderView::Summary);
    if delivery_requested {
        let mut delivery =
            cockpit_repository::prepare_archive_outcome_delivery(repo, id, runtime, language)
                .map_err(|error| error.to_string())?;
        delivery.delivery_state = "returned_to_consumer".into();
        delivery.host_confirmation = "unknown".into();
        let handoff = delivery.body.clone();
        let outcome = delivery
            .outcome
            .clone()
            .ok_or("archive Outcome delivery did not contain assembled Outcome facts")?;
        return Ok(json!({
            "workItemId": id,
            "outcome": outcome,
            "humanHandoff": handoff,
            "outcomeDelivery": delivery,
            "language": language,
            "contractLanguageBoundary": "Acceptance criteria remain in their original Contract language and are not machine-translated.",
            "hostDisplayConfirmation": "unknown"
        }));
    }
    let input = cockpit_repository::outcome_render_input_with_runtime(repo, id, runtime)
        .map_err(|error| error.to_string())?;
    let handoff = cockpit_repository::render_human_outcome_with_view(&input, language, view);
    Ok(json!({
        "workItemId": id,
        "outcome": input.outcome,
        "humanHandoff": handoff,
        "language": language,
        "contractLanguageBoundary": "Acceptance criteria remain in their original Contract language and are not machine-translated."
    }))
}

fn work_item_status(
    repo: &Path,
    arguments: &Value,
    runtime: &cockpit_protocol::RuntimeContext,
) -> Result<Value, String> {
    if arguments
        .get("all")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        let index = cockpit_repository::work_item_status_index_with_runtime(repo, runtime)
            .map_err(|error| error.to_string())?;
        return serde_json::to_value(index).map_err(|error| error.to_string());
    }
    let id = arguments
        .get("workItemId")
        .or_else(|| arguments.get("id"))
        .and_then(Value::as_str)
        .ok_or("workItemId argument is required")?;
    validate_id(id)?;
    let snapshot = cockpit_repository::work_item_status_snapshot_with_runtime(repo, id, runtime)
        .map_err(|error| error.to_string())?;
    serde_json::to_value(snapshot).map_err(|error| error.to_string())
}

fn work_item_validate(
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
    let report =
        cockpit_repository::validate_work_item_governance_controls_with_runtime(repo, id, runtime)
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

fn tool_error_response(id: Value, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [{"type": "text", "text": message}],
            "isError": true
        }
    })
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

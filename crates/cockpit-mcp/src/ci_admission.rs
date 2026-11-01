//! Thin, offline portable-evidence adapters. The repository service owns trust and admission.

use cockpit_protocol::RuntimeContext;
use serde_json::Value;
use std::path::Path;

pub(super) fn plan(
    repo: &Path,
    arguments: &Value,
    runtime: &RuntimeContext,
) -> Result<Value, String> {
    let caller = super::task_caller(arguments)?;
    let input = arguments["input"].as_str().ok_or("input is required")?;
    let work_item = arguments.get("workItemId").and_then(Value::as_str);
    cockpit_repository::plan_portable_evidence(
        repo,
        work_item,
        Path::new(input),
        runtime,
        caller.as_ref(),
    )
    .map_err(|error| error.to_string())
    .and_then(|plan| serde_json::to_value(plan).map_err(|error| error.to_string()))
}

pub(super) fn accept(
    repo: &Path,
    arguments: &Value,
    runtime: &RuntimeContext,
) -> Result<Value, String> {
    let caller = super::task_caller(arguments)?;
    let input = arguments["input"].as_str().ok_or("input is required")?;
    let work_item = arguments["workItemId"]
        .as_str()
        .ok_or("workItemId is required")?;
    let expected_plan_digest = arguments["expectedPlanDigest"]
        .as_str()
        .ok_or("expectedPlanDigest is required")?
        .parse::<cockpit_core::Digest>()
        .map_err(|error| format!("invalid expectedPlanDigest: {error}"))?;
    let result = if let Some(parameters) = arguments.get("sourceRequest") {
        let tool = cockpit_repository::PortableSourceTool::for_current_runtime(repo, runtime)
            .map_err(|e| e.to_string())?;
        cockpit_repository::accept_portable_evidence_with_source(
            repo,
            cockpit_repository::PortableSourceAdmissionRequest {
                work_item_id: work_item,
                input: Path::new(input),
                expected_plan_digest: &expected_plan_digest,
                parameters,
                node_id: arguments.get("nodeId").and_then(Value::as_str),
            },
            &tool,
            runtime,
            caller.as_ref(),
        )
    } else {
        if arguments.get("nodeId").is_some() {
            return Err("nodeId requires sourceRequest".into());
        }
        cockpit_repository::accept_portable_evidence(
            repo,
            work_item,
            Path::new(input),
            &expected_plan_digest,
            runtime,
            caller.as_ref(),
        )
    };
    result
        .map_err(|error| error.to_string())
        .and_then(|acceptance| serde_json::to_value(acceptance).map_err(|error| error.to_string()))
}

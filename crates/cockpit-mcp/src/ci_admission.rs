//! Thin, read-only portable-evidence inspection adapter. Supplied data remains untrusted.

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

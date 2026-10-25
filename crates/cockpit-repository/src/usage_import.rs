use super::{ObserverError, reject_duplicate_json_keys};
use cockpit_protocol::{
    RuntimeContext, UsageImportRequest, UsageReceipt, UsageRecordRequest, UsageSourceKind,
    UsageUnit,
};
use serde_json::Value;
use std::path::Path;

fn invalid(path: &str, message: impl Into<String>) -> ObserverError {
    ObserverError::State {
        path: path.into(),
        message: message.into(),
    }
}

fn count(value: Option<&Value>, path: &str) -> Result<Option<u64>, ObserverError> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_u64()
            .map(Some)
            .ok_or_else(|| invalid(path, "usage counter must be an unsigned integer or null")),
    }
}

fn detail<'a>(
    usage: &'a Value,
    name: &str,
    path: &str,
) -> Result<Option<&'a Value>, ObserverError> {
    match usage.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(value) if value.is_object() => Ok(Some(value)),
        Some(_) => Err(invalid(
            path,
            "usage counter details must be an object or null",
        )),
    }
}

fn resolve(
    import: &UsageImportRequest,
    source: &[u8],
) -> Result<UsageRecordRequest, ObserverError> {
    reject_duplicate_json_keys(source).map_err(|_| {
        invalid(
            &import.source_ref,
            "usage source JSON is malformed or contains duplicate keys",
        )
    })?;
    let value: Value = serde_json::from_slice(source).map_err(|_| {
        invalid(
            &import.source_ref,
            "usage source is not a single JSON response object",
        )
    })?;
    if value.get("object").and_then(Value::as_str) != Some("response")
        || value.get("status").and_then(Value::as_str) != Some("completed")
    {
        return Err(invalid(
            &import.source_ref,
            "usage source must be a completed response object",
        ));
    }
    let id = value
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .ok_or_else(|| {
            invalid(
                &import.source_ref,
                "usage response has no stable response id",
            )
        })?;
    let usage = value.get("usage").unwrap_or(&Value::Null);
    if !usage.is_null() && !usage.is_object() {
        return Err(invalid(
            &import.source_ref,
            "usage must be an object or null",
        ));
    }
    let input = count(usage.get("input_tokens"), &import.source_ref)?;
    let output = count(usage.get("output_tokens"), &import.source_ref)?;
    if let Some(total) = count(usage.get("total_tokens"), &import.source_ref)? {
        let actual = input.zip(output).and_then(|(a, b)| a.checked_add(b));
        if actual != Some(total) {
            return Err(invalid(
                &import.source_ref,
                "usage total differs from input plus output",
            ));
        }
    }
    let model = match value.get("model") {
        None | Some(Value::Null) => None,
        Some(Value::String(model)) => Some(model.clone()),
        Some(_) => {
            return Err(invalid(
                &import.source_ref,
                "usage response model must be a string or null",
            ));
        }
    };
    let result = UsageRecordRequest {
        schema_version: import.schema_version,
        repository_id: import.repository_id.clone(),
        work_item_id: import.work_item_id.clone(),
        source_event_id: format!("openai-response:{id}"),
        source_kind: UsageSourceKind::ProviderReported,
        actor: import.actor.clone(),
        configured_model: import.configured_model.clone(),
        reported_model: model,
        role: import.role.clone(),
        phase: import.phase.clone(),
        unit: UsageUnit::Invocation,
        input_tokens: input,
        output_tokens: output,
        cached_input_tokens: count(
            detail(usage, "input_tokens_details", &import.source_ref)?
                .and_then(|value| value.get("cached_tokens")),
            &import.source_ref,
        )?,
        reasoning_tokens: count(
            detail(usage, "output_tokens_details", &import.source_ref)?
                .and_then(|value| value.get("reasoning_tokens")),
            &import.source_ref,
        )?,
        source_observed_at: None,
        evidence_ref: import.source_ref.clone(),
        evidence_digest: import.source_digest.clone(),
    };
    result
        .validate()
        .map_err(|message| invalid(&import.source_ref, message))?;
    Ok(result)
}

/// Parse supplied bytes under the same lock/read/admission boundary as explicit usage records.
/// Source labels and hashes never promote the result to authenticated metering.
pub fn import_work_item_usage(
    root: &Path,
    request: &UsageImportRequest,
    runtime: &RuntimeContext,
) -> Result<UsageReceipt, ObserverError> {
    request
        .validate()
        .map_err(|message| invalid(&request.source_ref, message))?;
    let context = UsageRecordRequest {
        schema_version: request.schema_version,
        repository_id: request.repository_id.clone(),
        work_item_id: request.work_item_id.clone(),
        source_event_id: "usage-import-context".into(),
        source_kind: UsageSourceKind::ProviderReported,
        actor: request.actor.clone(),
        configured_model: request.configured_model.clone(),
        reported_model: None,
        role: request.role.clone(),
        phase: request.phase.clone(),
        unit: UsageUnit::Invocation,
        input_tokens: None,
        output_tokens: None,
        cached_input_tokens: None,
        reasoning_tokens: None,
        source_observed_at: None,
        evidence_ref: request.source_ref.clone(),
        evidence_digest: request.source_digest.clone(),
    };
    super::usage::record_usage_from_source(root, &context, runtime, |source| {
        resolve(request, source)
    })
}

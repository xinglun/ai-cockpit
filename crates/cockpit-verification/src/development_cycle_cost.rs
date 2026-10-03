//! Pure development-cycle cost report. Elapsed time, agent operations, and
//! preflight rejections remain independent observed dimensions.

use serde_json::{Value, json};

pub const STAGES: [&str; 3] = [
    "contract_to_reviewable",
    "verification_to_finish",
    "post_merge_cleanup",
];

fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(value) => value.as_f64().is_some_and(|value| value != 0.0),
        Value::String(value) => !value.is_empty(),
        Value::Array(value) => !value.is_empty(),
        Value::Object(value) => !value.is_empty(),
    }
}

fn sample(value: Option<&Value>) -> Result<f64, String> {
    let number = value
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && *value >= 0.0)
        .ok_or("cycle samples must be finite non-negative numbers")?;
    // Formatting rounds the original binary float to three decimal places.
    // Scaling first changes the rounding boundary for values such as 0.0005.
    format!("{number:.3}")
        .parse::<f64>()
        .map_err(|_| "cycle samples must be finite non-negative numbers".into())
}

fn count_binding(values: &[Value], reason: &str) -> Value {
    if values.iter().all(|value| value.as_u64().is_some()) {
        json!({"available": true, "values": values})
    } else {
        json!({"available": false, "reason": reason})
    }
}

fn nearest_rank(values: &[f64], fraction: f64) -> f64 {
    let mut ordered = values.to_vec();
    ordered.sort_by(f64::total_cmp);
    let index = ((fraction * ordered.len() as f64).ceil() as usize).saturating_sub(1);
    ordered[index]
}

fn stage_report(stage: &str, records: &[Value]) -> Result<Value, String> {
    if records.is_empty() {
        return Ok(json!({
            "stage": stage,
            "sourceRecords": [],
            "available": false,
            "reason": "stage_not_captured",
            "rawSamplesMs": [],
            "rawSampleCount": 0,
            "p50Ms": null,
            "p95Ms": null,
            "agentOperations": [],
            "preflightRejects": [],
            "agentOperationsBinding": {"available": false, "reason": "stage_not_captured"},
            "preflightRejectsBinding": {"available": false, "reason": "stage_not_captured"}
        }));
    }
    let mut raw = Vec::new();
    let mut agent_operations = Vec::new();
    let mut preflight_rejects = Vec::new();
    for record in records {
        if record.get("valid").is_none_or(truthy) {
            raw.push(sample(record.get("elapsedMs"))?);
        }
        agent_operations.push(
            record
                .get("agentOperations")
                .cloned()
                .unwrap_or(Value::Null),
        );
        preflight_rejects.push(
            record
                .get("preflightRejects")
                .cloned()
                .unwrap_or(Value::Null),
        );
    }
    let agent_binding = count_binding(
        &agent_operations,
        "agent operation count not persisted for one or more captures",
    );
    let reject_binding = count_binding(
        &preflight_rejects,
        "preflight rejection count not persisted for one or more captures",
    );
    let (available, reason, p50, p95) = if raw.is_empty() {
        (false, Some("no_valid_stage_samples"), None, None)
    } else {
        (
            true,
            None,
            Some(nearest_rank(&raw, 0.50)),
            Some(nearest_rank(&raw, 0.95)),
        )
    };
    let mut result = json!({
        "stage": stage,
        "sourceRecords": records,
        "rawSamplesMs": raw,
        "rawSampleCount": raw.len(),
        "agentOperations": agent_operations,
        "preflightRejects": preflight_rejects,
        "agentOperationsBinding": agent_binding,
        "preflightRejectsBinding": reject_binding,
        "available": available,
        "p50Ms": p50,
        "p95Ms": p95
    });
    if let Some(reason) = reason {
        result["reason"] = json!(reason);
    }
    Ok(result)
}

pub fn build_report(document: &Value) -> Result<Value, String> {
    let environment = document
        .get("environment")
        .filter(|value| value.is_object())
        .ok_or("environment identity is required")?;
    let captures = match document.get("captures") {
        None => &[][..],
        Some(Value::Array(captures)) => captures.as_slice(),
        Some(_) => return Err("captures must be an array".into()),
    };
    let mut grouped: [Vec<Value>; 3] = std::array::from_fn(|_| Vec::new());
    for record in captures {
        let stage = record
            .get("stage")
            .and_then(Value::as_str)
            .ok_or("each capture must name one known stage")?;
        let index = STAGES
            .iter()
            .position(|known| *known == stage)
            .ok_or("each capture must name one known stage")?;
        grouped[index].push(record.clone());
    }
    let stages = STAGES
        .iter()
        .enumerate()
        .map(|(index, stage)| stage_report(stage, &grouped[index]))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(json!({
        "schemaVersion": 1,
        "kind": "development-cycle-cost",
        "environment": environment,
        "stages": stages,
        "measurementBoundary": {
            "contractToReviewable": "from Contract creation to the first state where a Draft PR may be created",
            "verificationToFinish": "from verification start to a successful finish boundary",
            "postMergeCleanup": "from reviewed merge observation to exact resource cleanup",
            "overlap": "not inferred; each stage is reported from its own capture"
        }
    }))
}

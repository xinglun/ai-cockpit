use cockpit_protocol::{
    Contract, ContractAmendmentChange, ContractAmendmentError, ContractAmendmentOperation,
    ContractAmendmentRequest, ContractSource, VerificationDeclaration, apply_contract_amendment,
    digest_json,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const TYPED_REQUEST_FIELDS: [&str; 5] = [
    "schemaVersion",
    "changeId",
    "expectedContractDigest",
    "reason",
    "changes",
];

pub(crate) fn typed_request_from_value(
    input: &Value,
) -> Result<Option<ContractAmendmentRequest>, serde_json::Error> {
    let is_typed = input.as_object().is_some_and(|object| {
        TYPED_REQUEST_FIELDS
            .iter()
            .any(|field| object.contains_key(*field))
    });
    if is_typed {
        serde_json::from_value(input.clone()).map(Some)
    } else {
        Ok(None)
    }
}

pub(crate) fn legacy_request_from_value(
    current: &Value,
    input: &Value,
    reason: &str,
) -> Result<ContractAmendmentRequest, String> {
    let object = input
        .as_object()
        .ok_or_else(|| "legacy amendment input must be a JSON object".to_string())?;
    let legacy_fields = [
        "scopeAppend",
        "outOfScopeAppend",
        "sourcesAppend",
        "verificationAppend",
        "acceptanceAppend",
        "requiredEvidenceClassesAppend",
        "scenarioCoverageAppend",
        "scenarioCoveragePlanAppend",
    ];
    if let Some(field) = object
        .keys()
        .find(|field| !legacy_fields.contains(&field.as_str()))
    {
        return Err(format!("unsupported Contract amendment field {field}"));
    }
    let normalized_contract: Contract = serde_json::from_value(current.clone())
        .map_err(|error| format!("current Contract is invalid: {error}"))?;
    let mut normalized = serde_json::to_value(normalized_contract)
        .map_err(|error| format!("current Contract cannot be serialized: {error}"))?;
    let expected_contract_digest =
        digest_json(current).map_err(|error| format!("current Contract digest failed: {error}"))?;
    let mut changes = Vec::new();
    let mut seen = BTreeMap::<String, Vec<Value>>::new();

    for (field, target) in [
        ("scopeAppend", "scope"),
        ("outOfScopeAppend", "outOfScope"),
        ("acceptanceAppend", "acceptanceCriteria"),
        ("requiredEvidenceClassesAppend", "requiredEvidenceClasses"),
    ] {
        let Some(values) = object.get(field) else {
            continue;
        };
        let values = values
            .as_array()
            .ok_or_else(|| format!("{field} must be an array"))?;
        let existing = normalized[target]
            .as_array()
            .ok_or_else(|| format!("Contract field {target} is not an array"))?;
        let seen_values = seen
            .entry(target.into())
            .or_insert_with(|| existing.clone());
        for value in values {
            if value.as_str().is_none_or(|value| value.trim().is_empty()) {
                return Err(format!("{field} entries must be non-empty strings"));
            }
            if !seen_values.contains(value) {
                seen_values.push(value.clone());
                changes.push(ContractAmendmentChange {
                    path: format!("/{target}"),
                    operation: ContractAmendmentOperation::Add,
                    value: Some(value.clone()),
                });
            }
        }
    }

    for (field, target) in [
        ("sourcesAppend", "sources"),
        ("verificationAppend", "verification"),
    ] {
        let Some(values) = object.get(field) else {
            continue;
        };
        let values = values
            .as_array()
            .ok_or_else(|| format!("{field} must be an array"))?;
        let existing = normalized[target]
            .as_array()
            .ok_or_else(|| format!("Contract field {target} is not an array"))?;
        let seen_values = seen
            .entry(target.into())
            .or_insert_with(|| existing.clone());
        for value in values {
            let valid_non_empty = match field {
                "sourcesAppend" => {
                    serde_json::from_value::<ContractSource>(value.clone()).map(|source| {
                        match source {
                            ContractSource::Legacy(source) => !source.trim().is_empty(),
                            ContractSource::Structured(source) => {
                                !source.path.trim().is_empty() && !source.reason.trim().is_empty()
                            }
                        }
                    })
                }
                "verificationAppend" => serde_json::from_value::<VerificationDeclaration>(
                    value.clone(),
                )
                .map(|declaration| match declaration {
                    VerificationDeclaration::Legacy(command) => !command.trim().is_empty(),
                    VerificationDeclaration::Check(check) => !check.check.trim().is_empty(),
                }),
                _ => unreachable!(),
            }
            .map_err(|error| format!("{field} contains an invalid declaration: {error}"))?;
            if !valid_non_empty {
                return Err(format!("{field} entries must not be empty"));
            }
            if !seen_values.contains(value) {
                seen_values.push(value.clone());
                changes.push(ContractAmendmentChange {
                    path: format!("/{target}"),
                    operation: ContractAmendmentOperation::Add,
                    value: Some(value.clone()),
                });
            }
        }
    }

    if let Some(values) = object.get("scenarioCoverageAppend") {
        let values = values
            .as_array()
            .ok_or_else(|| "scenarioCoverageAppend must be an array".to_string())?;
        let mut coverage = normalized
            .get("scenarioCoverage")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let mut names = coverage
            .iter()
            .filter_map(|entry| {
                entry
                    .get("scenario")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
            .collect::<Vec<_>>();
        for value in values {
            let name = value
                .get("scenario")
                .and_then(Value::as_str)
                .filter(|name| !name.trim().is_empty())
                .ok_or_else(|| {
                    "scenarioCoverageAppend entries must contain a non-empty scenario string"
                        .to_string()
                })?;
            if names.iter().any(|existing| existing == name) {
                return Err(format!(
                    "scenarioCoverageAppend contains duplicate scenario {name:?}"
                ));
            }
            names.push(name.into());
            changes.push(ContractAmendmentChange {
                path: "/scenarioCoverage".into(),
                operation: ContractAmendmentOperation::Add,
                value: Some(value.clone()),
            });
            coverage.push(value.clone());
        }
        normalized["scenarioCoverage"] = json!(coverage);
    }

    if let Some(values) = object.get("scenarioCoveragePlanAppend") {
        let values = values
            .as_array()
            .ok_or_else(|| "scenarioCoveragePlanAppend must be an array".to_string())?;
        let mut coverage = normalized
            .get("scenarioCoverage")
            .and_then(Value::as_array)
            .cloned()
            .ok_or_else(|| {
                "scenarioCoveragePlanAppend requires an existing scenarioCoverage array".to_string()
            })?;
        let mut changed_names = Vec::new();
        for value in values {
            let name = value
                .get("scenario")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    "scenarioCoveragePlanAppend entries must contain a scenario string".to_string()
                })?;
            let plan = value
                .get("verificationPlan")
                .and_then(Value::as_str)
                .filter(|plan| !plan.trim().is_empty())
                .ok_or_else(|| {
                    "scenarioCoveragePlanAppend verificationPlan must be non-empty".to_string()
                })?;
            let (index, existing) = coverage
                .iter()
                .enumerate()
                .find(|(_, entry)| entry.get("scenario").and_then(Value::as_str) == Some(name))
                .ok_or_else(|| {
                    format!("scenarioCoveragePlanAppend names no existing scenario {name:?}")
                })?;
            if existing
                .get("verificationPlan")
                .and_then(Value::as_str)
                .is_some()
                || changed_names.iter().any(|changed| changed == name)
            {
                return Err(format!(
                    "scenarioCoveragePlanAppend cannot replace an existing verificationPlan for {name:?}"
                ));
            }
            changes.push(ContractAmendmentChange {
                path: format!("/scenarioCoverage/{index}/verificationPlan"),
                operation: ContractAmendmentOperation::Set,
                value: Some(json!(plan)),
            });
            coverage[index]["verificationPlan"] = json!(plan);
            changed_names.push(name.to_string());
        }
    }

    let request_seed = json!({
        "contractDigest": expected_contract_digest,
        "input": input,
        "reason": reason
    });
    let change_digest = digest_json(&request_seed)
        .map_err(|error| format!("legacy change identity failed: {error}"))?;
    let change_id = format!("legacy-{}", &change_digest.as_str()[7..]);
    Ok(ContractAmendmentRequest {
        schema_version: 1,
        change_id,
        expected_contract_digest,
        reason: reason.into(),
        changes,
    })
}

pub(crate) fn apply_typed_amendment(
    current: &Value,
    request: &ContractAmendmentRequest,
) -> Result<Value, Vec<ContractAmendmentError>> {
    let current_digest = digest_json(current).map_err(|source| {
        vec![ContractAmendmentError {
            code: "contract_digest_failed".into(),
            path: String::new(),
            message: source.to_string(),
        }]
    })?;
    if current_digest != request.expected_contract_digest {
        return Err(vec![ContractAmendmentError {
            code: "contract_digest_conflict".into(),
            path: "/expectedContractDigest".into(),
            message: format!(
                "expected {}, current Contract digest is {}",
                request.expected_contract_digest, current_digest
            ),
        }]);
    }
    let contract: Contract = serde_json::from_value(current.clone()).map_err(|source| {
        vec![ContractAmendmentError {
            code: "contract_schema_invalid".into(),
            path: String::new(),
            message: source.to_string(),
        }]
    })?;
    let amended = apply_contract_amendment(&contract, request)?;
    serde_json::to_value(amended).map_err(|source| {
        vec![ContractAmendmentError {
            code: "contract_serialize_failed".into(),
            path: String::new(),
            message: source.to_string(),
        }]
    })
}

pub(crate) fn amendment_error_message(errors: &[ContractAmendmentError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

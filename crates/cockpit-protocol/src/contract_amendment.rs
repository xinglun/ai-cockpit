use super::Contract;
use cockpit_core::Digest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const CONTRACT_AMENDMENT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractAmendmentRequest {
    pub schema_version: u32,
    pub change_id: String,
    pub expected_contract_digest: Digest,
    pub reason: String,
    pub changes: Vec<ContractAmendmentChange>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractAmendmentChange {
    pub path: String,
    pub operation: ContractAmendmentOperation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractAmendmentOperation {
    Add,
    Set,
    Clear,
    Remove,
    Replace,
    Reorder,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractAmendmentFieldClass {
    PlanEditable,
    SensitivePlanEditable,
}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[error("{code} at {path}: {message}")]
pub struct ContractAmendmentError {
    pub code: String,
    pub path: String,
    pub message: String,
}

fn error(code: &str, path: &str, message: impl Into<String>) -> ContractAmendmentError {
    ContractAmendmentError {
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}

fn errors(error: ContractAmendmentError) -> Vec<ContractAmendmentError> {
    vec![error]
}

fn parse_pointer(path: &str) -> Result<Vec<String>, ContractAmendmentError> {
    if !path.starts_with('/') || path == "/" {
        return Err(error(
            "invalid_path",
            path,
            "path must be a non-root canonical JSON Pointer",
        ));
    }
    let mut decoded = Vec::new();
    for segment in path[1..].split('/') {
        if segment.is_empty() {
            return Err(error(
                "invalid_path",
                path,
                "empty JSON Pointer segments are not supported",
            ));
        }
        let mut value = String::new();
        let mut chars = segment.chars();
        while let Some(ch) = chars.next() {
            if ch != '~' {
                value.push(ch);
                continue;
            }
            match chars.next() {
                Some('0') => value.push('~'),
                Some('1') => value.push('/'),
                _ => {
                    return Err(error(
                        "invalid_path",
                        path,
                        "JSON Pointer escapes must use ~0 or ~1",
                    ));
                }
            }
        }
        if escape_pointer_segment(&value) != segment {
            return Err(error(
                "invalid_path",
                path,
                "path is not in canonical JSON Pointer form",
            ));
        }
        if value == "-" {
            return Err(error(
                "invalid_path",
                path,
                "append indices are not accepted; target the collection itself",
            ));
        }
        if value.bytes().all(|byte| byte.is_ascii_digit())
            && value.len() > 1
            && value.starts_with('0')
        {
            return Err(error(
                "invalid_path",
                path,
                "array indices must not contain leading zeroes",
            ));
        }
        decoded.push(value);
    }
    Ok(decoded)
}

fn escape_pointer_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

fn field_class(field: &str) -> Option<ContractAmendmentFieldClass> {
    use ContractAmendmentFieldClass::{PlanEditable as Plan, SensitivePlanEditable as Sensitive};
    match field {
        // Human-authored intent and implementation-plan declarations.
        "title"
        | "intent"
        | "goal"
        | "problemStatement"
        | "sources"
        | "guidelines"
        | "rollbackNote"
        | "rollbackPlan"
        | "unknowns"
        | "humanDecisionPoints"
        | "documentationImpact"
        | "performanceImpact"
        | "residualRiskExpectation"
        | "implementationSurface"
        | "adoptionBootstrapPaths"
        | "requestedOperation"
        | "operation" => Some(Plan),
        // These are editable plan decisions, but they affect authorization,
        // policy, verification strength, or the set of admissible actions.
        "mode"
        | "scope"
        | "outOfScope"
        | "risk"
        | "authority"
        | "acceptanceCriteria"
        | "requiredEvidenceClasses"
        | "verification"
        | "riskAssessment"
        | "agentCapability"
        | "executionDecision"
        | "destructiveChangePolicy"
        | "governancePolicy"
        | "notCodable"
        | "scenarioCoverage"
        | "concurrencyBoundary"
        | "checkpointPolicy"
        | "governanceProfile" => Some(Sensitive),
        _ => None,
    }
}

fn protected_or_unknown_field_error(
    path: &str,
    segments: &[String],
) -> Option<ContractAmendmentError> {
    let root = segments.first().map(String::as_str).unwrap_or_default();
    if root == "acceptance" {
        return Some(error(
            "derived_field",
            path,
            "acceptance is a derived alias; amend acceptanceCriteria instead",
        ));
    }
    if field_class(root).is_none() {
        let protected = matches!(
            root,
            "protocolVersion"
                | "contractVersion"
                | "repositoryId"
                | "workItemId"
                | "state"
                | "createdAt"
                | "baseRevision"
                | "baseCommit"
                | "projectProfileDigest"
                | "repositorySnapshotDigest"
                | "resourceContext"
                | "baselineDirtyPaths"
                | "archiveSequence"
                | "resumeHistory"
                | "predecessorWorkItemId"
                | "predecessorContractDigest"
                | "recoveryDecisionPath"
                | "synchronizationCheckpoint"
                | "synchronizationHistory"
                | "preReviewWarnings"
                | "authorityEvidence"
                | "restrictedWriteApproval"
        );
        return Some(if protected {
            error(
                "protected_field",
                path,
                "Runtime identity, observed facts, lifecycle state, and immutable evidence cannot be amended",
            )
        } else {
            error(
                "unknown_field",
                path,
                format!("Contract field {root:?} is not in the amendment registry"),
            )
        });
    }

    if root == "scenarioCoverage" && segments.len() >= 2 {
        if segments.len() == 2 {
            return Some(error(
                "protected_field",
                path,
                "scenario entries must be changed through their declared plan fields or collection operations",
            ));
        }
        if matches!(segments[2].as_str(), "scenario" | "status" | "evidence") {
            return Some(error(
                "protected_field",
                path,
                "scenario identity and verification status/evidence are not plan-editable",
            ));
        }
        if !matches!(
            segments[2].as_str(),
            "required"
                | "reason"
                | "expected"
                | "expectedResult"
                | "expectedOutcome"
                | "verificationPlan"
                | "description"
        ) {
            return Some(error(
                "unknown_field",
                path,
                "scenario field is not in the amendment registry",
            ));
        }
    }
    if root == "destructiveChangePolicy" {
        if segments.len() == 1 {
            return Some(error(
                "protected_field",
                path,
                "amend policy declarations field-by-field so approval evidence remains immutable",
            ));
        }
        if matches!(
            segments[1].as_str(),
            "approvalEvidence" | "identityEvidence"
        ) {
            return Some(error(
                "protected_field",
                path,
                "approval evidence cannot be rewritten by a Contract amendment",
            ));
        }
    }
    if root == "checkpointPolicy"
        && segments
            .get(1)
            .is_some_and(|field| field == "schemaVersion")
    {
        return Some(error(
            "protected_field",
            path,
            "the policy schema version is protocol metadata, not an editable plan value",
        ));
    }
    None
}

/// Classify one canonical Contract JSON Pointer. Runtime-owned and unknown
/// paths are errors, not an implicit third mutable class.
pub fn contract_amendment_field_class(
    path: &str,
) -> Result<ContractAmendmentFieldClass, ContractAmendmentError> {
    let segments = parse_pointer(path)?;
    if let Some(error) = protected_or_unknown_field_error(path, &segments) {
        return Err(error);
    }
    let mut class = field_class(&segments[0])
        .ok_or_else(|| error("unknown_field", path, "Contract field is not amendable"))?;
    if segments
        .first()
        .is_some_and(|root| root == "scenarioCoverage")
        && segments.get(2).is_some_and(|field| field == "required")
    {
        class = ContractAmendmentFieldClass::SensitivePlanEditable;
    }
    Ok(class)
}

fn clearable_path(segments: &[String]) -> bool {
    let root = segments[0].as_str();
    match segments {
        [_] => matches!(
            root,
            "mode"
                | "title"
                | "operation"
                | "governancePolicy"
                | "problemStatement"
                | "riskAssessment"
                | "agentCapability"
                | "executionDecision"
                | "rollbackNote"
                | "rollbackPlan"
                | "notCodable"
                | "scenarioCoverage"
                | "concurrencyBoundary"
                | "checkpointPolicy"
                | "humanDecisionPoints"
                | "documentationImpact"
                | "performanceImpact"
                | "residualRiskExpectation"
                | "governanceProfile"
                | "requestedOperation"
                | "implementationSurface"
        ),
        [root, field] if root == "intent" => matches!(
            field.as_str(),
            "businessGoal" | "userGoal" | "problem" | "rationale"
        ),
        [root, _, field] if root == "scenarioCoverage" => matches!(
            field.as_str(),
            "reason"
                | "expected"
                | "expectedResult"
                | "expectedOutcome"
                | "verificationPlan"
                | "description"
        ),
        [root, field] if root == "riskAssessment" => field == "reason",
        [root, field] if root == "agentCapability" => field == "blockedReason",
        _ => false,
    }
}

fn pointer_for(segments: &[String]) -> String {
    segments
        .iter()
        .map(|segment| format!("/{}", escape_pointer_segment(segment)))
        .collect()
}

fn assign_existing_or_optional(
    document: &mut Value,
    path: &str,
    segments: &[String],
    value: Value,
) -> Result<(), ContractAmendmentError> {
    if let Some(target) = document.pointer_mut(path) {
        *target = value;
        return Ok(());
    }
    if !clearable_path(segments) {
        return Err(error(
            "missing_path",
            path,
            "set target must be an existing typed field",
        ));
    }
    let (field, parent_segments) = segments
        .split_last()
        .expect("validated non-empty pointer segments");
    let parent_path = pointer_for(parent_segments);
    let parent = if parent_path.is_empty() {
        document
    } else {
        document.pointer_mut(&parent_path).ok_or_else(|| {
            error(
                "missing_parent",
                path,
                "optional field parent does not exist",
            )
        })?
    };
    let object = parent.as_object_mut().ok_or_else(|| {
        error(
            "invalid_parent",
            path,
            "optional field parent must be a typed object",
        )
    })?;
    object.insert(field.clone(), value);
    Ok(())
}

fn stable_identity(collection_path: &str, value: &Value) -> Result<String, String> {
    let field = collection_path
        .rsplit('/')
        .next()
        .unwrap_or(collection_path);
    let identity = match field {
        "sources" => value
            .as_str()
            .map(|value| format!("legacy:{value}"))
            .or_else(|| {
                value
                    .get("path")
                    .and_then(Value::as_str)
                    .map(|value| format!("path:{value}"))
            }),
        "verification" => value
            .as_str()
            .map(|value| format!("legacy:{value}"))
            .or_else(|| {
                value
                    .get("check")
                    .and_then(Value::as_str)
                    .map(|value| format!("check:{value}"))
            }),
        "scenarioCoverage" => value
            .get("scenario")
            .and_then(Value::as_str)
            .map(|value| format!("scenario:{value}")),
        "rules" => value
            .get("operation")
            .and_then(Value::as_str)
            .map(|value| format!("operation:{value}")),
        _ => value.as_str().map(|value| format!("value:{value}")),
    };
    identity.ok_or_else(|| {
        format!("collection {collection_path} has an element without a stable typed identity")
    })
}

fn known_collection(field: &str) -> bool {
    matches!(
        field,
        "scope"
            | "outOfScope"
            | "acceptanceCriteria"
            | "requiredEvidenceClasses"
            | "sources"
            | "verification"
            | "scenarioCoverage"
            | "guidelines"
            | "unknowns"
            | "adoptionBootstrapPaths"
            | "constraints"
            | "nonGoals"
            | "coversScenarios"
            | "coversConstraints"
            | "riskTypes"
            | "implementationPaths"
            | "generatedEvidencePaths"
            | "verificationOutputPaths"
            | "serializedProjectionPaths"
            | "requiredStages"
            | "requiredChecks"
            | "allowPatterns"
            | "rules"
    )
}

fn validate_changed_collection_identities(
    document: &Value,
    segments: &[String],
    change_path: &str,
) -> Result<(), ContractAmendmentError> {
    for length in 1..=segments.len() {
        let prefix = &segments[..length];
        let collection_path = pointer_for(prefix);
        let field = prefix.last().map(String::as_str).unwrap_or_default();
        if !known_collection(field) {
            continue;
        }
        let Some(values) = document.pointer(&collection_path).and_then(Value::as_array) else {
            continue;
        };
        let mut identities = BTreeSet::new();
        for value in values {
            let identity = stable_identity(&collection_path, value)
                .map_err(|message| error("invalid_identity", change_path, message))?;
            if !identities.insert(identity) {
                return Err(error(
                    "duplicate_identity",
                    change_path,
                    format!("{collection_path} contains duplicate stable identities"),
                ));
            }
        }
    }
    Ok(())
}

fn amendment_operation(
    document: &mut Value,
    change: &ContractAmendmentChange,
    segments: &[String],
) -> Result<(), ContractAmendmentError> {
    let path = &change.path;
    let current = document.pointer(path).cloned();
    let fail = |code: &str, message: &str| error(code, path, message);
    match change.operation {
        ContractAmendmentOperation::Clear => {
            if change.value.is_some() {
                return Err(fail("unexpected_value", "clear does not accept a value"));
            }
            if !clearable_path(segments) {
                return Err(fail(
                    "not_clearable",
                    "clear is allowed only for registered optional plan fields",
                ));
            }
            let target = document
                .pointer_mut(path)
                .ok_or_else(|| fail("missing_path", "clear target does not exist"))?;
            *target = Value::Null;
        }
        ContractAmendmentOperation::Set => {
            let value = change
                .value
                .as_ref()
                .ok_or_else(|| fail("missing_value", "set requires a non-null value"))?;
            if value.is_null() {
                return Err(fail("null_value", "use clear for optional Contract fields"));
            }
            if current
                .as_ref()
                .is_some_and(|value| value.is_array() || value.is_object())
            {
                return Err(fail(
                    "set_requires_leaf",
                    "set assigns a scalar leaf; use replace for an existing structured value",
                ));
            }
            assign_existing_or_optional(document, path, segments, value.clone())?;
        }
        ContractAmendmentOperation::Replace => {
            let value = change
                .value
                .as_ref()
                .ok_or_else(|| fail("missing_value", "replace requires a non-null value"))?;
            if value.is_null() {
                return Err(fail("null_value", "use clear for optional Contract fields"));
            }
            if current.is_none() {
                return Err(fail("missing_path", "replace target must already exist"));
            }
            if segments
                .first()
                .is_some_and(|root| root == "scenarioCoverage")
                && segments.len() == 1
            {
                return Err(fail(
                    "protected_field",
                    "replace individual scenario plan fields; collection replacement could rewrite status/evidence",
                ));
            }
            *document
                .pointer_mut(path)
                .ok_or_else(|| fail("missing_path", "replace target must already exist"))? =
                value.clone();
        }
        ContractAmendmentOperation::Add => {
            let value = change
                .value
                .as_ref()
                .ok_or_else(|| fail("missing_value", "add requires exactly one value"))?;
            if value.is_null() {
                return Err(fail(
                    "null_value",
                    "null is not a Contract collection element",
                ));
            }
            let target = document
                .pointer_mut(path)
                .ok_or_else(|| fail("missing_path", "add target must be a collection"))?;
            if target.is_null() && path == "/scenarioCoverage" {
                *target = Value::Array(Vec::new());
            }
            let values = target
                .as_array_mut()
                .ok_or_else(|| fail("not_a_collection", "add target must be an array"))?;
            if path == "/scenarioCoverage"
                && (value.get("status").and_then(Value::as_str) != Some("unverified")
                    || value
                        .get("evidence")
                        .and_then(Value::as_array)
                        .is_none_or(|evidence| !evidence.is_empty()))
            {
                return Err(fail(
                    "protected_field",
                    "new scenarios must start unverified with no evidence",
                ));
            }
            let identity = stable_identity(path, value)
                .map_err(|message| fail("invalid_identity", &message))?;
            for existing in values.iter() {
                if stable_identity(path, existing).as_deref() == Ok(identity.as_str()) {
                    return Err(fail(
                        "duplicate_element",
                        "collection already contains this stable identity",
                    ));
                }
            }
            values.push(value.clone());
        }
        ContractAmendmentOperation::Remove => {
            let value = change
                .value
                .as_ref()
                .ok_or_else(|| fail("missing_value", "remove requires the exact element"))?;
            if value.is_null() {
                return Err(fail(
                    "null_value",
                    "null cannot identify a collection element",
                ));
            }
            let target = document
                .pointer_mut(path)
                .ok_or_else(|| fail("missing_path", "remove target must be a collection"))?;
            let values = target
                .as_array_mut()
                .ok_or_else(|| fail("not_a_collection", "remove target must be an array"))?;
            let matches = values
                .iter()
                .enumerate()
                .filter_map(|(index, candidate)| (candidate == value).then_some(index))
                .collect::<Vec<_>>();
            match matches.as_slice() {
                [index] => {
                    values.remove(*index);
                }
                [] => return Err(fail("element_not_found", "exact element is not present")),
                _ => {
                    return Err(fail(
                        "ambiguous_element",
                        "exact element occurs more than once",
                    ));
                }
            }
        }
        ContractAmendmentOperation::Reorder => {
            let order = change
                .value
                .as_ref()
                .and_then(Value::as_array)
                .ok_or_else(|| fail("invalid_order", "reorder requires the full element array"))?;
            let values = current
                .as_ref()
                .and_then(Value::as_array)
                .ok_or_else(|| fail("not_a_collection", "reorder target must be an array"))?;
            let mut existing = BTreeMap::new();
            for value in values {
                let identity = stable_identity(path, value)
                    .map_err(|message| fail("invalid_identity", &message))?;
                if existing.insert(identity, value).is_some() {
                    return Err(fail(
                        "ambiguous_identity",
                        "current collection contains duplicate stable identities",
                    ));
                }
            }
            let mut seen = BTreeSet::new();
            let mut reordered = Vec::with_capacity(order.len());
            for requested in order {
                let identity = stable_identity(path, requested)
                    .map_err(|message| fail("invalid_identity", &message))?;
                if !seen.insert(identity.clone()) {
                    return Err(fail(
                        "duplicate_identity",
                        "reorder repeats a stable identity",
                    ));
                }
                let Some(existing_value) = existing.get(&identity) else {
                    return Err(fail(
                        "not_a_permutation",
                        "reorder must contain every current element exactly once",
                    ));
                };
                if *existing_value != requested {
                    return Err(fail(
                        "reorder_changes_values",
                        "reorder may change order only; edit element values with another operation",
                    ));
                }
                reordered.push((*existing_value).clone());
            }
            if seen.len() != existing.len() {
                return Err(fail(
                    "not_a_permutation",
                    "reorder must contain every current element exactly once",
                ));
            }
            *document
                .pointer_mut(path)
                .ok_or_else(|| fail("missing_path", "reorder target must exist"))? =
                Value::Array(reordered);
        }
    }
    Ok(())
}

/// Apply a reasoned, ordered amendment batch to a typed Contract. The caller
/// binds `expected_contract_digest` to the repository's canonical persisted
/// bytes under its transaction lock before invoking this pure transformation.
pub fn apply_contract_amendment(
    contract: &Contract,
    request: &ContractAmendmentRequest,
) -> Result<Contract, Vec<ContractAmendmentError>> {
    if request.schema_version != CONTRACT_AMENDMENT_SCHEMA_VERSION {
        return Err(errors(error(
            "unsupported_schema_version",
            "/schemaVersion",
            format!(
                "expected {}, received {}",
                CONTRACT_AMENDMENT_SCHEMA_VERSION, request.schema_version
            ),
        )));
    }
    if request.change_id.trim().is_empty() {
        return Err(errors(error(
            "missing_change_id",
            "/changeId",
            "changeId must be non-empty",
        )));
    }
    if request.reason.trim().is_empty() {
        return Err(errors(error(
            "missing_reason",
            "/reason",
            "reason must be non-empty",
        )));
    }
    if request.changes.is_empty() {
        return Err(errors(error(
            "empty_changes",
            "/changes",
            "at least one amendment operation is required",
        )));
    }

    let mut document = serde_json::to_value(contract)
        .map_err(|source| errors(error("contract_serialize_failed", "", source.to_string())))?;
    let acceptance_was_present = document
        .get("acceptance")
        .is_some_and(|acceptance| !acceptance.is_null());

    for change in &request.changes {
        let segments = parse_pointer(&change.path).map_err(errors)?;
        contract_amendment_field_class(&change.path).map_err(errors)?;
        amendment_operation(&mut document, change, &segments).map_err(errors)?;
        validate_changed_collection_identities(&document, &segments, &change.path)
            .map_err(errors)?;
    }

    if acceptance_was_present
        && let Some(acceptance_criteria) = document.get("acceptanceCriteria").cloned()
    {
        document["acceptance"] = acceptance_criteria;
    }

    let prospective: Contract = serde_json::from_value(document)
        .map_err(|source| errors(error("contract_schema_invalid", "", source.to_string())))?;
    prospective.validate().map_err(|validation_errors| {
        validation_errors
            .into_iter()
            .map(|message| error("contract_invariant_failed", "", message))
            .collect::<Vec<_>>()
    })?;
    Ok(prospective)
}

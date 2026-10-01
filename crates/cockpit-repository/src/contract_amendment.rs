use super::{CoordinationError, CoordinationStore, ObserverError};
use cockpit_core::Digest;
use cockpit_git::GitRepository;
use cockpit_protocol::{
    COLLABORATION_CAPABILITY, COLLABORATION_SCHEMA_VERSION, Contract,
    ContractAmendmentAuthorization, ContractAmendmentChange, ContractAmendmentDecision,
    ContractAmendmentError, ContractAmendmentFieldClass, ContractAmendmentOperation,
    ContractAmendmentRequest, ContractAmendmentValueChange, ContractSource, EvidenceAssurance,
    RuntimeCapabilityBinding, RuntimeContext, VerificationDeclaration, apply_contract_amendment,
    apply_contract_amendment_with_trace, contract_amendment_field_class, digest_json,
};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractAmendmentChangedValue {
    pub path: String,
    pub operation: ContractAmendmentOperation,
    #[serde(
        default,
        deserialize_with = "deserialize_present_value",
        skip_serializing_if = "Option::is_none"
    )]
    pub old_value: Option<Value>,
    #[serde(
        default,
        deserialize_with = "deserialize_present_value",
        skip_serializing_if = "Option::is_none"
    )]
    pub new_value: Option<Value>,
}

fn deserialize_present_value<'de, D>(deserializer: D) -> Result<Option<Value>, D::Error>
where
    D: Deserializer<'de>,
{
    Value::deserialize(deserializer).map(Some)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractAmendmentReceipt {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub sequence: u64,
    pub change_id: String,
    pub request_digest: Digest,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorization: Option<ContractAmendmentAuthorization>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_input_digest: Option<Digest>,
    pub changed_values: Vec<ContractAmendmentChangedValue>,
    pub previous_contract_digest: Digest,
    pub new_contract_digest: Digest,
    pub repository_snapshot_digest: Digest,
    pub environment_observation_digest: Digest,
    pub runtime_version: String,
    pub runtime_digest: Digest,
    #[serde(default)]
    pub verification_started: bool,
    #[serde(default)]
    pub checkpointed: bool,
    pub invalidated_required_checks: Vec<String>,
    pub invalidated_evidence: Vec<String>,
    pub policy_review_required: bool,
    pub policy_review_requirements: Vec<String>,
    pub previous_journal_digest: Digest,
    pub journal_digest: Digest,
    pub recorded_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractAmendmentCheck {
    pub schema_version: u32,
    pub allowed: bool,
    pub repository_id: String,
    pub work_item_id: String,
    pub contract_digest: Digest,
    pub repository_snapshot_digest: Digest,
    pub request_digest: Digest,
    pub changed_paths: Vec<String>,
    pub blockers: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PreparedAmendment {
    schema_version: u32,
    repository_id: String,
    work_item_id: String,
    sequence: u64,
    request: ContractAmendmentRequest,
    request_digest: Digest,
    previous_contract: Value,
    previous_contract_digest: Digest,
    new_contract: Value,
    new_contract_digest: Digest,
    previous_summary_digest: Digest,
    repository_snapshot_digest: Digest,
    environment_observation_digest: Digest,
    runtime: RuntimeContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    verification_started: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    checkpointed: Option<bool>,
    previous_journal_digest: Digest,
    policy_review_requirements: Vec<String>,
    legacy_input_digest: Option<Digest>,
    prepared_at: String,
}

struct AmendmentPreparation<'a> {
    work_item_id: &'a str,
    request: &'a ContractAmendmentRequest,
    runtime: &'a RuntimeContext,
    previous_contract: Value,
    new_contract: Value,
    previous_summary: &'a Value,
    previous_journal_digest: Digest,
    sequence: u64,
    legacy_input_digest: Option<Digest>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PreparedRecord {
    prepared: PreparedAmendment,
    prepared_digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CommittedRecord {
    sequence: u64,
    prepared_digest: Digest,
    receipt: ContractAmendmentReceipt,
}

const ZERO_JOURNAL_DIGEST_SEED: &[u8] = b"cockpit-contract-amendment-journal-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AmendmentFailpoint {
    Prepared,
    ContractReplaced,
    SummaryProjected,
    CommitMarked,
}

#[cfg(test)]
thread_local! {
    static AMENDMENT_FAILPOINT: std::cell::Cell<Option<AmendmentFailpoint>> = const { std::cell::Cell::new(None) };
    static AMENDMENT_SNAPSHOT_MUTATION: std::cell::RefCell<Option<(PathBuf, Vec<u8>)>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn set_amendment_failpoint(point: AmendmentFailpoint) {
    AMENDMENT_FAILPOINT.with(|value| value.set(Some(point)));
}

#[cfg(test)]
fn mutate_snapshot_before_prepared_write(path: PathBuf, contents: Vec<u8>) {
    AMENDMENT_SNAPSHOT_MUTATION.with(|value| *value.borrow_mut() = Some((path, contents)));
}

#[cfg(test)]
fn apply_snapshot_mutation_before_prepared_write(root: &Path) {
    AMENDMENT_SNAPSHOT_MUTATION.with(|value| {
        if let Some((path, contents)) = value.borrow_mut().take() {
            fs::write(root.join(path), contents).expect("mutate source at prepared-write boundary");
        }
    });
}

fn maybe_fail(point: AmendmentFailpoint) -> Result<(), ObserverError> {
    #[cfg(test)]
    if AMENDMENT_FAILPOINT.with(|value| value.get()) == Some(point) {
        AMENDMENT_FAILPOINT.with(|value| value.set(None));
        return Err(ObserverError::State {
            path: PathBuf::from("contract-amendment-test-failpoint"),
            message: format!("injected interruption at {point:?}"),
        });
    }
    #[cfg(not(test))]
    let _ = point;
    Ok(())
}

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
        authorization: None,
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

fn apply_runtime_capability_requirements(
    amended: Value,
) -> Result<Value, Vec<ContractAmendmentError>> {
    let mut amended: Contract = serde_json::from_value(amended).map_err(|source| {
        vec![ContractAmendmentError {
            code: "contract_schema_invalid".into(),
            path: String::new(),
            message: source.to_string(),
        }]
    })?;
    amended.required_runtime_capabilities.extend([
        "work-item-contract-amendment".to_owned(),
        "work-item-environment-drift".to_owned(),
    ]);
    amended.required_runtime_capabilities.sort();
    amended.required_runtime_capabilities.dedup();
    amended.validate().map_err(|errors| {
        vec![ContractAmendmentError {
            code: "contract_invariant_failed".into(),
            path: "/requiredRuntimeCapabilities".into(),
            message: errors.join("; "),
        }]
    })?;
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

fn amendment_directory(root: &Path, work_item_id: &str) -> PathBuf {
    root.join(".ai/evidence")
        .join(format!("{work_item_id}.contract-amendments"))
}

fn amendment_record_path(root: &Path, work_item_id: &str, sequence: u64, suffix: &str) -> PathBuf {
    amendment_directory(root, work_item_id).join(format!("{sequence:08}.{suffix}.json"))
}

fn state_error(path: impl Into<PathBuf>, message: impl Into<String>) -> ObserverError {
    ObserverError::State {
        path: path.into(),
        message: message.into(),
    }
}

fn record_digest<T: Serialize>(value: &T, path: &Path) -> Result<Digest, ObserverError> {
    digest_json(value).map_err(|error| state_error(path, error.to_string()))
}

fn read_typed_record<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, ObserverError> {
    let value = super::read_json(path)?;
    serde_json::from_value(value).map_err(|error| state_error(path, error.to_string()))
}

fn write_immutable_record<T: Serialize>(path: &Path, value: &T) -> Result<(), ObserverError> {
    let bytes =
        serde_json::to_vec_pretty(value).map_err(|error| state_error(path, error.to_string()))?;
    if path.exists() {
        let existing = fs::read(path).map_err(|source| ObserverError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        return if existing == bytes {
            Ok(())
        } else {
            Err(state_error(
                path,
                "append-only Contract amendment record already exists with different bytes",
            ))
        };
    }
    let parent = path
        .parent()
        .ok_or_else(|| state_error(path, "amendment record has no parent directory"))?;
    fs::create_dir_all(parent).map_err(|source| ObserverError::Read {
        path: parent.to_path_buf(),
        source,
    })?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let temporary = parent.join(format!(".amendment-{}-{nonce}.tmp", std::process::id()));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|source| ObserverError::Read {
                path: temporary.clone(),
                source,
            })?;
        file.write_all(&bytes)
            .and_then(|()| file.sync_all())
            .map_err(|source| ObserverError::Read {
                path: temporary.clone(),
                source,
            })?;
        fs::rename(&temporary, path).map_err(|source| ObserverError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        sync_directory(parent)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn durable_replace_json(path: &Path, value: &Value) -> Result<(), ObserverError> {
    let bytes =
        serde_json::to_vec_pretty(value).map_err(|error| state_error(path, error.to_string()))?;
    let parent = path
        .parent()
        .ok_or_else(|| state_error(path, "replacement has no parent directory"))?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let temporary = parent.join(format!(
        ".amendment-replace-{}-{nonce}.tmp",
        std::process::id()
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|source| ObserverError::Read {
                path: temporary.clone(),
                source,
            })?;
        file.write_all(&bytes)
            .and_then(|()| file.sync_all())
            .map_err(|source| ObserverError::Read {
                path: temporary.clone(),
                source,
            })?;
        fs::rename(&temporary, path).map_err(|source| ObserverError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        sync_directory(parent)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn sync_directory(path: &Path) -> Result<(), ObserverError> {
    #[cfg(unix)]
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|source| ObserverError::Read {
            path: path.to_path_buf(),
            source,
        })?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

fn journal_digest(receipt: &ContractAmendmentReceipt) -> Result<Digest, ObserverError> {
    let mut core = serde_json::to_value(receipt)
        .map_err(|error| state_error("contract-amendment-journal", error.to_string()))?;
    core.as_object_mut()
        .expect("receipt serializes as an object")
        .remove("journalDigest");
    record_digest(
        &json!({
            "previousJournalDigest": receipt.previous_journal_digest,
            "receipt": core,
        }),
        Path::new("contract-amendment-journal"),
    )
}

fn load_history(
    root: &Path,
    work_item_id: &str,
    allow_pending: bool,
) -> Result<(Vec<ContractAmendmentReceipt>, Option<PreparedRecord>), ObserverError> {
    let directory = amendment_directory(root, work_item_id);
    if !directory.exists() {
        return Ok((Vec::new(), None));
    }
    let mut records = BTreeMap::<u64, (Option<PreparedRecord>, Option<CommittedRecord>)>::new();
    for item in fs::read_dir(&directory).map_err(|source| ObserverError::Read {
        path: directory.clone(),
        source,
    })? {
        let item = item.map_err(|source| ObserverError::Read {
            path: directory.clone(),
            source,
        })?;
        let path = item.path();
        let metadata = fs::symlink_metadata(&path).map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        if !metadata.file_type().is_file() {
            return Err(state_error(
                &path,
                "amendment history contains a non-file entry",
            ));
        }
        let name = item.file_name();
        let name = name.to_string_lossy();
        let Some((sequence, suffix)) = name.split_once('.') else {
            return Err(state_error(
                &path,
                "unrecognized Contract amendment journal entry",
            ));
        };
        let sequence = sequence
            .parse::<u64>()
            .map_err(|_| state_error(&path, "invalid Contract amendment sequence"))?;
        let suffix = suffix.strip_suffix(".json").unwrap_or_default();
        let pair = records.entry(sequence).or_default();
        match suffix {
            "prepared" if pair.0.is_none() => pair.0 = Some(read_typed_record(&path)?),
            "committed" if pair.1.is_none() => pair.1 = Some(read_typed_record(&path)?),
            _ => {
                return Err(state_error(
                    &path,
                    "duplicate or unknown amendment journal entry",
                ));
            }
        }
    }

    let mut receipts = Vec::new();
    let mut previous_digest = Digest::sha256_bytes(ZERO_JOURNAL_DIGEST_SEED);
    let repository_identity = super::repository_id(root).to_string();
    let mut pending = None;
    for (expected_sequence, (sequence, (prepared, committed))) in records
        .into_iter()
        .enumerate()
        .map(|(index, record)| (index as u64 + 1, record))
    {
        if sequence != expected_sequence {
            return Err(state_error(
                &directory,
                "Contract amendment journal sequence has a gap or duplicate",
            ));
        }
        let prepared = prepared.ok_or_else(|| {
            state_error(
                &directory,
                "committed Contract amendment lacks its prepared record",
            )
        })?;
        if prepared.prepared.sequence != sequence
            || prepared.prepared.work_item_id != work_item_id
            || prepared.prepared.repository_id != repository_identity
            || prepared.prepared_digest != record_digest(&prepared.prepared, &directory)?
            || prepared.prepared.previous_journal_digest != previous_digest
            || prepared.prepared.request_digest
                != record_digest(
                    &prepared.prepared.request,
                    Path::new("contract-amendment-request"),
                )?
            || prepared.prepared.request.expected_contract_digest
                != prepared.prepared.previous_contract_digest
            || prepared.prepared.previous_contract_digest
                != record_digest(&prepared.prepared.previous_contract, Path::new("Contract"))?
            || prepared.prepared.new_contract_digest
                != record_digest(&prepared.prepared.new_contract, Path::new("Contract"))?
            || prepared.prepared.runtime.protocol_version != cockpit_protocol::PROTOCOL_VERSION
            || prepared.prepared.runtime.runtime_version.trim().is_empty()
            || policy_review_requirements(&prepared.prepared.request)?
                != prepared.prepared.policy_review_requirements
        {
            return Err(state_error(
                &directory,
                "Contract amendment prepared record failed its identity or digest chain",
            ));
        }
        let mut derived_contract = apply_typed_amendment(
            &prepared.prepared.previous_contract,
            &prepared.prepared.request,
        )
        .map_err(|errors| {
            state_error(
                &directory,
                format!(
                    "prepared amendment request is invalid: {}",
                    amendment_error_message(&errors)
                ),
            )
        })?;
        if prepared
            .prepared
            .new_contract
            .get("requiredRuntimeCapabilities")
            .and_then(Value::as_array)
            .is_some_and(|capabilities| !capabilities.is_empty())
        {
            derived_contract =
                apply_runtime_capability_requirements(derived_contract).map_err(|errors| {
                    state_error(
                        &directory,
                        format!(
                            "prepared amendment Runtime capability requirements are invalid: {}",
                            amendment_error_message(&errors)
                        ),
                    )
                })?;
        }
        if derived_contract != prepared.prepared.new_contract {
            return Err(state_error(
                &directory,
                "prepared amendment result does not match its request",
            ));
        }
        let Some(committed) = committed else {
            if !allow_pending || pending.is_some() || sequence != records_len_hint(&directory)? {
                return Err(state_error(
                    &directory,
                    "unresolved Contract amendment transaction requires recovery",
                ));
            }
            pending = Some(prepared);
            break;
        };
        let receipt = committed.receipt;
        if committed.sequence != sequence
            || committed.prepared_digest != prepared.prepared_digest
            || receipt.sequence != sequence
            || receipt.repository_id != repository_identity
            || receipt.work_item_id != work_item_id
            || receipt.request_digest != prepared.prepared.request_digest
            || receipt.change_id != prepared.prepared.request.change_id
            || receipt.reason != prepared.prepared.request.reason
            || receipt.legacy_input_digest != prepared.prepared.legacy_input_digest
            || receipt.changed_values
                != changed_values(
                    &prepared.prepared.previous_contract,
                    &prepared.prepared.new_contract,
                    &prepared.prepared.request,
                )?
            || receipt.previous_contract_digest != prepared.prepared.previous_contract_digest
            || receipt.new_contract_digest != prepared.prepared.new_contract_digest
            || receipt.repository_snapshot_digest != prepared.prepared.repository_snapshot_digest
            || receipt.environment_observation_digest
                != prepared.prepared.environment_observation_digest
            || receipt.runtime_version != prepared.prepared.runtime.runtime_version
            || receipt.runtime_digest != prepared.prepared.runtime.runtime_digest
            || prepared
                .prepared
                .verification_started
                .is_some_and(|started| receipt.verification_started != started)
            || prepared
                .prepared
                .checkpointed
                .is_some_and(|checkpointed| receipt.checkpointed != checkpointed)
            || receipt.policy_review_required
                != !prepared.prepared.policy_review_requirements.is_empty()
            || receipt.policy_review_requirements != prepared.prepared.policy_review_requirements
            || receipt.previous_journal_digest != previous_digest
            || journal_digest(&receipt)? != receipt.journal_digest
        {
            return Err(state_error(
                &directory,
                "Contract amendment committed record is detached from its prepared amendment or has an invalid digest chain",
            ));
        }
        previous_digest = receipt.journal_digest.clone();
        receipts.push(receipt);
    }
    Ok((receipts, pending))
}

fn records_len_hint(directory: &Path) -> Result<u64, ObserverError> {
    let count = fs::read_dir(directory)
        .map_err(|source| ObserverError::Read {
            path: directory.to_path_buf(),
            source,
        })?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .ends_with(".prepared.json")
        })
        .count();
    Ok(count as u64)
}

pub fn read_work_item_contract_amendments(
    root: &Path,
    work_item_id: &str,
) -> Result<Vec<ContractAmendmentReceipt>, ObserverError> {
    super::validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.to_path_buf(),
        source,
    })?;
    let (receipts, pending) = load_history(&root, work_item_id, false)?;
    if pending.is_some() {
        return Err(state_error(
            amendment_directory(&root, work_item_id),
            "read-only amendment history cannot consume an unresolved transaction",
        ));
    }
    Ok(receipts)
}

/// Explain whether a typed amendment is admissible without writing repository
/// state. Apply always repeats this check while holding the shared repository
/// lock, so this read-only result is never itself a write authorization.
pub fn check_work_item_contract_amendment(
    root: &Path,
    work_item_id: &str,
    request: &ContractAmendmentRequest,
) -> Result<ContractAmendmentCheck, ObserverError> {
    super::validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.to_path_buf(),
        source,
    })?;
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let current = super::read_json(&contract_path)?;
    let contract: Contract = serde_json::from_value(current.clone())
        .map_err(|error| state_error(&contract_path, format!("invalid Contract: {error}")))?;
    let repository_id = super::repository_id(&root).to_string();
    let contract_digest = record_digest(&current, &contract_path)?;
    let (repository_snapshot_digest, _) = observed_inputs(&root)?;
    let request_digest = request
        .authorization_request_digest()
        .map_err(|error| state_error(&contract_path, format!("request digest failed: {error}")))?;
    let mut blockers = Vec::new();
    let mut changed_paths = Vec::new();

    if contract.repository_id != repository_id {
        blockers.push("repository_identity_mismatch".into());
    }
    if contract.work_item_id != work_item_id {
        blockers.push("work_item_identity_mismatch".into());
    }
    if contract_digest != request.expected_contract_digest {
        blockers.push("contract_digest_conflict".into());
    } else {
        match apply_contract_amendment_with_trace(&contract, request) {
            Ok((_, trace)) => {
                changed_paths = trace
                    .iter()
                    .map(|change| change.path.clone())
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .collect();
                if trace.is_empty()
                    || trace
                        .iter()
                        .all(|change| change.old_value == change.new_value)
                {
                    blockers.push("no_op_amendment".into());
                }
            }
            Err(errors) => blockers.extend(errors.into_iter().map(|error| error.code)),
        }
    }

    match request.authorization.as_ref() {
        None => blockers.push("authorization_missing".into()),
        Some(authorization) => {
            if authorization.schema_version
                != cockpit_protocol::CONTRACT_AMENDMENT_AUTHORIZATION_SCHEMA_VERSION
            {
                blockers.push("authorization_schema_unsupported".into());
            }
            if authorization.decision != ContractAmendmentDecision::AuthorizeChange {
                blockers.push("authorization_decision_unsupported".into());
            }
            if authorization.decision_id.trim().is_empty()
                || authorization.authorized_by.trim().is_empty()
                || authorization.authority_source.trim().is_empty()
                || authorization.executed_by.trim().is_empty()
            {
                blockers.push("authorization_provenance_incomplete".into());
            }
            if authorization.authorized_by == authorization.executed_by {
                blockers.push("authorizer_executor_not_distinct".into());
            }
            if authorization.assurance != EvidenceAssurance::SelfDeclared {
                blockers.push("authorization_assurance_not_supported".into());
            }
            if authorization.repository_id != repository_id {
                blockers.push("authorization_repository_mismatch".into());
            }
            if authorization.work_item_id != work_item_id {
                blockers.push("authorization_work_item_mismatch".into());
            }
            if authorization.contract_digest != contract_digest
                || authorization.contract_digest != request.expected_contract_digest
            {
                blockers.push("authorization_contract_mismatch".into());
            }
            if authorization.repository_snapshot_digest != repository_snapshot_digest {
                blockers.push("authorization_snapshot_mismatch".into());
            }
            if authorization.request_digest != request_digest {
                blockers.push("authorization_request_mismatch".into());
            }
            if authorization.changed_paths != changed_paths {
                blockers.push("authorization_changed_paths_mismatch".into());
            }
        }
    }

    blockers.sort();
    blockers.dedup();
    Ok(ContractAmendmentCheck {
        schema_version: 1,
        allowed: blockers.is_empty(),
        repository_id,
        work_item_id: work_item_id.into(),
        contract_digest,
        repository_snapshot_digest,
        request_digest,
        changed_paths,
        blockers,
    })
}

fn shared_amendment_store(
    root: &Path,
    runtime: &RuntimeContext,
) -> Result<CoordinationStore, ObserverError> {
    let git = GitRepository::discover(root)
        .map_err(|error| state_error(root, format!("repository topology unavailable: {error}")))?;
    let binding = RuntimeCapabilityBinding {
        schema_version: COLLABORATION_SCHEMA_VERSION,
        runtime_version: runtime.runtime_version.clone(),
        runtime_digest: runtime.runtime_digest.clone(),
        capability: COLLABORATION_CAPABILITY.into(),
    };
    CoordinationStore::open(&git, binding)
        .map_err(|error| state_error(root, format!("shared amendment lock unavailable: {error}")))
}

fn history_descends_from(
    receipts: &[ContractAmendmentReceipt],
    ancestor: &Digest,
    head: &Digest,
) -> bool {
    let mut cursor = ancestor.clone();
    let mut started = false;
    for receipt in receipts {
        if !started {
            if receipt.previous_contract_digest == cursor {
                cursor = receipt.new_contract_digest.clone();
                started = true;
            }
        } else if receipt.previous_contract_digest == cursor {
            cursor = receipt.new_contract_digest.clone();
        }
    }
    started && &cursor == head
}

fn validate_shared_worktree_amendment_state(
    root: &Path,
    work_item_id: &str,
    expected_contract_digest: &Digest,
) -> Result<(), ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.to_path_buf(),
        source,
    })?;
    let local_history = read_history_with_pending(&root, work_item_id)?;
    if local_history.1.is_some() {
        return Err(state_error(
            amendment_directory(&root, work_item_id),
            "local amendment transaction is pending recovery",
        ));
    }
    let known_ancestors = local_history
        .0
        .iter()
        .flat_map(|receipt| {
            [
                receipt.previous_contract_digest.to_string(),
                receipt.new_contract_digest.to_string(),
            ]
        })
        .collect::<std::collections::BTreeSet<_>>();
    for record in super::resource_lifecycle::git_worktree_records(&root)? {
        let Some(path) = record.path else { continue };
        let peer_root = match fs::canonicalize(&path) {
            Ok(peer_root) => peer_root,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && record.prunable => {
                continue;
            }
            Err(source) => {
                return Err(ObserverError::Read {
                    path: path.clone(),
                    source,
                });
            }
        };
        if peer_root == root {
            continue;
        }
        let contract_path = peer_root
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.contract.json"));
        match fs::symlink_metadata(&contract_path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(source) => {
                return Err(ObserverError::Read {
                    path: contract_path,
                    source,
                });
            }
            Ok(metadata) if !metadata.file_type().is_file() => {
                return Err(state_error(
                    &contract_path,
                    "linked Work Item Contract is not a regular file",
                ));
            }
            Ok(_) => {}
        }
        let peer_contract = super::read_json(&contract_path)?;
        let peer_digest = record_digest(&peer_contract, &contract_path)?;
        let (peer_history, pending) = read_history_with_pending(&peer_root, work_item_id)?;
        if pending.is_some() {
            return Err(state_error(
                &contract_path,
                "linked worktree has a pending Contract amendment transaction",
            ));
        }
        if &peer_digest == expected_contract_digest
            || known_ancestors.contains(&peer_digest.to_string())
        {
            continue;
        }
        if history_descends_from(&peer_history, expected_contract_digest, &peer_digest) {
            return Err(state_error(
                &contract_path,
                "contract_digest_conflict: a linked worktree already applied an amendment from this expected digest",
            ));
        }
        return Err(state_error(
            &contract_path,
            "linked worktree Contract diverges without a verifiable amendment lineage",
        ));
    }
    Ok(())
}

fn with_shared_amendment_lock<T>(
    root: &Path,
    runtime: &RuntimeContext,
    operation: impl FnOnce() -> Result<T, ObserverError>,
) -> Result<T, ObserverError> {
    let store = shared_amendment_store(root, runtime)?;
    store
        .with_lock(|| {
            operation().map_err(|error| CoordinationError::RecoveryRequired(error.to_string()))
        })
        .map_err(|error| state_error(root, format!("shared amendment operation failed: {error}")))
}

fn observed_inputs(root: &Path) -> Result<(Digest, Digest), ObserverError> {
    let execution = super::RepositoryExecutionContext::capture(root)?;
    let source_snapshot = super::snapshot_digest(execution.snapshot())?;
    let environment = super::execution_context::execution_environment_digest_from_values(
        std::env::vars_os().collect::<Vec<_>>(),
        root,
    )?;
    let environment = environment.parse::<Digest>().map_err(|error| {
        state_error(
            root,
            format!("invalid observed environment digest: {error}"),
        )
    })?;
    Ok((source_snapshot, environment))
}

fn changed_values(
    previous: &Value,
    next: &Value,
    request: &ContractAmendmentRequest,
) -> Result<Vec<ContractAmendmentChangedValue>, ObserverError> {
    let previous_contract: Contract =
        serde_json::from_value(previous.clone()).map_err(|error| {
            state_error(
                "contract-amendment-audit",
                format!("invalid previous Contract: {error}"),
            )
        })?;
    let (_, trace) =
        apply_contract_amendment_with_trace(&previous_contract, request).map_err(|errors| {
            state_error("contract-amendment-audit", amendment_error_message(&errors))
        })?;
    let mut values = trace
        .into_iter()
        .map(
            |ContractAmendmentValueChange {
                 path,
                 operation,
                 old_value,
                 new_value,
             }| ContractAmendmentChangedValue {
                path,
                operation,
                old_value,
                new_value,
            },
        )
        .collect::<Vec<_>>();
    let old_capabilities = previous.get("requiredRuntimeCapabilities").cloned();
    let new_capabilities = next.get("requiredRuntimeCapabilities").cloned();
    if old_capabilities != new_capabilities {
        values.push(ContractAmendmentChangedValue {
            path: "/requiredRuntimeCapabilities".into(),
            operation: ContractAmendmentOperation::Set,
            old_value: old_capabilities,
            new_value: new_capabilities,
        });
    }
    Ok(values)
}

pub(crate) fn has_sensitive_amendment(
    root: &Path,
    work_item_id: &str,
) -> Result<bool, ObserverError> {
    Ok(read_work_item_contract_amendments(root, work_item_id)?
        .iter()
        .any(|receipt| receipt.policy_review_required))
}

fn policy_review_requirements(
    request: &ContractAmendmentRequest,
) -> Result<Vec<String>, ObserverError> {
    let mut requirements = Vec::new();
    for change in &request.changes {
        match contract_amendment_field_class(&change.path) {
            Ok(ContractAmendmentFieldClass::SensitivePlanEditable) => {
                requirements.push(change.path.clone());
            }
            Ok(ContractAmendmentFieldClass::PlanEditable) => {}
            Err(error) => {
                return Err(state_error("contract-amendment-policy", error.to_string()));
            }
        }
    }
    requirements.sort();
    requirements.dedup();
    Ok(requirements)
}

fn prepare_transaction(
    root: &Path,
    input: AmendmentPreparation<'_>,
) -> Result<PreparedRecord, ObserverError> {
    let AmendmentPreparation {
        work_item_id,
        request,
        runtime,
        previous_contract,
        new_contract,
        previous_summary,
        previous_journal_digest,
        sequence,
        legacy_input_digest,
    } = input;
    let request_digest = record_digest(request, Path::new("contract-amendment-request"))?;
    let previous_contract_digest = record_digest(&previous_contract, Path::new("Contract"))?;
    let new_contract_digest = record_digest(&new_contract, Path::new("Contract"))?;
    let previous_summary_digest = record_digest(previous_summary, Path::new("Summary"))?;
    let (repository_snapshot_digest, environment_observation_digest) = observed_inputs(root)?;
    let verification_started =
        super::contract_amendment_verification_started(root, work_item_id, previous_summary);
    let checkpointed = previous_summary
        .get("checkpointCount")
        .and_then(Value::as_u64)
        .is_some_and(|count| count > 0);
    let prepared = PreparedAmendment {
        schema_version: 1,
        repository_id: super::repository_id(root).to_string(),
        work_item_id: work_item_id.into(),
        sequence,
        request: request.clone(),
        request_digest,
        previous_contract,
        previous_contract_digest,
        new_contract,
        new_contract_digest,
        previous_summary_digest,
        repository_snapshot_digest,
        environment_observation_digest,
        runtime: runtime.clone(),
        verification_started: Some(verification_started),
        checkpointed: Some(checkpointed),
        previous_journal_digest,
        policy_review_requirements: policy_review_requirements(request)?,
        legacy_input_digest,
        prepared_at: super::now(),
    };
    let prepared_digest = record_digest(&prepared, &amendment_directory(root, work_item_id))?;
    Ok(PreparedRecord {
        prepared,
        prepared_digest,
    })
}

fn write_prepared(root: &Path, prepared: &PreparedRecord) -> Result<(), ObserverError> {
    let path = amendment_record_path(
        root,
        &prepared.prepared.work_item_id,
        prepared.prepared.sequence,
        "prepared",
    );
    write_immutable_record(&path, prepared)
}

fn commit_prepared(
    root: &Path,
    prepared: &PreparedRecord,
    summary: &Value,
) -> Result<ContractAmendmentReceipt, ObserverError> {
    let data = &prepared.prepared;
    let invalidation = summary
        .get("verificationInvalidatedByContractAmendment")
        .filter(|value| {
            value.get("contractHash").and_then(Value::as_str)
                == Some(data.new_contract_digest.to_string().as_str())
        });
    let invalidated_required_checks = invalidation
        .and_then(|value| value.get("invalidatedRequiredChecks"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let verification_started = data.verification_started.unwrap_or_else(|| {
        super::contract_amendment_verification_started(root, &data.work_item_id, summary)
    });
    let checkpointed = data.checkpointed.unwrap_or_else(|| {
        summary
            .get("checkpointCount")
            .and_then(Value::as_u64)
            .is_some_and(|count| count > 0)
    });
    let invalidated_evidence = if verification_started {
        let path = format!(".ai/evidence/{}.verification.json", data.work_item_id);
        if root.join(&path).exists() {
            vec![path]
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    let mut receipt = ContractAmendmentReceipt {
        schema_version: 1,
        repository_id: data.repository_id.clone(),
        work_item_id: data.work_item_id.clone(),
        sequence: data.sequence,
        change_id: data.request.change_id.clone(),
        request_digest: data.request_digest.clone(),
        reason: data.request.reason.clone(),
        authorization: data.request.authorization.clone(),
        legacy_input_digest: data.legacy_input_digest.clone(),
        changed_values: changed_values(&data.previous_contract, &data.new_contract, &data.request)?,
        previous_contract_digest: data.previous_contract_digest.clone(),
        new_contract_digest: data.new_contract_digest.clone(),
        repository_snapshot_digest: data.repository_snapshot_digest.clone(),
        environment_observation_digest: data.environment_observation_digest.clone(),
        runtime_version: data.runtime.runtime_version.clone(),
        runtime_digest: data.runtime.runtime_digest.clone(),
        verification_started,
        checkpointed,
        invalidated_required_checks,
        invalidated_evidence,
        policy_review_required: !data.policy_review_requirements.is_empty(),
        policy_review_requirements: data.policy_review_requirements.clone(),
        previous_journal_digest: data.previous_journal_digest.clone(),
        journal_digest: Digest::sha256_bytes(b"uncommitted"),
        recorded_at: super::now(),
    };
    receipt.journal_digest = journal_digest(&receipt)?;
    let record = CommittedRecord {
        sequence: receipt.sequence,
        prepared_digest: prepared.prepared_digest.clone(),
        receipt: receipt.clone(),
    };
    let path = amendment_record_path(root, &receipt.work_item_id, receipt.sequence, "committed");
    write_immutable_record(&path, &record)?;
    maybe_fail(AmendmentFailpoint::CommitMarked)?;
    Ok(receipt)
}

fn finish_prepared(
    root: &Path,
    prepared: &PreparedRecord,
) -> Result<ContractAmendmentReceipt, ObserverError> {
    let data = &prepared.prepared;
    let directory = amendment_directory(root, &data.work_item_id);
    if data.repository_id != super::repository_id(root).to_string() {
        return Err(state_error(
            &directory,
            "prepared amendment repository identity changed",
        ));
    }
    let (snapshot_digest, _) = observed_inputs(root)?;
    if snapshot_digest != data.repository_snapshot_digest {
        return Err(state_error(
            &directory,
            "source snapshot changed during an unresolved Contract amendment transaction",
        ));
    }
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{}.contract.json", data.work_item_id));
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{}.summary.json", data.work_item_id));
    let current_contract = super::read_json(&contract_path)?;
    let current_contract_digest = record_digest(&current_contract, &contract_path)?;
    let summary = super::read_json(&summary_path)?;
    let current_summary_digest = record_digest(&summary, &summary_path)?;
    if current_contract_digest == data.previous_contract_digest {
        if current_summary_digest != data.previous_summary_digest {
            return Err(state_error(
                &summary_path,
                "Summary changed after amendment preparation; refusing to overwrite it",
            ));
        }
        super::validate_contract_amendment_revalidation_ready(root, &data.work_item_id, &summary)?;
        durable_replace_json(&contract_path, &data.new_contract)?;
        maybe_fail(AmendmentFailpoint::ContractReplaced)?;
    } else if current_contract_digest != data.new_contract_digest {
        return Err(state_error(
            &contract_path,
            "Contract matches neither side of its prepared amendment transaction",
        ));
    }

    let checkpointed = data.checkpointed.unwrap_or_else(|| {
        summary
            .get("checkpointCount")
            .and_then(Value::as_u64)
            .is_some_and(|count| count > 0)
    });
    if checkpointed {
        let current_summary = super::read_json(&summary_path)?;
        let summary_digest = record_digest(&current_summary, &summary_path)?;
        if summary_digest == data.previous_summary_digest {
            super::validate_contract_amendment_revalidation_ready(
                root,
                &data.work_item_id,
                &current_summary,
            )?;
            super::revalidate_contract_amendment(root, &data.work_item_id, &data.request.reason)?;
        } else {
            let is_already_projected = current_summary
                .get("verificationInvalidatedByContractAmendment")
                .and_then(|value| value.get("contractHash"))
                .and_then(Value::as_str)
                == Some(data.new_contract_digest.to_string().as_str());
            if !is_already_projected {
                return Err(state_error(
                    &summary_path,
                    "Summary matches neither side of its prepared amendment transaction",
                ));
            }
        }
        maybe_fail(AmendmentFailpoint::SummaryProjected)?;
    }
    let final_summary = super::read_json(&summary_path)?;
    commit_prepared(root, prepared, &final_summary)
}

fn read_history_with_pending(
    root: &Path,
    work_item_id: &str,
) -> Result<(Vec<ContractAmendmentReceipt>, Option<PreparedRecord>), ObserverError> {
    load_history(root, work_item_id, true)
}

pub fn apply_work_item_contract_amendment(
    root: &Path,
    work_item_id: &str,
    request: &ContractAmendmentRequest,
    runtime: &RuntimeContext,
) -> Result<ContractAmendmentReceipt, ObserverError> {
    super::validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.to_path_buf(),
        source,
    })?;
    let request_digest = record_digest(request, Path::new("contract-amendment-request"))?;
    let (history, pending) = read_history_with_pending(&root, work_item_id)?;
    if pending.is_none() {
        if let Some(existing) = history
            .iter()
            .find(|receipt| receipt.change_id == request.change_id)
        {
            if existing.request_digest == request_digest {
                return Ok(existing.clone());
            }
            return Err(state_error(
                amendment_directory(&root, work_item_id),
                format!(
                    "changeId {} already exists with different request bytes",
                    request.change_id
                ),
            ));
        }
        let admission = check_work_item_contract_amendment(&root, work_item_id, request)?;
        if !admission.allowed {
            return Err(state_error(
                root.join(".ai/work-items/active")
                    .join(format!("{work_item_id}.contract.json")),
                format!(
                    "amendment_admission_rejected: {}",
                    admission.blockers.join(", ")
                ),
            ));
        }
    }
    with_shared_amendment_lock(&root, runtime, || {
        let (mut history, pending) = read_history_with_pending(&root, work_item_id)?;
        if let Some(pending) = pending {
            finish_prepared(&root, &pending)?;
            (history, _) = load_history(&root, work_item_id, false)?;
        }
        if let Some(existing) = history
            .iter()
            .find(|receipt| receipt.change_id == request.change_id)
        {
            if existing.request_digest == request_digest {
                return Ok(existing.clone());
            }
            return Err(state_error(
                amendment_directory(&root, work_item_id),
                format!(
                    "changeId {} already exists with different request bytes",
                    request.change_id
                ),
            ));
        }
        let check = check_work_item_contract_amendment(&root, work_item_id, request)?;
        if !check.allowed {
            return Err(state_error(
                root.join(".ai/work-items/active")
                    .join(format!("{work_item_id}.contract.json")),
                format!(
                    "amendment_admission_rejected: {}",
                    check.blockers.join(", ")
                ),
            ));
        }
        validate_shared_worktree_amendment_state(
            &root,
            work_item_id,
            &request.expected_contract_digest,
        )?;
        apply_work_item_contract_amendment_with_legacy_input(
            &root,
            work_item_id,
            request,
            runtime,
            None,
        )
    })
}

pub(crate) fn apply_work_item_contract_amendment_with_legacy_input(
    root: &Path,
    work_item_id: &str,
    request: &ContractAmendmentRequest,
    runtime: &RuntimeContext,
    legacy_input_digest: Option<Digest>,
) -> Result<ContractAmendmentReceipt, ObserverError> {
    super::validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.to_path_buf(),
        source,
    })?;
    if runtime.protocol_version != cockpit_protocol::PROTOCOL_VERSION
        || runtime.runtime_version.trim().is_empty()
    {
        return Err(state_error(
            root.join(".ai"),
            "Contract amendment requires a validated Runtime context",
        ));
    }
    let _lock = super::acquire_lifecycle_lock(&root, work_item_id)?;
    let (mut history, pending) = read_history_with_pending(&root, work_item_id)?;
    if let Some(pending) = pending {
        finish_prepared(&root, &pending)?;
        (history, _) = load_history(&root, work_item_id, false)?;
    }
    let request_digest = record_digest(request, Path::new("contract-amendment-request"))?;
    if let Some(existing) = history
        .iter()
        .find(|receipt| receipt.change_id == request.change_id)
    {
        if existing.request_digest == request_digest
            || (legacy_input_digest.is_some()
                && existing.legacy_input_digest == legacy_input_digest)
        {
            return Ok(existing.clone());
        }
        return Err(state_error(
            amendment_directory(&root, work_item_id),
            format!(
                "changeId {} already exists with different request bytes",
                request.change_id
            ),
        ));
    }

    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let previous_contract = super::read_json(&contract_path)?;
    let previous_summary = super::read_json(&summary_path)?;
    let current_contract_digest = record_digest(&previous_contract, &contract_path)?;
    if current_contract_digest != request.expected_contract_digest {
        return Err(state_error(
            &contract_path,
            format!(
                "contract_digest_conflict: expected {}, current Contract digest is {}",
                request.expected_contract_digest, current_contract_digest
            ),
        ));
    }
    if previous_summary["recoveryRetryPending"] == Value::Bool(true) {
        let recovery = super::load_recovery_decision(&root, work_item_id, None)?;
        if recovery
            .as_ref()
            .is_none_or(|decision| decision.decision != "retry")
        {
            return Err(super::recovery_decision_error(
                root.join(".ai/decisions"),
                "retry_binding_missing",
                "pending retry cannot be advanced by a Contract amendment without its exact recovery receipt",
            ));
        }
    }
    if previous_summary
        .get("checkpointEvidence")
        .and_then(Value::as_array)
        .is_some_and(|entries| {
            entries
                .iter()
                .any(|entry| entry.get("stage").and_then(Value::as_str) == Some("before_finish"))
        })
    {
        return Err(state_error(
            &summary_path,
            "contract amendment after before_finish requires a recovery Work Item",
        ));
    }
    let next_contract = apply_typed_amendment(&previous_contract, request)
        .and_then(apply_runtime_capability_requirements)
        .map_err(|errors| state_error(&contract_path, amendment_error_message(&errors)))?;
    let required_evidence_classes = next_contract["requiredEvidenceClasses"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    super::validate_required_evidence_classes(&required_evidence_classes)
        .map_err(|message| state_error(&contract_path, message))?;
    if next_contract["scope"].as_array().is_none() {
        return Err(state_error(&contract_path, "Contract scope is malformed"));
    }
    super::validate_contract_amendment_revalidation_ready(&root, work_item_id, &previous_summary)?;

    let sequence = history.len() as u64 + 1;
    let previous_journal_digest = history
        .last()
        .map(|receipt| receipt.journal_digest.clone())
        .unwrap_or_else(|| Digest::sha256_bytes(ZERO_JOURNAL_DIGEST_SEED));
    let prepared = prepare_transaction(
        &root,
        AmendmentPreparation {
            work_item_id,
            request,
            runtime,
            previous_contract,
            new_contract: next_contract,
            previous_summary: &previous_summary,
            previous_journal_digest,
            sequence,
            legacy_input_digest,
        },
    )?;
    if let Some(authorization) = request.authorization.as_ref() {
        if prepared.prepared.repository_snapshot_digest != authorization.repository_snapshot_digest
        {
            return Err(state_error(
                &contract_path,
                "authorization_snapshot_mismatch: repository changed after amendment authorization",
            ));
        }
        #[cfg(test)]
        apply_snapshot_mutation_before_prepared_write(&root);
        let (latest_snapshot_digest, _) = observed_inputs(&root)?;
        if latest_snapshot_digest != authorization.repository_snapshot_digest
            || latest_snapshot_digest != prepared.prepared.repository_snapshot_digest
        {
            return Err(state_error(
                &contract_path,
                "authorization_snapshot_mismatch: repository changed before the amendment transaction was persisted",
            ));
        }
    }
    write_prepared(&root, &prepared)?;
    maybe_fail(AmendmentFailpoint::Prepared)?;
    let receipt = finish_prepared(&root, &prepared)?;
    history.push(receipt.clone());
    Ok(receipt)
}

#[cfg(test)]
mod tests {
    use super::{
        AmendmentFailpoint, mutate_snapshot_before_prepared_write, set_amendment_failpoint,
        validate_shared_worktree_amendment_state,
    };
    use crate::{
        ContractAmendmentReceipt, WorkItemStartOptions, amend_work_item_contract, attach,
        check_work_item_contract_amendment, checkpoint_work_item, preflight_work_item,
        read_work_item_contract_amendments, start_work_item_with_options,
    };
    use serde_json::{Value, json};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    fn repository() -> tempfile::TempDir {
        let directory = tempfile::tempdir().expect("repository tempdir");
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .current_dir(directory.path())
                .status()
                .expect("git init")
                .success()
        );
        attach(directory.path()).expect("attach repository");
        directory
    }

    fn linked_repository() -> (tempfile::TempDir, tempfile::TempDir) {
        let root_directory = tempfile::tempdir().expect("repository tempdir");
        let root = root_directory.path();
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .current_dir(root)
                .status()
                .expect("git init")
                .success()
        );
        for (key, value) in [
            ("user.name", "AI Cockpit test"),
            ("user.email", "test@example.invalid"),
        ] {
            assert!(
                Command::new("git")
                    .args(["config", key, value])
                    .current_dir(root)
                    .status()
                    .expect("configure Git identity")
                    .success()
            );
        }
        fs::write(root.join("README.md"), "linked-worktree fixture\n")
            .expect("write tracked fixture");
        assert!(
            Command::new("git")
                .args(["add", "README.md"])
                .current_dir(root)
                .status()
                .expect("git add")
                .success()
        );
        assert!(
            Command::new("git")
                .args(["commit", "-qm", "fixture"])
                .current_dir(root)
                .status()
                .expect("git commit")
                .success()
        );
        attach(root).expect("attach repository");

        let work_item_id = "WI-AMENDMENT-LINKED-RECOVERY";
        start_checkpointed(root, work_item_id);
        let peer_directory = tempfile::tempdir().expect("peer parent tempdir");
        let peer = peer_directory.path().join("peer");
        assert!(
            Command::new("git")
                .args(["worktree", "add", "--quiet", "-b", "amendment-peer"])
                .arg(&peer)
                .arg("HEAD")
                .current_dir(root)
                .status()
                .expect("create linked worktree")
                .success()
        );
        copy_tree(&root.join(".ai"), &peer.join(".ai"));
        (root_directory, peer_directory)
    }

    fn copy_tree(source: &Path, destination: &Path) {
        fs::create_dir_all(destination).expect("create copied directory");
        for entry in fs::read_dir(source).expect("read source directory") {
            let entry = entry.expect("read directory entry");
            let source_path = entry.path();
            let destination_path = destination.join(entry.file_name());
            let file_type = entry.file_type().expect("read entry type");
            if file_type.is_dir() {
                copy_tree(&source_path, &destination_path);
            } else if file_type.is_file() {
                fs::copy(&source_path, &destination_path).expect("copy file");
            }
        }
    }

    fn authorized_goal_amendment(root: &Path, work_item_id: &str, change_id: &str) -> Value {
        let contract_path = root
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.contract.json"));
        let contract = read_json(&contract_path);
        let request: cockpit_protocol::ContractAmendmentRequest = serde_json::from_value(json!({
            "schemaVersion": 1,
            "changeId": change_id,
            "expectedContractDigest": cockpit_protocol::digest_json(&contract)
                .expect("Contract digest"),
            "reason": "recover a prepared amendment before another linked worktree commits",
            "changes": [{
                "path": "/goal",
                "operation": "replace",
                "value": "amended after linked crash recovery"
            }]
        }))
        .expect("typed request");
        authorize_request(root, work_item_id, request)
    }

    fn authorize_request(
        root: &Path,
        work_item_id: &str,
        mut request: cockpit_protocol::ContractAmendmentRequest,
    ) -> Value {
        let check = check_work_item_contract_amendment(root, work_item_id, &request)
            .expect("read-only authorization binding");
        request.authorization = Some(cockpit_protocol::ContractAmendmentAuthorization {
            schema_version: cockpit_protocol::CONTRACT_AMENDMENT_AUTHORIZATION_SCHEMA_VERSION,
            decision_id: format!("test-{}", request.change_id),
            decision: cockpit_protocol::ContractAmendmentDecision::AuthorizeChange,
            authorized_by: "human:repository-test".into(),
            authority_source: "explicit process-level test authorization".into(),
            assurance: cockpit_protocol::EvidenceAssurance::SelfDeclared,
            executed_by: "agent:repository-test".into(),
            repository_id: check.repository_id,
            work_item_id: check.work_item_id,
            contract_digest: check.contract_digest,
            repository_snapshot_digest: check.repository_snapshot_digest,
            request_digest: check.request_digest,
            changed_paths: check.changed_paths,
        });
        serde_json::to_value(request).expect("authorized amendment request")
    }

    fn start_checkpointed(root: &Path, work_item_id: &str) -> PathBuf {
        start_work_item_with_options(
            root,
            work_item_id,
            "exercise amendment interruption recovery",
            "preserve one auditable plan change across a process interruption",
            &["crates/cockpit-repository/**".into()],
            &WorkItemStartOptions {
                authority: "authorized".into(),
                acceptance_criteria: vec!["the transaction recovers exactly once".into()],
                ..WorkItemStartOptions::default()
            },
        )
        .expect("start Work Item");
        let contract = root
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.contract.json"));
        preflight_work_item(root, &contract).expect("preflight");
        checkpoint_work_item(root, work_item_id).expect("checkpoint");
        contract
    }

    fn read_json(path: impl AsRef<Path>) -> Value {
        serde_json::from_slice(&fs::read(path).expect("read JSON file")).expect("valid JSON")
    }

    #[test]
    fn linked_worktree_pending_prepared_transaction_blocks_even_when_digest_matches() {
        let (root_directory, peer_directory) = linked_repository();
        let root = root_directory.path();
        let peer = peer_directory.path().join("peer");
        let work_item_id = "WI-AMENDMENT-LINKED-RECOVERY";
        let contract_path = root
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.contract.json"));
        let expected_digest = cockpit_protocol::digest_json(&read_json(&contract_path))
            .expect("expected Contract digest");
        let peer_request = authorized_goal_amendment(&peer, work_item_id, "linked-crash-pending");

        set_amendment_failpoint(AmendmentFailpoint::Prepared);
        let interrupted = amend_work_item_contract(
            &peer,
            work_item_id,
            &peer_request,
            "recover a prepared amendment before another linked worktree commits",
        )
        .expect_err("prepared journal is left at the injected process boundary");
        assert!(interrupted.to_string().contains("injected interruption"));
        let peer_contract = peer
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.contract.json"));
        assert_eq!(
            cockpit_protocol::digest_json(&read_json(&peer_contract)).expect("peer digest"),
            expected_digest,
            "crash simulation leaves the old Contract digest in place"
        );

        let root_contract = root
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.contract.json"));
        let root_summary = root
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.summary.json"));
        let peer_summary = peer
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.summary.json"));
        let root_contract_before = fs::read(&root_contract).expect("root Contract bytes");
        let root_summary_before = fs::read(&root_summary).expect("root Summary bytes");
        let peer_contract_before = fs::read(&peer_contract).expect("peer Contract bytes");
        let peer_summary_before = fs::read(&peer_summary).expect("peer Summary bytes");
        let peer_prepared = peer.join(".ai/evidence").join(format!(
            "{work_item_id}.contract-amendments/00000001.prepared.json"
        ));
        let peer_prepared_before = fs::read(&peer_prepared).expect("pending prepared bytes");

        let root_request = authorized_goal_amendment(root, work_item_id, "root-after-peer-pending");
        let blocked = amend_work_item_contract(
            root,
            work_item_id,
            &root_request,
            "recover a prepared amendment before another linked worktree commits",
        )
        .expect_err("a peer's pending transaction must block a second amendment");
        assert!(
            blocked
                .to_string()
                .contains("pending Contract amendment transaction"),
            "unexpected blocker: {blocked}"
        );
        assert_eq!(
            fs::read(&root_contract).expect("root Contract after rejection"),
            root_contract_before
        );
        assert_eq!(
            fs::read(&root_summary).expect("root Summary after rejection"),
            root_summary_before
        );
        assert_eq!(
            fs::read(&peer_contract).expect("peer Contract after rejection"),
            peer_contract_before
        );
        assert_eq!(
            fs::read(&peer_summary).expect("peer Summary after rejection"),
            peer_summary_before
        );
        assert_eq!(
            fs::read(&peer_prepared).expect("pending prepared remains"),
            peer_prepared_before
        );
        assert!(
            !root
                .join(".ai/evidence")
                .join(format!(
                    "{work_item_id}.contract-amendments/00000001.prepared.json"
                ))
                .exists(),
            "the rejected worktree must not create a prepared record"
        );
        assert!(
            !peer
                .join(".ai/evidence")
                .join(format!(
                    "{work_item_id}.contract-amendments/00000001.committed.json"
                ))
                .exists(),
            "the crash-pending transaction remains unresolved without a commit receipt"
        );
    }

    #[test]
    fn snapshot_drift_after_preparation_is_rejected_before_any_receipt_is_written() {
        let (root_directory, _peer_directory) = linked_repository();
        let root = root_directory.path();
        let work_item_id = "WI-AMENDMENT-LINKED-RECOVERY";
        let contract_path = root
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.contract.json"));
        let summary_path = root
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.summary.json"));
        let contract_before = fs::read(&contract_path).expect("Contract bytes before");
        let summary_before = fs::read(&summary_path).expect("Summary bytes before");
        let evidence_directory = root
            .join(".ai/evidence")
            .join(format!("{work_item_id}.contract-amendments"));
        assert!(
            !evidence_directory.exists(),
            "fixture starts with no amendment journal"
        );
        let source_path = root.join("README.md");
        assert_eq!(
            fs::read(&source_path).expect("source before"),
            b"linked-worktree fixture\n"
        );
        let request = authorized_goal_amendment(root, work_item_id, "snapshot-toctou");

        mutate_snapshot_before_prepared_write(
            PathBuf::from("README.md"),
            b"source changed after prepare and before first durable write\n".to_vec(),
        );
        let rejected = amend_work_item_contract(
            root,
            work_item_id,
            &request,
            "recover a prepared amendment before another linked worktree commits",
        )
        .expect_err("snapshot drift must invalidate the authorization before persistence");
        assert!(
            rejected.to_string().contains("snapshot"),
            "unexpected rejection: {rejected}"
        );
        assert_eq!(
            fs::read(&contract_path).expect("Contract bytes after"),
            contract_before
        );
        assert_eq!(
            fs::read(&summary_path).expect("Summary bytes after"),
            summary_before
        );
        assert_eq!(
            fs::read(&source_path).expect("source changed by deterministic race hook"),
            b"source changed after prepare and before first durable write\n"
        );
        assert!(
            !evidence_directory.exists(),
            "authorization mismatch must not create prepared or committed evidence"
        );
    }

    #[cfg(unix)]
    #[test]
    fn linked_worktree_canonicalization_loop_fails_closed() {
        let (root_directory, peer_directory) = linked_repository();
        let root = root_directory.path();
        let peer = peer_directory.path().join("peer");
        let work_item_id = "WI-AMENDMENT-LINKED-RECOVERY";
        let contract_path = root
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.contract.json"));
        let expected_digest = cockpit_protocol::digest_json(&read_json(&contract_path))
            .expect("expected Contract digest");

        fs::remove_dir_all(&peer).expect("remove linked worktree directory");
        std::os::unix::fs::symlink(&peer, &peer).expect("create a self-referential worktree path");

        let error = validate_shared_worktree_amendment_state(root, work_item_id, &expected_digest)
            .expect_err("non-NotFound peer path resolution errors must not be silently skipped");
        assert!(
            error.to_string().contains("linked") || error.to_string().contains("read"),
            "unexpected resolution failure: {error}"
        );
    }

    #[test]
    fn known_prunable_missing_linked_worktree_is_skipped() {
        let (root_directory, peer_directory) = linked_repository();
        let root = root_directory.path();
        let peer = peer_directory.path().join("peer");
        let work_item_id = "WI-AMENDMENT-LINKED-RECOVERY";
        let contract_path = root
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.contract.json"));
        let expected_digest = cockpit_protocol::digest_json(&read_json(&contract_path))
            .expect("expected Contract digest");
        fs::remove_dir_all(&peer).expect("remove prunable linked worktree");

        let records = crate::resource_lifecycle::git_worktree_records(root)
            .expect("inspect linked worktrees");
        let pruned = records
            .iter()
            .find(|record| {
                record.prunable
                    && record
                        .path
                        .as_deref()
                        .and_then(Path::file_name)
                        .is_some_and(|name| name == "peer")
            })
            .expect("Git retains the missing linked-worktree record");
        assert!(
            pruned.prunable,
            "Git must identify the absent peer as prunable"
        );
        validate_shared_worktree_amendment_state(root, work_item_id, &expected_digest)
            .expect("only the explicitly prunable missing peer is safely skipped");
    }

    #[test]
    fn amendment_recovers_exactly_once_at_every_durable_boundary() {
        let cases = [
            ("PREPARED", AmendmentFailpoint::Prepared, false, false),
            (
                "CONTRACT_REPLACED",
                AmendmentFailpoint::ContractReplaced,
                true,
                false,
            ),
            (
                "SUMMARY_PROJECTED",
                AmendmentFailpoint::SummaryProjected,
                true,
                true,
            ),
            (
                "COMMIT_MARKED",
                AmendmentFailpoint::CommitMarked,
                true,
                true,
            ),
        ];

        for (suffix, failpoint, contract_replaced, summary_projected) in cases {
            let directory = repository();
            let root = directory.path();
            let work_item_id = format!("WI-AMENDMENT-RECOVERY-{suffix}");
            let contract_path = start_checkpointed(root, &work_item_id);
            let summary_path = root
                .join(".ai/work-items/active")
                .join(format!("{work_item_id}.summary.json"));
            let original_contract = read_json(&contract_path);
            let original_summary_bytes = fs::read(&summary_path).expect("summary bytes");
            let reason = "resume the same exact amendment after an interrupted write";
            let request = json!({
                "schemaVersion": 1,
                "changeId": format!("recover-{suffix}"),
                "expectedContractDigest": cockpit_protocol::digest_json(&original_contract)
                    .expect("Contract digest"),
                "reason": reason,
                "changes": [{
                    "path": "/goal",
                    "operation": "replace",
                    "value": "recover this exact planned change once"
                }]
            });
            let request: cockpit_protocol::ContractAmendmentRequest =
                serde_json::from_value(request).expect("typed recovery request");
            let request = authorize_request(root, &work_item_id, request);

            set_amendment_failpoint(failpoint);
            let interruption = amend_work_item_contract(root, &work_item_id, &request, reason)
                .expect_err("selected durable boundary interrupts the first call");
            assert!(interruption.to_string().contains("injected interruption"));

            let current_contract = read_json(&contract_path);
            if contract_replaced {
                assert_eq!(
                    current_contract["goal"],
                    "recover this exact planned change once"
                );
            } else {
                assert_eq!(current_contract, original_contract);
            }
            let current_summary_bytes = fs::read(&summary_path).expect("current summary bytes");
            if summary_projected {
                assert_ne!(current_summary_bytes, original_summary_bytes);
            } else {
                assert_eq!(current_summary_bytes, original_summary_bytes);
            }

            let before_retry = read_work_item_contract_amendments(root, &work_item_id);
            if failpoint == AmendmentFailpoint::CommitMarked {
                assert_eq!(
                    before_retry.expect("commit marker is fully visible").len(),
                    1
                );
            } else {
                assert!(
                    before_retry
                        .expect_err("uncommitted journal remains fail-closed")
                        .to_string()
                        .contains("unresolved Contract amendment transaction")
                );
            }

            let retried = amend_work_item_contract(root, &work_item_id, &request, reason)
                .expect("retry recovers or returns the already committed receipt");
            let history = read_work_item_contract_amendments(root, &work_item_id)
                .expect("recovered history validates");
            assert_eq!(history.len(), 1, "no duplicate receipt for {suffix}");
            let mut retried_receipt = retried;
            let retried_fields = retried_receipt
                .as_object_mut()
                .expect("amendment result is an object");
            retried_fields.remove("stage");
            retried_fields.remove("recorded");
            retried_fields.remove("contractHash");
            let retried_receipt: ContractAmendmentReceipt =
                serde_json::from_value(retried_receipt).expect("typed retry receipt");
            assert_eq!(history[0], retried_receipt);
            assert_eq!(history[0].sequence, 1);
            let journal_directory = root
                .join(".ai/evidence")
                .join(format!("{work_item_id}.contract-amendments"));
            assert!(journal_directory.join("00000001.prepared.json").is_file());
            assert!(journal_directory.join("00000001.committed.json").is_file());
            assert_eq!(
                fs::read_dir(journal_directory)
                    .expect("journal directory")
                    .count(),
                2,
                "exactly one prepared and one committed record for {suffix}"
            );
        }
    }
}

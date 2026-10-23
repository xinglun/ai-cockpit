use super::ObserverError;
use cockpit_core::Digest;
use cockpit_protocol::{
    ACTIVATION_CONTINUITY_CAPABILITY, ActivationFieldDifference, ActivationOwnerBinding,
    ActivationRecoveryDecision, ActivationRecoveryHumanResponseSource, ActivationRecoveryRecord,
    ActivationRecoveryRequest, ActivationTransitionRecord, Contract, ContractAmendmentChange,
    ContractAmendmentError, ContractAmendmentFieldClass, ContractAmendmentOperation,
    ContractAmendmentRequest, ContractAmendmentValueChange, ContractSource, RuntimeContext,
    VerificationDeclaration, apply_contract_amendment, apply_contract_amendment_with_trace,
    contract_amendment_field_class, digest_json,
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

/// Runtime-captured local provenance for an agent technical review. It is
/// deliberately separate from HumanDecision/decisionEvidence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DelegatedDevelopmentReview {
    owner_contract: Value,
    owner_review: cockpit_protocol::PreflightDecisionEvidence,
    owner_receipt_path: String,
    owner_receipt_digest: Digest,
    policy_digest: Digest,
    // Capture the policy used at admission, including its absence. Historical
    // integrity must not depend on later mutable repository policy.
    repository_policy: Option<cockpit_protocol::GovernancePolicyDocument>,
    authority: cockpit_protocol::AuthorityEvidence,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegated_review: Option<DelegatedDevelopmentReview>,
    pub policy_review_required: bool,
    pub policy_review_requirements: Vec<String>,
    pub previous_journal_digest: Digest,
    pub journal_digest: Digest,
    pub recorded_at: String,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    delegated_review: Option<DelegatedDevelopmentReview>,
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

/// A historical Contract obtained from a completely validated committed
/// amendment chain. No caller can construct this from an arbitrary digest or
/// a mutable Summary invalidation marker.
pub(crate) struct ValidatedAmendmentPredecessor {
    contract: Contract,
    digest: Digest,
}

impl ValidatedAmendmentPredecessor {
    pub(crate) fn contract(&self) -> &Contract {
        &self.contract
    }

    pub(crate) fn digest(&self) -> &Digest {
        &self.digest
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CommittedRecord {
    sequence: u64,
    prepared_digest: Digest,
    receipt: ContractAmendmentReceipt,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct PreparedActivationTransition {
    record: ActivationTransitionRecord,
    record_digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ContinuityCommit {
    schema_version: u32,
    prepared_digest: Digest,
    recorded_at: String,
}

fn activation_directory(root: &Path, id: &str) -> PathBuf {
    root.join(".ai/evidence")
        .join(format!("{id}.activation-transition"))
}

fn recovery_directory(root: &Path, id: &str) -> PathBuf {
    root.join(".ai/evidence")
        .join(format!("{id}.activation-recovery"))
}

const RECOVERY_PLAN_ACTION: &str = "plan_activation_recovery";
const RECOVERY_RECORD_ACTION: &str = "record_activation_recovery";

struct LegacyActivationBoundary {
    before: Value,
    after: Value,
    boundary_journal: Digest,
    checkpoint_contract: Digest,
    differences: Vec<ActivationFieldDifference>,
}

fn checkpoint_sources_match(
    identity: &str,
    id: &str,
    history: &[ContractAmendmentReceipt],
    digest: &Digest,
    summary: &Value,
) -> bool {
    let Some(tail) = history.last() else {
        return false;
    };
    let Some(original) = history.iter().find(|receipt| receipt.checkpointed) else {
        return false;
    };
    let Some(records) = summary["checkpointEvidence"].as_array() else {
        return false;
    };
    let Some(before) = records
        .iter()
        .find(|record| record["stage"] == "before_edit")
    else {
        return false;
    };
    let Some(revalidation) = records
        .iter()
        .rfind(|record| record["stage"] == "contract_amendment_revalidation")
    else {
        return false;
    };
    let checks = json!(tail.invalidated_required_checks);
    summary["checkpointCount"] == json!(1)
        && [before, revalidation].iter().all(|record| {
            record["schemaVersion"] == json!(1)
                && record["recorded"] == json!(true)
                && record["repositoryId"] == identity
                && record["workItemId"] == id
                && record["recordedAt"]
                    .as_str()
                    .is_some_and(|time| chrono::DateTime::parse_from_rfc3339(time).is_ok())
        })
        && before["contractHash"] == json!(original.previous_contract_digest)
        && revalidation["originalBeforeEditContractHash"]
            == json!(original.previous_contract_digest)
        && revalidation["contractHash"] == json!(digest)
        && revalidation["previousContractHash"] == json!(tail.previous_contract_digest)
        && revalidation["repositorySnapshotDigest"] == json!(tail.repository_snapshot_digest)
        && revalidation["invalidatedRequiredChecks"] == checks
        && revalidation["verificationStarted"] == json!(tail.verification_started)
        && summary["verificationInvalidatedByContractAmendment"]["contractHash"] == json!(digest)
        && summary["verificationInvalidatedByContractAmendment"]["invalidatedRequiredChecks"]
            == checks
}

fn legacy_activation_boundary(
    root: &Path,
    id: &str,
    history: &[ContractAmendmentReceipt],
    contract: &Value,
    summary: &Value,
) -> Result<LegacyActivationBoundary, ObserverError> {
    let invalid = || {
        state_error(
            root,
            "legacy activation recovery requires exact start/checkpoint/history bindings",
        )
    };
    let identity = super::repository_id(root).to_string();
    let digest = record_digest(contract, root)?;
    if contract["repositoryId"] != identity
        || contract["workItemId"] != id
        || summary["repositoryId"] != identity
        || summary["workItemId"] != id
        || !summary["activationContinuity"].is_null()
        || !summary["activationRecovery"].is_null()
        || history
            .last()
            .is_none_or(|tail| tail.new_contract_digest != digest)
        || !checkpoint_sources_match(&identity, id, history, &digest, summary)
    {
        return Err(invalid());
    }
    let gaps: Vec<_> = history
        .windows(2)
        .enumerate()
        .filter(|(_, pair)| pair[0].new_contract_digest != pair[1].previous_contract_digest)
        .collect();
    if gaps.len() != 1 {
        return Err(invalid());
    }
    let (index, pair) = gaps[0];
    if pair[0].checkpointed
        || !pair[1].checkpointed
        || history[..=index].iter().any(|receipt| receipt.checkpointed)
    {
        return Err(state_error(
            root,
            "legacy recovery cannot explain a post-checkpoint Contract gap",
        ));
    }
    let a: PreparedRecord = read_typed_record(&amendment_record_path(
        root,
        id,
        pair[0].sequence,
        "prepared",
    ))?;
    let b: PreparedRecord = read_typed_record(&amendment_record_path(
        root,
        id,
        pair[1].sequence,
        "prepared",
    ))?;
    let before = a.prepared.new_contract;
    let after = b.prepared.previous_contract;
    let typed: Contract = serde_json::from_value(after.clone())
        .map_err(|error| state_error(root, error.to_string()))?;
    typed
        .validate()
        .map_err(|errors| state_error(root, errors.join("; ")))?;
    let differences = activation_differences(&before, &after, true)?;
    let start = &summary["lifecycleFacts"]["start"];
    if start["eventType"] != "work_item_started"
        || start["occurredAt"] != start["recordedAt"]
        || start["actorProvenance"] != "unknown"
        || start["recordedAt"]
            .as_str()
            .is_none_or(|time| chrono::DateTime::parse_from_rfc3339(time).is_err())
        || start["runtimeVersion"]
            .as_str()
            .is_none_or(|version| version.trim().is_empty())
        || start["runtimeDigest"]
            .as_str()
            .is_none_or(|digest| digest.parse::<Digest>().is_err())
    {
        return Err(invalid());
    }
    Ok(LegacyActivationBoundary {
        before,
        after,
        boundary_journal: pair[0].journal_digest.clone(),
        checkpoint_contract: pair[1].previous_contract_digest.clone(),
        differences,
    })
}

fn bound_raw_bytes(path: &Path) -> Result<Vec<u8>, ObserverError> {
    if !super::is_regular_non_symlink(path)? {
        return Err(state_error(path, "recovery input must be a regular file"));
    }
    let metadata = fs::metadata(path).map_err(|source| ObserverError::Read {
        path: path.into(),
        source,
    })?;
    if metadata.len() > 4 * 1024 * 1024 {
        return Err(state_error(
            path,
            "recovery metadata input exceeds its bound",
        ));
    }
    fs::read(path).map_err(|source| ObserverError::Read {
        path: path.into(),
        source,
    })
}

fn recovery_store(
    root: &Path,
    runtime: &RuntimeContext,
) -> Result<super::CoordinationStore, ObserverError> {
    if runtime.protocol_version != cockpit_protocol::PROTOCOL_VERSION {
        return Err(state_error(
            root,
            "unsupported activation recovery Runtime protocol",
        ));
    }
    let git = cockpit_git::GitRepository::discover(root)
        .map_err(|error| state_error(root, error.to_string()))?;
    super::CoordinationStore::open_read_only(
        &git,
        cockpit_protocol::RuntimeCapabilityBinding {
            schema_version: 1,
            runtime_version: runtime.runtime_version.clone(),
            runtime_digest: runtime.runtime_digest.clone(),
            capability: cockpit_protocol::ENVIRONMENT_DRIFT_CAPABILITY.into(),
        },
    )
    .map_err(|error| state_error(root, error.to_string()))
}

fn recovery_raw_inputs(
    root: &Path,
    id: &str,
    sequence: u64,
    store: &super::CoordinationStore,
) -> Result<BTreeMap<String, Vec<u8>>, ObserverError> {
    let mut inputs = BTreeMap::new();
    for (name, path) in [
        (
            "contract".to_owned(),
            root.join(format!(".ai/work-items/active/{id}.contract.json")),
        ),
        (
            "summary".to_owned(),
            root.join(format!(".ai/work-items/active/{id}.summary.json")),
        ),
        ("registration".to_owned(), store.registration_path(id)),
    ] {
        inputs.insert(name, bound_raw_bytes(&path)?);
    }
    for number in 1..=sequence {
        for phase in ["prepared", "committed"] {
            inputs.insert(
                format!("amendment/{number:08}.{phase}.json"),
                bound_raw_bytes(&amendment_record_path(root, id, number, phase))?,
            );
        }
    }
    let review = root.join(format!(".ai/decisions/{id}.preflight-review.json"));
    let bytes = if review.try_exists().map_err(|source| ObserverError::Read {
        path: review.clone(),
        source,
    })? {
        bound_raw_bytes(&review)?
    } else {
        Vec::new()
    };
    if !bytes.is_empty() {
        let evidence: cockpit_protocol::PreflightDecisionEvidence = serde_json::from_slice(&bytes)
            .map_err(|error| state_error(&review, error.to_string()))?;
        if evidence.schema_version != 1
            || evidence.work_item_id != id
            || evidence.repository_id != super::repository_id(root).to_string()
            || evidence.decision_id != "contract-preflight-review"
            || evidence.decision != "confirm_review"
        {
            return Err(state_error(
                &review,
                "foreign or contradictory historical review context",
            ));
        }
    }
    // Empty bytes bind observed absence; they are not a fabricated receipt.
    inputs.insert("preflight-review".into(), bytes);
    Ok(inputs)
}

fn recovery_environment_digest(root: &Path) -> Result<Digest, ObserverError> {
    Ok(super::RepositoryExecutionContext::capture(root)?
        .observe_environment_identity()?
        .environment_digest)
}

fn recovery_request_digest(request: &ActivationRecoveryRequest) -> Result<Digest, ObserverError> {
    let mut value = serde_json::to_value(request)
        .map_err(|error| state_error("recovery-request", error.to_string()))?;
    value
        .as_object_mut()
        .expect("typed request object")
        .remove("requestDigest");
    record_digest(&value, Path::new("recovery-request"))
}

fn build_activation_recovery_request(
    root: &Path,
    id: &str,
    runtime: &RuntimeContext,
) -> Result<ActivationRecoveryRequest, ObserverError> {
    super::validate_work_item_id(id)?;
    if activation_directory(root, id)
        .try_exists()
        .map_err(|source| ObserverError::Read {
            path: root.into(),
            source,
        })?
        || recovery_directory(root, id)
            .try_exists()
            .map_err(|source| ObserverError::Read {
                path: root.into(),
                source,
            })?
    {
        return Err(state_error(
            root,
            "existing or partial continuity evidence cannot be replaced by legacy recovery",
        ));
    }
    let history = read_work_item_contract_amendments(root, id)?;
    let contract =
        super::read_json(&root.join(format!(".ai/work-items/active/{id}.contract.json")))?;
    let summary = super::read_json(&root.join(format!(".ai/work-items/active/{id}.summary.json")))?;
    let boundary = legacy_activation_boundary(root, id, &history, &contract, &summary)?;
    let tail = history
        .last()
        .ok_or_else(|| state_error(root, "recovery journal is empty"))?;
    let store = recovery_store(root, runtime)?;
    let raw = recovery_raw_inputs(root, id, tail.sequence, &store)?;
    if serde_json::from_slice::<Value>(&raw["contract"])
        .ok()
        .as_ref()
        != Some(&contract)
        || serde_json::from_slice::<Value>(&raw["summary"])
            .ok()
            .as_ref()
            != Some(&summary)
    {
        return Err(state_error(
            root,
            "activation recovery inputs changed during observation",
        ));
    }
    let registration: cockpit_protocol::WorktreeRegistration =
        serde_json::from_slice(&raw["registration"])
            .map_err(|error| state_error(root, error.to_string()))?;
    let owner = observed_activation_owner(root)?;
    let head = cockpit_git::GitRepository::discover(root)
        .and_then(|git| git.topology())
        .map_err(|error| state_error(root, error.to_string()))?
        .head
        .ok_or_else(|| state_error(root, "missing recovery HEAD"))?;
    if registration.work_item_id != id
        || registration.repository_id.to_string() != super::repository_id(root).to_string()
        || registration.contract_digest != record_digest(&contract, root)?
        || registration.worktree_path != owner.worktree_path
        || registration.branch != owner.branch
        || registration.head != head
        || registration.runtime.runtime_digest != runtime.runtime_digest
        || registration.runtime.runtime_version != runtime.runtime_version
        || store
            .environment_drift_pending(&cockpit_protocol::EnvironmentDriftRequest {
                schema_version: 1,
                work_item_id: id.into(),
                expected_generation: registration.generation,
            })
            .map_err(|error| state_error(root, error.to_string()))?
    {
        return Err(state_error(
            root,
            "activation recovery requires a current exact owner/Runtime registration without pending drift",
        ));
    }
    let mut request = ActivationRecoveryRequest {
        schema_version: 1, capability: ACTIVATION_CONTINUITY_CAPABILITY.into(),
        repository_id: super::repository_id(root).to_string(), work_item_id: id.into(), owner,
        runtime: runtime.clone(), head, registration_generation: registration.generation,
        registration_digest: Digest::sha256_bytes(&raw["registration"]),
        contract_digest: record_digest(&contract, root)?, summary_digest: record_digest(&summary, root)?,
        repository_snapshot_digest: observed_inputs(root)?.0,
        environment_digest: recovery_environment_digest(root)?,
        journal_sequence: tail.sequence, journal_digest: tail.journal_digest.clone(),
        boundary_journal_digest: boundary.boundary_journal,
        before_activation_contract_digest: record_digest(&boundary.before, root)?,
        after_activation_contract_digest: record_digest(&boundary.after, root)?,
        checkpoint_contract_digest: boundary.checkpoint_contract,
        start_fact_digest: record_digest(&summary["lifecycleFacts"]["start"], root)?,
        differences: boundary.differences,
        raw_evidence_digests: raw.iter().map(|(name, bytes)| (name.clone(), Digest::sha256_bytes(bytes))).collect(),
        stage: "before_first_checkpoint".into(),
        retained_unknowns: vec!["historical_activation_argv_missing".into(), "historical_transition_authentication_unknown".into(), "human_identity_self_declared".into()],
        required_response: "accept_present_baseline".into(),
        question: "Accept only this exact present baseline and enumerated differences with historical activation authentication unknown, for a new Contract preflight request? This does not approve Contract C or business actions.".into(),
        historical_transition_authentication: "unknown".into(), business_authority_granted: false,
        fresh_contract_review_required: true, request_digest: Digest::sha256_bytes(b"unsealed request"),
    };
    request.request_digest = recovery_request_digest(&request)?;
    Ok(request)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PreparedActivationRecovery {
    record: ActivationRecoveryRecord,
    record_digest: Digest,
}

fn recovery_summary(previous: &Value, request: &ActivationRecoveryRequest, at: &str) -> Value {
    let mut summary = previous.clone();
    summary["preflightState"] = json!("not_run");
    summary["activationRecovery"] = json!({
        "schemaVersion":1, "requestDigest":request.request_digest, "recordedAt":at,
        "historicalTransitionAuthentication":"unknown", "businessAuthorityGranted":false,
        "freshContractReviewRequired":true,
    });
    summary
}

fn response_source_path(root: &Path, id: &str, relative: &str) -> Result<PathBuf, ObserverError> {
    let path = Path::new(relative);
    let prefix = PathBuf::from(format!(".ai/evidence/{id}"));
    if !path.starts_with(&prefix)
        || path.extension().is_none_or(|extension| extension != "json")
        || path
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(state_error(
            path,
            "recovery response source must be an exact own-WI evidence JSON path",
        ));
    }
    let full = root.join(path);
    let canonical = fs::canonicalize(&full).map_err(|source| ObserverError::Read {
        path: full.clone(),
        source,
    })?;
    if canonical != full || !super::is_regular_non_symlink(&full)? {
        return Err(state_error(
            full,
            "recovery response source must not traverse symlinks",
        ));
    }
    Ok(full)
}

fn validate_recovery_response(
    root: &Path,
    id: &str,
    request: &ActivationRecoveryRequest,
    decision: &ActivationRecoveryDecision,
    bytes: &[u8],
) -> Result<(), ObserverError> {
    let source: ActivationRecoveryHumanResponseSource =
        serde_json::from_slice(bytes).map_err(|error| {
            state_error(
                root,
                format!("invalid newly supplied human response source: {error}"),
            )
        })?;
    if bytes.len() > 65536
        || decision.schema_version != 1
        || decision.decision_id != "activation-present-baseline-review"
        || decision.response != "accept_present_baseline"
        || decision.request_digest != request.request_digest
        || decision.assurance != "self_declared"
        || !decision.human_actor.starts_with("human:")
        || decision.human_actor.len() <= "human:".len()
        || decision.human_actor["human:".len()..].trim().is_empty()
        || decision.human_actor.trim() != decision.human_actor
        || decision.human_actor.chars().any(char::is_control)
        || decision.recorder_actor.trim().is_empty()
        || decision.recorder_actor.trim() != decision.recorder_actor
        || decision.recorder_actor.chars().any(char::is_control)
        || decision.human_actor == decision.recorder_actor
        || decision.reason.trim().is_empty()
        || chrono::DateTime::parse_from_rfc3339(&decision.responded_at).is_err()
        || decision.authority_source_digest != Digest::sha256_bytes(bytes)
        || source.schema_version != 1
        || source.kind != "human_activation_recovery_response_source"
        || source.request_digest != request.request_digest
        || source.response != decision.response
        || source.human_actor != decision.human_actor
        || source.responded_at != decision.responded_at
        || source.provenance.trim().is_empty()
    {
        return Err(state_error(
            root,
            format!(
                "{id}: exact newly supplied present-baseline response is required; old review is not recovery authority"
            ),
        ));
    }
    Ok(())
}

fn read_activation_recovery(
    root: &Path,
    id: &str,
    history: &[ContractAmendmentReceipt],
) -> Result<Option<ActivationRecoveryRecord>, ObserverError> {
    let directory = recovery_directory(root, id);
    if !continuity_record_directory(&directory)? {
        return Ok(None);
    }
    if !directory
        .join("prepared.json")
        .try_exists()
        .map_err(|source| ObserverError::Read {
            path: directory.clone(),
            source,
        })?
        || !directory
            .join("committed.json")
            .try_exists()
            .map_err(|source| ObserverError::Read {
                path: directory.clone(),
                source,
            })?
    {
        return Err(state_error(
            directory,
            "unresolved activation recovery transaction",
        ));
    }
    let prepared: PreparedActivationRecovery = read_typed_record(&directory.join("prepared.json"))?;
    let commit: ContinuityCommit = read_typed_record(&directory.join("committed.json"))?;
    let record = prepared.record;
    let request = &record.request;
    let invalid = || {
        state_error(
            &directory,
            "activation recovery record binding or retained evidence is invalid",
        )
    };
    let prefix_length = usize::try_from(request.journal_sequence).map_err(|_| invalid())?;
    if prefix_length == 0
        || prefix_length > history.len()
        || prepared.record_digest != record_digest(&record, &directory)?
        || commit.schema_version != 1
        || commit.prepared_digest != prepared.record_digest
        || chrono::DateTime::parse_from_rfc3339(&commit.recorded_at).is_err()
        || record.schema_version != 1
        || record.capability != ACTIVATION_CONTINUITY_CAPABILITY
        || request.schema_version != 1
        || request.capability != ACTIVATION_CONTINUITY_CAPABILITY
        || request.repository_id != super::repository_id(root).to_string()
        || request.work_item_id != id
        || request.owner != observed_activation_owner(root)?
        || request.runtime.protocol_version != cockpit_protocol::PROTOCOL_VERSION
        || request.runtime.runtime_version.trim().is_empty()
        || request.runtime.runtime_version == "0.2.105"
        || !super::valid_git_object_id(&request.head)
        || request.registration_generation == 0
        || request.stage != "before_first_checkpoint"
        || request.required_response != "accept_present_baseline"
        || request.question.trim().is_empty()
        || request.retained_unknowns
            != [
                "historical_activation_argv_missing",
                "historical_transition_authentication_unknown",
                "human_identity_self_declared",
            ]
        || request.historical_transition_authentication != "unknown"
        || request.business_authority_granted
        || !request.fresh_contract_review_required
        || record.historical_transition_authentication != "unknown"
        || record.business_authority_granted
        || !record.fresh_contract_review_required
        || request.request_digest != recovery_request_digest(request)?
        || record.raw_evidence_snapshots.len() != prefix_length * 2 + 4
        || record.raw_evidence_snapshots.len() != request.raw_evidence_digests.len()
        || record.raw_evidence_snapshots.iter().any(|(name, bytes)| {
            request.raw_evidence_digests.get(name) != Some(&Digest::sha256_bytes(bytes))
        })
    {
        return Err(invalid());
    }
    let decode = |name: &str| -> Result<Value, ObserverError> {
        serde_json::from_slice(
            record
                .raw_evidence_snapshots
                .get(name)
                .ok_or_else(invalid)?,
        )
        .map_err(|_| invalid())
    };
    let contract = decode("contract")?;
    let summary = decode("summary")?;
    let registration: cockpit_protocol::WorktreeRegistration =
        serde_json::from_value(decode("registration")?).map_err(|_| invalid())?;
    let review = record
        .raw_evidence_snapshots
        .get("preflight-review")
        .ok_or_else(invalid)?;
    if !review.is_empty() {
        let evidence: cockpit_protocol::PreflightDecisionEvidence =
            serde_json::from_slice(review).map_err(|_| invalid())?;
        if evidence.schema_version != 1
            || evidence.repository_id != request.repository_id
            || evidence.work_item_id != id
            || evidence.decision_id != "contract-preflight-review"
            || evidence.decision != "confirm_review"
        {
            return Err(invalid());
        }
    }
    let boundary =
        legacy_activation_boundary(root, id, &history[..prefix_length], &contract, &summary)?;
    if record.previous_summary != summary
        || request.contract_digest != record_digest(&contract, &directory)?
        || request.summary_digest != record_digest(&summary, &directory)?
        || request.registration_digest
            != Digest::sha256_bytes(&record.raw_evidence_snapshots["registration"])
        || registration.work_item_id != id
        || registration.repository_id.to_string() != request.repository_id
        || registration.contract_digest != request.contract_digest
        || registration.head != request.head
        || registration.worktree_path != request.owner.worktree_path
        || registration.branch != request.owner.branch
        || registration.generation != request.registration_generation
        || registration.runtime.runtime_version != request.runtime.runtime_version
        || registration.runtime.runtime_digest != request.runtime.runtime_digest
        || registration.runtime.capability != cockpit_protocol::ENVIRONMENT_DRIFT_CAPABILITY
        || registration.runtime.validate_candidate().is_err()
        || request.journal_digest != history[prefix_length - 1].journal_digest
        || request.boundary_journal_digest != boundary.boundary_journal
        || request.before_activation_contract_digest != record_digest(&boundary.before, &directory)?
        || request.after_activation_contract_digest != record_digest(&boundary.after, &directory)?
        || request.checkpoint_contract_digest != boundary.checkpoint_contract
        || request.start_fact_digest
            != record_digest(&summary["lifecycleFacts"]["start"], &directory)?
        || request.differences != boundary.differences
        || record.projected_summary_digest
            != record_digest(
                &recovery_summary(&summary, request, &record.recorded_at),
                &directory,
            )?
        || chrono::DateTime::parse_from_rfc3339(&record.recorded_at).is_err()
    {
        return Err(invalid());
    }
    for receipt in &history[..prefix_length] {
        for phase in ["prepared", "committed"] {
            let name = format!("amendment/{:08}.{phase}.json", receipt.sequence);
            if record.raw_evidence_snapshots.get(&name)
                != Some(&bound_raw_bytes(&amendment_record_path(
                    root,
                    id,
                    receipt.sequence,
                    phase,
                ))?)
            {
                return Err(invalid());
            }
        }
    }
    let source_path = response_source_path(root, id, &record.decision.authority_source_path)?;
    if bound_raw_bytes(&source_path)? != record.authority_source_bytes {
        return Err(invalid());
    }
    validate_recovery_response(
        root,
        id,
        request,
        &record.decision,
        &record.authority_source_bytes,
    )?;
    let current_summary =
        super::read_json(&root.join(format!(".ai/work-items/active/{id}.summary.json")))?;
    let mut marker = current_summary["activationRecovery"].clone();
    let review_request = marker
        .as_object_mut()
        .and_then(|value| value.remove("reviewRequest"));
    if marker != recovery_summary(&summary, request, &record.recorded_at)["activationRecovery"]
        || review_request.as_ref().is_some_and(|binding| {
            binding.as_object().is_none_or(|object| object.len() != 3)
                || [
                    "contractDigest",
                    "preflightDecisionDigest",
                    "repositorySnapshotDigest",
                ]
                .iter()
                .any(|field| {
                    binding[*field]
                        .as_str()
                        .is_none_or(|value| value.parse::<Digest>().is_err())
                })
        })
    {
        return Err(state_error(
            directory,
            "activation recovery Summary projection is missing or mismatched",
        ));
    }
    Ok(Some(record))
}

/// Read-only: return a request only for the exact finite legacy boundary.
pub fn plan_activation_recovery(
    root: &Path,
    id: &str,
    runtime: &RuntimeContext,
) -> Result<ActivationRecoveryRequest, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let request = build_activation_recovery_request(&root, id, runtime)?;
    super::require_current_action_admission(&root, id, RECOVERY_PLAN_ACTION, runtime)?;
    if request != build_activation_recovery_request(&root, id, runtime)? {
        return Err(state_error(
            root,
            "activation recovery request changed while planning",
        ));
    }
    Ok(request)
}

/// Append a present-time baseline acceptance. This never records a Contract
/// approval and cannot authenticate the missing historical transition.
pub fn record_activation_recovery(
    root: &Path,
    id: &str,
    decision: &ActivationRecoveryDecision,
    runtime: &RuntimeContext,
) -> Result<ActivationRecoveryRecord, ObserverError> {
    super::validate_work_item_id(id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    super::require_current_action_admission(&root, id, RECOVERY_RECORD_ACTION, runtime)?;
    let _lifecycle = super::acquire_lifecycle_lock(&root, id)?;
    let store = recovery_store(&root, runtime)?;
    store
        .with_lock(|| {
            let operation = || -> Result<ActivationRecoveryRecord, ObserverError> {
                super::require_current_action_admission(
                    &root,
                    id,
                    RECOVERY_RECORD_ACTION,
                    runtime,
                )?;
                let request = build_activation_recovery_request(&root, id, runtime)?;
                if request.request_digest != decision.request_digest {
                    return Err(state_error(
                        &root,
                        "stale activation recovery request; no write performed",
                    ));
                }
                let source_path = response_source_path(&root, id, &decision.authority_source_path)?;
                let source = bound_raw_bytes(&source_path)?;
                validate_recovery_response(&root, id, &request, decision, &source)?;
                let raw = recovery_raw_inputs(&root, id, request.journal_sequence, &store)?;
                let previous_summary: Value = serde_json::from_slice(&raw["summary"])
                    .map_err(|error| state_error(&root, error.to_string()))?;
                let at = super::usage::lifecycle_now();
                let projected = recovery_summary(&previous_summary, &request, &at);
                let record = ActivationRecoveryRecord {
                    schema_version: 1,
                    capability: ACTIVATION_CONTINUITY_CAPABILITY.into(),
                    request: request.clone(),
                    decision: decision.clone(),
                    previous_summary,
                    projected_summary_digest: record_digest(&projected, &root)?,
                    raw_evidence_snapshots: raw.clone(),
                    authority_source_bytes: source.clone(),
                    historical_transition_authentication: "unknown".into(),
                    business_authority_granted: false,
                    fresh_contract_review_required: true,
                    recorded_at: at,
                };
                // Exact CAS is repeated under both locks immediately before the
                // first immutable publication. Never finish unrelated prepared work.
                if request != build_activation_recovery_request(&root, id, runtime)?
                    || raw != recovery_raw_inputs(&root, id, request.journal_sequence, &store)?
                    || source != bound_raw_bytes(&source_path)?
                {
                    return Err(state_error(
                        &root,
                        "activation recovery CAS changed before publication",
                    ));
                }
                let directory = recovery_directory(&root, id);
                let prepared = PreparedActivationRecovery {
                    record_digest: record_digest(&record, &root)?,
                    record: record.clone(),
                };
                write_immutable_record(&directory.join("prepared.json"), &prepared)?;
                durable_replace_json(
                    &root.join(format!(".ai/work-items/active/{id}.summary.json")),
                    &projected,
                )?;
                let current_raw = recovery_raw_inputs(&root, id, request.journal_sequence, &store)?;
                let current_head = cockpit_git::GitRepository::discover(&root)
                    .and_then(|git| git.topology())
                    .map_err(|error| state_error(&root, error.to_string()))?
                    .head;
                if super::read_json(&root.join(format!(".ai/work-items/active/{id}.summary.json")))?
                    != projected
                    || raw
                        .iter()
                        .filter(|(name, _)| name.as_str() != "summary")
                        .any(|(name, bytes)| current_raw.get(name) != Some(bytes))
                    || source != bound_raw_bytes(&source_path)?
                    || observed_inputs(&root)?.0 != request.repository_snapshot_digest
                    || recovery_environment_digest(&root)? != request.environment_digest
                    || current_head.as_deref() != Some(request.head.as_str())
                    || observed_activation_owner(&root)? != request.owner
                {
                    return Err(state_error(
                        &root,
                        "activation recovery changed before commit; prepared state retained",
                    ));
                }
                write_immutable_record(
                    &directory.join("committed.json"),
                    &ContinuityCommit {
                        schema_version: 1,
                        prepared_digest: prepared.record_digest,
                        recorded_at: super::usage::lifecycle_now(),
                    },
                )?;
                Ok(record)
            };
            operation()
                .map_err(|error| super::CoordinationError::RecoveryRequired(error.to_string()))
        })
        .map_err(|error| state_error(&root, error.to_string()))
}

pub(crate) fn activation_recovery_request_if_needed(
    root: &Path,
    id: &str,
    runtime: &RuntimeContext,
    history: &[ContractAmendmentReceipt],
    current: &Digest,
) -> Result<Option<ActivationRecoveryRequest>, ObserverError> {
    match validate_transition_chain(root, id, history, Some(current)) {
        Ok(()) => Ok(None),
        Err(error) => {
            if activation_directory(root, id).exists() || recovery_directory(root, id).exists() {
                return Err(error);
            }
            build_activation_recovery_request(root, id, runtime).map(Some)
        }
    }
}

pub(crate) fn activation_recovery_needs_fresh_review(
    root: &Path,
    id: &str,
    digest: &Digest,
    summary: &Value,
) -> Result<Option<(ActivationRecoveryRequest, bool)>, ObserverError> {
    let history = read_work_item_contract_amendments(root, id)?;
    let Some(record) = read_activation_recovery(root, id, &history)? else {
        return Ok(None);
    };
    let evidence = &summary["decisionEvidence"];
    let new_time = evidence["recordedAt"]
        .as_str()
        .and_then(|time| chrono::DateTime::parse_from_rfc3339(time).ok());
    let record_time = chrono::DateTime::parse_from_rfc3339(&record.recorded_at)
        .map_err(|error| state_error(root, error.to_string()))?;
    let binding = &summary["activationRecovery"]["reviewRequest"];
    let preflight_generated = binding["contractDigest"] == json!(digest)
        && binding["preflightDecisionDigest"].is_string()
        && binding["preflightDecisionDigest"] != record.previous_summary["preflightDecisionDigest"];
    let fresh = preflight_generated
        && super::governance_controls::preflight_decision_evidence_state(root, id, digest)
            == super::governance_controls::PreflightDecisionEvidenceState::Valid
        && evidence != &record.previous_summary["decisionEvidence"]
        && evidence["preflightDecisionDigest"] == binding["preflightDecisionDigest"]
        && evidence["repositorySnapshotDigest"] == binding["repositorySnapshotDigest"]
        && new_time.is_some_and(|time| time >= record_time);
    Ok((!fresh).then_some((record.request, preflight_generated)))
}

fn observed_activation_owner(root: &Path) -> Result<ActivationOwnerBinding, ObserverError> {
    let git = cockpit_git::GitRepository::discover(root)
        .map_err(|error| state_error(root, error.to_string()))?;
    let topology = git
        .topology()
        .map_err(|error| state_error(root, error.to_string()))?;
    Ok(ActivationOwnerBinding {
        worktree_path: topology.repository_root.to_string_lossy().into_owned(),
        common_directory_id: Digest::sha256_bytes(topology.common_dir.to_string_lossy().as_bytes()),
        branch: topology
            .branch
            .ok_or_else(|| state_error(root, "activation owner requires a named branch"))?,
    })
}

fn activation_differences(
    before: &Value,
    after: &Value,
    legacy_csv: bool,
) -> Result<Vec<ActivationFieldDifference>, ObserverError> {
    let invalid = || {
        state_error(
            "activation-transition",
            "unexplained activation Contract change",
        )
    };
    if before["state"] != "not_ready" || after["state"] != "implementation_active" {
        return Err(invalid());
    }
    let a = before.as_object().ok_or_else(invalid)?;
    let b = after.as_object().ok_or_else(invalid)?;
    if a.keys().ne(b.keys()) {
        return Err(invalid());
    }
    let mut differences = Vec::new();
    for (key, old) in a {
        let new = &b[key];
        if old == new {
            continue;
        }
        let permitted = matches!(
            key.as_str(),
            "state" | "baseRevision" | "repositorySnapshotDigest"
        ) || (legacy_csv
            && key == "outOfScope"
            && old.as_array().is_some_and(|values| {
                let mut split = Vec::new();
                for value in values {
                    let Some(value) = value.as_str() else {
                        return false;
                    };
                    for part in value.split(',') {
                        if part.is_empty() {
                            return false;
                        }
                        split.push(Value::String(part.into()));
                    }
                }
                *new == Value::Array(split)
            }));
        if !permitted {
            return Err(state_error(
                "activation-transition",
                format!("unexplained activation Contract change at /{key}; amend the plan first"),
            ));
        }
        differences.push(ActivationFieldDifference {
            path: format!("/{key}"),
            old_value: old.clone(),
            new_value: new.clone(),
        });
    }
    Ok(differences)
}

fn activation_input_digest(contract: &Value) -> Result<Digest, ObserverError> {
    let fields = [
        "intent",
        "goal",
        "scope",
        "outOfScope",
        "risk",
        "authority",
        "acceptanceCriteria",
        "requiredEvidenceClasses",
        "verification",
    ];
    let input = fields
        .into_iter()
        .map(|key| (key.to_owned(), contract[key].clone()))
        .collect::<BTreeMap<_, _>>();
    record_digest(&input, Path::new("activation-input"))
}

fn continuity_record_directory(path: &Path) -> Result<bool, ObserverError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(source) => {
            return Err(ObserverError::Read {
                path: path.into(),
                source,
            });
        }
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(state_error(
            path,
            "continuity evidence directory must be regular",
        ));
    }
    for entry in fs::read_dir(path).map_err(|source| ObserverError::Read {
        path: path.into(),
        source,
    })? {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: path.into(),
            source,
        })?;
        if !matches!(
            entry.file_name().to_str(),
            Some("prepared.json" | "committed.json")
        ) || !super::is_regular_non_symlink(&entry.path())?
        {
            return Err(state_error(
                path,
                "unexpected or non-regular continuity evidence",
            ));
        }
    }
    Ok(true)
}

fn read_activation_transition(
    root: &Path,
    id: &str,
) -> Result<Option<PreparedActivationTransition>, ObserverError> {
    let directory = activation_directory(root, id);
    if !continuity_record_directory(&directory)? {
        return Ok(None);
    }
    let prepared_path = directory.join("prepared.json");
    let committed_path = directory.join("committed.json");
    // The directory inventory above has already rejected non-regular entries.
    // Missing one phase is a partial transaction, not an absent file error.
    let prepared_exists = prepared_path
        .try_exists()
        .map_err(|source| ObserverError::Read {
            path: prepared_path.clone(),
            source,
        })?;
    let committed_exists = committed_path
        .try_exists()
        .map_err(|source| ObserverError::Read {
            path: committed_path.clone(),
            source,
        })?;
    if !prepared_exists && !committed_exists {
        return Ok(None);
    }
    if !prepared_exists || !committed_exists {
        return Err(state_error(
            directory,
            "unresolved activation transition transaction",
        ));
    }
    let prepared: PreparedActivationTransition = read_typed_record(&prepared_path)?;
    let committed: ContinuityCommit = read_typed_record(&committed_path)?;
    let record = &prepared.record;
    let expected_start = json!({
        "eventType":"work_item_started", "occurredAt":record.recorded_at,
        "recordedAt":record.recorded_at, "actorProvenance":"unknown",
        "runtimeVersion":record.runtime.runtime_version,
        "runtimeDigest":record.runtime.runtime_digest,
    });
    if prepared.record_digest != record_digest(record, &prepared_path)?
        || committed.schema_version != 1
        || committed.prepared_digest != prepared.record_digest
        || chrono::DateTime::parse_from_rfc3339(&committed.recorded_at).is_err()
        || record.schema_version != 1
        || record.capability != ACTIVATION_CONTINUITY_CAPABILITY
        || record.repository_id != super::repository_id(root).to_string()
        || record.work_item_id != id
        || record.owner != observed_activation_owner(root)?
        || record.runtime.protocol_version != cockpit_protocol::PROTOCOL_VERSION
        || record.runtime.runtime_version.trim().is_empty()
        || !super::valid_git_object_id(&record.observed_head)
        || record.previous_contract["repositoryId"] != json!(record.repository_id)
        || record.new_contract["repositoryId"] != json!(record.repository_id)
        || record.previous_contract["workItemId"] != id
        || record.new_contract["workItemId"] != id
        || record.previous_contract_digest
            != record_digest(&record.previous_contract, &prepared_path)?
        || record.new_contract_digest != record_digest(&record.new_contract, &prepared_path)?
        || record.previous_summary_digest
            != record_digest(&record.previous_summary, &prepared_path)?
        || record.new_summary_digest != record_digest(&record.new_summary, &prepared_path)?
        || record.previous_summary["state"] != "not_ready"
        || record.previous_summary["checkpointCount"] != json!(0)
        || record.previous_summary["repositoryId"] != json!(record.repository_id)
        || record.previous_summary["workItemId"] != id
        || record.new_summary["state"] != "implementation_active"
        || record.new_summary["checkpointCount"] != json!(0)
        || record.new_summary["repositoryId"] != json!(record.repository_id)
        || record.new_summary["workItemId"] != id
        || record.new_summary["lifecycleFacts"]["start"] != record.start_fact
        || record.new_contract["repositorySnapshotDigest"]
            != json!(record.repository_snapshot_digest)
        || record.new_contract["baseRevision"] != record.observed_base_revision
        || record.differences
            != activation_differences(&record.previous_contract, &record.new_contract, false)?
        || record.input_digest != activation_input_digest(&record.new_contract)?
        || record.start_fact != expected_start
        || chrono::DateTime::parse_from_rfc3339(&record.recorded_at).is_err()
    {
        return Err(state_error(
            directory,
            "activation transition binding or digest is invalid",
        ));
    }
    Ok(Some(prepared))
}

pub(crate) fn validate_transition_chain(
    root: &Path,
    id: &str,
    history: &[ContractAmendmentReceipt],
    candidate_previous: Option<&Digest>,
) -> Result<(), ObserverError> {
    let transition = read_activation_transition(root, id)?;
    let recovery = read_activation_recovery(root, id, history)?;
    let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
    if let Ok(summary) = super::read_json(&summary_path)
        && ((!summary["activationContinuity"].is_null() && transition.is_none())
            || (!summary["activationRecovery"].is_null() && recovery.is_none()))
    {
        return Err(state_error(
            summary_path,
            "orphan continuity Summary marker has no committed native evidence",
        ));
    }
    if transition.is_some() && recovery.is_some() {
        return Err(state_error(
            root,
            "forward proof and legacy recovery cannot explain the same activation",
        ));
    }
    let mut previous: Option<&ContractAmendmentReceipt> = None;
    for (index, next_previous) in history
        .iter()
        .map(|receipt| &receipt.previous_contract_digest)
        .chain(candidate_previous)
        .enumerate()
    {
        if let Some(tail) = previous
            && &tail.new_contract_digest != next_previous
        {
            let valid = transition.as_ref().is_some_and(|prepared| {
                let proof = &prepared.record;
                !tail.checkpointed
                    && proof.journal_sequence == tail.sequence
                    && proof.journal_digest == tail.journal_digest
                    && proof.previous_contract_digest == tail.new_contract_digest
                    && &proof.new_contract_digest == next_previous
            }) || recovery.as_ref().is_some_and(|record| {
                let request = &record.request;
                !tail.checkpointed
                    && request.boundary_journal_digest == tail.journal_digest
                    && request.before_activation_contract_digest == tail.new_contract_digest
                    && &request.after_activation_contract_digest == next_previous
            });
            if !valid {
                return Err(state_error(
                    amendment_directory(root, id),
                    "amendment Contract transition chain is discontinuous",
                ));
            }
        }
        previous = history.get(index);
    }
    Ok(())
}

/// The caller holds the Work Item lifecycle lock. A zero-history scaffold has
/// no amendment boundary to bridge; its existing initial-declaration behavior
/// remains unchanged. Journal-relevant activation is strictly three-field.
pub(crate) fn prepare_activation_transition(
    root: &Path,
    id: &str,
    before: &Value,
    after: &Value,
    previous_summary: &Value,
    new_summary: &Value,
    runtime: Option<&RuntimeContext>,
) -> Result<Option<PreparedActivationTransition>, ObserverError> {
    let (history, pending) = read_history_with_pending(root, id)?;
    if pending.is_some() {
        return Err(state_error(
            root,
            "activation cannot finish a pending amendment",
        ));
    }
    validate_transition_chain(root, id, &history, None)?;
    let Some(tail) = history.last() else {
        return Ok(None);
    };
    let runtime =
        runtime.ok_or_else(|| state_error(root, "activation proof requires Runtime identity"))?;
    if runtime.protocol_version != cockpit_protocol::PROTOCOL_VERSION
        || runtime.runtime_version.trim().is_empty()
        || runtime.runtime_version == "0.2.105"
        || previous_summary["checkpointCount"] != json!(0)
        || previous_summary["state"] != "not_ready"
        || previous_summary["repositoryId"] != super::repository_id(root).to_string()
        || previous_summary["workItemId"] != id
        || new_summary["checkpointCount"] != json!(0)
        || new_summary["state"] != "implementation_active"
        || tail.checkpointed
        || record_digest(before, root)? != tail.new_contract_digest
    {
        return Err(state_error(
            root,
            "activation requires an exact uncheckpointed journal baseline",
        ));
    }
    let directory = activation_directory(root, id);
    if continuity_record_directory(&directory)?
        && (directory.join("prepared.json").exists() || directory.join("committed.json").exists())
    {
        return Err(state_error(
            directory,
            "activation transition already exists or is pending",
        ));
    }
    let typed: Contract = serde_json::from_value(after.clone())
        .map_err(|error| state_error(root, error.to_string()))?;
    typed
        .validate()
        .map_err(|errors| state_error(root, errors.join("; ")))?;
    if after["repositorySnapshotDigest"] != json!(observed_inputs(root)?.0) {
        return Err(state_error(
            root,
            "activation snapshot changed before preparation",
        ));
    }
    let record = ActivationTransitionRecord {
        schema_version: 1,
        capability: ACTIVATION_CONTINUITY_CAPABILITY.into(),
        repository_id: super::repository_id(root).to_string(),
        work_item_id: id.into(),
        owner: observed_activation_owner(root)?,
        runtime: runtime.clone(),
        observed_head: cockpit_git::GitRepository::discover(root)
            .and_then(|git| git.topology())
            .map_err(|error| state_error(root, error.to_string()))?
            .head
            .ok_or_else(|| state_error(root, "activation head is missing"))?,
        journal_sequence: tail.sequence,
        journal_digest: tail.journal_digest.clone(),
        previous_contract: before.clone(),
        new_contract: after.clone(),
        previous_contract_digest: record_digest(before, root)?,
        new_contract_digest: record_digest(after, root)?,
        previous_summary_digest: record_digest(previous_summary, root)?,
        new_summary_digest: record_digest(new_summary, root)?,
        previous_summary: previous_summary.clone(),
        new_summary: new_summary.clone(),
        repository_snapshot_digest: after["repositorySnapshotDigest"]
            .as_str()
            .ok_or_else(|| state_error(root, "activation snapshot is missing"))?
            .parse::<Digest>()
            .map_err(|error| state_error(root, error.to_string()))?,
        observed_base_revision: after["baseRevision"]
            .as_str()
            .ok_or_else(|| state_error(root, "activation base is missing"))?
            .into(),
        input_digest: activation_input_digest(after)?,
        differences: activation_differences(before, after, false)?,
        start_fact: new_summary["lifecycleFacts"]["start"].clone(),
        recorded_at: new_summary["lifecycleFacts"]["start"]["recordedAt"]
            .as_str()
            .ok_or_else(|| state_error(root, "activation start fact is missing"))?
            .into(),
    };
    let prepared = PreparedActivationTransition {
        record_digest: record_digest(&record, root)?,
        record,
    };
    write_immutable_record(&directory.join("prepared.json"), &prepared)?;
    Ok(Some(prepared))
}

pub(crate) fn commit_activation_transition(
    root: &Path,
    id: &str,
    prepared: &PreparedActivationTransition,
) -> Result<(), ObserverError> {
    let record = &prepared.record;
    let contract =
        super::read_json(&root.join(format!(".ai/work-items/active/{id}.contract.json")))?;
    let summary = super::read_json(&root.join(format!(".ai/work-items/active/{id}.summary.json")))?;
    let history = read_work_item_contract_amendments(root, id)?;
    if record.owner != observed_activation_owner(root)?
        || cockpit_git::GitRepository::discover(root)
            .and_then(|git| git.topology())
            .map_err(|error| state_error(root, error.to_string()))?
            .head
            .as_deref()
            != Some(record.observed_head.as_str())
        || observed_inputs(root)?.0 != record.repository_snapshot_digest
        || record_digest(&contract, root)? != record.new_contract_digest
        || record_digest(&summary, root)? != record.new_summary_digest
        || history.last().is_none_or(|tail| {
            tail.sequence != record.journal_sequence || tail.journal_digest != record.journal_digest
        })
    {
        return Err(state_error(
            root,
            "activation changed before its commit marker",
        ));
    }
    write_immutable_record(
        &activation_directory(root, id).join("committed.json"),
        &ContinuityCommit {
            schema_version: 1,
            prepared_digest: prepared.record_digest.clone(),
            recorded_at: super::usage::lifecycle_now(),
        },
    )
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
}

#[cfg(test)]
fn set_amendment_failpoint(point: AmendmentFailpoint) {
    AMENDMENT_FAILPOINT.with(|value| value.set(Some(point)));
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
        development_review: None,
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
    let (history, pending, _) =
        load_history_with_predecessor(root, work_item_id, allow_pending, None)?;
    Ok((history, pending))
}

type HistoryWithPredecessor = (
    Vec<ContractAmendmentReceipt>,
    Option<PreparedRecord>,
    Option<PredecessorCandidate>,
);

struct PredecessorCandidate {
    contract: Value,
    checkpointed_verification: bool,
}

fn load_history_with_predecessor(
    root: &Path,
    work_item_id: &str,
    allow_pending: bool,
    predecessor: Option<&Digest>,
) -> Result<HistoryWithPredecessor, ObserverError> {
    let directory = amendment_directory(root, work_item_id);
    if !directory.exists() {
        return Ok((Vec::new(), None, None));
    }
    // The historical-recovery reader pins every parent and reads each record
    // without following symlinks. Its parsed values and digest checks consume
    // the same bytes, even if the path is replaced during observation.
    let pinned_directory = if predecessor.is_some() {
        let root_dir = cap_std::fs::Dir::open_ambient_dir(root, cap_std::ambient_authority())
            .map_err(|source| ObserverError::Read {
                path: root.into(),
                source,
            })?;
        let ai = super::open_cap_directory_nofollow_strict(&root_dir, ".ai", &root.join(".ai"))?;
        let evidence =
            super::open_cap_directory_nofollow_strict(&ai, "evidence", &root.join(".ai/evidence"))?;
        Some(super::open_cap_directory_nofollow_strict(
            &evidence,
            &format!("{work_item_id}.contract-amendments"),
            &directory,
        )?)
    } else {
        None
    };
    let read_record = |path: &Path| -> Result<Value, ObserverError> {
        let Some(pinned) = pinned_directory.as_ref() else {
            return super::read_json(path);
        };
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| state_error(path, "invalid amendment record filename"))?;
        let bytes = super::read_cap_file_nofollow_bounded(
            pinned,
            name,
            path,
            cockpit_git::MAX_BOUNDED_GIT_OUTPUT_BYTES as u64,
        )?;
        super::reject_duplicate_json_keys(&bytes).map_err(|message| state_error(path, message))?;
        serde_json::from_slice(&bytes).map_err(|error| state_error(path, error.to_string()))
    };
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
            "prepared" if pair.0.is_none() => {
                pair.0 = Some(
                    serde_json::from_value(read_record(&path)?)
                        .map_err(|error| state_error(&path, error.to_string()))?,
                );
            }
            "committed" if pair.1.is_none() => {
                pair.1 = Some(
                    serde_json::from_value(read_record(&path)?)
                        .map_err(|error| state_error(&path, error.to_string()))?,
                );
            }
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
    let mut predecessor_contract = None;
    let mut previous_delegation: Option<DelegatedDevelopmentReview> = None;
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
        if let Some(review) = &prepared.prepared.delegated_review {
            validate_delegated_review(root, &prepared.prepared, review)?;
            let owner_digest = record_digest(&review.owner_contract, &directory)?;
            if owner_digest != prepared.prepared.previous_contract_digest
                && previous_delegation.as_ref().is_none_or(|prior| {
                    prior.owner_review != review.owner_review
                        || prior.owner_contract != review.owner_contract
                        || prior.owner_receipt_path != review.owner_receipt_path
                })
            {
                return Err(state_error(
                    &directory,
                    "delegated owner review is not anchored to the native predecessor chain",
                ));
            }
        } else if prepared.prepared.request.development_review.is_some() {
            return Err(state_error(
                &directory,
                "technical request lacks its native delegated review",
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
                != (!prepared.prepared.policy_review_requirements.is_empty()
                    && prepared.prepared.delegated_review.is_none())
            || receipt.delegated_review != prepared.prepared.delegated_review
            || receipt.policy_review_requirements != prepared.prepared.policy_review_requirements
            || receipt.previous_journal_digest != previous_digest
            || journal_digest(&receipt)? != receipt.journal_digest
        {
            return Err(state_error(
                &directory,
                "Contract amendment committed record is detached from its prepared amendment or has an invalid digest chain",
            ));
        }
        previous_delegation = receipt.delegated_review.clone();
        previous_digest = receipt.journal_digest.clone();
        if predecessor == Some(&prepared.prepared.previous_contract_digest) {
            if predecessor_contract.is_some() {
                return Err(state_error(
                    &directory,
                    "historical Contract has an ambiguous predecessor position",
                ));
            }
            predecessor_contract = Some(PredecessorCandidate {
                contract: prepared.prepared.previous_contract,
                checkpointed_verification: receipt.checkpointed && receipt.verification_started,
            });
        }
        receipts.push(receipt);
    }
    Ok((receipts, pending, predecessor_contract))
}

pub(crate) fn validated_amendment_predecessor(
    root: &Path,
    work_item_id: &str,
    predecessor: &Digest,
    current: &Digest,
) -> Result<Option<ValidatedAmendmentPredecessor>, ObserverError> {
    if predecessor == current {
        return Ok(None);
    }
    let (history, pending, contract) =
        load_history_with_predecessor(root, work_item_id, false, Some(predecessor))?;
    if pending.is_some()
        || history
            .last()
            .is_none_or(|tail| tail.new_contract_digest != *current)
    {
        return Ok(None);
    }
    validate_transition_chain(root, work_item_id, &history, Some(current))?;
    let Some(candidate) = contract else {
        return Ok(None);
    };
    if !candidate.checkpointed_verification {
        return Ok(None);
    }
    let contract: Contract = serde_json::from_value(candidate.contract)
        .map_err(|error| state_error(root, error.to_string()))?;
    if contract.repository_id != super::repository_id(root).to_string()
        || contract.work_item_id != work_item_id
    {
        return Ok(None);
    }
    Ok(Some(ValidatedAmendmentPredecessor {
        contract,
        digest: predecessor.clone(),
    }))
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
    if delegated_review_is_current(root, work_item_id, None)? {
        return Ok(false);
    }
    let contract = super::read_json(&root.join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    )))?;
    Ok(development_policy(&contract)?.is_some()
        || read_work_item_contract_amendments(root, work_item_id)?
            .iter()
            .any(|receipt| receipt.policy_review_required))
}

/// Validate the native amendment and its current invalidation projection
/// before admitting only the operation that generates a review request.
pub(crate) fn current_amendment_review_request_available(
    root: &Path,
    work_item_id: &str,
    current_contract_digest: &Digest,
    summary: &Value,
    lifecycle_phase: &str,
) -> Result<bool, ObserverError> {
    if !matches!(
        lifecycle_phase,
        "implementation_active" | "checkpointed" | "finish_ready"
    ) {
        return Ok(false);
    }
    let identity = super::repository_id(root).to_string();
    let contract_path = root.join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    if !super::is_regular_non_symlink(&contract_path)? {
        return Ok(false);
    }
    let contract = super::read_json(&contract_path)?;
    if contract["repositoryId"].as_str() != Some(identity.as_str())
        || contract["workItemId"].as_str() != Some(work_item_id)
        || summary["repositoryId"].as_str() != Some(identity.as_str())
        || summary["workItemId"].as_str() != Some(work_item_id)
        || record_digest(&contract, &contract_path)? != *current_contract_digest
    {
        return Err(state_error(
            &contract_path,
            "amendment review request identity or current Contract mismatch",
        ));
    }
    // This reader verifies the complete prepared/committed chain, typed
    // request derivation, identities and digests, and rejects pending work.
    let history = read_work_item_contract_amendments(root, work_item_id)?;
    let Some(tail) = history.last() else {
        return Ok(false);
    };
    validate_transition_chain(root, work_item_id, &history, None)?;
    if tail.new_contract_digest != *current_contract_digest
        || !history.iter().any(|receipt| receipt.policy_review_required)
        || !tail.checkpointed
        || summary["checkpointCount"] != json!(1)
        || !matches!(
            summary["preflightState"].as_str(),
            Some("not_run" | "green" | "yellow" | "red")
        )
        || super::governance_controls::preflight_decision_evidence_state(
            root,
            work_item_id,
            current_contract_digest,
        ) == super::governance_controls::PreflightDecisionEvidenceState::Valid
    {
        return Ok(false);
    }
    // The first native transition after checkpoint is the immutable source
    // of before_edit. Comparing two Summary fields alone accepts both null
    // values or the same forged digest, and the latest transition may refer
    // to a later Contract after multiple amendments.
    let Some(original_checkpoint_contract) = history
        .iter()
        .find(|receipt| receipt.checkpointed)
        .map(|receipt| &receipt.previous_contract_digest)
    else {
        return Ok(false);
    };
    let digest = current_contract_digest.to_string();
    let invalidation = &summary["verificationInvalidatedByContractAmendment"];
    let checks = json!(tail.invalidated_required_checks);
    if invalidation["contractHash"].as_str() != Some(digest.as_str())
        || invalidation["invalidatedRequiredChecks"] != checks
    {
        return Ok(false);
    }
    let Some(checkpoints) = summary["checkpointEvidence"].as_array() else {
        return Ok(false);
    };
    let Some(before_edit) = checkpoints
        .iter()
        .find(|record| record["stage"] == "before_edit")
    else {
        return Ok(false);
    };
    let Some(revalidation) = checkpoints
        .iter()
        .rfind(|record| record["stage"] == "contract_amendment_revalidation")
    else {
        return Ok(false);
    };
    for record in [before_edit, revalidation] {
        if record["schemaVersion"] != json!(1)
            || record["recorded"] != json!(true)
            || record["repositoryId"].as_str() != Some(identity.as_str())
            || record["workItemId"].as_str() != Some(work_item_id)
        {
            return Ok(false);
        }
    }
    Ok(
        revalidation["contractHash"].as_str() == Some(digest.as_str())
            && revalidation["previousContractHash"] == json!(tail.previous_contract_digest)
            && before_edit["contractHash"] == json!(original_checkpoint_contract)
            && revalidation["originalBeforeEditContractHash"]
                == json!(original_checkpoint_contract)
            && revalidation["repositorySnapshotDigest"] == json!(tail.repository_snapshot_digest)
            && revalidation["invalidatedRequiredChecks"] == checks
            && revalidation["verificationStarted"] == json!(tail.verification_started),
    )
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
    let delegated_review = prepare_delegated_review(
        root,
        work_item_id,
        request,
        runtime,
        &previous_contract,
        &new_contract,
        previous_summary,
        &repository_snapshot_digest,
    )?;
    let prepared = PreparedAmendment {
        delegated_review,
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
        delegated_review: data.delegated_review.clone(),
        policy_review_required: !data.policy_review_requirements.is_empty()
            && data.delegated_review.is_none(),
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
    if let Some(review) = &data.delegated_review {
        validate_delegated_review(root, data, review)?;
    }
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
    apply_work_item_contract_amendment_with_legacy_input(root, work_item_id, request, runtime, None)
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
    let request_digest = record_digest(request, Path::new("contract-amendment-request"))?;
    if let Some(pending) = pending {
        let same_request = pending.prepared.request_digest == request_digest
            || (legacy_input_digest.is_some()
                && pending.prepared.legacy_input_digest == legacy_input_digest);
        if !same_request {
            return Err(state_error(
                amendment_directory(&root, work_item_id),
                "incoming request cannot finish an unrelated pending amendment",
            ));
        }
        validate_transition_chain(
            &root,
            work_item_id,
            &history,
            Some(&pending.prepared.previous_contract_digest),
        )?;
        finish_prepared(&root, &pending)?;
        (history, _) = load_history(&root, work_item_id, false)?;
    }
    validate_transition_chain(&root, work_item_id, &history, None)?;
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
    // Validate the whole prospective Contract chain before publishing even
    // its prepared phase. A later admission query must not be the first place
    // that discovers an unexplained old-to-current Contract boundary.
    validate_transition_chain(
        &root,
        work_item_id,
        &history,
        Some(&current_contract_digest),
    )?;

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
    write_prepared(&root, &prepared)?;
    maybe_fail(AmendmentFailpoint::Prepared)?;
    let receipt = finish_prepared(&root, &prepared)?;
    history.push(receipt.clone());
    Ok(receipt)
}

fn development_policy(
    contract: &Value,
) -> Result<Option<cockpit_protocol::DevelopmentReviewPolicy>, ObserverError> {
    let c: Contract = serde_json::from_value(contract.clone())
        .map_err(|e| state_error("development-review", e.to_string()))?;
    c.development_review_policy()
        .map_err(|e| state_error("development-review", e))
}

fn delegated_policy_boundary_allows(
    root: &Path,
    contract: &Value,
    document: Option<&cockpit_protocol::GovernancePolicyDocument>,
) -> Result<bool, ObserverError> {
    let c: Contract =
        serde_json::from_value(contract.clone()).map_err(|e| state_error(root, e.to_string()))?;
    let mut policies = vec![c.governance_policy.as_ref()];
    if let Some(document) = document {
        if document.schema_version != 1
            || document.organization.is_none() && document.project.is_none()
            || document
                .organization
                .as_ref()
                .is_some_and(|p| p.layer != cockpit_protocol::PolicyLayer::Organization)
            || document
                .project
                .as_ref()
                .is_some_and(|p| p.layer != cockpit_protocol::PolicyLayer::Project)
        {
            return Err(state_error(root, "invalid captured repository policy"));
        }
        policies.extend([document.organization.as_ref(), document.project.as_ref()]);
    }
    let operation = super::verification_operation_for_contract(&c);
    Ok(!policies
        .into_iter()
        .flatten()
        .flat_map(|p| &p.rules)
        .any(|rule| {
            (rule.operation == operation
                || matches!(
                    rule.operation.as_str(),
                    "*" | "modify_source" | "review_development_amendment"
                ))
                && rule.approval_mode != cockpit_protocol::ApprovalMode::NoHumanApprovalForLowRisk
        })
        && !c
            .human_decision_points
            .as_ref()
            .is_some_and(|p| !p.is_null() && p != &json!([]) && p != &json!({})))
}

fn native_owner_receipt_json(root: &Path, relative: &str) -> Result<Value, ObserverError> {
    let pinned = super::legacy_closeout_source::PinnedRoot::open(root)?;
    let root_dir = pinned.directory()?;
    let ai = super::open_cap_directory_nofollow_strict(&root_dir, ".ai", &root.join(".ai"))?;
    let directory_path = root.join(".ai/decisions");
    let decisions = super::open_cap_directory_nofollow_strict(&ai, "decisions", &directory_path)?;
    let path = root.join(relative);
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| state_error(&path, "invalid native review filename"))?;
    let bytes = super::legacy_closeout_source::read_regular_cap_bounded(
        &decisions,
        name,
        &path,
        1024 * 1024,
    )?;
    super::reject_duplicate_json_keys(&bytes).map_err(|e| state_error(&path, e))?;
    pinned.require_current()?;
    let current_root = pinned.directory()?;
    let current_ai =
        super::open_cap_directory_nofollow_strict(&current_root, ".ai", &root.join(".ai"))?;
    let current_decisions =
        super::open_cap_directory_nofollow_strict(&current_ai, "decisions", &directory_path)?;
    if super::legacy_closeout_source::directory_identity(&root.join(".ai"), &ai)?
        != super::legacy_closeout_source::directory_identity(&root.join(".ai"), &current_ai)?
        || super::legacy_closeout_source::directory_identity(&directory_path, &decisions)?
            != super::legacy_closeout_source::directory_identity(
                &directory_path,
                &current_decisions,
            )?
    {
        return Err(state_error(
            &directory_path,
            "native owner review parent identity changed",
        ));
    }
    serde_json::from_slice(&bytes).map_err(|e| state_error(&path, e.to_string()))
}

fn owner_receipt_value(
    root: &Path,
    wi: &str,
    review: &DelegatedDevelopmentReview,
) -> Result<Value, ObserverError> {
    let value =
        serde_json::to_value(&review.owner_review).map_err(|e| state_error(root, e.to_string()))?;
    let digest = record_digest(&value, root)?;
    let suffix = digest.to_string();
    let canonical = format!(".ai/decisions/{wi}.preflight-review.json");
    let indexed = format!(
        ".ai/decisions/{wi}.preflight-review.{}.json",
        suffix.strip_prefix("sha256:").unwrap_or(&suffix)
    );
    if review.owner_receipt_path != canonical && review.owner_receipt_path != indexed {
        return Err(state_error(
            root,
            "delegated owner receipt is not at its native indexed path",
        ));
    }
    let path = root.join(&review.owner_receipt_path);
    if native_owner_receipt_json(root, &review.owner_receipt_path)? != value
        || digest != review.owner_receipt_digest
    {
        return Err(state_error(
            path,
            "delegated owner receipt is missing, replaced, foreign or invalid",
        ));
    }
    Ok(value)
}

fn validate_delegated_review(
    root: &Path,
    data: &PreparedAmendment,
    review: &DelegatedDevelopmentReview,
) -> Result<(), ObserverError> {
    validate_delegated_review_values(
        root,
        &data.repository_id,
        &data.work_item_id,
        &data.previous_contract,
        &data.new_contract,
        &data.request,
        review,
    )
}

fn validate_delegated_review_values(
    root: &Path,
    repository: &str,
    wi: &str,
    before: &Value,
    after: &Value,
    request: &ContractAmendmentRequest,
    review: &DelegatedDevelopmentReview,
) -> Result<(), ObserverError> {
    let fail = |message: &str| state_error(amendment_directory(root, wi), message);
    let policy = development_policy(&review.owner_contract)?
        .ok_or_else(|| fail("owner baseline has no explicit developmentReview"))?;
    let before_policy = development_policy(before)?;
    let after_policy = development_policy(after)?;
    if before_policy.as_ref() != Some(&policy)
        || after_policy.as_ref() != Some(&policy)
        || record_digest(&policy, root)? != review.policy_digest
    {
        return Err(fail(
            "delegated policy is not the exact owner-reviewed baseline policy",
        ));
    }
    let owner = &review.owner_review;
    if owner.schema_version != 1
        || owner.decision_id != "contract-preflight-review"
        || owner.decision != "confirm_review"
        || owner.repository_id != repository
        || owner.work_item_id != wi
        || review.owner_contract["repositoryId"] != repository
        || review.owner_contract["workItemId"] != wi
        || owner.contract_digest != record_digest(&review.owner_contract, root)?
        || chrono::DateTime::parse_from_rfc3339(&owner.recorded_at).is_err()
        || owner.recorded_by.trim().is_empty()
        || owner.reason.trim().is_empty()
    {
        return Err(fail(
            "delegated owner review identity or baseline binding is invalid",
        ));
    }
    owner_receipt_value(root, wi, review)?;
    let authority = request
        .development_review
        .as_ref()
        .ok_or_else(|| fail("agent review input is missing"))?;
    cockpit_protocol::validate_authority_evidence(authority).map_err(|e| fail(&e.to_string()))?;
    if authority != &review.authority
        || authority.actor != policy.reviewer_actor
        || authority.authority_source != policy.authority_source
        || authority.assurance != cockpit_protocol::AssuranceLevel::SelfDeclared
        || authority.operations != ["review_development_amendment"]
    {
        return Err(fail(
            "delegated reviewer, source, operation or assurance is outside the exact policy",
        ));
    }
    if !delegated_policy_boundary_allows(root, before, review.repository_policy.as_ref())? {
        return Err(fail(
            "captured human/provider/multi-party policy cannot be discharged by delegated review",
        ));
    }
    cockpit_protocol::validate_development_review_changes(before, request, &policy)
        .map_err(|e| fail(&e))?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn prepare_delegated_review(
    root: &Path,
    wi: &str,
    request: &ContractAmendmentRequest,
    runtime: &RuntimeContext,
    before: &Value,
    after: &Value,
    summary: &Value,
    snapshot: &Digest,
) -> Result<Option<DelegatedDevelopmentReview>, ObserverError> {
    let Some(authority) = &request.development_review else {
        return Ok(None);
    };
    let fail = |message: &str| state_error(amendment_directory(root, wi), message);
    let policy = development_policy(before)?
        .ok_or_else(|| fail("strict mode has no delegated development policy"))?;
    let repository_policy = super::policy_document(root)?;
    if !delegated_policy_boundary_allows(root, before, repository_policy.as_ref())? {
        return Err(fail(
            "current human/provider/multi-party policy cannot be discharged by delegated review",
        ));
    }
    let before_digest = record_digest(before, root)?;
    if summary["preflightContractDigest"] != before_digest.to_string()
        || summary["preflightRepositorySnapshotDigest"] != snapshot.to_string()
        || !matches!(summary["preflightState"].as_str(), Some("green" | "yellow"))
    {
        return Err(fail(
            "delegated review requires current non-red preflight Contract and snapshot bindings",
        ));
    }
    let mut review = if super::governance_controls::preflight_decision_evidence_state(
        root,
        wi,
        &before_digest,
    ) == super::governance_controls::PreflightDecisionEvidenceState::Valid
    {
        let owner: cockpit_protocol::PreflightDecisionEvidence =
            serde_json::from_value(summary["decisionEvidence"].clone())
                .map_err(|e| fail(&e.to_string()))?;
        let value = serde_json::to_value(&owner).map_err(|e| fail(&e.to_string()))?;
        let digest = record_digest(&value, root)?;
        let canonical = format!(".ai/decisions/{wi}.preflight-review.json");
        let path = if super::is_regular_non_symlink(&root.join(&canonical))?
            && native_owner_receipt_json(root, &canonical)? == value
        {
            canonical
        } else {
            format!(
                ".ai/decisions/{wi}.preflight-review.{}.json",
                digest.to_string().trim_start_matches("sha256:")
            )
        };
        DelegatedDevelopmentReview {
            owner_contract: before.clone(),
            owner_review: owner,
            owner_receipt_path: path,
            owner_receipt_digest: digest,
            policy_digest: record_digest(&policy, root)?,
            repository_policy: repository_policy.clone(),
            authority: authority.clone(),
        }
    } else {
        let history = read_work_item_contract_amendments(root, wi)?;
        history.last().filter(|r| r.new_contract_digest == before_digest && r.runtime_digest == runtime.runtime_digest && r.runtime_version == runtime.runtime_version)
            .and_then(|r| r.delegated_review.clone()).ok_or_else(|| fail("delegated review requires a formal owner baseline or exact native delegated predecessor"))?
    };
    review.authority = authority.clone();
    review.repository_policy = repository_policy;
    validate_delegated_review_values(
        root,
        &super::repository_id(root).to_string(),
        wi,
        before,
        after,
        request,
        &review,
    )?;
    Ok(Some(review))
}

pub(crate) fn delegated_review_is_current(
    root: &Path,
    wi: &str,
    runtime: Option<&RuntimeContext>,
) -> Result<bool, ObserverError> {
    let history = read_work_item_contract_amendments(root, wi)?;
    let Some(tail) = history.last().filter(|r| r.delegated_review.is_some()) else {
        return Ok(false);
    };
    let contract = root.join(format!(".ai/work-items/active/{wi}.contract.json"));
    let current_contract = super::read_json(&contract)?;
    let current_policy = super::policy_document(root)?;
    if !delegated_policy_boundary_allows(root, &current_contract, current_policy.as_ref())? {
        return Ok(false);
    }
    Ok(
        super::contract_digest(&contract)? == tail.new_contract_digest
            && runtime.is_none_or(|r| {
                r.runtime_digest == tail.runtime_digest && r.runtime_version == tail.runtime_version
            }),
    )
}

#[cfg(test)]
mod tests {
    use super::{AmendmentFailpoint, set_amendment_failpoint};
    use crate::{
        ContractAmendmentReceipt, WorkItemStartOptions, amend_work_item_contract, attach,
        checkpoint_work_item, preflight_work_item, read_work_item_contract_amendments,
        start_work_item_with_options,
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
    fn request_binding_rejects_a_rehashed_discontinuous_contract_chain() {
        let directory = repository();
        let root = directory.path();
        let id = "WI-REVIEW-CONTRACT-CHAIN";
        let contract = start_checkpointed(root, id);
        for path in ["first/**", "second/**"] {
            amend_work_item_contract(root, id, &json!({"scopeAppend":[path]}), path)
                .expect("native amendment");
        }
        let native_summary =
            read_json(root.join(format!(".ai/work-items/active/{id}.summary.json")));
        let native_digest = crate::contract_digest(&contract).expect("current native digest");
        let native_history = read_work_item_contract_amendments(root, id).expect("native history");
        assert_ne!(
            native_history[0].previous_contract_digest,
            native_history[1].previous_contract_digest
        );
        assert!(
            super::current_amendment_review_request_available(
                root,
                id,
                &native_digest,
                &native_summary,
                "checkpointed"
            )
            .expect("two consecutive native amendments remain eligible")
        );
        let prepared_path = super::amendment_record_path(root, id, 1, "prepared");
        let committed_path = super::amendment_record_path(root, id, 1, "committed");
        let mut prepared: super::PreparedRecord =
            super::read_typed_record(&prepared_path).expect("prepared");
        let mut committed: super::CommittedRecord =
            super::read_typed_record(&committed_path).expect("committed");
        // Preserve all per-record hashes and journal links while forking the
        // first Contract transition away from the second one's predecessor.
        prepared.prepared.request.changes[0].value = Some(json!("fork/**"));
        let derived = super::apply_typed_amendment(
            &prepared.prepared.previous_contract,
            &prepared.prepared.request,
        )
        .expect("typed fork");
        prepared.prepared.new_contract =
            super::apply_runtime_capability_requirements(derived).expect("capabilities");
        prepared.prepared.request_digest =
            super::record_digest(&prepared.prepared.request, &prepared_path)
                .expect("request digest");
        prepared.prepared.new_contract_digest =
            super::record_digest(&prepared.prepared.new_contract, &prepared_path)
                .expect("Contract digest");
        prepared.prepared_digest =
            super::record_digest(&prepared.prepared, &prepared_path).expect("prepared digest");
        committed.prepared_digest = prepared.prepared_digest.clone();
        committed.receipt.request_digest = prepared.prepared.request_digest.clone();
        committed.receipt.new_contract_digest = prepared.prepared.new_contract_digest.clone();
        committed.receipt.changed_values = super::changed_values(
            &prepared.prepared.previous_contract,
            &prepared.prepared.new_contract,
            &prepared.prepared.request,
        )
        .expect("changed values");
        committed.receipt.journal_digest =
            super::journal_digest(&committed.receipt).expect("first journal digest");
        fs::write(
            &prepared_path,
            serde_json::to_vec(&prepared).expect("prepared JSON"),
        )
        .expect("isolated fork");
        fs::write(
            &committed_path,
            serde_json::to_vec(&committed).expect("committed JSON"),
        )
        .expect("isolated fork receipt");
        let second_prepared_path = super::amendment_record_path(root, id, 2, "prepared");
        let second_committed_path = super::amendment_record_path(root, id, 2, "committed");
        let mut second: super::PreparedRecord =
            super::read_typed_record(&second_prepared_path).expect("second prepared");
        let mut second_commit: super::CommittedRecord =
            super::read_typed_record(&second_committed_path).expect("second committed");
        second.prepared.previous_journal_digest = committed.receipt.journal_digest.clone();
        second.prepared_digest = super::record_digest(&second.prepared, &second_prepared_path)
            .expect("second prepared digest");
        second_commit.prepared_digest = second.prepared_digest.clone();
        second_commit.receipt.previous_journal_digest = committed.receipt.journal_digest;
        second_commit.receipt.journal_digest =
            super::journal_digest(&second_commit.receipt).expect("second journal digest");
        fs::write(
            &second_prepared_path,
            serde_json::to_vec(&second).expect("second prepared JSON"),
        )
        .expect("isolated link");
        fs::write(
            &second_committed_path,
            serde_json::to_vec(&second_commit).expect("second receipt JSON"),
        )
        .expect("isolated linked receipt");
        assert_eq!(
            read_work_item_contract_amendments(root, id)
                .expect("all native per-record and journal checks pass")
                .len(),
            2
        );
        let summary = read_json(root.join(format!(".ai/work-items/active/{id}.summary.json")));
        let digest = crate::contract_digest(&contract).expect("current Contract digest");
        let error = super::current_amendment_review_request_available(
            root,
            id,
            &digest,
            &summary,
            "checkpointed",
        )
        .expect_err("Contract transition discontinuity must be refused");
        assert!(error.to_string().contains("discontinuous"), "{error}");
        let history =
            read_work_item_contract_amendments(root, id).expect("retained resealed history");
        let error =
            super::legacy_activation_boundary(root, id, &history, &read_json(&contract), &summary)
                .err()
                .expect("present-baseline recovery cannot excuse a post-checkpoint gap");
        assert!(error.to_string().contains("post-checkpoint"), "{error}");
        assert!(!super::recovery_directory(root, id).exists());
    }

    #[test]
    fn finite_activation_policy_rejects_plan_expansion_and_non_exact_csv() {
        let before = json!({"state":"not_ready", "baseRevision":"old", "repositorySnapshotDigest":"old",
            "outOfScope":["first,second","third"], "scope":["bounded/**"], "authority":"authorized",
            "actor":"human:fixture", "acceptanceCriteria":["bounded"], "sources":["existing"]});
        let mut after = before.clone();
        after["state"] = json!("implementation_active");
        after["baseRevision"] = json!("new");
        after["repositorySnapshotDigest"] = json!("new");
        let forward =
            super::activation_differences(&before, &after, false).expect("three finite fields");
        assert_eq!(forward.len(), 3);
        for field in [
            "scope",
            "authority",
            "actor",
            "acceptanceCriteria",
            "sources",
        ] {
            let mut expanded = after.clone();
            expanded[field] = json!("expanded");
            assert!(
                super::activation_differences(&before, &expanded, true).is_err(),
                "{field}"
            );
        }
        after["outOfScope"] = json!(["first", "second", "third"]);
        assert!(super::activation_differences(&before, &after, false).is_err());
        let exact = super::activation_differences(&before, &after, true)
            .expect("exact separately reviewed legacy split");
        let csv = exact
            .iter()
            .find(|difference| difference.path == "/outOfScope")
            .expect("enumerated CSV");
        assert_eq!(csv.old_value, before["outOfScope"]);
        assert_eq!(csv.new_value, after["outOfScope"]);
        for invalid in [
            json!(["first", "third", "second"]),
            json!(["first", "second", "third", "extra"]),
            json!(["first,second", "third", "extra"]),
        ] {
            after["outOfScope"] = invalid;
            assert!(super::activation_differences(&before, &after, true).is_err());
        }
        let mut post_checkpoint = before.clone();
        post_checkpoint["state"] = json!("implementation_active");
        assert!(super::activation_differences(&post_checkpoint, &after, true).is_err());
    }

    #[test]
    fn unrelated_request_does_not_finish_a_pending_amendment() {
        for (suffix, failpoint) in [
            ("PREPARED", AmendmentFailpoint::Prepared),
            ("CONTRACT", AmendmentFailpoint::ContractReplaced),
            ("SUMMARY", AmendmentFailpoint::SummaryProjected),
        ] {
            let directory = repository();
            let root = directory.path();
            let id = format!("WI-UNRELATED-PENDING-{suffix}");
            let contract_path = start_checkpointed(root, &id);
            let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
            let original = read_json(&contract_path);
            let request = json!({
                "schemaVersion":1, "changeId":"the-interrupted-request",
                "expectedContractDigest":cockpit_protocol::digest_json(&original).expect("digest"),
                "reason":"complete this exact fixture request only",
                "changes":[{"path":"/goal","operation":"set","value":"the interrupted plan"}]
            });
            set_amendment_failpoint(failpoint);
            amend_work_item_contract(root, &id, &request, "")
                .expect_err("fixture interrupts the exact transaction");
            let before_contract = fs::read(&contract_path).expect("current Contract");
            let before_summary = fs::read(&summary_path).expect("current Summary");
            let prepared_path = super::amendment_record_path(root, &id, 1, "prepared");
            let before_prepared = fs::read(&prepared_path).expect("prepared bytes");
            let current = read_json(&contract_path);
            let other = json!({
                "schemaVersion":1, "changeId":"a-different-request",
                "expectedContractDigest":cockpit_protocol::digest_json(&current).expect("digest"),
                "reason":"this request must not complete somebody else's transaction",
                "changes":[{"path":"/goal","operation":"set","value":"a different plan"}]
            });
            let error = amend_work_item_contract(root, &id, &other, "")
                .expect_err("an unrelated request must be rejected before any write");
            assert!(error.to_string().contains("unrelated pending"), "{error}");
            assert_eq!(fs::read(&contract_path).expect("Contract"), before_contract);
            assert_eq!(fs::read(&summary_path).expect("Summary"), before_summary);
            assert_eq!(fs::read(&prepared_path).expect("prepared"), before_prepared);
            assert!(!super::amendment_record_path(root, &id, 1, "committed").exists());
            assert!(read_work_item_contract_amendments(root, &id).is_err());
        }
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

use super::status_projection;
use super::{
    MAX_EXTERNAL_EVIDENCE_BYTES, ObserverError, acquire_lifecycle_lock, atomic_json,
    close_decision_is_valid_for_status, contract_digest, discover_worktree_layout,
    finalization_observation_error, git_text, infer_legacy_shared_worktree_retained,
    is_regular_non_symlink, load_recovery_decision, now, read_contract, read_json,
    recovery_decision_error, recovery_successor_resolves_pending_close, reject_duplicate_json_keys,
    repository_id, repository_readiness, repository_relative_path,
    set_resource_context_on_active_contract, valid_git_object_id, validate_work_item_id,
    verify_archive_manifest,
};
use cockpit_core::Digest;
use cockpit_protocol::{
    Contract, DelegatedEvidenceReceipt, EvidenceAssurance, EvidenceValidity, FinalizationErrorCode,
    HistoricalFinalizationKind, HistoricalFinalizationRecoveryReceipt, PostMergeSyncEvidence,
    PostMergeSyncRemoteBranchState, ResourceFinalizationContext, ResourceFinalizationDisposition,
    ResourceFinalizationPullRequestState, ResourceFinalizationReceipt,
    ResourceFinalizationTransitionReceipt, RuntimeContext,
    validate_historical_finalization_recovery, validate_resource_finalization_context,
    validate_resource_finalization_receipt_for, validate_resource_finalization_replay,
    validate_resource_finalization_transition,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ArchivedResourceContextBinding {
    schema_version: u32,
    operation: String,
    work_item_id: String,
    repository_id: String,
    contract_digest: Digest,
    archive_manifest_digest: Digest,
    resource_context: ResourceFinalizationContext,
    runtime_version: String,
    runtime_digest: Digest,
    recorded_at: String,
}

pub fn plan_resource_finalization(
    root: &Path,
    work_item_id: &str,
    context: &ResourceFinalizationContext,
) -> Result<serde_json::Value, ObserverError> {
    plan_resource_finalization_internal(root, work_item_id, context, None)
}

/// Runtime-aware variant used by the CLI/MCP entrypoints.  The legacy
/// three-argument function remains available for existing library callers that
/// only plan an active Contract; an archived handoff requires the executing
/// Runtime identity so the append-only binding is auditable.
pub fn plan_resource_finalization_with_runtime(
    root: &Path,
    work_item_id: &str,
    context: &ResourceFinalizationContext,
    runtime: &RuntimeContext,
) -> Result<serde_json::Value, ObserverError> {
    plan_resource_finalization_internal(root, work_item_id, context, Some(runtime))
}

fn plan_resource_finalization_internal(
    root: &Path,
    work_item_id: &str,
    context: &ResourceFinalizationContext,
    runtime: Option<&RuntimeContext>,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let _lifecycle_lock = acquire_lifecycle_lock(&root, work_item_id)?;
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    if fs::symlink_metadata(&contract_path).is_err() {
        return plan_archived_resource_finalization(&root, work_item_id, context, runtime);
    }
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let contract = read_contract(&contract_path)?;
    let summary = read_json(&summary_path)?;
    if !matches!(
        summary["state"].as_str(),
        Some("implementation_active" | "checkpointed")
    ) {
        return Err(ObserverError::State {
            path: summary_path,
            message: "finalize-plan must run before verification/finish_ready; changing resource context after the verification cycle would invalidate lifecycle evidence".into(),
        });
    }
    if let Some(existing) = &contract.resource_context
        && existing != context
        && !existing.is_provisional()
    {
        return Err(ObserverError::State {
            path: contract_path,
            message:
                "resource finalization context is already bound; use the same context for replay"
                    .into(),
        });
    }
    let retry_pending = summary["recoveryRetryPending"] == serde_json::json!(true);
    let retry_binding_valid = if retry_pending {
        let recovery = load_recovery_decision(&root, work_item_id, None)?;
        if recovery
            .as_ref()
            .is_none_or(|decision| decision.decision != "retry")
        {
            return Err(recovery_decision_error(
                root.join(".ai/decisions"),
                "retry_binding_missing",
                "pending retry cannot be advanced by finalize-plan without its exact recovery receipt",
            ));
        }
        true
    } else {
        false
    };
    let evidence_path = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.verification.json"));
    let changes_identity = contract.resource_context.as_ref() != Some(context);
    let has_verification_evidence = fs::symlink_metadata(&evidence_path).is_ok();
    if changes_identity && has_verification_evidence && !retry_binding_valid {
        return Err(ObserverError::State {
            path: contract_path,
            message:
                "finalize-plan must run before verification; changing resource context now requires an explicit Contract revalidation"
                    .into(),
        });
    }
    if summary["state"] == serde_json::json!("finish_ready")
        && fs::symlink_metadata(&evidence_path).is_ok()
        && contract.resource_context.as_ref() != Some(context)
    {
        return Err(ObserverError::State {
            path: evidence_path,
            message: "finalize-plan must run before verification evidence is recorded; re-run verify after changing the context".into(),
        });
    }
    set_resource_context_on_active_contract(&root, work_item_id, context)?;
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let digest =
        Digest::sha256_bytes(
            &fs::read(&contract_path).map_err(|source| ObserverError::Read {
                path: contract_path.clone(),
                source,
            })?,
        );
    if retry_pending {
        let mut summary = read_json(&summary_path)?;
        summary["recoveryRetryContractDigest"] =
            serde_json::json!(contract_digest(&contract_path)?.to_string());
        atomic_json(&summary_path, &summary)?;
    }
    Ok(serde_json::json!({
        "protocolVersion": 1,
        "workItemId": work_item_id,
        "state": "planned",
        "resourceContext": context,
        "contractDigest": digest,
        "next": ["archive", "finalize", "finalize-verify", "close"]
    }))
}

fn plan_archived_resource_finalization(
    root: &Path,
    work_item_id: &str,
    context: &ResourceFinalizationContext,
    runtime: Option<&RuntimeContext>,
) -> Result<serde_json::Value, ObserverError> {
    let binding_path = archived_resource_context_binding_path(root, work_item_id);
    let runtime = runtime.ok_or_else(|| ObserverError::State {
        path: binding_path.clone(),
        message:
            "archived finalize-plan requires the Runtime-aware entrypoint so the handoff identity can be recorded"
                .into(),
    })?;
    validate_resource_finalization_context(context).map_err(|error| ObserverError::State {
        path: binding_path.clone(),
        message: error.to_string(),
    })?;
    if context.is_provisional() {
        return Err(ObserverError::State {
            path: binding_path,
            message: "archived finalize-plan requires a non-provisional resource context".into(),
        });
    }

    let manifest_path = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.archive.json"));
    let manifest = read_json(&manifest_path)?;
    verify_archive_manifest(root, work_item_id, &manifest)?;
    if manifest["state"] != serde_json::json!("archived") {
        return Err(ObserverError::State {
            path: manifest_path,
            message: "archived resource binding requires an archived archive manifest".into(),
        });
    }
    let (contract, contract_digest) = archived_contract_digest(root, work_item_id)?;
    if let Some(existing) = contract.resource_context.as_ref() {
        if existing != context {
            return Err(ObserverError::State {
                path: root
                    .join(".ai/work-items/archive")
                    .join(format!("{work_item_id}.contract.json")),
                message: "archived Contract already binds a different resource context".into(),
            });
        }
        return Ok(serde_json::json!({
            "protocolVersion": 1,
            "workItemId": work_item_id,
            "state": "idempotent",
            "resourceContext": context,
            "contractDigest": contract_digest,
            "source": "archivedContract",
            "next": ["finalize", "finalize-verify", "close"]
        }));
    }

    let manifest_digest =
        Digest::sha256_bytes(
            &fs::read(&manifest_path).map_err(|source| ObserverError::Read {
                path: manifest_path.clone(),
                source,
            })?,
        );
    let binding = ArchivedResourceContextBinding {
        schema_version: 1,
        operation: "resource_context_binding".into(),
        work_item_id: work_item_id.into(),
        repository_id: contract.repository_id.clone(),
        contract_digest: contract_digest.clone(),
        archive_manifest_digest: manifest_digest.clone(),
        resource_context: context.clone(),
        runtime_version: runtime.runtime_version.clone(),
        runtime_digest: runtime.runtime_digest.clone(),
        recorded_at: now(),
    };
    let value = serde_json::to_value(&binding).map_err(|error| ObserverError::State {
        path: binding_path.clone(),
        message: error.to_string(),
    })?;
    if fs::symlink_metadata(&binding_path).is_ok() {
        if !is_regular_non_symlink(&binding_path)? {
            return Err(ObserverError::State {
                path: binding_path,
                message: "archived resource context binding must be a regular non-symlink file"
                    .into(),
            });
        }
        let existing = read_archived_resource_context_binding(root, work_item_id, &contract)?
            .ok_or_else(|| ObserverError::State {
                path: binding_path.clone(),
                message: "archived resource context binding disappeared during replay".into(),
            })?;
        if existing.resource_context == *context {
            return Ok(serde_json::json!({
                "workItemId": work_item_id,
                "state": "idempotent",
                "resourceContext": context,
                "contractDigest": contract_digest,
                "archiveManifestDigest": manifest_digest,
                "path": repository_relative_path(root, &binding_path)
            }));
        }
        return Err(ObserverError::State {
            path: binding_path,
            message: "archived resource context binding already exists with different content"
                .into(),
        });
    }
    atomic_json(&binding_path, &value)?;
    Ok(serde_json::json!({
        "protocolVersion": 1,
        "workItemId": work_item_id,
        "state": "recorded",
        "resourceContext": context,
        "contractDigest": contract_digest,
        "archiveManifestDigest": manifest_digest,
        "runtimeVersion": runtime.runtime_version,
        "runtimeDigest": runtime.runtime_digest,
        "path": repository_relative_path(root, &binding_path),
        "next": ["finalize", "finalize-verify", "close"]
    }))
}

pub(crate) fn archived_resource_context_binding_path(root: &Path, work_item_id: &str) -> PathBuf {
    root.join(".ai/decisions")
        .join(format!("{work_item_id}.resource-context.json"))
}

pub(crate) fn effective_resource_context(
    root: &Path,
    work_item_id: &str,
    contract: &Contract,
) -> Result<Option<ResourceFinalizationContext>, ObserverError> {
    let binding_path = archived_resource_context_binding_path(root, work_item_id);
    let binding = read_archived_resource_context_binding(root, work_item_id, contract)?;
    if let Some(context) = contract.resource_context.as_ref() {
        if let Some(binding) = binding
            && binding.resource_context != *context
        {
            return Err(ObserverError::State {
                path: binding_path,
                message: "archived resource context binding disagrees with the Contract".into(),
            });
        }
        return Ok(Some(context.clone()));
    }
    Ok(binding.map(|binding| binding.resource_context))
}

fn read_archived_resource_context_binding(
    root: &Path,
    work_item_id: &str,
    contract: &Contract,
) -> Result<Option<ArchivedResourceContextBinding>, ObserverError> {
    let path = archived_resource_context_binding_path(root, work_item_id);
    if fs::symlink_metadata(&path).is_err() {
        return Ok(None);
    }
    if !is_regular_non_symlink(&path)? {
        return Err(ObserverError::State {
            path,
            message: "archived resource context binding must be a regular non-symlink file".into(),
        });
    }
    let bytes = fs::read(&path).map_err(|source| ObserverError::Read {
        path: path.clone(),
        source,
    })?;
    reject_duplicate_json_keys(&bytes).map_err(|message| ObserverError::State {
        path: path.clone(),
        message: format!("invalid archived resource context binding JSON: {message}"),
    })?;
    let binding: ArchivedResourceContextBinding =
        serde_json::from_slice(&bytes).map_err(|error| ObserverError::State {
            path: path.clone(),
            message: format!("invalid archived resource context binding: {error}"),
        })?;
    if binding.schema_version != 1
        || binding.operation != "resource_context_binding"
        || binding.work_item_id != work_item_id
        || binding.repository_id != contract.repository_id
    {
        return Err(ObserverError::State {
            path,
            message: "archived resource context binding identity is invalid".into(),
        });
    }
    let contract_path = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.contract.json"));
    let expected_contract_digest = Digest::sha256_bytes(&fs::read(&contract_path).map_err(
        |source| ObserverError::Read {
            path: contract_path.clone(),
            source,
        },
    )?);
    if binding.contract_digest != expected_contract_digest {
        return Err(ObserverError::State {
            path: path.clone(),
            message: "archived resource context binding contract digest is stale".into(),
        });
    }
    let manifest_path = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.archive.json"));
    let manifest = read_json(&manifest_path)?;
    verify_archive_manifest(root, work_item_id, &manifest)?;
    let expected_manifest_digest = Digest::sha256_bytes(&fs::read(&manifest_path).map_err(
        |source| ObserverError::Read {
            path: manifest_path.clone(),
            source,
        },
    )?);
    if binding.archive_manifest_digest != expected_manifest_digest {
        return Err(ObserverError::State {
            path,
            message: "archived resource context binding archive manifest digest is stale".into(),
        });
    }
    validate_resource_finalization_context(&binding.resource_context).map_err(|error| {
        ObserverError::State {
            path: archived_resource_context_binding_path(root, work_item_id),
            message: format!("archived resource context binding is invalid: {error}"),
        }
    })?;
    if binding.resource_context.is_provisional() {
        return Err(ObserverError::State {
            path: archived_resource_context_binding_path(root, work_item_id),
            message: "archived resource context binding must not be provisional".into(),
        });
    }
    Ok(Some(binding))
}

pub(crate) fn read_resource_finalization_receipt(
    path: &Path,
) -> Result<ResourceFinalizationReceipt, ObserverError> {
    if !is_regular_non_symlink(path)? {
        return Err(ObserverError::State {
            path: path.into(),
            message: "resource finalization receipt must be a regular non-symlink file".into(),
        });
    }
    let bytes = fs::read(path).map_err(|source| ObserverError::Read {
        path: path.into(),
        source,
    })?;
    reject_duplicate_json_keys(&bytes).map_err(|message| ObserverError::State {
        path: path.into(),
        message: format!("invalid resource finalization receipt JSON: {message}"),
    })?;
    serde_json::from_slice(&bytes).map_err(|error| ObserverError::State {
        path: path.into(),
        message: format!("invalid resource finalization receipt: {error}"),
    })
}

pub(crate) fn resource_finalization_decision_path(root: &Path, work_item_id: &str) -> PathBuf {
    root.join(".ai/decisions")
        .join(format!("{work_item_id}.finalize.json"))
}

pub(crate) fn read_resource_finalization_transition(
    path: &Path,
) -> Result<ResourceFinalizationTransitionReceipt, ObserverError> {
    if !is_regular_non_symlink(path)? {
        return Err(ObserverError::State {
            path: path.into(),
            message: "resource finalization transition must be a regular non-symlink file".into(),
        });
    }
    let bytes = fs::read(path).map_err(|source| ObserverError::Read {
        path: path.into(),
        source,
    })?;
    reject_duplicate_json_keys(&bytes).map_err(|message| ObserverError::State {
        path: path.into(),
        message: format!("invalid resource finalization transition JSON: {message}"),
    })?;
    serde_json::from_slice(&bytes).map_err(|error| ObserverError::State {
        path: path.into(),
        message: format!("invalid resource finalization transition: {error}"),
    })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PostFinalizeQualityRoutePathDecision {
    path: String,
    profile: String,
    reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PostFinalizeQualityRouteReceipt {
    schema_version: u32,
    kind: String,
    automatic_profile: String,
    base_revision: String,
    changed_paths: Vec<String>,
    contract_digest: Digest,
    contract_path: String,
    head_revision: String,
    manifest_digest: Digest,
    path_decisions: Vec<PostFinalizeQualityRoutePathDecision>,
    reasons: Vec<String>,
    receipt_digest: Digest,
    requested_profile: Option<String>,
    requested_risk: String,
    required_gate_ids: Vec<String>,
    risk: String,
    selected_profile: String,
    stage: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PostFinalizeRepositoryGate {
    id: String,
    category: String,
    command: Vec<String>,
    #[serde(default)]
    covers: Vec<String>,
    state: String,
    exit_code: i32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PostFinalizeRepositoryGateRoute {
    manifest_digest: Digest,
    receipt_digest: Digest,
    required_gate_ids: Vec<String>,
    selected_profile: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PostFinalizeRepositoryGatesReceipt {
    schema_version: u32,
    state: String,
    route: PostFinalizeRepositoryGateRoute,
    gates: Vec<PostFinalizeRepositoryGate>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PostFinalizeEvidenceKind {
    QualityRoute,
    RepositoryGates,
}

fn post_finalize_evidence_kind(
    work_item_id: &str,
    candidate: &str,
) -> Option<PostFinalizeEvidenceKind> {
    let prefix = format!(".ai/evidence/{work_item_id}/");
    match candidate.strip_prefix(&prefix)? {
        "quality-route-post-finalize.json" => Some(PostFinalizeEvidenceKind::QualityRoute),
        "repository-gates-post-finalize.json" => Some(PostFinalizeEvidenceKind::RepositoryGates),
        _ => None,
    }
}

fn read_governance_append_blob(
    root: &Path,
    revision: &str,
    candidate: &str,
    path: &Path,
) -> Result<Vec<u8>, ObserverError> {
    let object = format!("{revision}:{candidate}");
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "blob", &object])
        .output()
        .map_err(|source| ObserverError::Read {
            path: path.into(),
            source,
        })?;
    if !output.status.success() {
        return Err(ObserverError::State {
            path: path.into(),
            message: "cannot read governance append evidence blob".into(),
        });
    }
    if output.stdout.len() > MAX_EXTERNAL_EVIDENCE_BYTES {
        return Err(ObserverError::State {
            path: path.into(),
            message: "governance append evidence exceeds the bounded size limit".into(),
        });
    }
    reject_duplicate_json_keys(&output.stdout).map_err(|message| ObserverError::State {
        path: path.into(),
        message: format!("invalid governance append evidence JSON: {message}"),
    })?;
    Ok(output.stdout)
}

fn nonempty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn validate_post_finalize_evidence_bundle(
    root: &Path,
    work_item_id: &str,
    previous: &ResourceFinalizationReceipt,
    append_revision: &str,
    quality_bytes: &[u8],
    gates_bytes: &[u8],
    path: &Path,
) -> Result<(), ObserverError> {
    let quality_value: serde_json::Value =
        serde_json::from_slice(quality_bytes).map_err(|error| ObserverError::State {
            path: path.into(),
            message: format!("invalid post-finalize quality route JSON: {error}"),
        })?;
    let quality: PostFinalizeQualityRouteReceipt = serde_json::from_value(quality_value.clone())
        .map_err(|error| ObserverError::State {
            path: path.into(),
            message: format!("invalid post-finalize quality route schema: {error}"),
        })?;
    let gates: PostFinalizeRepositoryGatesReceipt =
        serde_json::from_slice(gates_bytes).map_err(|error| ObserverError::State {
            path: path.into(),
            message: format!("invalid post-finalize repository gates schema: {error}"),
        })?;

    let expected_contract_path = format!(".ai/work-items/archive/{work_item_id}.contract.json");
    let Some(expected_contract_digest) = previous.contract_digest.as_ref() else {
        return Err(ObserverError::State {
            path: path.into(),
            message: "post-finalize evidence requires a contract-bound predecessor receipt".into(),
        });
    };
    let quality_head_is_bounded = valid_git_object_id(&quality.head_revision)
        && git_text(
            root,
            &[
                "merge-base",
                "--is-ancestor",
                &previous.pull_request.head_revision,
                &quality.head_revision,
            ],
        )
        .is_some()
        && git_text(
            root,
            &[
                "merge-base",
                "--is-ancestor",
                &quality.head_revision,
                append_revision,
            ],
        )
        .is_some();
    let string_lists_are_valid = [
        quality.changed_paths.as_slice(),
        quality.reasons.as_slice(),
        quality.required_gate_ids.as_slice(),
    ]
    .into_iter()
    .all(|values| !values.is_empty() && values.iter().all(|value| nonempty(value)));
    let path_decisions_are_valid = !quality.path_decisions.is_empty()
        && quality.path_decisions.iter().all(|decision| {
            nonempty(&decision.path) && nonempty(&decision.profile) && nonempty(&decision.reason)
        })
        && quality
            .path_decisions
            .iter()
            .map(|decision| decision.path.as_str())
            .eq(quality.changed_paths.iter().map(String::as_str));
    if quality.schema_version != 1
        || quality.kind != "repository_quality_route"
        || quality.stage != "pull_request"
        || quality.contract_path != expected_contract_path
        || &quality.contract_digest != expected_contract_digest
        || quality.base_revision != previous.pull_request.base_revision
        || !quality_head_is_bounded
        || !string_lists_are_valid
        || !path_decisions_are_valid
        || !nonempty(&quality.automatic_profile)
        || !nonempty(&quality.risk)
        || !nonempty(&quality.requested_risk)
        || quality
            .requested_profile
            .as_deref()
            .is_some_and(|value| !nonempty(value))
        || !nonempty(&quality.selected_profile)
    {
        return Err(ObserverError::State {
            path: path.into(),
            message: "post-finalize quality route binding is invalid".into(),
        });
    }

    let mut digest_payload = quality_value;
    let Some(payload) = digest_payload.as_object_mut() else {
        return Err(ObserverError::State {
            path: path.into(),
            message: "post-finalize quality route must be a JSON object".into(),
        });
    };
    payload.remove("receiptDigest");
    let computed_receipt_digest =
        cockpit_protocol::digest_json(&digest_payload).map_err(|error| ObserverError::State {
            path: path.into(),
            message: format!("cannot digest post-finalize quality route: {error}"),
        })?;
    if quality.receipt_digest != computed_receipt_digest {
        return Err(ObserverError::State {
            path: path.into(),
            message: "post-finalize quality route receipt digest mismatch".into(),
        });
    }

    let route_ids = quality
        .required_gate_ids
        .iter()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let gate_ids = gates
        .gates
        .iter()
        .map(|gate| gate.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let gates_are_valid = !gates.gates.is_empty()
        && gate_ids.len() == gates.gates.len()
        && route_ids.len() == quality.required_gate_ids.len()
        && route_ids == gate_ids
        && gates.gates.iter().all(|gate| {
            nonempty(&gate.id)
                && nonempty(&gate.category)
                && !gate.command.is_empty()
                && gate.command.iter().all(|value| nonempty(value))
                && gate.covers.iter().all(|value| nonempty(value))
                && gate.state == "passed"
                && gate.exit_code == 0
        });
    if gates.schema_version != 2
        || gates.state != "passed"
        || gates.route.manifest_digest != quality.manifest_digest
        || gates.route.receipt_digest != quality.receipt_digest
        || gates.route.required_gate_ids != quality.required_gate_ids
        || gates.route.selected_profile != quality.selected_profile
        || !gates_are_valid
    {
        return Err(ObserverError::State {
            path: path.into(),
            message: "post-finalize repository gates binding is invalid".into(),
        });
    }
    Ok(())
}

fn validate_governance_append_revision(
    root: &Path,
    work_item_id: &str,
    previous: &ResourceFinalizationReceipt,
    transition: &ResourceFinalizationTransitionReceipt,
    path: &Path,
) -> Result<(), ObserverError> {
    let Some(append_revision) = transition.governance_append_revision.as_deref() else {
        return Ok(());
    };
    let previous_spec = format!("{}^{{commit}}", previous.pull_request.head_revision);
    let append_spec = format!("{append_revision}^{{commit}}");
    let previous_revision = git_text(
        root,
        &["rev-parse", "--verify", "--end-of-options", &previous_spec],
    )
    .ok_or_else(|| ObserverError::State {
        path: path.into(),
        message: "governance append predecessor revision is not a local commit".into(),
    })?;
    let append_revision = git_text(
        root,
        &["rev-parse", "--verify", "--end-of-options", &append_spec],
    )
    .ok_or_else(|| ObserverError::State {
        path: path.into(),
        message: "governance append revision is not a local commit".into(),
    })?;
    if git_text(
        root,
        &[
            "merge-base",
            "--is-ancestor",
            &previous_revision,
            &append_revision,
        ],
    )
    .is_none()
    {
        return Err(ObserverError::State {
            path: path.into(),
            message: "governance append revision does not descend from the predecessor head".into(),
        });
    }
    let changes = git_text(
        root,
        &[
            "diff",
            "--name-status",
            &previous_revision,
            &append_revision,
            "--",
        ],
    )
    .ok_or_else(|| ObserverError::State {
        path: path.into(),
        message: "cannot inspect governance append revision changes".into(),
    })?;
    let canonical = format!(".ai/decisions/{work_item_id}.finalize.json");
    let transition_prefix = format!(".ai/decisions/{work_item_id}.finalize.");
    let allowed = |candidate: &str| {
        candidate == canonical
            || candidate
                .strip_prefix(&transition_prefix)
                .and_then(|suffix| suffix.strip_suffix(".json"))
                .is_some_and(|digest| {
                    digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
                })
    };
    let mut finalization_count = 0usize;
    let mut quality_route = None;
    let mut repository_gates = None;
    for change in changes.lines() {
        let Some(candidate) = change.strip_prefix("A\t") else {
            return Err(ObserverError::State {
                path: path.into(),
                message: "governance append revision contains a non-append change".into(),
            });
        };
        let evidence_kind = post_finalize_evidence_kind(work_item_id, candidate);
        if !allowed(candidate) && evidence_kind.is_none() {
            return Err(ObserverError::State {
                path: path.into(),
                message: "governance append revision contains a foreign path".into(),
            });
        }
        let tree_entry = git_text(root, &["ls-tree", &append_revision, "--", candidate])
            .ok_or_else(|| ObserverError::State {
                path: path.into(),
                message: "cannot inspect governance append receipt file mode".into(),
            })?;
        if !tree_entry.starts_with("100644 blob ") || !tree_entry.ends_with(candidate) {
            return Err(ObserverError::State {
                path: path.into(),
                message: "governance append receipt is not a regular non-symlink JSON file".into(),
            });
        }
        if allowed(candidate) {
            finalization_count += 1;
        } else if let Some(kind) = evidence_kind {
            let bytes = read_governance_append_blob(root, &append_revision, candidate, path)?;
            match kind {
                PostFinalizeEvidenceKind::QualityRoute => quality_route = Some(bytes),
                PostFinalizeEvidenceKind::RepositoryGates => repository_gates = Some(bytes),
            }
        }
    }
    if finalization_count == 0 {
        return Err(ObserverError::State {
            path: path.into(),
            message: "governance append revision contains no finalization receipt append".into(),
        });
    }
    match (quality_route.as_deref(), repository_gates.as_deref()) {
        (None, None) => {}
        (Some(quality_route), Some(repository_gates)) => validate_post_finalize_evidence_bundle(
            root,
            work_item_id,
            previous,
            &append_revision,
            quality_route,
            repository_gates,
            path,
        )?,
        _ => {
            return Err(ObserverError::State {
                path: path.into(),
                message: "governance append revision must include the complete post-finalize evidence bundle"
                    .into(),
            });
        }
    }
    Ok(())
}

pub(crate) fn resolve_resource_finalization_head(
    root: &Path,
    work_item_id: &str,
) -> Result<(ResourceFinalizationReceipt, PathBuf, Digest, u64), ObserverError> {
    let (receipt, path, digest) = read_resource_finalization_head(root, work_item_id)?;
    let prefix = format!("{work_item_id}.finalize.");
    let canonical_name = format!("{work_item_id}.finalize.json");
    let candidates = fs::read_dir(root.join(".ai/decisions"))
        .map_err(|source| ObserverError::Read {
            path: root.join(".ai/decisions"),
            source,
        })?
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            (name != canonical_name && name.starts_with(&prefix) && name.ends_with(".json"))
                .then_some((entry.path(), name))
        })
        .map(|(candidate, name)| {
            let value = read_resource_finalization_transition(&candidate)?;
            let encoded = serde_json::to_value(&value).map_err(|error| ObserverError::State {
                path: candidate.clone(),
                message: error.to_string(),
            })?;
            let digest =
                cockpit_protocol::digest_json(&encoded).map_err(|error| ObserverError::State {
                    path: candidate.clone(),
                    message: error.to_string(),
                })?;
            let digest = digest.to_string();
            let expected = format!(
                "{work_item_id}.finalize.{}.json",
                digest.strip_prefix("sha256:").unwrap_or(&digest)
            );
            if name != expected {
                return Err(ObserverError::State {
                    path: candidate,
                    message: "resource finalization transition filename digest mismatch".into(),
                });
            }
            Ok((candidate, value))
        })
        .collect::<Result<Vec<_>, _>>()?;
    resolve_resource_finalization_head_from_observed(
        root,
        work_item_id,
        receipt,
        path,
        digest,
        candidates,
    )
}

pub(crate) fn resolve_resource_finalization_head_with_index(
    root: &Path,
    work_item_id: &str,
    index: &status_projection::FinalizationTransitionIndex,
) -> Result<(ResourceFinalizationReceipt, PathBuf, Digest, u64), ObserverError> {
    let (receipt, path, digest) = read_resource_finalization_head(root, work_item_id)?;
    let candidates = index
        .candidates(work_item_id)
        .iter()
        .map(|candidate| match candidate {
            status_projection::IndexedFinalizationTransition::Valid {
                path,
                digest,
                value,
            } => {
                let _validated_digest = digest;
                Ok((path.clone(), (**value).clone()))
            }
            status_projection::IndexedFinalizationTransition::Invalid { path, message } => {
                Err(ObserverError::State {
                    path: path.clone(),
                    message: message.clone(),
                })
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    resolve_resource_finalization_head_from_observed(
        root,
        work_item_id,
        receipt,
        path,
        digest,
        candidates,
    )
}

fn read_resource_finalization_head(
    root: &Path,
    work_item_id: &str,
) -> Result<(ResourceFinalizationReceipt, PathBuf, Digest), ObserverError> {
    let canonical = resource_finalization_decision_path(root, work_item_id);
    let receipt = read_resource_finalization_receipt(&canonical)?;
    let digest =
        cockpit_protocol::digest_json(&serde_json::to_value(&receipt).map_err(|error| {
            ObserverError::State {
                path: canonical.clone(),
                message: error.to_string(),
            }
        })?)
        .map_err(|error| ObserverError::State {
            path: canonical.clone(),
            message: error.to_string(),
        })?;
    Ok((receipt, canonical, digest))
}

fn resolve_resource_finalization_head_from_observed(
    root: &Path,
    work_item_id: &str,
    mut receipt: ResourceFinalizationReceipt,
    mut path: PathBuf,
    mut digest: Digest,
    mut candidates: Vec<(PathBuf, ResourceFinalizationTransitionReceipt)>,
) -> Result<(ResourceFinalizationReceipt, PathBuf, Digest, u64), ObserverError> {
    let schema_v2_root = receipt.is_post_merge_sync();
    if schema_v2_root && receipt.result.disposition != ResourceFinalizationDisposition::Retained {
        return Err(ObserverError::State {
            path: path.clone(),
            message: "schema-v2 post-merge sync canonical root must be Retained; Deleted must be an append-only transition".into(),
        });
    }
    let mut sequence = 0;
    loop {
        let matches = candidates
            .iter()
            .enumerate()
            .filter(|(_, (_, value))| value.predecessor_receipt_digest == digest)
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if matches.is_empty() {
            break;
        }
        if matches.len() != 1 {
            return Err(ObserverError::State {
                path: path.clone(),
                message: "resource finalization transition chain is forked".into(),
            });
        }
        let (next_path, transition) = candidates.remove(matches[0]);
        validate_resource_finalization_transition(&receipt, &transition, sequence + 1).map_err(
            |error| ObserverError::State {
                path: next_path.clone(),
                message: error.to_string(),
            },
        )?;
        validate_governance_append_revision(root, work_item_id, &receipt, &transition, &next_path)?;
        sequence += 1;
        receipt = transition.receipt;
        path = next_path;
        digest =
            cockpit_protocol::digest_json(&serde_json::to_value(&receipt).map_err(|error| {
                ObserverError::State {
                    path: path.clone(),
                    message: error.to_string(),
                }
            })?)
            .map_err(|error| ObserverError::State {
                path: path.clone(),
                message: error.to_string(),
            })?;
    }
    if !candidates.is_empty() {
        return Err(ObserverError::State {
            path: candidates[0].0.clone(),
            message: "resource finalization transition has a missing or stale predecessor".into(),
        });
    }
    Ok((receipt, path, digest, sequence))
}

pub(crate) fn archived_contract_digest(
    root: &Path,
    work_item_id: &str,
) -> Result<(Contract, Digest), ObserverError> {
    let path = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&path)?;
    let digest = Digest::sha256_bytes(&fs::read(&path).map_err(|source| ObserverError::Read {
        path: path.clone(),
        source,
    })?);
    Ok((contract, digest))
}

fn ensure_resource_runtime_identity(
    receipt: &ResourceFinalizationReceipt,
    runtime: &RuntimeContext,
    path: &Path,
) -> Result<(), ObserverError> {
    if receipt.runtime_version != runtime.runtime_version
        || receipt.runtime_digest != runtime.runtime_digest
    {
        return Err(ObserverError::State {
            path: path.into(),
            message: "resource finalization receipt Runtime identity does not match the executing Runtime".into(),
        });
    }
    Ok(())
}

/// Return the historical classification for an old-runtime finalization head
/// that is already closed.  Historical bytes remain immutable: the only
/// authority for this projection is the close receipt binding its exact head
/// path, digest, sequence, Work Item, and repository identity.
pub(crate) fn closed_finalization_projection_kind(
    root: &Path,
    work_item_id: &str,
    receipt: &ResourceFinalizationReceipt,
    receipt_path: &Path,
    receipt_digest: &Digest,
    sequence: u64,
    repository_id: &str,
) -> Option<&'static str> {
    if !close_decision_is_valid_for_status(root, work_item_id, repository_id) {
        return None;
    }
    let close_path = root
        .join(".ai/decisions")
        .join(format!("{work_item_id}.close.json"));
    let close = read_json(&close_path).ok()?;
    if close.get("resourceFinalizationSequence")?.as_u64()? != sequence
        || close.get("resourceFinalizationHeadPath")?.as_str()?
            != repository_relative_path(root, receipt_path)
        || close.get("resourceFinalizationHeadDigest")?.as_str()? != receipt_digest.to_string()
    {
        return None;
    }
    if let Some(historical) = close.get("historicalRevalidation")
        && historical.get("state").and_then(serde_json::Value::as_str)
            == Some("current_successor_revalidated")
        && let Ok(Some(recovery)) = load_recovery_decision(root, work_item_id, None)
        && recovery.successor_binding_mode.as_deref() == Some("contract_amendment_revalidation")
        && recovery.successor_work_item_id.as_deref()
            == historical
                .get("successorWorkItemId")
                .and_then(serde_json::Value::as_str)
        && recovery.predecessor_finalization_contract_digest.as_ref()
            == receipt.contract_digest.as_ref()
        && historical
            .get("assurance")
            .and_then(serde_json::Value::as_str)
            == Some("historical_low")
        && historical
            .get("originalEvidencePreserved")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
        && historical_digest_matches(
            historical,
            "historicalContractDigest",
            &recovery.predecessor_contract_digest,
        )
        && recovery
            .current_contract_digest
            .as_ref()
            .is_some_and(|digest| {
                historical_digest_matches(historical, "currentContractDigest", digest)
            })
        && recovery
            .predecessor_verification_evidence_digest
            .as_ref()
            .is_some_and(|digest| {
                historical_digest_matches(
                    historical,
                    "historicalVerificationEvidenceDigest",
                    digest,
                )
            })
        && recovery
            .predecessor_archive_manifest_digest
            .as_ref()
            .is_some_and(|digest| {
                historical_digest_matches(historical, "archiveManifestDigest", digest)
            })
        && recovery_successor_resolves_pending_close(root, work_item_id, repository_id)
    {
        return Some("contract_amendment_revalidation");
    }
    if close.get("historicalRevalidation").is_some() {
        // A close record that declares this projection must satisfy the full
        // lineage binding above. Do not downgrade an invalid declaration to
        // the generic shared-worktree compatibility lane.
        return None;
    }
    if matches!(
        receipt.result.disposition,
        ResourceFinalizationDisposition::Retained
    ) && receipt.before.branch == receipt.after.branch
        && receipt.before.worktree == receipt.after.worktree
    {
        Some("shared_worktree_retained")
    } else {
        Some("legacy_runtime")
    }
}

fn historical_digest_matches(
    historical: &serde_json::Value,
    field: &str,
    expected: &Digest,
) -> bool {
    historical
        .get(field)
        .and_then(serde_json::Value::as_str)
        .and_then(|value| value.parse::<Digest>().ok())
        .is_some_and(|actual| actual == *expected)
}

pub(crate) fn ensure_resource_finalization_base_binding(
    receipt: &ResourceFinalizationReceipt,
    contract: &Contract,
    path: &Path,
) -> Result<(), ObserverError> {
    if let Some(contract_base_revision) = receipt.contract_base_revision.as_deref() {
        if contract_base_revision != contract.base_revision {
            return Err(ObserverError::State {
                path: path.into(),
                message: format!(
                    "resource finalization Contract base revision binding does not match the archived Contract base revision: expected {}, receipt has {}",
                    contract.base_revision, contract_base_revision
                ),
            });
        }
        // The provider's PR base is an independent comparison identity.  The
        // explicit Contract binding above is the authorization boundary, so
        // a refreshed branch may legitimately report a different PR base.
        return Ok(());
    }
    if receipt.pull_request.base_revision == contract.base_revision {
        return Ok(());
    }
    let direct_merge = receipt.historical.as_ref().filter(|historical| {
        matches!(
            historical.kind,
            cockpit_protocol::HistoricalFinalizationKind::DirectMergeNoPr
        )
    });
    if let Some(historical) = direct_merge {
        if historical.contract_base_revision.as_deref() == Some(contract.base_revision.as_str()) {
            // `pullRequest.baseRevision` is the real first parent of the
            // historical merge. The immutable Contract base is bound
            // separately so a bundled merge can be recorded without
            // rewriting either fact or inventing a PR.
            return Ok(());
        }
        return Err(ObserverError::State {
            path: path.into(),
            message: format!(
                "historical direct-merge Contract base mismatch: receipt must bind historical.contractBaseRevision={} while pullRequest.baseRevision remains the real merge first parent ({})",
                contract.base_revision, receipt.pull_request.base_revision
            ),
        });
    }
    Err(ObserverError::State {
        path: path.into(),
        message: "resource finalization pull request base revision does not match the archived Contract base revision".into(),
    })
}

/// Validate the additional facts that make a historical compatibility receipt
/// honest.  The protocol validates the typed shape; this repository-bound
/// check binds a direct merge to the actual Git commit/parents and restricts
/// shared-worktree history to the repository's primary checkout.
pub(crate) fn validate_historical_finalization(
    root: &Path,
    receipt: &ResourceFinalizationReceipt,
    path: &Path,
) -> Result<(), ObserverError> {
    let Some(historical) = receipt.historical.as_ref() else {
        return Ok(());
    };
    match historical.kind {
        HistoricalFinalizationKind::SharedWorktreeRetained => {
            let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
                path: root.into(),
                source,
            })?;
            let worktree = fs::canonicalize(&receipt.worktree.path).map_err(|source| {
                ObserverError::State {
                    path: path.into(),
                    message: format!("historical shared worktree is not present: {source}"),
                }
            })?;
            if worktree != root || receipt.worktree.branch != receipt.branch.name {
                return Err(ObserverError::State {
                    path: path.into(),
                    message: "historical shared-worktree receipt is not bound to the primary repository worktree".into(),
                });
            }
        }
        HistoricalFinalizationKind::DirectMergeNoPr => {
            let merge_commit = receipt
                .historical
                .as_ref()
                .and_then(|value| value.merge_commit.as_deref())
                .ok_or_else(|| ObserverError::State {
                    path: path.into(),
                    message: "historical direct-merge receipt is missing merge commit".into(),
                })?;
            let observation = git_text(root, &["rev-list", "--parents", "-n", "1", merge_commit])
                .ok_or_else(|| ObserverError::State {
                path: path.into(),
                message: "historical direct-merge commit is not present in this repository".into(),
            })?;
            let parts = observation.split_whitespace().collect::<Vec<_>>();
            let parents = receipt
                .historical
                .as_ref()
                .map(|value| {
                    value
                        .merge_parents
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if parts.first().copied() != Some(merge_commit)
                || parts.get(1..).unwrap_or_default() != parents.as_slice()
                || parents.len() < 2
            {
                return Err(ObserverError::State {
                    path: path.into(),
                    message: "historical direct-merge commit parents do not match Git".into(),
                });
            }
            let base = receipt
                .historical
                .as_ref()
                .map(|value| value.base_revision.as_str())
                .unwrap_or_default();
            if parents.first().copied() != Some(base) {
                return Err(ObserverError::State {
                    path: path.into(),
                    message: format!(
                        "historical direct-merge base must equal Git's first parent: expected {}, receipt has {}",
                        parents.first().copied().unwrap_or("<missing>"),
                        base
                    ),
                });
            }
            let ancestor = Command::new("git")
                .arg("-C")
                .arg(root)
                .args(["merge-base", "--is-ancestor", base, merge_commit])
                .status()
                .map(|status| status.success())
                .unwrap_or(false);
            if !ancestor {
                return Err(ObserverError::State {
                    path: path.into(),
                    message: "historical direct-merge base is not an ancestor of merge commit"
                        .into(),
                });
            }
        }
        HistoricalFinalizationKind::LegacyRuntime => {}
    }
    Ok(())
}

fn historical_finalization_recovery_path(root: &Path, work_item_id: &str) -> PathBuf {
    root.join(".ai/decisions")
        .join(format!("{work_item_id}.finalize-recovery.json"))
}

fn read_historical_finalization_recovery(
    path: &Path,
) -> Result<HistoricalFinalizationRecoveryReceipt, ObserverError> {
    if !is_regular_non_symlink(path)? {
        return Err(ObserverError::State {
            path: path.into(),
            message: "historical finalization recovery must be a regular non-symlink file".into(),
        });
    }
    let bytes = fs::read(path).map_err(|source| ObserverError::Read {
        path: path.into(),
        source,
    })?;
    if bytes.len() > MAX_EXTERNAL_EVIDENCE_BYTES {
        return Err(ObserverError::State {
            path: path.into(),
            message: "historical finalization recovery exceeds the bounded size limit".into(),
        });
    }
    reject_duplicate_json_keys(&bytes).map_err(|message| ObserverError::State {
        path: path.into(),
        message: format!("invalid historical finalization recovery JSON: {message}"),
    })?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|error| ObserverError::State {
            path: path.into(),
            message: format!("invalid historical finalization recovery JSON: {error}"),
        })?;
    serde_json::from_value(value).map_err(|error| ObserverError::State {
        path: path.into(),
        message: format!("invalid historical finalization recovery: {error}"),
    })
}

fn validate_historical_finalization_recovery_binding(
    root: &Path,
    work_item_id: &str,
    recovery: &HistoricalFinalizationRecoveryReceipt,
    receipt: &ResourceFinalizationReceipt,
    contract: &Contract,
    recovery_path: &Path,
    current_runtime: &RuntimeContext,
) -> Result<(), ObserverError> {
    validate_historical_finalization_recovery(recovery).map_err(|error| ObserverError::State {
        path: recovery_path.into(),
        message: error.to_string(),
    })?;
    let expected_repository_id = repository_id(root).to_string();
    if recovery.work_item_id != work_item_id
        || recovery.repository_id != expected_repository_id
        || receipt.work_item_id != work_item_id
        || receipt.repository_id != expected_repository_id
    {
        return Err(ObserverError::State {
            path: recovery_path.into(),
            message: "historical finalization recovery repository or Work Item identity mismatch"
                .into(),
        });
    }
    let expected_path = repository_relative_path(
        root,
        &resource_finalization_decision_path(root, work_item_id),
    );
    if recovery.predecessor_path != expected_path {
        return Err(ObserverError::State {
            path: recovery_path.into(),
            message: "historical finalization recovery predecessor path mismatch".into(),
        });
    }
    let predecessor_path = root.join(&recovery.predecessor_path);
    let predecessor_value =
        serde_json::to_value(receipt).map_err(|error| ObserverError::State {
            path: predecessor_path.clone(),
            message: error.to_string(),
        })?;
    let predecessor_digest =
        cockpit_protocol::digest_json(&predecessor_value).map_err(|error| {
            ObserverError::State {
                path: predecessor_path.clone(),
                message: error.to_string(),
            }
        })?;
    if recovery.predecessor_receipt_digest != predecessor_digest {
        return Err(ObserverError::State {
            path: recovery_path.into(),
            message: "historical finalization recovery predecessor digest mismatch".into(),
        });
    }
    if recovery.base_revision != contract.base_revision
        || receipt.pull_request.base_revision != contract.base_revision
    {
        return Err(ObserverError::State {
            path: recovery_path.into(),
            message: "historical finalization recovery base revision mismatch".into(),
        });
    }
    if recovery.runtime_version != current_runtime.runtime_version
        || recovery.runtime_digest != current_runtime.runtime_digest
    {
        return Err(ObserverError::State {
            path: recovery_path.into(),
            message: "historical finalization recovery Runtime identity does not match the executing Runtime".into(),
        });
    }
    if receipt.runtime_version == current_runtime.runtime_version
        && receipt.runtime_digest == current_runtime.runtime_digest
    {
        return Err(ObserverError::State {
            path: recovery_path.into(),
            message: "historical finalization recovery requires an older predecessor Runtime"
                .into(),
        });
    }
    match recovery.historical_kind {
        HistoricalFinalizationKind::SharedWorktreeRetained => {
            if !matches!(
                receipt.result.disposition,
                ResourceFinalizationDisposition::Retained
            ) {
                return Err(ObserverError::State {
                    path: recovery_path.into(),
                    message: "shared-worktree historical recovery requires retained disposition"
                        .into(),
                });
            }
            let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
                path: root.into(),
                source,
            })?;
            let worktree = fs::canonicalize(&receipt.worktree.path).map_err(|source| {
                ObserverError::State {
                    path: recovery_path.into(),
                    message: format!("historical shared worktree is not present: {source}"),
                }
            })?;
            if worktree != root || receipt.worktree.branch != receipt.branch.name {
                return Err(ObserverError::State {
                    path: recovery_path.into(),
                    message: "historical shared-worktree recovery is not bound to the primary repository worktree".into(),
                });
            }
        }
        HistoricalFinalizationKind::DirectMergeNoPr => {
            return Err(ObserverError::State {
                path: recovery_path.into(),
                message: "direct-merge history must be recorded as a complete historical finalization receipt, not a reclassification of a PR receipt".into(),
            });
        }
        HistoricalFinalizationKind::LegacyRuntime => {
            if let Err(reason) = validate_legacy_runtime_candidate(receipt) {
                return Err(ObserverError::State {
                    path: recovery_path.into(),
                    message: format!("legacy-runtime recovery is not applicable: {reason}"),
                });
            }
        }
    }
    Ok(())
}

fn validate_legacy_runtime_candidate(
    receipt: &ResourceFinalizationReceipt,
) -> Result<(), &'static str> {
    if let Some(historical) = receipt.historical.as_ref()
        && !matches!(historical.kind, HistoricalFinalizationKind::LegacyRuntime)
    {
        return Err("the predecessor historical classification does not match legacy_runtime");
    }
    if receipt.pull_request.number == 0 {
        return Err("a real merged pull request number is required");
    }
    if receipt.pull_request.merge_commit.is_none() {
        return Err("a merge commit is required");
    }
    if !matches!(
        receipt.before.pull_request,
        ResourceFinalizationPullRequestState::Merged
    ) || !matches!(
        receipt.after.pull_request,
        ResourceFinalizationPullRequestState::Merged
    ) {
        return Err(
            "the predecessor must prove merged pull-request state before and after finalization",
        );
    }
    Ok(())
}

fn load_historical_finalization_recovery(
    root: &Path,
    work_item_id: &str,
    receipt: &ResourceFinalizationReceipt,
    contract: &Contract,
    current_runtime: &RuntimeContext,
) -> Result<Option<HistoricalFinalizationRecoveryReceipt>, ObserverError> {
    let path = historical_finalization_recovery_path(root, work_item_id);
    if fs::symlink_metadata(&path).is_err() {
        return Ok(None);
    }
    let recovery = read_historical_finalization_recovery(&path)?;
    validate_historical_finalization_recovery_binding(
        root,
        work_item_id,
        &recovery,
        receipt,
        contract,
        &path,
        current_runtime,
    )?;
    Ok(Some(recovery))
}

/// Record an explicit Runtime-bound classification for a legacy finalization
/// receipt. The predecessor remains byte-for-byte immutable; the new record
/// is accepted only when it binds the exact predecessor digest and the
/// current Runtime identity.
pub fn record_historical_finalization_recovery(
    root: &Path,
    work_item_id: &str,
    input_path: &Path,
    runtime: &RuntimeContext,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let _lifecycle_lock = acquire_lifecycle_lock(&root, work_item_id)?;
    let (contract, contract_digest) = archived_contract_digest(&root, work_item_id)?;
    let expected_resource_context = effective_resource_context(&root, work_item_id, &contract)?;
    let predecessor_path = resource_finalization_decision_path(&root, work_item_id);
    if fs::symlink_metadata(&predecessor_path).is_err() {
        // A direct merge without a PR can be the first finalization record:
        // there is no immutable predecessor to classify.  Accept only a
        // complete, explicitly historical direct-merge receipt and route it
        // through the same strict archive/Contract/Git/runtime validation as
        // `finalize`; all other recovery inputs remain fail-closed.
        if let Ok(candidate) = read_resource_finalization_receipt(input_path)
            && matches!(
                candidate.historical.as_ref().map(|value| &value.kind),
                Some(HistoricalFinalizationKind::DirectMergeNoPr)
            )
        {
            return record_resource_finalization(&root, work_item_id, input_path, runtime);
        }
        return Err(ObserverError::State {
            path: predecessor_path,
            message: "historical finalization recovery requires an existing predecessor; for a first-record direct merge use a complete direct_merge_no_pr receipt".into(),
        });
    }
    let receipt = read_resource_finalization_receipt(&predecessor_path)?;
    validate_resource_finalization_receipt_for(
        &receipt,
        &contract.repository_id,
        work_item_id,
        Some(&contract_digest),
        expected_resource_context.as_ref(),
    )
    .map_err(|error| ObserverError::State {
        path: predecessor_path.clone(),
        message: error.to_string(),
    })?;
    let recovery = read_historical_finalization_recovery(input_path)?;
    validate_historical_finalization_recovery_binding(
        &root,
        work_item_id,
        &recovery,
        &receipt,
        &contract,
        input_path,
        runtime,
    )?;
    let recovery_path = historical_finalization_recovery_path(&root, work_item_id);
    let value = serde_json::to_value(&recovery).map_err(|error| ObserverError::State {
        path: recovery_path.clone(),
        message: error.to_string(),
    })?;
    if fs::symlink_metadata(&recovery_path).is_ok() {
        if !is_regular_non_symlink(&recovery_path)? {
            return Err(ObserverError::State {
                path: recovery_path,
                message: "historical finalization recovery destination is not a regular file"
                    .into(),
            });
        }
        let existing = read_json(&recovery_path)?;
        if existing == value {
            return Ok(serde_json::json!({
                "workItemId": work_item_id,
                "state": "idempotent",
                "path": repository_relative_path(&root, &recovery_path)
            }));
        }
        return Err(ObserverError::State {
            path: recovery_path,
            message: "historical finalization recovery already exists with different content"
                .into(),
        });
    }
    atomic_json(&recovery_path, &value)?;
    Ok(serde_json::json!({
        "workItemId": work_item_id,
        "state": "recorded",
        "historicalKind": recovery.historical_kind,
        "assurance": recovery.assurance,
        "path": repository_relative_path(&root, &recovery_path)
    }))
}

/// Produce a read-only, fact-bound recovery plan for a legacy finalization.
/// The plan deliberately contains no generated human authority or decision;
/// it only supplies immutable predecessor facts and, when explicitly given,
/// Git's real direct-merge parents.
pub fn historical_finalization_recovery_plan(
    root: &Path,
    work_item_id: &str,
    runtime: &RuntimeContext,
    merge_commit: Option<&str>,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let repository_id = repository_id(&root).to_string();
    let mut result = serde_json::json!({
        "workItemId": work_item_id,
        "repositoryId": repository_id,
        "runtimeVersion": runtime.runtime_version,
        "runtimeDigest": runtime.runtime_digest,
        "state": "needs_human_review",
        "writesRepositoryState": false,
        "humanInputRequired": ["authoritySource", "reason", "decidedAt"],
    });

    // An archived Contract is read-only input to the plan.  Expose its digest
    // and immutable base so a human can bind the eventual receipt without
    // guessing which historical snapshot the Work Item used.  Provisional
    // resource-context values are reported as facts, never promoted to
    // concrete provider identity.
    let archived_contract = archived_contract_digest(&root, work_item_id).ok();
    if let Some((contract, digest)) = archived_contract.as_ref() {
        result["contractDigest"] = digest.to_string().into();
        result["contractBaseRevision"] = contract.base_revision.clone().into();
        if let Some(context) = contract.resource_context.as_ref() {
            result["contractResourceContext"] =
                serde_json::to_value(context).map_err(|error| ObserverError::State {
                    path: root.clone(),
                    message: error.to_string(),
                })?;
        }
    }

    if let Ok((receipt, path, digest, sequence)) =
        resolve_resource_finalization_head(&root, work_item_id)
    {
        let contract = archived_contract_digest(&root, work_item_id).ok();
        if matches!(
            receipt
                .historical
                .as_ref()
                .map(|historical| &historical.kind),
            Some(HistoricalFinalizationKind::LegacyRuntime)
        ) && let Err(reason) = validate_legacy_runtime_candidate(&receipt)
        {
            return Err(ObserverError::State {
                path: path.clone(),
                message: format!("legacy-runtime recovery is not applicable: {reason}"),
            });
        }
        let kind = receipt
            .historical
            .as_ref()
            .map(|historical| match historical.kind {
                HistoricalFinalizationKind::SharedWorktreeRetained => "shared_worktree_retained",
                HistoricalFinalizationKind::DirectMergeNoPr => "direct_merge_no_pr",
                HistoricalFinalizationKind::LegacyRuntime => "legacy_runtime",
            })
            .or_else(|| {
                contract.as_ref().and_then(|(contract, _)| {
                    infer_legacy_shared_worktree_retained(&root, &receipt, contract)
                        .then_some("shared_worktree_retained")
                })
            });
        let kind = kind.or_else(|| {
            validate_legacy_runtime_candidate(&receipt)
                .is_ok()
                .then_some("legacy_runtime")
        });
        let stale = receipt.runtime_version != runtime.runtime_version
            || receipt.runtime_digest != runtime.runtime_digest;
        let closed = stale
            && contract.as_ref().is_some_and(|(contract, _)| {
                closed_finalization_projection_kind(
                    &root,
                    work_item_id,
                    &receipt,
                    &path,
                    &digest,
                    sequence,
                    &contract.repository_id,
                )
                .is_some()
            });
        result["predecessorPath"] = repository_relative_path(&root, &path).into();
        result["predecessorDigest"] = digest.to_string().into();
        result["sequence"] = sequence.into();
        result["predecessorRuntimeVersion"] = receipt.runtime_version.clone().into();
        result["predecessorRuntimeDigest"] = receipt.runtime_digest.to_string().into();
        result["historicalKind"] = kind
            .map(serde_json::Value::from)
            .unwrap_or(serde_json::Value::Null);
        result["baseRevision"] = receipt.pull_request.base_revision.clone().into();
        if closed {
            result["state"] = "already_closed_historical".into();
            result["assurance"] = "historical_low".into();
            result["humanInputRequired"] = serde_json::json!([]);
        } else if !stale {
            result["state"] = "current_runtime_no_recovery_required".into();
            result["humanInputRequired"] = serde_json::json!([]);
        } else if let Some(kind) = kind {
            result["suggestedRecovery"] = serde_json::json!({
                "kind": "historical_finalization_recovery",
                "historicalKind": kind,
                "assurance": "historical_low",
                "workItemId": work_item_id,
                "repositoryId": repository_id,
                "predecessorPath": repository_relative_path(&root, &path),
                "predecessorReceiptDigest": digest,
                "baseRevision": receipt.pull_request.base_revision,
                "runtimeVersion": runtime.runtime_version,
                "runtimeDigest": runtime.runtime_digest,
            });
        }
        return Ok(result);
    }

    let Some(merge_commit) = merge_commit else {
        result["state"] = "predecessor_missing_or_unreadable".into();
        result["safeAction"] =
            "provide a real merge commit for historical direct-merge inspection".into();
        return Ok(result);
    };
    let parents = git_text(&root, &["rev-list", "--parents", "-n", "1", merge_commit])
        .ok_or_else(|| ObserverError::State {
            path: root.clone(),
            message: "cannot inspect historical direct-merge commit".into(),
        })?
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if parents.len() < 3 {
        return Err(ObserverError::State {
            path: root,
            message: "historical direct-merge commit must have at least two parents".into(),
        });
    }
    result["state"] = "direct_merge_candidate".into();
    result["historicalKind"] = "direct_merge_no_pr".into();
    result["assurance"] = "historical_low".into();
    result["mergeCommit"] = merge_commit.into();
    result["mergeParents"] =
        serde_json::to_value(&parents[1..]).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    result["baseRevision"] = parents[1].clone().into();
    let head_revision = parents[2].clone();
    result["knownFacts"] = serde_json::json!({
        "schemaVersion": 1,
        "workItemId": work_item_id,
        "repositoryId": repository_id,
        "runtimeVersion": runtime.runtime_version,
        "runtimeDigest": runtime.runtime_digest,
        "contractBaseRevision": archived_contract
            .as_ref()
            .map(|(contract, _)| contract.base_revision.clone()),
        "pullRequest": {
            "number": 0,
            "url": format!("historical://direct-merge/{merge_commit}"),
            "headRevision": head_revision,
            "baseRevision": parents[1],
            "mergeCommit": merge_commit
        },
        "historical": {
            "kind": "direct_merge_no_pr",
            "assurance": "historical_low",
            "baseRevision": parents[1],
            "mergeCommit": merge_commit,
            "mergeParents": &parents[1..]
        }
    });
    if let Some((contract, _)) = archived_contract.as_ref() {
        result["knownFacts"]["historical"]["contractBaseRevision"] =
            contract.base_revision.clone().into();
    }
    if let Some((_, digest)) = archived_contract.as_ref() {
        result["knownFacts"]["contractDigest"] = digest.to_string().into();
    }
    if let Some((contract, _)) = archived_contract.as_ref()
        && let Some(context) = contract.resource_context.as_ref()
        && !context.is_provisional()
    {
        result["knownFacts"]["pullRequest"]["baseBranch"] = context.base_branch.clone().into();
        result["knownFacts"]["pullRequest"]["baseRemote"] = context.base_remote.clone().into();
    }
    // The merge commit proves that the historical operation was merged, but
    // it does not prove whether the old branch/worktree was later removed.
    // Emit that distinction explicitly instead of asking the caller to
    // reconstruct a partially typed receipt.  `retained` plus a stable
    // unknown code is the conservative closeable projection for historical
    // low-assurance records; a human may replace it with `deleted` only when
    // fresh cleanup evidence proves the stronger claim.
    let historical_url = format!("historical://direct-merge/{merge_commit}");
    let context = archived_contract
        .as_ref()
        .and_then(|(contract, _)| contract.resource_context.as_ref())
        .filter(|context| !context.is_provisional());
    let mut human_input_required = vec![
        "actor".to_owned(),
        "authoritySource".to_owned(),
        "reason".to_owned(),
        "timestamp".to_owned(),
    ];
    let mut suggested_receipt = serde_json::json!({
        "schemaVersion": 1,
        "receiptId": format!("historical-direct-merge-{work_item_id}-{merge_commit}"),
        "operationId": format!("historical-direct-merge-operation-{merge_commit}"),
        "runtimeVersion": runtime.runtime_version,
        "runtimeDigest": runtime.runtime_digest,
        "pullRequest": {
            "number": 0,
            "url": historical_url,
            "headRevision": parents[2],
            "baseBranch": "unknown",
            "baseRemote": "unknown",
            "baseRevision": parents[1],
            "mergeCommit": merge_commit
        },
        "provider": "historical",
        "branch": {
            "name": "unknown",
            "remote": "unknown",
            "headRevision": parents[2]
        },
        "worktree": {
            "worktreeId": "unknown",
            "path": "unknown",
            "branch": "unknown",
            "headRevision": parents[2]
        },
        "before": {
            "pullRequest": "merged",
            "branch": "unknown",
            "worktree": "unknown"
        },
        "after": {
            "pullRequest": "merged",
            "branch": "unknown",
            "worktree": "unknown"
        },
        "result": {
            "disposition": "retained",
            "failureCodes": [],
            "unknownCodes": ["historical_resource_state_unknown"]
        },
        "historical": {
            "kind": "direct_merge_no_pr",
            "assurance": "historical_low",
            "baseRevision": parents[1],
            "mergeCommit": merge_commit,
            "mergeParents": &parents[1..],
            "contractBaseRevision": archived_contract
                .as_ref()
                .map(|(contract, _)| contract.base_revision.clone())
        },
        "workItemId": work_item_id,
        "repositoryId": repository_id
    });
    if let Some(context) = context {
        suggested_receipt["pullRequest"]["baseBranch"] = context.base_branch.clone().into();
        suggested_receipt["pullRequest"]["baseRemote"] = context.base_remote.clone().into();
        suggested_receipt["branch"] = serde_json::json!({
            "name": context.branch,
            "remote": context.base_remote,
            "headRevision": parents[2]
        });
        suggested_receipt["worktree"] = serde_json::json!({
            "worktreeId": format!("historical-direct-merge-{merge_commit}"),
            "path": context.worktree,
            "branch": context.branch,
            "headRevision": parents[2]
        });
        suggested_receipt["resourceContext"] = serde_json::json!({
            "branch": context.branch,
            "worktree": context.worktree,
            "baseBranch": context.base_branch,
            "baseRemote": context.base_remote,
            "provider": "historical",
            "pullRequest": historical_url,
        });
    } else {
        human_input_required.extend([
            "pullRequest.baseBranch".to_owned(),
            "pullRequest.baseRemote".to_owned(),
            "resourceContext".to_owned(),
            "branch.name".to_owned(),
            "branch.remote".to_owned(),
            "worktree.worktreeId".to_owned(),
            "worktree.path".to_owned(),
            "worktree.branch".to_owned(),
        ]);
    }
    result["humanInputRequired"] = human_input_required.into();
    result["suggestedReceipt"] = suggested_receipt;
    if let Some((_, digest)) = archived_contract.as_ref() {
        result["suggestedReceipt"]["contractDigest"] = digest.to_string().into();
    }
    Ok(result)
}

/// Persist a provider-side finalization receipt after strict identity and
/// local postcondition validation.  The Runtime never calls a provider or
/// deletes a branch implicitly; it records delegated evidence and refuses
/// close on blocked/unknown/contradictory results.
pub fn record_resource_finalization(
    root: &Path,
    work_item_id: &str,
    receipt_path: &Path,
    runtime: &RuntimeContext,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let close_path = root
        .join(".ai/decisions")
        .join(format!("{work_item_id}.close.json"));
    let close_present = fs::symlink_metadata(&close_path).is_ok();
    let manifest_path = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.archive.json"));
    let manifest = read_json(&manifest_path)?;
    verify_archive_manifest(&root, work_item_id, &manifest)?;
    let (contract, contract_digest) = archived_contract_digest(&root, work_item_id)?;
    let expected_resource_context = effective_resource_context(&root, work_item_id, &contract)?;
    let input_value = read_json(receipt_path)?;
    let transition = input_value
        .get("receipt")
        .map(|_| read_resource_finalization_transition(receipt_path))
        .transpose()?;
    let receipt = if let Some(transition) = &transition {
        transition.receipt.clone()
    } else {
        read_resource_finalization_receipt(receipt_path)?
    };
    if close_present && transition.is_none() {
        return Err(ObserverError::State {
            path: close_path.clone(),
            message: "resource finalization reconciliation after close requires an append-only transition".into(),
        });
    }
    if transition.is_none() {
        validate_resource_finalization_receipt_for(
            &receipt,
            &contract.repository_id,
            work_item_id,
            Some(&contract_digest),
            expected_resource_context.as_ref(),
        )
        .map_err(|error| ObserverError::State {
            path: receipt_path.into(),
            message: error.to_string(),
        })?;
    }
    validate_historical_finalization(&root, &receipt, receipt_path)?;
    validate_post_merge_sync_repository_evidence(&root, &receipt)?;
    ensure_resource_finalization_base_binding(&receipt, &contract, receipt_path)?;
    if receipt.provider == "unknown"
        || receipt
            .resource_context
            .as_ref()
            .is_some_and(|context| context.provider == "unknown")
    {
        return Err(ObserverError::State {
            path: receipt_path.into(),
            message: "resource finalization requires an identified external provider".into(),
        });
    }
    // A post-close transition repairs an immutable receipt emitted by an older
    // Runtime.  Bind it to the predecessor's Runtime identity below instead
    // of rejecting a valid historical chain merely because the validator was
    // upgraded.  New canonical receipts and pre-close transitions still must
    // match the executing Runtime.
    if !(close_present && transition.is_some()) {
        ensure_resource_runtime_identity(&receipt, runtime, receipt_path)?;
    }
    if matches!(
        receipt.result.disposition,
        ResourceFinalizationDisposition::Deleted | ResourceFinalizationDisposition::Abandoned
    ) && !local_resources_deleted(&root, &receipt)?
    {
        return Err(ObserverError::State {
            path: receipt_path.into(),
            message:
                "finalization receipt does not match local branch/worktree cleanup postconditions"
                    .into(),
        });
    }
    let decision_path = resource_finalization_decision_path(&root, work_item_id);
    if decision_path.exists() {
        let (existing, head_path, _head_digest, sequence) =
            resolve_resource_finalization_head(&root, work_item_id)?;
        if let Some(transition) = transition.as_ref() {
            if close_present {
                validate_post_close_finalization_reconciliation(
                    &root,
                    work_item_id,
                    &close_path,
                    &existing,
                    &head_path,
                    sequence,
                    transition,
                )?;
            }
            validate_resource_finalization_transition(&existing, transition, sequence + 1)
                .map_err(|error| ObserverError::State {
                    path: receipt_path.into(),
                    message: error.to_string(),
                })?;
            validate_governance_append_revision(
                &root,
                work_item_id,
                &existing,
                transition,
                receipt_path,
            )?;
            let value = serde_json::to_value(transition).map_err(|error| ObserverError::State {
                path: receipt_path.into(),
                message: error.to_string(),
            })?;
            let transition_digest =
                cockpit_protocol::digest_json(&value).map_err(|error| ObserverError::State {
                    path: receipt_path.into(),
                    message: error.to_string(),
                })?;
            let suffix = transition_digest.to_string();
            let appended_path = root.join(".ai/decisions").join(format!(
                "{work_item_id}.finalize.{}.json",
                suffix.strip_prefix("sha256:").unwrap_or(&suffix)
            ));
            if appended_path.exists() {
                let existing_value = read_json(&appended_path)?;
                if existing_value == value {
                    return Ok(
                        serde_json::json!({"workItemId": work_item_id, "state": "idempotent", "disposition": receipt.result.disposition, "path": repository_relative_path(&root, &appended_path)}),
                    );
                }
                return Err(ObserverError::State {
                    path: appended_path,
                    message: "resource finalization transition digest collision".into(),
                });
            }
            atomic_json(&appended_path, &value)?;
            return Ok(serde_json::json!({
                "workItemId": work_item_id,
                "state": "appended",
                "sequence": transition.sequence,
                "disposition": receipt.result.disposition,
                "predecessorPath": repository_relative_path(&root, &head_path),
                "path": repository_relative_path(&root, &appended_path)
            }));
        }
        validate_resource_finalization_replay(&existing, &receipt).map_err(|error| {
            ObserverError::State {
                path: head_path.to_owned(),
                message: error.to_string(),
            }
        })?;
        return Ok(serde_json::json!({
            "workItemId": work_item_id,
            "state": "idempotent",
            "disposition": existing.result.disposition,
            "path": repository_relative_path(&root, &head_path)
        }));
    }
    if transition.is_some() {
        return Err(ObserverError::State {
            path: receipt_path.into(),
            message: "an append-only finalization transition requires an existing immutable predecessor receipt".into(),
        });
    }
    if receipt.is_post_merge_sync()
        && receipt.result.disposition != ResourceFinalizationDisposition::Retained
    {
        return Err(ObserverError::State {
            path: receipt_path.into(),
            message: "a schema-v2 post-merge sync root must retain the verified clean resources; Deleted requires an append-only transition from that root".into(),
        });
    }
    fs::create_dir_all(decision_path.parent().unwrap_or(root.as_path())).map_err(|source| {
        ObserverError::Read {
            path: decision_path.clone(),
            source,
        }
    })?;
    let value = serde_json::to_value(&receipt).map_err(|error| ObserverError::State {
        path: decision_path.clone(),
        message: error.to_string(),
    })?;
    atomic_json(&decision_path, &value)?;
    Ok(serde_json::json!({
        "workItemId": work_item_id,
        "state": "recorded",
        "disposition": receipt.result.disposition,
        "path": repository_relative_path(&root, &decision_path)
    }))
}

/// A close receipt is immutable, but an older Runtime could record close
/// while the provider-side finalization receipt was still retained.  Permit
/// exactly one append-only cleanup transition for that legacy case: the
/// close must bind the current finalization head, and the new transition must
/// be the next sequence with a fully deleted result.  New closes are blocked
/// before this path by `require_resource_finalization_for_close`.
fn validate_post_close_finalization_reconciliation(
    root: &Path,
    work_item_id: &str,
    close_path: &Path,
    previous: &ResourceFinalizationReceipt,
    previous_path: &Path,
    previous_sequence: u64,
    transition: &ResourceFinalizationTransitionReceipt,
) -> Result<(), ObserverError> {
    let close_metadata =
        fs::symlink_metadata(close_path).map_err(|source| ObserverError::Read {
            path: close_path.into(),
            source,
        })?;
    if !close_metadata.is_file() || close_metadata.file_type().is_symlink() {
        return Err(ObserverError::State {
            path: close_path.into(),
            message: "post-close finalization reconciliation requires a regular close receipt"
                .into(),
        });
    }
    let close = read_json(close_path)?;
    if close["state"] != serde_json::json!("closed")
        || close["workItemId"] != serde_json::json!(work_item_id)
        || close["repositoryId"] != serde_json::json!(repository_id(root).to_string())
        || close["decisionState"] != serde_json::json!("confirmed")
        || close["humanDecision"] != serde_json::json!("approved")
        || close["resourceFinalizationSequence"] != serde_json::json!(previous_sequence)
        || close["resourceFinalizationHeadPath"]
            != serde_json::json!(repository_relative_path(root, previous_path))
    {
        return Err(ObserverError::State {
            path: close_path.into(),
            message: "post-close finalization reconciliation is not bound to the closed head"
                .into(),
        });
    }
    let previous_value = serde_json::to_value(previous).map_err(|error| ObserverError::State {
        path: previous_path.into(),
        message: error.to_string(),
    })?;
    let previous_digest =
        cockpit_protocol::digest_json(&previous_value).map_err(|error| ObserverError::State {
            path: previous_path.into(),
            message: error.to_string(),
        })?;
    if transition.receipt.runtime_version != previous.runtime_version
        || transition.receipt.runtime_digest != previous.runtime_digest
    {
        return Err(ObserverError::State {
            path: close_path.into(),
            message:
                "post-close reconciliation Runtime identity must match the historical predecessor"
                    .into(),
        });
    }
    if close["resourceFinalizationHeadDigest"] != serde_json::json!(previous_digest.to_string())
        || transition.sequence != previous_sequence + 1
        || !matches!(
            transition.receipt.result.disposition,
            ResourceFinalizationDisposition::Deleted
        )
    {
        return Err(ObserverError::State {
            path: close_path.into(),
            message: "post-close reconciliation must append the next deleted finalization head"
                .into(),
        });
    }
    Ok(())
}

fn local_resources_deleted(
    root: &Path,
    receipt: &ResourceFinalizationReceipt,
) -> Result<bool, ObserverError> {
    let branches = git_text(root, &["branch", "--format=%(refname:short)"]).ok_or_else(|| {
        ObserverError::State {
            path: root.to_path_buf(),
            message: "cannot determine local branch state".into(),
        }
    })?;
    if branches
        .lines()
        .any(|branch| branch.trim() == receipt.branch.name)
    {
        return Ok(false);
    }
    let worktrees = git_text(root, &["worktree", "list", "--porcelain"]).ok_or_else(|| {
        ObserverError::State {
            path: root.to_path_buf(),
            message: "cannot determine local worktree state".into(),
        }
    })?;
    // Git reports the canonical worktree path on platforms such as macOS,
    // while a provider receipt may retain the path spelling captured before
    // canonicalization (for example, /var versus /private/var).  Compare
    // existing paths by filesystem identity so a live worktree cannot be
    // mistaken for a removed one merely because its spelling differs.  A
    // missing receipt path is intentionally compared literally: it cannot
    // identify an existing worktree through canonicalization.
    let receipt_worktree = Path::new(&receipt.worktree.path)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&receipt.worktree.path));
    if worktrees.lines().any(|line| {
        line.strip_prefix("worktree ").is_some_and(|path| {
            let actual_worktree = Path::new(path);
            actual_worktree
                .canonicalize()
                .map(|canonical| canonical == receipt_worktree)
                .unwrap_or_else(|_| actual_worktree == receipt_worktree)
        })
    }) {
        return Ok(false);
    }
    if receipt.is_post_merge_sync() {
        let sync = receipt
            .post_merge_sync
            .as_ref()
            .ok_or_else(|| ObserverError::State {
                path: root.to_path_buf(),
                message: "schema-v2 finalization is missing postMergeSync evidence".into(),
            })?;
        let branch_ref = format!("refs/heads/{}", receipt.branch.name);
        let local_refs = git_text(root, &["for-each-ref", "--format=%(refname)", "refs/heads"])
            .ok_or_else(|| ObserverError::State {
                path: root.to_path_buf(),
                message: "cannot prove local branch state for schema-v2 finalization".into(),
            })?;
        if local_refs.lines().any(|reference| reference == branch_ref) {
            return Ok(false);
        }
        let target_path = Path::new(&receipt.worktree.path);
        match fs::symlink_metadata(target_path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Ok(_) => return Ok(false),
            Err(source) => {
                return Err(ObserverError::Read {
                    path: target_path.into(),
                    source,
                });
            }
        }
        let expected_git_dir = Path::new(&sync.worktree_git_dir);
        match fs::symlink_metadata(expected_git_dir) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Ok(_) => return Ok(false),
            Err(source) => {
                return Err(ObserverError::Read {
                    path: expected_git_dir.into(),
                    source,
                });
            }
        }
        let records = git_worktree_records(root)?;
        if records.iter().any(|record| {
            record
                .path
                .as_deref()
                .is_some_and(|path| worktree_paths_identical(path, target_path))
                || record.path.as_deref().is_some_and(|path| {
                    worktree_git_dir_reference(path)
                        .is_some_and(|git_dir| git_dir_paths_identical(&git_dir, expected_git_dir))
                })
        }) {
            return Ok(false);
        }
        if git_common_dir(root)? != canonical_expected_path(&sync.git_common_dir)? {
            return Ok(false);
        }
        if canonical_path(root)? == canonical_expected_path(&receipt.worktree.path)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn canonical_path(path: &Path) -> Result<PathBuf, ObserverError> {
    fs::canonicalize(path).map_err(|source| ObserverError::Read {
        path: path.to_path_buf(),
        source,
    })
}

fn canonical_expected_path(path: &str) -> Result<PathBuf, ObserverError> {
    let path = Path::new(path);
    path.canonicalize()
        .or_else(|_| {
            if path.is_absolute() {
                Ok(path.to_path_buf())
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "expected path must be absolute",
                ))
            }
        })
        .map_err(|source| ObserverError::Read {
            path: path.to_path_buf(),
            source,
        })
}

fn worktree_paths_identical(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

fn git_dir_paths_identical(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

fn worktree_git_dir_reference(worktree: &Path) -> Option<PathBuf> {
    let git_file = worktree.join(".git");
    let metadata = fs::symlink_metadata(&git_file).ok()?;
    let value = if metadata.file_type().is_symlink() {
        fs::read_link(&git_file).ok()?
    } else if metadata.is_file() {
        let contents = fs::read_to_string(&git_file).ok()?;
        let target = contents.trim().strip_prefix("gitdir: ")?;
        PathBuf::from(target)
    } else {
        return None;
    };
    if value.is_absolute() {
        Some(value)
    } else {
        Some(worktree.join(value))
    }
}

fn git_common_dir(root: &Path) -> Result<PathBuf, ObserverError> {
    let reported = git_text(
        root,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )
    .ok_or_else(|| ObserverError::State {
        path: root.to_path_buf(),
        message: "cannot determine Git common directory".into(),
    })?;
    canonical_path(Path::new(&reported))
}

fn post_merge_sync_error(root: &Path, message: impl Into<String>) -> ObserverError {
    ObserverError::State {
        path: root.join(".ai/decisions"),
        message: message.into(),
    }
}

fn commit_tree_and_parents(
    root: &Path,
    revision: &str,
) -> Result<(String, Vec<String>), ObserverError> {
    let output = git_text(root, &["show", "-s", "--format=%T %P", revision]).ok_or_else(|| {
        post_merge_sync_error(
            root,
            format!("post-merge sync commit is unavailable: {revision}"),
        )
    })?;
    let mut fields = output.split_whitespace();
    let tree = fields.next().unwrap_or_default().to_owned();
    let parents = fields.map(str::to_owned).collect::<Vec<_>>();
    if !valid_git_object_id(&tree) || parents.iter().any(|parent| !valid_git_object_id(parent)) {
        return Err(post_merge_sync_error(
            root,
            format!("post-merge sync commit metadata is invalid: {revision}"),
        ));
    }
    Ok((tree, parents))
}

fn is_ancestor(root: &Path, ancestor: &str, descendant: &str) -> Result<bool, ObserverError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["merge-base", "--is-ancestor", ancestor, descendant])
        .output()
        .map_err(|source| ObserverError::Read {
            path: root.to_path_buf(),
            source,
        })?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(post_merge_sync_error(
            root,
            format!("cannot determine commit ancestry: {ancestor} -> {descendant}"),
        )),
    }
}

fn require_exact_parent_chain(
    root: &Path,
    revision: &str,
    expected_parents: &[&str],
) -> Result<(), ObserverError> {
    let (_, actual_parents) = commit_tree_and_parents(root, revision)?;
    if actual_parents
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        != expected_parents
    {
        return Err(post_merge_sync_error(
            root,
            format!("unexpected parent list for post-merge sync commit {revision}"),
        ));
    }
    Ok(())
}

fn git_blob_bytes(root: &Path, revision: &str, path: &str) -> Result<Vec<u8>, ObserverError> {
    let spec = format!("{revision}:{path}");
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["show", &spec])
        .output()
        .map_err(|source| ObserverError::Read {
            path: root.to_path_buf(),
            source,
        })?;
    if !output.status.success() {
        return Err(post_merge_sync_error(
            root,
            format!(
                "required committed blob could not be read: {spec}; Git status={}; stderr={}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    Ok(output.stdout)
}

fn git_json(root: &Path, revision: &str, path: &str) -> Result<serde_json::Value, ObserverError> {
    let bytes = git_blob_bytes(root, revision, path)?;
    super::reject_duplicate_json_keys(&bytes).map_err(|error| {
        post_merge_sync_error(
            root,
            format!("committed JSON has duplicate keys at {path}: {error}"),
        )
    })?;
    serde_json::from_slice(&bytes).map_err(|error| {
        post_merge_sync_error(
            root,
            format!("committed JSON is invalid at {path}: {error}"),
        )
    })
}

struct HistoricalGitCheckout {
    directory: PathBuf,
    repository: PathBuf,
}

impl Drop for HistoricalGitCheckout {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn historical_clone_location(source: &Path) -> Result<(PathBuf, PathBuf), ObserverError> {
    let cwd = source.to_path_buf();
    // Git source discovery/upload-pack cannot bootstrap the long canonical
    // directory even with core.longpaths. Use an existing equivalent short
    // name only at this command boundary; consumers retain the canonical root.
    #[cfg(windows)]
    let cwd = if cwd.as_os_str().len() > 200 {
        existing_windows_short_clone_source(source)?
    } else {
        cwd
    };
    let argument = PathBuf::from(".");
    let resolved = fs::canonicalize(cwd.join(&argument)).map_err(|source| ObserverError::Read {
        path: cwd.join(&argument),
        source,
    })?;
    if resolved != source {
        return Err(post_merge_sync_error(
            source,
            "local clone source identity changed",
        ));
    }
    Ok((cwd, argument))
}

#[cfg(windows)]
fn existing_windows_short_clone_source(source: &Path) -> Result<PathBuf, ObserverError> {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use windows_sys::Win32::Storage::FileSystem::GetShortPathNameW;
    let input: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    // SAFETY: input is NUL-terminated; the sizing call has no output buffer.
    let required = unsafe { GetShortPathNameW(input.as_ptr(), std::ptr::null_mut(), 0) };
    if required == 0 {
        return Err(ObserverError::Read {
            path: source.to_path_buf(),
            source: std::io::Error::last_os_error(),
        });
    }
    if required > 32768 {
        return Err(post_merge_sync_error(
            source,
            "invalid Windows short-name buffer size",
        ));
    }
    let mut output = vec![0u16; required as usize];
    // SAFETY: output is writable for exactly the advertised buffer capacity.
    let written = unsafe { GetShortPathNameW(input.as_ptr(), output.as_mut_ptr(), required) };
    if written == 0 {
        return Err(ObserverError::Read {
            path: source.to_path_buf(),
            source: std::io::Error::last_os_error(),
        });
    }
    if written >= required {
        return Err(post_merge_sync_error(
            source,
            "Windows short-name source changed during lookup",
        ));
    }
    let alias = PathBuf::from(std::ffi::OsString::from_wide(&output[..written as usize]));
    if alias.as_os_str().len() > 200 {
        return Err(post_merge_sync_error(
            source,
            "no existing short source name fits the Windows Git bootstrap limit",
        ));
    }
    // No alias is created and no volume/global setting is changed. The caller
    // must prove canonicalize(alias/.) == the original source before spawning.
    Ok(alias)
}

fn materialize_git_revision(
    root: &Path,
    revision: &str,
) -> Result<HistoricalGitCheckout, ObserverError> {
    let source = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.to_path_buf(),
        source,
    })?;
    #[cfg(windows)]
    let use_bundle = source.as_os_str().len() > 200;
    #[cfg(not(windows))]
    let use_bundle = false;
    let (clone_cwd, clone_source) = if use_bundle {
        (source.to_path_buf(), PathBuf::new())
    } else {
        historical_clone_location(&source)?
    };
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let directory = std::env::temp_dir().join(format!(
        "cockpit-finalization-history-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&directory).map_err(|source| ObserverError::Read {
        path: directory.clone(),
        source,
    })?;
    let mut checkout = HistoricalGitCheckout {
        repository: directory.join("repository"),
        directory,
    };
    #[cfg(windows)]
    let (clone_cwd, clone_source) = if use_bundle {
        prepare_windows_source_bundle(&source, revision, &checkout.directory)?
    } else {
        (clone_cwd, clone_source)
    };
    // Canonical containment identity remains unchanged; only the child cwd and
    // proven equivalent relative source argument are selected for Git.
    let mut clone_command = Command::new("git");
    clone_command
        .current_dir(&clone_cwd)
        .args(["-c", "core.longpaths=true", "clone", "--quiet"]);
    if !use_bundle {
        clone_command.args(["--shared", "--local"]);
    }
    let clone = clone_command
        .args([
            "--no-checkout",
            "--config",
            "core.autocrlf=false",
            "--config",
            "core.longpaths=true",
        ])
        .arg("--")
        .arg(&clone_source)
        .arg(&checkout.repository)
        .output()
        .map_err(|source| ObserverError::Read {
            path: checkout.repository.clone(),
            source,
        })?;
    if !clone.status.success() {
        return Err(post_merge_sync_error(
            root,
            format!(
                "cannot create an isolated historical checkout: {}",
                String::from_utf8_lossy(&clone.stderr).trim()
            ),
        ));
    }
    let detached = Command::new("git")
        .arg("-C")
        .arg(&checkout.repository)
        .args(["checkout", "--quiet", "--detach"])
        .arg(revision)
        .output()
        .map_err(|source| ObserverError::Read {
            path: checkout.repository.clone(),
            source,
        })?;
    if !detached.status.success() {
        return Err(post_merge_sync_error(
            root,
            format!(
                "cannot materialize archive-stage revision {revision}: {}",
                String::from_utf8_lossy(&detached.stderr).trim()
            ),
        ));
    }
    // Consumers bind descendant paths to a canonical root. In particular,
    // Windows expands the ambient temp directory's 8.3 name and verbatim
    // prefix during canonicalization; mixing it with the original Summary
    // path would correctly fail the strict material reader's containment check.
    checkout.repository =
        fs::canonicalize(&checkout.repository).map_err(|source| ObserverError::Read {
            path: checkout.repository.clone(),
            source,
        })?;
    #[cfg(windows)]
    if use_bundle {
        verify_windows_bundle_checkout(&source, revision, &checkout.repository)?;
    }
    Ok(checkout)
}

#[cfg(windows)]
fn windows_dos_metadata_locator(source: &Path) -> Result<PathBuf, ObserverError> {
    use std::os::windows::{
        ffi::{OsStrExt, OsStringExt},
        fs::MetadataExt,
    };
    for entry in [
        source.join(".git"),
        source.join(".git/HEAD"),
        source.join(".git/objects"),
    ] {
        let metadata = fs::symlink_metadata(&entry).map_err(|source| ObserverError::Read {
            path: entry.clone(),
            source,
        })?;
        if metadata.file_type().is_symlink() || metadata.file_attributes() & 0x400 != 0 {
            return Err(post_merge_sync_error(
                source,
                "Windows bundle source metadata is a reparse point",
            ));
        }
    }
    if !source.join(".git").is_dir() {
        return Err(post_merge_sync_error(
            source,
            "Windows long-source bundle requires an ordinary Git directory",
        ));
    }
    let alias = existing_windows_short_clone_source(source)?.join(".git");
    let mut units: Vec<u16> = alias.as_os_str().encode_wide().collect();
    if units.starts_with(&[92, 92, 63, 92]) && units.get(5) == Some(&58) {
        units.drain(..4);
    }
    for unit in &mut units {
        if *unit == 92 {
            *unit = 47;
        }
    }
    let locator = PathBuf::from(std::ffi::OsString::from_wide(&units));
    let canonical = fs::canonicalize(&locator).map_err(|source| ObserverError::Read {
        path: locator.clone(),
        source,
    })?;
    if canonical != source.join(".git") {
        return Err(post_merge_sync_error(
            source,
            "Windows bundle metadata source identity changed",
        ));
    }
    Ok(locator)
}

#[cfg(windows)]
fn windows_metadata_output(
    source: &Path,
    args: &[&std::ffi::OsStr],
) -> Result<Vec<u8>, ObserverError> {
    let cwd = source
        .ancestors()
        .find(|path| path.as_os_str().len() <= 200)
        .ok_or_else(|| post_merge_sync_error(source, "no short physical metadata cwd"))?;
    let locator = windows_dos_metadata_locator(source)?;
    let output = Command::new("git")
        .current_dir(cwd)
        .args(["-c", "core.longpaths=true", "--git-dir"])
        .arg(locator)
        .args(args)
        .output()
        .map_err(|source| ObserverError::Read {
            path: cwd.to_path_buf(),
            source,
        })?;
    if !output.status.success() {
        return Err(post_merge_sync_error(
            source,
            format!(
                "Windows metadata transport failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    Ok(output.stdout)
}

#[cfg(windows)]
fn prepare_windows_source_bundle(
    source: &Path,
    revision: &str,
    directory: &Path,
) -> Result<(PathBuf, PathBuf), ObserverError> {
    use std::ffi::OsStr;
    let bundle = directory.join("source.bundle");
    windows_metadata_output(
        source,
        &[
            OsStr::new("bundle"),
            OsStr::new("create"),
            bundle.as_os_str(),
            OsStr::new("--all"),
            OsStr::new(revision),
        ],
    )?;
    let cwd = source
        .ancestors()
        .find(|path| path.as_os_str().len() <= 200)
        .ok_or_else(|| post_merge_sync_error(source, "no short physical bundle cwd"))?;
    Ok((cwd.to_path_buf(), bundle))
}

#[cfg(windows)]
fn verify_windows_bundle_checkout(
    source: &Path,
    revision: &str,
    checkout: &Path,
) -> Result<(), ObserverError> {
    use std::ffi::OsStr;
    let requested_tree = format!("{revision}^{{tree}}");
    let expected_tree = windows_metadata_output(
        source,
        &[
            OsStr::new("rev-parse"),
            OsStr::new("--verify"),
            OsStr::new(&requested_tree),
        ],
    )?;
    let output = Command::new("git")
        .arg("-C")
        .arg(checkout)
        .args(["rev-parse", "HEAD", "HEAD^{tree}"])
        .output()
        .map_err(|source| ObserverError::Read {
            path: checkout.to_path_buf(),
            source,
        })?;
    let actual = String::from_utf8_lossy(&output.stdout);
    let mut lines = actual.lines();
    if !output.status.success()
        || lines.next() != Some(revision)
        || lines.next() != Some(String::from_utf8_lossy(&expected_tree).trim())
    {
        return Err(post_merge_sync_error(
            source,
            "Windows bundle checkout HEAD/tree identity mismatch",
        ));
    }
    Ok(())
}

struct FinishAttemptBindings {
    repository_id: String,
    contract_digest: Digest,
    scope_digest: Digest,
    snapshot_digest: Digest,
    runtime_version: String,
    runtime_digest: Digest,
}

fn finish_attempt_bindings(
    root: &Path,
    work_item_id: &str,
    source_head: &str,
    finish_revision: &str,
) -> Result<FinishAttemptBindings, ObserverError> {
    let contract_path = format!(".ai/work-items/active/{work_item_id}.contract.json");
    let contract = git_json(root, source_head, &contract_path)?;
    if contract["workItemId"] != serde_json::json!(work_item_id) {
        return Err(post_merge_sync_error(
            root,
            "source PR Contract does not identify the finish-stage Work Item",
        ));
    }
    let contract_digest = super::canonical_contract_digest(&contract).map_err(|error| {
        post_merge_sync_error(
            root,
            format!("source PR Contract digest is invalid: {error}"),
        )
    })?;
    let scope_digest = cockpit_protocol::digest_json(&serde_json::json!({
        "scope": contract.get("scope").cloned().unwrap_or_else(|| serde_json::json!([])),
    }))
    .map_err(|error| {
        post_merge_sync_error(
            root,
            format!("source PR Contract scope digest is invalid: {error}"),
        )
    })?;
    let verification = git_json(
        root,
        finish_revision,
        &format!(".ai/evidence/{work_item_id}.verification.json"),
    )?;
    let runtime_digest: Digest = serde_json::from_value(verification["runtimeDigest"].clone())
        .map_err(|error| {
            post_merge_sync_error(
                root,
                format!("finish-stage Runtime digest is invalid: {error}"),
            )
        })?;
    let snapshot_digest: Digest = serde_json::from_value(
        verification["repositorySnapshotDigest"].clone(),
    )
    .map_err(|error| {
        post_merge_sync_error(
            root,
            format!("finish-stage repository snapshot digest is invalid: {error}"),
        )
    })?;
    let runtime_version = verification["runtimeVersion"]
        .as_str()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| post_merge_sync_error(root, "finish-stage Runtime version is missing"))?
        .to_owned();
    if verification["passed"] != serde_json::json!(true)
        || verification["repositoryId"] != serde_json::json!(super::repository_id(root).to_string())
        || verification["contractDigest"] != serde_json::json!(contract_digest.to_string())
        || verification["receipt"]["passed"] != serde_json::json!(true)
        || verification["receipt"]["runtimeVersion"] != serde_json::json!(runtime_version)
        || verification["receipt"]["runtimeDigest"] != serde_json::json!(runtime_digest.to_string())
    {
        return Err(post_merge_sync_error(
            root,
            "finish-stage verification does not bind the source Contract and Runtime identity",
        ));
    }
    Ok(FinishAttemptBindings {
        repository_id: super::repository_id(root).to_string(),
        contract_digest,
        scope_digest,
        snapshot_digest,
        runtime_version,
        runtime_digest,
    })
}

fn finish_attempt_matches(
    attempt: &serde_json::Value,
    filename_digest: &str,
    work_item_id: &str,
    bindings: &FinishAttemptBindings,
) -> bool {
    let Some(attempt_id) = attempt
        .get("attemptId")
        .and_then(serde_json::Value::as_str)
        .and_then(|value| value.parse::<Digest>().ok())
    else {
        return false;
    };
    let mut unsigned = attempt.clone();
    let Some(unsigned_object) = unsigned.as_object_mut() else {
        return false;
    };
    unsigned_object.remove("attemptId");
    let Ok(content_digest) = cockpit_protocol::digest_json(&unsigned) else {
        return false;
    };
    let state = attempt["state"].as_str().unwrap_or_default();
    let passed = attempt["passed"].as_bool();
    let process_count = attempt["processesSpawned"].as_u64();
    let records = attempt
        .get("executionRecords")
        .and_then(serde_json::Value::as_array);
    if filename_digest
        != attempt_id
            .to_string()
            .strip_prefix("sha256:")
            .unwrap_or_default()
        || content_digest != attempt_id
        || attempt["protocolVersion"] != serde_json::json!(1)
        || attempt["schemaVersion"] != serde_json::json!(1)
        || attempt["kind"] != serde_json::json!("verification_attempt")
        || attempt["workItemId"] != serde_json::json!(work_item_id)
        || attempt["repositoryId"] != serde_json::json!(bindings.repository_id)
        || attempt["contractDigest"] != serde_json::json!(bindings.contract_digest.to_string())
        || attempt["contractExecutionScopeDigest"]
            != serde_json::json!(bindings.scope_digest.to_string())
        || attempt["repositorySnapshotDigest"]
            != serde_json::json!(bindings.snapshot_digest.to_string())
        || attempt["runtimeVersion"] != serde_json::json!(bindings.runtime_version)
        || attempt["runtimeDigest"] != serde_json::json!(bindings.runtime_digest.to_string())
        || !attempt["createdAt"]
            .as_str()
            .is_some_and(|value| chrono::DateTime::parse_from_rfc3339(value).is_ok())
        || passed.is_none()
        || process_count.is_none()
        || records.is_none()
    {
        return false;
    }

    let Some(commands) = finish_attempt_command_index(attempt, bindings) else {
        return false;
    };

    match state {
        "precondition_rejected" => {
            passed == Some(false)
                && process_count == Some(0)
                && records.is_some_and(Vec::is_empty)
                && attempt["receipt"].is_null()
        }
        "execution_failed" | "execution_completed" | "formal_receipt_rejected" => {
            let Some(receipt_value) = attempt.get("receipt").filter(|value| !value.is_null())
            else {
                return false;
            };
            let Ok(receipt) = serde_json::from_value::<cockpit_verification::VerificationReceipt>(
                receipt_value.clone(),
            ) else {
                return false;
            };
            let expected_passed = state != "execution_failed";
            let receipt_records =
                serde_json::to_value(&receipt.execution_records).unwrap_or_default();
            let state_consistent = match state {
                "execution_failed" => passed == Some(false),
                "execution_completed" => passed == Some(true),
                "formal_receipt_rejected" => {
                    passed == Some(true)
                        && attempt["diagnostic"]["code"]
                            .as_str()
                            .is_some_and(|value| !value.is_empty())
                        && attempt["diagnostic"]["message"]
                            .as_str()
                            .is_some_and(|value| !value.is_empty())
                }
                _ => false,
            };
            receipt.work_item_id.as_deref() == Some(work_item_id)
                && receipt.repository_id.as_deref() == Some(bindings.repository_id.as_str())
                && receipt.runtime_version.as_deref() == Some(bindings.runtime_version.as_str())
                && receipt.runtime_digest.as_deref()
                    == Some(bindings.runtime_digest.to_string().as_str())
                && receipt.passed == expected_passed
                && attempt["executionRecords"] == receipt_records
                && attempt["processesSpawned"].as_u64() == Some(receipt.processes_spawned as u64)
                && state_consistent
                && finish_attempt_receipt_matches_commands(
                    &receipt,
                    &commands,
                    process_count.unwrap_or_default(),
                )
        }
        _ => false,
    }
}

fn finish_attempt_command_index(
    attempt: &serde_json::Value,
    bindings: &FinishAttemptBindings,
) -> Option<BTreeMap<String, (String, u64)>> {
    let commands = attempt.get("commands")?.as_array()?;
    let mut index = BTreeMap::new();
    for command in commands {
        let node_id = command
            .get("nodeId")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.trim().is_empty())?;
        let command_digest = command
            .get("commandDigest")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| value.parse::<Digest>().ok())?
            .to_string();
        let runtime_digest = command
            .get("runtimeDigest")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| value.parse::<Digest>().ok())?;
        let timeout_seconds = command
            .get("timeoutSeconds")
            .and_then(serde_json::Value::as_u64)
            .filter(|value| *value > 0)?;
        if command
            .get("program")
            .and_then(serde_json::Value::as_str)
            .is_none_or(|value| value.trim().is_empty())
            || !command
                .get("args")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|values| values.iter().all(serde_json::Value::is_string))
            || !command
                .get("scope")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|values| values.iter().all(serde_json::Value::is_string))
            || command
                .get("stage")
                .and_then(serde_json::Value::as_str)
                .is_none_or(|value| value.trim().is_empty())
            || command
                .get("runner")
                .and_then(serde_json::Value::as_str)
                .is_none_or(|value| value.trim().is_empty())
            || command.get("baseCommit").is_none_or(|value| {
                !value.is_null() && !value.as_str().is_some_and(valid_git_object_id)
            })
            || runtime_digest != bindings.runtime_digest
            || index
                .insert(node_id.to_owned(), (command_digest, timeout_seconds))
                .is_some()
        {
            return None;
        }
    }
    Some(index)
}

fn finish_attempt_receipt_matches_commands(
    receipt: &cockpit_verification::VerificationReceipt,
    commands: &BTreeMap<String, (String, u64)>,
    attempt_processes_spawned: u64,
) -> bool {
    if commands.is_empty()
        || receipt.nodes_planned != commands.len()
        || receipt.results.len() != commands.len()
        || receipt.nodes_executed != receipt.execution_records.len()
    {
        return false;
    }

    let mut results_by_node = BTreeMap::new();
    for result in &receipt.results {
        if !commands.contains_key(&result.node_id)
            || results_by_node
                .insert(result.node_id.as_str(), result)
                .is_some()
        {
            return false;
        }
    }
    if results_by_node.len() != commands.len()
        || receipt.nodes_reused
            != receipt
                .results
                .iter()
                .filter(|result| result.reused)
                .count()
        || receipt.passed != receipt.results.iter().all(|result| result.passed)
    {
        return false;
    }

    let mut record_ids = BTreeSet::new();
    let mut processes_spawned = 0_u64;
    let mut process_spawn_failures = 0_u64;
    for record in &receipt.execution_records {
        let Some((command_digest, timeout_seconds)) = commands.get(&record.node_id) else {
            return false;
        };
        if !record_ids.insert(record.node_id.as_str())
            || record.command_digest != *command_digest
            || record.timeout_seconds != *timeout_seconds
        {
            return false;
        }
        let Some(result) = results_by_node.get(record.node_id.as_str()) else {
            return false;
        };
        if result.reused || result.passed != record.passed || result.timed_out != record.timed_out {
            return false;
        }
        if record.passed && (!record.spawned || record.exit_code != Some(0) || record.timed_out) {
            return false;
        }
        if record.spawned {
            processes_spawned = processes_spawned.saturating_add(1);
        } else {
            process_spawn_failures = process_spawn_failures.saturating_add(1);
        }
    }

    let passed_non_reused_results_have_records = receipt
        .results
        .iter()
        .filter(|result| result.passed && !result.reused)
        .all(|result| record_ids.contains(result.node_id.as_str()));

    passed_non_reused_results_have_records
        && attempt_processes_spawned == processes_spawned
        && receipt.processes_spawned as u64 == processes_spawned
        && receipt.process_spawn_failures as u64 == process_spawn_failures
}

fn validate_finish_attempt_history(
    root: &Path,
    work_item_id: &str,
    source_head: &str,
    finish_revision: &str,
    sync_merge: &str,
) -> Result<(), ObserverError> {
    let range = format!("{source_head}..{finish_revision}");
    let commits =
        git_text(root, &["rev-list", "--first-parent", "--reverse", &range]).ok_or_else(|| {
            post_merge_sync_error(root, "cannot enumerate finish-stage first-parent history")
        })?;
    let commits = commits
        .lines()
        .filter(|commit| !commit.is_empty())
        .collect::<Vec<_>>();
    if commits.last().copied() != Some(finish_revision) {
        return Err(post_merge_sync_error(
            root,
            "finish revision is not on the exact first-parent chain from the source PR head",
        ));
    }
    let bindings = finish_attempt_bindings(root, work_item_id, source_head, finish_revision)?;
    let mut predecessor = source_head;
    for commit in commits.iter().take(commits.len().saturating_sub(1)) {
        require_exact_parent_chain(root, commit, &[predecessor])?;
        let changes = git_text(
            root,
            &[
                "diff-tree",
                "--raw",
                "--no-commit-id",
                "-r",
                "--no-renames",
                predecessor,
                commit,
            ],
        )
        .ok_or_else(|| post_merge_sync_error(root, "cannot inspect pre-finish commit"))?;
        let prefix = format!(".ai/evidence/{work_item_id}.verification-attempt.");
        let (metadata, path) = changes.trim_end().split_once('\t').ok_or_else(|| {
            post_merge_sync_error(root, "pre-finish commit must append one attempt artifact")
        })?;
        let fields = metadata.split_whitespace().collect::<Vec<_>>();
        if changes.lines().count() != 1
            || fields.len() != 5
            || fields[0] != ":000000"
            || fields[1] != "100644"
            || fields[4] != "A"
            || !path.starts_with(&prefix)
            || !path.ends_with(".json")
        {
            return Err(post_merge_sync_error(
                root,
                format!(
                    "pre-finish commit contains a change other than one immutable verification attempt: {commit}"
                ),
            ));
        }
        let suffix = path
            .strip_prefix(&prefix)
            .and_then(|value| value.strip_suffix(".json"))
            .ok_or_else(|| {
                post_merge_sync_error(root, "verification attempt artifact path is malformed")
            })?;
        if suffix.len() != 64 || !suffix.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(post_merge_sync_error(
                root,
                "verification attempt filename digest is malformed",
            ));
        }
        let attempt = git_json(root, commit, path)?;
        if !finish_attempt_matches(&attempt, suffix, work_item_id, &bindings) {
            return Err(post_merge_sync_error(
                root,
                "verification attempt content, Runtime receipt, or Contract/snapshot identity binding is invalid",
            ));
        }
        require_same_git_blob(root, commit, sync_merge, path)?;
        predecessor = commit;
    }
    require_exact_parent_chain(root, finish_revision, &[predecessor])?;
    Ok(())
}

fn validate_archive_stage_bytes(
    root: &Path,
    work_item_id: &str,
    finish_revision: &str,
    archive_revision: &str,
) -> Result<(), ObserverError> {
    let active = format!(".ai/work-items/active/{work_item_id}");
    let archive = format!(".ai/work-items/archive/{work_item_id}");
    let source_report =
        git_blob_bytes(root, finish_revision, &format!("{active}.task-report.json"))?;
    let report_value: serde_json::Value =
        serde_json::from_slice(&source_report).map_err(|error| {
            post_merge_sync_error(
                root,
                format!("finish-stage Task Outcome report is invalid: {error}"),
            )
        })?;
    let cutoff_bound_report = report_value
        .get("usage")
        .is_some_and(|usage| !usage.is_null());
    let mut expected_artifacts = Vec::new();
    for suffix in [
        "contract.json",
        "summary.json",
        "task-report.json",
        "task-report.md",
        "events.jsonl",
    ] {
        let source = git_blob_bytes(root, finish_revision, &format!("{active}.{suffix}"))?;
        let expected =
            if cutoff_bound_report && matches!(suffix, "task-report.json" | "task-report.md") {
                source
            } else {
                super::normalized_archive_artifact_bytes(suffix, &source, work_item_id)?
            };
        expected_artifacts.push((suffix, expected));
    }
    let mut outcome = super::normalized_archive_artifact_bytes(
        "outcome.json",
        &git_blob_bytes(root, finish_revision, &format!("{active}.outcome.json"))?,
        work_item_id,
    )?;
    let task_report_digest = expected_artifacts
        .iter()
        .find(|(suffix, _)| *suffix == "task-report.json")
        .map(|(_, bytes)| Digest::sha256_bytes(bytes).to_string());
    let task_report_markdown_digest = expected_artifacts
        .iter()
        .find(|(suffix, _)| *suffix == "task-report.md")
        .map(|(_, bytes)| Digest::sha256_bytes(bytes).to_string());
    let mut outcome_value: serde_json::Value =
        serde_json::from_slice(&outcome).map_err(|error| {
            post_merge_sync_error(
                root,
                format!("normalized archive Outcome is invalid: {error}"),
            )
        })?;
    if let Some(digest) = task_report_digest {
        outcome_value["taskReportDigest"] = serde_json::json!(digest);
    }
    if let Some(digest) = task_report_markdown_digest {
        outcome_value["taskReportMarkdownDigest"] = serde_json::json!(digest);
    }
    outcome = serde_json::to_vec_pretty(&outcome_value).map_err(|error| {
        post_merge_sync_error(
            root,
            format!("normalized archive Outcome cannot be encoded: {error}"),
        )
    })?;
    expected_artifacts.push(("outcome.json", outcome));
    for (suffix, expected) in expected_artifacts {
        let actual = git_blob_bytes(root, archive_revision, &format!("{archive}.{suffix}"))?;
        if actual != expected {
            return Err(post_merge_sync_error(
                root,
                format!(
                    "archive-stage {suffix} bytes are not the exact normalized finish-stage artifact"
                ),
            ));
        }
    }
    Ok(())
}

fn validate_current_archive_matches_revision(
    root: &Path,
    work_item_id: &str,
    archive_revision: &str,
) -> Result<(), ObserverError> {
    let prefix = format!(".ai/work-items/archive/{work_item_id}");
    for suffix in [
        "contract.json",
        "summary.json",
        "outcome.json",
        "task-report.json",
        "task-report.md",
        "events.jsonl",
        "archive.json",
    ] {
        let relative = format!("{prefix}.{suffix}");
        let path = root.join(&relative);
        let metadata = fs::symlink_metadata(&path).map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(post_merge_sync_error(
                root,
                format!("current archive artifact is not a regular file: {relative}"),
            ));
        }
        let current = fs::read(&path).map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        let committed = git_blob_bytes(root, archive_revision, &relative)?;
        if current != committed {
            return Err(post_merge_sync_error(
                root,
                format!(
                    "current archive artifact differs from the selected immutable archive commit: {relative}"
                ),
            ));
        }
    }
    Ok(())
}

fn archived_contract_digest_at_revision(
    root: &Path,
    work_item_id: &str,
    archive_revision: &str,
) -> Result<Digest, ObserverError> {
    let path = format!(".ai/work-items/archive/{work_item_id}.contract.json");
    let contract = git_json(root, archive_revision, &path)?;
    if contract["workItemId"] != serde_json::json!(work_item_id) {
        return Err(post_merge_sync_error(
            root,
            "selected archive Contract identifies a different Work Item",
        ));
    }
    super::canonical_contract_digest(&contract).map_err(|error| {
        post_merge_sync_error(
            root,
            format!("selected archive Contract is invalid: {error}"),
        )
    })
}

fn validate_documentation_projection(
    root: &Path,
    work_item_id: &str,
    archive_revision: &str,
    docs_revision: &str,
) -> Result<(), ObserverError> {
    let archive_digest =
        archived_contract_digest_at_revision(root, work_item_id, archive_revision)?;
    let short_id = work_item_id
        .split('-')
        .take(2)
        .collect::<Vec<_>>()
        .join("-");
    for suffix in ["", ".zh-CN", ".ja"] {
        let parity_path = format!("docs/reference/reference-parity{suffix}.md");
        let before = String::from_utf8(git_blob_bytes(root, archive_revision, &parity_path)?)
            .map_err(|error| {
                post_merge_sync_error(root, format!("parity source is not UTF-8: {error}"))
            })?;
        let after = String::from_utf8(git_blob_bytes(root, docs_revision, &parity_path)?).map_err(
            |error| post_merge_sync_error(root, format!("parity projection is not UTF-8: {error}")),
        )?;
        let (before_lines, before_row) = parity_row(root, work_item_id, &short_id, &before)?;
        let (after_lines, after_row) = parity_row(root, work_item_id, &short_id, &after)?;
        if before_lines != after_lines {
            return Err(post_merge_sync_error(
                root,
                "parity projection changed rows outside the target Work Item",
            ));
        }
        let old_cells = parity_cells(before_row);
        let new_cells = parity_cells(after_row);
        if old_cells.len() != 3
            || new_cells.len() != 3
            || old_cells[0] != new_cells[0]
            || old_cells[1] != new_cells[1]
            || !pending_status_projection(new_cells[1])
        {
            return Err(post_merge_sync_error(
                root,
                "parity projection must preserve the target Work Item's pending status",
            ));
        }
        let page_suffix = if suffix.is_empty() { "" } else { suffix };
        let source_evidence = old_cells[2];
        let evidence = new_cells[2];
        for required in [
            format!("[Work Item](../work-items/{work_item_id}{page_suffix}.md)"),
            format!(".ai/work-items/archive/{work_item_id}.contract.json"),
            format!(".ai/evidence/{work_item_id}.verification.json"),
            format!(".ai/work-items/archive/{work_item_id}.archive.json"),
            format!(".ai/decisions/{work_item_id}.close.json"),
            archive_digest.to_string(),
        ] {
            if !evidence.contains(&required) {
                return Err(post_merge_sync_error(
                    root,
                    format!("parity evidence cell is missing {required}"),
                ));
            }
        }
        if !evidence.contains(&format!(".ai/decisions/{work_item_id}.finalize")) {
            return Err(post_merge_sync_error(
                root,
                "parity evidence cell is missing the finalization receipt reference",
            ));
        }
        if !source_evidence.contains(&format!(
            ".ai/work-items/active/{work_item_id}.contract.json"
        )) || !source_evidence.contains(&format!(".ai/decisions/{work_item_id}.finalize"))
            || !source_evidence.contains(&format!(".ai/decisions/{work_item_id}.close.json"))
        {
            return Err(post_merge_sync_error(
                root,
                "parity source row does not describe the active-to-archive pending lifecycle",
            ));
        }
        if !pending_archive_evidence(evidence, work_item_id, suffix)
            || evidence.to_ascii_lowercase().contains("terminal")
            || evidence.to_ascii_lowercase().contains("implemented")
        {
            return Err(post_merge_sync_error(
                root,
                "parity evidence must record verification/archive and leave finalization/close pending",
            ));
        }

        let document_path = format!("docs/work-items/{work_item_id}{suffix}.md");
        let before = String::from_utf8(git_blob_bytes(root, archive_revision, &document_path)?)
            .map_err(|error| {
                post_merge_sync_error(root, format!("Work Item document is not UTF-8: {error}"))
            })?;
        let after = String::from_utf8(git_blob_bytes(root, docs_revision, &document_path)?)
            .map_err(|error| {
                post_merge_sync_error(
                    root,
                    format!("projected Work Item document is not UTF-8: {error}"),
                )
            })?;
        validate_work_item_document_projection(
            root,
            work_item_id,
            suffix,
            &archive_digest,
            &before,
            &after,
        )?;
    }
    Ok(())
}

fn pending_status_projection(status: &str) -> bool {
    let status = status.to_lowercase();
    status.contains("in progress") || status.contains("进行中") || status.contains("進行中")
}

fn pending_archive_evidence(evidence: &str, work_item_id: &str, suffix: &str) -> bool {
    let finalization = format!(".ai/decisions/{work_item_id}.finalize");
    let close = format!(".ai/decisions/{work_item_id}.close.json");
    let Some(finalization_at) = evidence.find(&finalization) else {
        return false;
    };
    let Some(close_at) = evidence.find(&close) else {
        return false;
    };
    if finalization_at >= close_at {
        return false;
    }
    let after_close = &evidence[close_at + close.len()..];
    if suffix == ".zh-CN" {
        evidence.contains("已通过")
            && evidence.contains("已记录")
            && after_close.contains("仍待完成")
    } else if suffix == ".ja" {
        evidence.contains("検証済み")
            && evidence.contains("記録済み")
            && after_close.contains("未完了")
    } else {
        let lower = evidence.to_ascii_lowercase();
        (lower.contains("is verified") || lower.contains("is passed"))
            && lower.contains("is recorded")
            && after_close.to_ascii_lowercase().contains("remain pending")
    }
}

fn parity_row<'a>(
    root: &Path,
    work_item_id: &str,
    short_id: &str,
    text: &'a str,
) -> Result<(Vec<&'a str>, &'a str), ObserverError> {
    let lines = text.split_inclusive('\n').collect::<Vec<_>>();
    let matches = |candidate: &str| {
        let cells = candidate
            .trim_end_matches('\n')
            .split('|')
            .collect::<Vec<_>>();
        let first = cells.get(1).map(|value| value.trim()).unwrap_or_default();
        first == work_item_id
            || first.starts_with(&format!("{work_item_id} "))
            || first.starts_with(&format!("{work_item_id}—"))
    };
    let mut target = lines
        .iter()
        .copied()
        .filter(|line| matches(line))
        .collect::<Vec<_>>();
    if target.is_empty() {
        target = lines
            .iter()
            .copied()
            .filter(|line| {
                let first = line
                    .trim_end_matches('\n')
                    .split('|')
                    .nth(1)
                    .unwrap_or_default()
                    .trim();
                first == short_id
                    || first.starts_with(&format!("{short_id} "))
                    || first.starts_with(&format!("{short_id}—"))
            })
            .collect();
    }
    if target.len() != 1 {
        return Err(post_merge_sync_error(
            root,
            "parity projection requires exactly one target row",
        ));
    }
    let row = target[0];
    Ok((lines.into_iter().filter(|line| *line != row).collect(), row))
}

fn parity_cells(row: &str) -> Vec<&str> {
    let raw = row.trim_end_matches('\n').split('|').collect::<Vec<_>>();
    raw.iter()
        .skip(1)
        .take(raw.len().saturating_sub(2))
        .map(|cell| cell.trim())
        .collect()
}

fn validate_work_item_document_projection(
    root: &Path,
    work_item_id: &str,
    suffix: &str,
    archive_digest: &Digest,
    before: &str,
    after: &str,
) -> Result<(), ObserverError> {
    let (before_header, before_body) = markdown_frontmatter(before).ok_or_else(|| {
        post_merge_sync_error(root, "archived Work Item page has invalid frontmatter")
    })?;
    let (after_header, after_body) = markdown_frontmatter(after).ok_or_else(|| {
        post_merge_sync_error(root, "promoted Work Item page has invalid frontmatter")
    })?;
    if unprojected_frontmatter_lines(before_header) != unprojected_frontmatter_lines(after_header) {
        return Err(post_merge_sync_error(
            root,
            "Work Item promotion changed unrelated frontmatter",
        ));
    }
    let has_terminal_field = |header: &str| {
        header.lines().any(|line| {
            line.split_once(": ")
                .is_some_and(|(key, _)| key.starts_with("terminal"))
        })
    };
    let old_digest = unique_frontmatter_field(before_header, "contractDigest");
    let new_digest = unique_frontmatter_field(after_header, "contractDigest");
    if unique_frontmatter_field(before_header, "status") != Some("in_progress")
        || unique_frontmatter_field(after_header, "status") != Some("in_progress")
        || unique_frontmatter_field(before_header, "workItemId") != Some(work_item_id)
        || unique_frontmatter_field(after_header, "workItemId") != Some(work_item_id)
        || old_digest
            .and_then(|value| value.parse::<Digest>().ok())
            .is_none()
        || new_digest != Some(archive_digest.to_string().as_str())
        || has_terminal_field(before_header)
        || has_terminal_field(after_header)
    {
        return Err(post_merge_sync_error(
            root,
            "Work Item document must remain pending and bind the archived Contract digest without terminal fields",
        ));
    }
    let old_digest = old_digest.expect("the Contract digest was validated above");
    let old_target = format!("../../.ai/work-items/active/{work_item_id}.contract.json");
    let new_target = format!("../../.ai/work-items/archive/{work_item_id}.contract.json");
    if before_body.matches(&old_target).count() != 1 {
        return Err(post_merge_sync_error(
            root,
            "archived Work Item page must contain exactly one active Contract link before projection",
        ));
    }
    let mut expected_body = before_body.replace(&old_target, &new_target);
    let (old_label, new_label) = match suffix {
        ".zh-CN" => ("当前 [", "已归档的 ["),
        ".ja" => (" active な [", "アーカイブ済みの ["),
        _ => ("active [", "archived ["),
    };
    if expected_body.matches(old_label).count() != 1 {
        return Err(post_merge_sync_error(
            root,
            "Work Item page Contract link label is not the expected active projection",
        ));
    }
    expected_body = expected_body.replacen(old_label, new_label, 1);
    expected_body = expected_body.replace(old_digest, &archive_digest.to_string());
    if expected_body != after_body {
        return Err(post_merge_sync_error(
            root,
            "Work Item page body changed beyond the Contract path and digest projection",
        ));
    }
    Ok(())
}

fn unique_frontmatter_field<'a>(header: &'a str, key: &str) -> Option<&'a str> {
    let mut matches = header
        .lines()
        .filter_map(|line| line.split_once(": "))
        .filter(|(candidate, _)| *candidate == key)
        .map(|(_, value)| value);
    let value = matches.next()?;
    matches.next().is_none().then_some(value)
}

fn markdown_frontmatter(text: &str) -> Option<(&str, &str)> {
    if !text.starts_with("---\n") {
        return None;
    }
    let end = text[4..].find("\n---\n")? + 4;
    Some((&text[4..end], &text[end..]))
}

fn unprojected_frontmatter_lines(header: &str) -> Vec<String> {
    let projected = ["contractDigest"];
    header
        .lines()
        .filter(|line| {
            !projected.iter().any(|key| {
                line.strip_prefix(key)
                    .is_some_and(|tail| tail.starts_with(':'))
            })
        })
        .map(str::to_owned)
        .collect()
}

fn validate_post_merge_sync_commit_stages(
    root: &Path,
    work_item_id: &str,
    sync: &PostMergeSyncEvidence,
) -> Result<(), ObserverError> {
    let source = &sync.source_pull_request;
    let followup = &sync.sync_pull_request;
    let source_merge = source
        .merge_commit
        .as_deref()
        .ok_or_else(|| post_merge_sync_error(root, "source PR merge commit is missing"))?;
    let sync_merge = followup
        .merge_commit
        .as_deref()
        .ok_or_else(|| post_merge_sync_error(root, "sync PR merge commit is missing"))?;
    require_exact_parent_chain(
        root,
        source_merge,
        &[&source.base_revision, &source.head_revision],
    )?;
    require_exact_parent_chain(root, sync_merge, &[source_merge, &followup.head_revision])?;
    let (source_merge_tree, _) = commit_tree_and_parents(root, source_merge)?;
    let (source_head_tree, _) = commit_tree_and_parents(root, &source.head_revision)?;
    let (sync_merge_tree, _) = commit_tree_and_parents(root, sync_merge)?;
    let (sync_head_tree, _) = commit_tree_and_parents(root, &followup.head_revision)?;
    if source_merge_tree != source_head_tree || sync_merge_tree != sync_head_tree {
        return Err(post_merge_sync_error(
            root,
            "the reviewed merge trees differ from their exact PR head trees",
        ));
    }
    let common = git_text(root, &["merge-base", source_merge, &followup.head_revision])
        .ok_or_else(|| post_merge_sync_error(root, "cannot determine source/sync merge base"))?;
    if common != source.head_revision || is_ancestor(root, source_merge, &followup.head_revision)? {
        return Err(post_merge_sync_error(
            root,
            "source merge must remain separate from the sync PR head, with the source PR head as their merge base",
        ));
    }
    if !is_ancestor(root, &source.head_revision, &sync.finish_revision)?
        || !is_ancestor(root, &sync.finish_revision, &sync.archive_revision)?
        || !is_ancestor(root, &sync.archive_revision, &sync.docs_revision)?
        || !is_ancestor(root, &sync.docs_revision, sync_merge)?
    {
        return Err(post_merge_sync_error(
            root,
            "finish, archive, docs, and sync merge do not form the required ancestry",
        ));
    }
    require_exact_parent_chain(root, &sync.archive_revision, &[&sync.finish_revision])?;
    require_exact_parent_chain(root, &sync.docs_revision, &[&sync.archive_revision])?;
    validate_finish_attempt_history(
        root,
        work_item_id,
        &source.head_revision,
        &sync.finish_revision,
        sync_merge,
    )?;

    let active_prefix = format!(".ai/work-items/active/{work_item_id}");
    let archive_prefix = format!(".ai/work-items/archive/{work_item_id}");
    let (_, finish_parents) = commit_tree_and_parents(root, &sync.finish_revision)?;
    let finish_parent = finish_parents
        .first()
        .ok_or_else(|| post_merge_sync_error(root, "finish stage has no first-parent revision"))?;
    let finish_operation = |path: &str| {
        let spec = format!("{finish_parent}:{path}");
        let operation = if git_text(root, &["rev-parse", "--verify", &spec]).is_some() {
            "M"
        } else {
            "A"
        };
        format!("{operation}:{path}")
    };
    let expected_finish = [
        finish_operation(&format!(".ai/evidence/{work_item_id}.verification.json")),
        finish_operation(&format!("{active_prefix}.events.jsonl")),
        finish_operation(&format!("{active_prefix}.outcome.json")),
        format!("M:{active_prefix}.summary.json"),
        finish_operation(&format!("{active_prefix}.task-report.json")),
        finish_operation(&format!("{active_prefix}.task-report.md")),
    ];
    let mut expected_archive = vec![
        format!("A:{archive_prefix}.archive.json"),
        format!("D:{active_prefix}.events.jsonl"),
        format!("A:{archive_prefix}.events.jsonl"),
    ];
    for suffix in [
        "contract.json",
        "outcome.json",
        "summary.json",
        "task-report.json",
        "task-report.md",
    ] {
        expected_archive.push(format!("D:{active_prefix}.{suffix}"));
        expected_archive.push(format!("A:{archive_prefix}.{suffix}"));
    }
    let expected_docs = [
        "M:docs/reference/reference-parity.ja.md".to_owned(),
        "M:docs/reference/reference-parity.md".to_owned(),
        "M:docs/reference/reference-parity.zh-CN.md".to_owned(),
        format!("M:docs/work-items/{work_item_id}.ja.md"),
        format!("M:docs/work-items/{work_item_id}.md"),
        format!("M:docs/work-items/{work_item_id}.zh-CN.md"),
    ];
    validate_stage_changes(root, &sync.finish_revision, &expected_finish)?;
    validate_stage_changes(root, &sync.archive_revision, &expected_archive)?;
    validate_stage_changes(root, &sync.docs_revision, &expected_docs)?;
    validate_archive_stage_bytes(
        root,
        work_item_id,
        &sync.finish_revision,
        &sync.archive_revision,
    )?;
    validate_current_archive_matches_revision(root, work_item_id, &sync.archive_revision)?;
    validate_documentation_projection(
        root,
        work_item_id,
        &sync.archive_revision,
        &sync.docs_revision,
    )?;

    let verification_path = format!(".ai/evidence/{work_item_id}.verification.json");
    require_same_git_blob(root, &sync.finish_revision, sync_merge, &verification_path)?;
    let verification = git_json(root, &sync.finish_revision, &verification_path)?;
    if verification["passed"] != serde_json::json!(true) {
        return Err(post_merge_sync_error(
            root,
            "the immutable finish-stage verification evidence is not passed",
        ));
    }
    let archived_runtime_digest: Digest = serde_json::from_value(
        verification["receipt"]["runtimeDigest"].clone(),
    )
    .map_err(|error| {
        post_merge_sync_error(
            root,
            format!("finish-stage verification Runtime digest is invalid: {error}"),
        )
    })?;
    let archived_runtime_version = verification["receipt"]["runtimeVersion"]
        .as_str()
        .ok_or_else(|| post_merge_sync_error(root, "finish-stage Runtime version is missing"))?;
    let archive_manifest_path = format!("{archive_prefix}.archive.json");
    require_same_git_blob(
        root,
        &sync.archive_revision,
        sync_merge,
        &archive_manifest_path,
    )?;
    let archive_manifest = git_json(root, &sync.archive_revision, &archive_manifest_path)?;
    let manifest_runtime_digest: Digest =
        serde_json::from_value(archive_manifest["runtimeDigest"].clone()).map_err(|error| {
            post_merge_sync_error(
                root,
                format!("archive manifest Runtime digest is invalid: {error}"),
            )
        })?;
    if manifest_runtime_digest != archived_runtime_digest
        || archive_manifest["runtimeVersion"].as_str() != Some(archived_runtime_version)
    {
        return Err(post_merge_sync_error(
            root,
            "archive manifest does not preserve the finish-stage Runtime identity",
        ));
    }
    let historical_checkout = materialize_git_revision(root, &sync.archive_revision)?;
    super::verify_archive_manifest(
        &historical_checkout.repository,
        work_item_id,
        &archive_manifest,
    )?;
    for name in [
        "contract.json",
        "events.jsonl",
        "outcome.json",
        "summary.json",
        "task-report.json",
        "task-report.md",
    ] {
        let path = format!("{archive_prefix}.{name}");
        require_same_git_blob(root, &sync.archive_revision, sync_merge, &path)?;
    }
    let main_ref = format!(
        "refs/remotes/{}/{}",
        followup.base_remote, followup.base_branch
    );
    let remote_main = git_text(root, &["rev-parse", "--verify", &main_ref]).ok_or_else(|| {
        post_merge_sync_error(
            root,
            format!("cannot resolve synced default branch {main_ref}"),
        )
    })?;
    if !is_ancestor(root, &sync.current_main_revision, &remote_main)? {
        return Err(post_merge_sync_error(
            root,
            "the remote-tracking default branch no longer contains the reviewed sync merge",
        ));
    }
    let common_dir = git_common_dir(root)?;
    if common_dir != canonical_expected_path(&sync.git_common_dir)? {
        return Err(post_merge_sync_error(
            root,
            "post-merge sync evidence is bound to a different Git common directory",
        ));
    }
    Ok(())
}

fn validate_stage_changes(
    root: &Path,
    revision: &str,
    expected: &[String],
) -> Result<(), ObserverError> {
    let (_, parents) = commit_tree_and_parents(root, revision)?;
    if parents.len() != 1 {
        return Err(post_merge_sync_error(
            root,
            format!("stage commit must have exactly one parent: {revision}"),
        ));
    }
    let changes = git_text(
        root,
        &[
            "diff-tree",
            "--raw",
            "--no-commit-id",
            "-r",
            "--no-renames",
            &parents[0],
            revision,
        ],
    )
    .ok_or_else(|| {
        post_merge_sync_error(root, format!("cannot inspect stage commit {revision}"))
    })?;
    let mut actual = Vec::new();
    for line in changes.lines() {
        let (metadata, paths) = line.split_once('\t').ok_or_else(|| {
            post_merge_sync_error(root, format!("malformed Git diff record for {revision}"))
        })?;
        let fields = metadata.split_whitespace().collect::<Vec<_>>();
        if fields.len() != 5 || !fields[0].starts_with(':') {
            return Err(post_merge_sync_error(
                root,
                format!("malformed Git diff metadata for {revision}"),
            ));
        }
        let old_mode = fields[0].trim_start_matches(':');
        let new_mode = fields[1];
        let status = fields[4];
        if !matches!(old_mode, "000000" | "100644") || !matches!(new_mode, "000000" | "100644") {
            return Err(post_merge_sync_error(
                root,
                format!("stage commit changes an unexpected file mode: {revision}"),
            ));
        }
        let operation = match status.as_bytes().first().copied() {
            Some(b'M') if old_mode == "100644" && new_mode == "100644" => {
                format!("M:{paths}")
            }
            Some(b'A') if old_mode == "000000" && new_mode == "100644" => {
                format!("A:{paths}")
            }
            Some(b'D') if old_mode == "100644" && new_mode == "000000" => {
                format!("D:{paths}")
            }
            _ => {
                return Err(post_merge_sync_error(
                    root,
                    format!("stage commit contains an unapproved path operation: {revision}"),
                ));
            }
        };
        actual.push(operation);
    }
    actual.sort();
    let mut expected = expected.to_vec();
    expected.sort();
    if actual != expected {
        return Err(post_merge_sync_error(
            root,
            format!("stage commit changed paths outside its exact approved set: {revision}"),
        ));
    }
    Ok(())
}

fn require_same_git_blob(
    root: &Path,
    left_revision: &str,
    right_revision: &str,
    path: &str,
) -> Result<(), ObserverError> {
    let left_spec = format!("{left_revision}:{path}");
    let right_spec = format!("{right_revision}:{path}");
    let left = git_text(root, &["rev-parse", "--verify", &left_spec]).ok_or_else(|| {
        post_merge_sync_error(
            root,
            format!("required historical blob is missing: {left_spec}"),
        )
    })?;
    let right = git_text(root, &["rev-parse", "--verify", &right_spec]).ok_or_else(|| {
        post_merge_sync_error(
            root,
            format!("required final blob is missing: {right_spec}"),
        )
    })?;
    if left != right {
        return Err(post_merge_sync_error(
            root,
            format!("immutable historical blob changed after its stage: {path}"),
        ));
    }
    Ok(())
}

fn github_repository_from_pull_request_url(url: &str) -> Option<(String, u64)> {
    let path = url.strip_prefix("https://github.com/")?;
    let parts = path.split('/').collect::<Vec<_>>();
    if parts.len() != 4 || parts[2] != "pull" || parts[0].is_empty() || parts[1].is_empty() {
        return None;
    }
    let number = parts[3].parse::<u64>().ok()?;
    (number > 0).then(|| (format!("{}/{}", parts[0], parts[1]), number))
}

fn delegated_raw_json(
    root: &Path,
    evidence: &cockpit_protocol::DelegatedEvidence,
) -> Result<serde_json::Value, ObserverError> {
    let path = super::external_evidence_file(root, &evidence.raw_evidence_ref)?;
    let bytes = fs::read(&path).map_err(|source| ObserverError::Read {
        path: path.clone(),
        source,
    })?;
    if Digest::sha256_bytes(&bytes) != evidence.digest {
        return Err(post_merge_sync_error(
            root,
            "delegated raw provider response digest differs from its imported evidence receipt",
        ));
    }
    super::reject_duplicate_json_keys(&bytes).map_err(|error| {
        post_merge_sync_error(
            root,
            format!("delegated provider response is ambiguous JSON: {error}"),
        )
    })?;
    serde_json::from_slice(&bytes).map_err(|error| {
        post_merge_sync_error(
            root,
            format!("delegated provider response is not JSON: {error}"),
        )
    })
}

fn validate_github_pull_request_observation(
    root: &Path,
    evidence: &cockpit_protocol::DelegatedEvidence,
    identity: &cockpit_protocol::ResourceFinalizationPullRequestIdentity,
    repository: &str,
    expected_head_ref: Option<&str>,
) -> Result<(), ObserverError> {
    let (observed_repository, observed_number) =
        github_repository_from_pull_request_url(&identity.url).ok_or_else(|| {
            post_merge_sync_error(root, "bound pull request URL is not an exact GitHub PR URL")
        })?;
    let expected_origin =
        format!("https://api.github.com/repos/{observed_repository}/pulls/{observed_number}");
    if observed_repository != repository
        || observed_number != identity.number
        || evidence.provider != "github"
        || evidence.subject != identity.url
        || evidence.origin != expected_origin
    {
        return Err(post_merge_sync_error(
            root,
            "delegated pull request evidence origin or identity does not match the bound GitHub PR",
        ));
    }
    let value = delegated_raw_json(root, evidence)?;
    let head_ref = value["head"]["ref"].as_str().unwrap_or_default();
    let merged = value["merged"].as_bool() == Some(true);
    if value["number"].as_u64() != Some(identity.number)
        || value["html_url"].as_str() != Some(identity.url.as_str())
        || value["state"].as_str() != Some("closed")
        || !merged
        || value["merge_commit_sha"].as_str() != identity.merge_commit.as_deref()
        || value["head"]["sha"].as_str() != Some(identity.head_revision.as_str())
        || head_ref.is_empty()
        || expected_head_ref.is_some_and(|expected| expected != head_ref)
        || value["head"]["repo"]["full_name"].as_str() != Some(repository)
        || value["base"]["ref"].as_str() != Some(identity.base_branch.as_str())
        || value["base"]["sha"].as_str() != Some(identity.base_revision.as_str())
        || value["base"]["repo"]["full_name"].as_str() != Some(repository)
    {
        return Err(post_merge_sync_error(
            root,
            "delegated GitHub PR response does not prove the exact merged PR, head, base, and merge commit",
        ));
    }
    Ok(())
}

fn validate_github_remote_ref_observation(
    root: &Path,
    evidence: &cockpit_protocol::DelegatedEvidence,
    sync: &PostMergeSyncEvidence,
    repository: &str,
    retained_root_timestamp: Option<&str>,
    evidence_bound_at: &str,
    finalization_timestamp: &str,
) -> Result<(), ObserverError> {
    let branch = sync
        .remote_branch_ref
        .strip_prefix("refs/heads/")
        .filter(|branch| !branch.is_empty())
        .ok_or_else(|| post_merge_sync_error(root, "remote branch ref is not a branch ref"))?;
    let expected_origin =
        format!("https://api.github.com/repos/{repository}/git/ref/heads/{branch}");
    if evidence.provider != "github"
        || evidence.subject != sync.remote_branch_ref
        || evidence.origin != expected_origin
    {
        return Err(post_merge_sync_error(
            root,
            "delegated remote ref origin or subject does not match the bound GitHub branch",
        ));
    }
    let value = delegated_raw_json(root, evidence)?;
    match sync.remote_branch_state {
        PostMergeSyncRemoteBranchState::Present => {
            if value["ref"].as_str() != Some(sync.remote_branch_ref.as_str())
                || value["object"]["type"].as_str() != Some("commit")
                || value["object"]["sha"].as_str()
                    != Some(sync.sync_pull_request.head_revision.as_str())
            {
                return Err(post_merge_sync_error(
                    root,
                    "delegated GitHub ref response does not prove the exact present branch commit",
                ));
            }
        }
        PostMergeSyncRemoteBranchState::Deleted => {
            let access_url = format!(
                "https://api.github.com/repos/{repository}/git/matching-refs/heads/{branch}"
            );
            let query_url =
                format!("https://api.github.com/repos/{repository}/git/ref/heads/{branch}");
            let Some(observation_id) = value["observationId"]
                .as_str()
                .filter(|value| !value.is_empty())
            else {
                return Err(post_merge_sync_error(
                    root,
                    "deleted ref observation has no stable observation id",
                ));
            };
            let Some(principal) = value["principal"]
                .as_str()
                .filter(|value| !value.is_empty())
            else {
                return Err(post_merge_sync_error(
                    root,
                    "deleted ref observation has no provider principal",
                ));
            };
            let Some(observed_at) = value["observedAt"]
                .as_str()
                .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            else {
                return Err(post_merge_sync_error(
                    root,
                    "deleted ref observation time is invalid",
                ));
            };
            let refs = value["accessCheck"]["refs"].as_array();
            let access = &value["accessCheck"];
            let query = &value["refQuery"];
            let access_identity_matches = access["observationId"] == observation_id
                && access["principal"] == principal
                && access["repository"] == repository
                && access["ref"] == sync.remote_branch_ref
                && access["method"] == "GET"
                && access["url"] == access_url
                && access["httpStatus"] == 200
                && access["nextPageUrl"].is_null()
                && refs.is_some_and(|refs| {
                    refs.iter().all(|item| {
                        item["ref"].as_str().is_some_and(|name| {
                            name.starts_with(&sync.remote_branch_ref)
                                && name != sync.remote_branch_ref
                        })
                    })
                });
            let query_identity_matches = query["observationId"] == observation_id
                && query["principal"] == principal
                && query["repository"] == repository
                && query["ref"] == sync.remote_branch_ref
                && query["method"] == "GET"
                && query["url"] == query_url
                && query["httpStatus"] == 404
                && query["body"]["message"] == "Not Found"
                && query["body"]["status"] == "404"
                && query["body"].get("ref").is_none()
                && query["body"].get("object").is_none();
            let fresh_after_retained_root = retained_root_timestamp
                .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
                .is_some_and(|retained_at| observed_at > retained_at);
            let collected_at = chrono::DateTime::parse_from_rfc3339(&evidence.collected_at).ok();
            let bound_at = chrono::DateTime::parse_from_rfc3339(evidence_bound_at).ok();
            let operation_at = chrono::DateTime::parse_from_rfc3339(finalization_timestamp).ok();
            let timely_for_operation = matches!(
                (collected_at, bound_at, operation_at),
                (Some(collected), Some(bound), Some(operation))
                    if collected == observed_at
                        && observed_at <= bound
                        && bound <= operation
                        && bound - observed_at <= chrono::Duration::minutes(5)
                        && observed_at <= operation
                        && operation - observed_at <= chrono::Duration::minutes(5)
            );
            if value["schemaVersion"] != 1
                || value["repository"] != repository
                || value["ref"] != sync.remote_branch_ref
                || !access_identity_matches
                || !query_identity_matches
                || !fresh_after_retained_root
                || !timely_for_operation
            {
                return Err(post_merge_sync_error(
                    root,
                    "deleted remote branch requires a fresh exact-ref 404 plus a successful accessible ref-list observation after the Retained root",
                ));
            }
        }
        PostMergeSyncRemoteBranchState::Unknown => {
            return Err(post_merge_sync_error(
                root,
                "unknown remote branch observation cannot finalize resources",
            ));
        }
    }
    Ok(())
}

fn validate_post_merge_sync_delegated_evidence(
    root: &Path,
    receipt: &ResourceFinalizationReceipt,
    sync: &PostMergeSyncEvidence,
) -> Result<(), ObserverError> {
    let evidence_receipts = if receipt.schema_version
        == cockpit_protocol::RESOURCE_FINALIZATION_EVIDENCE_OBSERVATION_SCHEMA_VERSION
    {
        let selectors = sync.observation_selectors.as_ref().ok_or_else(|| {
            post_merge_sync_error(
                root,
                "schema-v4 finalization is missing observation selectors",
            )
        })?;
        let observations =
            super::list_delegated_evidence_observations(root, &receipt.work_item_id)?;
        let mut selected = Vec::with_capacity(3);
        for (role, selector) in [
            ("source", &selectors.source),
            ("sync", &selectors.sync),
            ("remote", &selectors.remote),
        ] {
            cockpit_protocol::validate_delegated_evidence_observation_selector(selector).map_err(
                |message| post_merge_sync_error(root, format!("{role} selector: {message}")),
            )?;
            let matches = observations
                .iter()
                .filter(|observation| observation.selector == *selector)
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Err(post_merge_sync_error(
                    root,
                    format!("{role} observation selector must resolve exactly once"),
                ));
            }
            let observation = &matches[0].receipt;
            selected.push(DelegatedEvidenceReceipt {
                schema_version: 1,
                repository_id: observation.repository_id.clone(),
                work_item_id: observation.work_item_id.clone(),
                evidence: observation.evidence.clone(),
                runtime_version: observation.runtime_version.clone(),
                runtime_digest: observation.runtime_digest.clone(),
                bound_at: observation.bound_at.clone(),
            });
        }
        selected
    } else {
        super::list_delegated_evidence(root, &receipt.work_item_id)?
    };
    let timestamp = chrono::DateTime::parse_from_rfc3339(&receipt.timestamp).map_err(|error| {
        post_merge_sync_error(root, format!("finalization timestamp is invalid: {error}"))
    })?;
    let retained_root_timestamp =
        if receipt.result.disposition == ResourceFinalizationDisposition::Deleted {
            let root_receipt = super::read_resource_finalization_receipt(
                &super::resource_finalization_decision_path(root, &receipt.work_item_id),
            )?;
            if root_receipt.work_item_id != receipt.work_item_id
                || root_receipt.repository_id != receipt.repository_id
                || root_receipt.result.disposition != ResourceFinalizationDisposition::Retained
            {
                return Err(post_merge_sync_error(
                    root,
                    "Deleted remote-ref evidence must follow the exact Retained finalization root",
                ));
            }
            Some(root_receipt.timestamp)
        } else {
            None
        };
    let bindings = [
        (
            &sync.source_evidence_digest,
            sync.source_pull_request.url.as_str(),
        ),
        (
            &sync.sync_evidence_digest,
            sync.sync_pull_request.url.as_str(),
        ),
        (
            &sync.remote_evidence_digest,
            sync.remote_branch_ref.as_str(),
        ),
    ];
    for (digest, subject) in bindings {
        let matches = evidence_receipts
            .iter()
            .filter(|candidate| &candidate.evidence.digest == digest)
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err(post_merge_sync_error(
                root,
                format!("delegated evidence digest must resolve exactly once: {digest}"),
            ));
        }
        let evidence = &matches[0].evidence;
        if evidence.provider != receipt.provider
            || evidence.subject != subject
            || evidence.validity != EvidenceValidity::Valid
            || evidence.assurance.rank() < EvidenceAssurance::ProviderVerified.rank()
        {
            return Err(post_merge_sync_error(
                root,
                format!("delegated evidence binding is insufficient for {subject}"),
            ));
        }
        let collected_at =
            chrono::DateTime::parse_from_rfc3339(&evidence.collected_at).map_err(|error| {
                post_merge_sync_error(root, format!("delegated evidence time is invalid: {error}"))
            })?;
        let bound_at =
            chrono::DateTime::parse_from_rfc3339(&matches[0].bound_at).map_err(|error| {
                post_merge_sync_error(
                    root,
                    format!("delegated evidence binding time is invalid: {error}"),
                )
            })?;
        if collected_at > bound_at || bound_at > timestamp {
            return Err(post_merge_sync_error(
                root,
                "delegated evidence collection and binding times must precede the finalization receipt",
            ));
        }
    }
    let source_repository = github_repository_from_pull_request_url(&sync.source_pull_request.url)
        .map(|(repository, _)| repository)
        .ok_or_else(|| post_merge_sync_error(root, "source PR URL is not a GitHub PR URL"))?;
    validate_github_pull_request_observation(
        root,
        &evidence_receipts
            .iter()
            .find(|candidate| candidate.evidence.digest == sync.source_evidence_digest)
            .expect("the source evidence digest was matched above")
            .evidence,
        &sync.source_pull_request,
        &source_repository,
        None,
    )?;
    validate_github_pull_request_observation(
        root,
        &evidence_receipts
            .iter()
            .find(|candidate| candidate.evidence.digest == sync.sync_evidence_digest)
            .expect("the sync evidence digest was matched above")
            .evidence,
        &sync.sync_pull_request,
        &source_repository,
        Some(&receipt.branch.name),
    )?;
    let remote = evidence_receipts
        .iter()
        .find(|candidate| candidate.evidence.digest == sync.remote_evidence_digest)
        .expect("the remote evidence digest was matched above");
    validate_github_remote_ref_observation(
        root,
        &remote.evidence,
        sync,
        &source_repository,
        retained_root_timestamp.as_deref(),
        &remote.bound_at,
        &receipt.timestamp,
    )?;
    Ok(())
}

fn validate_post_merge_sync_local_state(
    root: &Path,
    receipt: &ResourceFinalizationReceipt,
    sync: &PostMergeSyncEvidence,
) -> Result<(), ObserverError> {
    let expected_remote_state = match receipt.result.disposition {
        ResourceFinalizationDisposition::Retained => PostMergeSyncRemoteBranchState::Present,
        ResourceFinalizationDisposition::Deleted => PostMergeSyncRemoteBranchState::Deleted,
        _ => {
            return Err(post_merge_sync_error(
                root,
                "post-merge sync supports only retained roots and deleted transitions",
            ));
        }
    };
    if sync.remote_branch_state != expected_remote_state {
        return Err(post_merge_sync_error(
            root,
            "delegated remote branch state does not match the finalization disposition",
        ));
    }
    if canonical_path(root)? == canonical_expected_path(&receipt.worktree.path)? {
        return Err(post_merge_sync_error(
            root,
            "post-merge sync proof must be inspected from a surviving control checkout",
        ));
    }
    let target = Path::new(&receipt.worktree.path);
    let expected_git_dir_path = Path::new(&sync.worktree_git_dir);
    let expected_git_dir = canonical_expected_path(&sync.worktree_git_dir)?;
    let records = git_worktree_records(root)?;
    let matching = records
        .iter()
        .filter(|record| {
            record
                .path
                .as_deref()
                .is_some_and(|path| worktree_paths_identical(path, target))
        })
        .collect::<Vec<_>>();
    if receipt.result.disposition == ResourceFinalizationDisposition::Retained {
        let git_dir_metadata =
            fs::symlink_metadata(expected_git_dir_path).map_err(|source| ObserverError::Read {
                path: expected_git_dir_path.to_path_buf(),
                source,
            })?;
        if git_dir_metadata.file_type().is_symlink() || !git_dir_metadata.is_dir() {
            return Err(post_merge_sync_error(
                root,
                "retained worktree Git directory is not a real directory",
            ));
        }
        let branch_ref = format!("refs/heads/{}", receipt.branch.name);
        let ref_state = git_text(
            root,
            &[
                "for-each-ref",
                "--format=%(objectname) %(refname)",
                &branch_ref,
            ],
        )
        .ok_or_else(|| post_merge_sync_error(root, "cannot determine retained local branch"))?;
        if ref_state != format!("{} {branch_ref}", receipt.branch.head_revision)
            || matching.len() != 1
            || matching[0].head.as_deref() != Some(receipt.worktree.head_revision.as_str())
            || matching[0].branch_ref.as_deref() != Some(branch_ref.as_str())
        {
            return Err(post_merge_sync_error(
                root,
                "retained branch/worktree does not match the exact local Git registration",
            ));
        }
        let actual_branch = git_text(target, &["branch", "--show-current"])
            .ok_or_else(|| post_merge_sync_error(root, "cannot read retained worktree branch"))?;
        let actual_head = git_text(target, &["rev-parse", "--verify", "HEAD"])
            .ok_or_else(|| post_merge_sync_error(root, "cannot read retained worktree HEAD"))?;
        let actual_status = git_text(target, &["status", "--porcelain", "--untracked-files=all"])
            .ok_or_else(|| {
            post_merge_sync_error(root, "cannot read retained worktree status")
        })?;
        if actual_branch != receipt.branch.name
            || actual_head != receipt.worktree.head_revision
            || !actual_status.is_empty()
        {
            return Err(post_merge_sync_error(
                root,
                "retained worktree branch, HEAD, or clean state differs from the receipt",
            ));
        }
        let target_metadata =
            fs::symlink_metadata(target).map_err(|source| ObserverError::Read {
                path: target.to_path_buf(),
                source,
            })?;
        if target_metadata.file_type().is_symlink() || !target_metadata.is_dir() {
            return Err(post_merge_sync_error(
                root,
                "retained worktree path is not a real directory",
            ));
        }
        let git_file = target.join(".git");
        let git_file_metadata =
            fs::symlink_metadata(&git_file).map_err(|source| ObserverError::Read {
                path: git_file.clone(),
                source,
            })?;
        if git_file_metadata.file_type().is_symlink() || !git_file_metadata.is_file() {
            return Err(post_merge_sync_error(
                root,
                "retained worktree has no regular .git file",
            ));
        }
        let git_file_contents =
            fs::read_to_string(&git_file).map_err(|source| ObserverError::Read {
                path: git_file.clone(),
                source,
            })?;
        let git_dir_line = git_file_contents
            .trim()
            .strip_prefix("gitdir: ")
            .ok_or_else(|| {
                post_merge_sync_error(root, "retained worktree .git file is malformed")
            })?;
        let reported_git_dir = Path::new(git_dir_line);
        let reported_git_dir = if reported_git_dir.is_absolute() {
            reported_git_dir.to_path_buf()
        } else {
            git_file
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(reported_git_dir)
        };
        if canonical_path(&reported_git_dir)? != expected_git_dir {
            return Err(post_merge_sync_error(
                root,
                "retained worktree .git identity differs",
            ));
        }
    } else if receipt.result.disposition == ResourceFinalizationDisposition::Deleted
        && !local_resources_deleted(root, receipt)?
    {
        return Err(post_merge_sync_error(
            root,
            "deleted finalization still has a local branch or registered/recreated worktree",
        ));
    }
    if git_common_dir(root)? != canonical_expected_path(&sync.git_common_dir)? {
        return Err(post_merge_sync_error(
            root,
            "control checkout Git common directory differs",
        ));
    }
    Ok(())
}

/// Schema-v3 validates lifecycle artifacts rather than prescribing how many
/// commits the operator used. No historical stage is synthesized or rewritten.
fn validate_post_merge_layout_stages(
    root: &Path,
    id: &str,
    sync: &PostMergeSyncEvidence,
) -> Result<(), ObserverError> {
    use cockpit_protocol::{PostMergeDocumentation as Docs, PostMergeLifecycleLayout as Layout};
    let source = &sync.source_pull_request;
    let followup = &sync.sync_pull_request;
    let m0 = source
        .merge_commit
        .as_deref()
        .ok_or_else(|| post_merge_sync_error(root, "missing source merge"))?;
    let m1 = followup
        .merge_commit
        .as_deref()
        .ok_or_else(|| post_merge_sync_error(root, "missing sync merge"))?;
    prove_clean_merge(root, &source.base_revision, &source.head_revision, m0)?;
    if !is_ancestor(root, m0, &followup.base_revision)? {
        return Err(post_merge_sync_error(
            root,
            "actual sync base does not contain the source merge",
        ));
    }
    let (_, m1_parents) = commit_tree_and_parents(root, m1)?;
    if m1_parents.len() != 2 {
        return Err(post_merge_sync_error(
            root,
            "sync merge commit must have exactly two parents",
        ));
    }
    let actual_base = &m1_parents[0];
    if m1_parents[1] != followup.head_revision {
        return Err(post_merge_sync_error(
            root,
            "actual sync merge second parent does not match the reviewed PR head",
        ));
    }
    if !is_ancestor(root, &followup.base_revision, actual_base)? {
        return Err(post_merge_sync_error(
            root,
            "provider-reported sync base is not an ancestor of the actual merge first parent",
        ));
    }
    if git_text(root, &["merge-base", actual_base, &followup.head_revision]).as_deref()
        != Some(source.head_revision.as_str())
    {
        return Err(post_merge_sync_error(
            root,
            "actual sync base must preserve the original source-head fork",
        ));
    }
    prove_clean_merge(root, actual_base, &followup.head_revision, m1)?;
    if git_text(
        root,
        &[
            "merge-base",
            &followup.base_revision,
            &followup.head_revision,
        ],
    )
    .as_deref()
        != Some(source.head_revision.as_str())
        || is_ancestor(root, m0, &followup.head_revision)?
    {
        return Err(post_merge_sync_error(
            root,
            "sync branch must preserve the original source-head fork",
        ));
    }
    let combined = sync.lifecycle_layout == Some(Layout::Combined);
    let finish_parent = single_parent(root, &sync.finish_revision)?;
    validate_layout_attempt_history(
        root,
        id,
        &source.head_revision,
        &finish_parent,
        &sync.finish_revision,
        m1,
    )?;
    let active = format!(".ai/work-items/active/{id}");
    let archive = format!(".ai/work-items/archive/{id}");
    require_same_git_blob_at(
        root,
        &source.head_revision,
        &format!("{active}.contract.json"),
        &sync.archive_revision,
        &format!("{archive}.contract.json"),
    )?;
    let artifacts = [
        "contract.json",
        "summary.json",
        "outcome.json",
        "events.jsonl",
        "task-report.json",
        "task-report.md",
    ];
    let final_prefix = if combined { &archive } else { &active };
    let mut lifecycle_changes = Vec::new();
    for suffix in artifacts {
        let old = format!("{active}.{suffix}");
        let new = format!("{final_prefix}.{suffix}");
        if combined {
            if git_path_exists(root, &finish_parent, &format!("{archive}.{suffix}")) {
                return Err(post_merge_sync_error(
                    root,
                    "combined lifecycle cannot overwrite an existing archive",
                ));
            }
            if git_path_exists(root, &finish_parent, &old) {
                lifecycle_changes.push(format!("D:{old}"));
            }
            lifecycle_changes.push(format!("A:{new}"));
        } else if suffix != "contract.json" {
            lifecycle_changes.push(format!(
                "{}:{new}",
                if git_path_exists(root, &finish_parent, &old) {
                    "M"
                } else {
                    "A"
                }
            ));
        }
    }
    if combined {
        let path = format!("{archive}.archive.json");
        if git_path_exists(root, &finish_parent, &path) {
            return Err(post_merge_sync_error(
                root,
                "archive manifest already exists",
            ));
        }
        lifecycle_changes.push(format!("A:{path}"));
    }
    let verification_path = format!(".ai/evidence/{id}.verification.json");
    if git_blob_bytes(root, &finish_parent, &verification_path).ok()
        != git_blob_bytes(root, &sync.finish_revision, &verification_path).ok()
    {
        lifecycle_changes.push(format!(
            "{}:{verification_path}",
            if git_path_exists(root, &finish_parent, &verification_path) {
                "M"
            } else {
                "A"
            }
        ));
    }
    for path in layout_new_attempt_paths(root, id, &source.head_revision, &sync.finish_revision)? {
        if !git_path_exists(root, &finish_parent, &path) {
            lifecycle_changes.push(format!("A:{path}"));
        }
    }
    validate_stage_changes(root, &sync.finish_revision, &lifecycle_changes)?;
    validate_layout_finish_semantics(
        root,
        id,
        &finish_parent,
        &sync.finish_revision,
        &sync.archive_revision,
        combined,
    )?;
    if !combined {
        require_exact_parent_chain(root, &sync.archive_revision, &[&sync.finish_revision])?;
        let mut archive_changes = vec![format!("A:{archive}.archive.json")];
        for suffix in artifacts {
            archive_changes.push(format!("D:{active}.{suffix}"));
            archive_changes.push(format!("A:{archive}.{suffix}"));
        }
        validate_stage_changes(root, &sync.archive_revision, &archive_changes)?;
        validate_archive_stage_bytes(root, id, &sync.finish_revision, &sync.archive_revision)?;
    }
    match sync.documentation {
        Some(Docs::Projected) => {
            require_exact_parent_chain(root, &sync.docs_revision, &[&sync.archive_revision])?;
            let expected = [
                "M:docs/reference/reference-parity.md".to_owned(),
                "M:docs/reference/reference-parity.zh-CN.md".to_owned(),
                "M:docs/reference/reference-parity.ja.md".to_owned(),
                format!("M:docs/work-items/{id}.md"),
                format!("M:docs/work-items/{id}.zh-CN.md"),
                format!("M:docs/work-items/{id}.ja.md"),
            ];
            validate_stage_changes(root, &sync.docs_revision, &expected)?;
            validate_documentation_projection(
                root,
                id,
                &sync.archive_revision,
                &sync.docs_revision,
            )?;
            for path in expected.iter().map(|entry| &entry[2..]) {
                require_same_git_blob(root, &sync.docs_revision, m1, path)?;
            }
        }
        Some(Docs::Absent) => {
            let contract = git_json(
                root,
                &source.head_revision,
                &format!("{active}.contract.json"),
            )?;
            // Unknown documentation obligations cannot be treated as optional.
            if !contract["documentationImpact"].is_null()
                || contract["scope"].as_array().is_some_and(|paths| {
                    paths.iter().any(|p| {
                        p.as_str().is_some_and(|p| {
                            p.starts_with("docs/work-items/") || p.contains("reference-parity")
                        })
                    })
                })
            {
                return Err(post_merge_sync_error(
                    root,
                    "Contract declares documentation obligations incompatible with absent projection",
                ));
            }
            for rev in [
                &source.head_revision,
                &finish_parent,
                &sync.archive_revision,
                &followup.base_revision,
                &followup.head_revision,
                actual_base,
                &m1.to_owned(),
            ] {
                validate_absent_projection(root, id, rev)?;
            }
        }
        None => {
            return Err(post_merge_sync_error(
                root,
                "documentation mode is required",
            ));
        }
    }
    // No additional commits or content after the declared projection boundary.
    if sync.docs_revision != followup.head_revision {
        return Err(post_merge_sync_error(
            root,
            "sync head differs from the final lifecycle/documentation revision",
        ));
    }
    validate_current_archive_matches_revision(root, id, &sync.archive_revision)?;
    let archive_checkout = materialize_git_revision(root, &sync.archive_revision)?;
    let manifest = git_json(
        root,
        &sync.archive_revision,
        &format!("{archive}.archive.json"),
    )?;
    super::verify_archive_manifest(&archive_checkout.repository, id, &manifest)?;
    for suffix in artifacts.into_iter().chain(["archive.json"]) {
        require_same_git_blob(
            root,
            &sync.archive_revision,
            m1,
            &format!("{archive}.{suffix}"),
        )?;
    }
    require_same_git_blob(root, &sync.finish_revision, m1, &verification_path)?;
    let main_ref = format!(
        "refs/remotes/{}/{}",
        followup.base_remote, followup.base_branch
    );
    let current = git_text(root, &["rev-parse", "--verify", &main_ref])
        .ok_or_else(|| post_merge_sync_error(root, "cannot resolve current main"))?;
    if !is_ancestor(root, m1, &current)? {
        return Err(post_merge_sync_error(
            root,
            "current main does not contain the reviewed sync merge",
        ));
    }
    Ok(())
}

fn git_path_exists(root: &Path, rev: &str, path: &str) -> bool {
    git_text(root, &["rev-parse", "--verify", &format!("{rev}:{path}")]).is_some()
}

fn single_parent(root: &Path, rev: &str) -> Result<String, ObserverError> {
    let (_, parents) = commit_tree_and_parents(root, rev)?;
    if parents.len() != 1 {
        return Err(post_merge_sync_error(
            root,
            "lifecycle revision must have exactly one parent",
        ));
    }
    Ok(parents[0].clone())
}

fn require_same_git_blob_at(
    root: &Path,
    a: &str,
    ap: &str,
    b: &str,
    bp: &str,
) -> Result<(), ObserverError> {
    if git_blob_bytes(root, a, ap)? != git_blob_bytes(root, b, bp)? {
        return Err(post_merge_sync_error(
            root,
            format!("immutable artifact changed: {ap} -> {bp}"),
        ));
    }
    Ok(())
}

fn prove_clean_merge(
    root: &Path,
    base: &str,
    head: &str,
    merge: &str,
) -> Result<(), ObserverError> {
    require_exact_parent_chain(root, merge, &[base, head])?;
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["merge-tree", "--write-tree", base, head])
        .output()
        .map_err(|source| ObserverError::Read {
            path: root.to_path_buf(),
            source,
        })?;
    if !output.status.success() {
        return Err(post_merge_sync_error(
            root,
            "merge result is conflicted or cannot be proven",
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected = stdout.lines().next().unwrap_or_default();
    if !valid_git_object_id(expected) || commit_tree_and_parents(root, merge)?.0 != expected {
        return Err(post_merge_sync_error(
            root,
            "actual merge tree contains unproven content",
        ));
    }
    Ok(())
}

fn validate_absent_projection(root: &Path, id: &str, rev: &str) -> Result<(), ObserverError> {
    let paths = git_text(root, &["ls-tree", "-r", "--name-only", rev])
        .ok_or_else(|| post_merge_sync_error(root, "cannot inventory documentation"))?;
    let short = id.split('-').take(2).collect::<Vec<_>>().join("-");
    if paths.lines().any(|path| {
        path.starts_with(&format!("docs/work-items/{id}."))
            || path.starts_with(&format!("docs/work-items/{short}."))
            || path.starts_with(&format!("docs/work-items/{short}-"))
    }) {
        return Err(post_merge_sync_error(
            root,
            "absent projection has an existing Work Item page",
        ));
    }
    let grep = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "-c",
            "color.grep=false",
            "grep",
            "-I",
            "-n",
            "-F",
            "-e",
            id,
            "-e",
            &short,
            rev,
            "--",
            "docs/",
            "agents/",
        ])
        .output()
        .map_err(|source| ObserverError::Read {
            path: root.into(),
            source,
        })?;
    match grep.status.code() {
        Some(1) => return Ok(()),
        Some(0) => {}
        _ => {
            return Err(post_merge_sync_error(
                root,
                "cannot inspect documentation references",
            ));
        }
    }
    for line in String::from_utf8_lossy(&grep.stdout).lines() {
        let fields = line.splitn(4, ':').collect::<Vec<_>>();
        if fields.len() != 4 {
            return Err(post_merge_sync_error(
                root,
                "documentation observation is malformed",
            ));
        }
        let path = fields[1];
        let text = fields[3];
        if (path.contains("reference-parity")
            && text
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '-')
                .any(|token| token == short || token == id))
            || text.contains(&format!(".ai/work-items/active/{id}"))
        {
            return Err(post_merge_sync_error(
                root,
                "absent projection has a parity entry or active document reference",
            ));
        }
    }
    Ok(())
}

fn validate_layout_attempt_history(
    root: &Path,
    id: &str,
    source: &str,
    finish_parent: &str,
    finish: &str,
    merge: &str,
) -> Result<(), ObserverError> {
    let bindings = finish_attempt_bindings(root, id, source, finish)?;
    let prefix = format!(".ai/evidence/{id}.verification-attempt.");
    let range = format!("{source}..{finish_parent}");
    let commits = git_text(root, &["rev-list", "--first-parent", "--reverse", &range])
        .ok_or_else(|| post_merge_sync_error(root, "cannot enumerate lifecycle history"))?;
    let mut previous = source.to_owned();
    for commit in commits.lines() {
        require_exact_parent_chain(root, commit, &[&previous])?;
        let changes = git_text(
            root,
            &[
                "diff-tree",
                "--no-commit-id",
                "--name-status",
                "--no-renames",
                "-r",
                &previous,
                commit,
            ],
        )
        .ok_or_else(|| post_merge_sync_error(root, "cannot inspect lifecycle history"))?;
        if changes.is_empty()
            || changes
                .lines()
                .any(|line| !line.starts_with(&format!("A\t{prefix}")))
        {
            return Err(post_merge_sync_error(
                root,
                "pre-lifecycle history may only append immutable attempts",
            ));
        }
        let expected = changes
            .lines()
            .map(|line| format!("A:{}", line.split_once('\t').unwrap().1))
            .collect::<Vec<_>>();
        validate_stage_changes(root, commit, &expected)?;
        previous = commit.to_owned();
    }
    if previous != finish_parent {
        return Err(post_merge_sync_error(
            root,
            "lifecycle parent is not the declared source chain",
        ));
    }
    // Existing attempts retain their original Runtime, Contract and snapshot;
    // only attempts newly appended after H0 are checked against this finish.
    let paths = git_text(root, &["ls-tree", "-r", "--name-only", finish_parent])
        .ok_or_else(|| post_merge_sync_error(root, "cannot inventory immutable attempts"))?;
    for path in paths.lines().filter(|p| p.starts_with(&prefix)) {
        require_same_git_blob(root, finish_parent, finish, path)?;
        require_same_git_blob(root, finish_parent, merge, path)?;
    }
    let verification_path = format!(".ai/evidence/{id}.verification.json");
    let verification = git_json(root, finish, &verification_path)?;
    let changed_verification = git_blob_bytes(root, source, &verification_path).ok()
        != git_blob_bytes(root, finish, &verification_path).ok();
    let mut successful_attempt_binds_receipt = false;
    for path in layout_new_attempt_paths(root, id, source, finish)? {
        let suffix = path
            .strip_prefix(&prefix)
            .and_then(|s| s.strip_suffix(".json"))
            .unwrap_or_default();
        let attempt = git_json(root, finish, &path)?;
        if !finish_attempt_matches(&attempt, suffix, id, &bindings) {
            return Err(post_merge_sync_error(
                root,
                "new attempt has invalid identity or execution binding",
            ));
        }
        require_same_git_blob(root, finish, merge, &path)?;
        if attempt["passed"] == true
            && attempt["state"] == "execution_completed"
            && attempt["receipt"] == verification["receipt"]
        {
            successful_attempt_binds_receipt = true;
        }
    }
    if changed_verification && !successful_attempt_binds_receipt {
        return Err(post_merge_sync_error(
            root,
            "changed verification requires a new successful immutable attempt with the exact receipt",
        ));
    }
    Ok(())
}

fn layout_new_attempt_paths(
    root: &Path,
    id: &str,
    source: &str,
    finish: &str,
) -> Result<Vec<String>, ObserverError> {
    let prefix = format!(".ai/evidence/{id}.verification-attempt.");
    let paths = git_text(root, &["ls-tree", "-r", "--name-only", finish])
        .ok_or_else(|| post_merge_sync_error(root, "cannot inventory new attempts"))?;
    Ok(paths
        .lines()
        .filter(|p| p.starts_with(&prefix) && !git_path_exists(root, source, p))
        .map(str::to_owned)
        .collect())
}

// A historical lifecycle consumes its frozen review, not the new consumer's
// analysis implementation identity. This does not admit current source work.
fn analysis_recipe_source_matches(historical: &[u8], compiled: &[u8]) -> bool {
    // Rust treats platform CRLF source line endings as LF. This comparison
    // only checks whether the digest recipe code is supported; all evidence,
    // implementation-input hashes and receipt bytes remain completely raw.
    let normalized = |bytes: &[u8]| {
        bytes
            .iter()
            .enumerate()
            .filter_map(|(i, b)| (!(*b == b'\r' && bytes.get(i + 1) == Some(&b'\n'))).then_some(*b))
            .collect::<Vec<_>>()
    };
    normalized(historical) == normalized(compiled)
}

fn apply_historical_material_projection(
    root: &Path,
    contract: &Contract,
    summary_path: &Path,
    decision: &mut cockpit_core::GovernanceDecision,
) -> Result<(), ObserverError> {
    use cockpit_protocol::{
        MaterialInspectionReviewDecisionInput, MaterialInspectionReviewDecisionReceipt,
    };
    let mut projection =
        super::material_review::material_review_gate_projection(root, contract, summary_path)?;
    if projection
        .effective_unknowns
        .iter()
        .any(|s| s == "material_review_receipt_stale")
    {
        let summary: serde_json::Value = read_json(summary_path)?;
        let pointer = &summary["materialReviewReceipt"];
        let path = pointer["path"].as_str().ok_or_else(|| {
            post_merge_sync_error(root, "historical material receipt path missing")
        })?;
        if !path.starts_with(&format!(
            ".ai/evidence/material-inspection-review/{}/",
            contract.work_item_id
        )) || path.contains("..")
        {
            return Err(post_merge_sync_error(
                root,
                "historical material receipt path is foreign",
            ));
        }
        let bytes = fs::read(root.join(path)).map_err(|source| ObserverError::Read {
            path: root.join(path),
            source,
        })?;
        if pointer["digest"] != Digest::sha256_bytes(&bytes).to_string() {
            return Err(post_merge_sync_error(
                root,
                "historical material receipt byte binding is invalid",
            ));
        }
        super::reject_duplicate_json_keys(&bytes).map_err(|e| post_merge_sync_error(root, e))?;
        let receipt: MaterialInspectionReviewDecisionReceipt = serde_json::from_slice(&bytes)
            .map_err(|e| post_merge_sync_error(root, e.to_string()))?;
        receipt
            .validate_integrity()
            .map_err(|e| post_merge_sync_error(root, e))?;
        let mut request = super::material_review::material_review_request(root, contract)
            .map_err(|e| post_merge_sync_error(root, e.to_string()))?;
        let reviewed = receipt.reviewed_source_head.as_deref().ok_or_else(|| {
            post_merge_sync_error(root, "historical reviewed source head missing")
        })?;
        if !is_ancestor(root, reviewed, &request.reviewed_source_head)? {
            return Err(post_merge_sync_error(
                root,
                "historical review source is not an ancestor",
            ));
        }
        // Fail closed if the digest recipe changed. The original scanner code
        // and all implementation inputs must be preserved in committed history.
        let recipe_source = git_blob_bytes(
            root,
            reviewed,
            "crates/cockpit-repository/src/material_review.rs",
        )?;
        if !analysis_recipe_source_matches(&recipe_source, include_bytes!("material_review.rs")) {
            return Err(post_merge_sync_error(
                root,
                "historical material digest recipe is unsupported",
            ));
        }
        use sha2::{Digest as _, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"ai-cockpit:material-analysis-implementation:v1\0");
        for path in [
            "Cargo.lock",
            "crates/cockpit-core/src/lib.rs",
            "crates/cockpit-git/src/lib.rs",
            "crates/cockpit-protocol/src/lib.rs",
            "crates/cockpit-protocol/src/contract_amendment.rs",
            "crates/cockpit-repository/Cargo.toml",
            "crates/cockpit-repository/src/governance_controls.rs",
            "crates/cockpit-repository/src/lib.rs",
            "crates/cockpit-repository/src/material_review.rs",
            "crates/cockpit-repository/src/rust_material.rs",
            "crates/cockpit-repository/src/status_projection.rs",
        ] {
            require_same_git_blob(root, reviewed, &request.reviewed_source_head, path)?;
            let bytes = git_blob_bytes(root, reviewed, path)?;
            hasher.update((path.len() as u64).to_le_bytes());
            hasher.update(path.as_bytes());
            hasher.update((bytes.len() as u64).to_le_bytes());
            hasher.update(&bytes);
        }
        let typed_contract_digest = request.contract_digest.clone();
        request.contract_digest = super::contract_digest(&root.join(format!(
            ".ai/work-items/active/{}.contract.json",
            contract.work_item_id
        )))?;
        request.analysis_implementation_digest = Digest::sha256_bytes(&hasher.finalize());
        request.reviewed_source_head = reviewed.into();
        let mut value = serde_json::to_value(&request)
            .map_err(|e| post_merge_sync_error(root, e.to_string()))?;
        for key in ["requestDigest", "reviewedSourceHead", "reviewDiagnostic"] {
            value.as_object_mut().unwrap().remove(key);
        }
        request.request_digest = cockpit_protocol::digest_json(&serde_json::json!([
            "ai-cockpit:material-review-request:v1",
            value
        ]))
        .map_err(|e| post_merge_sync_error(root, e.to_string()))?;
        let input = MaterialInspectionReviewDecisionInput {
            schema_version: receipt.schema_version,
            decision: receipt.decision,
            request_digest: receipt.request_digest.clone(),
            reviewer_actor: receipt.reviewer_actor.clone(),
            authority_source: receipt.authority_source.clone(),
            assurance: receipt.assurance,
            evidence_refs: receipt.evidence_refs.clone(),
            rationale: receipt.rationale.clone(),
            residual_risk: receipt.residual_risk.clone(),
        };
        if request.request_digest != receipt.request_digest
            || request.contract_digest != receipt.contract_digest
        {
            return Err(post_merge_sync_error(
                root,
                "historical review does not bind the original canonical Contract/request",
            ));
        }
        // The public pure validator consumes a typed Contract; old canonical
        // documents can also carry preserved legacy fields. Validate policy
        // with its typed digest, then reconstruct the original canonical wire
        // identity independently. Neither source bytes nor Runtime identity change.
        let original_contract_digest = request.contract_digest.clone();
        let original_request_digest = request.request_digest.clone();
        request.contract_digest = typed_contract_digest;
        let mut typed_value = serde_json::to_value(&request)
            .map_err(|e| post_merge_sync_error(root, e.to_string()))?;
        for key in ["requestDigest", "reviewedSourceHead", "reviewDiagnostic"] {
            typed_value.as_object_mut().unwrap().remove(key);
        }
        request.request_digest = cockpit_protocol::digest_json(&serde_json::json!([
            "ai-cockpit:material-review-request:v1",
            typed_value
        ]))
        .map_err(|e| post_merge_sync_error(root, e.to_string()))?;
        let mut typed_input = input;
        typed_input.request_digest = request.request_digest.clone();
        let mut validated = super::material_review::validate_material_review_decision(
            contract,
            &request,
            &typed_input,
            &receipt.recorded_by,
            &receipt.recorded_at,
        )
        .map_err(|e| post_merge_sync_error(root, e.to_string()))?;
        validated.contract_digest = original_contract_digest;
        validated.request_digest = original_request_digest;
        validated.receipt_digest = validated
            .canonical_digest()
            .map_err(|e| post_merge_sync_error(root, e))?;
        if validated != receipt
            || pointer["receiptDigest"] != receipt.receipt_digest.to_string()
            || pointer["requestDigest"] != receipt.request_digest.to_string()
        {
            return Err(post_merge_sync_error(
                root,
                "historical material receipt does not bind its original request",
            ));
        }
        projection.effective_unknowns.retain(|s| {
            s != "material_review_receipt_stale"
                && s != "repository_material_inspection_unavailable"
        });
        projection.discharged_unknowns = vec!["repository_material_inspection_unavailable".into()];
        projection.review_receipt_digest = Some(receipt.receipt_digest);
        projection.review_assurance = Some(receipt.assurance);
    }
    let projection =
        super::material_review::apply_material_review_projection_to_decision(projection, decision);
    if projection.blocked_by_finding
        || projection.projection_unavailable
        || !decision.unknowns.is_empty()
        || !decision.blockers.is_empty()
    {
        return Err(post_merge_sync_error(
            root,
            format!(
                "historical material projection remains blocked or unknown: unknowns={:?}, findings={:?}, unavailable={}, blocked={}",
                decision.unknowns,
                projection.finding_codes,
                projection.projection_unavailable,
                projection.blocked_by_finding
            ),
        ));
    }
    Ok(())
}

fn validate_layout_archive_manifest(
    root: &Path,
    id: &str,
    revision: &str,
    manifest: &serde_json::Value,
) -> Result<(), ObserverError> {
    let keys = [
        "actorProvenance",
        "closeRequired",
        "createdAt",
        "files",
        "historicalArtifacts",
        "occurredAt",
        "protocolVersion",
        "recordedAt",
        "repositoryId",
        "runtimeDigest",
        "runtimeVersion",
        "state",
        "workItemId",
    ];
    if manifest.as_object().is_none_or(|object| {
        object.len() != keys.len() || keys.iter().any(|key| !object.contains_key(*key))
    }) || manifest["protocolVersion"] != 1
        || manifest["workItemId"] != id
        || manifest["repositoryId"] != repository_id(root).to_string()
        || manifest["state"] != "archived"
        || manifest["closeRequired"] != true
        || manifest["actorProvenance"] != "unknown"
        || manifest["historicalArtifacts"] != serde_json::json!([])
        || manifest["recordedAt"] != manifest["occurredAt"]
    {
        return Err(post_merge_sync_error(
            root,
            "ordinary archive lifecycle metadata is invalid or would weaken the real human close boundary",
        ));
    }
    let created_text = manifest["createdAt"].as_str().unwrap_or_default();
    let occurred_text = manifest["occurredAt"].as_str().unwrap_or_default();
    let created = chrono::DateTime::parse_from_rfc3339(created_text)
        .map_err(|_| post_merge_sync_error(root, "ordinary archive creation time is invalid"))?;
    let occurred = chrono::DateTime::parse_from_rfc3339(occurred_text)
        .map_err(|_| post_merge_sync_error(root, "ordinary archive occurrence time is invalid"))?;
    // Runtime takes second-resolution now() immediately before lifecycle_now().
    // Scheduler delay is permitted; both times still precede the manifest write.
    if !created_text.ends_with('Z')
        || !occurred_text.ends_with('Z')
        || created > occurred
        || created_text != created.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    {
        return Err(post_merge_sync_error(
            root,
            "ordinary archive actor/time binding is invalid",
        ));
    }
    let mut files = serde_json::Map::new();
    for (name, suffix) in [
        ("contract", "contract.json"),
        ("summary", "summary.json"),
        ("outcome", "outcome.json"),
        ("events", "events.jsonl"),
        ("taskReport", "task-report.json"),
        ("taskReportMarkdown", "task-report.md"),
    ] {
        let path = format!(".ai/work-items/archive/{id}.{suffix}");
        files.insert(format!("{name}Path"), path.clone().into());
        files.insert(
            format!("{name}Digest"),
            Digest::sha256_bytes(&git_blob_bytes(root, revision, &path)?)
                .to_string()
                .into(),
        );
    }
    if manifest["files"] != serde_json::Value::Object(files) {
        return Err(post_merge_sync_error(
            root,
            "ordinary archive artifact map is not the exact canonical set of paths and byte digests",
        ));
    }
    Ok(())
}

fn validate_layout_finish_semantics(
    root: &Path,
    id: &str,
    parent: &str,
    finish: &str,
    archive: &str,
    combined: bool,
) -> Result<(), ObserverError> {
    use cockpit_core::DecisionState;
    use cockpit_protocol::{HumanBenefitReport, OutcomeState, OutcomeV2};
    let active = format!(".ai/work-items/active/{id}");
    let target = format!(
        ".ai/work-items/{}/{id}",
        if combined { "archive" } else { "active" }
    );
    let mut summary = git_json(root, parent, &format!("{active}.summary.json"))?;
    let observed = git_json(root, finish, &format!("{target}.summary.json"))?;
    let evidence_path = format!(".ai/evidence/{id}.verification.json");
    let evidence = git_json(root, finish, &evidence_path)?;
    let manifest = git_json(
        root,
        archive,
        &format!(".ai/work-items/archive/{id}.archive.json"),
    )?;
    validate_layout_archive_manifest(root, id, archive, &manifest)?;
    let checkout = materialize_git_revision(root, parent)?;
    let isolated = &checkout.repository;
    let contract_path = isolated.join(format!("{active}.contract.json"));
    let contract = read_contract(&contract_path)?;
    let snapshot = cockpit_git::GitRepository::discover(isolated)
        .and_then(|g| g.snapshot())
        .map_err(|e| post_merge_sync_error(root, e.to_string()))?;
    let verification_bytes = git_blob_bytes(root, finish, &evidence_path)?;
    if git_blob_bytes(root, parent, &evidence_path).ok().as_ref() != Some(&verification_bytes) {
        let runtime = RuntimeContext {
            runtime_version: evidence["runtimeVersion"]
                .as_str()
                .ok_or_else(|| post_merge_sync_error(root, "verification Runtime version missing"))?
                .into(),
            runtime_digest: serde_json::from_value(evidence["runtimeDigest"].clone())
                .map_err(|e| post_merge_sync_error(root, e.to_string()))?,
            protocol_version: 1,
        };
        super::record_verification_with_runtime(
            isolated,
            id,
            &evidence["receipt"],
            &runtime,
            &snapshot,
        )?;
        summary = read_json(&isolated.join(format!("{active}.summary.json")))?;
        // The recorder's clock is nondeterministic, but the captured refresh
        // time must be between this verification and the recorded finish.
        let refresh = observed["preflightAt"]
            .as_str()
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok());
        let created = evidence["createdAt"]
            .as_str()
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok());
        let completed = observed["updatedAt"]
            .as_str()
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok());
        if !matches!((refresh, created, completed), (Some(r), Some(c), Some(f)) if c <= r && r <= f)
        {
            return Err(post_merge_sync_error(
                root,
                "verification refresh time is outside captured lifecycle bounds",
            ));
        }
        summary["preflightAt"] = observed["preflightAt"].clone();
    }
    fs::write(isolated.join(&evidence_path), &verification_bytes).map_err(|source| {
        ObserverError::Read {
            path: isolated.join(&evidence_path),
            source,
        }
    })?;

    // A cached green Summary is not authority. Real finish unconditionally
    // requires material review and governance; refresh metadata alone is conditional.
    let runtime = RuntimeContext {
        runtime_version: evidence["runtimeVersion"]
            .as_str()
            .ok_or_else(|| post_merge_sync_error(root, "verification Runtime version missing"))?
            .into(),
        runtime_digest: serde_json::from_value(evidence["runtimeDigest"].clone())
            .map_err(|e| post_merge_sync_error(root, e.to_string()))?,
        protocol_version: 1,
    };
    let mut raw = super::governance_decision_for_pre_execution_boundary(
        isolated,
        &contract,
        &snapshot,
        Some(&runtime),
        None,
    )?;
    apply_historical_material_projection(
        isolated,
        &contract,
        &isolated.join(format!("{active}.summary.json")),
        &mut raw,
    )?;
    let decision = super::apply_preflight_review_evidence(isolated, &contract, raw.clone(), false)?;
    if decision.state != DecisionState::Green {
        return Err(post_merge_sync_error(
            root,
            "captured preflight refresh does not admit finish",
        ));
    }
    let controls =
        super::validate_work_item_governance_controls_with_runtime(isolated, id, &runtime)?;
    let gaps = super::governance_controls::governance_controls_gaps_for_finish(&controls);
    if !gaps.is_empty() {
        return Err(post_merge_sync_error(
            root,
            format!(
                "historical finish governance controls do not admit finish: {}",
                gaps.join(", ")
            ),
        ));
    }
    if summary["preflightDecisionDigest"] != observed["preflightDecisionDigest"]
        || summary["preflightState"] != observed["preflightState"]
    {
        summary["preflightState"] = "green".into();
        summary["preflightDecisionDigest"] = cockpit_protocol::digest_json(
            &serde_json::to_value(raw).map_err(|e| post_merge_sync_error(root, e.to_string()))?,
        )
        .map_err(|e| post_merge_sync_error(root, e.to_string()))?
        .to_string()
        .into();
        summary["preflightContractDigest"] =
            super::contract_digest(&contract_path)?.to_string().into();
        summary["preflightRepositorySnapshotDigest"] =
            super::snapshot_digest(&snapshot)?.to_string().into();
        let refresh = observed["preflightAt"]
            .as_str()
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok());
        let created = evidence["createdAt"]
            .as_str()
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok());
        let completed = observed["updatedAt"]
            .as_str()
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok());
        if !matches!((refresh, created, completed), (Some(r), Some(c), Some(f)) if c <= r && r <= f)
        {
            return Err(post_merge_sync_error(
                root,
                "preflight refresh time is outside captured verification/finish bounds",
            ));
        }
        summary["preflightAt"] = observed["preflightAt"].clone();
    }
    let fact = &observed["lifecycleFacts"]["finish"];
    let time = fact["occurredAt"]
        .as_str()
        .ok_or_else(|| post_merge_sync_error(root, "finish time is missing"))?;
    let parsed_time = chrono::DateTime::parse_from_rfc3339(time)
        .map_err(|_| post_merge_sync_error(root, "finish time is invalid"))?;
    let archive_time = manifest["occurredAt"]
        .as_str()
        .or_else(|| manifest["createdAt"].as_str())
        .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
        .ok_or_else(|| post_merge_sync_error(root, "archive time is invalid"))?;
    if parsed_time > archive_time
        || manifest["createdAt"]
            .as_str()
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
            .is_none_or(|t| t.timestamp() < parsed_time.timestamp())
        || summary["state"] != "checkpointed"
        || summary["checkpointCount"] != 1
        || summary["preflightState"] != "green"
    {
        return Err(post_merge_sync_error(
            root,
            format!(
                "lifecycle state or time order is invalid: state={} checkpoint={} preflight={} finish={} archive={}",
                summary["state"],
                summary["checkpointCount"],
                summary["preflightState"],
                time,
                archive_time
            ),
        ));
    }
    if manifest["runtimeDigest"] != fact["runtimeDigest"]
        || manifest["runtimeVersion"] != fact["runtimeVersion"]
        || fact["runtimeDigest"] != evidence["runtimeDigest"]
        || fact["runtimeVersion"] != evidence["runtimeVersion"]
    {
        return Err(post_merge_sync_error(
            root,
            "finish/archive must preserve verification Runtime identity",
        ));
    }
    let retry = summary["recoveryRetryPending"] == true;
    let replace = retry || summary["verificationRecoveryReconciled"].as_str().is_some();
    for key in [
        "failedGate",
        "recoveryCondition",
        "outcomeState",
        "verificationRecoveryReconciled",
    ] {
        summary.as_object_mut().unwrap().remove(key);
    }
    summary["state"] = "finish_ready".into();
    summary["updatedAt"] = time.into();
    summary["lifecycleFacts"]["finish"] = serde_json::json!({
        "eventType": "work_item_finished", "occurredAt": time, "recordedAt": time,
        "actorProvenance": "unknown", "runtimeDigest": evidence["runtimeDigest"], "runtimeVersion": evidence["runtimeVersion"]
    });
    let report_summary = summary.clone();
    if retry {
        for key in [
            "recoveryRetryPending",
            "recoveryRetryDecisionPath",
            "recoveryRetryDecisionDigest",
            "recoveryRetryContractDigest",
        ] {
            summary.as_object_mut().unwrap().remove(key);
        }
    }
    let mut expected_summary = serde_json::to_vec_pretty(&summary)
        .map_err(|e| post_merge_sync_error(root, e.to_string()))?;
    if combined {
        expected_summary =
            super::normalized_archive_artifact_bytes("summary.json", &expected_summary, id)?;
    }
    if expected_summary != git_blob_bytes(root, finish, &format!("{target}.summary.json"))? {
        let expected: serde_json::Value = serde_json::from_slice(&expected_summary)
            .map_err(|e| post_merge_sync_error(root, e.to_string()))?;
        let differences = expected
            .as_object()
            .unwrap()
            .keys()
            .filter(|k| observed.get(*k) != expected.get(*k))
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        return Err(post_merge_sync_error(
            root,
            format!(
                "Summary contains changes outside the Runtime finish projection: {differences}"
            ),
        ));
    }
    if super::verification_evidence_state(isolated, &contract, &snapshot, false, None)?
        != super::EvidenceState::Complete
    {
        return Err(post_merge_sync_error(
            root,
            "finish verification is not a complete historical receipt",
        ));
    }
    let report = super::task_outcome_report(super::TaskOutcomeReportInput {
        root: isolated,
        contract_path: &contract_path,
        contract: &contract,
        summary: Some(&report_summary),
        snapshot_digest: Some(super::snapshot_digest(&snapshot)?),
        state: OutcomeState::Verified,
        decision_state: DecisionState::Green,
        summary_text: "Verification evidence passed; human-visible benefit remains explicitly unknown unless declared by the Work Item owner.",
        unknowns: &["user_visible_benefit_not_declared".into()],
        evidence_ref: &evidence_path,
        failed_gate_override: None,
        recovery_condition_override: None,
        historical: false,
        usage_cutoff: Some(time),
        closed_usage_validation: None,
    })?;
    let report_bytes = serde_json::to_vec_pretty(
        &serde_json::to_value(&report).map_err(|e| post_merge_sync_error(root, e.to_string()))?,
    )
    .map_err(|e| post_merge_sync_error(root, e.to_string()))?;
    let markdown = super::task_outcome_markdown(&report).into_bytes();
    // Usage-bound reports are frozen at finish, including legal active refs.
    for (suffix, bytes) in [
        ("task-report.json", &report_bytes),
        ("task-report.md", &markdown),
    ] {
        if *bytes != git_blob_bytes(root, finish, &format!("{target}.{suffix}"))? {
            return Err(post_merge_sync_error(
                root,
                "Task Outcome report differs from the canonical finish projection",
            ));
        }
    }
    let outcome = OutcomeV2 {
        schema_version: 2, repository_id: contract.repository_id.clone(), work_item_id: id.into(), state: OutcomeState::Verified,
        decision_state: Some(DecisionState::Green), summary: "Verification evidence passed; human-visible benefit remains explicitly unknown unless declared by the Work Item owner.".into(),
        acceptance_results: contract.acceptance_criteria.clone(), unknowns: vec!["user_visible_benefit_not_declared".into()], evidence_refs: vec![evidence_path.clone()],
        human_benefit_report: HumanBenefitReport { state: OutcomeState::Unknown, user_visible_changes: vec![], affected_users: vec![], unknowns: vec!["user_visible_benefit_not_declared".into()], evidence_refs: vec![evidence_path.clone()] },
        task_outcome_report: Some(report.clone()), failed_gate: None, recovery_condition: None, recovery_decision: None, historical_status: None, governance_reasons: vec![], finalization: None,
    };
    let mut value =
        serde_json::to_value(outcome).map_err(|e| post_merge_sync_error(root, e.to_string()))?;
    value["protocolVersion"] = 1.into();
    value["state"] = "finish_ready".into();
    value["createdAt"] = time.into();
    value["verification"] =
        serde_json::json!({"status": "verified", "required": true, "evidencePath": evidence_path});
    value["evidenceDigest"] = cockpit_protocol::digest_json(&evidence)
        .map_err(|e| post_merge_sync_error(root, e.to_string()))?
        .to_string()
        .into();
    value["taskReportDigest"] = Digest::sha256_bytes(&report_bytes).to_string().into();
    value["taskReportMarkdownDigest"] = Digest::sha256_bytes(&markdown).to_string().into();
    let mut outcome_bytes = serde_json::to_vec_pretty(&value)
        .map_err(|e| post_merge_sync_error(root, e.to_string()))?;
    if combined {
        outcome_bytes =
            super::normalized_archive_artifact_bytes("outcome.json", &outcome_bytes, id)?;
    }
    if outcome_bytes != git_blob_bytes(root, finish, &format!("{target}.outcome.json"))? {
        return Err(post_merge_sync_error(
            root,
            "Outcome differs from the canonical finish/archive projection",
        ));
    }
    let event_path = isolated.join(format!("{active}.events.jsonl"));
    let old_events = fs::read(&event_path).unwrap_or_default();
    let count = old_events
        .split(|b| *b == b'\n')
        .filter(|line| !line.is_empty())
        .count();
    let actual_events = git_blob_bytes(root, finish, &format!("{target}.events.jsonl"))?;
    let first_new = actual_events
        .split(|b| *b == b'\n')
        .filter(|line| !line.is_empty())
        .nth(count)
        .ok_or_else(|| post_merge_sync_error(root, "completion event is missing"))?;
    let first: serde_json::Value = serde_json::from_slice(first_new)
        .map_err(|e| post_merge_sync_error(root, e.to_string()))?;
    let event_time = first["timestamp"]
        .as_str()
        .ok_or_else(|| post_merge_sync_error(root, "completion event time is missing"))?;
    let parsed_event = chrono::DateTime::parse_from_rfc3339(event_time)
        .map_err(|_| post_merge_sync_error(root, "completion event time is invalid"))?;
    if parsed_event.timestamp() < parsed_time.timestamp() || parsed_event > archive_time {
        return Err(post_merge_sync_error(
            root,
            "completion event is outside lifecycle time bounds",
        ));
    }
    super::append_task_outcome_events_at(isolated, &contract, &report, replace, event_time)?;
    let mut expected_events = fs::read(&event_path).map_err(|source| ObserverError::Read {
        path: event_path,
        source,
    })?;
    if combined {
        expected_events =
            super::normalized_archive_artifact_bytes("events.jsonl", &expected_events, id)?;
    }
    if expected_events != actual_events {
        return Err(post_merge_sync_error(
            root,
            "event stream differs from the canonical finish/archive projection",
        ));
    }
    Ok(())
}

fn validate_post_merge_sync_repository_evidence(
    root: &Path,
    receipt: &ResourceFinalizationReceipt,
) -> Result<(), ObserverError> {
    if !receipt.is_post_merge_sync() {
        return Ok(());
    }
    let sync = receipt.post_merge_sync.as_ref().ok_or_else(|| {
        post_merge_sync_error(
            root,
            "schema-v2 finalization is missing postMergeSync evidence",
        )
    })?;
    if matches!(
        receipt.schema_version,
        cockpit_protocol::RESOURCE_FINALIZATION_LIFECYCLE_LAYOUT_SCHEMA_VERSION
            | cockpit_protocol::RESOURCE_FINALIZATION_EVIDENCE_OBSERVATION_SCHEMA_VERSION
    ) {
        validate_post_merge_layout_stages(root, &receipt.work_item_id, sync)?;
    } else {
        validate_post_merge_sync_commit_stages(root, &receipt.work_item_id, sync)?;
    }
    validate_post_merge_sync_delegated_evidence(root, receipt, sync)?;
    validate_post_merge_sync_local_state(root, receipt, sync)?;
    Ok(())
}

/// Revalidate a stored finalization receipt and local cleanup postconditions.
pub fn verify_resource_finalization(
    root: &Path,
    work_item_id: &str,
    runtime: &RuntimeContext,
) -> Result<serde_json::Value, ObserverError> {
    verify_resource_finalization_internal(root, work_item_id, Some(runtime))
}

pub(crate) fn verify_resource_finalization_internal(
    root: &Path,
    work_item_id: &str,
    runtime: Option<&RuntimeContext>,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let (receipt, path, receipt_digest, sequence) =
        resolve_resource_finalization_head(&root, work_item_id).map_err(|error| {
            finalization_observation_error(
                resource_finalization_decision_path(&root, work_item_id),
                FinalizationErrorCode::RecordCorrupt,
                error,
            )
        })?;
    if receipt.is_post_merge_sync()
        && (sequence != 1 || receipt.result.disposition != ResourceFinalizationDisposition::Deleted)
    {
        return Err(finalization_observation_error(
            path.clone(),
            FinalizationErrorCode::CleanupPending,
            ObserverError::State {
                path: path.clone(),
                message: "schema-v2 finalization verification requires the sequence-1 Deleted transition after its Retained root".into(),
            },
        ));
    }
    let (contract, finalization_contract_digest) = archived_contract_digest(&root, work_item_id)
        .map_err(|error| {
            finalization_observation_error(
                root.join(".ai/work-items/archive")
                    .join(format!("{work_item_id}.contract.json")),
                FinalizationErrorCode::RecordCorrupt,
                error,
            )
        })?;
    let expected_resource_context = effective_resource_context(&root, work_item_id, &contract)
        .map_err(|error| {
            finalization_observation_error(
                root.join(".ai/decisions")
                    .join(format!("{work_item_id}.resource-context.json")),
                FinalizationErrorCode::RecordCorrupt,
                error,
            )
        })?;
    let current_contract_canonical_digest = contract_digest(
        &root
            .join(".ai/work-items/archive")
            .join(format!("{work_item_id}.contract.json")),
    )?;
    let contract_amendment_revalidation = runtime
        .and_then(|_| {
            load_recovery_decision(&root, work_item_id, None)
                .ok()
                .flatten()
        })
        .filter(|decision| {
            decision.successor_binding_mode.as_deref() == Some("contract_amendment_revalidation")
                && decision.current_contract_digest.as_ref()
                    == Some(&current_contract_canonical_digest)
                && decision.current_contract_digest.as_ref()
                    != Some(&decision.predecessor_contract_digest)
                && receipt.contract_digest.as_ref()
                    == Some(
                        decision
                            .predecessor_finalization_contract_digest
                            .as_ref()
                            .unwrap_or(&decision.predecessor_contract_digest),
                    )
        })
        .is_some();
    // A Contract-amendment successor is an explicit current revalidation of
    // the amended Contract. Once that successor has completed its own
    // verified close, the predecessor's provider finalization remains valid
    // historical evidence even when it was emitted by an older Runtime. This
    // projection is deliberately narrower than a general Runtime upgrade:
    // the recovery decision, current archive, historical evidence, successor,
    // and finalization head must all bind before the old Runtime identity is
    // tolerated.
    let contract_amendment_revalidation_resolved = contract_amendment_revalidation
        && recovery_successor_resolves_pending_close(&root, work_item_id, &contract.repository_id);
    let mut receipt_for_context_validation = receipt.clone();
    if sequence > 0
        && matches!(
            receipt.result.disposition,
            ResourceFinalizationDisposition::Retained
        )
        && matches!(
            receipt.before.pull_request,
            cockpit_protocol::ResourceFinalizationPullRequestState::Unmerged
        )
        && matches!(
            receipt.after.pull_request,
            cockpit_protocol::ResourceFinalizationPullRequestState::Merged
        )
        && receipt.before.branch == receipt.after.branch
        && receipt.before.worktree == receipt.after.worktree
    {
        // The resolver has already validated this member in transition
        // context. Normalize only the intrinsic precondition while binding
        // the unchanged repository, Contract, and resource identities below.
        receipt_for_context_validation.before.pull_request =
            cockpit_protocol::ResourceFinalizationPullRequestState::Merged;
    }
    validate_resource_finalization_receipt_for(
        &receipt_for_context_validation,
        &contract.repository_id,
        work_item_id,
        (!contract_amendment_revalidation).then_some(&finalization_contract_digest),
        expected_resource_context.as_ref(),
    )
    .map_err(|error| {
        finalization_observation_error(path.clone(), error.finalization_error_code(), error)
    })?;
    validate_post_merge_sync_repository_evidence(&root, &receipt).map_err(|error| {
        finalization_observation_error(path.clone(), FinalizationErrorCode::IdentityMismatch, error)
    })?;
    validate_historical_finalization(&root, &receipt, &path).map_err(|error| {
        finalization_observation_error(path.clone(), FinalizationErrorCode::RecordCorrupt, error)
    })?;
    ensure_resource_finalization_base_binding(&receipt, &contract, &path).map_err(|error| {
        finalization_observation_error(path.clone(), FinalizationErrorCode::BaseMismatch, error)
    })?;
    let inferred_legacy_shared_worktree =
        infer_legacy_shared_worktree_retained(&root, &receipt, &contract);
    let historical_recovery = if let Some(runtime) = runtime {
        load_historical_finalization_recovery(&root, work_item_id, &receipt, &contract, runtime)
            .map_err(|error| {
                finalization_observation_error(
                    historical_finalization_recovery_path(&root, work_item_id),
                    FinalizationErrorCode::HistoricalRecoveryRequired,
                    error,
                )
            })?
    } else {
        None
    };
    let historical_runtime_projection = if let Some(runtime) = runtime {
        if historical_recovery.is_none()
            && (receipt.runtime_version != runtime.runtime_version
                || receipt.runtime_digest != runtime.runtime_digest)
        {
            let kind = closed_finalization_projection_kind(
                &root,
                work_item_id,
                &receipt,
                &path,
                &receipt_digest,
                sequence,
                &contract.repository_id,
            )
            .or_else(|| {
                if fs::symlink_metadata(
                    root.join(".ai/decisions")
                        .join(format!("{work_item_id}.close.json")),
                )
                .is_ok()
                {
                    // Once a close record exists, its historical projection
                    // must be validated by `closed_finalization_projection_kind`.
                    // Do not fall back to the pre-close successor proof when
                    // the close binding itself is missing, malformed, or
                    // tampered.
                    None
                } else {
                    contract_amendment_revalidation_resolved
                        .then_some("contract_amendment_revalidation")
                }
            })
            .or_else(|| inferred_legacy_shared_worktree.then_some("shared_worktree_retained"));
            if kind.is_none()
                && let Err(error) = ensure_resource_runtime_identity(&receipt, runtime, &path)
            {
                return Err(finalization_observation_error(
                    path.clone(),
                    FinalizationErrorCode::RuntimeMismatch,
                    format!(
                        "{error}; inspect with `ai-cockpit work-item finalize-recovery-plan --repo <repository> --id {work_item_id}` before recording historical recovery"
                    ),
                ));
            }
            kind
        } else {
            None
        }
    } else {
        None
    };
    let resources_deleted = local_resources_deleted(&root, &receipt).map_err(|error| {
        finalization_observation_error(
            path.clone(),
            FinalizationErrorCode::ObservationUnavailable,
            error,
        )
    })?;
    if matches!(
        receipt.result.disposition,
        ResourceFinalizationDisposition::Deleted | ResourceFinalizationDisposition::Abandoned
    ) && !resources_deleted
    {
        return Err(finalization_observation_error(
            path,
            FinalizationErrorCode::CleanupPending,
            "resource finalization cleanup postconditions are not satisfied",
        ));
    }
    let mut result = serde_json::json!({
        "workItemId": work_item_id,
        "state": if historical_runtime_projection.is_some() {
            "historical_verified"
        } else {
            "verified"
        },
        "disposition": receipt.result.disposition,
        "sequence": sequence,
        "headPath": repository_relative_path(&root, &path),
        "headDigest": receipt_digest,
        "receipt": receipt
    });
    if let Some(recovery) = historical_recovery {
        result["historical"] = serde_json::json!(true);
        result["historicalKind"] =
            serde_json::to_value(recovery.historical_kind).map_err(|error| {
                ObserverError::State {
                    path: path.clone(),
                    message: error.to_string(),
                }
            })?;
        result["assurance"] = recovery.assurance.into();
        result["recoveryPath"] = repository_relative_path(
            &root,
            &historical_finalization_recovery_path(&root, work_item_id),
        )
        .into();
    } else if let Some(kind) = historical_runtime_projection {
        result["historical"] = serde_json::json!(true);
        result["historicalKind"] = kind.into();
        result["assurance"] = "historical_low".into();
        result["historicalReason"] =
            "closed predecessor receipt was verified under an older Runtime".into();
    } else if inferred_legacy_shared_worktree {
        result["historical"] = serde_json::json!(true);
        result["historicalKind"] = "shared_worktree_retained".into();
        result["assurance"] = "historical_low".into();
        result["historicalReason"] =
            "legacy local shared-worktree receipt was verified from repository-bound facts".into();
    } else if let Some(historical) = receipt.historical.as_ref() {
        result["historical"] = serde_json::json!(true);
        result["historicalKind"] =
            serde_json::to_value(&historical.kind).map_err(|error| ObserverError::State {
                path: path.clone(),
                message: error.to_string(),
            })?;
        result["assurance"] = historical.assurance.clone().into();
    }
    Ok(result)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct OrdinaryCleanupBinding {
    schema_version: u32,
    repository_id: String,
    work_item_id: String,
    contract_digest: Digest,
    archive_manifest_digest: Digest,
    branch: String,
    branch_ref: String,
    head_revision: String,
    worktree_path: String,
    worktree_git_dir: String,
    worktree_id: Digest,
    runtime_version: String,
    runtime_digest: Digest,
    captured_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrdinaryCleanupObservation {
    pub branch: String,
    pub worktree: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrdinaryCleanupResult {
    pub state: String,
    #[serde(default)]
    pub failure_codes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrdinaryCleanupReceipt {
    pub schema_version: u32,
    pub operation_id: String,
    pub repository_id: String,
    pub work_item_id: String,
    pub contract_digest: Digest,
    pub binding_digest: Digest,
    pub branch_ref: String,
    pub head_revision: String,
    pub worktree_id: Digest,
    pub sequence: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predecessor_receipt_digest: Option<Digest>,
    pub runtime_version: String,
    pub runtime_digest: Digest,
    pub observed_at: String,
    pub observation: OrdinaryCleanupObservation,
    pub result: OrdinaryCleanupResult,
}

pub(crate) struct ValidatedOrdinaryCleanupReceipt {
    pub(crate) receipt: OrdinaryCleanupReceipt,
    pub(crate) digest: Digest,
}

#[derive(Default)]
pub(crate) struct GitWorktreeRecord {
    pub(crate) path: Option<PathBuf>,
    pub(crate) head: Option<String>,
    pub(crate) branch_ref: Option<String>,
}

fn git_bytes(root: &Path, args: &[&str]) -> Option<Vec<u8>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    output.status.success().then_some(output.stdout)
}

pub(crate) fn git_worktree_records(root: &Path) -> Result<Vec<GitWorktreeRecord>, ObserverError> {
    let output = git_text(root, &["worktree", "list", "--porcelain"]).ok_or_else(|| {
        ObserverError::State {
            path: root.into(),
            message: "ordinary close cannot prove an exact worktree: `git worktree list --porcelain` failed"
                .into(),
        }
    })?;
    let mut records = Vec::new();
    for block in output
        .split("\n\n")
        .filter(|block| !block.trim().is_empty())
    {
        let mut record = GitWorktreeRecord::default();
        for line in block.lines() {
            if let Some(value) = line.strip_prefix("worktree ") {
                record.path = Some(PathBuf::from(value));
            } else if let Some(value) = line.strip_prefix("HEAD ") {
                record.head = Some(value.into());
            } else if let Some(value) = line.strip_prefix("branch ") {
                record.branch_ref = Some(value.into());
            }
        }
        records.push(record);
    }
    Ok(records)
}

fn ordinary_worktree_id(repository_id: &str, worktree_path: &str, git_dir: &str) -> Digest {
    Digest::sha256_bytes(
        format!("ordinary-worktree-v1\0{repository_id}\0{worktree_path}\0{git_dir}").as_bytes(),
    )
}

fn archive_bytes_are_at_head(
    root: &Path,
    work_item_id: &str,
    artifact: &str,
) -> Result<(), ObserverError> {
    let relative = format!(".ai/work-items/archive/{work_item_id}.{artifact}.json");
    let path = root.join(&relative);
    let expected = fs::read(&path).map_err(|source| ObserverError::Read {
        path: path.clone(),
        source,
    })?;
    let spec = format!("HEAD:{relative}");
    let Some(actual) = git_bytes(root, &["show", &spec]) else {
        return Err(ObserverError::State {
            path,
            message: format!(
                "ordinary close cannot prove an exact archived {artifact} at current HEAD"
            ),
        });
    };
    if actual != expected {
        return Err(ObserverError::State {
            path,
            message: format!(
                "ordinary close cannot prove exact archived {artifact} bytes at current HEAD"
            ),
        });
    }
    Ok(())
}

pub(crate) fn capture_ordinary_cleanup_binding(
    root: &Path,
    work_item_id: &str,
    contract: &Contract,
    contract_path: &Path,
    archive_manifest_path: &Path,
    runtime: &RuntimeContext,
) -> Result<OrdinaryCleanupBinding, ObserverError> {
    archive_bytes_are_at_head(root, work_item_id, "contract")?;
    archive_bytes_are_at_head(root, work_item_id, "archive")?;

    let canonical_root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let readiness = repository_readiness(&canonical_root)?;
    let layout = discover_worktree_layout(&canonical_root)?;
    let current_is_primary = layout.primary == canonical_root;
    let current_is_discovered_default =
        readiness.current_branch.as_deref() == readiness.default_branch.as_deref();
    if current_is_primary && (layout.paths.len() > 1 || current_is_discovered_default) {
        let discovered_default = readiness
            .default_branch
            .as_deref()
            .map(|branch| format!("discovered default branch {branch}"))
            .unwrap_or_else(|| "the primary repository worktree".into());
        return Err(ObserverError::State {
            path: canonical_root,
            message: format!(
                "ordinary close requires a dedicated linked worktree on a non-default branch; current checkout is {discovered_default}; switch to the Work Item worktree before close"
            ),
        });
    }
    let records = git_worktree_records(root)?;
    let matching = records
        .into_iter()
        .filter_map(|record| {
            let path = record.path.clone()?;
            let canonical = fs::canonicalize(&path).ok()?;
            (canonical == canonical_root).then_some(record)
        })
        .collect::<Vec<_>>();
    if matching.len() != 1 {
        return Err(ObserverError::State {
            path: canonical_root,
            message: format!(
                "ordinary close requires exactly one associated worktree record for the repository root; found {}",
                matching.len()
            ),
        });
    }
    let record = &matching[0];
    let branch_ref = record
        .branch_ref
        .as_deref()
        .ok_or_else(|| ObserverError::State {
            path: root.into(),
            message: "ordinary close cannot bind cleanup from a detached worktree".into(),
        })?;
    let branch = branch_ref
        .strip_prefix("refs/heads/")
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ObserverError::State {
            path: root.into(),
            message: format!(
                "ordinary close requires an exact local branch ref, found {branch_ref}"
            ),
        })?;
    let symbolic_ref = git_text(root, &["symbolic-ref", "--quiet", "HEAD"]).ok_or_else(|| {
        ObserverError::State {
            path: root.into(),
            message: "ordinary close cannot prove the current symbolic branch ref".into(),
        }
    })?;
    if symbolic_ref != branch_ref {
        return Err(ObserverError::State {
            path: root.into(),
            message: format!(
                "ordinary close branch identity mismatch: worktree records {branch_ref}, HEAD records {symbolic_ref}"
            ),
        });
    }
    let head_revision =
        git_text(root, &["rev-parse", "--verify", "HEAD^{commit}"]).ok_or_else(|| {
            ObserverError::State {
                path: root.into(),
                message: "ordinary close cannot prove an exact current HEAD commit".into(),
            }
        })?;
    if !valid_git_object_id(&head_revision)
        || record.head.as_deref() != Some(head_revision.as_str())
    {
        return Err(ObserverError::State {
            path: root.into(),
            message: "ordinary close worktree HEAD does not match the exact current HEAD commit"
                .into(),
        });
    }
    let git_dir = git_text(root, &["rev-parse", "--absolute-git-dir"])
        .map(PathBuf::from)
        .and_then(|path| fs::canonicalize(path).ok())
        .ok_or_else(|| ObserverError::State {
            path: root.into(),
            message: "ordinary close cannot prove a stable exact worktree Git directory".into(),
        })?;
    let worktree_path = canonical_root.display().to_string();
    let worktree_git_dir = git_dir.display().to_string();
    Ok(OrdinaryCleanupBinding {
        schema_version: 1,
        repository_id: contract.repository_id.clone(),
        work_item_id: work_item_id.into(),
        contract_digest: contract_digest(contract_path)?,
        archive_manifest_digest: Digest::sha256_bytes(&fs::read(archive_manifest_path).map_err(
            |source| ObserverError::Read {
                path: archive_manifest_path.into(),
                source,
            },
        )?),
        branch: branch.into(),
        branch_ref: branch_ref.into(),
        head_revision,
        worktree_id: ordinary_worktree_id(
            &contract.repository_id,
            &worktree_path,
            &worktree_git_dir,
        ),
        worktree_path,
        worktree_git_dir,
        runtime_version: runtime.runtime_version.clone(),
        runtime_digest: runtime.runtime_digest.clone(),
        captured_at: now(),
    })
}

pub(crate) fn ordinary_cleanup_binding_from_decision(
    root: &Path,
    work_item_id: &str,
    repository_id: &str,
    decision: &serde_json::Value,
) -> Result<Option<OrdinaryCleanupBinding>, ObserverError> {
    let binding_value = decision.get("ordinaryCleanupBinding");
    let digest_value = decision
        .get("ordinaryCleanupBindingDigest")
        .and_then(serde_json::Value::as_str);
    if binding_value.is_none() && digest_value.is_none() {
        return Ok(None);
    }
    let binding_value = binding_value.ok_or_else(|| ObserverError::State {
        path: root.join(".ai/decisions"),
        message: "ordinary cleanup binding digest exists without its binding".into(),
    })?;
    let expected_digest = digest_value.ok_or_else(|| ObserverError::State {
        path: root.join(".ai/decisions"),
        message: "ordinary cleanup binding is missing its digest".into(),
    })?;
    let actual_digest = cockpit_protocol::digest_json(binding_value)
        .map_err(|error| ObserverError::State {
            path: root.join(".ai/decisions"),
            message: error.to_string(),
        })?
        .to_string();
    if actual_digest != expected_digest {
        return Err(ObserverError::State {
            path: root.join(".ai/decisions"),
            message: "ordinary cleanup binding digest mismatch".into(),
        });
    }
    let binding: OrdinaryCleanupBinding =
        serde_json::from_value(binding_value.clone()).map_err(|error| ObserverError::State {
            path: root.join(".ai/decisions"),
            message: format!("ordinary cleanup binding is invalid: {error}"),
        })?;
    if binding.schema_version != 1
        || binding.repository_id != repository_id
        || binding.work_item_id != work_item_id
        || binding.branch_ref != format!("refs/heads/{}", binding.branch)
        || !valid_git_object_id(&binding.head_revision)
        || !Path::new(&binding.worktree_path).is_absolute()
        || !Path::new(&binding.worktree_git_dir).is_absolute()
        || Path::new(&binding.worktree_path)
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
        || Path::new(&binding.worktree_git_dir)
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(ObserverError::State {
            path: root.join(".ai/decisions"),
            message: "ordinary cleanup binding identity fields are invalid".into(),
        });
    }
    let contract_path = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.contract.json"));
    let manifest_path = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.archive.json"));
    if binding.contract_digest != contract_digest(&contract_path)?
        || binding.archive_manifest_digest
            != Digest::sha256_bytes(&fs::read(&manifest_path).map_err(|source| {
                ObserverError::Read {
                    path: manifest_path.clone(),
                    source,
                }
            })?)
    {
        return Err(ObserverError::State {
            path: contract_path,
            message: "ordinary cleanup binding does not match the immutable archive".into(),
        });
    }
    if binding.worktree_id
        != ordinary_worktree_id(
            repository_id,
            &binding.worktree_path,
            &binding.worktree_git_dir,
        )
    {
        return Err(ObserverError::State {
            path: root.join(".ai/decisions"),
            message: "ordinary cleanup binding worktree identity digest mismatch".into(),
        });
    }
    let common_dir = git_text(
        root,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )
    .map(PathBuf::from)
    .and_then(|path| fs::canonicalize(path).ok())
    .ok_or_else(|| ObserverError::State {
        path: root.into(),
        message: "cannot prove the repository common Git directory".into(),
    })?;
    let bound_git_dir = Path::new(&binding.worktree_git_dir);
    if bound_git_dir != common_dir && !bound_git_dir.starts_with(common_dir.join("worktrees")) {
        return Err(ObserverError::State {
            path: bound_git_dir.into(),
            message:
                "ordinary cleanup binding points outside this repository's worktree identities"
                    .into(),
        });
    }
    Ok(Some(binding))
}

pub(crate) fn ordinary_cleanup_receipt_head(
    root: &Path,
    work_item_id: &str,
    binding: &OrdinaryCleanupBinding,
    binding_digest: &Digest,
) -> Result<Option<ValidatedOrdinaryCleanupReceipt>, ObserverError> {
    let decisions = root.join(".ai/decisions");
    let prefix = format!("{work_item_id}.cleanup.");
    let mut receipts = Vec::new();
    for entry in fs::read_dir(&decisions).map_err(|source| ObserverError::Read {
        path: decisions.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: decisions.clone(),
            source,
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with(&prefix) {
            continue;
        }
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
            return Err(ObserverError::State {
                path,
                message: "ordinary cleanup receipt must be a regular non-symlink file".into(),
            });
        }
        let suffix = name
            .strip_prefix(&prefix)
            .and_then(|value| value.strip_suffix(".json"))
            .ok_or_else(|| ObserverError::State {
                path: path.clone(),
                message: "ordinary cleanup receipt filename is malformed".into(),
            })?;
        let (sequence_text, digest_text) =
            suffix.split_once('.').ok_or_else(|| ObserverError::State {
                path: path.clone(),
                message: "ordinary cleanup receipt filename is missing sequence or digest".into(),
            })?;
        let filename_sequence = sequence_text
            .parse::<u64>()
            .map_err(|_| ObserverError::State {
                path: path.clone(),
                message: "ordinary cleanup receipt filename sequence is invalid".into(),
            })?;
        let value = read_json(&path)?;
        let receipt: OrdinaryCleanupReceipt =
            serde_json::from_value(value.clone()).map_err(|error| ObserverError::State {
                path: path.clone(),
                message: format!("ordinary cleanup receipt is invalid: {error}"),
            })?;
        let digest =
            cockpit_protocol::digest_json(&value).map_err(|error| ObserverError::State {
                path: path.clone(),
                message: error.to_string(),
            })?;
        let expected_filename_digest = digest
            .to_string()
            .strip_prefix("sha256:")
            .unwrap_or_default()
            .to_owned();
        if filename_sequence != receipt.sequence || digest_text != expected_filename_digest {
            return Err(ObserverError::State {
                path,
                message: "ordinary cleanup receipt filename does not match its content".into(),
            });
        }
        if receipt.schema_version != 1
            || receipt.repository_id != binding.repository_id
            || receipt.work_item_id != work_item_id
            || receipt.contract_digest != binding.contract_digest
            || &receipt.binding_digest != binding_digest
            || receipt.branch_ref != binding.branch_ref
            || receipt.head_revision != binding.head_revision
            || receipt.worktree_id != binding.worktree_id
            || !matches!(receipt.result.state.as_str(), "failed" | "verified")
        {
            return Err(ObserverError::State {
                path,
                message: "ordinary cleanup receipt identity or state is invalid".into(),
            });
        }
        receipts.push(ValidatedOrdinaryCleanupReceipt { receipt, digest });
    }
    receipts.sort_by_key(|item| item.receipt.sequence);
    let mut predecessor = None;
    for (index, item) in receipts.iter().enumerate() {
        let expected_sequence = index as u64 + 1;
        if item.receipt.sequence != expected_sequence
            || item.receipt.predecessor_receipt_digest != predecessor
        {
            return Err(ObserverError::State {
                path: decisions.clone(),
                message: "ordinary cleanup receipt chain is not contiguous and append-only".into(),
            });
        }
        predecessor = Some(item.digest.clone());
    }
    Ok(receipts.pop())
}

fn observe_bound_ordinary_resources(
    root: &Path,
    binding: &OrdinaryCleanupBinding,
) -> Result<OrdinaryCleanupObservation, ObserverError> {
    let branch_output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "for-each-ref",
            "--format=%(objectname)",
            &binding.branch_ref,
        ])
        .output()
        .map_err(|source| ObserverError::State {
            path: root.into(),
            message: format!("cannot observe exact local branch ref: {source}"),
        })?;
    let branch = if branch_output.status.success() {
        let target = String::from_utf8(branch_output.stdout)
            .map_err(|error| ObserverError::State {
                path: root.into(),
                message: format!("local branch observation is not UTF-8: {error}"),
            })?
            .trim()
            .to_owned();
        if target.is_empty() {
            "removed"
        } else if target == binding.head_revision {
            "present"
        } else {
            "identity_conflict"
        }
    } else {
        return Err(ObserverError::State {
            path: root.into(),
            message: format!(
                "cannot distinguish an absent local branch from Git observation failure: {}",
                String::from_utf8_lossy(&branch_output.stderr).trim()
            ),
        });
    };

    let mut associated = Vec::new();
    let mut observation_unknown = false;
    for record in git_worktree_records(root)? {
        let Some(path) = record.path.as_ref() else {
            observation_unknown = true;
            continue;
        };
        let canonical_path = match fs::canonicalize(path) {
            Ok(path) => Some(path),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(_) => {
                observation_unknown = true;
                None
            }
        };
        let canonical_git_dir = if canonical_path.is_some() {
            match git_text(path, &["rev-parse", "--absolute-git-dir"]) {
                Some(git_dir) => match fs::canonicalize(git_dir) {
                    Ok(path) => Some(path),
                    Err(_) => {
                        observation_unknown = true;
                        None
                    }
                },
                None => {
                    observation_unknown = true;
                    None
                }
            }
        } else {
            None
        };
        if canonical_path
            .as_ref()
            .is_some_and(|path| path == Path::new(&binding.worktree_path))
            || canonical_git_dir
                .as_ref()
                .is_some_and(|path| path == Path::new(&binding.worktree_git_dir))
        {
            associated.push((record, canonical_path, canonical_git_dir));
        }
    }
    let worktree = if associated.len() > 1 {
        "identity_conflict"
    } else if let Some((record, path, git_dir)) = associated.first() {
        if path.as_deref() == Some(Path::new(&binding.worktree_path))
            && git_dir.as_deref() == Some(Path::new(&binding.worktree_git_dir))
            && record.branch_ref.as_deref() == Some(binding.branch_ref.as_str())
            && record.head.as_deref() == Some(binding.head_revision.as_str())
        {
            "present"
        } else if observation_unknown {
            "unknown"
        } else {
            "identity_conflict"
        }
    } else {
        let worktree_path = match fs::symlink_metadata(&binding.worktree_path) {
            Ok(_) => Some(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Some(false),
            Err(_) => None,
        };
        let worktree_git_dir = match fs::symlink_metadata(&binding.worktree_git_dir) {
            Ok(_) => Some(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Some(false),
            Err(_) => None,
        };
        if worktree_path == Some(true) || worktree_git_dir == Some(true) {
            "identity_conflict"
        } else if worktree_path.is_none() || worktree_git_dir.is_none() || observation_unknown {
            "unknown"
        } else {
            "removed"
        }
    };
    Ok(OrdinaryCleanupObservation {
        branch: branch.into(),
        worktree: worktree.into(),
    })
}

pub fn record_ordinary_cleanup_with_runtime(
    root: &Path,
    work_item_id: &str,
    runtime: &RuntimeContext,
) -> Result<OrdinaryCleanupReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let _lifecycle_lock = acquire_lifecycle_lock(&root, work_item_id)?;
    let contract_path = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&contract_path)?;
    if effective_resource_context(&root, work_item_id, &contract)?.is_some() {
        return Err(ObserverError::State {
            path: contract_path,
            message: "ordinary post-close cleanup is not available for provider-bound Work Items"
                .into(),
        });
    }
    let close_path = root
        .join(".ai/decisions")
        .join(format!("{work_item_id}.close.json"));
    let decision = read_json(&close_path)?;
    let binding = ordinary_cleanup_binding_from_decision(
        &root,
        work_item_id,
        &contract.repository_id,
        &decision,
    )?
    .ok_or_else(|| ObserverError::State {
        path: close_path.clone(),
        message: "historical close lacks a Runtime-owned ordinary cleanup binding; cleanup is unsupported"
            .into(),
    })?;
    if !close_decision_is_valid_for_status(&root, work_item_id, &contract.repository_id) {
        return Err(ObserverError::State {
            path: close_path,
            message: "ordinary cleanup requires a valid closed decision".into(),
        });
    }
    let binding_value = serde_json::to_value(&binding).map_err(|error| ObserverError::State {
        path: root.join(".ai/decisions"),
        message: error.to_string(),
    })?;
    let binding_digest =
        cockpit_protocol::digest_json(&binding_value).map_err(|error| ObserverError::State {
            path: root.join(".ai/decisions"),
            message: error.to_string(),
        })?;
    let predecessor =
        ordinary_cleanup_receipt_head(&root, work_item_id, &binding, &binding_digest)?;
    let observation = observe_bound_ordinary_resources(&root, &binding)?;
    let mut failure_codes = Vec::new();
    if observation.branch != "removed" {
        failure_codes.push(format!("local_branch_{}", observation.branch));
    }
    if observation.worktree != "removed" {
        failure_codes.push(format!("worktree_{}", observation.worktree));
    }
    let state = if failure_codes.is_empty() {
        "verified"
    } else {
        "failed"
    };
    let sequence = predecessor
        .as_ref()
        .map_or(1, |head| head.receipt.sequence + 1);
    let observed_at = now();
    let operation_id = Digest::sha256_bytes(
        format!(
            "ordinary-cleanup-v1\0{}\0{work_item_id}\0{sequence}\0{observed_at}",
            binding.repository_id
        )
        .as_bytes(),
    )
    .to_string();
    let receipt = OrdinaryCleanupReceipt {
        schema_version: 1,
        operation_id,
        repository_id: binding.repository_id.clone(),
        work_item_id: work_item_id.into(),
        contract_digest: binding.contract_digest.clone(),
        binding_digest,
        branch_ref: binding.branch_ref.clone(),
        head_revision: binding.head_revision.clone(),
        worktree_id: binding.worktree_id.clone(),
        sequence,
        predecessor_receipt_digest: predecessor.map(|head| head.digest),
        runtime_version: runtime.runtime_version.clone(),
        runtime_digest: runtime.runtime_digest.clone(),
        observed_at,
        observation,
        result: OrdinaryCleanupResult {
            state: state.into(),
            failure_codes,
        },
    };
    let value = serde_json::to_value(&receipt).map_err(|error| ObserverError::State {
        path: root.join(".ai/decisions"),
        message: error.to_string(),
    })?;
    let digest = cockpit_protocol::digest_json(&value).map_err(|error| ObserverError::State {
        path: root.join(".ai/decisions"),
        message: error.to_string(),
    })?;
    let digest_text = digest.to_string();
    let digest_suffix = digest_text.strip_prefix("sha256:").unwrap_or(&digest_text);
    let path = root.join(".ai/decisions").join(format!(
        "{work_item_id}.cleanup.{sequence:06}.{digest_suffix}.json"
    ));
    if fs::symlink_metadata(&path).is_ok() {
        return Err(ObserverError::State {
            path,
            message: "ordinary cleanup receipt already exists".into(),
        });
    }
    atomic_json(&path, &value)?;
    Ok(receipt)
}

pub(crate) fn resource_cleanup_completion_state(
    root: &Path,
    work_item_id: &str,
    contract: &Contract,
    close_decision_valid: bool,
    runtime: &RuntimeContext,
) -> String {
    let resource_bound = match effective_resource_context(root, work_item_id, contract) {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(_) => return "unknown".into(),
    };
    if resource_bound {
        return match verify_resource_finalization_internal(root, work_item_id, Some(runtime)) {
            Ok(value) if matches!(value["disposition"].as_str(), Some("deleted" | "abandoned")) => {
                "verified".into()
            }
            Ok(_) => "pending".into(),
            Err(_) if resource_finalization_decision_path(root, work_item_id).exists() => {
                "failed".into()
            }
            Err(_) => "pending".into(),
        };
    }
    if !close_decision_valid {
        return "not_started".into();
    }
    let path = root
        .join(".ai/decisions")
        .join(format!("{work_item_id}.close.json"));
    let Ok(decision) = read_json(&path) else {
        return "unknown".into();
    };
    match ordinary_cleanup_binding_from_decision(
        root,
        work_item_id,
        &contract.repository_id,
        &decision,
    ) {
        Ok(Some(binding)) => {
            let Ok(binding_value) = serde_json::to_value(&binding) else {
                return "unknown".into();
            };
            let Ok(binding_digest) = cockpit_protocol::digest_json(&binding_value) else {
                return "unknown".into();
            };
            match ordinary_cleanup_receipt_head(root, work_item_id, &binding, &binding_digest) {
                Ok(Some(head)) => head.receipt.result.state,
                Ok(None) => "pending".into(),
                Err(_) => "unknown".into(),
            }
        }
        Ok(None) => "unsupported".into(),
        Err(_) => "unknown".into(),
    }
}

#[cfg(test)]
mod post_merge_sync_tests {
    use super::{local_resources_deleted, validate_post_merge_sync_local_state};
    use crate::{
        RepositoryVerificationPolicy, RepositoryVerificationRequest,
        archive_work_item_with_runtime, checkpoint_work_item, finish_work_item_with_runtime,
        preflight_work_item_with_runtime, record_verification_with_runtime,
        run_repository_verification, start_work_item_with_options,
    };
    use cockpit_core::Digest;
    use cockpit_protocol::{
        AssuranceLevel, DelegatedEvidence, DelegatedEvidenceObservationSelector, EvidenceValidity,
        PostMergeSyncEvidence, PostMergeSyncEvidenceObservationSelectors,
        PostMergeSyncRemoteBranchState, ResourceFinalizationBranchIdentity,
        ResourceFinalizationBranchState, ResourceFinalizationContext,
        ResourceFinalizationDisposition, ResourceFinalizationPullRequestIdentity,
        ResourceFinalizationPullRequestState, ResourceFinalizationReceipt,
        ResourceFinalizationResult, ResourceFinalizationState,
        ResourceFinalizationTransitionReceipt, ResourceFinalizationWorktreeIdentity,
        ResourceFinalizationWorktreeState, RuntimeContext,
    };
    use std::path::{Path, PathBuf};
    use std::process::Command;

    fn git(root: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .expect("git command starts");
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("git output is UTF-8")
            .trim()
            .to_owned()
    }

    fn real_combined_checkout() -> super::HistoricalGitCheckout {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let checkout =
            super::materialize_git_revision(&source, "d1a731570fb0af2ee5142e13b5d7cf9a7d4b0b50")
                .unwrap();
        let root = &checkout.repository;
        git(root, &["config", "user.name", "Fixture"]);
        git(root, &["config", "user.email", "fixture@example.invalid"]);
        git(
            root,
            &[
                "merge",
                "--no-ff",
                "-q",
                "d18be4f8624d08f91fe1c961dfed287cfe9543df",
                "-m",
                "synthetic fixture sync merge, never a provider observation",
            ],
        );
        let merge = git(root, &["rev-parse", "HEAD"]);
        git(root, &["update-ref", "refs/remotes/origin/main", &merge]);
        checkout
    }

    fn real_combined_sync(root: &Path) -> PostMergeSyncEvidence {
        let mut receipt = deleted_receipt(
            "codex/fixture",
            &root.join("removed"),
            &root.join(".git/worktrees/removed"),
            &root.join(".git"),
        );
        let sync = receipt.post_merge_sync.as_mut().unwrap();
        sync.lifecycle_layout = Some(cockpit_protocol::PostMergeLifecycleLayout::Combined);
        sync.documentation = Some(cockpit_protocol::PostMergeDocumentation::Absent);
        sync.source_pull_request.head_revision = "ba4350249313b2ffd95c5bae52a3ffff8e9597a9".into();
        sync.source_pull_request.base_revision = "ae1d0db0221c3afed297b05a0af34b4cd5f0b6c9".into();
        sync.source_pull_request.merge_commit =
            Some("ec249eadb1d4075b351db729d6c719d7905b3f16".into());
        sync.sync_pull_request.base_revision = "d1a731570fb0af2ee5142e13b5d7cf9a7d4b0b50".into();
        sync.sync_pull_request.head_revision = "d18be4f8624d08f91fe1c961dfed287cfe9543df".into();
        let merge = git(root, &["rev-parse", "HEAD"]);
        sync.sync_pull_request.merge_commit = Some(merge.clone());
        sync.current_main_revision = merge;
        sync.finish_revision = sync.sync_pull_request.head_revision.clone();
        sync.archive_revision = sync.finish_revision.clone();
        sync.docs_revision = sync.finish_revision.clone();
        sync.clone()
    }

    fn real_m1_checkout() -> super::HistoricalGitCheckout {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let checkout =
            super::materialize_git_revision(&source, "37cc4e540ce23a721382d9c6b510218dc5ba69c1")
                .unwrap();
        git(&checkout.repository, &["config", "user.name", "Fixture"]);
        git(
            &checkout.repository,
            &["config", "user.email", "fixture@example.invalid"],
        );
        git(
            &checkout.repository,
            &[
                "update-ref",
                "refs/remotes/origin/main",
                "37cc4e540ce23a721382d9c6b510218dc5ba69c1",
            ],
        );
        checkout
    }

    fn github_pull_request_origin_for_repository(
        identity: &ResourceFinalizationPullRequestIdentity,
    ) -> String {
        let (repository, number) =
            super::github_repository_from_pull_request_url(&identity.url).unwrap();
        format!("https://api.github.com/repos/{repository}/pulls/{number}")
    }

    fn github_pull_request_body_for_repository(
        identity: &ResourceFinalizationPullRequestIdentity,
        head_ref: &str,
    ) -> Vec<u8> {
        let (repository, _) =
            super::github_repository_from_pull_request_url(&identity.url).unwrap();
        serde_json::to_vec(&serde_json::json!({
            "number": identity.number,
            "html_url": identity.url,
            "state": "closed",
            "merged": true,
            "merge_commit_sha": identity.merge_commit,
            "head": {
                "ref": head_ref,
                "sha": identity.head_revision,
                "repo": {"full_name": repository}
            },
            "base": {
                "ref": identity.base_branch,
                "sha": identity.base_revision,
                "repo": {"full_name": repository}
            }
        }))
        .expect("GitHub pull request observation serializes")
    }

    fn commit_tree_with_parents(
        root: &Path,
        tree: &str,
        parents: &[&str],
        message: &str,
    ) -> String {
        let mut args = vec!["commit-tree", tree];
        for parent in parents {
            args.extend(["-p", parent]);
        }
        args.extend(["-m", message]);
        git(root, &args)
    }

    #[test]
    fn post_merge_layout_real_1078_combined_absent_advanced_base_preserves_old_attempts() {
        let fixture = real_combined_checkout();
        let root = &fixture.repository;
        let sync = real_combined_sync(root);
        super::validate_post_merge_layout_stages(root, "WI-1078-v1-0-1-post-merge-sync-finalization", &sync)
            .expect("actual combined artifacts and frozen prior attempts must validate against a synthetic clean advanced-base merge");
    }

    #[test]
    fn post_merge_layout_real_m1_accepts_provider_base_ancestor_of_actual_first_parent() {
        let fixture = real_m1_checkout();
        let root = &fixture.repository;
        let id = "WI-1078-v1-0-1-post-merge-sync-finalization";
        let mut sync = real_combined_sync(root);
        let m1 = "37cc4e540ce23a721382d9c6b510218dc5ba69c1";
        let actual_base = "508a0a0f47dcbb957aacb8b03220f6234a87549d";
        let provider_base = "d1a731570fb0af2ee5142e13b5d7cf9a7d4b0b50";
        let h1 = "d18be4f8624d08f91fe1c961dfed287cfe9543df";
        assert_eq!(git(root, &["rev-parse", "HEAD"]), m1);
        assert_eq!(
            super::commit_tree_and_parents(root, m1).unwrap().1,
            vec![actual_base, h1]
        );
        assert_ne!(provider_base, actual_base);
        assert!(super::is_ancestor(root, provider_base, actual_base).unwrap());

        let common_dir = super::git_common_dir(root).unwrap();
        let mut receipt = deleted_receipt(
            "codex/wi1078-post-merge-sync-finalization",
            &root.join("removed-worktree"),
            &common_dir.join("worktrees/removed-worktree"),
            &common_dir,
        );
        receipt.schema_version =
            cockpit_protocol::RESOURCE_FINALIZATION_LIFECYCLE_LAYOUT_SCHEMA_VERSION;
        receipt.work_item_id = id.into();
        receipt.repository_id = super::super::repository_id(root).to_string();
        receipt.branch.name = "codex/wi1078-post-merge-sync-finalization".into();
        receipt.branch.head_revision = h1.into();
        receipt.worktree.head_revision = h1.into();
        receipt.timestamp = (chrono::Utc::now() + chrono::Duration::minutes(2))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        sync.source_pull_request.number = 1032;
        sync.source_pull_request.url = "https://github.com/xinglun/ai-cockpit/pull/1032".into();
        sync.sync_pull_request.number = 1034;
        sync.sync_pull_request.url = "https://github.com/xinglun/ai-cockpit/pull/1034".into();
        sync.sync_pull_request.base_revision = provider_base.into();
        sync.sync_pull_request.head_revision = h1.into();
        sync.sync_pull_request.merge_commit = Some(m1.into());
        sync.current_main_revision = m1.into();
        sync.remote_branch_ref = "refs/heads/codex/wi1078-post-merge-sync-finalization".into();
        receipt.pull_request = sync.source_pull_request.clone();
        receipt.post_merge_sync = Some(sync.clone());

        let mut context = super::effective_resource_context(
            root,
            id,
            &super::archived_contract_digest(root, id).unwrap().0,
        )
        .unwrap()
        .expect("archived Contract binds the reviewed GitHub resource");
        let worktree_path = fixture.directory.join("synthetic-resource-worktree");
        context.worktree = worktree_path.display().to_string();
        assert!(
            worktree_path.starts_with(&fixture.directory),
            "positive lifecycle fixtures must keep worktree resources inside their TempDir"
        );
        assert!(
            !worktree_path.exists(),
            "the archived resource path must be free in this isolated fixture"
        );
        git(
            root,
            &[
                "worktree",
                "add",
                "-b",
                &receipt.branch.name,
                worktree_path.to_str().unwrap(),
                h1,
            ],
        );
        struct WorktreeCleanup {
            repository: PathBuf,
            path: PathBuf,
        }
        impl Drop for WorktreeCleanup {
            fn drop(&mut self) {
                let _ = Command::new("git")
                    .arg("-C")
                    .arg(&self.repository)
                    .args(["worktree", "remove", "--force"])
                    .arg(&self.path)
                    .output();
            }
        }
        let _worktree_cleanup = WorktreeCleanup {
            repository: root.to_path_buf(),
            path: worktree_path.clone(),
        };
        receipt.worktree.path = worktree_path.display().to_string();
        receipt.worktree.head_revision = h1.into();
        receipt.resource_context = Some(context);
        let (contract, contract_digest) = super::archived_contract_digest(root, id).unwrap();
        receipt.repository_id = super::super::repository_id(root).to_string();
        receipt.contract_digest = Some(contract_digest);
        receipt.contract_base_revision = Some(contract.base_revision.clone());
        receipt.runtime_version = "test-runtime".into();
        receipt.runtime_digest = Digest::sha256_bytes(b"test-runtime");
        receipt.result.disposition = ResourceFinalizationDisposition::Retained;
        receipt.after.branch = ResourceFinalizationBranchState::Present;
        receipt.after.worktree = ResourceFinalizationWorktreeState::Clean;

        let runtime = RuntimeContext {
            runtime_version: "test-runtime".into(),
            protocol_version: 1,
            runtime_digest: Digest::sha256_bytes(b"test-runtime"),
        };
        let sync = receipt.post_merge_sync.as_mut().unwrap();
        sync.remote_branch_state = PostMergeSyncRemoteBranchState::Present;
        sync.worktree_git_dir = git(&worktree_path, &["rev-parse", "--absolute-git-dir"]);
        let remote_ref = sync.remote_branch_ref.clone();
        let repository = "xinglun/ai-cockpit";
        let collect_time = (chrono::Utc::now() - chrono::Duration::seconds(10))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let imports = [
            (
                "source",
                sync.source_pull_request.url.clone(),
                github_pull_request_origin_for_repository(&sync.source_pull_request),
                github_pull_request_body_for_repository(
                    &sync.source_pull_request,
                    "codex/wi1078-post-merge-sync-finalization",
                ),
            ),
            (
                "sync",
                sync.sync_pull_request.url.clone(),
                github_pull_request_origin_for_repository(&sync.sync_pull_request),
                github_pull_request_body_for_repository(
                    &sync.sync_pull_request,
                    "codex/wi1078-post-merge-sync-finalization",
                ),
            ),
            (
                "remote",
                remote_ref.clone(),
                format!(
                    "https://api.github.com/repos/{repository}/git/ref/heads/{}",
                    remote_ref.strip_prefix("refs/heads/").unwrap()
                ),
                github_present_ref_body(&remote_ref, h1),
            ),
        ];
        let mut digests = Vec::new();
        for (name, subject, origin, raw) in imports {
            let evidence = DelegatedEvidence {
                provider: "github".into(),
                subject,
                origin,
                assurance: AssuranceLevel::ProviderVerified,
                collected_at: collect_time.clone(),
                digest: Digest::sha256_bytes(&raw),
                validity: EvidenceValidity::Valid,
                raw_evidence_ref: format!(".ai/evidence/external/wi1080-{name}.json"),
            };
            super::super::import_delegated_evidence(root, id, &evidence, &raw, &runtime)
                .expect("import exact provider fixture bytes");
            digests.push(evidence.digest);
        }
        sync.source_evidence_digest = digests[0].clone();
        sync.sync_evidence_digest = digests[1].clone();
        sync.remote_evidence_digest = digests[2].clone();

        cockpit_protocol::validate_resource_finalization_receipt_for(
            &receipt,
            &receipt.repository_id,
            id,
            receipt.contract_digest.as_ref(),
            receipt.resource_context.as_ref(),
        )
        .expect("the legitimate schema-v3 receipt must pass protocol identity validation");
        let mut tampered_identity = receipt.clone();
        tampered_identity
            .post_merge_sync
            .as_mut()
            .unwrap()
            .sync_pull_request
            .base_revision = actual_base.into();
        let tampered_sync = tampered_identity.post_merge_sync.as_ref().unwrap();
        assert!(
            super::validate_post_merge_sync_delegated_evidence(
                root,
                &tampered_identity,
                tampered_sync,
            )
            .is_err(),
            "receipt identity tampering must not override provider raw base bytes"
        );
        receipt.timestamp = (chrono::Utc::now() + chrono::Duration::minutes(2))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        super::validate_post_merge_sync_repository_evidence(root, &receipt).expect(
            "the same v3 receipt must pass provider-byte, Git-stage, and local-state validation",
        );
    }

    #[test]
    fn record_resource_finalization_accepts_tempdir_bound_initial_contract() {
        let fixture = tempfile::tempdir().expect("fixture directory");
        let root = fixture.path().join("repository");
        let worktree_path = fixture.path().join("synthetic-resource-worktree");
        std::fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["config", "user.name", "AI Cockpit Tests"]);
        git(&root, &["config", "user.email", "tests@example.invalid"]);
        git(&root, &["checkout", "-q", "-b", "main"]);
        std::fs::write(root.join("source.txt"), "synthetic repository\n").unwrap();
        git(&root, &["add", "source.txt"]);
        let head = commit(&root, "synthetic initial repository");

        let id = "WI-1080-resource-finalization-tempdir";
        let branch = "codex/wi1080-resource-finalization-tempdir";
        let pull_request = "https://github.com/xinglun/ai-cockpit/pull/9001";
        let context = ResourceFinalizationContext {
            branch: branch.into(),
            worktree: worktree_path.display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: pull_request.into(),
        };
        git(
            &root,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                branch,
                worktree_path.to_str().unwrap(),
                &head,
            ],
        );

        let repository_id = super::super::repository_id(&root).to_string();
        let contract = serde_json::json!({
            "protocolVersion": 1,
            "repositoryId": repository_id,
            "workItemId": id,
            "intent": "Exercise the public resource-finalization boundary with an isolated fixture.",
            "goal": "Bind a synthetic retained resource to a TempDir-owned initial Contract.",
            "scope": ["source.txt"],
            "outOfScope": [],
            "risk": "normal",
            "authority": "authorized",
            "acceptanceCriteria": ["The retained resource receipt matches the initial Contract."],
            "requiredEvidenceClasses": ["verification"],
            "baseRevision": head,
            "projectProfileDigest": Digest::sha256_bytes(b"synthetic project profile").to_string(),
            "repositorySnapshotDigest": Digest::sha256_bytes(b"synthetic repository snapshot").to_string(),
            "resourceContext": serde_json::to_value(&context).unwrap()
        });
        let contract: cockpit_protocol::Contract =
            serde_json::from_value(contract.clone()).expect("synthetic initial Contract parses");
        contract
            .validate()
            .expect("synthetic initial Contract is valid");
        let contract_bytes = serde_json::to_vec_pretty(&contract).unwrap();
        let contract_digest = Digest::sha256_bytes(&contract_bytes);

        let archive = root.join(".ai/work-items/archive");
        let active = root.join(".ai/work-items/active");
        std::fs::create_dir_all(&archive).unwrap();
        std::fs::create_dir_all(&active).unwrap();
        let contract_path = archive.join(format!("{id}.contract.json"));
        std::fs::write(&contract_path, &contract_bytes).unwrap();
        std::fs::write(active.join(format!("{id}.contract.json")), &contract_bytes).unwrap();
        let summary_bytes = serde_json::to_vec(&serde_json::json!({"workItemId": id})).unwrap();
        let outcome_bytes = serde_json::to_vec(&serde_json::json!({"workItemId": id})).unwrap();
        std::fs::write(archive.join(format!("{id}.summary.json")), &summary_bytes).unwrap();
        std::fs::write(archive.join(format!("{id}.outcome.json")), &outcome_bytes).unwrap();
        let manifest = serde_json::json!({
            "workItemId": id,
            "state": "archived",
            "files": {
                "contractDigest": contract_digest.to_string(),
                "summaryDigest": Digest::sha256_bytes(&summary_bytes).to_string(),
                "outcomeDigest": Digest::sha256_bytes(&outcome_bytes).to_string()
            }
        });
        std::fs::write(
            archive.join(format!("{id}.archive.json")),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let runtime = RuntimeContext {
            runtime_version: "test-runtime".into(),
            protocol_version: 1,
            runtime_digest: Digest::sha256_bytes(b"synthetic test runtime"),
        };
        let mut receipt = deleted_receipt(
            branch,
            &worktree_path,
            &worktree_path.join(".git"),
            &root.join(".git"),
        );
        receipt.schema_version = cockpit_protocol::RESOURCE_FINALIZATION_SCHEMA_VERSION;
        receipt.work_item_id = id.into();
        receipt.repository_id = repository_id;
        receipt.runtime_version = runtime.runtime_version.clone();
        receipt.runtime_digest = runtime.runtime_digest.clone();
        receipt.provider = "github".into();
        receipt.pull_request = ResourceFinalizationPullRequestIdentity {
            number: 9001,
            url: pull_request.into(),
            head_revision: head.clone(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            base_revision: head.clone(),
            merge_commit: Some(head.clone()),
        };
        receipt.branch = ResourceFinalizationBranchIdentity {
            name: branch.into(),
            remote: "origin".into(),
            head_revision: head.clone(),
        };
        receipt.worktree = ResourceFinalizationWorktreeIdentity {
            worktree_id: "synthetic-tempdir-worktree".into(),
            path: worktree_path.display().to_string(),
            branch: branch.into(),
            head_revision: head,
        };
        receipt.before = ResourceFinalizationState {
            pull_request: ResourceFinalizationPullRequestState::Merged,
            branch: ResourceFinalizationBranchState::Present,
            worktree: ResourceFinalizationWorktreeState::Clean,
        };
        receipt.after = receipt.before.clone();
        receipt.result.disposition = ResourceFinalizationDisposition::Retained;
        receipt.contract_digest = Some(contract_digest.clone());
        receipt.contract_base_revision = Some(contract.base_revision.clone());
        receipt.resource_context = Some(context.clone());
        receipt.post_merge_sync = None;
        receipt.timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

        cockpit_protocol::validate_resource_finalization_receipt_for(
            &receipt,
            &contract.repository_id,
            id,
            Some(&contract_digest),
            Some(&context),
        )
        .expect("the synthetic receipt must pass protocol identity validation");
        let receipt_path = fixture.path().join("retained-resource-receipt.json");
        std::fs::write(&receipt_path, serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
        let result = super::record_resource_finalization(&root, id, &receipt_path, &runtime)
            .expect("the public repository finalization entry point accepts the TempDir fixture");
        assert_eq!(result["state"], "recorded");
    }

    #[test]
    fn record_resource_finalization_accepts_tempdir_bound_schema_v3_history() {
        use cockpit_protocol::PROTOCOL_VERSION;

        let fixture = tempfile::tempdir().expect("fixture directory");
        let root = fixture.path().join("repository");
        let worktree_path = fixture.path().join("synthetic-resource-worktree");
        std::fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "--quiet", "-b", "main"]);
        git(&root, &["config", "core.autocrlf", "false"]);
        git(&root, &["config", "user.name", "AI Cockpit Tests"]);
        git(&root, &["config", "user.email", "tests@example.invalid"]);
        std::fs::write(root.join("seed.txt"), "synthetic repository\n").unwrap();
        super::super::attach(&root).expect("attach synthetic repository");
        git(&root, &["add", "--all"]);
        git(
            &root,
            &["commit", "--quiet", "-m", "synthetic repository baseline"],
        );
        let base = git(&root, &["rev-parse", "HEAD"]);

        let id = "WI-1080-resource-finalization-v3-tempdir";
        let branch = "codex/wi1080-resource-finalization-v3-tempdir";
        let source_pr_url = "https://github.com/xinglun/ai-cockpit/pull/9001";
        let sync_pr_url = "https://github.com/xinglun/ai-cockpit/pull/9002";
        let context = ResourceFinalizationContext {
            branch: branch.into(),
            worktree: worktree_path.display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: source_pr_url.into(),
        };
        git(&root, &["checkout", "--quiet", "-b", branch]);
        start_work_item_with_options(
            &root,
            id,
            "exercise schema-v3 resource finalization end to end",
            "validate one TempDir-bound v3 receipt through the public recorder",
            &[".ai/**".into(), "seed.txt".into()],
            &crate::WorkItemStartOptions {
                authority: "authorized".into(),
                acceptance_criteria: vec![
                    "the same v3 receipt passes both validation layers".into(),
                ],
                required_evidence_classes: vec!["verification".into()],
                ..crate::WorkItemStartOptions::default()
            },
        )
        .expect("start synthetic lifecycle");
        super::plan_resource_finalization(&root, id, &context)
            .expect("bind the TempDir-owned resource before verification");
        let runtime = RuntimeContext {
            runtime_version: "test-runtime".into(),
            protocol_version: PROTOCOL_VERSION,
            runtime_digest: Digest::sha256_bytes(b"synthetic v3 test runtime"),
        };
        let contract_path = root
            .join(".ai/work-items/active")
            .join(format!("{id}.contract.json"));
        preflight_work_item_with_runtime(&root, &contract_path, &runtime)
            .expect("record initial preflight");
        checkpoint_work_item(&root, id).expect("checkpoint synthetic source");
        let h0 = commit(&root, "source PR head with resource Contract");

        // M0 is the source PR merge on main. The subsequent lifecycle commits
        // advance the provider head from H0 to H1 while preserving H0 as their
        // merge base with M0.
        git(&root, &["checkout", "--quiet", "main"]);
        git(
            &root,
            &[
                "merge",
                "--quiet",
                "--no-ff",
                branch,
                "-m",
                "synthetic source PR merge",
            ],
        );
        let m0 = git(&root, &["rev-parse", "HEAD"]);
        git(
            &root,
            &[
                "commit",
                "--quiet",
                "--allow-empty",
                "-m",
                "synthetic provider-base advancement",
            ],
        );
        let provider_base = m0.clone();
        let actual_base = git(&root, &["rev-parse", "HEAD"]);
        git(&root, &["checkout", "--quiet", branch]);
        let request = RepositoryVerificationRequest {
            node_id: "resource-finalization-v3-tempdir".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec![".ai/**".into(), "seed.txt".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: Some(id.into()),
            timeout_seconds: None,
            policy: RepositoryVerificationPolicy::NeverReuse,
        };
        let execution =
            run_repository_verification(&root, &request).expect("run synthetic verification");
        let verification_receipt =
            serde_json::to_value(&execution.receipt).expect("verification receipt serializes");
        super::super::persist_verification_attempt(
            &root,
            id,
            &[request],
            &execution.final_snapshot,
            &runtime,
            "execution_completed",
            None,
            Some(&verification_receipt),
        )
        .expect("persist immutable successful verification attempt");
        record_verification_with_runtime(
            &root,
            id,
            &verification_receipt,
            &runtime,
            &execution.final_snapshot,
        )
        .expect("record synthetic verification");
        git(
            &root,
            &[
                "add",
                "--all",
                "--",
                &format!(".ai/evidence/{id}.verification-attempt.*"),
            ],
        );
        git(
            &root,
            &[
                "commit",
                "--quiet",
                "-m",
                "append immutable verification attempt",
            ],
        );
        finish_work_item_with_runtime(&root, id, &runtime).expect("finish synthetic lifecycle");
        let finish = commit(&root, "finish verified synthetic Work Item");
        archive_work_item_with_runtime(&root, id, &runtime).expect("archive synthetic lifecycle");
        let h1 = commit(&root, "archive synthetic Work Item");

        git(&root, &["checkout", "--quiet", "main"]);
        git(
            &root,
            &[
                "merge",
                "--quiet",
                "--no-ff",
                branch,
                "-m",
                "synthetic schema-v3 M1",
            ],
        );
        let m1 = git(&root, &["rev-parse", "HEAD"]);
        assert_eq!(
            git(&root, &["rev-list", "--parents", "-n", "1", &m1])
                .split_whitespace()
                .skip(1)
                .collect::<Vec<_>>(),
            vec![actual_base.as_str(), h1.as_str()],
            "the fixture M1 must bind its actual first parent and declared H1"
        );
        git(&root, &["update-ref", "refs/remotes/origin/main", &m1]);
        git(
            &root,
            &["update-ref", &format!("refs/remotes/origin/{branch}"), &h1],
        );
        git(
            &root,
            &[
                "worktree",
                "add",
                "--quiet",
                worktree_path.to_str().unwrap(),
                branch,
            ],
        );

        let repository_id = super::super::repository_id(&root).to_string();
        let common_dir = super::git_common_dir(&root).expect("repository common dir");
        let mut receipt = deleted_receipt(branch, &worktree_path, Path::new("unused"), &common_dir);
        let source_identity = ResourceFinalizationPullRequestIdentity {
            number: 9001,
            url: source_pr_url.into(),
            head_revision: h0.clone(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            base_revision: base.clone(),
            merge_commit: Some(m0.clone()),
        };
        let sync_identity = ResourceFinalizationPullRequestIdentity {
            number: 9002,
            url: sync_pr_url.into(),
            head_revision: h1.clone(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            base_revision: provider_base.clone(),
            merge_commit: Some(m1.clone()),
        };
        receipt.schema_version =
            cockpit_protocol::RESOURCE_FINALIZATION_LIFECYCLE_LAYOUT_SCHEMA_VERSION;
        receipt.work_item_id = id.into();
        receipt.repository_id = repository_id.clone();
        receipt.runtime_version = runtime.runtime_version.clone();
        receipt.runtime_digest = runtime.runtime_digest.clone();
        receipt.pull_request = source_identity.clone();
        receipt.branch = ResourceFinalizationBranchIdentity {
            name: branch.into(),
            remote: "origin".into(),
            head_revision: h1.clone(),
        };
        receipt.worktree = ResourceFinalizationWorktreeIdentity {
            worktree_id: "synthetic-v3-tempdir-worktree".into(),
            path: worktree_path.display().to_string(),
            branch: branch.into(),
            head_revision: h1.clone(),
        };
        receipt.before = ResourceFinalizationState {
            pull_request: ResourceFinalizationPullRequestState::Merged,
            branch: ResourceFinalizationBranchState::Present,
            worktree: ResourceFinalizationWorktreeState::Clean,
        };
        receipt.after = receipt.before.clone();
        receipt.result.disposition = ResourceFinalizationDisposition::Retained;
        receipt.contract_digest = Some(
            super::archived_contract_digest(&root, id)
                .expect("archived Contract digest")
                .1,
        );
        let (contract, _) = super::archived_contract_digest(&root, id).unwrap();
        receipt.contract_base_revision = Some(contract.base_revision.clone());
        receipt.resource_context = Some(context.clone());
        receipt.timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

        let sync = receipt.post_merge_sync.as_mut().unwrap();
        sync.lifecycle_layout = Some(cockpit_protocol::PostMergeLifecycleLayout::Separate);
        sync.documentation = Some(cockpit_protocol::PostMergeDocumentation::Absent);
        sync.source_pull_request = source_identity.clone();
        sync.sync_pull_request = sync_identity.clone();
        sync.finish_revision = finish;
        sync.archive_revision = h1.clone();
        sync.docs_revision = h1.clone();
        sync.current_main_revision = m1.clone();
        sync.remote_branch_ref = format!("refs/heads/{branch}");
        sync.remote_branch_state = PostMergeSyncRemoteBranchState::Present;
        sync.git_common_dir = common_dir.display().to_string();
        sync.worktree_git_dir = git(&worktree_path, &["rev-parse", "--absolute-git-dir"]);

        let collect_time = (chrono::Utc::now() - chrono::Duration::seconds(10))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let imports = [
            (
                "source",
                source_identity.clone(),
                github_pull_request_body_for_repository(&source_identity, branch),
            ),
            (
                "sync",
                sync_identity.clone(),
                github_pull_request_body_for_repository(&sync_identity, branch),
            ),
        ];
        let mut digests = Vec::new();
        for (name, identity, raw) in imports {
            let evidence = DelegatedEvidence {
                provider: "github".into(),
                subject: identity.url.clone(),
                origin: github_pull_request_origin_for_repository(&identity),
                assurance: AssuranceLevel::ProviderVerified,
                collected_at: collect_time.clone(),
                digest: Digest::sha256_bytes(&raw),
                validity: EvidenceValidity::Valid,
                raw_evidence_ref: format!(".ai/evidence/external/{id}-{name}.json"),
            };
            super::super::import_delegated_evidence(&root, id, &evidence, &raw, &runtime)
                .expect("import exact synthetic provider bytes");
            digests.push(evidence.digest);
        }
        let remote_ref = sync.remote_branch_ref.clone();
        let remote_raw = github_present_ref_body(&remote_ref, &h1);
        let remote_evidence = DelegatedEvidence {
            provider: "github".into(),
            subject: remote_ref.clone(),
            origin: format!(
                "https://api.github.com/repos/xinglun/ai-cockpit/git/ref/heads/{branch}"
            ),
            assurance: AssuranceLevel::ProviderVerified,
            collected_at: collect_time,
            digest: Digest::sha256_bytes(&remote_raw),
            validity: EvidenceValidity::Valid,
            raw_evidence_ref: format!(".ai/evidence/external/{id}-remote.json"),
        };
        super::super::import_delegated_evidence(&root, id, &remote_evidence, &remote_raw, &runtime)
            .expect("import exact synthetic remote-ref bytes");
        sync.source_evidence_digest = digests[0].clone();
        sync.sync_evidence_digest = digests[1].clone();
        sync.remote_evidence_digest = remote_evidence.digest;
        receipt.timestamp = (chrono::Utc::now() + chrono::Duration::minutes(2))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

        let (_, contract_digest) = super::archived_contract_digest(&root, id).unwrap();
        cockpit_protocol::validate_resource_finalization_receipt_for(
            &receipt,
            &repository_id,
            id,
            Some(&contract_digest),
            Some(&context),
        )
        .expect("the exact synthetic v3 receipt passes protocol validation");
        let receipt_path = fixture.path().join("retained-v3-resource-receipt.json");
        std::fs::write(&receipt_path, serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
        let result = super::record_resource_finalization(&root, id, &receipt_path, &runtime)
            .expect("the same v3 receipt passes the public repository recorder");
        assert_eq!(result["state"], "recorded");
        let recorded: serde_json::Value =
            super::read_json(&root.join(result["path"].as_str().unwrap()))
                .expect("public recorder persists the accepted receipt");
        assert_eq!(recorded, serde_json::to_value(&receipt).unwrap());
        assert_eq!(
            recorded["postMergeSync"]["syncPullRequest"]["baseRevision"], provider_base,
            "provider raw base remains distinct from the actual M1 first parent"
        );
    }

    #[test]
    fn post_merge_layout_v3_rejects_wrong_m1_parents_non_ancestor_base_and_tampered_tree() {
        let fixture = real_m1_checkout();
        let root = &fixture.repository;
        let id = "WI-1078-v1-0-1-post-merge-sync-finalization";
        let baseline = real_combined_sync(root);
        let m1 = "37cc4e540ce23a721382d9c6b510218dc5ba69c1";
        let actual_base = "508a0a0f47dcbb957aacb8b03220f6234a87549d";
        let provider_base = "d1a731570fb0af2ee5142e13b5d7cf9a7d4b0b50";
        let h1 = "d18be4f8624d08f91fe1c961dfed287cfe9543df";
        let h0 = "ba4350249313b2ffd95c5bae52a3ffff8e9597a9";
        let source_merge = "ec249eadb1d4075b351db729d6c719d7905b3f16";
        let m1_tree = git(root, &["rev-parse", &format!("{m1}^{{tree}}")]);

        let forked_actual_base_tree = git(root, &["merge-tree", "--write-tree", provider_base, h1]);
        let forked_actual_base = commit_tree_with_parents(
            root,
            forked_actual_base_tree.lines().next().unwrap(),
            &[provider_base, h1],
            "synthetic actual base that already contains the sync head",
        );
        let forked_m1_tree = git(
            root,
            &["merge-tree", "--write-tree", &forked_actual_base, h1],
        );
        let forked_m1 = commit_tree_with_parents(
            root,
            forked_m1_tree.lines().next().unwrap(),
            &[&forked_actual_base, h1],
            "synthetic M1 whose actual base already contains H1",
        );
        assert_eq!(git(root, &["merge-base", &forked_actual_base, h1],), h1);
        let mut actual_base_fork = baseline.clone();
        actual_base_fork.sync_pull_request.merge_commit = Some(forked_m1.clone());
        actual_base_fork.current_main_revision = forked_m1;
        git(
            root,
            &[
                "update-ref",
                "refs/remotes/origin/main",
                actual_base_fork
                    .sync_pull_request
                    .merge_commit
                    .as_deref()
                    .unwrap(),
            ],
        );
        let error =
            super::validate_post_merge_layout_stages(root, id, &actual_base_fork).unwrap_err();
        assert!(
            format!("{error:?}")
                .contains("actual sync base must preserve the original source-head fork"),
            "actual first parent must remain forked at H0, got {error:?}"
        );

        for (case, parents, expected) in [
            ("one parent", vec![actual_base], "exactly two parents"),
            (
                "extra parent",
                vec![actual_base, h1, h0],
                "exactly two parents",
            ),
            (
                "wrong parent order",
                vec![h1, actual_base],
                "second parent does not match",
            ),
            (
                "wrong PR head parent",
                vec![actual_base, h0],
                "second parent does not match",
            ),
        ] {
            let candidate =
                commit_tree_with_parents(root, &m1_tree, &parents, &format!("synthetic {case} M1"));
            let mut sync = baseline.clone();
            sync.sync_pull_request.merge_commit = Some(candidate.clone());
            sync.current_main_revision = candidate;
            let error = super::validate_post_merge_layout_stages(root, id, &sync).unwrap_err();
            assert!(
                format!("{error:?}").contains(expected),
                "must reject {case} at the M1 parent boundary, got {error:?}"
            );
        }

        let source_merge_tree = git(root, &["rev-parse", &format!("{source_merge}^{{tree}}")]);
        let non_ancestor_base = commit_tree_with_parents(
            root,
            &source_merge_tree,
            &[source_merge],
            "synthetic provider base outside actual M1 ancestry",
        );
        let mut non_ancestor = baseline.clone();
        non_ancestor.sync_pull_request.base_revision = non_ancestor_base;
        let error = super::validate_post_merge_layout_stages(root, id, &non_ancestor).unwrap_err();
        assert!(
            format!("{error:?}").contains("not an ancestor of the actual merge first parent"),
            "provider base must be an ancestor of actual M1 first parent, got {error:?}"
        );

        std::fs::write(root.join("wi1080-unreviewed.txt"), "unreviewed\n").unwrap();
        git(root, &["add", "wi1080-unreviewed.txt"]);
        let tampered_tree = git(root, &["write-tree"]);
        git(root, &["reset", "--hard", m1]);
        let tampered_m1 = commit_tree_with_parents(
            root,
            &tampered_tree,
            &[actual_base, h1],
            "synthetic M1 with unreviewed tree content",
        );
        let mut tampered = baseline.clone();
        tampered.sync_pull_request.merge_commit = Some(tampered_m1.clone());
        tampered.current_main_revision = tampered_m1;
        let error = super::validate_post_merge_layout_stages(root, id, &tampered).unwrap_err();
        assert!(
            format!("{error:?}").contains("actual merge tree contains unproven content"),
            "M1 tree must equal the clean merge of its actual parents, got {error:?}"
        );

        git(root, &["checkout", "-q", "--detach", actual_base]);
        std::fs::write(root.join("wi1080-conflict.txt"), "base side\n").unwrap();
        git(root, &["add", "wi1080-conflict.txt"]);
        let conflicting_base = commit(root, "synthetic merge base conflict side");
        git(root, &["checkout", "-q", "--detach", h0]);
        std::fs::write(root.join("wi1080-conflict.txt"), "head side\n").unwrap();
        git(root, &["add", "wi1080-conflict.txt"]);
        let conflicting_head = commit(root, "synthetic merge head conflict side");
        let conflicted_m1 = commit_tree_with_parents(
            root,
            &m1_tree,
            &[&conflicting_base, &conflicting_head],
            "synthetic conflicted M1",
        );
        let mut conflicted = baseline;
        conflicted.sync_pull_request.head_revision = conflicting_head;
        conflicted.sync_pull_request.merge_commit = Some(conflicted_m1.clone());
        conflicted.current_main_revision = conflicted_m1;
        let error = super::validate_post_merge_layout_stages(root, id, &conflicted).unwrap_err();
        assert!(
            format!("{error:?}").contains("merge result is conflicted or cannot be proven"),
            "conflicted parent trees must be rejected, got {error:?}"
        );
        assert!(super::is_ancestor(root, provider_base, actual_base).unwrap());
    }

    #[test]
    fn post_merge_layout_v3_checks_absent_docs_at_actual_first_parent() {
        let fixture = real_m1_checkout();
        let root = &fixture.repository;
        let id = "WI-1078-v1-0-1-post-merge-sync-finalization";
        let mut sync = real_combined_sync(root);
        let provider_base = "d1a731570fb0af2ee5142e13b5d7cf9a7d4b0b50";
        let actual_base = "508a0a0f47dcbb957aacb8b03220f6234a87549d";
        let h1 = "d18be4f8624d08f91fe1c961dfed287cfe9543df";

        git(root, &["checkout", "-q", "--detach", actual_base]);
        let page = root.join(format!("docs/work-items/{id}.md"));
        std::fs::create_dir_all(page.parent().unwrap()).unwrap();
        std::fs::write(&page, "unexpected page at actual M1 base\n").unwrap();
        git(root, &["add", &format!("docs/work-items/{id}.md")]);
        let actual_base_with_docs = commit(root, "synthetic docs at actual M1 first parent");
        let merge_tree = git(
            root,
            &["merge-tree", "--write-tree", &actual_base_with_docs, h1],
        );
        let m1_with_docs = commit_tree_with_parents(
            root,
            merge_tree.lines().next().unwrap(),
            &[&actual_base_with_docs, h1],
            "synthetic clean M1 with docs at actual first parent",
        );
        git(
            root,
            &["update-ref", "refs/remotes/origin/main", &m1_with_docs],
        );
        sync.sync_pull_request.base_revision = provider_base.into();
        sync.sync_pull_request.merge_commit = Some(m1_with_docs.clone());
        sync.current_main_revision = m1_with_docs;
        assert!(super::is_ancestor(root, provider_base, &actual_base_with_docs).unwrap());
        let error = super::validate_post_merge_layout_stages(root, id, &sync).unwrap_err();
        assert!(
            format!("{error:?}").contains("absent projection has an existing Work Item page"),
            "documentation absence must include actual M1 first parent, got {error:?}"
        );
    }

    #[test]
    fn post_merge_layout_real_1076_separate_projected_semantics_remain_valid() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let fixture =
            super::materialize_git_revision(&source, "ae1d0db0221c3afed297b05a0af34b4cd5f0b6c9")
                .unwrap();
        let root = &fixture.repository;
        let mut sync = real_combined_sync(root);
        sync.lifecycle_layout = Some(cockpit_protocol::PostMergeLifecycleLayout::Separate);
        sync.documentation = Some(cockpit_protocol::PostMergeDocumentation::Projected);
        sync.source_pull_request.head_revision = "b0e98ae9efe54cde951bb8f547762736411c1628".into();
        sync.source_pull_request.base_revision = "095e185b710b0c0016c655d3792f1b3438a62bac".into();
        sync.source_pull_request.merge_commit =
            Some("686a71a07cc0bd6f3d76ab7261c829067db7ee26".into());
        sync.sync_pull_request.head_revision = "0f1d54b334104a67eb383f5946ce2305b7b0ba94".into();
        sync.sync_pull_request.base_revision = "686a71a07cc0bd6f3d76ab7261c829067db7ee26".into();
        sync.sync_pull_request.merge_commit =
            Some("ae1d0db0221c3afed297b05a0af34b4cd5f0b6c9".into());
        sync.current_main_revision = sync.sync_pull_request.merge_commit.clone().unwrap();
        sync.finish_revision = "d651f93d4a8bcbe9413d055d501ceae6f7d0132e".into();
        sync.archive_revision = "700f6cd5dcb81833aa63d38913908c5543865a8d".into();
        sync.docs_revision = sync.sync_pull_request.head_revision.clone();
        git(
            root,
            &[
                "update-ref",
                "refs/remotes/origin/main",
                &sync.current_main_revision,
            ],
        );
        super::validate_post_merge_layout_stages(
            root,
            "WI-1076-governance-recovered-predecessor-projection",
            &sync,
        )
        .expect("real separate lifecycle and pending-close projection remain supported");
    }

    #[test]
    fn post_merge_layout_merge_proof_rejects_wrong_base_extra_content_and_conflict() {
        let fixture = post_merge_history_fixture();
        let root = fixture.path();
        let baseline = git(root, &["rev-parse", "HEAD"]);
        std::fs::write(root.join("shared.txt"), "baseline\n").unwrap();
        let fork = commit(root, "fork");
        git(root, &["checkout", "-q", "-b", "feature"]);
        std::fs::write(root.join("feature.txt"), "feature\n").unwrap();
        let head = commit(root, "feature");
        git(root, &["checkout", "-q", "--detach", &fork]);
        std::fs::write(root.join("advance.txt"), "main advanced\n").unwrap();
        let base = commit(root, "base advanced");
        git(
            root,
            &["merge", "-q", "--no-ff", &head, "-m", "clean merge"],
        );
        let merge = git(root, &["rev-parse", "HEAD"]);
        super::prove_clean_merge(root, &base, &head, &merge).unwrap();
        assert!(super::prove_clean_merge(root, &baseline, &head, &merge).is_err());
        std::fs::write(root.join("private.txt"), "unreviewed content\n").unwrap();
        git(root, &["add", "private.txt"]);
        git(root, &["commit", "-q", "--amend", "--no-edit"]);
        let tampered = git(root, &["rev-parse", "HEAD"]);
        assert!(super::prove_clean_merge(root, &base, &head, &tampered).is_err());
        git(root, &["checkout", "-q", "--detach", &fork]);
        std::fs::write(root.join("shared.txt"), "base conflict\n").unwrap();
        let conflicting_base = commit(root, "conflicting base");
        git(root, &["checkout", "-q", "--detach", &head]);
        std::fs::write(root.join("shared.txt"), "feature conflict\n").unwrap();
        let conflicting_head = commit(root, "conflicting head");
        let tree = git(root, &["rev-parse", &format!("{merge}^{{tree}}")]);
        let resolved = git(
            root,
            &[
                "commit-tree",
                &tree,
                "-p",
                &conflicting_base,
                "-p",
                &conflicting_head,
                "-m",
                "synthetic resolved conflict fixture",
            ],
        );
        assert!(
            super::prove_clean_merge(root, &conflicting_base, &conflicting_head, &resolved)
                .is_err()
        );
    }

    #[test]
    fn post_merge_layout_absent_projection_rejects_existing_pages_parity_and_active_refs() {
        let fixture = post_merge_history_fixture();
        let root = fixture.path();
        let id = "WI-9997-layout";
        let baseline = git(root, &["rev-parse", "HEAD"]);
        super::validate_absent_projection(root, id, &baseline).unwrap();
        for path in [
            "docs/work-items/WI-9997-layout.md",
            "docs/reference/reference-parity.md",
            "docs/reference/example.md",
        ] {
            git(root, &["checkout", "-q", "--detach", &baseline]);
            let absolute = root.join(path);
            std::fs::create_dir_all(absolute.parent().unwrap()).unwrap();
            std::fs::write(
                &absolute,
                ".ai/work-items/active/WI-9997-layout.contract.json\n| WI-9997 | pending |\n",
            )
            .unwrap();
            let revision = commit(root, "existing documentation, never absent");
            assert!(super::validate_absent_projection(root, id, &revision).is_err());
            std::fs::remove_file(&absolute).unwrap();
            let deleted = commit(root, "cannot disguise old projection as absent");
            super::validate_absent_projection(root, id, &deleted).unwrap();
            assert!(super::validate_absent_projection(root, id, &revision).is_err());
        }
    }

    #[test]
    fn post_merge_layout_rejects_resealed_artifacts_foreign_paths_and_history_loss() {
        let fixture = real_combined_checkout();
        let root = &fixture.repository;
        let original = real_combined_sync(root);
        let id = "WI-1078-v1-0-1-post-merge-sync-finalization";
        let archive = format!(".ai/work-items/archive/{id}");
        let source = original.source_pull_request.head_revision.clone();
        let original_lifecycle = original.finish_revision.clone();
        let attempts = git(
            root,
            &["ls-tree", "-r", "--name-only", &source, ".ai/evidence"],
        );
        let old_attempt = attempts
            .lines()
            .find(|p| p.starts_with(&format!(".ai/evidence/{id}.verification-attempt.")))
            .unwrap()
            .to_owned();
        for case in [
            "source",
            "foreign_wi",
            "unknown_artifact",
            "contract",
            "summary",
            "events",
            "report",
            "outcome",
            "deleted_attempt",
            "changed_attempt",
            "verification",
            "runtime_rebinding",
        ] {
            git(root, &["checkout", "-q", "--detach", &source]);
            git(root, &["read-tree", "--reset", "-u", &original_lifecycle]);
            let path = match case {
                "source" => "README.md".to_owned(),
                "foreign_wi" => ".ai/work-items/archive/WI-foreign.summary.json".to_owned(),
                "unknown_artifact" => format!("{archive}.unknown.json"),
                "contract" => format!("{archive}.contract.json"),
                "summary" => format!("{archive}.summary.json"),
                "events" => format!("{archive}.events.jsonl"),
                "report" => format!("{archive}.task-report.json"),
                "outcome" => format!("{archive}.outcome.json"),
                "deleted_attempt" | "changed_attempt" => old_attempt.clone(),
                _ => format!(".ai/evidence/{id}.verification.json"),
            };
            let absolute = root.join(&path);
            if case == "deleted_attempt" {
                std::fs::remove_file(&absolute).unwrap();
            } else if case == "events" {
                let text = std::fs::read_to_string(&absolute).unwrap();
                std::fs::write(
                    &absolute,
                    text.replace(
                        "Verification evidence passed;",
                        "Unproven release published;",
                    ),
                )
                .unwrap();
            } else if matches!(case, "source" | "foreign_wi" | "unknown_artifact") {
                std::fs::write(&absolute, "unreviewed content\n").unwrap();
            } else {
                let mut value: serde_json::Value =
                    serde_json::from_slice(&std::fs::read(&absolute).unwrap()).unwrap();
                match case {
                    "contract" => value["goal"] = "forged Contract goal".into(),
                    "summary" => value["checkpointCount"] = 2.into(),
                    "report" => {
                        value["sections"]["outcomeSummary"][0]["text"] = "forged report".into()
                    }
                    "outcome" => value["acceptanceResults"][0] = "forged acceptance".into(),
                    "verification" => value["passed"] = false.into(),
                    _ => {
                        value["runtimeDigest"] =
                            Digest::sha256_bytes(b"forged runtime").to_string().into()
                    }
                }
                std::fs::write(&absolute, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
            }
            git(root, &["add", "-f", "--", &path]);
            // Reseal the manifest to prove that self-consistency is insufficient.
            if path.starts_with(&archive) && !matches!(case, "unknown_artifact") {
                let manifest_path = root.join(format!("{archive}.archive.json"));
                let mut manifest: serde_json::Value =
                    serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
                let key = match case {
                    "contract" => "contractDigest",
                    "summary" => "summaryDigest",
                    "events" => "eventsDigest",
                    "report" => "taskReportDigest",
                    "outcome" => "outcomeDigest",
                    _ => unreachable!(),
                };
                manifest["files"][key] = Digest::sha256_bytes(&std::fs::read(&absolute).unwrap())
                    .to_string()
                    .into();
                std::fs::write(
                    &manifest_path,
                    serde_json::to_vec_pretty(&manifest).unwrap(),
                )
                .unwrap();
                git(
                    root,
                    &["add", "-f", "--", &format!("{archive}.archive.json")],
                );
            }
            let tree = git(root, &["write-tree"]);
            let lifecycle = git(
                root,
                &[
                    "commit-tree",
                    &tree,
                    "-p",
                    &source,
                    "-m",
                    "synthetic tampered lifecycle fixture",
                ],
            );
            let merge_tree = git(
                root,
                &[
                    "merge-tree",
                    "--write-tree",
                    &original.sync_pull_request.base_revision,
                    &lifecycle,
                ],
            );
            let merged = git(
                root,
                &[
                    "commit-tree",
                    merge_tree.lines().next().unwrap(),
                    "-p",
                    &original.sync_pull_request.base_revision,
                    "-p",
                    &lifecycle,
                    "-m",
                    "synthetic fixture merge",
                ],
            );
            git(root, &["reset", "--hard", &merged]);
            git(root, &["update-ref", "refs/remotes/origin/main", &merged]);
            let mut sync = original.clone();
            sync.finish_revision = lifecycle.clone();
            sync.archive_revision = lifecycle.clone();
            sync.docs_revision = lifecycle.clone();
            sync.sync_pull_request.head_revision = lifecycle;
            sync.sync_pull_request.merge_commit = Some(merged.clone());
            sync.current_main_revision = merged;
            assert!(
                super::validate_post_merge_layout_stages(root, id, &sync).is_err(),
                "must reject {case} even when the archive is resealed"
            );
        }
    }

    fn same_green_lifecycle_fixture(case: &str) -> (super::HistoricalGitCheckout, String, String) {
        let fixture = real_combined_checkout();
        let root = &fixture.repository;
        let id = "WI-1078-v1-0-1-post-merge-sync-finalization";
        let active = format!(".ai/work-items/active/{id}");
        let archive = format!(".ai/work-items/archive/{id}");
        let observed =
            super::git_json(root, "d18be4f8", &format!("{archive}.summary.json")).unwrap();
        git(root, &["checkout", "-q", "--detach", "ba435024"]);
        let summary_path = root.join(format!("{active}.summary.json"));
        let mut summary: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&summary_path).unwrap()).unwrap();
        for key in ["preflightState", "preflightDecisionDigest", "preflightAt"] {
            summary[key] = observed[key].clone();
        }
        std::fs::write(&summary_path, serde_json::to_vec_pretty(&summary).unwrap()).unwrap();
        git(
            root,
            &["add", "-f", "--", &format!("{active}.summary.json")],
        );
        let receipt_path = summary["materialReviewReceipt"]["path"].as_str().unwrap();
        if case == "missing_material" {
            std::fs::remove_file(root.join(receipt_path)).unwrap();
        }
        if case == "invalid_material" {
            let mut receipt: serde_json::Value =
                serde_json::from_slice(&std::fs::read(root.join(receipt_path)).unwrap()).unwrap();
            receipt["rationale"] = "tampered historical material sidecar".into();
            std::fs::write(
                root.join(receipt_path),
                serde_json::to_vec_pretty(&receipt).unwrap(),
            )
            .unwrap();
        }
        git(root, &["add", "-f", "--", receipt_path]);
        let tree = git(root, &["write-tree"]);
        let parent = git(
            root,
            &[
                "commit-tree",
                &tree,
                "-p",
                "ba435024",
                "-m",
                "synthetic same-green preflight fixture",
            ],
        );
        git(root, &["read-tree", "--reset", "-u", "d18be4f8"]);
        let tree = git(root, &["write-tree"]);
        let lifecycle = git(
            root,
            &[
                "commit-tree",
                &tree,
                "-p",
                &parent,
                "-m",
                "synthetic combined lifecycle fixture",
            ],
        );
        git(root, &["reset", "--hard", &lifecycle]);
        (fixture, parent, lifecycle)
    }

    #[test]
    fn post_merge_layout_same_green_preflight_still_requires_valid_material() {
        let id = "WI-1078-v1-0-1-post-merge-sync-finalization";
        let (valid, parent, lifecycle) = same_green_lifecycle_fixture("valid");
        super::validate_layout_finish_semantics(
            &valid.repository,
            id,
            &parent,
            &lifecycle,
            &lifecycle,
            true,
        )
        .expect("valid historical material must still pass with unchanged green preflight fields");
        for case in ["missing_material", "invalid_material"] {
            let (fixture, parent, lifecycle) = same_green_lifecycle_fixture(case);
            assert!(
                super::validate_layout_finish_semantics(
                    &fixture.repository,
                    id,
                    &parent,
                    &lifecycle,
                    &lifecycle,
                    true
                )
                .is_err(),
                "unchanged green Summary must not bypass {case}"
            );
        }
    }

    #[test]
    fn post_merge_layout_archive_metadata_cannot_disable_real_human_close() {
        let fixture = real_combined_checkout();
        let root = &fixture.repository;
        let id = "WI-1078-v1-0-1-post-merge-sync-finalization";
        let path = format!(".ai/work-items/archive/{id}.archive.json");
        let original = super::git_json(root, "d18be4f8", &path).unwrap();
        let mut extra_files = original["files"].clone();
        extra_files["unexpectedPath"] = "outside-the-Work-Item.json".into();
        let mut missing_files = original["files"].clone();
        missing_files
            .as_object_mut()
            .unwrap()
            .remove("taskReportMarkdownPath");
        let changes = [
            ("files", extra_files),
            ("files", missing_files),
            ("workItemId", serde_json::json!("WI-foreign")),
            ("closeRequired", serde_json::json!(false)),
            ("state", serde_json::json!("retired")),
            (
                "repositoryId",
                serde_json::json!(Digest::sha256_bytes(b"foreign repository").to_string()),
            ),
            ("protocolVersion", serde_json::json!(2)),
            ("actorProvenance", serde_json::json!("human:forged")),
            ("recordedAt", serde_json::json!("2026-10-13T00:00:00Z")),
            ("createdAt", serde_json::json!("not a timestamp")),
            ("unexpectedMetadata", serde_json::json!(true)),
        ];
        for (key, value) in changes {
            git(root, &["checkout", "-q", "--detach", "d18be4f8"]);
            let mut manifest = original.clone();
            manifest[key] = value;
            std::fs::write(
                root.join(&path),
                serde_json::to_vec_pretty(&manifest).unwrap(),
            )
            .unwrap();
            git(root, &["add", "-f", "--", &path]);
            let tree = git(root, &["write-tree"]);
            let lifecycle = git(
                root,
                &[
                    "commit-tree",
                    &tree,
                    "-p",
                    "ba435024",
                    "-m",
                    "synthetic single-field archive tamper fixture",
                ],
            );
            git(root, &["reset", "--hard", &lifecycle]);
            assert!(
                super::validate_layout_finish_semantics(
                    root, id, "ba435024", &lifecycle, &lifecycle, true
                )
                .is_err(),
                "ordinary archive must reject mutated {key}"
            );
        }
    }

    #[test]
    fn post_merge_layout_historical_checkout_pins_byte_preserving_git_configuration() {
        let fixture = post_merge_history_fixture();
        let root = fixture.path();
        let revision = git(root, &["rev-parse", "HEAD"]);
        let checkout = super::materialize_git_revision(root, &revision).unwrap();
        assert_eq!(
            checkout.repository,
            std::fs::canonicalize(&checkout.repository).unwrap(),
            "historical consumers must derive paths from the exact canonical root"
        );
        for (key, expected) in [("core.autocrlf", "false"), ("core.longpaths", "true")] {
            let output = Command::new("git")
                .arg("-C")
                .arg(&checkout.repository)
                .args(["config", "--local", "--get", key])
                .output()
                .unwrap();
            assert_eq!(
                String::from_utf8_lossy(&output.stdout).trim(),
                expected,
                "historical checkout must explicitly pin {key} without altering global configuration"
            );
        }
    }

    #[test]
    fn post_merge_layout_nested_checkout_uses_local_paths_with_spaces() {
        let fixture = post_merge_history_fixture();
        let root = fixture.path();
        let revision = git(root, &["rev-parse", "HEAD"]);
        let first = super::materialize_git_revision(root, &revision).unwrap();
        // Windows ordinary directory components cannot end with a space.
        // Use the canonical parent for the long-path filesystem operation;
        // keep internal spaces and the >260-character source path assertion.
        let long_parent = std::fs::canonicalize(&first.directory)
            .unwrap()
            .join("long local path ".repeat(6).trim_end())
            .join("nested path ".repeat(8).trim_end());
        std::fs::create_dir_all(&long_parent).unwrap();
        let spaced = long_parent.join("local repository with spaces");
        std::fs::rename(&first.repository, &spaced).unwrap();
        let canonical = std::fs::canonicalize(&spaced).unwrap();
        assert!(canonical.as_os_str().len() > 260);
        let (cwd, source_argument) = super::historical_clone_location(&canonical).unwrap();
        assert!(source_argument.is_relative());
        assert_eq!(
            std::fs::canonicalize(cwd.join(&source_argument)).unwrap(),
            canonical
        );
        #[cfg(windows)]
        assert!(cwd.as_os_str().len() <= 200);
        let nested = super::materialize_git_revision(&canonical, &revision);
        #[cfg(windows)]
        if nested.is_err() {
            diagnose_long_local_clone(
                &cwd,
                &source_argument,
                &canonical,
                &first.directory,
                &revision,
            );
        }
        let nested = nested.unwrap();
        assert_eq!(git(&nested.repository, &["rev-parse", "HEAD"]), revision);
        #[cfg(windows)]
        let source_identity = String::from_utf8(
            super::windows_metadata_output(
                &canonical,
                &[
                    std::ffi::OsStr::new("rev-parse"),
                    std::ffi::OsStr::new("HEAD"),
                    std::ffi::OsStr::new("HEAD^{tree}"),
                ],
            )
            .unwrap(),
        )
        .unwrap();
        #[cfg(not(windows))]
        let source_identity = format!(
            "{}\n{}",
            git(&cwd, &["rev-parse", "HEAD"]),
            git(&cwd, &["rev-parse", "HEAD^{tree}"])
        );
        let mut source_lines = source_identity.lines();
        assert_eq!(source_lines.next(), Some(revision.as_str()));
        assert_eq!(
            Some(git(&nested.repository, &["rev-parse", "HEAD^{tree}"])).as_deref(),
            source_lines.next()
        );
        assert_eq!(
            std::fs::read(nested.repository.join("source.txt")).unwrap(),
            std::fs::read(canonical.join("source.txt")).unwrap()
        );
        assert!(
            super::materialize_git_revision(
                std::path::Path::new("ssh://invalid.example/repository"),
                &revision
            )
            .is_err(),
            "remote-looking input must fail local filesystem resolution"
        );
    }

    // Failure-only, test-only probes. They retain the original assertion and
    // distinguish local source discovery from upload-pack's enter_repo stage.
    #[cfg(windows)]
    fn diagnose_long_local_clone(
        cwd: &Path,
        native: &Path,
        source: &Path,
        directory: &Path,
        revision: &str,
    ) {
        use std::os::windows::ffi::{OsStrExt, OsStringExt};
        let units: Vec<u16> = native
            .as_os_str()
            .encode_wide()
            .map(|unit| {
                if unit == u16::from(b'\\') {
                    u16::from(b'/')
                } else {
                    unit
                }
            })
            .collect();
        let slashes = PathBuf::from(std::ffi::OsString::from_wide(&units));
        assert_eq!(std::fs::canonicalize(cwd.join(&slashes)).unwrap(), source);
        eprintln!(
            "long clone diagnostic: cwd_units={} source_units={} relative_units={} git_head_read={:?}",
            cwd.as_os_str().encode_wide().count(),
            source.as_os_str().encode_wide().count(),
            native.as_os_str().encode_wide().count(),
            std::fs::read(source.join(".git/HEAD")).map(|bytes| bytes.len())
        );
        let report_from = |label: &str, mut command: Command, process_cwd: &Path| {
            let started = std::time::Instant::now();
            command
                .current_dir(process_cwd)
                .stdin(std::process::Stdio::null())
                .env("GIT_TRACE", "1");
            match command.output() {
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    eprintln!(
                        "long clone probe {label}: elapsed_ms={} exit_code={:?} stdout_bytes={} stderr_bytes={} upload_pack_seen={} enter_repo_rejected={} path_too_long={} permission_denied={}",
                        started.elapsed().as_millis(),
                        output.status.code(),
                        output.stdout.len(),
                        output.stderr.len(),
                        stderr.contains("git-upload-pack") || stderr.contains("git upload-pack"),
                        stderr.contains("does not appear to be a git repository"),
                        stderr.contains("Filename too long")
                            || stderr.contains("File name too long"),
                        stderr.contains("Permission denied")
                    );
                    if let Some(fatal) = stderr.lines().find(|line| line.starts_with("fatal:")) {
                        let redacted = fatal
                            .split(char::from(39))
                            .enumerate()
                            .map(|(index, part)| if index % 2 == 0 { part } else { "<path>" })
                            .collect::<Vec<_>>()
                            .join("'");
                        eprintln!("long clone probe {label}: fatal_template={redacted}");
                    }
                    drop(stderr);
                    Some(output)
                }
                Err(error) => {
                    eprintln!(
                        "long clone probe {label}: elapsed_ms={} spawn_error={error:?}",
                        started.elapsed().as_millis()
                    );
                    None
                }
            }
        };
        let report = |label: &str, command: Command| {
            let _ = report_from(label, command, cwd);
        };
        for (label, argument) in [("native", native), ("forward-slash", slashes.as_path())] {
            let mut discover = Command::new("git");
            discover
                .args(["-c", "core.longpaths=true", "-C"])
                .arg(argument)
                .args(["rev-parse", "--absolute-git-dir", "HEAD"]);
            report(&format!("discover-{label}"), discover);
            let mut clone = Command::new("git");
            clone
                .args([
                    "-c",
                    "core.longpaths=true",
                    "clone",
                    "--quiet",
                    "--shared",
                    "--local",
                    "--no-checkout",
                    "--config",
                    "core.autocrlf=false",
                    "--config",
                    "core.longpaths=true",
                    "--",
                ])
                .arg(argument)
                .arg(directory.join(format!("diagnostic-{label}")));
            report(&format!("clone-{label}"), clone);
        }
        for strict in [false, true] {
            let mut upload = Command::new("git");
            upload.args([
                "-c",
                "core.longpaths=true",
                "upload-pack",
                "--stateless-rpc",
                "--advertise-refs",
            ]);
            if strict {
                upload.arg("--strict");
            }
            upload.arg(source.join(".git"));
            report(
                if strict {
                    "upload-pack-strict"
                } else {
                    "upload-pack-default"
                },
                upload,
            );
        }
        // Keep every process at a genuinely short canonical ancestor. The
        // metadata-only source locator is verified before these probes.
        let safe_cwd = source
            .ancestors()
            .find(|path| path.as_os_str().len() <= 200)
            .unwrap();
        let git_dir = cwd.join(native).join(".git");
        assert_eq!(
            std::fs::canonicalize(&git_dir).unwrap(),
            source.join(".git")
        );
        eprintln!(
            "long clone bundle geometry: safe_cwd_units={} git_dir_units={}",
            safe_cwd.as_os_str().encode_wide().count(),
            git_dir.as_os_str().encode_wide().count()
        );
        // Compare only command-boundary representations of the same Git dir.
        let mut dos_units: Vec<u16> = git_dir.as_os_str().encode_wide().collect();
        if dos_units.starts_with(&[92, 92, 63, 92]) && dos_units.get(5) == Some(&58) {
            dos_units.drain(..4);
        }
        for unit in &mut dos_units {
            if *unit == 92 {
                *unit = 47;
            }
        }
        let dos_git_dir = PathBuf::from(std::ffi::OsString::from_wide(&dos_units));
        assert_eq!(
            std::fs::canonicalize(&dos_git_dir).unwrap(),
            source.join(".git")
        );
        let mut selected_dir = git_dir.clone();
        let mut selected_bare = false;
        let mut source_output = None;
        for (form, locator) in [
            ("verbatim", git_dir.as_path()),
            ("dos", dos_git_dir.as_path()),
        ] {
            for bare in [false, true] {
                let mut metadata = Command::new("git");
                metadata.args(["-c", "core.longpaths=true"]);
                if bare {
                    metadata.arg("--bare");
                }
                metadata
                    .arg("--git-dir")
                    .arg(locator)
                    .args(["rev-parse", "HEAD", "HEAD^{tree}"]);
                let output = report_from(
                    &format!("explicit-metadata-{form}-bare-{bare}"),
                    metadata,
                    safe_cwd,
                );
                if source_output.is_none() && output.as_ref().is_some_and(|o| o.status.success()) {
                    source_output = output;
                    selected_dir = locator.to_path_buf();
                    selected_bare = bare;
                }
            }
        }
        let expected_tree = source_output
            .as_ref()
            .filter(|o| o.status.success())
            .and_then(|o| std::str::from_utf8(&o.stdout).ok())
            .and_then(|s| s.lines().nth(1));
        let source_head_match = source_output
            .as_ref()
            .filter(|o| o.status.success())
            .and_then(|o| std::str::from_utf8(&o.stdout).ok())
            .and_then(|s| s.lines().next())
            == Some(revision);
        eprintln!(
            "long clone bundle source: head_match={source_head_match} tree_read={}",
            expected_tree.is_some()
        );
        let bundle = directory.join("diagnostic-source.bundle");
        let mut create = Command::new("git");
        create.args(["-c", "core.longpaths=true"]);
        if selected_bare {
            create.arg("--bare");
        }
        create
            .arg("--git-dir")
            .arg(&selected_dir)
            .args(["bundle", "create"])
            .arg(&bundle)
            .arg("--all")
            .arg(revision);
        let created = report_from("bundle-create", create, safe_cwd);
        if created.is_some_and(|o| o.status.success()) {
            let target = directory.join("diagnostic-bundle-clone");
            let mut clone = Command::new("git");
            clone
                .args([
                    "-c",
                    "core.longpaths=true",
                    "clone",
                    "--quiet",
                    "--no-checkout",
                    "--config",
                    "core.autocrlf=false",
                    "--config",
                    "core.longpaths=true",
                    "--",
                ])
                .arg(&bundle)
                .arg(&target);
            let cloned = report_from("bundle-clone", clone, safe_cwd);
            if cloned.is_some_and(|o| o.status.success()) {
                let mut checkout = Command::new("git");
                checkout
                    .arg("-C")
                    .arg(&target)
                    .args(["checkout", "--quiet", "--detach"])
                    .arg(revision);
                let checked = report_from("bundle-checkout", checkout, safe_cwd);
                let mut identity = Command::new("git");
                identity
                    .arg("-C")
                    .arg(&target)
                    .args(["rev-parse", "HEAD", "HEAD^{tree}"]);
                let identity = report_from("bundle-identity", identity, safe_cwd);
                let lines = identity
                    .as_ref()
                    .filter(|o| o.status.success())
                    .and_then(|o| std::str::from_utf8(&o.stdout).ok());
                let bytes_match = std::fs::read(target.join("source.txt"))
                    .ok()
                    .zip(std::fs::read(source.join("source.txt")).ok())
                    .is_some_and(|(a, b)| a == b);
                eprintln!(
                    "long clone bundle proof: checkout_ok={} head_match={} tree_match={} bytes_match={bytes_match}",
                    checked.is_some_and(|o| o.status.success()),
                    lines.and_then(|s| s.lines().next()) == Some(revision),
                    expected_tree.is_some()
                        && lines.and_then(|s| s.lines().nth(1)) == expected_tree
                );
            }
        }
    }

    #[test]
    fn post_merge_layout_recipe_comparison_accepts_only_platform_line_endings() {
        assert!(super::analysis_recipe_source_matches(
            b"same recipe\n",
            b"same recipe\r\n"
        ));
        assert!(!super::analysis_recipe_source_matches(
            b"same recipe\n",
            b"changed recipe\r\n"
        ));
        assert!(!super::analysis_recipe_source_matches(
            b"same recipe\n",
            b"same recipe\r"
        ));
    }

    fn github_pull_request_origin(identity: &ResourceFinalizationPullRequestIdentity) -> String {
        format!(
            "https://api.github.com/repos/example/fixture/pulls/{}",
            identity.number
        )
    }

    fn github_pull_request_body(
        identity: &ResourceFinalizationPullRequestIdentity,
        head_ref: &str,
    ) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "number": identity.number,
            "html_url": identity.url,
            "state": "closed",
            "merged": true,
            "merge_commit_sha": identity.merge_commit,
            "head": {
                "ref": head_ref,
                "sha": identity.head_revision,
                "repo": {"full_name": "example/fixture"}
            },
            "base": {
                "ref": identity.base_branch,
                "sha": identity.base_revision,
                "repo": {"full_name": "example/fixture"}
            }
        }))
        .expect("GitHub pull request observation serializes")
    }

    fn github_ref_origin(reference: &str) -> String {
        let branch = reference
            .strip_prefix("refs/heads/")
            .expect("branch ref has refs/heads prefix");
        format!("https://api.github.com/repos/example/fixture/git/ref/heads/{branch}")
    }

    fn github_present_ref_body(reference: &str, sha: &str) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "ref": reference,
            "node_id": "fixture-node",
            "url": "https://api.github.com/repos/example/fixture/git/refs/heads/fixture",
            "object": {
                "sha": sha,
                "type": "commit",
                "url": "https://api.github.com/repos/example/fixture/git/commits/fixture"
            }
        }))
        .expect("GitHub present-ref observation serializes")
    }

    fn github_deleted_ref_body(
        repository: &str,
        reference: &str,
        observation_id: &str,
        principal: &str,
        observed_at: &str,
    ) -> Vec<u8> {
        let branch = reference.strip_prefix("refs/heads/").unwrap();
        serde_json::to_vec(&serde_json::json!({
            "schemaVersion": 1,
            "observationId": observation_id,
            "principal": principal,
            "repository": repository,
            "ref": reference,
            "observedAt": observed_at,
            "accessCheck": {
                "observationId": observation_id,
                "principal": principal,
                "repository": repository,
                "ref": reference,
                "method": "GET",
                "url": format!("https://api.github.com/repos/{repository}/git/matching-refs/heads/{branch}"),
                "httpStatus": 200,
                "nextPageUrl": null,
                "refs": []
            },
            "refQuery": {
                "observationId": observation_id,
                "principal": principal,
                "repository": repository,
                "ref": reference,
                "method": "GET",
                "url": format!("https://api.github.com/repos/{repository}/git/ref/heads/{branch}"),
                "httpStatus": 404,
                "body": {
                    "message": "Not Found",
                    "documentation_url": "https://docs.github.com/rest/git/refs#get-a-reference",
                    "status": "404"
                }
            }
        }))
        .expect("GitHub deleted-ref observation serializes")
    }

    fn deleted_receipt(
        branch: &str,
        worktree: &Path,
        worktree_git_dir: &Path,
        common_dir: &Path,
    ) -> ResourceFinalizationReceipt {
        let revision = "head-1".to_owned();
        let branch = branch.to_owned();
        ResourceFinalizationReceipt {
            schema_version: cockpit_protocol::RESOURCE_FINALIZATION_POST_MERGE_SYNC_SCHEMA_VERSION,
            receipt_id: "receipt-1".into(),
            operation_id: "operation-1".into(),
            repository_id:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            work_item_id: "WI-9997-post-merge-sync-fixture".into(),
            runtime_version: "test-runtime".into(),
            runtime_digest: Digest::sha256_bytes(b"runtime"),
            provider: "github".into(),
            pull_request: ResourceFinalizationPullRequestIdentity {
                number: 9001,
                url: "https://github.com/example/fixture/pull/9001".into(),
                head_revision: "source-head-revision".into(),
                base_branch: "main".into(),
                base_remote: "origin".into(),
                base_revision: "source-base-revision".into(),
                merge_commit: Some("source-merge-commit".into()),
            },
            branch: ResourceFinalizationBranchIdentity {
                name: branch.clone(),
                remote: "origin".into(),
                head_revision: revision.clone(),
            },
            worktree: ResourceFinalizationWorktreeIdentity {
                worktree_id: "worktree-1".into(),
                path: worktree.display().to_string(),
                branch: branch.clone(),
                head_revision: revision,
            },
            before: ResourceFinalizationState {
                pull_request: ResourceFinalizationPullRequestState::Merged,
                branch: ResourceFinalizationBranchState::Present,
                worktree: ResourceFinalizationWorktreeState::Clean,
            },
            after: ResourceFinalizationState {
                pull_request: ResourceFinalizationPullRequestState::Merged,
                branch: ResourceFinalizationBranchState::Deleted,
                worktree: ResourceFinalizationWorktreeState::Removed,
            },
            result: ResourceFinalizationResult {
                disposition: ResourceFinalizationDisposition::Deleted,
                failure_codes: Vec::new(),
                unknown_codes: Vec::new(),
            },
            actor: "human:test".into(),
            authority_source: "test".into(),
            reason: "test deleted post-merge sync resources".into(),
            timestamp: "2026-10-12T00:00:00Z".into(),
            contract_digest: None,
            contract_base_revision: None,
            resource_context: None,
            historical: None,
            post_merge_sync: Some(PostMergeSyncEvidence {
                lifecycle_layout: None,
                documentation: None,
                observation_selectors: None,
                source_pull_request: ResourceFinalizationPullRequestIdentity {
                    number: 9001,
                    url: "https://github.com/example/fixture/pull/9001".into(),
                    head_revision: "source-head-revision".into(),
                    base_branch: "main".into(),
                    base_remote: "origin".into(),
                    base_revision: "source-base-revision".into(),
                    merge_commit: Some("source-merge-commit".into()),
                },
                sync_pull_request: ResourceFinalizationPullRequestIdentity {
                    number: 9002,
                    url: "https://github.com/example/fixture/pull/9002".into(),
                    head_revision: "head-1".into(),
                    base_branch: "main".into(),
                    base_remote: "origin".into(),
                    base_revision: "source-merge-commit".into(),
                    merge_commit: Some("merge-commit".into()),
                },
                source_evidence_digest: Digest::sha256_bytes(b"source"),
                sync_evidence_digest: Digest::sha256_bytes(b"sync"),
                finish_revision: "finish-1".into(),
                archive_revision: "archive-1".into(),
                docs_revision: "head-1".into(),
                current_main_revision: "merge-commit".into(),
                remote_branch_ref: format!("refs/heads/{branch}"),
                remote_branch_state: PostMergeSyncRemoteBranchState::Deleted,
                remote_evidence_digest: Digest::sha256_bytes(b"remote"),
                git_common_dir: common_dir.display().to_string(),
                worktree_git_dir: worktree_git_dir.display().to_string(),
            }),
        }
    }

    fn commit(root: &Path, message: &str) -> String {
        git(root, &["add", "--all"]);
        git(root, &["commit", "-q", "-m", message]);
        git(root, &["rev-parse", "HEAD"])
    }

    fn fixture_attempt_receipt(
        work_item_id: &str,
        bindings: &super::FinishAttemptBindings,
        passed: bool,
        execution_records: serde_json::Value,
        processes_spawned: u64,
        process_spawn_failures: u64,
    ) -> serde_json::Value {
        serde_json::json!({
            "workItemId": work_item_id,
            "repositoryId": bindings.repository_id,
            "runtimeVersion": bindings.runtime_version,
            "runtimeDigest": bindings.runtime_digest.to_string(),
            "results": [{
                "nodeId": "fixture-command",
                "passed": passed,
                "reused": false,
                "protected": false,
                "action": "execute",
                "state": "not_applicable",
                "reason": "reuse_not_configured",
                "receiptId": null,
                "outputDigest": Digest::sha256_bytes(b"fixture output").to_string(),
                "outputTruncated": false,
                "timedOut": false,
                "satisfiedBy": "execution"
            }],
            "executionRecords": execution_records,
            "receiptCandidates": [],
            "nodesPlanned": 1,
            "nodesExecuted": 1,
            "nodesReused": 0,
            "rerunStale": 0,
            "rerunUnknown": 0,
            "protectedNodesExecuted": 0,
            "protectedNodesSkipped": 0,
            "planningElapsedMs": 0,
            "executionElapsedMs": 1,
            "processesSpawned": processes_spawned,
            "maxConcurrentProcesses": processes_spawned,
            "processSpawnFailures": process_spawn_failures,
            "gitCalls": 0,
            "filesRead": 0,
            "filesHashed": 0,
            "elapsedMs": 1,
            "passed": passed
        })
    }

    fn fixture_attempt_command(bindings: &super::FinishAttemptBindings) -> serde_json::Value {
        serde_json::json!({
            "nodeId": "fixture-command",
            "program": "cargo",
            "args": ["test", "--locked", "--package", "cockpit-repository"],
            "scope": ["source.txt"],
            "stage": "task",
            "runner": "local",
            "runtimeDigest": bindings.runtime_digest.to_string(),
            "baseCommit": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "timeoutSeconds": 10,
            "commandDigest": Digest::sha256_bytes(b"fixture command").to_string()
        })
    }

    fn fixture_attempt_value(
        work_item_id: &str,
        bindings: &super::FinishAttemptBindings,
        state: &str,
        passed: bool,
    ) -> serde_json::Value {
        let execution_records = if state == "precondition_rejected" {
            serde_json::json!([])
        } else {
            serde_json::json!([{
                "nodeId": "fixture-command",
                "commandDigest": Digest::sha256_bytes(b"fixture command").to_string(),
                "timeoutSeconds": 10,
                "deadlineMs": 1000,
                "spawned": true,
                "passed": passed,
                "exitCode": if passed { 0 } else { 1 },
                "stdout": "",
                "stderr": "",
                "stdoutTruncated": false,
                "stderrTruncated": false,
                "timedOut": false,
                "elapsedMs": 1
            }])
        };
        let processes_spawned = if state == "precondition_rejected" {
            0
        } else {
            1
        };
        let receipt = if state == "precondition_rejected" {
            serde_json::Value::Null
        } else {
            fixture_attempt_receipt(
                work_item_id,
                bindings,
                passed,
                execution_records.clone(),
                processes_spawned,
                0,
            )
        };
        let mut attempt = serde_json::json!({
            "protocolVersion": 1,
            "schemaVersion": 1,
            "kind": "verification_attempt",
            "state": state,
            "workItemId": work_item_id,
            "repositoryId": bindings.repository_id,
            "runtimeVersion": bindings.runtime_version,
            "runtimeDigest": bindings.runtime_digest.to_string(),
            "contractDigest": bindings.contract_digest.to_string(),
            "contractExecutionScopeDigest": bindings.scope_digest.to_string(),
            "repositorySnapshotDigest": bindings.snapshot_digest.to_string(),
            "commands": [fixture_attempt_command(bindings)],
            "passed": passed,
            "processesSpawned": processes_spawned,
            "executionRecords": execution_records,
            "receipt": receipt,
            "createdAt": "2026-10-12T00:00:00Z"
        });
        if state == "formal_receipt_rejected" {
            attempt["diagnostic"] =
                serde_json::json!({"code": "fixture_rejected", "message": "fixture diagnostic"});
        }
        seal_attempt(&mut attempt);
        attempt
    }

    fn seal_attempt(attempt: &mut serde_json::Value) {
        if let Some(object) = attempt.as_object_mut() {
            object.remove("attemptId");
        }
        let digest = cockpit_protocol::digest_json(&attempt).unwrap();
        attempt["attemptId"] = serde_json::json!(digest.to_string());
    }

    fn attempt_filename_digest(attempt: &serde_json::Value) -> String {
        attempt["attemptId"]
            .as_str()
            .unwrap()
            .strip_prefix("sha256:")
            .unwrap()
            .to_owned()
    }

    fn append_attempt(
        root: &Path,
        work_item_id: &str,
        bindings: &super::FinishAttemptBindings,
    ) -> String {
        let attempt = fixture_attempt_value(work_item_id, bindings, "execution_failed", false);
        let suffix = attempt["attemptId"]
            .as_str()
            .unwrap()
            .strip_prefix("sha256:")
            .unwrap()
            .to_owned();
        let path = root.join(format!(
            ".ai/evidence/{work_item_id}.verification-attempt.{suffix}.json"
        ));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, serde_json::to_vec_pretty(&attempt).unwrap()).unwrap();
        suffix
    }

    fn fixture_finish_bindings(
        root: &Path,
        work_item_id: &str,
    ) -> (String, super::FinishAttemptBindings) {
        let contract_path = root.join(format!(
            ".ai/work-items/active/{work_item_id}.contract.json"
        ));
        std::fs::create_dir_all(contract_path.parent().unwrap()).unwrap();
        let contract = serde_json::json!({"workItemId": work_item_id, "scope": ["source.txt"]});
        std::fs::write(
            &contract_path,
            serde_json::to_vec_pretty(&contract).unwrap(),
        )
        .unwrap();
        let source_head = commit(root, "source Contract");
        let contract_digest = super::super::canonical_contract_digest(&contract).unwrap();
        let scope_digest = cockpit_protocol::digest_json(&serde_json::json!({
            "scope": contract["scope"],
        }))
        .unwrap();
        let bindings = super::FinishAttemptBindings {
            repository_id: super::super::repository_id(root).to_string(),
            contract_digest,
            scope_digest,
            snapshot_digest: Digest::sha256_bytes(b"fixture source snapshot"),
            runtime_version: "test-runtime".into(),
            runtime_digest: Digest::sha256_bytes(b"test runtime"),
        };
        (source_head, bindings)
    }

    fn write_finish_verification(
        root: &Path,
        work_item_id: &str,
        bindings: &super::FinishAttemptBindings,
    ) {
        let evidence = root.join(format!(".ai/evidence/{work_item_id}.verification.json"));
        std::fs::create_dir_all(evidence.parent().unwrap()).unwrap();
        let value = serde_json::json!({
            "passed": true,
            "repositoryId": bindings.repository_id,
            "contractDigest": bindings.contract_digest.to_string(),
            "repositorySnapshotDigest": bindings.snapshot_digest.to_string(),
            "runtimeVersion": bindings.runtime_version,
            "runtimeDigest": bindings.runtime_digest.to_string(),
            "receipt": {
                "passed": true,
                "runtimeVersion": bindings.runtime_version,
                "runtimeDigest": bindings.runtime_digest.to_string()
            }
        });
        std::fs::write(evidence, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    }

    fn post_merge_history_fixture() -> tempfile::TempDir {
        let fixture = tempfile::tempdir().expect("fixture directory");
        let root = fixture.path();
        git(root, &["init", "-q"]);
        git(root, &["config", "user.name", "AI Cockpit Tests"]);
        git(root, &["config", "user.email", "tests@example.invalid"]);
        std::fs::write(root.join("source.txt"), "synthetic source\n").unwrap();
        commit(root, "source head");
        fixture
    }

    #[test]
    fn post_merge_sync_finish_history_accepts_only_append_only_attempt_commits() {
        let fixture = post_merge_history_fixture();
        let root = fixture.path();
        let work_item_id = "WI-9999-post-merge-sync-fixture";
        let (source_head, bindings) = fixture_finish_bindings(root, work_item_id);
        append_attempt(root, work_item_id, &bindings);
        commit(root, "append failed verification attempt");
        std::fs::write(root.join("finish-evidence.txt"), "finish\n").unwrap();
        write_finish_verification(root, work_item_id, &bindings);
        let finish = commit(root, "finish");
        super::validate_finish_attempt_history(root, work_item_id, &source_head, &finish, &finish)
            .expect(
                "a Runtime execution_failed attempt may be appended before a successful finish",
            );
    }

    #[test]
    fn post_merge_sync_finish_history_rejects_even_reverted_intermediate_product_changes() {
        let fixture = post_merge_history_fixture();
        let root = fixture.path();
        let work_item_id = "WI-9998-post-merge-sync-fixture";
        let (source_head, bindings) = fixture_finish_bindings(root, work_item_id);
        append_attempt(root, work_item_id, &bindings);
        commit(root, "append failed verification attempt");
        std::fs::write(root.join("product.txt"), "temporary product change\n").unwrap();
        commit(root, "unexpected product change");
        git(root, &["rm", "-q", "product.txt"]);
        commit(root, "revert product change");
        std::fs::write(root.join("finish-evidence.txt"), "finish\n").unwrap();
        write_finish_verification(root, work_item_id, &bindings);
        let finish = commit(root, "finish");
        assert!(
            super::validate_finish_attempt_history(
                root,
                work_item_id,
                &source_head,
                &finish,
                &finish
            )
            .is_err(),
            "reverting an intermediate product change must not erase it from the reviewed history"
        );
    }

    #[test]
    fn post_merge_sync_finish_attempts_bind_contract_snapshot_runtime_and_receipt() {
        let fixture = post_merge_history_fixture();
        let root = fixture.path();
        let id = "WI-9991-post-merge-sync-fixture";
        let (_, bindings) = fixture_finish_bindings(root, id);
        let valid = fixture_attempt_value(id, &bindings, "execution_failed", false);
        assert!(super::finish_attempt_matches(
            &valid,
            &attempt_filename_digest(&valid),
            id,
            &bindings,
        ));
        let precondition = fixture_attempt_value(id, &bindings, "precondition_rejected", false);
        assert!(super::finish_attempt_matches(
            &precondition,
            &attempt_filename_digest(&precondition),
            id,
            &bindings,
        ));

        let mut spawn_failure = valid.clone();
        spawn_failure["executionRecords"][0]["spawned"] = false.into();
        spawn_failure["executionRecords"][0]["exitCode"] = serde_json::Value::Null;
        spawn_failure["receipt"]["executionRecords"] = spawn_failure["executionRecords"].clone();
        spawn_failure["processesSpawned"] = 0.into();
        spawn_failure["receipt"]["processesSpawned"] = 0.into();
        spawn_failure["receipt"]["processSpawnFailures"] = 1.into();
        spawn_failure["receipt"]["maxConcurrentProcesses"] = 1.into();
        seal_attempt(&mut spawn_failure);
        assert!(
            super::finish_attempt_matches(
                &spawn_failure,
                &attempt_filename_digest(&spawn_failure),
                id,
                &bindings,
            ),
            "a typed failed spawn must be retained as an unspawned execution record"
        );

        let mut wrong_contract = valid.clone();
        wrong_contract["contractDigest"] =
            serde_json::json!(Digest::sha256_bytes(b"other contract").to_string());
        seal_attempt(&mut wrong_contract);
        assert!(!super::finish_attempt_matches(
            &wrong_contract,
            &attempt_filename_digest(&wrong_contract),
            id,
            &bindings,
        ));

        let mut wrong_snapshot = valid.clone();
        wrong_snapshot["repositorySnapshotDigest"] =
            serde_json::json!(Digest::sha256_bytes(b"other snapshot").to_string());
        seal_attempt(&mut wrong_snapshot);
        assert!(!super::finish_attempt_matches(
            &wrong_snapshot,
            &attempt_filename_digest(&wrong_snapshot),
            id,
            &bindings,
        ));

        let mut wrong_runtime = valid.clone();
        wrong_runtime["runtimeDigest"] =
            serde_json::json!(Digest::sha256_bytes(b"other runtime").to_string());
        seal_attempt(&mut wrong_runtime);
        assert!(!super::finish_attempt_matches(
            &wrong_runtime,
            &attempt_filename_digest(&wrong_runtime),
            id,
            &bindings,
        ));

        let mut spawned_without_command = valid.clone();
        spawned_without_command["commands"] = serde_json::json!([]);
        seal_attempt(&mut spawned_without_command);
        assert!(
            !super::finish_attempt_matches(
                &spawned_without_command,
                &attempt_filename_digest(&spawned_without_command),
                id,
                &bindings,
            ),
            "a spawned execution record must resolve to a declared command"
        );

        let mut missing_command_id = valid.clone();
        missing_command_id["commands"][0]
            .as_object_mut()
            .unwrap()
            .remove("nodeId");
        seal_attempt(&mut missing_command_id);
        assert!(!super::finish_attempt_matches(
            &missing_command_id,
            &attempt_filename_digest(&missing_command_id),
            id,
            &bindings,
        ));

        let mut missing_command_digest = valid.clone();
        missing_command_digest["commands"][0]
            .as_object_mut()
            .unwrap()
            .remove("commandDigest");
        seal_attempt(&mut missing_command_digest);
        assert!(!super::finish_attempt_matches(
            &missing_command_digest,
            &attempt_filename_digest(&missing_command_digest),
            id,
            &bindings,
        ));

        let mut mismatched_command_digest = valid.clone();
        mismatched_command_digest["commands"][0]["commandDigest"] =
            Digest::sha256_bytes(b"another command").to_string().into();
        seal_attempt(&mut mismatched_command_digest);
        assert!(
            !super::finish_attempt_matches(
                &mismatched_command_digest,
                &attempt_filename_digest(&mismatched_command_digest),
                id,
                &bindings,
            ),
            "execution records must bind the exact declared command digest"
        );

        let mut wrong_spawned_count = valid.clone();
        wrong_spawned_count["processesSpawned"] = 0.into();
        wrong_spawned_count["receipt"]["processesSpawned"] = 0.into();
        seal_attempt(&mut wrong_spawned_count);
        assert!(!super::finish_attempt_matches(
            &wrong_spawned_count,
            &attempt_filename_digest(&wrong_spawned_count),
            id,
            &bindings,
        ));

        let mut mismatched_result = valid.clone();
        mismatched_result["receipt"]["results"][0]["passed"] = true.into();
        seal_attempt(&mut mismatched_result);
        assert!(!super::finish_attempt_matches(
            &mismatched_result,
            &attempt_filename_digest(&mismatched_result),
            id,
            &bindings,
        ));

        let mut duplicate_command_id = valid.clone();
        let duplicate = duplicate_command_id["commands"][0].clone();
        duplicate_command_id["commands"]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        seal_attempt(&mut duplicate_command_id);
        assert!(
            !super::finish_attempt_matches(
                &duplicate_command_id,
                &attempt_filename_digest(&duplicate_command_id),
                id,
                &bindings,
            ),
            "the command index must reject duplicate node IDs"
        );

        let mut duplicate_execution_id = valid.clone();
        let duplicate = duplicate_execution_id["executionRecords"][0].clone();
        duplicate_execution_id["executionRecords"]
            .as_array_mut()
            .unwrap()
            .push(duplicate.clone());
        duplicate_execution_id["receipt"]["executionRecords"] =
            duplicate_execution_id["executionRecords"].clone();
        duplicate_execution_id["receipt"]["processesSpawned"] = 2.into();
        duplicate_execution_id["processesSpawned"] = 2.into();
        duplicate_execution_id["receipt"]["processSpawnFailures"] = 0.into();
        duplicate_execution_id["receipt"]["nodesExecuted"] = 2.into();
        duplicate_execution_id["receipt"]["maxConcurrentProcesses"] = 2.into();
        seal_attempt(&mut duplicate_execution_id);
        assert!(
            !super::finish_attempt_matches(
                &duplicate_execution_id,
                &attempt_filename_digest(&duplicate_execution_id),
                id,
                &bindings,
            ),
            "the execution history must reject duplicate node IDs"
        );

        let mut wrong_receipt = valid.clone();
        wrong_receipt["receipt"]["runtimeDigest"] =
            serde_json::json!(Digest::sha256_bytes(b"foreign receipt runtime").to_string());
        seal_attempt(&mut wrong_receipt);
        assert!(!super::finish_attempt_matches(
            &wrong_receipt,
            &attempt_filename_digest(&wrong_receipt),
            id,
            &bindings,
        ));

        let mut invalid_state = valid;
        invalid_state["state"] = serde_json::json!("execution_completed");
        seal_attempt(&mut invalid_state);
        assert!(!super::finish_attempt_matches(
            &invalid_state,
            &attempt_filename_digest(&invalid_state),
            id,
            &bindings,
        ));
    }

    #[test]
    fn post_merge_sync_finish_attempt_rejects_passed_non_reused_result_without_execution_record() {
        let fixture = post_merge_history_fixture();
        let root = fixture.path();
        let id = "WI-9981-post-merge-sync-fixture";
        let (_, bindings) = fixture_finish_bindings(root, id);
        let mut attempt = fixture_attempt_value(id, &bindings, "execution_completed", true);
        attempt["executionRecords"] = serde_json::json!([]);
        attempt["receipt"]["executionRecords"] = serde_json::json!([]);
        attempt["receipt"]["nodesExecuted"] = 0.into();
        attempt["receipt"]["processesSpawned"] = 0.into();
        attempt["receipt"]["maxConcurrentProcesses"] = 0.into();
        attempt["processesSpawned"] = 0.into();
        seal_attempt(&mut attempt);

        assert!(
            !super::finish_attempt_matches(
                &attempt,
                &attempt_filename_digest(&attempt),
                id,
                &bindings,
            ),
            "every passed non-reused result must have one matching execution record"
        );
    }

    #[test]
    fn post_merge_sync_finish_attempt_rejects_passed_record_with_nonzero_exit_code() {
        let fixture = post_merge_history_fixture();
        let root = fixture.path();
        let id = "WI-9982-post-merge-sync-fixture";
        let (_, bindings) = fixture_finish_bindings(root, id);
        let mut attempt = fixture_attempt_value(id, &bindings, "execution_failed", false);
        attempt["state"] = "execution_completed".into();
        attempt["passed"] = true.into();
        attempt["receipt"]["passed"] = true.into();
        attempt["receipt"]["results"][0]["passed"] = true.into();
        attempt["executionRecords"][0]["passed"] = true.into();
        attempt["receipt"]["executionRecords"][0]["passed"] = true.into();
        seal_attempt(&mut attempt);

        assert!(
            !super::finish_attempt_matches(
                &attempt,
                &attempt_filename_digest(&attempt),
                id,
                &bindings,
            ),
            "a passed record must be spawned, successful, and not timed out"
        );
    }

    #[test]
    fn post_merge_sync_finish_attempt_preserves_spawned_observer_failure_without_exit_code() {
        let fixture = post_merge_history_fixture();
        let root = fixture.path();
        let id = "WI-9983-post-merge-sync-fixture";
        let (_, bindings) = fixture_finish_bindings(root, id);
        let mut attempt = fixture_attempt_value(id, &bindings, "execution_failed", false);
        attempt["executionRecords"][0]["exitCode"] = serde_json::Value::Null;
        attempt["receipt"]["executionRecords"][0]["exitCode"] = serde_json::Value::Null;
        seal_attempt(&mut attempt);

        assert!(
            super::finish_attempt_matches(
                &attempt,
                &attempt_filename_digest(&attempt),
                id,
                &bindings,
            ),
            "a process observer failure can be spawned but have no exit code"
        );
    }

    #[test]
    fn post_merge_sync_finish_attempt_preserves_success_with_truncated_output() {
        let fixture = post_merge_history_fixture();
        let root = fixture.path();
        let id = "WI-9984-post-merge-sync-fixture";
        let (_, bindings) = fixture_finish_bindings(root, id);
        let mut attempt = fixture_attempt_value(id, &bindings, "execution_completed", true);
        attempt["executionRecords"][0]["stdoutTruncated"] = true.into();
        attempt["receipt"]["executionRecords"][0]["stdoutTruncated"] = true.into();
        attempt["receipt"]["results"][0]["outputTruncated"] = true.into();
        seal_attempt(&mut attempt);

        assert!(
            super::finish_attempt_matches(
                &attempt,
                &attempt_filename_digest(&attempt),
                id,
                &bindings,
            ),
            "successful execution remains valid when captured output is truncated"
        );
    }

    #[test]
    fn post_merge_sync_historical_json_is_read_from_the_committed_revision() {
        let fixture = post_merge_history_fixture();
        let root = fixture.path();
        let path = ".ai/evidence/WI-9996-post-merge-sync-fixture.verification.json";
        let absolute = root.join(path);
        std::fs::create_dir_all(absolute.parent().unwrap()).unwrap();
        std::fs::write(&absolute, br#"{"passed":false}"#).unwrap();
        let revision = commit(root, "commit immutable verification evidence");
        std::fs::write(&absolute, br#"{"passed":true}"#).unwrap();

        let historical = super::git_json(root, &revision, path).expect("read committed JSON blob");
        assert_eq!(historical["passed"], serde_json::json!(false));
        let checkout = super::materialize_git_revision(root, &revision)
            .expect("materialize an isolated historical checkout");
        let materialized = std::fs::read_to_string(checkout.repository.join(path)).unwrap();
        assert!(materialized.contains("false"));
        assert!(!materialized.contains("true"));
    }

    #[test]
    fn post_merge_sync_current_archive_must_match_the_selected_archive_commit() {
        let fixture = post_merge_history_fixture();
        let root = fixture.path();
        let id = "WI-9992-post-merge-sync-fixture";
        let archive = root.join(format!(".ai/work-items/archive/{id}"));
        std::fs::create_dir_all(archive.parent().unwrap()).unwrap();
        for (suffix, contents) in [
            (
                "contract.json",
                b"{\"workItemId\":\"fixture\"}\n".as_slice(),
            ),
            ("summary.json", b"summary\n".as_slice()),
            ("outcome.json", b"outcome\n".as_slice()),
            ("task-report.json", b"report\n".as_slice()),
            ("task-report.md", b"report markdown\n".as_slice()),
            ("events.jsonl", b"event\n".as_slice()),
            ("archive.json", b"manifest\n".as_slice()),
        ] {
            std::fs::write(format!("{}.{}", archive.display(), suffix), contents).unwrap();
        }
        let archive_revision = commit(root, "select archive identity");
        super::validate_current_archive_matches_revision(root, id, &archive_revision)
            .expect("live archive records match their immutable archive commit");

        std::fs::write(
            format!("{}.summary.json", archive.display()),
            b"self-consistent replacement summary\n",
        )
        .unwrap();
        assert!(
            super::validate_current_archive_matches_revision(root, id, &archive_revision).is_err(),
            "a self-consistent replacement archive set cannot replace the selected commit blobs"
        );
    }

    #[test]
    fn post_merge_sync_archive_stage_requires_exact_normalized_finish_bytes() {
        let fixture = post_merge_history_fixture();
        let root = fixture.path();
        let id = "WI-9994-post-merge-sync-fixture";
        let active = root.join(format!(".ai/work-items/active/{id}"));
        std::fs::create_dir_all(active.parent().unwrap()).unwrap();
        for (suffix, bytes) in [
            ("contract.json", b"contract\n".as_slice()),
            ("outcome.json", b"{}\n".as_slice()),
            ("summary.json", b"summary\n".as_slice()),
            ("task-report.json", b"{}\n".as_slice()),
            ("task-report.md", b"report\n".as_slice()),
            ("events.jsonl", b"event\n".as_slice()),
        ] {
            std::fs::write(format!("{}.{}", active.display(), suffix), bytes).unwrap();
        }
        let finish = commit(root, "finish stage");

        let archive = root.join(format!(".ai/work-items/archive/{id}"));
        std::fs::create_dir_all(archive.parent().unwrap()).unwrap();
        for suffix in [
            "contract.json",
            "summary.json",
            "task-report.json",
            "task-report.md",
            "events.jsonl",
        ] {
            let bytes = std::fs::read(format!("{}.{}", active.display(), suffix)).unwrap();
            std::fs::write(
                format!("{}.{}", archive.display(), suffix),
                super::super::normalized_archive_artifact_bytes(suffix, &bytes, id).unwrap(),
            )
            .unwrap();
        }
        let report = std::fs::read(format!("{}.task-report.json", archive.display())).unwrap();
        let markdown = std::fs::read(format!("{}.task-report.md", archive.display())).unwrap();
        let mut outcome = serde_json::json!({});
        outcome["taskReportDigest"] = serde_json::json!(Digest::sha256_bytes(&report).to_string());
        outcome["taskReportMarkdownDigest"] =
            serde_json::json!(Digest::sha256_bytes(&markdown).to_string());
        std::fs::write(
            format!("{}.outcome.json", archive.display()),
            serde_json::to_vec_pretty(&outcome).unwrap(),
        )
        .unwrap();
        let archive_revision = commit(root, "archive stage");
        super::validate_archive_stage_bytes(root, id, &finish, &archive_revision)
            .expect("archive stage may copy the exact normal form of finish-stage artifacts");

        std::fs::write(
            format!("{}.summary.json", archive.display()),
            b"substituted summary\n",
        )
        .unwrap();
        let substituted_archive = commit(root, "tamper with archived summary");
        assert!(
            super::validate_archive_stage_bytes(root, id, &finish, &substituted_archive).is_err(),
            "an archive manifest cannot authorize a substituted archived summary blob"
        );
    }

    #[test]
    fn post_merge_sync_document_projection_preserves_pending_state() {
        let id = "WI-9995-post-merge-sync-fixture";
        let old_digest = Digest::sha256_bytes(b"old contract");
        let archive_digest = Digest::sha256_bytes(b"archived contract");
        let before = format!(
            "---\ntitle: Fixture\nworkItemId: {id}\nstatus: in_progress\nlastVerifiedBy: {id}\ncontractDigest: {old_digest}\n---\n\nThis page projects the active [{id} Contract](../../.ai/work-items/active/{id}.contract.json), bound to `{old_digest}`. Runtime records remain authoritative.\n"
        );
        let after = format!(
            "---\ntitle: Fixture\nworkItemId: {id}\nstatus: in_progress\nlastVerifiedBy: {id}\ncontractDigest: {archive_digest}\n---\n\nThis page projects the archived [{id} Contract](../../.ai/work-items/archive/{id}.contract.json), bound to `{archive_digest}`. Runtime records remain authoritative.\n"
        );
        super::validate_work_item_document_projection(
            Path::new("/synthetic/repository"),
            id,
            "",
            &archive_digest,
            &before,
            &after,
        )
        .expect("the archived Contract link and digest may change while the page stays pending");

        let premature_terminal = after.replace("status: in_progress", "status: implemented");
        assert!(
            super::validate_work_item_document_projection(
                Path::new("/synthetic/repository"),
                id,
                "",
                &archive_digest,
                &before,
                &premature_terminal,
            )
            .is_err(),
            "documentation projection must reject premature terminal status"
        );

        let changed_body = after.replace("Runtime records remain authoritative.", "unrelated edit");
        assert!(
            super::validate_work_item_document_projection(
                Path::new("/synthetic/repository"),
                id,
                "",
                &archive_digest,
                &before,
                &changed_body,
            )
            .is_err()
        );
    }

    #[test]
    fn post_merge_sync_docs_projection_accepts_pending_archive_and_rejects_other_row_changes() {
        let fixture = post_merge_history_fixture();
        let root = fixture.path();
        let id = "WI-9993-post-merge-sync-fixture";
        let archive = root.join(format!(".ai/work-items/archive/{id}.contract.json"));
        std::fs::create_dir_all(archive.parent().unwrap()).unwrap();
        let contract = serde_json::json!({"workItemId": id, "scope": ["source.txt"]});
        std::fs::write(&archive, serde_json::to_vec_pretty(&contract).unwrap()).unwrap();
        let archive_digest = super::super::canonical_contract_digest(&contract).unwrap();
        let old_digest = Digest::sha256_bytes(b"active contract digest");
        for (suffix, status, active_label) in [
            (
                "",
                "In progress → Implemented after verified close",
                "active",
            ),
            (".zh-CN", "进行中 → 验证关闭后已实现", "当前"),
            (".ja", "進行中 → 検証済みクローズ後に実装済み", "active な"),
        ] {
            let parity = root.join(format!("docs/reference/reference-parity{suffix}.md"));
            std::fs::create_dir_all(parity.parent().unwrap()).unwrap();
            let before_evidence = format!(
                "[Work Item](../work-items/{id}{suffix}.md); {active_label} Contract `.ai/work-items/active/{id}.contract.json` (`{old_digest}`); planned archive `.ai/work-items/archive/{id}.contract.json`; planned verification `.ai/evidence/{id}.verification.json`; planned finalization `.ai/decisions/{id}.finalize.json`; planned close `.ai/decisions/{id}.close.json`."
            );
            std::fs::write(
                parity,
                format!("| Work Item | Status | Evidence |\n| --- | --- | --- |\n| WI-9993 | {status} | {before_evidence} |\n| WI-8888 | Unrelated | unchanged |\n"),
            )
            .unwrap();
            let document = root.join(format!("docs/work-items/{id}{suffix}.md"));
            std::fs::create_dir_all(document.parent().unwrap()).unwrap();
            let (before_label, before_digest_text) = match suffix {
                ".zh-CN" => ("当前", "，绑定摘要"),
                ".ja" => ("active な", " に結び付いています。"),
                _ => ("active", ", bound to"),
            };
            let body = match suffix {
                ".zh-CN" => format!(
                    "本页投影{before_label} [{id} Contract](../../.ai/work-items/active/{id}.contract.json){before_digest_text} `{old_digest}`。Runtime records remain authoritative.\n"
                ),
                ".ja" => format!(
                    "このページは {before_label} [{id} Contract](../../.ai/work-items/active/{id}.contract.json) の投影であり、`{old_digest}`{before_digest_text}Runtime records remain authoritative.\n"
                ),
                _ => format!(
                    "This page projects the {before_label} [{id} Contract](../../.ai/work-items/active/{id}.contract.json){before_digest_text} `{old_digest}`. Runtime records remain authoritative.\n"
                ),
            };
            std::fs::write(
                document,
                format!("---\ntitle: Fixture\nworkItemId: {id}\nstatus: in_progress\nlastVerifiedBy: {id}\ncontractDigest: {old_digest}\n---\n\n{body}"),
            )
            .unwrap();
        }
        let archive_revision = commit(root, "archive projection source");
        for (suffix, status) in [
            ("", "In progress → Implemented after verified close"),
            (".zh-CN", "进行中 → 验证关闭后已实现"),
            (".ja", "進行中 → 検証済みクローズ後に実装済み"),
        ] {
            let parity = root.join(format!("docs/reference/reference-parity{suffix}.md"));
            let mut current = std::fs::read_to_string(&parity).unwrap();
            let work_item = format!("[Work Item](../work-items/{id}{suffix}.md)");
            let new_evidence = match suffix {
                ".zh-CN" => format!(
                    "{work_item}; archived Contract `.ai/work-items/archive/{id}.contract.json` (canonical Contract digest `{archive_digest}`); 验证 `.ai/evidence/{id}.verification.json` 已通过；归档 `.ai/work-items/archive/{id}.archive.json` 已记录；provider 最终化 `.ai/decisions/{id}.finalize.json` 和真人 close `.ai/decisions/{id}.close.json` 仍待完成."
                ),
                ".ja" => format!(
                    "{work_item}; archived Contract `.ai/work-items/archive/{id}.contract.json` (canonical Contract digest `{archive_digest}`); verification `.ai/evidence/{id}.verification.json` は検証済み; archive `.ai/work-items/archive/{id}.archive.json` は記録済み; provider finalization `.ai/decisions/{id}.finalize.json` と human close `.ai/decisions/{id}.close.json` は未完了."
                ),
                _ => format!(
                    "{work_item}; archived Contract `.ai/work-items/archive/{id}.contract.json` (canonical Contract digest `{archive_digest}`); verification `.ai/evidence/{id}.verification.json` is verified; archive `.ai/work-items/archive/{id}.archive.json` is recorded; provider finalization `.ai/decisions/{id}.finalize.json` and human close `.ai/decisions/{id}.close.json` remain pending."
                ),
            };
            let old_row = current
                .lines()
                .find(|line| line.starts_with("| WI-9993 |"))
                .expect("target fixture row exists");
            let new_row = format!("| WI-9993 | {status} | {new_evidence} |");
            current = current.replace(old_row, &new_row);
            std::fs::write(&parity, current).unwrap();
            let document = root.join(format!("docs/work-items/{id}{suffix}.md"));
            let mut current = std::fs::read_to_string(&document).unwrap();
            let target = format!("../../.ai/work-items/active/{id}.contract.json");
            let archive_target = format!("../../.ai/work-items/archive/{id}.contract.json");
            current = current.replace(&target, &archive_target);
            let (old_label, new_label) = match suffix {
                ".zh-CN" => ("当前 [", "已归档的 ["),
                ".ja" => (" active な [", "アーカイブ済みの ["),
                _ => ("active [", "archived ["),
            };
            current = current.replace(old_label, new_label);
            current = current.replace(&old_digest.to_string(), &archive_digest.to_string());
            std::fs::write(document, current).unwrap();
            let row = format!("| WI-9993 | {status} | ");
            assert!(std::fs::read_to_string(parity).unwrap().contains(&row));
        }
        let docs_revision = commit(root, "sync pending archive documentation");
        super::validate_documentation_projection(root, id, &archive_revision, &docs_revision)
            .expect("the real archive-pending-close projection must pass in all languages");

        let parity = root.join("docs/reference/reference-parity.md");
        let current = std::fs::read_to_string(&parity).unwrap().replace(
            "WI-8888 | Unrelated | unchanged",
            "WI-8888 | Unrelated | changed",
        );
        std::fs::write(parity, current).unwrap();
        let tampered_docs = commit(root, "change unrelated parity row");
        assert!(
            super::validate_documentation_projection(root, id, &archive_revision, &tampered_docs)
                .is_err(),
            "documentation projection must reject unrelated Work Item row changes"
        );
    }

    #[test]
    fn post_merge_sync_accepts_the_real_archive_pending_close_projection() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let archive_revision = git(&root, &["rev-parse", "700f6cd5^{commit}"]);
        let docs_revision = git(&root, &["rev-parse", "0f1d54b3^{commit}"]);
        super::validate_documentation_projection(
            &root,
            "WI-1076-governance-recovered-predecessor-projection",
            &archive_revision,
            &docs_revision,
        )
        .expect("the actual archive-to-docs commit pair preserves the pending close lifecycle");
    }

    #[test]
    fn schema_v2_first_root_must_be_retained() {
        let fixture = tempfile::tempdir().expect("fixture directory");
        let root = fixture.path();
        let common = root.join(".git");
        let worktree = root.join("target-worktree");
        let worktree_git_dir = root.join(".git/worktrees/target-worktree");
        let deleted = deleted_receipt(
            "codex/test-post-merge-sync",
            &worktree,
            &worktree_git_dir,
            &common,
        );
        let deleted_digest =
            cockpit_protocol::digest_json(&serde_json::to_value(&deleted).unwrap()).unwrap();
        assert!(
            super::resolve_resource_finalization_head_from_observed(
                root,
                &deleted.work_item_id,
                deleted.clone(),
                PathBuf::from(".ai/decisions/root.json"),
                deleted_digest,
                Vec::new(),
            )
            .is_err(),
            "a schema-v2 Deleted receipt cannot masquerade as the canonical root"
        );

        let mut retained = deleted;
        retained.result.disposition = ResourceFinalizationDisposition::Retained;
        retained.after.branch = ResourceFinalizationBranchState::Present;
        retained.after.worktree = ResourceFinalizationWorktreeState::Clean;
        retained
            .post_merge_sync
            .as_mut()
            .unwrap()
            .remote_branch_state = PostMergeSyncRemoteBranchState::Present;
        let digest =
            cockpit_protocol::digest_json(&serde_json::to_value(&retained).unwrap()).unwrap();
        let retained_work_item_id = retained.work_item_id.clone();
        super::resolve_resource_finalization_head_from_observed(
            root,
            &retained_work_item_id,
            retained,
            PathBuf::from(".ai/decisions/root.json"),
            digest,
            Vec::new(),
        )
        .expect("a schema-v2 Retained receipt is a valid first root");
    }

    #[test]
    fn post_merge_sync_requires_imported_provider_assurance_without_upgrading_evidence() {
        let fixture = tempfile::tempdir().expect("fixture directory");
        let root = fixture.path();
        let output = Command::new("git")
            .args(["init", "-q"])
            .current_dir(root)
            .status()
            .expect("git init starts");
        assert!(output.success());
        git(root, &["config", "user.name", "AI Cockpit Tests"]);
        git(root, &["config", "user.email", "tests@example.invalid"]);
        std::fs::write(root.join("tracked.txt"), "fixture\n").unwrap();
        git(root, &["add", "tracked.txt"]);
        git(root, &["commit", "-q", "-m", "fixture baseline"]);
        super::super::attach(root).expect("attach fixture repository");
        super::super::start_work_item(
            root,
            "WI-9997-post-merge-sync-fixture",
            "bind post-merge evidence",
            "test the delegated evidence trust boundary",
            &[".ai/**".into()],
        )
        .expect("start Work Item for evidence binding");
        let runtime = RuntimeContext {
            runtime_version: "test-runtime".into(),
            protocol_version: 1,
            runtime_digest: Digest::sha256_bytes(b"test-runtime"),
        };
        let collect_time = (chrono::Utc::now() - chrono::Duration::seconds(10))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let common_dir = super::git_common_dir(root).unwrap();
        let mut receipt = deleted_receipt(
            "codex/test-post-merge-sync",
            &fixture.path().join("target-worktree"),
            &fixture.path().join(".git/worktrees/target-worktree"),
            &common_dir,
        );
        receipt.timestamp = (chrono::Utc::now() + chrono::Duration::minutes(2))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let post_merge_sync = receipt.post_merge_sync.as_ref().unwrap().clone();
        let remote_reference = post_merge_sync.remote_branch_ref.clone();
        let mut retained_root = receipt.clone();
        retained_root.result.disposition = ResourceFinalizationDisposition::Retained;
        retained_root.after.branch = ResourceFinalizationBranchState::Present;
        retained_root.after.worktree = ResourceFinalizationWorktreeState::Clean;
        retained_root.timestamp = (chrono::Utc::now() - chrono::Duration::seconds(60))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        retained_root
            .post_merge_sync
            .as_mut()
            .unwrap()
            .remote_branch_state = PostMergeSyncRemoteBranchState::Present;
        let retained_root_path =
            super::resource_finalization_decision_path(root, "WI-9997-post-merge-sync-fixture");
        std::fs::create_dir_all(retained_root_path.parent().unwrap()).unwrap();
        std::fs::write(
            &retained_root_path,
            serde_json::to_vec_pretty(&retained_root).unwrap(),
        )
        .unwrap();
        let imports = [
            (
                "source",
                post_merge_sync.source_pull_request.url.clone(),
                github_pull_request_origin(&post_merge_sync.source_pull_request),
                github_pull_request_body(&post_merge_sync.source_pull_request, "source"),
                AssuranceLevel::ProviderVerified,
            ),
            (
                "sync",
                post_merge_sync.sync_pull_request.url.clone(),
                github_pull_request_origin(&post_merge_sync.sync_pull_request),
                github_pull_request_body(&post_merge_sync.sync_pull_request, &receipt.branch.name),
                AssuranceLevel::ProviderVerified,
            ),
            (
                "remote",
                remote_reference.clone(),
                github_ref_origin(&remote_reference),
                github_deleted_ref_body(
                    "example/fixture",
                    &remote_reference,
                    "observation-1",
                    "fixture-user",
                    &collect_time,
                ),
                AssuranceLevel::ProviderVerified,
            ),
        ];
        let mut digests = Vec::new();
        for (name, subject, origin, raw, assurance) in imports {
            let evidence = DelegatedEvidence {
                provider: "github".into(),
                subject,
                origin,
                assurance,
                collected_at: collect_time.clone(),
                digest: Digest::sha256_bytes(&raw),
                validity: EvidenceValidity::Valid,
                raw_evidence_ref: format!(".ai/evidence/external/sync-fixture-{name}.json"),
            };
            super::super::import_delegated_evidence(
                root,
                "WI-9997-post-merge-sync-fixture",
                &evidence,
                &raw,
                &runtime,
            )
            .expect("import delegated evidence");
            digests.push(evidence.digest);
        }
        {
            let sync = receipt.post_merge_sync.as_mut().unwrap();
            sync.source_evidence_digest = digests[0].clone();
            sync.sync_evidence_digest = digests[1].clone();
            sync.remote_evidence_digest = digests[2].clone();
        }
        super::validate_post_merge_sync_delegated_evidence(
            root,
            &receipt,
            receipt.post_merge_sync.as_ref().unwrap(),
        )
        .expect("structured provider facts must match both PR identities and exact ref state");

        let import_remote = |name: &str, raw: Vec<u8>, collected_at: String| {
            let evidence = DelegatedEvidence {
                provider: "github".into(),
                subject: remote_reference.clone(),
                origin: github_ref_origin(&remote_reference),
                assurance: AssuranceLevel::ProviderVerified,
                collected_at,
                digest: Digest::sha256_bytes(&raw),
                validity: EvidenceValidity::Valid,
                raw_evidence_ref: format!(".ai/evidence/external/sync-fixture-{name}.json"),
            };
            super::super::import_delegated_evidence(
                root,
                "WI-9997-post-merge-sync-fixture",
                &evidence,
                &raw,
                &runtime,
            )
            .expect("import remote observation fixture");
            evidence.digest
        };

        let mut permission_denied: serde_json::Value =
            serde_json::from_slice(&github_deleted_ref_body(
                "example/fixture",
                &remote_reference,
                "permission-404",
                "fixture-user",
                &collect_time,
            ))
            .unwrap();
        permission_denied["accessCheck"]["httpStatus"] = serde_json::json!(404);
        let permission_digest = import_remote(
            "permission-404",
            serde_json::to_vec(&permission_denied).unwrap(),
            collect_time.clone(),
        );
        let mut permission_receipt = receipt.clone();
        permission_receipt
            .post_merge_sync
            .as_mut()
            .unwrap()
            .remote_evidence_digest = permission_digest;
        assert!(
            super::validate_post_merge_sync_delegated_evidence(
                root,
                &permission_receipt,
                permission_receipt.post_merge_sync.as_ref().unwrap(),
            )
            .is_err(),
            "a permission-denied 404 cannot prove that an accessible exact ref is absent"
        );

        retained_root.timestamp = (chrono::Utc::now() - chrono::Duration::minutes(12))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        std::fs::write(
            &retained_root_path,
            serde_json::to_vec_pretty(&retained_root).unwrap(),
        )
        .unwrap();
        let stale_time = (chrono::Utc::now() - chrono::Duration::minutes(10))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let mut stale_replay: serde_json::Value = serde_json::from_slice(&github_deleted_ref_body(
            "example/fixture",
            &remote_reference,
            "observation-1",
            "fixture-user",
            &stale_time,
        ))
        .unwrap();
        stale_replay["accessCheck"]["refs"] = serde_json::json!([
            {"ref": format!("{remote_reference}/nested"), "object": {"sha": "child"}}
        ]);
        let stale_digest = import_remote(
            "stale-replay-rehashed",
            serde_json::to_vec(&stale_replay).unwrap(),
            stale_time,
        );
        let mut stale_receipt = receipt.clone();
        stale_receipt
            .post_merge_sync
            .as_mut()
            .unwrap()
            .remote_evidence_digest = stale_digest;
        assert!(
            super::validate_post_merge_sync_delegated_evidence(
                root,
                &stale_receipt,
                stale_receipt.post_merge_sync.as_ref().unwrap(),
            )
            .is_err(),
            "rehashing a replayed observation cannot refresh its timestamp"
        );

        let future_time = (chrono::Utc::now() + chrono::Duration::minutes(10))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let future_digest = import_remote(
            "future-observation",
            github_deleted_ref_body(
                "example/fixture",
                &remote_reference,
                "future-observation",
                "fixture-user",
                &future_time,
            ),
            future_time,
        );
        let mut future_receipt = receipt.clone();
        future_receipt
            .post_merge_sync
            .as_mut()
            .unwrap()
            .remote_evidence_digest = future_digest;
        assert!(
            super::validate_post_merge_sync_delegated_evidence(
                root,
                &future_receipt,
                future_receipt.post_merge_sync.as_ref().unwrap(),
            )
            .is_err(),
            "a future-dated provider observation cannot authorize deletion"
        );

        let arbitrary_pr = b"source PR metadata";
        let arbitrary_pr_evidence = DelegatedEvidence {
            provider: "github".into(),
            subject: post_merge_sync.source_pull_request.url.clone(),
            origin: github_pull_request_origin(&post_merge_sync.source_pull_request),
            assurance: AssuranceLevel::ProviderVerified,
            collected_at: collect_time.clone(),
            digest: Digest::sha256_bytes(arbitrary_pr),
            validity: EvidenceValidity::Valid,
            raw_evidence_ref: ".ai/evidence/external/sync-fixture-arbitrary-pr.json".into(),
        };
        super::super::import_delegated_evidence(
            root,
            "WI-9997-post-merge-sync-fixture",
            &arbitrary_pr_evidence,
            arbitrary_pr,
            &runtime,
        )
        .expect("arbitrary bytes remain digest-bound but must not become semantic proof");
        let mut arbitrary_pr_receipt = receipt.clone();
        arbitrary_pr_receipt
            .post_merge_sync
            .as_mut()
            .unwrap()
            .source_evidence_digest = arbitrary_pr_evidence.digest;
        assert!(
            super::validate_post_merge_sync_delegated_evidence(
                root,
                &arbitrary_pr_receipt,
                arbitrary_pr_receipt.post_merge_sync.as_ref().unwrap(),
            )
            .is_err(),
            "arbitrary provider-verified text must not assert a merged source PR"
        );

        let mut wrong_pr_value: serde_json::Value = serde_json::from_slice(
            &github_pull_request_body(&post_merge_sync.source_pull_request, "source"),
        )
        .unwrap();
        wrong_pr_value["head"]["sha"] = serde_json::json!("different-source-head");
        let wrong_pr_bytes = serde_json::to_vec(&wrong_pr_value).unwrap();
        let wrong_pr_evidence = DelegatedEvidence {
            provider: "github".into(),
            subject: post_merge_sync.source_pull_request.url.clone(),
            origin: github_pull_request_origin(&post_merge_sync.source_pull_request),
            assurance: AssuranceLevel::ProviderVerified,
            collected_at: collect_time.clone(),
            digest: Digest::sha256_bytes(&wrong_pr_bytes),
            validity: EvidenceValidity::Valid,
            raw_evidence_ref: ".ai/evidence/external/sync-fixture-wrong-source-head.json".into(),
        };
        super::super::import_delegated_evidence(
            root,
            "WI-9997-post-merge-sync-fixture",
            &wrong_pr_evidence,
            &wrong_pr_bytes,
            &runtime,
        )
        .expect("wrong-head GitHub response remains raw imported evidence");
        let mut wrong_pr_receipt = receipt.clone();
        wrong_pr_receipt
            .post_merge_sync
            .as_mut()
            .unwrap()
            .source_evidence_digest = wrong_pr_evidence.digest;
        assert!(
            super::validate_post_merge_sync_delegated_evidence(
                root,
                &wrong_pr_receipt,
                wrong_pr_receipt.post_merge_sync.as_ref().unwrap(),
            )
            .is_err(),
            "ProviderVerified metadata cannot override a mismatched PR head SHA in the raw body"
        );

        let arbitrary_remote = b"remote branch metadata";
        let arbitrary_remote_evidence = DelegatedEvidence {
            provider: "github".into(),
            subject: remote_reference.clone(),
            origin: github_ref_origin(&remote_reference),
            assurance: AssuranceLevel::ProviderVerified,
            collected_at: collect_time.clone(),
            digest: Digest::sha256_bytes(arbitrary_remote),
            validity: EvidenceValidity::Valid,
            raw_evidence_ref: ".ai/evidence/external/sync-fixture-arbitrary-remote.json".into(),
        };
        super::super::import_delegated_evidence(
            root,
            "WI-9997-post-merge-sync-fixture",
            &arbitrary_remote_evidence,
            arbitrary_remote,
            &runtime,
        )
        .expect("arbitrary ref bytes remain digest-bound");
        let mut arbitrary_remote_receipt = receipt.clone();
        arbitrary_remote_receipt
            .post_merge_sync
            .as_mut()
            .unwrap()
            .remote_evidence_digest = arbitrary_remote_evidence.digest;
        assert!(
            super::validate_post_merge_sync_delegated_evidence(
                root,
                &arbitrary_remote_receipt,
                arbitrary_remote_receipt.post_merge_sync.as_ref().unwrap(),
            )
            .is_err(),
            "arbitrary provider-verified text must not assert an absent remote ref"
        );

        let present_remote = github_present_ref_body(&remote_reference, "head-1");
        let present_remote_evidence = DelegatedEvidence {
            provider: "github".into(),
            subject: remote_reference.clone(),
            origin: github_ref_origin(&remote_reference),
            assurance: AssuranceLevel::ProviderVerified,
            collected_at: collect_time.clone(),
            digest: Digest::sha256_bytes(&present_remote),
            validity: EvidenceValidity::Valid,
            raw_evidence_ref: ".ai/evidence/external/sync-fixture-present-remote.json".into(),
        };
        super::super::import_delegated_evidence(
            root,
            "WI-9997-post-merge-sync-fixture",
            &present_remote_evidence,
            &present_remote,
            &runtime,
        )
        .expect("present ref observation imports as raw provider evidence");
        let mut deleted_from_present_receipt = receipt.clone();
        deleted_from_present_receipt
            .post_merge_sync
            .as_mut()
            .unwrap()
            .remote_evidence_digest = present_remote_evidence.digest.clone();
        assert!(
            super::validate_post_merge_sync_delegated_evidence(
                root,
                &deleted_from_present_receipt,
                deleted_from_present_receipt
                    .post_merge_sync
                    .as_ref()
                    .unwrap(),
            )
            .is_err(),
            "a present ref with any SHA must not authorize Deleted"
        );
        let wrong_sha = github_present_ref_body(&remote_reference, "wrong-sha");
        let wrong_sha_evidence = DelegatedEvidence {
            provider: "github".into(),
            subject: remote_reference.clone(),
            origin: github_ref_origin(&remote_reference),
            assurance: AssuranceLevel::ProviderVerified,
            collected_at: collect_time.clone(),
            digest: Digest::sha256_bytes(&wrong_sha),
            validity: EvidenceValidity::Valid,
            raw_evidence_ref: ".ai/evidence/external/sync-fixture-wrong-sha.json".into(),
        };
        super::super::import_delegated_evidence(
            root,
            "WI-9997-post-merge-sync-fixture",
            &wrong_sha_evidence,
            &wrong_sha,
            &runtime,
        )
        .expect("wrong-SHA observation imports as raw provider evidence");
        let mut present_from_wrong_sha_receipt = receipt.clone();
        present_from_wrong_sha_receipt.result.disposition =
            ResourceFinalizationDisposition::Retained;
        present_from_wrong_sha_receipt
            .post_merge_sync
            .as_mut()
            .unwrap()
            .remote_branch_state = PostMergeSyncRemoteBranchState::Present;
        present_from_wrong_sha_receipt
            .post_merge_sync
            .as_mut()
            .unwrap()
            .remote_evidence_digest = wrong_sha_evidence.digest;
        assert!(
            super::validate_post_merge_sync_delegated_evidence(
                root,
                &present_from_wrong_sha_receipt,
                present_from_wrong_sha_receipt
                    .post_merge_sync
                    .as_ref()
                    .unwrap(),
            )
            .is_err(),
            "Present must bind the exact expected branch head SHA"
        );

        let raw = b"self-declared source metadata";
        let evidence = DelegatedEvidence {
            provider: "github".into(),
            subject: "https://github.com/example/fixture/pull/9001".into(),
            origin: "https://api.github.com/pulls/9001".into(),
            assurance: AssuranceLevel::SelfDeclared,
            collected_at: collect_time,
            digest: Digest::sha256_bytes(raw),
            validity: EvidenceValidity::Valid,
            raw_evidence_ref: ".ai/evidence/external/sync-fixture-self-declared.json".into(),
        };
        let imported = super::super::import_delegated_evidence(
            root,
            "WI-9997-post-merge-sync-fixture",
            &evidence,
            raw,
            &runtime,
        )
        .expect("import SelfDeclared evidence without changing its assurance");
        assert_eq!(imported.evidence.assurance, AssuranceLevel::SelfDeclared);
        receipt
            .post_merge_sync
            .as_mut()
            .unwrap()
            .source_evidence_digest = evidence.digest;
        let sync = receipt.post_merge_sync.as_ref().unwrap().clone();
        assert!(
            super::validate_post_merge_sync_delegated_evidence(root, &receipt, &sync,).is_err(),
            "SelfDeclared evidence must not be promoted to provider-backed proof"
        );
    }

    #[test]
    fn schema_v4_requires_explicit_versioned_observation_selectors_and_keeps_v3_closed() {
        let fixture = tempfile::tempdir().expect("fixture directory");
        let mut receipt = deleted_receipt(
            "codex/v4-fixture",
            &fixture.path().join("removed"),
            &fixture.path().join(".git/worktrees/removed"),
            &fixture.path().join(".git"),
        );
        receipt.schema_version = 4;
        let sync = receipt.post_merge_sync.as_mut().unwrap();
        sync.lifecycle_layout = Some(cockpit_protocol::PostMergeLifecycleLayout::Separate);
        sync.documentation = Some(cockpit_protocol::PostMergeDocumentation::Projected);
        sync.source_pull_request.head_revision = "1".repeat(40);
        sync.source_pull_request.base_revision = "2".repeat(40);
        sync.source_pull_request.merge_commit = Some("3".repeat(40));
        sync.sync_pull_request.head_revision = "4".repeat(40);
        sync.sync_pull_request.base_revision = "5".repeat(40);
        sync.sync_pull_request.merge_commit = Some("6".repeat(40));
        sync.finish_revision = "7".repeat(40);
        sync.archive_revision = "8".repeat(40);
        sync.docs_revision = "4".repeat(40);
        sync.current_main_revision = "6".repeat(40);
        receipt.pull_request = sync.source_pull_request.clone();
        receipt.branch.head_revision = sync.sync_pull_request.head_revision.clone();
        receipt.worktree.head_revision = sync.sync_pull_request.head_revision.clone();
        let mut legacy_v3 = receipt.clone();
        legacy_v3.schema_version =
            cockpit_protocol::RESOURCE_FINALIZATION_LIFECYCLE_LAYOUT_SCHEMA_VERSION;
        let legacy_json = serde_json::to_value(&legacy_v3).expect("legacy v3 receipt JSON");
        assert!(
            !legacy_json["postMergeSync"]
                .as_object()
                .unwrap()
                .contains_key("observationSelectors"),
            "legacy v3 output must not acquire a selector field"
        );
        let selector = |name: &[u8]| {
            serde_json::json!({
                "schemaVersion": 1,
                "observationDigest": Digest::sha256_bytes(name).to_string(),
                "receiptDigest": Digest::sha256_bytes(&[name, b"-receipt"].concat()).to_string()
            })
        };
        let selectors = serde_json::json!({
            "source": selector(b"source"),
            "sync": selector(b"sync"),
            "remote": selector(b"remote")
        });
        let mut value = serde_json::to_value(&receipt).expect("receipt JSON");
        value["schemaVersion"] = serde_json::json!(4);
        value["postMergeSync"]["observationSelectors"] = selectors;

        let parsed: ResourceFinalizationReceipt =
            serde_json::from_value(value.clone()).expect("schema-v4 selectors must parse");
        cockpit_protocol::validate_resource_finalization_receipt(&parsed)
            .expect("schema-v4 receipt with three versioned selectors is valid");

        let mut missing_selector_value = value.clone();
        missing_selector_value["postMergeSync"]
            .as_object_mut()
            .unwrap()
            .remove("observationSelectors");
        let missing_selectors: ResourceFinalizationReceipt =
            serde_json::from_value(missing_selector_value).expect("missing selector parses");
        assert!(
            cockpit_protocol::validate_resource_finalization_receipt(&missing_selectors).is_err(),
            "schema-v4 receipts must require all three exact selectors"
        );

        let mut unsupported_selector_value = value.clone();
        unsupported_selector_value["postMergeSync"]["observationSelectors"]["source"]["schemaVersion"] =
            serde_json::json!(2);
        let unsupported_selector: ResourceFinalizationReceipt =
            serde_json::from_value(unsupported_selector_value).expect("selector parses");
        assert!(
            cockpit_protocol::validate_resource_finalization_receipt(&unsupported_selector)
                .is_err(),
            "unsupported selector versions must fail closed"
        );

        value["schemaVersion"] = serde_json::json!(3);
        let legacy_with_selector: ResourceFinalizationReceipt =
            serde_json::from_value(value).expect("legacy shape remains readable for validation");
        assert!(
            cockpit_protocol::validate_resource_finalization_receipt(&legacy_with_selector)
                .is_err(),
            "v1-v3 receipts must not consume v4 selectors"
        );
    }

    #[test]
    fn schema_v4_deleted_transition_preserves_source_selectors_and_appends_remote_selector() {
        let fixture = tempfile::tempdir().expect("fixture directory");
        let mut retained = deleted_receipt(
            "codex/v4-transition-fixture",
            &fixture.path().join("removed"),
            &fixture.path().join(".git/worktrees/removed"),
            &fixture.path().join(".git"),
        );
        retained.schema_version =
            cockpit_protocol::RESOURCE_FINALIZATION_EVIDENCE_OBSERVATION_SCHEMA_VERSION;
        retained.result.disposition = ResourceFinalizationDisposition::Retained;
        retained.after = retained.before.clone();
        {
            let sync = retained.post_merge_sync.as_mut().unwrap();
            sync.lifecycle_layout = Some(cockpit_protocol::PostMergeLifecycleLayout::Separate);
            sync.documentation = Some(cockpit_protocol::PostMergeDocumentation::Projected);
            sync.source_pull_request.head_revision = "1".repeat(40);
            sync.source_pull_request.base_revision = "2".repeat(40);
            sync.source_pull_request.merge_commit = Some("3".repeat(40));
            sync.sync_pull_request.head_revision = "4".repeat(40);
            sync.sync_pull_request.base_revision = "5".repeat(40);
            sync.sync_pull_request.merge_commit = Some("6".repeat(40));
            sync.finish_revision = "7".repeat(40);
            sync.archive_revision = "8".repeat(40);
            sync.docs_revision = "4".repeat(40);
            sync.current_main_revision = "6".repeat(40);
            sync.remote_branch_state = PostMergeSyncRemoteBranchState::Present;
            sync.observation_selectors = Some(PostMergeSyncEvidenceObservationSelectors {
                source: DelegatedEvidenceObservationSelector {
                    schema_version: 1,
                    observation_digest: Digest::sha256_bytes(b"source observation"),
                    receipt_digest: Digest::sha256_bytes(b"source receipt"),
                },
                sync: DelegatedEvidenceObservationSelector {
                    schema_version: 1,
                    observation_digest: Digest::sha256_bytes(b"sync observation"),
                    receipt_digest: Digest::sha256_bytes(b"sync receipt"),
                },
                remote: DelegatedEvidenceObservationSelector {
                    schema_version: 1,
                    observation_digest: Digest::sha256_bytes(b"remote present observation"),
                    receipt_digest: Digest::sha256_bytes(b"remote present receipt"),
                },
            });
            sync.remote_evidence_digest = Digest::sha256_bytes(b"remote present bytes");
        }
        retained.pull_request = retained
            .post_merge_sync
            .as_ref()
            .unwrap()
            .source_pull_request
            .clone();
        let sync_head = retained
            .post_merge_sync
            .as_ref()
            .unwrap()
            .sync_pull_request
            .head_revision
            .clone();
        retained.branch.head_revision = sync_head.clone();
        retained.worktree.head_revision = sync_head;

        let mut deleted = retained.clone();
        deleted.before = retained.after.clone();
        deleted.after.branch = ResourceFinalizationBranchState::Deleted;
        deleted.after.worktree = ResourceFinalizationWorktreeState::Removed;
        deleted.result.disposition = ResourceFinalizationDisposition::Deleted;
        let next_remote_digest = Digest::sha256_bytes(b"remote deleted bytes");
        let sync = deleted.post_merge_sync.as_mut().unwrap();
        sync.remote_branch_state = PostMergeSyncRemoteBranchState::Deleted;
        sync.remote_evidence_digest = next_remote_digest;
        sync.observation_selectors.as_mut().unwrap().remote =
            DelegatedEvidenceObservationSelector {
                schema_version: 1,
                observation_digest: Digest::sha256_bytes(b"remote deleted observation"),
                receipt_digest: Digest::sha256_bytes(b"remote deleted receipt"),
            };
        let predecessor_value = serde_json::to_value(&retained).expect("retained receipt JSON");
        let transition = ResourceFinalizationTransitionReceipt {
            schema_version: 1,
            transition_id: "transition-1".into(),
            sequence: 1,
            predecessor_receipt_digest: cockpit_protocol::digest_json(&predecessor_value)
                .expect("predecessor digest"),
            governance_append_revision: None,
            receipt: deleted,
        };

        cockpit_protocol::validate_resource_finalization_transition(&retained, &transition, 1)
            .expect("v4 transition preserves exact source/sync observations and appends remote");

        let mut changed_source = transition.clone();
        changed_source
            .receipt
            .post_merge_sync
            .as_mut()
            .unwrap()
            .observation_selectors
            .as_mut()
            .unwrap()
            .source
            .observation_digest = Digest::sha256_bytes(b"changed source selector");
        assert!(
            cockpit_protocol::validate_resource_finalization_transition(
                &retained,
                &changed_source,
                1
            )
            .is_err(),
            "a Deleted transition must not rewrite its source observation selector"
        );
    }

    #[test]
    fn schema_v4_uses_the_selected_corrected_observation_when_raw_digest_is_ambiguous() {
        let fixture = tempfile::tempdir().expect("fixture directory");
        let root = fixture.path();
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .current_dir(root)
                .status()
                .expect("git init")
                .success()
        );
        git(root, &["config", "user.name", "AI Cockpit Tests"]);
        git(root, &["config", "user.email", "tests@example.invalid"]);
        std::fs::write(root.join("tracked.txt"), "fixture\n").unwrap();
        git(root, &["add", "tracked.txt"]);
        git(root, &["commit", "-q", "-m", "fixture baseline"]);
        super::super::attach(root).expect("attach fixture repository");
        super::super::start_work_item_with_options(
            root,
            "WI-9997-post-merge-sync-v4-fixture",
            "bind v4 post-merge evidence",
            "resolve exact observation selectors",
            &[".ai/**".into()],
            &crate::WorkItemStartOptions {
                authority: "authorized".into(),
                ..Default::default()
            },
        )
        .expect("start Work Item for v4 evidence binding");
        let runtime = RuntimeContext {
            runtime_version: "test-runtime".into(),
            protocol_version: 1,
            runtime_digest: Digest::sha256_bytes(b"test-runtime"),
        };
        let collected_at = (chrono::Utc::now() - chrono::Duration::seconds(10))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let remote_ref = "refs/heads/codex/v4-fixture";
        let mut receipt = deleted_receipt(
            "codex/v4-fixture",
            &root.join("target-worktree"),
            &root.join(".git/worktrees/target-worktree"),
            &root.join(".git"),
        );
        receipt.work_item_id = "WI-9997-post-merge-sync-v4-fixture".into();
        receipt.repository_id = super::super::repository_id(root).to_string();
        receipt.schema_version = 4;
        receipt.result.disposition = ResourceFinalizationDisposition::Retained;
        receipt.after = receipt.before.clone();
        receipt.timestamp = (chrono::Utc::now() + chrono::Duration::minutes(1))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let sync = receipt.post_merge_sync.as_mut().unwrap();
        sync.lifecycle_layout = Some(cockpit_protocol::PostMergeLifecycleLayout::Separate);
        sync.documentation = Some(cockpit_protocol::PostMergeDocumentation::Projected);
        sync.source_pull_request.head_revision = "1".repeat(40);
        sync.source_pull_request.base_revision = "2".repeat(40);
        sync.source_pull_request.merge_commit = Some("3".repeat(40));
        sync.sync_pull_request.head_revision = "4".repeat(40);
        sync.sync_pull_request.base_revision = "5".repeat(40);
        sync.sync_pull_request.merge_commit = Some("6".repeat(40));
        sync.finish_revision = "7".repeat(40);
        sync.archive_revision = "8".repeat(40);
        sync.docs_revision = "4".repeat(40);
        sync.current_main_revision = "6".repeat(40);
        sync.remote_branch_ref = remote_ref.into();
        sync.remote_branch_state = PostMergeSyncRemoteBranchState::Present;
        receipt.pull_request = sync.source_pull_request.clone();
        receipt.branch.head_revision = sync.sync_pull_request.head_revision.clone();
        receipt.worktree.head_revision = sync.sync_pull_request.head_revision.clone();

        let source_identity = sync.source_pull_request.clone();
        let source_raw = github_pull_request_body(&source_identity, "source");
        let incorrect_source = DelegatedEvidence {
            provider: "github".into(),
            subject: source_identity.url.clone(),
            origin: "https://api.github.com/repos/example/fixture/issues/9001".into(),
            assurance: AssuranceLevel::ProviderVerified,
            collected_at: collected_at.clone(),
            digest: Digest::sha256_bytes(&source_raw),
            validity: EvidenceValidity::Valid,
            raw_evidence_ref: ".ai/evidence/external/v4-source.json".into(),
        };
        let incorrect_observation = super::super::import_delegated_evidence_observation(
            root,
            &receipt.work_item_id,
            &incorrect_source,
            &source_raw,
            &runtime,
        )
        .expect("retain the original incorrect source observation");
        let corrected_source = DelegatedEvidence {
            origin: github_pull_request_origin(&source_identity),
            ..incorrect_source.clone()
        };
        let corrected_observation = super::super::import_delegated_evidence_observation(
            root,
            &receipt.work_item_id,
            &corrected_source,
            &source_raw,
            &runtime,
        )
        .expect("append the corrected source with the same raw digest");
        assert_eq!(incorrect_source.digest, corrected_source.digest);
        assert_ne!(
            incorrect_observation.selector.observation_digest,
            corrected_observation.selector.observation_digest
        );

        let sync_identity = sync.sync_pull_request.clone();
        let sync_raw = github_pull_request_body(&sync_identity, &receipt.branch.name);
        let sync_evidence = DelegatedEvidence {
            provider: "github".into(),
            subject: sync_identity.url.clone(),
            origin: github_pull_request_origin(&sync_identity),
            assurance: AssuranceLevel::ProviderVerified,
            collected_at: collected_at.clone(),
            digest: Digest::sha256_bytes(&sync_raw),
            validity: EvidenceValidity::Valid,
            raw_evidence_ref: ".ai/evidence/external/v4-sync.json".into(),
        };
        let sync_observation = super::super::import_delegated_evidence_observation(
            root,
            &receipt.work_item_id,
            &sync_evidence,
            &sync_raw,
            &runtime,
        )
        .expect("append sync observation");
        let remote_raw = github_present_ref_body(remote_ref, &"4".repeat(40));
        let remote_evidence = DelegatedEvidence {
            provider: "github".into(),
            subject: remote_ref.into(),
            origin: github_ref_origin(remote_ref),
            assurance: AssuranceLevel::ProviderVerified,
            collected_at: collected_at.clone(),
            digest: Digest::sha256_bytes(&remote_raw),
            validity: EvidenceValidity::Valid,
            raw_evidence_ref: ".ai/evidence/external/v4-remote.json".into(),
        };
        let remote_observation = super::super::import_delegated_evidence_observation(
            root,
            &receipt.work_item_id,
            &remote_evidence,
            &remote_raw,
            &runtime,
        )
        .expect("append remote observation");

        let mut value = serde_json::to_value(&receipt).expect("v4 receipt JSON");
        value["postMergeSync"]["sourceEvidenceDigest"] =
            serde_json::json!(corrected_source.digest.to_string());
        value["postMergeSync"]["syncEvidenceDigest"] =
            serde_json::json!(sync_evidence.digest.to_string());
        value["postMergeSync"]["remoteEvidenceDigest"] =
            serde_json::json!(remote_evidence.digest.to_string());
        value["postMergeSync"]["observationSelectors"] = serde_json::json!({
            "source": corrected_observation.selector,
            "sync": sync_observation.selector,
            "remote": remote_observation.selector
        });
        let v4_receipt: ResourceFinalizationReceipt =
            serde_json::from_value(value.clone()).expect("schema-v4 selectors parse");
        cockpit_protocol::validate_resource_finalization_receipt(&v4_receipt)
            .expect("schema-v4 identity is valid");
        super::validate_post_merge_sync_delegated_evidence(
            root,
            &v4_receipt,
            v4_receipt.post_merge_sync.as_ref().unwrap(),
        )
        .expect("schema-v4 evidence validation must use the corrected exact selector");

        value["postMergeSync"]["observationSelectors"]["source"] =
            serde_json::to_value(&incorrect_observation.selector).unwrap();
        let wrong_selector_receipt: ResourceFinalizationReceipt =
            serde_json::from_value(value).expect("wrong selector remains structurally valid");
        assert!(
            super::validate_post_merge_sync_delegated_evidence(
                root,
                &wrong_selector_receipt,
                wrong_selector_receipt.post_merge_sync.as_ref().unwrap(),
            )
            .is_err(),
            "the preserved incorrect observation cannot be selected as source proof"
        );
    }

    #[test]
    fn schema_v2_deleted_requires_branch_worktree_and_gitdir_absence() {
        let fixture = tempfile::tempdir().expect("fixture directory");
        let root = fixture.path().join("control");
        let worktree = fixture.path().join("target-worktree");
        std::fs::create_dir_all(&root).expect("control checkout directory");
        let output = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .status()
            .expect("git init starts");
        assert!(output.success());
        git(&root, &["config", "user.name", "AI Cockpit Tests"]);
        git(&root, &["config", "user.email", "tests@example.invalid"]);
        std::fs::write(root.join("tracked.txt"), "fixture\n").unwrap();
        git(&root, &["add", "tracked.txt"]);
        git(&root, &["commit", "-q", "-m", "fixture baseline"]);
        git(&root, &["branch", "-M", "main"]);
        git(
            &root,
            &[
                "worktree",
                "add",
                "-b",
                "codex/sync-fixture-post-merge-proof",
                worktree.to_str().unwrap(),
                "HEAD",
            ],
        );
        let git_file = std::fs::read_to_string(worktree.join(".git")).unwrap();
        let worktree_git_dir = PathBuf::from(git_file.trim().strip_prefix("gitdir: ").unwrap());
        let common_dir = git(
            &root,
            &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        );
        git(
            &root,
            &["worktree", "remove", "--force", worktree.to_str().unwrap()],
        );
        git(
            &root,
            &["branch", "-D", "codex/sync-fixture-post-merge-proof"],
        );

        let receipt = deleted_receipt(
            "codex/sync-fixture-post-merge-proof",
            &worktree,
            &worktree_git_dir,
            Path::new(&common_dir),
        );
        assert!(local_resources_deleted(&root, &receipt).unwrap());

        std::fs::create_dir(&worktree).unwrap();
        assert!(!local_resources_deleted(&root, &receipt).unwrap());
        std::fs::remove_dir(&worktree).unwrap();

        std::fs::create_dir_all(&worktree_git_dir).unwrap();
        assert!(!local_resources_deleted(&root, &receipt).unwrap());
        std::fs::remove_dir_all(&worktree_git_dir).unwrap();

        git(
            &root,
            &["branch", "codex/sync-fixture-post-merge-proof", "HEAD"],
        );
        assert!(!local_resources_deleted(&root, &receipt).unwrap());
    }

    #[test]
    fn schema_v2_retained_requires_exact_local_worktree_identity() {
        let fixture = tempfile::tempdir().expect("fixture directory");
        let root = fixture.path().join("control");
        let worktree = fixture.path().join("target-worktree");
        std::fs::create_dir_all(&root).expect("control checkout directory");
        let output = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .status()
            .expect("git init starts");
        assert!(output.success());
        git(&root, &["config", "user.name", "AI Cockpit Tests"]);
        git(&root, &["config", "user.email", "tests@example.invalid"]);
        std::fs::write(root.join("tracked.txt"), "fixture\n").unwrap();
        git(&root, &["add", "tracked.txt"]);
        git(&root, &["commit", "-q", "-m", "fixture baseline"]);
        git(&root, &["branch", "-M", "main"]);
        let branch = "codex/sync-fixture-retained-proof";
        git(
            &root,
            &[
                "worktree",
                "add",
                "-b",
                branch,
                worktree.to_str().unwrap(),
                "HEAD",
            ],
        );
        let head = git(&worktree, &["rev-parse", "--verify", "HEAD"]);
        let git_file = std::fs::read_to_string(worktree.join(".git")).unwrap();
        let worktree_git_dir = PathBuf::from(git_file.trim().strip_prefix("gitdir: ").unwrap());
        let common_dir = super::git_common_dir(&root).unwrap();
        let mut receipt = deleted_receipt(branch, &worktree, &worktree_git_dir, &common_dir);
        receipt.branch.head_revision = head.clone();
        receipt.worktree.head_revision = head.clone();
        receipt.before.branch = ResourceFinalizationBranchState::Present;
        receipt.before.worktree = ResourceFinalizationWorktreeState::Clean;
        receipt.after.branch = ResourceFinalizationBranchState::Present;
        receipt.after.worktree = ResourceFinalizationWorktreeState::Clean;
        receipt.result.disposition = ResourceFinalizationDisposition::Retained;
        let sync = receipt.post_merge_sync.as_mut().unwrap();
        sync.sync_pull_request.head_revision = head;
        sync.docs_revision = sync.sync_pull_request.head_revision.clone();
        sync.remote_branch_state = PostMergeSyncRemoteBranchState::Present;
        let sync = receipt.post_merge_sync.as_ref().unwrap().clone();
        validate_post_merge_sync_local_state(&root, &receipt, &sync)
            .expect("retained worktree identity should match the receipt");

        let mut wrong_git_dir = sync.clone();
        wrong_git_dir.worktree_git_dir = fixture.path().join("wrong-git-dir").display().to_string();
        assert!(validate_post_merge_sync_local_state(&root, &receipt, &wrong_git_dir).is_err());

        std::fs::write(worktree.join("untracked.txt"), "dirty\n").unwrap();
        assert!(validate_post_merge_sync_local_state(&root, &receipt, &sync).is_err());
    }
}

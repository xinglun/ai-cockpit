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
    Contract, FinalizationErrorCode, HistoricalFinalizationKind,
    HistoricalFinalizationRecoveryReceipt, ResourceFinalizationContext,
    ResourceFinalizationDisposition, ResourceFinalizationReceipt,
    ResourceFinalizationTransitionReceipt, RuntimeContext,
    validate_historical_finalization_recovery, validate_resource_finalization_receipt_for,
    validate_resource_finalization_replay, validate_resource_finalization_transition,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn plan_resource_finalization(
    root: &Path,
    work_item_id: &str,
    context: &ResourceFinalizationContext,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
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
        contract.resource_context.as_ref(),
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
        let kind = receipt
            .historical
            .as_ref()
            .map(|historical| match historical.kind {
                HistoricalFinalizationKind::SharedWorktreeRetained => "shared_worktree_retained",
                HistoricalFinalizationKind::DirectMergeNoPr => "direct_merge_no_pr",
            })
            .or_else(|| {
                contract.as_ref().and_then(|(contract, _)| {
                    infer_legacy_shared_worktree_retained(&root, &receipt, contract)
                        .then_some("shared_worktree_retained")
                })
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
        } else {
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
            contract.resource_context.as_ref(),
        )
        .map_err(|error| ObserverError::State {
            path: receipt_path.into(),
            message: error.to_string(),
        })?;
    }
    validate_historical_finalization(&root, &receipt, receipt_path)?;
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
        if let Some(transition) = transition {
            if close_present {
                validate_post_close_finalization_reconciliation(
                    &root,
                    work_item_id,
                    &close_path,
                    &existing,
                    &head_path,
                    sequence,
                    &transition,
                )?;
            }
            validate_resource_finalization_transition(&existing, &transition, sequence + 1)
                .map_err(|error| ObserverError::State {
                    path: receipt_path.into(),
                    message: error.to_string(),
                })?;
            validate_governance_append_revision(
                &root,
                work_item_id,
                &existing,
                &transition,
                receipt_path,
            )?;
            let value =
                serde_json::to_value(&transition).map_err(|error| ObserverError::State {
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
    Ok(true)
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
    let (contract, finalization_contract_digest) = archived_contract_digest(&root, work_item_id)
        .map_err(|error| {
            finalization_observation_error(
                root.join(".ai/work-items/archive")
                    .join(format!("{work_item_id}.contract.json")),
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
        contract.resource_context.as_ref(),
    )
    .map_err(|error| {
        finalization_observation_error(path.clone(), error.finalization_error_code(), error)
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

struct ValidatedOrdinaryCleanupReceipt {
    receipt: OrdinaryCleanupReceipt,
    digest: Digest,
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

fn ordinary_cleanup_receipt_head(
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
    if contract.resource_context.is_some() {
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
    if contract.resource_context.is_some() {
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

use super::{
    AttachedProfile, DecisionState, Digest, HistoricalDebtItem,
    HistoricalFinalizationInventoryItem, ObserverError, OutcomeState, RepositoryConfig,
    RepositoryReadiness, RepositorySnapshot, RepositoryStatus, ResourceFinalizationReceipt,
    ResourceFinalizationTransitionReceipt, RuntimeContext, WorkItemActionExplanation,
    WorkItemActionIssue, WorkItemActionIssueKind, WorkItemAdmissionState,
    WorkItemEvidenceFreshness, WorkItemStatusIndex, WorkItemStatusIndexEntry,
    WorkItemStatusSnapshot, active_artifact_variants, archived_contract_digest,
    close_decision_is_valid_for_status, closed_finalization_projection_kind, contract_digest,
    count_suffix, git_text, infer_legacy_shared_worktree_retained, is_regular_non_symlink,
    legacy_verification_evidence, load_recovery_decision, orphaned_active_artifact_names,
    outcome_state_name, outcome_v2_internal_with_snapshot, read_contract, read_json,
    read_resource_finalization_transition, repository_id, repository_relative_path,
    resolve_resource_finalization_head_with_index, resource_cleanup_completion_state,
    resource_finalization_decision_path,
    selected_successor_lineage_recovery_resolves_pending_close, snapshot_digest,
    validate_protocol_version, validate_work_item_id, verify_archive_manifest,
    verify_resource_finalization_internal,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn status(root: &Path) -> Result<RepositoryStatus, ObserverError> {
    status_with_runtime(root, None)
}

/// Read repository status with the executing Runtime identity so stale
/// finalization records can be surfaced as explicit historical debt.
pub fn status_with_runtime(
    root: &Path,
    runtime: Option<&RuntimeContext>,
) -> Result<RepositoryStatus, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let ai = root.join(".ai");
    let config_path = ai.join("cockpit.toml");
    let config_text = fs::read_to_string(&config_path).map_err(|source| ObserverError::Read {
        path: config_path.clone(),
        source,
    })?;
    let config: RepositoryConfig =
        toml::from_str(&config_text).map_err(|error| ObserverError::State {
            path: config_path.clone(),
            message: error.to_string(),
        })?;
    validate_protocol_version(config.protocol_version).map_err(|error| ObserverError::State {
        path: config_path.clone(),
        message: error.to_string(),
    })?;
    let profile_bytes =
        fs::read(ai.join("project.json")).map_err(|source| ObserverError::Read {
            path: ai.join("project.json"),
            source,
        })?;
    let profile: AttachedProfile =
        serde_json::from_slice(&profile_bytes).map_err(|error| ObserverError::State {
            path: ai.join("project.json"),
            message: error.to_string(),
        })?;
    if profile.repository_id != config.repository_id
        || config.repository_id.parse::<Digest>().is_err()
    {
        return Err(ObserverError::State {
            path: config_path,
            message: "repository identity does not match protocol state".into(),
        });
    }
    // Capture one immutable snapshot for the complete status projection.  The
    // readiness projection used to capture a second snapshot, duplicating
    // four Git subprocesses on every `status` request.
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let finalization_transition_index = runtime
        .map(|_| FinalizationTransitionIndex::from_directory(&root))
        .transpose()?;
    let active_artifacts = active_artifact_variants(&ai.join("work-items/active"))?
        .into_iter()
        .map(|variant| variant.name)
        .collect::<Vec<_>>();
    let orphaned_active_artifacts = orphaned_active_artifact_names(&root)?;
    let readiness = repository_readiness_from_snapshot_with_runtime(
        &root,
        &snapshot,
        &config.repository_id,
        runtime,
        finalization_transition_index.as_ref(),
    )?;
    Ok(RepositoryStatus {
        protocol_version: config.protocol_version,
        repository_schema_version: config.repository_schema_version,
        repository_id: profile.repository_id,
        state: profile.state,
        profile_version: profile.profile_version,
        active_work_items: count_suffix(&ai.join("work-items/active"), ".contract.json"),
        archived_work_items: count_suffix(&ai.join("work-items/archive"), ".archive.json"),
        active_artifacts,
        orphaned_active_artifacts,
        readiness,
    })
}

/// Readiness is a deterministic, read-only projection used before entering a
/// new Work Item.  It deliberately does not become a process-global
/// scheduler: every invocation resolves one repository root and one fresh
/// snapshot.  Missing remote metadata is represented as `unknown`, never as
/// a green `ready_on_base` claim.
pub(super) fn repository_readiness(root: &Path) -> Result<RepositoryReadiness, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let expected_repository_id = repository_id(&root).to_string();
    repository_readiness_from_snapshot(&root, &snapshot, &expected_repository_id)
}

fn repository_readiness_from_snapshot(
    root: &Path,
    snapshot: &RepositorySnapshot,
    expected_repository_id: &str,
) -> Result<RepositoryReadiness, ObserverError> {
    repository_readiness_from_snapshot_with_runtime(
        root,
        snapshot,
        expected_repository_id,
        None,
        None,
    )
}

fn repository_readiness_from_snapshot_with_runtime(
    root: &Path,
    snapshot: &RepositorySnapshot,
    expected_repository_id: &str,
    runtime: Option<&RuntimeContext>,
    finalization_transition_index: Option<&FinalizationTransitionIndex>,
) -> Result<RepositoryReadiness, ObserverError> {
    let current_branch = git_text(root, &["symbolic-ref", "--quiet", "--short", "HEAD"])
        .filter(|value| !value.is_empty());
    let current_revision = snapshot.head.clone();
    let default_base = discover_default_base(root);
    let dirty_paths = non_governance_changed_paths(snapshot);
    let unclosed_archived_work_items =
        unclosed_archived_work_items_with_id(root, expected_repository_id)?;
    let historical_debt = classify_historical_debt(root, &unclosed_archived_work_items);
    let historical_finalization = match (runtime, finalization_transition_index) {
        (Some(runtime), Some(index)) => {
            historical_finalization_inventory_with_index(root, runtime, index)?
        }
        _ => historical_finalization_inventory(root, runtime)?,
    };
    let active_work_items = count_suffix(&root.join(".ai/work-items/active"), ".contract.json");
    let orphaned_active_artifacts = orphaned_active_artifact_names(root)?;

    let mut blockers = Vec::new();
    if active_work_items > 0 {
        blockers.push("active_work_items_present".into());
    }
    if !orphaned_active_artifacts.is_empty() {
        blockers.push("orphaned_active_artifacts_present".into());
    }
    if !dirty_paths.is_empty() {
        blockers.push("working_tree_dirty_before_start".into());
    }
    if current_branch.is_none() {
        blockers.push("detached_head".into());
    }
    if let (Some(current), Some(default)) = (&current_revision, &default_base)
        && current != &default.revision
    {
        blockers.push("base_revision_not_synchronized".into());
    }

    let mut unknowns = Vec::new();
    if default_base.is_none() {
        unknowns.push("default_base_unknown".into());
    }
    if current_revision.is_none() {
        unknowns.push("current_revision_unknown".into());
    }
    if blockers.is_empty() && current_branch.is_some() && default_base.is_some() {
        unknowns.clear();
    }
    blockers.sort();
    blockers.dedup();
    unknowns.sort();
    unknowns.dedup();
    let ready_on_base = blockers.is_empty() && unknowns.is_empty();
    let state = if !blockers.is_empty() {
        "blocked"
    } else if !unknowns.is_empty() {
        "unknown"
    } else {
        "ready_on_base"
    };
    Ok(RepositoryReadiness {
        state: state.into(),
        ready_on_base,
        blockers,
        unknowns,
        current_branch,
        default_remote: default_base.as_ref().map(|base| base.remote.clone()),
        default_branch: default_base.as_ref().map(|base| base.branch.clone()),
        current_revision,
        default_revision: default_base.map(|base| base.revision),
        dirty_paths,
        unclosed_archived_work_items,
        historical_debt,
        orphaned_active_artifacts,
        historical_finalization,
    })
}

#[derive(Clone, Debug)]
pub(super) enum IndexedFinalizationTransition {
    Valid {
        path: PathBuf,
        digest: Digest,
        value: Box<ResourceFinalizationTransitionReceipt>,
    },
    Invalid {
        path: PathBuf,
        message: String,
    },
}

#[derive(Clone, Debug, Default)]
pub(super) struct FinalizationTransitionIndex {
    pub(super) by_work_item: BTreeMap<String, Vec<IndexedFinalizationTransition>>,
    #[cfg(test)]
    parsed_transition_count: usize,
}

impl FinalizationTransitionIndex {
    fn from_directory(root: &Path) -> Result<Self, ObserverError> {
        let decisions = root.join(".ai/decisions");
        let entries = match fs::read_dir(&decisions) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(source) => {
                return Err(ObserverError::Read {
                    path: decisions,
                    source,
                });
            }
        };
        let mut index = Self::default();
        for entry in entries {
            let entry = entry.map_err(|source| ObserverError::Read {
                path: decisions.clone(),
                source,
            })?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some((work_item_id, suffix)) = name.split_once(".finalize.") else {
                continue;
            };
            if suffix == "json"
                || !suffix.ends_with(".json")
                || validate_work_item_id(work_item_id).is_err()
            {
                continue;
            }
            let path = entry.path();
            let indexed = match read_resource_finalization_transition(&path) {
                Ok(value) => match serde_json::to_value(&value)
                    .map_err(|error| error.to_string())
                    .and_then(|encoded| {
                        cockpit_protocol::digest_json(&encoded).map_err(|error| error.to_string())
                    }) {
                    Ok(digest) => {
                        let expected = format!(
                            "{work_item_id}.finalize.{}.json",
                            digest
                                .to_string()
                                .strip_prefix("sha256:")
                                .unwrap_or_default()
                        );
                        if name == expected {
                            #[cfg(test)]
                            {
                                index.parsed_transition_count += 1;
                            }
                            IndexedFinalizationTransition::Valid {
                                path,
                                digest,
                                value: Box::new(value),
                            }
                        } else {
                            IndexedFinalizationTransition::Invalid {
                                path,
                                message:
                                    "resource finalization transition filename digest mismatch"
                                        .into(),
                            }
                        }
                    }
                    Err(message) => IndexedFinalizationTransition::Invalid { path, message },
                },
                Err(error) => IndexedFinalizationTransition::Invalid {
                    path,
                    message: error.to_string(),
                },
            };
            index
                .by_work_item
                .entry(work_item_id.to_owned())
                .or_default()
                .push(indexed);
        }
        for candidates in index.by_work_item.values_mut() {
            candidates.sort_by(|left, right| {
                let left_path = match left {
                    IndexedFinalizationTransition::Valid { path, .. }
                    | IndexedFinalizationTransition::Invalid { path, .. } => path,
                };
                let right_path = match right {
                    IndexedFinalizationTransition::Valid { path, .. }
                    | IndexedFinalizationTransition::Invalid { path, .. } => path,
                };
                left_path.cmp(right_path)
            });
        }
        Ok(index)
    }

    pub(super) fn candidates(&self, work_item_id: &str) -> &[IndexedFinalizationTransition] {
        self.by_work_item
            .get(work_item_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    #[cfg(test)]
    fn parsed_transition_count(&self) -> usize {
        self.parsed_transition_count
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DefaultBaseRef {
    remote: String,
    branch: String,
    revision: String,
}

/// Resolve the locally known remote default branch without network access.
/// A missing or ambiguous symbolic ref is intentionally unknown; guessing
/// `main`/`master` would turn an unproven base into authorization.
fn discover_default_base(root: &Path) -> Option<DefaultBaseRef> {
    let mut candidates = Vec::new();
    // Resolve every remote HEAD in one Git invocation. The previous remote →
    // symbolic-ref → rev-parse loop spawned three subprocesses per remote on
    // every readiness/status request. An ambiguous result remains unknown.
    let refs = git_text(
        root,
        &[
            "for-each-ref",
            "--format=%(refname)%00%(symref)%00%(objectname)",
            "refs/remotes/*/HEAD",
        ],
    )?;
    for line in refs.lines() {
        let mut fields = line.split('\0');
        let Some(refname) = fields
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let Some(remote) = refname
            .strip_prefix("refs/remotes/")
            .and_then(|value| value.strip_suffix("/HEAD"))
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let Some(symbolic) = fields
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let Some(revision) = fields
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let prefix = format!("refs/remotes/{remote}/");
        let Some(branch) = symbolic.strip_prefix(&prefix).map(str::trim) else {
            continue;
        };
        if branch.is_empty() {
            continue;
        }
        candidates.push(DefaultBaseRef {
            remote: remote.into(),
            branch: branch.into(),
            revision: revision.into(),
        });
    }
    if candidates.len() == 1 {
        candidates.pop()
    } else {
        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct WorktreeLayout {
    pub(super) primary: PathBuf,
    pub(super) paths: Vec<PathBuf>,
}

/// Read Git's worktree topology without relying on the process cwd.  The
/// first entry is Git's primary worktree; linked worktrees are explicit
/// execution resources and must not be confused with that repository root.
pub(super) fn discover_worktree_layout(root: &Path) -> Result<WorktreeLayout, ObserverError> {
    let raw = git_text(root, &["worktree", "list", "--porcelain"]).ok_or_else(|| {
        ObserverError::State {
            path: root.to_path_buf(),
            message: "cannot determine Git worktree topology".into(),
        }
    })?;
    let mut paths = Vec::new();
    for line in raw.lines() {
        let Some(path) = line.strip_prefix("worktree ") else {
            continue;
        };
        let path = fs::canonicalize(path).map_err(|source| ObserverError::Read {
            path: PathBuf::from(path),
            source,
        })?;
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    let Some(primary) = paths.first().cloned() else {
        return Err(ObserverError::State {
            path: root.to_path_buf(),
            message: "Git returned no primary worktree".into(),
        });
    };
    Ok(WorktreeLayout { primary, paths })
}

fn non_governance_changed_paths(snapshot: &RepositorySnapshot) -> Vec<String> {
    let mut paths = snapshot
        .changed_paths
        .iter()
        .filter(|path| !path.starts_with(".ai/") && path.as_str() != ".ai")
        .cloned()
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    paths
}

pub(super) fn unclosed_archived_work_items(root: &Path) -> Result<Vec<String>, ObserverError> {
    let expected_repository_id = repository_id(root).to_string();
    unclosed_archived_work_items_with_id(root, &expected_repository_id)
}

fn unclosed_archived_work_items_with_id(
    root: &Path,
    expected_repository_id: &str,
) -> Result<Vec<String>, ObserverError> {
    let archive = root.join(".ai/work-items/archive");
    let entries = match fs::read_dir(&archive) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(ObserverError::Read {
                path: archive,
                source,
            });
        }
    };
    let mut archived_ids = std::collections::BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: archive.clone(),
            source,
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if let Some(work_item_id) = name.strip_suffix(".archive.json") {
            archived_ids.insert(work_item_id.to_owned());
        } else if let Some(work_item_id) = name.strip_suffix(".contract.json") {
            // A partially written archive is still an unresolved lifecycle
            // boundary; do not let a missing manifest hide it from entry.
            archived_ids.insert(work_item_id.to_owned());
        }
    }
    let mut pending = archived_ids
        .into_iter()
        .filter(|work_item_id| {
            archive_requires_close(root, work_item_id)
                && !close_decision_is_valid_for_status(root, work_item_id, expected_repository_id)
                && !recovery_successor_resolves_pending_close(
                    root,
                    work_item_id,
                    expected_repository_id,
                )
                && !selected_successor_lineage_recovery_resolves_pending_close(
                    root,
                    work_item_id,
                    expected_repository_id,
                )
        })
        .collect::<Vec<_>>();
    pending.sort();
    pending.dedup();
    Ok(pending)
}

/// A predecessor with an explicit recovery successor is no longer a live
/// close obligation once that successor has reached its own verified,
/// repository-bound terminal boundary.  The recovery receipt is still
/// validated through the normal append-only loader; this helper only decides
/// whether the predecessor should remain in the repository-level entry gate.
/// Missing, stale, foreign, malformed, or incomplete successor evidence keeps
/// the predecessor blocking (fail closed).
pub(super) fn recovery_successor_resolves_pending_close(
    root: &Path,
    predecessor_id: &str,
    expected_repository_id: &str,
) -> bool {
    let Ok(Some(recovery)) = load_recovery_decision(root, predecessor_id, None) else {
        return false;
    };
    if !matches!(recovery.decision.as_str(), "successor" | "supersede")
        || recovery.repository_id != expected_repository_id
    {
        return false;
    }
    let Some(successor_id) = recovery.successor_work_item_id.as_deref() else {
        return false;
    };

    let archive_root = root.join(".ai/work-items/archive");
    let contract_path = archive_root.join(format!("{successor_id}.contract.json"));
    let manifest_path = archive_root.join(format!("{successor_id}.archive.json"));
    let summary_path = archive_root.join(format!("{successor_id}.summary.json"));
    let outcome_path = archive_root.join(format!("{successor_id}.outcome.json"));
    if !is_regular_non_symlink(&contract_path).unwrap_or(false)
        || !is_regular_non_symlink(&manifest_path).unwrap_or(false)
        || !is_regular_non_symlink(&summary_path).unwrap_or(false)
        || !is_regular_non_symlink(&outcome_path).unwrap_or(false)
    {
        return false;
    }
    let Ok(contract) = read_contract(&contract_path) else {
        return false;
    };
    if contract.work_item_id != successor_id || contract.repository_id != expected_repository_id {
        return false;
    }
    let Ok(manifest) = read_json(&manifest_path) else {
        return false;
    };
    if verify_archive_manifest(root, successor_id, &manifest).is_err() {
        return false;
    }
    let Ok(summary) = read_json(&summary_path) else {
        return false;
    };
    if summary["workItemId"] != serde_json::json!(successor_id)
        || summary["state"] != serde_json::json!("finish_ready")
        || summary["checkpointCount"] != serde_json::json!(1)
        || summary["preflightState"] != serde_json::json!("green")
    {
        return false;
    }
    let Ok(outcome) = read_json(&outcome_path) else {
        return false;
    };
    if outcome["workItemId"] != serde_json::json!(successor_id)
        || outcome["verification"]["status"] != serde_json::json!("verified")
    {
        return false;
    }
    close_decision_is_valid_for_status(root, successor_id, expected_repository_id)
}

fn classify_historical_debt(root: &Path, work_items: &[String]) -> Vec<HistoricalDebtItem> {
    let mut result = Vec::new();
    for work_item_id in work_items {
        let finalize = root
            .join(".ai/decisions")
            .join(format!("{work_item_id}.finalize.json"));
        let value = read_json(&finalize).ok();
        let historical_kind = value
            .as_ref()
            .and_then(|item| item.get("historical"))
            .and_then(|item| item.get("kind"))
            .and_then(serde_json::Value::as_str);
        let recovery = root
            .join(".ai/decisions")
            .join(format!("{work_item_id}.finalize-recovery.json"));
        let recovery_kind = read_json(&recovery).ok().and_then(|item| {
            item.get("historicalKind")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        });
        let disposition = value
            .as_ref()
            .and_then(|item| item.get("result"))
            .and_then(|item| item.get("disposition"))
            .and_then(serde_json::Value::as_str);
        let (category, recovery_action) = match recovery_kind.as_deref().or(historical_kind) {
            Some("shared_worktree_retained") => (
                "historical_shared_worktree_retained",
                "verify the historical recovery binding, then close with explicit human decision",
            ),
            Some("direct_merge_no_pr") => (
                "historical_direct_merge_no_pr",
                "verify the direct-merge receipt against Git, then close with explicit human decision",
            ),
            _ if disposition == Some("retained") => (
                "legacy_retained_requires_classification",
                "inspect immutable receipt and append an explicit historical compatibility record; do not edit the predecessor",
            ),
            _ => (
                "legacy_missing_or_invalid_finalization",
                "create a recovery Work Item and attach a human-authorized historical recovery decision",
            ),
        };
        result.push(HistoricalDebtItem {
            work_item_id: work_item_id.clone(),
            category: category.into(),
            assurance: "historical_low".into(),
            recovery_action: recovery_action.into(),
        });
    }
    result
}

/// Inventory stale finalization heads without mutating any repository state.
/// Only a newer executing Runtime can decide whether a producer identity is
/// stale; callers that do not have one retain the historical schema-only plan.
pub(super) fn historical_finalization_inventory(
    root: &Path,
    runtime: Option<&RuntimeContext>,
) -> Result<Vec<HistoricalFinalizationInventoryItem>, ObserverError> {
    let Some(runtime) = runtime else {
        return Ok(Vec::new());
    };
    let index = FinalizationTransitionIndex::from_directory(root)?;
    historical_finalization_inventory_with_index(root, runtime, &index)
}

fn historical_finalization_inventory_with_index(
    root: &Path,
    runtime: &RuntimeContext,
    index: &FinalizationTransitionIndex,
) -> Result<Vec<HistoricalFinalizationInventoryItem>, ObserverError> {
    let decisions = root.join(".ai/decisions");
    let entries = match fs::read_dir(&decisions) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(ObserverError::Read {
                path: decisions,
                source,
            });
        }
    };
    let repository_id = repository_id(root).to_string();
    let mut inventory = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: decisions.clone(),
            source,
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(work_item_id) = name.strip_suffix(".finalize.json") else {
            continue;
        };
        if validate_work_item_id(work_item_id).is_err() {
            continue;
        }
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
            continue;
        }
        let bytes = fs::read(&path).map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        let raw_digest = Digest::sha256_bytes(&bytes);
        let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
            inventory.push(HistoricalFinalizationInventoryItem {
                work_item_id: work_item_id.into(),
                state: "invalid".into(),
                assurance: "unknown".into(),
                predecessor_path: repository_relative_path(root, &path),
                predecessor_digest: Some(raw_digest),
                sequence: 0,
                runtime_version: None,
                runtime_digest: None,
                historical_kind: None,
                safe_actions: vec![format!(
                    "inspect malformed finalization receipt before recovery: {work_item_id}"
                )],
            });
            continue;
        };
        let Ok(receipt) = serde_json::from_value::<ResourceFinalizationReceipt>(value) else {
            inventory.push(HistoricalFinalizationInventoryItem {
                work_item_id: work_item_id.into(),
                state: "invalid".into(),
                assurance: "unknown".into(),
                predecessor_path: repository_relative_path(root, &path),
                predecessor_digest: Some(raw_digest),
                sequence: 0,
                runtime_version: None,
                runtime_digest: None,
                historical_kind: None,
                safe_actions: vec![format!(
                    "inspect legacy finalization receipt schema before recovery: {work_item_id}"
                )],
            });
            continue;
        };
        if receipt.runtime_version == runtime.runtime_version
            && receipt.runtime_digest == runtime.runtime_digest
        {
            continue;
        }
        let Ok((head, head_path, head_digest, sequence)) =
            resolve_resource_finalization_head_with_index(root, work_item_id, index)
        else {
            inventory.push(HistoricalFinalizationInventoryItem {
                work_item_id: work_item_id.into(),
                state: "recovery_required".into(),
                assurance: "unknown".into(),
                predecessor_path: repository_relative_path(root, &path),
                predecessor_digest: Some(raw_digest),
                sequence: 0,
                runtime_version: Some(receipt.runtime_version.clone()),
                runtime_digest: Some(receipt.runtime_digest.clone()),
                historical_kind: None,
                safe_actions: vec![format!(
                    "inspect finalization history before recovery: {work_item_id}"
                )],
            });
            continue;
        };
        let kind = closed_finalization_projection_kind(
            root,
            work_item_id,
            &head,
            &head_path,
            &head_digest,
            sequence,
            &repository_id,
        );
        let inferred_kind = kind.or_else(|| {
            archived_contract_digest(root, work_item_id)
                .ok()
                .and_then(|(contract, _)| {
                    infer_legacy_shared_worktree_retained(root, &head, &contract)
                        .then_some("shared_worktree_retained")
                })
        });
        let state = if kind.is_some() {
            "historical_verified"
        } else {
            "recovery_required"
        };
        let safe_actions = if kind.is_some() {
            vec!["read-only historical projection; no migration required".into()]
        } else if inferred_kind == Some("shared_worktree_retained") {
            vec![
                format!("review historical recovery plan: work-item {work_item_id}"),
                format!("record explicit historical recovery before close: {work_item_id}"),
            ]
        } else {
            vec![format!(
                "inspect immutable finalization facts before recovery: {work_item_id}"
            )]
        };
        inventory.push(HistoricalFinalizationInventoryItem {
            work_item_id: work_item_id.into(),
            state: state.into(),
            assurance: if kind.is_some() {
                "historical_low".into()
            } else {
                "unknown".into()
            },
            predecessor_path: repository_relative_path(root, &head_path),
            predecessor_digest: Some(head_digest),
            sequence,
            runtime_version: Some(head.runtime_version),
            runtime_digest: Some(head.runtime_digest),
            historical_kind: inferred_kind.map(str::to_owned),
            safe_actions,
        });
    }
    inventory.sort_by(|left, right| left.work_item_id.cmp(&right.work_item_id));
    Ok(inventory)
}

/// New archive manifests explicitly opt into the close gate.  Older archive
/// bytes predate that gate and remain historical, so they do not deadlock
/// entry into a new Work Item. Superseded archives are resolved by their
/// recovery successor and never require a second close decision.
fn archive_requires_close(root: &Path, work_item_id: &str) -> bool {
    let path = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.archive.json"));
    let Ok(manifest) = read_json(&path) else {
        // If an archive marker is missing or corrupt, its close policy cannot
        // be proven. Keep the partial archive visible so scope validation can
        // fail closed. Valid historical manifests without closeRequired still
        // retain their legacy, non-blocking behavior below.
        return true;
    };
    manifest.get("state").and_then(serde_json::Value::as_str) == Some("archived")
        && manifest
            .get("closeRequired")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
}

fn action_issue_kind(code: &str) -> WorkItemActionIssueKind {
    let normalized = code.to_ascii_lowercase();
    if normalized.contains("malformed") || normalized.contains("invalid") {
        WorkItemActionIssueKind::Malformed
    } else if normalized.contains("unsupported") {
        WorkItemActionIssueKind::Unsupported
    } else if normalized.contains("contradict") {
        WorkItemActionIssueKind::Contradictory
    } else if normalized.contains("missing") || normalized.contains("required") {
        WorkItemActionIssueKind::Missing
    } else {
        WorkItemActionIssueKind::Unknown
    }
}

fn action_issue(code: &str, message: impl Into<String>) -> WorkItemActionIssue {
    WorkItemActionIssue {
        kind: action_issue_kind(code),
        code: code.to_owned(),
        message: message.into(),
    }
}

fn work_item_action_explanation(
    repository_id: &str,
    work_item_id: &str,
    contract_digest: &Digest,
    snapshot_digest: &Digest,
    runtime: &RuntimeContext,
    operation: Option<&str>,
    resource_bound: bool,
    lifecycle_phase: &str,
    verification: &str,
    blocking: bool,
    human_decision_required: bool,
    blockers: &[String],
    unknowns: &[String],
    missing_inputs: &[String],
    evidence_freshness: &WorkItemEvidenceFreshness,
    safe_actions: &[String],
    recommended_action: Option<String>,
    malformed_evidence: bool,
) -> Result<WorkItemActionExplanation, ObserverError> {
    let operation_lower = operation.unwrap_or_default().to_ascii_lowercase();
    let failed_verification = matches!(verification, "partial" | "unknown")
        || evidence_freshness.state == "stale_or_invalid"
        || unknowns.iter().any(|unknown| {
            let normalized = unknown.to_ascii_lowercase();
            normalized.contains("evidence_stale")
                || normalized.contains("evidence_contradictory")
                || normalized.contains("evidence_unknown")
                || normalized.contains("lifecycle_gate_failed")
        })
        || malformed_evidence;
    let guide_id = if failed_verification {
        "verification-failure-recovery"
    } else if operation_lower.contains("release") || operation_lower.contains("upgrade") {
        "release-upgrade-acceptance"
    } else if resource_bound {
        "provider-resource-finalization"
    } else {
        "ordinary-work-item"
    };
    let admission_state = if human_decision_required {
        WorkItemAdmissionState::NeedsHumanDecision
    } else if blocking {
        WorkItemAdmissionState::Blocked
    } else if recommended_action.is_some() {
        WorkItemAdmissionState::Allowed
    } else {
        WorkItemAdmissionState::Unknown
    };
    let recommendation_reason = match admission_state {
        WorkItemAdmissionState::Allowed => {
            format!("Runtime admits the next safe action for lifecycle phase {lifecycle_phase}")
        }
        WorkItemAdmissionState::NeedsHumanDecision => {
            "Runtime requires a current human decision before continuing".into()
        }
        WorkItemAdmissionState::Blocked => format!(
            "Runtime blocks continuation because current blockers are: {}",
            blockers.join(", ")
        ),
        WorkItemAdmissionState::Unknown => {
            "Runtime cannot identify an admitted next action from current evidence".into()
        }
    };
    let mut issues = unknowns
        .iter()
        .map(|unknown| action_issue(unknown, format!("current projection reports {unknown}")))
        .collect::<Vec<_>>();
    if malformed_evidence
        && !issues
            .iter()
            .any(|issue| issue.code == "verification_evidence_malformed")
    {
        issues.push(action_issue(
            "verification_evidence_malformed",
            "verification evidence is not valid JSON",
        ));
    }
    issues.sort_by(|left, right| left.code.cmp(&right.code));
    issues.dedup_by(|left, right| left.code == right.code);
    let admission_digest = cockpit_protocol::digest_json(&serde_json::json!({
        "repositoryId": repository_id,
        "workItemId": work_item_id,
        "contractDigest": contract_digest,
        "snapshotDigest": snapshot_digest,
        "runtimeDigest": runtime.runtime_digest,
        "safeActions": safe_actions,
        "blockers": blockers,
        "unknowns": unknowns,
        "evidenceFreshness": evidence_freshness,
    }))
    .map_err(|error| ObserverError::State {
        path: PathBuf::from(".ai/work-items"),
        message: format!("action admission digest failed: {error}"),
    })?;
    Ok(WorkItemActionExplanation {
        guide_id: guide_id.into(),
        recommended_action,
        recommendation_reason,
        admission_state,
        issues,
        human_decision_required,
        missing_inputs: missing_inputs.to_vec(),
        admission_digest,
    })
}

/// Derive a request-scoped Work Item status without writing any repository
/// state.  This is intentionally a projection over the existing Contract,
/// Summary, Outcome, and evidence records; it is not a second scheduler or
/// governance authority.
pub fn work_item_status_snapshot_with_runtime(
    root: &Path,
    work_item_id: &str,
    runtime: &RuntimeContext,
) -> Result<WorkItemStatusSnapshot, ObserverError> {
    work_item_status_snapshot_with_snapshot(root, work_item_id, runtime, None)
}

/// Project one Work Item using a snapshot captured by the caller.  The
/// aggregate status path uses this to avoid recapturing the same Git snapshot
/// once per Work Item; the snapshot and its digest remain request-scoped and
/// are never shared across repositories or processes.
fn work_item_status_snapshot_with_snapshot(
    root: &Path,
    work_item_id: &str,
    runtime: &RuntimeContext,
    snapshot_override: Option<(&RepositorySnapshot, &Digest)>,
) -> Result<WorkItemStatusSnapshot, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let active = root.join(".ai/work-items/active");
    let archive = root.join(".ai/work-items/archive");
    let (contract_path, archived) = [
        (active.join(format!("{work_item_id}.contract.json")), false),
        (archive.join(format!("{work_item_id}.contract.json")), true),
    ]
    .into_iter()
    .find(|(path, _)| path.is_file())
    .ok_or_else(|| ObserverError::State {
        path: active.join(format!("{work_item_id}.contract.json")),
        message: "work item contract not found".into(),
    })?;
    let contract = read_contract(&contract_path)?;
    let expected_repository_id = repository_id(&root).to_string();
    if contract.repository_id != expected_repository_id {
        return Err(ObserverError::State {
            path: contract_path,
            message: format!(
                "contract repository identity mismatch: expected {expected_repository_id}, found {}",
                contract.repository_id
            ),
        });
    }
    let base_commit = contract
        .base_commit
        .clone()
        .unwrap_or_else(|| contract.base_revision.clone());
    let branch = contract
        .resource_context
        .as_ref()
        .map(|context| context.branch.clone());
    let (owned_snapshot, snapshot_digest_value) = if let Some((_, provided_digest)) =
        snapshot_override
    {
        (None, provided_digest.clone())
    } else {
        let git =
            cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
                path: root.clone(),
                message: error.to_string(),
            })?;
        let captured_snapshot = git.snapshot().map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
        let captured_digest = snapshot_digest(&captured_snapshot)?;
        (Some(captured_snapshot), captured_digest)
    };
    let snapshot_ref = snapshot_override
        .map(|(provided_snapshot, _)| provided_snapshot)
        .or_else(|| owned_snapshot.as_ref());
    let outcome =
        outcome_v2_internal_with_snapshot(&root, work_item_id, Some(runtime), snapshot_override)?;
    let summary_path = contract_path
        .parent()
        .unwrap_or(&active)
        .join(format!("{work_item_id}.summary.json"));
    let summary = read_json(&summary_path).unwrap_or_else(|_| serde_json::json!({}));
    let close_decision_path = root
        .join(".ai/decisions")
        .join(format!("{work_item_id}.close.json"));
    let close_decision_present = fs::symlink_metadata(&close_decision_path).is_ok();
    let close_decision_valid = archived
        && close_decision_is_valid_for_status(&root, work_item_id, &contract.repository_id);
    // An older Runtime may have left an immutable, non-canonical close
    // decision behind even though its explicitly bound successor has since
    // completed the current lifecycle.  Treat that exact lineage as
    // recovered, not as an ordinary close and not as an unresolved close
    // obligation.  A missing close, incomplete successor, or invalid
    // recovery remains fail-closed below.
    let historical_recovery_resolved = archived
        && close_decision_present
        && !close_decision_valid
        && recovery_successor_resolves_pending_close(&root, work_item_id, &contract.repository_id);
    let lifecycle_phase = if close_decision_valid {
        "closed".to_string()
    } else if historical_recovery_resolved {
        "recovered".to_string()
    } else if archived {
        "archived".to_string()
    } else {
        summary["state"]
            .as_str()
            .or_else(|| Some(outcome_state_name(&outcome.state)))
            .unwrap_or(if archived { "archived" } else { "unknown" })
            .to_string()
    };
    let governance_state = match outcome.decision_state {
        Some(DecisionState::Green) => "green",
        Some(DecisionState::Yellow) => "yellow",
        Some(DecisionState::Red) => "red",
        None => "unknown",
    }
    .to_string();
    let verification = match outcome.state {
        OutcomeState::Verified => "verified",
        OutcomeState::Partial => "partial",
        OutcomeState::NotReady => "not_ready",
        OutcomeState::Unknown => "unknown",
    }
    .to_string();
    let historical = legacy_verification_evidence(&root, work_item_id);
    let activity_health = if historical {
        "historical"
    } else if verification == "unknown" {
        "degraded"
    } else if verification == "not_ready" {
        "waiting"
    } else if archived {
        "inactive"
    } else {
        "active"
    }
    .to_string();

    let acceptance_total = contract.acceptance_criteria.len() as u64;
    let acceptance_evidence = summary["acceptanceEvidence"]
        .as_object()
        .map(|value| value.len() as u64)
        .unwrap_or_default();
    let mut progress_facts = BTreeMap::new();
    progress_facts.insert("acceptanceCriteriaDeclared".into(), acceptance_total);
    progress_facts.insert("acceptanceEvidenceEntries".into(), acceptance_evidence);
    progress_facts.insert(
        "checkpointCount".into(),
        summary["checkpointCount"].as_u64().unwrap_or_default(),
    );
    progress_facts.insert(
        "changedPathCount".into(),
        summary["changedPaths"]
            .as_array()
            .map(|value| value.len() as u64)
            .unwrap_or_default(),
    );

    let mut unknowns = outcome.unknowns.clone();
    if historical_recovery_resolved {
        // The preserved historical close is intentionally non-canonical, but
        // its invalidity is already resolved by the exact recovery lineage;
        // expose the preservation fact below instead of reporting the same
        // close as an unresolved current failure.
        unknowns.retain(|unknown| unknown != "close_decision_invalid");
    }
    if historical {
        unknowns.push("legacy_evidence_historical".into());
    }
    if historical_recovery_resolved {
        unknowns.push("historical_close_decision_preserved".into());
    } else if archived && !close_decision_valid {
        if close_decision_present {
            unknowns.push("close_decision_invalid".into());
        } else {
            unknowns.push("close_decision_pending".into());
        }
    }
    unknowns.sort();
    unknowns.dedup();
    let mut blockers = Vec::new();
    if governance_state == "red" {
        blockers.push("governance_red".into());
    }
    if archived && !close_decision_valid && !historical_recovery_resolved {
        blockers.push("archived_work_item_pending_close".into());
    }
    let blocking = !blockers.is_empty();
    let preflight_human_decision_required =
        summary["preflightState"] == "yellow" && summary["decisionEvidence"].is_null();
    let missing_evidence = unknowns
        .iter()
        .filter(|value| value.contains("evidence") || value.contains("verification"))
        .cloned()
        .collect::<Vec<_>>();
    let dependencies = summary["dependencies"]
        .as_array()
        .or_else(|| summary["dependsOn"].as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    let mut risks = Vec::new();
    if !contract.risk.trim().is_empty() {
        risks.push(contract.risk.clone());
    }
    if let Some(items) = summary["risks"].as_array() {
        risks.extend(
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned)),
        );
    }
    risks.sort();
    risks.dedup();
    let mut completion_domains = BTreeMap::new();
    completion_domains.insert(
        "implementation".into(),
        if matches!(
            lifecycle_phase.as_str(),
            "implementation_active" | "checkpointed"
        ) {
            "active"
        } else if archived {
            "recorded"
        } else {
            "not_started"
        }
        .into(),
    );
    completion_domains.insert("verification".into(), verification.clone());
    completion_domains.insert("workResult".into(), verification.clone());
    completion_domains.insert(
        "resourceCleanup".into(),
        resource_cleanup_completion_state(
            &root,
            work_item_id,
            &contract,
            close_decision_valid,
            runtime,
        ),
    );
    completion_domains.insert(
        "review".into(),
        if governance_state == "green" {
            "available"
        } else {
            "required"
        }
        .into(),
    );
    completion_domains.insert(
        "integration".into(),
        if archived {
            "recorded"
        } else {
            "not_applicable"
        }
        .into(),
    );
    completion_domains.insert(
        "closure".into(),
        if close_decision_valid {
            "closed"
        } else if historical_recovery_resolved {
            "recovered"
        } else if archived {
            "archived"
        } else {
            "open"
        }
        .into(),
    );
    let mut governance_permissions = vec!["read_status".into(), "read_outcome".into()];
    if governance_state == "green" && !historical {
        governance_permissions.push("review_evidence".into());
    }
    let contract_digest_value = contract_digest(&contract_path)?;
    let mut source_digests = BTreeMap::new();
    source_digests.insert("contract".into(), contract_digest_value.clone());
    source_digests.insert("repositorySnapshot".into(), snapshot_digest_value.clone());
    if summary_path.is_file()
        && let Ok(digest) = cockpit_protocol::digest_json(&summary)
    {
        source_digests.insert("summary".into(), digest);
    }
    let evidence_path = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.verification.json"));
    let evidence = read_json(&evidence_path).ok();
    let malformed_evidence = evidence_path.is_file() && evidence.is_none();
    if let Some(evidence) = &evidence
        && let Ok(digest) = cockpit_protocol::digest_json(&evidence)
    {
        source_digests.insert("verificationEvidence".into(), digest);
    }
    let last_verification_at = evidence
        .as_ref()
        .and_then(|value| value["createdAt"].as_str())
        .map(str::to_owned);
    let evidence_freshness = if historical {
        WorkItemEvidenceFreshness {
            state: "historical".into(),
            reason: "verification evidence is immutable historical input and was not revalidated"
                .into(),
        }
    } else if evidence.is_none() {
        WorkItemEvidenceFreshness {
            state: "missing".into(),
            reason: "verification evidence is missing".into(),
        }
    } else if verification == "verified" {
        WorkItemEvidenceFreshness {
            state: "fresh".into(),
            reason: "verification evidence matches the current repository and Runtime bindings"
                .into(),
        }
    } else {
        WorkItemEvidenceFreshness {
            state: "stale_or_invalid".into(),
            reason: "verification evidence exists but does not authorize the current status".into(),
        }
    };
    let updated_at = summary["updatedAt"]
        .as_str()
        .or_else(|| summary["createdAt"].as_str())
        .or(contract.created_at.as_deref())
        .map(str::to_owned);
    let human_decisions = if close_decision_valid {
        vec!["close_decision_recorded".into()]
    } else {
        Vec::new()
    };
    let mut diagnostics = Vec::new();
    if historical {
        diagnostics.push("historical_evidence_not_revalidated".into());
    }
    if historical_recovery_resolved {
        diagnostics.push("historical_close_decision_preserved".into());
    } else if archived && !close_decision_valid {
        diagnostics.push(if close_decision_present {
            "close_decision_not_accepted".into()
        } else {
            "lifecycle_cleanup_required".into()
        });
    }
    let mut safe_actions = if historical_recovery_resolved {
        vec!["read_outcome".into()]
    } else if archived && !close_decision_valid {
        let mut actions = Vec::new();
        if contract.resource_context.is_some() {
            let finalization_path = resource_finalization_decision_path(&root, work_item_id);
            let finalization_state = if fs::symlink_metadata(&finalization_path).is_err() {
                "missing"
            } else {
                match verify_resource_finalization_internal(&root, work_item_id, Some(runtime)) {
                    Ok(value) if value["disposition"].as_str() == Some("deleted") => "deleted",
                    Ok(_) => "retained",
                    Err(_) => "invalid",
                }
            };
            match finalization_state {
                "deleted" => {
                    actions.extend(["finalize_verify", "close"].into_iter().map(str::to_owned))
                }
                "retained" => actions.extend(
                    [
                        "cleanup_resources",
                        "record_finalization",
                        "finalize_verify",
                        "close_after_cleanup",
                    ]
                    .into_iter()
                    .map(str::to_owned),
                ),
                "missing" => actions.extend(
                    [
                        "finalize_resources",
                        "record_finalization",
                        "finalize_verify",
                        "close_after_cleanup",
                    ]
                    .into_iter()
                    .map(str::to_owned),
                ),
                _ => actions.extend(
                    [
                        "repair_finalization",
                        "finalize_verify",
                        "close_after_cleanup",
                    ]
                    .into_iter()
                    .map(str::to_owned),
                ),
            }
        } else {
            actions.push("close_after_review".into());
        }
        actions
    } else if blocking {
        vec!["resolve_blockers".into(), "stop".into()]
    } else {
        match lifecycle_phase.as_str() {
            "implementation_active" => vec!["run_preflight".into()],
            "checkpointed" if verification != "verified" => vec!["run_verification".into()],
            // A verified checkpoint is ready for finish, while the existing
            // verification entrypoint still admits an explicit revalidation
            // or receipt-reuse request. Keep finish first so the projection's
            // recommendation remains the ordinary success path.
            "checkpointed" => vec!["finish".into(), "run_verification".into()],
            "finish_ready" => vec!["archive_when_reviewed".into()],
            "archived" => vec!["read_outcome".into()],
            "closed" => Vec::new(),
            _ if verification != "verified" => vec!["run_verification".into()],
            _ => vec!["read_outcome".into()],
        }
    };
    let verification_precondition_error = if safe_actions
        .iter()
        .any(|action| action == "run_verification")
        && !archived
    {
        snapshot_ref.and_then(|snapshot| {
            super::check_verification_preconditions(root.as_path(), work_item_id, runtime, snapshot)
                .err()
                .map(|error| error.to_string())
        })
    } else {
        None
    };
    if verification_precondition_error.is_some() {
        safe_actions.retain(|action| action != "run_verification");
    }
    if let Some(error) = &verification_precondition_error {
        unknowns.push("verification_action_preconditions_blocked".into());
        diagnostics.push(error.clone());
    }
    unknowns.sort();
    unknowns.dedup();
    let recommended_action = safe_actions
        .iter()
        .find(|action| action.as_str() != "refresh_status")
        .cloned()
        .or_else(|| safe_actions.first().cloned());
    safe_actions.push("refresh_status".into());
    safe_actions.sort();
    safe_actions.dedup();
    // A yellow preflight may still explicitly admit the first typed
    // verification or its one replacement execution. That action is a
    // governed evidence collection step, not a human decision boundary.
    let human_decision_required = preflight_human_decision_required
        && !safe_actions
            .iter()
            .any(|action| action == "run_verification");
    let action_explanation = work_item_action_explanation(
        &contract.repository_id,
        work_item_id,
        &contract_digest_value,
        &snapshot_digest_value,
        runtime,
        contract.operation.as_deref(),
        contract.resource_context.is_some(),
        &lifecycle_phase,
        &verification,
        blocking,
        human_decision_required,
        &blockers,
        &unknowns,
        &missing_evidence,
        &evidence_freshness,
        &safe_actions,
        recommended_action,
        malformed_evidence,
    )?;
    let status_digest = cockpit_protocol::digest_json(&serde_json::json!({
        "schemaVersion": 1,
        "repositoryId": contract.repository_id,
        "workItemId": work_item_id,
        "baseCommit": base_commit,
        "branch": branch,
        "lifecyclePhase": lifecycle_phase,
        "governanceState": governance_state,
        "activityHealth": activity_health,
        "blocking": blocking,
        "humanDecisionRequired": human_decision_required,
        "progressFacts": progress_facts,
        "blockers": blockers,
        "missingEvidence": missing_evidence,
        "dependencies": dependencies,
        "humanDecisions": human_decisions,
        "risks": risks,
        "verification": verification,
        "completionDomains": completion_domains,
        "governancePermissions": governance_permissions,
        "sourceDigests": source_digests,
        "unknowns": unknowns,
        "diagnostics": diagnostics,
        "snapshotDigest": snapshot_digest_value,
        "evidenceFreshness": evidence_freshness,
        "lastVerificationAt": last_verification_at,
        "updatedAt": updated_at,
        "safeActions": safe_actions,
        "actionExplanation": action_explanation,
        "historical": historical,
    }))
    .map_err(|error| ObserverError::State {
        path: contract_path.clone(),
        message: error.to_string(),
    })?;
    Ok(WorkItemStatusSnapshot {
        schema_version: 1,
        repository_id: contract.repository_id,
        work_item_id: work_item_id.into(),
        base_commit,
        branch,
        lifecycle_phase,
        governance_state,
        activity_health,
        blocking,
        human_decision_required,
        progress_facts,
        blockers,
        missing_evidence,
        dependencies,
        human_decisions,
        risks,
        verification,
        completion_domains,
        governance_permissions,
        source_digests,
        unknowns,
        diagnostics,
        snapshot_digest: snapshot_digest_value,
        evidence_freshness,
        last_verification_at,
        updated_at,
        safe_actions,
        action_explanation: Some(action_explanation),
        status_digest,
        historical,
    })
}

/// Aggregate every active and archived Work Item into a stable read-only
/// projection. An unreadable member is retained as an explicit unknown entry;
/// it cannot hide other members or promote any count to green.
pub fn work_item_status_index_with_runtime(
    root: &Path,
    runtime: &RuntimeContext,
) -> Result<WorkItemStatusIndex, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let repository_snapshot_digest = snapshot_digest(&snapshot)?;
    let expected_repository_id = repository_id(&root).to_string();

    let mut work_item_ids = BTreeMap::<String, ()>::new();
    let mut index_unknowns = Vec::new();
    let mut index_diagnostics = Vec::new();
    for relative in [".ai/work-items/active", ".ai/work-items/archive"] {
        let directory = root.join(relative);
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                index_unknowns.push(format!("work_item_directory_unreadable:{relative}"));
                index_diagnostics
                    .push(format!("work_item_directory_unreadable:{relative}:{error}"));
                continue;
            }
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let Some(work_item_id) = name.strip_suffix(".contract.json") else {
                continue;
            };
            let is_regular = fs::symlink_metadata(entry.path())
                .ok()
                .is_some_and(|metadata| metadata.file_type().is_file());
            if is_regular {
                work_item_ids.insert(work_item_id.to_string(), ());
            } else {
                index_unknowns.push(format!("contract_not_regular:{work_item_id}"));
            }
        }
    }

    let mut counts = BTreeMap::from([
        ("green".into(), 0_u64),
        ("red".into(), 0_u64),
        ("unknown".into(), 0_u64),
        ("yellow".into(), 0_u64),
    ]);
    let mut items = Vec::with_capacity(work_item_ids.len());
    for work_item_id in work_item_ids.keys() {
        let entry = match work_item_status_snapshot_with_snapshot(
            &root,
            work_item_id,
            runtime,
            Some((&snapshot, &repository_snapshot_digest)),
        ) {
            Ok(status) => {
                let status_digest = status.status_digest.clone();
                WorkItemStatusIndexEntry {
                    work_item_id: work_item_id.clone(),
                    governance_state: status.governance_state.clone(),
                    status_digest,
                    unknowns: status.unknowns.clone(),
                    diagnostics: status.diagnostics.clone(),
                    status: Some(status),
                }
            }
            Err(error) => {
                let unknowns = vec!["status_projection_failed".into()];
                let diagnostics = vec![format!("status_projection_failed:{error}")];
                let stable = serde_json::json!({
                    "workItemId": work_item_id,
                    "governanceState": "unknown",
                    "unknowns": unknowns,
                    "diagnostics": diagnostics,
                });
                let status_digest = cockpit_protocol::digest_json(&stable).map_err(|error| {
                    ObserverError::State {
                        path: root.clone(),
                        message: error.to_string(),
                    }
                })?;
                WorkItemStatusIndexEntry {
                    work_item_id: work_item_id.clone(),
                    governance_state: "unknown".into(),
                    status_digest,
                    status: None,
                    unknowns,
                    diagnostics,
                }
            }
        };
        *counts.entry(entry.governance_state.clone()).or_default() += 1;
        items.push(entry);
    }
    index_unknowns.sort();
    index_unknowns.dedup();
    index_diagnostics.sort();
    index_diagnostics.dedup();
    index_diagnostics.push(format!("work_items_aggregated:{}", items.len()));
    let stable = serde_json::json!({
        "schemaVersion": 1,
        "repositoryId": expected_repository_id,
        "snapshotDigest": repository_snapshot_digest,
        "counts": counts,
        "items": items,
        "unknowns": index_unknowns,
        "diagnostics": index_diagnostics,
    });
    let index_digest =
        cockpit_protocol::digest_json(&stable).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;

    Ok(WorkItemStatusIndex {
        schema_version: 1,
        repository_id: expected_repository_id,
        snapshot_digest: repository_snapshot_digest,
        counts,
        items,
        unknowns: index_unknowns,
        diagnostics: index_diagnostics,
        index_digest,
    })
}

#[cfg(test)]
mod transition_index_tests {
    use super::*;
    use serde_json::json;

    fn legacy_receipt(work_item_id: &str, repository_id: &str) -> serde_json::Value {
        json!({
            "schemaVersion": 1,
            "receiptId": format!("{work_item_id}-receipt"),
            "operationId": format!("{work_item_id}-operation"),
            "repositoryId": repository_id,
            "workItemId": work_item_id,
            "runtimeVersion": "legacy-runtime",
            "runtimeDigest": Digest::sha256_bytes(b"legacy-runtime"),
            "provider": "github",
            "pullRequest": {
                "number": 1,
                "url": format!("https://example.invalid/{work_item_id}"),
                "headRevision": "head",
                "baseBranch": "main",
                "baseRemote": "origin",
                "baseRevision": "base"
            },
            "branch": {
                "name": format!("feature/{work_item_id}"),
                "remote": "origin",
                "headRevision": "head"
            },
            "worktree": {
                "worktreeId": format!("worktree-{work_item_id}"),
                "path": format!("/tmp/{work_item_id}"),
                "branch": format!("feature/{work_item_id}"),
                "headRevision": "head"
            },
            "before": {
                "pullRequest": "unmerged",
                "branch": "present",
                "worktree": "clean"
            },
            "after": {
                "pullRequest": "unmerged",
                "branch": "present",
                "worktree": "clean"
            },
            "result": {
                "disposition": "blocked",
                "failureCodes": ["unmerged_pull_request"],
                "unknownCodes": []
            },
            "actor": "human:test",
            "authoritySource": "test",
            "reason": "transition index counter fixture",
            "timestamp": "2026-09-08T00:00:00Z"
        })
    }

    #[test]
    fn historical_finalization_transition_is_parsed_once_per_observation() {
        let root = tempfile::tempdir().expect("tempdir");
        let decisions = root.path().join(".ai/decisions");
        fs::create_dir_all(&decisions).expect("decisions");
        let repository_id = repository_id(root.path()).to_string();
        let predecessor = legacy_receipt("WI-INDEX-ALPHA", &repository_id);
        let mut head = predecessor.clone();
        head["receiptId"] = "WI-INDEX-ALPHA-head".into();
        head["operationId"] = "WI-INDEX-ALPHA-head-operation".into();
        head["before"] = predecessor["after"].clone();
        head["after"]["pullRequest"] = "merged".into();
        head["result"] = json!({
            "disposition": "retained",
            "failureCodes": [],
            "unknownCodes": []
        });
        let transition = json!({
            "schemaVersion": 1,
            "transitionId": "WI-INDEX-ALPHA-transition-1",
            "sequence": 1,
            "predecessorReceiptDigest": cockpit_protocol::digest_json(&predecessor).expect("digest"),
            "receipt": head
        });
        let transition_digest = cockpit_protocol::digest_json(&transition).expect("digest");
        fs::write(
            decisions.join(format!(
                "WI-INDEX-ALPHA.finalize.{}.json",
                transition_digest
                    .to_string()
                    .strip_prefix("sha256:")
                    .expect("sha256 prefix")
            )),
            serde_json::to_vec_pretty(&transition).expect("encode"),
        )
        .expect("transition");

        let first =
            FinalizationTransitionIndex::from_directory(root.path()).expect("first observation");
        assert_eq!(first.parsed_transition_count(), 1);

        let second =
            FinalizationTransitionIndex::from_directory(root.path()).expect("second observation");
        assert_eq!(second.parsed_transition_count(), 1);
    }

    #[test]
    fn indexed_resolution_preserves_canonical_error_precedence() {
        let root = tempfile::tempdir().expect("tempdir");
        let decisions = root.path().join(".ai/decisions");
        fs::create_dir_all(&decisions).expect("decisions");
        fs::write(
            decisions.join("WI-INDEX-ALPHA.finalize.malformed.json"),
            b"{malformed",
        )
        .expect("malformed transition");

        let index = FinalizationTransitionIndex::from_directory(root.path()).expect("index");
        let error = crate::resolve_resource_finalization_head_with_index(
            root.path(),
            "WI-INDEX-ALPHA",
            &index,
        )
        .expect_err("missing canonical receipt must fail");
        assert!(
            matches!(error, ObserverError::State { path, .. } if path.ends_with("WI-INDEX-ALPHA.finalize.json"))
        );
    }
}

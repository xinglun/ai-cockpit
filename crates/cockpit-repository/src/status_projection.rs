use super::*;

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
    if !unclosed_archived_work_items.is_empty() {
        blockers.push("archived_work_items_pending_close".into());
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
        value: ResourceFinalizationTransitionReceipt,
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
                                value,
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
        return false;
    };
    manifest.get("state").and_then(serde_json::Value::as_str) == Some("archived")
        && manifest
            .get("closeRequired")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
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

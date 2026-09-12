use super::*;

pub fn start_work_item(
    root: &Path,
    work_item_id: &str,
    intent: &str,
    goal: &str,
    scope: &[String],
) -> Result<LifecycleReceipt, ObserverError> {
    start_work_item_with_options(
        root,
        work_item_id,
        intent,
        goal,
        scope,
        &WorkItemStartOptions {
            risk: "normal".into(),
            authority: "missing".into(),
            ..WorkItemStartOptions::default()
        },
    )
}

pub fn start_work_item_with_options(
    root: &Path,
    work_item_id: &str,
    intent: &str,
    goal: &str,
    scope: &[String],
    options: &WorkItemStartOptions,
) -> Result<LifecycleReceipt, ObserverError> {
    // Recovery-generated `not_ready` scaffolds are an explicit continuation
    // of an existing lifecycle and may be activated while their predecessor
    // is still awaiting closure.  All ordinary starts must pass the same
    // repository entry gate as `work-item new`.
    let recovery_continuation = recovery_scaffold_exists(root, work_item_id);
    validate_start_entry(root, !recovery_continuation, recovery_continuation)?;
    if let Some(receipt) =
        activate_not_ready_scaffold(root, work_item_id, intent, goal, scope, options)?
    {
        return Ok(receipt);
    }
    if !recovery_continuation {
        ensure_no_unclosed_archived_work_items(root)?;
    }
    create_work_item_scaffold(
        root,
        &ContractScaffoldInput {
            work_item_id,
            mode: "implementation",
            intent,
            goal,
            scope,
            options,
            state: "implementation_active",
        },
    )?;
    Ok(LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "implementation_active".into(),
        timestamp: now(),
    })
}

/// Activate a recovery-generated `not_ready` scaffold without replacing its
/// predecessor binding or repository facts.  A scaffold created by
/// `work-item recover` is intentionally reserved first; `start` is the
/// explicit human-owned transition that supplies governance fields and makes
/// it eligible for preflight.  Ordinary duplicate starts still fail closed.
fn activate_not_ready_scaffold(
    root: &Path,
    work_item_id: &str,
    intent: &str,
    goal: &str,
    scope: &[String],
    options: &WorkItemStartOptions,
) -> Result<Option<LifecycleReceipt>, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let active = root.join(".ai/work-items/active");
    let contract_path = active.join(format!("{work_item_id}.contract.json"));
    let summary_path = active.join(format!("{work_item_id}.summary.json"));
    if !contract_path.is_file() || !summary_path.is_file() {
        return Ok(None);
    }
    let mut contract = read_json(&contract_path)?;
    let mut summary = read_json(&summary_path)?;
    if contract["state"] != serde_json::json!("not_ready")
        || summary["state"] != serde_json::json!("not_ready")
    {
        return Ok(None);
    }
    if contract["workItemId"] != serde_json::json!(work_item_id)
        || summary["workItemId"] != serde_json::json!(work_item_id)
        || contract["repositoryId"] != serde_json::json!(repository_id(&root).to_string())
    {
        return Err(ObserverError::State {
            path: contract_path,
            message: "recovery scaffold identity does not match this repository or Work Item"
                .into(),
        });
    }
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let profile_path = root.join(".ai/project.json");
    let profile: AttachedProfile = read_json(&profile_path).and_then(|value| {
        serde_json::from_value(value).map_err(|error| ObserverError::State {
            path: profile_path.clone(),
            message: error.to_string(),
        })
    })?;
    let profile_digest = attached_profile_digest(&profile, &profile_path)?;
    let current_snapshot_digest = snapshot_digest(&snapshot)?;
    contract["repositoryId"] = serde_json::json!(profile.repository_id);
    contract["baseRevision"] =
        serde_json::json!(snapshot.head.clone().unwrap_or_else(|| "unborn".into()));
    contract["projectProfileDigest"] = serde_json::json!(profile_digest);
    contract["repositorySnapshotDigest"] = serde_json::json!(current_snapshot_digest);
    contract["state"] = serde_json::json!("implementation_active");
    contract["intent"] = serde_json::json!(intent);
    contract["goal"] = serde_json::json!(goal);
    contract["scope"] = serde_json::to_value(scope).map_err(|error| ObserverError::State {
        path: contract_path.clone(),
        message: error.to_string(),
    })?;
    contract["outOfScope"] =
        serde_json::to_value(&options.out_of_scope).map_err(|error| ObserverError::State {
            path: contract_path.clone(),
            message: error.to_string(),
        })?;
    contract["risk"] = serde_json::json!(options.risk);
    contract["authority"] = serde_json::json!(options.authority);
    contract["acceptanceCriteria"] =
        serde_json::to_value(&options.acceptance_criteria).map_err(|error| {
            ObserverError::State {
                path: contract_path.clone(),
                message: error.to_string(),
            }
        })?;
    contract["requiredEvidenceClasses"] = serde_json::to_value(&options.required_evidence_classes)
        .map_err(|error| ObserverError::State {
            path: contract_path.clone(),
            message: error.to_string(),
        })?;
    contract["verification"] =
        serde_json::to_value(default_verification_commands(&root)).map_err(|error| {
            ObserverError::State {
                path: contract_path.clone(),
                message: error.to_string(),
            }
        })?;
    contract["resourceContext"] = serde_json::to_value(provisional_resource_context(&root))
        .map_err(|error| ObserverError::State {
            path: contract_path.clone(),
            message: error.to_string(),
        })?;
    summary["state"] = serde_json::json!("implementation_active");
    summary["repositoryId"] = serde_json::json!(profile.repository_id);
    summary["changedPaths"] = serde_json::json!(snapshot.changed_paths);
    summary["checkpointCount"] = serde_json::json!(0);
    summary["preflightState"] = serde_json::json!("not_run");
    summary["updatedAt"] = serde_json::json!(now());
    atomic_json(&contract_path, &contract)?;
    atomic_json(&summary_path, &summary)?;
    Ok(Some(LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "implementation_active".into(),
        timestamp: now(),
    }))
}

/// Select only verification commands that the current repository can support
/// from observed build facts. Cargo's `--locked` mode is valid only when the
/// repository actually contains a lockfile; declaring it for a lockfile-less
/// adopter makes the Contract impossible to execute. Non-Cargo repositories
/// receive no invented command and must declare an owner-approved check.
fn default_verification_commands(root: &Path) -> Vec<String> {
    let cargo_manifest = root.join("Cargo.toml");
    if !cargo_manifest.is_file() {
        return Vec::new();
    }
    if root.join("Cargo.lock").is_file() {
        vec!["cargo test --locked --workspace".into()]
    } else {
        vec!["cargo test --workspace".into()]
    }
}

/// Create a deterministic, validator-readable Work Item skeleton.
///
/// This is the single scaffold writer used by both the transitional `start`
/// lifecycle and the user-facing `work-item new` command. The caller supplies
/// only fields that are explicitly human-owned; the repository facts are read
/// from one fresh snapshot and the attached profile.
pub fn scaffold_work_item(
    root: &Path,
    work_item_id: &str,
    mode: &str,
) -> Result<WorkItemScaffoldReceipt, ObserverError> {
    validate_start_entry(root, true, false)?;
    ensure_no_unclosed_archived_work_items(root)?;
    scaffold_work_item_internal(root, work_item_id, mode)
}

/// Create a recovery successor scaffold.  Recovery is not an independent
/// next Work Item, so it may be created while its predecessor is archived and
/// awaiting the explicit recovery/close decision.  It still uses the same
/// atomic scaffold writer and repository-local identity facts.
fn scaffold_work_item_for_recovery(
    root: &Path,
    work_item_id: &str,
    mode: &str,
) -> Result<WorkItemScaffoldReceipt, ObserverError> {
    scaffold_work_item_internal(root, work_item_id, mode)
}

fn scaffold_work_item_internal(
    root: &Path,
    work_item_id: &str,
    mode: &str,
) -> Result<WorkItemScaffoldReceipt, ObserverError> {
    let mode = mode.trim();
    if mode.is_empty() {
        return Err(ObserverError::State {
            path: root.join(".ai/work-items/active"),
            message: "work item mode must not be empty".into(),
        });
    }
    let options = WorkItemStartOptions {
        out_of_scope: Vec::new(),
        risk: "unknown".into(),
        authority: "unknown".into(),
        acceptance_criteria: Vec::new(),
        required_evidence_classes: Vec::new(),
    };
    let scaffold = create_work_item_scaffold(
        root,
        &ContractScaffoldInput {
            work_item_id,
            mode,
            intent: "",
            goal: "",
            scope: &[],
            options: &options,
            state: "not_ready",
        },
    )?;
    Ok(WorkItemScaffoldReceipt {
        work_item_id: work_item_id.into(),
        mode: mode.into(),
        contract_path: scaffold.contract_path,
        state: "not_ready".into(),
        known_facts: scaffold.facts,
        human_input_required: vec![
            "intent".into(),
            "scope".into(),
            "acceptanceCriteria".into(),
            "authority".into(),
        ],
    })
}

struct CreatedWorkItemScaffold {
    contract_path: String,
    facts: WorkItemScaffoldFacts,
}

struct ContractScaffoldInput<'a> {
    work_item_id: &'a str,
    mode: &'a str,
    intent: &'a str,
    goal: &'a str,
    scope: &'a [String],
    options: &'a WorkItemStartOptions,
    state: &'a str,
}

struct WorkItemScaffoldReservation {
    reservation_path: PathBuf,
    contract_path: PathBuf,
    summary_path: PathBuf,
    contract_created: bool,
    summary_created: bool,
    committed: bool,
}

impl WorkItemScaffoldReservation {
    fn mark_contract_created(&mut self) {
        self.contract_created = true;
    }

    fn mark_summary_created(&mut self) {
        self.summary_created = true;
    }

    fn commit(&mut self) {
        self.committed = true;
    }
}

impl Drop for WorkItemScaffoldReservation {
    fn drop(&mut self) {
        if !self.committed {
            if self.summary_created {
                let _ = fs::remove_file(&self.summary_path);
            }
            if self.contract_created {
                let _ = fs::remove_file(&self.contract_path);
            }
        }
        let _ = fs::remove_file(&self.reservation_path);
    }
}

fn create_work_item_scaffold(
    root: &Path,
    input: &ContractScaffoldInput<'_>,
) -> Result<CreatedWorkItemScaffold, ObserverError> {
    validate_work_item_id(input.work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let ai = root.join(".ai");
    if !ai.join("cockpit.toml").exists() {
        attach(&root)?;
    }
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let profile_path = ai.join("project.json");
    let profile: AttachedProfile = read_json(&profile_path).and_then(|value| {
        serde_json::from_value(value).map_err(|error| ObserverError::State {
            path: profile_path.clone(),
            message: error.to_string(),
        })
    })?;
    let profile_digest = attached_profile_digest(&profile, &profile_path)?;
    let facts = WorkItemScaffoldFacts {
        repository_id: profile.repository_id.clone(),
        base_revision: snapshot.head.clone().unwrap_or_else(|| "unborn".into()),
        project_profile_digest: profile_digest,
        repository_snapshot_digest: snapshot_digest(&snapshot)?,
    };
    let now = now();
    let contract = serde_json::json!({
        "protocolVersion": 1,
        "repositoryId": facts.repository_id,
        "workItemId": input.work_item_id,
        "mode": input.mode,
        "state": input.state,
        "intent": input.intent,
        "goal": input.goal,
        "scope": input.scope,
        "outOfScope": input.options.out_of_scope.clone(),
        "risk": input.options.risk.clone(),
        "authority": input.options.authority.clone(),
        "acceptanceCriteria": input.options.acceptance_criteria.clone(),
        "requiredEvidenceClasses": input.options.required_evidence_classes.clone(),
        "sources": [],
        "verification": [],
        "baseRevision": facts.base_revision,
        "projectProfileDigest": facts.project_profile_digest,
        "repositorySnapshotDigest": facts.repository_snapshot_digest,
        "resourceContext": provisional_resource_context(&root),
        "createdAt": now,
    });
    let summary = serde_json::json!({
        "protocolVersion": 1,
        "repositoryId": facts.repository_id,
        "workItemId": input.work_item_id,
        "mode": input.mode,
        "state": input.state,
        "changedPaths": snapshot.changed_paths,
        "checkpointCount": 0,
        "preflightState": "not_run",
        "createdAt": now,
        "updatedAt": now,
    });
    let active = ai.join("work-items/active");
    let archive = ai.join("work-items/archive");
    // A repository may legitimately have no active Work Items after all
    // previous work has been archived.  Scaffolding a new item must restore
    // the protocol directories instead of failing on the first atomic write.
    for directory in [&active, &archive] {
        fs::create_dir_all(directory).map_err(|source| ObserverError::Read {
            path: directory.clone(),
            source,
        })?;
    }
    let contract_path = active.join(format!("{}.contract.json", input.work_item_id));
    let summary_path = active.join(format!("{}.summary.json", input.work_item_id));
    let archive_path = archive.join(format!("{}.archive.json", input.work_item_id));
    let reservation_path = active.join(format!(".{}.scaffold.reserve", input.work_item_id));
    let mut reservation_file = match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&reservation_path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(ObserverError::State {
                path: reservation_path,
                message: "work item already exists or scaffold reservation is active".into(),
            });
        }
        Err(source) => {
            return Err(ObserverError::Read {
                path: reservation_path,
                source,
            });
        }
    };
    if let Err(source) = reservation_file
        .write_all(b"ai-cockpit work-item scaffold reservation\n")
        .and_then(|()| reservation_file.sync_all())
    {
        drop(reservation_file);
        let _ = fs::remove_file(&reservation_path);
        return Err(ObserverError::Read {
            path: reservation_path,
            source,
        });
    }
    drop(reservation_file);
    let mut reservation = WorkItemScaffoldReservation {
        reservation_path,
        contract_path: contract_path.clone(),
        summary_path: summary_path.clone(),
        contract_created: false,
        summary_created: false,
        committed: false,
    };
    if [contract_path.clone(), summary_path.clone(), archive_path]
        .iter()
        .any(|path| path.exists())
    {
        return Err(ObserverError::State {
            path: contract_path,
            message: "work item already exists".into(),
        });
    }
    atomic_json(&contract_path, &contract)?;
    reservation.mark_contract_created();
    atomic_json(&summary_path, &summary)?;
    reservation.mark_summary_created();
    reservation.commit();
    Ok(CreatedWorkItemScaffold {
        contract_path: contract_path.to_string_lossy().into_owned(),
        facts,
    })
}

/// Facts gathered for one checkpoint request. This is the observation phase;
/// governance and persistence consume the same Contract, Git snapshot, and
/// digests rather than capturing a second snapshot in the use case body.
struct CheckpointObservation {
    contract_path: PathBuf,
    contract: Contract,
    snapshot: RepositorySnapshot,
    current_snapshot_digest: String,
    current_contract_digest: String,
}

fn checkpoint_observe(
    root: &Path,
    work_item_id: &str,
) -> Result<CheckpointObservation, ObserverError> {
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&contract_path)?;
    let git = cockpit_git::GitRepository::discover(root).map_err(|error| ObserverError::State {
        path: root.to_path_buf(),
        message: error.to_string(),
    })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.to_path_buf(),
        message: error.to_string(),
    })?;
    let current_snapshot_digest = snapshot_digest(&snapshot)?.to_string();
    let current_contract_digest = contract_digest(&contract_path)?.to_string();
    Ok(CheckpointObservation {
        contract_path,
        contract,
        snapshot,
        current_snapshot_digest,
        current_contract_digest,
    })
}

/// Governance-only checks over already observed checkpoint facts. The
/// existing governance decision remains the single policy authority and is
/// evaluated against the captured snapshot; this helper does not persist.
fn checkpoint_governance_checks(
    root: &Path,
    summary_path: &Path,
    summary: &serde_json::Value,
    preflight_state: &str,
    observation: &CheckpointObservation,
) -> Result<(), ObserverError> {
    if summary["preflightRepositorySnapshotDigest"]
        .as_str()
        .is_none_or(|value| value != observation.current_snapshot_digest)
    {
        return Err(ObserverError::State {
            path: summary_path.to_path_buf(),
            message: "checkpoint requires a preflight result for the current repository snapshot"
                .into(),
        });
    }
    if summary["preflightContractDigest"]
        .as_str()
        .is_none_or(|value| value != observation.current_contract_digest)
    {
        return Err(ObserverError::State {
            path: summary_path.to_path_buf(),
            message: "checkpoint requires a preflight result for the current contract".into(),
        });
    }
    require_green_or_yellow_preflight_governance(
        root,
        &observation.contract_path,
        &observation.contract,
        &observation.snapshot,
        preflight_state,
    )?;
    // `before_edit` is the authorization-to-edit boundary. Once any
    // verification result exists, recording that checkpoint would rewrite
    // phase ordering and could make post-verification work appear authorized
    // before execution. Keep this reference-defined boundary fail-closed.
    if summary
        .get("verification")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|entries| !entries.is_empty())
    {
        return Err(ObserverError::State {
            path: summary_path.to_path_buf(),
            message: "before_edit checkpoint must be recorded before required verification".into(),
        });
    }
    Ok(())
}

pub fn checkpoint_work_item(
    root: &Path,
    work_item_id: &str,
) -> Result<LifecycleReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let mut summary: serde_json::Value = read_json(&path)?;
    let count = summary["checkpointCount"].as_u64().unwrap_or(0);
    if count != 0 {
        return Err(ObserverError::State {
            path: path.clone(),
            message: "work item already has a checkpoint; duplicate checkpoint is not allowed"
                .into(),
        });
    }
    let state = summary["state"].as_str().unwrap_or("");
    if state != "implementation_active" {
        return Err(ObserverError::State {
            path: path.clone(),
            message: format!(
                "checkpoint is invalid from state {state:?}; expected implementation_active"
            ),
        });
    }
    let preflight_state = summary["preflightState"].as_str().unwrap_or("").to_string();
    if !matches!(preflight_state.as_str(), "green" | "yellow") {
        return Err(ObserverError::State {
            path: path.clone(),
            message: format!(
                "checkpoint requires a recorded non-red preflight result (state={preflight_state:?})"
            ),
        });
    }
    let observation = checkpoint_observe(&root, work_item_id)?;
    checkpoint_governance_checks(&root, &path, &summary, &preflight_state, &observation)?;
    let timestamp = now();
    if observation.contract.checkpoint_policy.is_some() {
        append_checkpoint_evidence(
            &mut summary,
            &root,
            &observation.contract,
            "before_edit",
            &observation.snapshot,
            &observation.current_contract_digest,
            0,
            &timestamp,
        )?;
    }
    summary["checkpointCount"] = 1.into();
    summary["state"] = "checkpointed".into();
    summary["checkpointAt"] = timestamp.clone().into();
    summary["checkpointContractDigest"] = observation.current_contract_digest.into();
    summary["checkpointRepositorySnapshotDigest"] = observation.current_snapshot_digest.into();
    summary["updatedAt"] = timestamp.clone().into();
    atomic_json(&path, &summary)?;
    Ok(LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "checkpointed".into(),
        timestamp,
    })
}

fn checkpoint_required_check_names(contract: &Contract) -> Vec<String> {
    let mut checks = required_verification_checks(contract);
    if let Some(policy) = contract.checkpoint_policy.as_ref() {
        checks.extend(policy.required_checks.iter().cloned());
    }
    checks.sort();
    checks.dedup();
    checks
}

fn checkpoint_passed_check_count(contract: &Contract, summary: &serde_json::Value) -> u64 {
    let names = checkpoint_required_check_names(contract);
    names
        .iter()
        .filter(|name| {
            summary
                .get("verification")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|items| {
                    items.iter().any(|item| {
                        item.get("check").and_then(serde_json::Value::as_str) == Some(name.as_str())
                            && item.get("result").and_then(serde_json::Value::as_str)
                                == Some("passed")
                    })
                })
        })
        .count() as u64
}

#[allow(clippy::too_many_arguments)]
fn append_checkpoint_evidence(
    summary: &mut serde_json::Value,
    root: &Path,
    contract: &Contract,
    stage: &str,
    snapshot: &RepositorySnapshot,
    contract_hash: &str,
    required_checks_passed: u64,
    recorded_at: &str,
) -> Result<(), ObserverError> {
    let evidence = summary
        .as_object_mut()
        .expect("Work Item Summary is an object")
        .entry("checkpointEvidence")
        .or_insert_with(|| serde_json::json!([]));
    let entries = evidence
        .as_array_mut()
        .ok_or_else(|| ObserverError::State {
            path: root.join(".ai/work-items/active"),
            message: "checkpointEvidence must be an array".into(),
        })?;
    if entries
        .iter()
        .any(|item| item.get("stage").and_then(serde_json::Value::as_str) == Some(stage))
    {
        return Err(ObserverError::State {
            path: root.join(".ai/work-items/active"),
            message: format!("checkpointEvidence stage {stage} is already recorded"),
        });
    }
    let required_checks = checkpoint_required_check_names(contract);
    entries.push(serde_json::json!({
        "schemaVersion": 1,
        "repositoryId": repository_id(root),
        "workItemId": contract.work_item_id,
        "stage": stage,
        "recorded": true,
        "contractHash": contract_hash,
        "repositorySnapshotDigest": snapshot_digest(snapshot)?,
        "acceptanceCount": contract.acceptance_criteria.len(),
        "unknownCount": contract.unknowns.len(),
        "requiredChecks": required_checks.len(),
        "requiredChecksPassed": required_checks_passed,
        "recordedAt": recorded_at,
    }));
    Ok(())
}

/// Append a Contract-amendment revalidation record without rewriting the
/// immutable `before_edit` checkpoint.  A post-verification amendment marks
/// every prior required result stale; a fresh preflight and verification must
/// clear that marker before finish/archive/close can proceed.
pub fn revalidate_contract_amendment(
    root: &Path,
    work_item_id: &str,
    reason: &str,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    if reason.trim().is_empty() {
        return Err(ObserverError::State {
            path: root.join(".ai/work-items/active"),
            message: "contract amendment reason must not be empty".into(),
        });
    }
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let active = root.join(".ai/work-items/active");
    let contract_path = active.join(format!("{work_item_id}.contract.json"));
    let summary_path = active.join(format!("{work_item_id}.summary.json"));
    let contract = read_contract(&contract_path)?;
    let mut summary = read_json(&summary_path)?;
    // Contracts created before typed checkpoint evidence was introduced may
    // still have the original checkpoint identity fields on Summary while
    // `checkpointEvidence` is absent.  Upgrade that deterministic legacy
    // record in-memory so an explicit amendment can proceed; the original
    // Contract/Summary bytes remain bound by the generated before_edit hash.
    if !summary
        .get("checkpointEvidence")
        .is_some_and(serde_json::Value::is_array)
    {
        let legacy_contract_hash = summary
            .get("checkpointContractDigest")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| ObserverError::State {
                path: summary_path.clone(),
                message: "contract amendment requires a legacy checkpoint Contract digest".into(),
            })?;
        let legacy_snapshot_digest = summary
            .get("checkpointRepositorySnapshotDigest")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| ObserverError::State {
                path: summary_path.clone(),
                message: "contract amendment requires a legacy checkpoint snapshot digest".into(),
            })?;
        let recorded_at = summary
            .get("checkpointAt")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| ObserverError::State {
                path: summary_path.clone(),
                message: "contract amendment requires a legacy checkpoint timestamp".into(),
            })?;
        if summary["checkpointCount"] != serde_json::json!(1) {
            return Err(ObserverError::State {
                path: summary_path.clone(),
                message: "contract amendment requires exactly one legacy checkpoint".into(),
            });
        }
        summary["checkpointEvidence"] = serde_json::json!([{
            "schemaVersion": 1,
            "repositoryId": repository_id(&root),
            "workItemId": work_item_id,
            "stage": "before_edit",
            "recorded": true,
            "contractHash": legacy_contract_hash,
            "repositorySnapshotDigest": legacy_snapshot_digest,
            "acceptanceCount": contract.acceptance_criteria.len(),
            "unknownCount": contract.unknowns.len(),
            "requiredChecks": 0,
            "requiredChecksPassed": 0,
            "recordedAt": recorded_at,
        }]);
    }
    let evidence = summary
        .get("checkpointEvidence")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| ObserverError::State {
            path: summary_path.clone(),
            message: "contract amendment requires typed checkpointEvidence".into(),
        })?;
    let before_edit = evidence
        .iter()
        .find(|entry| entry.get("stage").and_then(serde_json::Value::as_str) == Some("before_edit"))
        .cloned()
        .ok_or_else(|| ObserverError::State {
            path: summary_path.clone(),
            message: "contract amendment requires a before_edit checkpoint".into(),
        })?;
    let before_edit: CheckpointEvidence =
        serde_json::from_value(before_edit).map_err(|error| ObserverError::State {
            path: summary_path.clone(),
            message: format!("before_edit checkpoint is malformed: {error}"),
        })?;
    if evidence.iter().any(|entry| {
        entry.get("stage").and_then(serde_json::Value::as_str) == Some("before_finish")
    }) {
        return Err(ObserverError::State {
            path: summary_path,
            message: "contract amendment after before_finish requires a recovery Work Item".into(),
        });
    }
    let amendment_hashes = evidence
        .iter()
        .filter(|entry| {
            entry.get("stage").and_then(serde_json::Value::as_str)
                == Some("contract_amendment_revalidation")
        })
        .map(|entry| {
            serde_json::from_value::<CheckpointEvidence>(entry.clone()).map_err(|error| {
                ObserverError::State {
                    path: summary_path.clone(),
                    message: format!("contract amendment checkpoint is malformed: {error}"),
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let previous_contract_hash = amendment_hashes
        .last()
        .map(|entry| entry.contract_hash.clone())
        .unwrap_or_else(|| before_edit.contract_hash.clone());
    let current_contract_hash = contract_digest(&contract_path)?.to_string();
    if current_contract_hash == previous_contract_hash {
        return Err(ObserverError::State {
            path: contract_path,
            message: "contract amendment must change Contract bytes".into(),
        });
    }
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let required_checks = checkpoint_required_check_names(&contract);
    let required_checks_passed = checkpoint_passed_check_count(&contract, &summary);
    let verification_started = summary
        .get("verification")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|entries| {
            entries.iter().any(|entry| {
                matches!(
                    entry.get("result").and_then(serde_json::Value::as_str),
                    Some("passed" | "failed" | "warning" | "blocked")
                )
            })
        })
        // A legacy command-only Contract has no typed required gates to
        // invalidate. Keep the amendment's historical fact, but do not mark
        // it as a gate invalidation that must contain a non-empty list.
        && !required_checks.is_empty();
    let record = serde_json::json!({
        "schemaVersion": 1,
        "repositoryId": repository_id(&root),
        "workItemId": work_item_id,
        "stage": "contract_amendment_revalidation",
        "recorded": true,
        "contractHash": current_contract_hash.clone(),
        "repositorySnapshotDigest": snapshot_digest(&snapshot)?,
        "acceptanceCount": contract.acceptance_criteria.len(),
        "unknownCount": contract.unknowns.len(),
        "requiredChecks": required_checks.len(),
        "requiredChecksPassed": 0,
        "originalBeforeEditContractHash": before_edit.contract_hash,
        "previousContractHash": previous_contract_hash,
        "reason": reason.trim(),
        "verificationStarted": verification_started,
        "invalidatedRequiredChecks": if verification_started { required_checks.clone() } else { Vec::new() },
        "requiredChecksPassedAtAmendment": if verification_started { Some(required_checks_passed) } else { None },
        "recordedAt": now(),
    });
    let entries = summary
        .get_mut("checkpointEvidence")
        .and_then(serde_json::Value::as_array_mut)
        .expect("checkpointEvidence was validated as an array");
    entries.push(record.clone());
    // A Contract amendment invalidates the finish-ready projection.  Reopen
    // the single checkpointed recovery state so the normal preflight → verify
    // → finish path can be replayed without hand-editing generated Summary
    // bytes.  The previous Outcome/evidence remain immutable predecessor
    // facts; a fresh verification will replace the active projection.
    if summary["state"] == serde_json::json!("finish_ready") {
        summary["state"] = "checkpointed".into();
        if let Some(object) = summary.as_object_mut() {
            object.remove("failedGate");
            object.remove("recoveryCondition");
            object.remove("outcomeState");
        }
    }
    summary["preflightState"] = "not_run".into();
    summary["verificationInvalidatedByContractAmendment"] = serde_json::json!({
        "contractHash": current_contract_hash,
        "invalidatedRequiredChecks": if verification_started { required_checks } else { Vec::new() },
        "recordedAt": now(),
    });
    atomic_json(&summary_path, &summary)?;
    Ok(record)
}

/// Apply a bounded, append-only Contract amendment and record its revalidation.
/// Only additive scope, out-of-scope, acceptance, required-evidence, and
/// scenario-coverage entries are accepted; the Runtime never lets an
/// amendment rewrite identity, authority, base, mode, or existing criteria.
pub fn amend_work_item_contract(
    root: &Path,
    work_item_id: &str,
    input: &serde_json::Value,
    reason: &str,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let mut contract = read_json(&path)?;
    let summary = read_json(&summary_path)?;
    for key in input
        .as_object()
        .into_iter()
        .flat_map(|object| object.keys())
    {
        if !matches!(
            key.as_str(),
            "scopeAppend"
                | "outOfScopeAppend"
                | "acceptanceAppend"
                | "requiredEvidenceClassesAppend"
                | "scenarioCoverageAppend"
        ) {
            return Err(ObserverError::State {
                path: path.clone(),
                message: format!("unsupported Contract amendment field {key}"),
            });
        }
    }
    let pre_checkpoint_scenario_declaration = input.get("scenarioCoverageAppend").is_some()
        && summary["checkpointCount"] == serde_json::json!(0)
        && input.as_object().is_some_and(|object| object.len() == 1);
    for (field, target) in [
        ("scopeAppend", "scope"),
        ("outOfScopeAppend", "outOfScope"),
        ("acceptanceAppend", "acceptanceCriteria"),
        ("requiredEvidenceClassesAppend", "requiredEvidenceClasses"),
    ] {
        let Some(values) = input.get(field) else {
            continue;
        };
        let values = values.as_array().ok_or_else(|| ObserverError::State {
            path: path.clone(),
            message: format!("{field} must be an array"),
        })?;
        let existing = contract[target]
            .as_array_mut()
            .ok_or_else(|| ObserverError::State {
                path: path.clone(),
                message: format!("Contract field {target} is not an array"),
            })?;
        for value in values {
            let value = value
                .as_str()
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| ObserverError::State {
                    path: path.clone(),
                    message: format!("{field} entries must be non-empty strings"),
                })?;
            if !existing.iter().any(|entry| entry.as_str() == Some(value)) {
                existing.push(serde_json::json!(value));
            }
        }
    }
    if let Some(values) = input.get("scenarioCoverageAppend") {
        let values = values.as_array().ok_or_else(|| ObserverError::State {
            path: path.clone(),
            message: "scenarioCoverageAppend must be an array".into(),
        })?;
        if contract
            .get("scenarioCoverage")
            .is_none_or(serde_json::Value::is_null)
        {
            contract["scenarioCoverage"] = serde_json::json!([]);
        }
        let coverage =
            contract["scenarioCoverage"]
                .as_array_mut()
                .ok_or_else(|| ObserverError::State {
                    path: path.clone(),
                    message: "Contract field scenarioCoverage is not an array".into(),
                })?;
        for value in values {
            let Some(name) = value.get("scenario").and_then(serde_json::Value::as_str) else {
                return Err(ObserverError::State {
                    path: path.clone(),
                    message: "scenarioCoverageAppend entries must contain a scenario string".into(),
                });
            };
            if name.trim().is_empty()
                || coverage.iter().any(|entry| {
                    entry.get("scenario").and_then(serde_json::Value::as_str) == Some(name)
                })
            {
                return Err(ObserverError::State {
                    path: path.clone(),
                    message: format!(
                        "scenarioCoverageAppend contains duplicate or empty scenario {name:?}"
                    ),
                });
            }
            coverage.push(value.clone());
        }
        if let Err(errors) =
            cockpit_protocol::validate_scenario_coverage_projection(&contract["scenarioCoverage"])
        {
            return Err(ObserverError::State {
                path: path.clone(),
                message: format!("scenarioCoverageAppend is invalid: {}", errors.join(", ")),
            });
        }
    }
    if pre_checkpoint_scenario_declaration {
        serde_json::from_value::<Contract>(contract.clone()).map_err(|error| {
            ObserverError::State {
                path: path.clone(),
                message: format!("scenarioCoverageAppend produced an invalid Contract: {error}"),
            }
        })?;
        atomic_json(&path, &contract)?;
        return Ok(serde_json::json!({
            "schemaVersion": 1,
            "repositoryId": repository_id(&root),
            "workItemId": work_item_id,
            "stage": "pre_checkpoint_scenario_declaration",
            "recorded": true,
            "contractHash": contract_digest(&path)?,
            "recordedAt": now()
        }));
    }
    if contract["scope"].as_array().is_none() {
        return Err(ObserverError::State {
            path,
            message: "Contract scope is malformed".into(),
        });
    }
    atomic_json(&path, &contract)?;
    revalidate_contract_amendment(&root, work_item_id, reason)
}

/// Evaluate and persist the preflight decision for an active Work Item.
///
/// Preflight is intentionally a repository-local receipt rather than process
/// state. A yellow result may be recorded before verification (for example,
/// when the Contract requires a verification receipt that does not exist yet),
/// while evidence produced only by later release/adopter/close stages is
/// deferred to those completion boundaries. `finish` requires a fresh green
/// result for the current source-verification boundary.
pub fn preflight_work_item(
    root: &Path,
    contract_path: &Path,
) -> Result<GovernanceDecision, ObserverError> {
    preflight_work_item_internal(root, contract_path, None)
}

/// Evaluate and persist preflight while binding evidence checks to the
/// Runtime executing the request.  CLI and MCP use this entry point so a
/// foreign Runtime receipt cannot make the lifecycle appear green.
pub fn preflight_work_item_with_runtime(
    root: &Path,
    contract_path: &Path,
    runtime: &RuntimeContext,
) -> Result<GovernanceDecision, ObserverError> {
    preflight_work_item_internal(root, contract_path, Some(runtime))
}

fn preflight_work_item_internal(
    root: &Path,
    contract_path: &Path,
    current_runtime: Option<&RuntimeContext>,
) -> Result<GovernanceDecision, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let contract_path = if contract_path.is_absolute() {
        contract_path.to_path_buf()
    } else {
        root.join(contract_path)
    };
    let contract = read_contract(&contract_path)?;
    let repository_context = RepositoryExecutionContext::capture(&root)?;
    let observation_context = repository_context.observe_phase_with_contract(
        ObservationPhase::BeforeGovernance,
        current_runtime,
        &contract_path,
    )?;
    let snapshot = observation_context.snapshot().clone();
    let raw_decision = governance_decision_for_pre_execution_boundary(
        &root,
        &contract,
        &snapshot,
        current_runtime,
        Some(&observation_context),
    )?;
    let decision = apply_preflight_review_evidence(
        &root,
        &contract,
        &snapshot,
        raw_decision.clone(),
        false,
        None,
    )?;
    observation_context.validate_current()?;

    let active = root.join(".ai/work-items/active");
    let active_contract = active.join(format!("{}.contract.json", contract.work_item_id));
    let summary_path = active.join(format!("{}.summary.json", contract.work_item_id));
    if active_contract.is_file() && summary_path.is_file() {
        let active_contract =
            fs::canonicalize(&active_contract).map_err(|source| ObserverError::Read {
                path: active_contract.clone(),
                source,
            })?;
        let requested_contract =
            fs::canonicalize(&contract_path).map_err(|source| ObserverError::Read {
                path: contract_path.clone(),
                source,
            })?;
        if active_contract != requested_contract {
            return Err(ObserverError::State {
                path: contract_path,
                message: "preflight contract is not the active Work Item contract".into(),
            });
        }
        let mut summary: serde_json::Value = read_json(&summary_path)?;
        require_current_retry_recovery_binding(
            &root,
            &contract.work_item_id,
            &summary,
            current_runtime,
        )?;
        let current_state = summary["state"].as_str().unwrap_or("");
        // A scaffold is intentionally not an active lifecycle item yet.  Keep
        // the historical read-only preflight behavior for this state so
        // callers can inspect the candidate decision before `start` supplies
        // the human governance fields and activates the item.
        if current_state == "not_ready" {
            let state = decision_state_name(decision.state.clone());
            let decision_value =
                serde_json::to_value(&raw_decision).map_err(|error| ObserverError::State {
                    path: active_contract.clone(),
                    message: error.to_string(),
                })?;
            summary["preflightState"] = state.into();
            summary["preflightDecisionDigest"] = cockpit_protocol::digest_json(&decision_value)
                .map_err(|error| ObserverError::State {
                    path: active_contract.clone(),
                    message: error.to_string(),
                })?
                .to_string()
                .into();
            summary["preflightRepositorySnapshotDigest"] =
                snapshot_digest(&snapshot)?.to_string().into();
            summary["preflightContractDigest"] =
                contract_digest(&active_contract)?.to_string().into();
            summary["preflightAt"] = now().into();
            atomic_json(&summary_path, &summary)?;
            return Ok(decision);
        }
        if !matches!(current_state, "implementation_active" | "checkpointed") {
            return Err(ObserverError::State {
                path: summary_path,
                message: format!(
                    "preflight is invalid from state {current_state:?}; expected implementation_active or checkpointed"
                ),
            });
        }
        let state = decision_state_name(decision.state.clone());
        let decision_value =
            serde_json::to_value(&raw_decision).map_err(|error| ObserverError::State {
                path: active_contract.clone(),
                message: error.to_string(),
            })?;
        summary["preflightState"] = state.into();
        summary["preflightDecisionDigest"] = cockpit_protocol::digest_json(&decision_value)
            .map_err(|error| ObserverError::State {
                path: active_contract.clone(),
                message: error.to_string(),
            })?
            .to_string()
            .into();
        summary["preflightRepositorySnapshotDigest"] =
            snapshot_digest(&snapshot)?.to_string().into();
        summary["preflightContractDigest"] = contract_digest(&active_contract)?.to_string().into();
        summary["preflightAt"] = now().into();
        atomic_json(&summary_path, &summary)?;
    }
    Ok(decision)
}

pub(super) fn decision_state_name(state: DecisionState) -> &'static str {
    match state {
        DecisionState::Green => "green",
        DecisionState::Yellow => "yellow",
        DecisionState::Red => "red",
    }
}

pub(super) fn contract_digest(path: &Path) -> Result<Digest, ObserverError> {
    let contract: serde_json::Value = read_json(path)?;
    cockpit_protocol::digest_json(&contract).map_err(|error| ObserverError::State {
        path: path.to_path_buf(),
        message: error.to_string(),
    })
}

pub(super) fn contract_digest_for_evidence(
    root: &Path,
    contract: &cockpit_protocol::Contract,
) -> Result<Digest, ObserverError> {
    let active = root
        .join(".ai/work-items/active")
        .join(format!("{}.contract.json", contract.work_item_id));
    let path = if active.is_file() {
        active
    } else {
        root.join(".ai/work-items/archive")
            .join(format!("{}.contract.json", contract.work_item_id))
    };
    contract_digest(&path)
}

fn require_green_or_yellow_preflight_governance(
    root: &Path,
    contract_path: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    preflight_state: &str,
) -> Result<(), ObserverError> {
    let decision =
        governance_decision_for_pre_execution_boundary(root, contract, snapshot, None, None)?;
    let decision =
        super::apply_preflight_review_evidence(root, contract, snapshot, decision, false, None)?;
    let current_state = decision_state_name(decision.state.clone());
    if current_state == "red" || preflight_state == "red" {
        return Err(ObserverError::State {
            path: contract_path.to_path_buf(),
            message: format!(
                "checkpoint requires a non-red governance result (preflight={preflight_state}, current={})",
                current_state
            ),
        });
    }
    if preflight_state == "yellow"
        && matches!(
            decision.review_state.as_deref(),
            Some("needs_human_confirmation")
        )
    {
        return Err(ObserverError::State {
            path: contract_path.to_path_buf(),
            message:
                "checkpoint requires human confirmation for an incomplete or uncertain Contract"
                    .into(),
        });
    }
    Ok(())
}

pub fn finish_work_item(
    root: &Path,
    work_item_id: &str,
) -> Result<LifecycleReceipt, ObserverError> {
    finish_work_item_internal(root, work_item_id, None)
}

/// Finish a Work Item while requiring evidence produced by the current
/// Runtime executable.  The unbound wrapper above is retained for embedders
/// that manage Runtime identity outside this crate; CLI/MCP use this bound
/// entry point.
pub fn finish_work_item_with_runtime(
    root: &Path,
    work_item_id: &str,
    runtime: &RuntimeContext,
) -> Result<LifecycleReceipt, ObserverError> {
    finish_work_item_internal(root, work_item_id, Some(runtime))
}

fn finish_work_item_internal(
    root: &Path,
    work_item_id: &str,
    current_runtime: Option<&RuntimeContext>,
) -> Result<LifecycleReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let canonical_root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let _lifecycle_lock = acquire_lifecycle_lock(&canonical_root, work_item_id)?;
    let result = finish_work_item_internal_unlocked(&canonical_root, work_item_id, current_runtime);
    if let Err(error) = &result
        && let Err(persist_error) =
            persist_blocked_lifecycle_outcome(&canonical_root, work_item_id, error)
    {
        return Err(ObserverError::State {
            path: canonical_root
                .join(".ai/work-items/active")
                .join(format!("{work_item_id}.outcome.json")),
            message: format!(
                "lifecycle gate failed: {error}; blocked Outcome persistence failed: {persist_error}"
            ),
        });
    }
    result
}

fn finish_work_item_internal_unlocked(
    root: &Path,
    work_item_id: &str,
    current_runtime: Option<&RuntimeContext>,
) -> Result<LifecycleReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let active = root.join(".ai/work-items/active");
    let summary_path = active.join(format!("{work_item_id}.summary.json"));
    let mut summary: serde_json::Value = read_json(&summary_path)?;
    let summary_state = summary["state"].as_str().unwrap_or("");
    let retry_recovery_pending = summary["recoveryRetryPending"] == serde_json::json!(true);
    let verification_recovery_reconciled =
        summary["verificationRecoveryReconciled"].as_str().is_some();
    require_current_retry_recovery_binding(&root, work_item_id, &summary, current_runtime)?;
    if summary_state != "checkpointed" {
        return Err(ObserverError::State {
            path: summary_path.clone(),
            message: format!(
                "finish is invalid from state {summary_state:?}; expected checkpointed"
            ),
        });
    }
    if summary["checkpointCount"] != serde_json::json!(1) {
        return Err(ObserverError::State {
            path: summary_path.clone(),
            message: "finish requires exactly one checkpoint".into(),
        });
    }
    let contract_path = active.join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&contract_path)?;
    if contract.checkpoint_policy.is_some() {
        let current_contract_hash = contract_digest(&contract_path)?.to_string();
        if summary["checkpointContractDigest"] != serde_json::json!(current_contract_hash) {
            return Err(ObserverError::State {
                path: summary_path.clone(),
                message: "finish requires a checkpoint for the current Contract".into(),
            });
        }
        let checkpoint_snapshot = summary["checkpointRepositorySnapshotDigest"]
            .as_str()
            .unwrap_or_default();
        let preflight_snapshot = summary["preflightRepositorySnapshotDigest"]
            .as_str()
            .unwrap_or_default();
        if checkpoint_snapshot.is_empty() || checkpoint_snapshot != preflight_snapshot {
            return Err(ObserverError::State {
                path: summary_path.clone(),
                message: "finish requires a checkpoint for the current repository snapshot".into(),
            });
        }
    }
    require_explicit_resource_finalization_plan(&contract, &contract_path, "finish")?;
    let original_summary = summary.clone();
    let evidence_path = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.verification.json"));
    let evidence = read_json(&evidence_path).map_err(|_| ObserverError::State {
        path: evidence_path.clone(),
        message: "finish requires a recorded verification receipt".into(),
    })?;
    if evidence["workItemId"].as_str() != Some(work_item_id)
        || evidence["passed"] != serde_json::Value::Bool(true)
    {
        return Err(ObserverError::State {
            path: evidence_path,
            message: "verification receipt is not a passed receipt for this work item".into(),
        });
    }
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let current_digest = snapshot_digest(&snapshot)?;
    if summary["preflightRepositorySnapshotDigest"]
        .as_str()
        .is_none_or(|value| value != current_digest.as_str())
    {
        return Err(ObserverError::State {
            path: summary_path.clone(),
            message: "finish requires a green preflight result for the current repository snapshot"
                .into(),
        });
    }
    if evidence["repositorySnapshotDigest"] != serde_json::Value::String(current_digest.to_string())
    {
        return Err(ObserverError::State {
            path: evidence_path,
            message: "verification receipt is stale for the current repository snapshot".into(),
        });
    }
    let contract_value = read_json(&contract_path)?;
    let controls = if let Some(runtime) = current_runtime {
        validate_contract_summary_controls_with_runtime(
            &contract,
            &contract_value,
            &summary,
            runtime,
        )
    } else {
        validate_contract_summary_controls(&contract, &contract_value, &summary)
    };
    if controls.state == "blocked" {
        return Err(ObserverError::State {
            path: contract_path,
            message: format!(
                "Contract/Summary governance controls are blocked: {}",
                controls
                    .findings
                    .iter()
                    .map(|item| item.code.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        });
    }
    if verification_evidence_state(&root, &contract, &snapshot, false, current_runtime)?
        != EvidenceState::Complete
    {
        return Err(ObserverError::State {
            path: evidence_path,
            message: "verification evidence is not a valid current receipt".into(),
        });
    }
    if summary["preflightState"] != serde_json::json!("green") {
        return Err(ObserverError::State {
            path: summary_path.clone(),
            message: "finish requires a green preflight result after verification".into(),
        });
    }
    if contract.checkpoint_policy.is_some() {
        let current_contract_hash = contract_digest(&contract_path)?.to_string();
        let has_before_finish = summary
            .get("checkpointEvidence")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|entries| {
                entries.iter().any(|entry| {
                    entry.get("stage").and_then(serde_json::Value::as_str) == Some("before_finish")
                })
            });
        if !has_before_finish {
            let passed = checkpoint_passed_check_count(&contract, &summary);
            append_checkpoint_evidence(
                &mut summary,
                &root,
                &contract,
                "before_finish",
                &snapshot,
                &current_contract_hash,
                passed,
                &now(),
            )?;
            atomic_json(&summary_path, &summary)?;
        }
        if let Err(errors) = validate_checkpoint_evidence_bindings(
            &contract,
            &summary,
            &repository_id(&root).to_string(),
            &current_digest.to_string(),
            &current_contract_hash,
        ) {
            return Err(ObserverError::State {
                path: summary_path.clone(),
                message: format!("checkpoint evidence is invalid: {}", errors.join(", ")),
            });
        }
    }
    if let Some(runtime) = current_runtime {
        require_green_governance_with_runtime(
            &root,
            &contract_path,
            &contract,
            &snapshot,
            "finish",
            runtime,
        )?;
    } else {
        require_green_governance(&root, &contract_path, &contract, &snapshot, "finish")?;
    }
    let timestamp = now();
    // A prior failed `finish` persists a blocked projection so recovery is
    // visible.  Once a fresh verification and governance pass succeeds, that
    // transient failure metadata is no longer current; keeping it would make
    // CI treat the repaired Work Item as stale and reject the branch.  Remove
    // only these generated projection fields before writing the new terminal
    // candidate; append-only failure events remain intact.
    if let Some(object) = summary.as_object_mut() {
        object.remove("failedGate");
        object.remove("recoveryCondition");
        object.remove("outcomeState");
        object.remove("verificationRecoveryReconciled");
    }
    summary["state"] = "finish_ready".into();
    summary["updatedAt"] = timestamp.clone().into();
    atomic_json(&summary_path, &summary)?;
    let evidence_ref = format!(".ai/evidence/{work_item_id}.verification.json");
    let task_report = task_outcome_report(TaskOutcomeReportInput {
        root: &root,
        contract_path: &contract_path,
        contract: &contract,
        summary: Some(&summary),
        snapshot_digest: snapshot_digest(&snapshot).ok(),
        state: OutcomeState::Verified,
        decision_state: DecisionState::Green,
        summary_text: "Verification evidence passed; human-visible benefit remains explicitly unknown unless declared by the Work Item owner.",
        unknowns: &["user_visible_benefit_not_declared".into()],
        evidence_ref: &evidence_ref,
        failed_gate_override: None,
        recovery_condition_override: None,
        historical: false,
    });
    let (task_report_digest, task_report_markdown_digest) = write_task_outcome_artifacts(
        &root,
        work_item_id,
        &task_report,
        retry_recovery_pending || verification_recovery_reconciled,
    )?;
    if retry_recovery_pending {
        summary
            .as_object_mut()
            .expect("Work Item Summary is an object")
            .remove("recoveryRetryPending");
        summary
            .as_object_mut()
            .expect("Work Item Summary is an object")
            .remove("recoveryRetryDecisionPath");
        summary
            .as_object_mut()
            .expect("Work Item Summary is an object")
            .remove("recoveryRetryDecisionDigest");
        if let Err(error) = atomic_json(&summary_path, &summary) {
            // marker の削除に失敗した場合は元の Summary と今回のレポートを戻し、
            // finish_ready と retry marker の矛盾した投影を残さない。
            let _ = atomic_json(&summary_path, &original_summary);
            let _ = fs::remove_file(active.join(format!("{work_item_id}.task-report.json")));
            let _ = fs::remove_file(active.join(format!("{work_item_id}.task-report.md")));
            return Err(error);
        }
    }
    let outcome_v2 = OutcomeV2 {
        schema_version: 2,
        repository_id: contract.repository_id.clone(),
        work_item_id: work_item_id.into(),
        state: OutcomeState::Verified,
        decision_state: Some(DecisionState::Green),
        summary: "Verification evidence passed; human-visible benefit remains explicitly unknown unless declared by the Work Item owner.".into(),
        acceptance_results: contract.acceptance_criteria.clone(),
        unknowns: vec!["user_visible_benefit_not_declared".into()],
        evidence_refs: vec![evidence_ref],
        human_benefit_report: HumanBenefitReport {
            state: OutcomeState::Unknown,
            user_visible_changes: Vec::new(),
            affected_users: Vec::new(),
            unknowns: vec!["user_visible_benefit_not_declared".into()],
            evidence_refs: vec![format!(".ai/evidence/{work_item_id}.verification.json")],
        },
        task_outcome_report: Some(task_report.clone()),
        failed_gate: None,
        recovery_condition: None,
        recovery_decision: None,
        historical_status: None,
        governance_reasons: Vec::new(),
        finalization: None,
    };
    let mut outcome = serde_json::to_value(&outcome_v2).map_err(|error| ObserverError::State {
        path: active.join(format!("{work_item_id}.outcome.json")),
        message: error.to_string(),
    })?;
    outcome["protocolVersion"] = serde_json::json!(1);
    outcome["workItemId"] = serde_json::json!(work_item_id);
    outcome["state"] = serde_json::json!("finish_ready");
    outcome["verification"] = serde_json::json!({
        "status": "verified",
        "required": true,
        "evidencePath": format!(".ai/evidence/{work_item_id}.verification.json"),
    });
    outcome["evidenceDigest"] = cockpit_protocol::digest_json(&evidence)
        .map_err(|error| ObserverError::State {
            path: root.join(".ai/evidence"),
            message: error.to_string(),
        })?
        .to_string()
        .into();
    outcome["taskReportDigest"] = task_report_digest.to_string().into();
    outcome["taskReportMarkdownDigest"] = task_report_markdown_digest.to_string().into();
    outcome["createdAt"] = timestamp.clone().into();
    if let Err(error) = atomic_json(
        &active.join(format!("{work_item_id}.outcome.json")),
        &outcome,
    ) {
        let _ = atomic_json(&summary_path, &original_summary);
        let _ = fs::remove_file(active.join(format!("{work_item_id}.task-report.json")));
        let _ = fs::remove_file(active.join(format!("{work_item_id}.task-report.md")));
        return Err(error);
    }
    if let Err(error) = append_task_outcome_events(
        &root,
        &contract,
        &task_report,
        retry_recovery_pending || verification_recovery_reconciled,
    ) {
        let _ = fs::remove_file(active.join(format!("{work_item_id}.outcome.json")));
        let _ = fs::remove_file(active.join(format!("{work_item_id}.task-report.json")));
        let _ = fs::remove_file(active.join(format!("{work_item_id}.task-report.md")));
        let _ = atomic_json(&summary_path, &original_summary);
        return Err(error);
    }
    Ok(LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "finish_ready".into(),
        timestamp,
    })
}

pub fn record_verification(
    root: &Path,
    work_item_id: &str,
    receipt: &serde_json::Value,
    runtime_version: &str,
    runtime_digest: &Digest,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
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
    record_verification_with_snapshot(
        &root,
        work_item_id,
        receipt,
        runtime_version,
        runtime_digest,
        &snapshot,
    )
}

pub(super) const RECOVERY_DECISION_INVALID: &str = "recovery_decision_invalid";

pub(super) fn recovery_decision_error(
    path: impl Into<PathBuf>,
    code: &str,
    detail: impl std::fmt::Display,
) -> ObserverError {
    ObserverError::State {
        path: path.into(),
        message: format!("{RECOVERY_DECISION_INVALID}:{code}: {detail}"),
    }
}

/// Validate the historical verification envelope used by an archived
/// Contract-amendment revalidation.  The current archive Contract may have a
/// different digest, but the old evidence must remain an intact, successful,
/// repository-bound v2 record and its bytes are never rewritten.
pub(super) fn validate_archived_revalidation_evidence(
    root: &Path,
    work_item_id: &str,
    historical_contract_digest: &Digest,
    expected_evidence_digest: &Digest,
) -> Result<(), ObserverError> {
    let path = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.verification.json"));
    if !is_regular_non_symlink(&path)? {
        return Err(recovery_decision_error(
            path,
            "revalidation_evidence_invalid",
            "historical verification evidence must be a regular non-symlink file",
        ));
    }
    let bytes = fs::read(&path).map_err(|source| ObserverError::Read {
        path: path.clone(),
        source,
    })?;
    if Digest::sha256_bytes(&bytes) != *expected_evidence_digest {
        return Err(recovery_decision_error(
            path,
            "revalidation_evidence_digest_mismatch",
            "historical verification evidence bytes do not match the recovery binding",
        ));
    }
    reject_duplicate_json_keys(&bytes).map_err(|message| {
        recovery_decision_error(
            path.clone(),
            "revalidation_evidence_invalid",
            format!("historical verification evidence JSON is invalid: {message}"),
        )
    })?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
        recovery_decision_error(
            path.clone(),
            "revalidation_evidence_invalid",
            format!("historical verification evidence JSON is invalid: {error}"),
        )
    })?;
    let envelope: VerificationEvidenceEnvelope =
        serde_json::from_value(value.clone()).map_err(|error| {
            recovery_decision_error(
                path.clone(),
                "revalidation_evidence_invalid",
                format!("historical verification evidence schema is invalid: {error}"),
            )
        })?;
    let expected_repository_id = repository_id(root).to_string();
    if envelope.protocol_version != 1
        || envelope.evidence_schema_version != 2
        || envelope.work_item_id != work_item_id
        || envelope.repository_id != expected_repository_id
        || !envelope.passed
        || envelope.contract_digest.as_ref() != Some(historical_contract_digest)
        || envelope.runtime_version.trim().is_empty()
        || !valid_sha256_digest(&envelope.runtime_digest.to_string())
        || !valid_sha256_digest(&envelope.repository_snapshot_digest.to_string())
        || !valid_sha256_digest(&envelope.receipt_digest.to_string())
        || chrono::DateTime::parse_from_rfc3339(&envelope.created_at).is_err()
    {
        return Err(recovery_decision_error(
            path,
            "revalidation_evidence_invalid",
            "historical verification evidence is missing a strict predecessor binding",
        ));
    }
    match envelope.capture_mode {
        VerificationCaptureMode::DigestOnly => {
            if envelope.receipt.is_some() {
                return Err(recovery_decision_error(
                    path,
                    "revalidation_evidence_invalid",
                    "digest-only historical verification evidence must not contain a receipt",
                ));
            }
        }
        VerificationCaptureMode::FullCapture | VerificationCaptureMode::RedactedCapture => {
            let Some(receipt) = envelope.receipt.as_ref() else {
                return Err(recovery_decision_error(
                    path,
                    "revalidation_evidence_invalid",
                    "historical verification evidence is missing its captured receipt",
                ));
            };
            let typed: cockpit_verification::VerificationReceipt =
                serde_json::from_value(receipt.clone()).map_err(|error| {
                    recovery_decision_error(
                        path.clone(),
                        "revalidation_evidence_invalid",
                        format!("historical verification receipt schema is invalid: {error}"),
                    )
                })?;
            if !typed.passed
                || typed.work_item_id.as_deref() != Some(work_item_id)
                || typed.repository_id.as_deref() != Some(expected_repository_id.as_str())
                || typed.runtime_version.as_deref() != Some(envelope.runtime_version.as_str())
                || typed.runtime_digest.as_deref()
                    != Some(envelope.runtime_digest.to_string().as_str())
            {
                return Err(recovery_decision_error(
                    path,
                    "revalidation_evidence_invalid",
                    "historical verification receipt identity is not repository-bound",
                ));
            }
            let receipt_digest = cockpit_protocol::digest_json(receipt).map_err(|error| {
                recovery_decision_error(path.clone(), "revalidation_evidence_invalid", error)
            })?;
            if receipt_digest != envelope.receipt_digest {
                return Err(recovery_decision_error(
                    path,
                    "revalidation_evidence_digest_mismatch",
                    "historical verification receipt digest does not match its envelope",
                ));
            }
        }
        VerificationCaptureMode::LegacyUntyped => {
            return Err(recovery_decision_error(
                path,
                "revalidation_evidence_invalid",
                "legacy untyped verification evidence cannot authorize revalidation",
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_recovery_predecessor_bindings(
    root: &Path,
    work_item_id: &str,
    receipt: &RecoveryDecisionReceipt,
    current_runtime: Option<&RuntimeContext>,
    contract_path: &Path,
    summary_path: &Path,
    candidate_path: Option<&Path>,
) -> Result<(), ObserverError> {
    let decisions = root.join(".ai/decisions");
    let amendment_revalidation =
        receipt.successor_binding_mode.as_deref() == Some("contract_amendment_revalidation");
    if receipt.schema_version != 1
        || receipt.decision_id != "work-item-recovery"
        || !matches!(
            receipt.decision.as_str(),
            "retry" | "successor" | "supersede"
        )
        || receipt.work_item_id != work_item_id
        || receipt.predecessor_work_item_id != work_item_id
        || receipt.repository_id != repository_id(root).to_string()
        || receipt.actor.trim().is_empty()
        || receipt.authority_source.trim().is_empty()
        || receipt.reason.trim().is_empty()
        || receipt.resume_condition.trim().is_empty()
    {
        return Err(recovery_decision_error(
            decisions,
            "identity_mismatch",
            "repository, Work Item, decision, or authority identity is invalid",
        ));
    }
    if receipt.predecessor_archive_manifest_digest.is_some() {
        validate_recovery_archive_manifest_binding(root, work_item_id, receipt)?;
    }
    if amendment_revalidation {
        if receipt.decision != "successor"
            || receipt.current_contract_digest.is_none()
            || receipt.predecessor_archive_manifest_digest.is_none()
            || receipt.predecessor_verification_evidence_digest.is_none()
        {
            return Err(recovery_decision_error(
                decisions,
                "revalidation_binding_invalid",
                "contract amendment revalidation requires successor, current Contract, archive manifest, and historical verification bindings",
            ));
        }
    } else if receipt.current_contract_digest.is_some()
        || receipt.predecessor_verification_evidence_digest.is_some()
    {
        return Err(recovery_decision_error(
            decisions,
            "revalidation_binding_invalid",
            "current Contract and historical verification bindings require contract_amendment_revalidation",
        ));
    }
    match receipt.decision.as_str() {
        "retry" if receipt.successor_work_item_id.is_some() => {
            return Err(recovery_decision_error(
                decisions,
                "successor_identity_invalid",
                "retry recovery decision must not include successorWorkItemId",
            ));
        }
        "successor" | "supersede" => {
            let Some(successor_id) = receipt.successor_work_item_id.as_deref() else {
                return Err(recovery_decision_error(
                    decisions,
                    "successor_identity_invalid",
                    "successor recovery decision requires successorWorkItemId",
                ));
            };
            validate_work_item_id(successor_id).map_err(|_| {
                recovery_decision_error(
                    root.join(".ai/work-items"),
                    "successor_identity_invalid",
                    "successor Work Item identity is invalid",
                )
            })?;
            if successor_id == work_item_id {
                return Err(recovery_decision_error(
                    root.join(".ai/work-items"),
                    "successor_identity_invalid",
                    "successor Work Item equals predecessor",
                ));
            }
        }
        _ => {}
    }
    if chrono::DateTime::parse_from_rfc3339(&receipt.decided_at).is_err() {
        return Err(recovery_decision_error(
            decisions,
            "timestamp_invalid",
            "decidedAt must be RFC3339",
        ));
    }
    if let Some(runtime) = current_runtime
        && (receipt.runtime_version != runtime.runtime_version
            || receipt.runtime_digest != runtime.runtime_digest)
    {
        return Err(recovery_decision_error(
            decisions,
            "runtime_mismatch",
            "recovery decision Runtime identity does not match the current Runtime",
        ));
    }

    let expected_contract_digest = contract_digest(contract_path)?;
    let contract_binding = receipt
        .current_contract_digest
        .as_ref()
        .unwrap_or(&receipt.predecessor_contract_digest);
    if contract_binding != &expected_contract_digest {
        return Err(recovery_decision_error(
            contract_path,
            "predecessor_contract_mismatch",
            "predecessor Contract digest mismatch",
        ));
    }
    if amendment_revalidation {
        if receipt.predecessor_contract_digest == expected_contract_digest {
            return Err(recovery_decision_error(
                contract_path,
                "revalidation_binding_invalid",
                "contract amendment revalidation requires a historical Contract digest distinct from the current Contract",
            ));
        }
        validate_archived_revalidation_evidence(
            root,
            work_item_id,
            &receipt.predecessor_contract_digest,
            receipt
                .predecessor_verification_evidence_digest
                .as_ref()
                .expect("revalidation evidence binding validated"),
        )?;
    }
    let summary = read_json(summary_path)?;
    let expected_summary_digest =
        cockpit_protocol::digest_json(&summary).map_err(|error| ObserverError::State {
            path: summary_path.into(),
            message: error.to_string(),
        })?;
    let retry_binding =
        retry_recovery_binding_matches(root, work_item_id, &summary, receipt, candidate_path)?;
    if receipt.predecessor_summary_digest != expected_summary_digest && !retry_binding {
        return Err(recovery_decision_error(
            summary_path,
            "predecessor_summary_mismatch",
            "predecessor Summary digest mismatch",
        ));
    }

    let outcome_path = work_item_artifact_path_optional(root, work_item_id, "outcome.json")?;
    match (&receipt.predecessor_outcome_digest, outcome_path.as_ref()) {
        (Some(expected), Some(path)) => {
            let actual = cockpit_protocol::digest_json(&read_json(path)?).map_err(|error| {
                ObserverError::State {
                    path: path.clone(),
                    message: error.to_string(),
                }
            })?;
            if expected != &actual && !retry_binding {
                return Err(recovery_decision_error(
                    path,
                    "predecessor_outcome_mismatch",
                    "predecessor Outcome digest mismatch",
                ));
            }
        }
        (Some(_), None) | (None, Some(_)) => {
            return Err(recovery_decision_error(
                root.join(".ai/work-items"),
                "predecessor_outcome_presence_mismatch",
                "predecessor Outcome presence does not match the recovery decision",
            ));
        }
        (None, None) => {}
    }

    let events_path = work_item_artifact_path_optional(root, work_item_id, "events.jsonl")?;
    match (&receipt.predecessor_events_digest, events_path.as_ref()) {
        (Some(expected), Some(path)) => {
            let actual =
                Digest::sha256_bytes(&fs::read(path).map_err(|source| ObserverError::Read {
                    path: path.clone(),
                    source,
                })?);
            if expected != &actual && !retry_binding {
                return Err(recovery_decision_error(
                    path,
                    "predecessor_events_mismatch",
                    "predecessor Events digest mismatch",
                ));
            }
        }
        (Some(_), None) | (None, Some(_)) => {
            return Err(recovery_decision_error(
                root.join(".ai/work-items"),
                "predecessor_events_presence_mismatch",
                "predecessor Events presence does not match the recovery decision",
            ));
        }
        (None, None) => {}
    }
    Ok(())
}

fn retry_recovery_binding_matches(
    root: &Path,
    work_item_id: &str,
    summary: &serde_json::Value,
    receipt: &RecoveryDecisionReceipt,
    candidate_path: Option<&Path>,
) -> Result<bool, ObserverError> {
    if receipt.decision != "retry"
        || summary["state"] != serde_json::json!("checkpointed")
        || summary["recoveryRetryPending"] != serde_json::json!(true)
    {
        return Ok(false);
    }
    let Some(candidate_path) = candidate_path else {
        return Ok(false);
    };
    let Some(file_name) = candidate_path.file_name().and_then(|value| value.to_str()) else {
        return Ok(false);
    };
    let canonical = format!("{work_item_id}.recovery.json");
    let versioned_prefix = format!("{work_item_id}.recovery.");
    if file_name != canonical
        && !(file_name.starts_with(&versioned_prefix) && file_name.ends_with(".json"))
    {
        return Ok(false);
    }
    let expected_path = summary["recoveryRetryDecisionPath"].as_str();
    if expected_path != Some(repository_relative_path(root, candidate_path).as_str()) {
        return Ok(false);
    }
    let value = serde_json::to_value(receipt).map_err(|error| ObserverError::State {
        path: root.join(".ai/decisions"),
        message: error.to_string(),
    })?;
    let digest = cockpit_protocol::digest_json(&value).map_err(|error| ObserverError::State {
        path: root.join(".ai/decisions"),
        message: error.to_string(),
    })?;
    Ok(summary["recoveryRetryDecisionDigest"] == serde_json::json!(digest.to_string()))
}

/// Verify that a pending retry marker is backed by the current Runtime-owned
/// recovery receipt before any lifecycle operation consumes the marker.
fn require_current_retry_recovery_binding(
    root: &Path,
    work_item_id: &str,
    summary: &serde_json::Value,
    current_runtime: Option<&RuntimeContext>,
) -> Result<(), ObserverError> {
    if summary["recoveryRetryPending"] != serde_json::json!(true) {
        return Ok(());
    }
    let decision = load_recovery_decision(root, work_item_id, current_runtime)?;
    if decision
        .as_ref()
        .is_some_and(|value| value.decision == "retry")
    {
        return Ok(());
    }
    Err(recovery_decision_error(
        root.join(".ai/decisions"),
        "retry_binding_missing",
        "pending retry marker has no current retry recovery receipt",
    ))
}

pub(super) fn validate_recovery_successor_binding(
    root: &Path,
    work_item_id: &str,
    receipt: &RecoveryDecisionReceipt,
) -> Result<bool, ObserverError> {
    if receipt.decision == "retry" {
        return Ok(false);
    }
    let successor_id = receipt
        .successor_work_item_id
        .as_deref()
        .expect("recovery identity validator requires a successor");
    let successor_contract_path =
        work_item_artifact_path_optional(root, successor_id, "contract.json")?.ok_or_else(
            || {
                recovery_decision_error(
                    root.join(".ai/work-items"),
                    "successor_binding_missing",
                    "recovery decision requires an existing successor Contract",
                )
            },
        )?;
    let successor_contract = read_contract(&successor_contract_path).map_err(|error| {
        recovery_decision_error(&successor_contract_path, "successor_binding_invalid", error)
    })?;
    let expected_repository_id = repository_id(root).to_string();
    if successor_contract.work_item_id != successor_id
        || successor_contract.repository_id != expected_repository_id
    {
        return Err(recovery_decision_error(
            successor_contract_path.clone(),
            "successor_binding_mismatch",
            "successor Contract does not bind the predecessor repository, identity, and Contract digest",
        ));
    }
    if successor_contract.recovery_decision_path.is_none()
        && successor_contract.predecessor_work_item_id.is_some()
    {
        return Err(recovery_decision_error(
            successor_contract_path.clone(),
            "successor_binding_mismatch",
            "successor Contract has an incomplete predecessor binding",
        ));
    }
    if let Some(mode) = receipt.successor_binding_mode.as_deref()
        && !matches!(
            mode,
            "legacy_terminal_evidence" | "contract_amendment_revalidation"
        )
    {
        return Err(recovery_decision_error(
            successor_contract_path.clone(),
            "successor_binding_mode_invalid",
            "successorBindingMode is not a recognized Runtime marker",
        ));
    }
    let expected_predecessor_contract_digest = receipt
        .current_contract_digest
        .as_ref()
        .unwrap_or(&receipt.predecessor_contract_digest);
    let strictly_bound = successor_contract.predecessor_work_item_id.as_deref()
        == Some(work_item_id)
        && successor_contract.predecessor_contract_digest.as_ref()
            == Some(expected_predecessor_contract_digest)
        && successor_contract.recovery_decision_path.is_some();
    if strictly_bound {
        if receipt.successor_binding_mode.is_some()
            && receipt.successor_binding_mode.as_deref() != Some("contract_amendment_revalidation")
        {
            return Err(recovery_decision_error(
                successor_contract_path,
                "successor_binding_mode_invalid",
                "legacy successorBindingMode cannot be used by a strictly bound successor",
            ));
        }
        return Ok(false);
    }

    // Older Runtime versions could create a successor selected by a valid
    // recovery receipt before predecessor fields became mandatory. Preserve
    // that historical path only when all lineage fields are absent and the
    // successor already has complete, repository-bound terminal evidence.
    let legacy_shape = successor_contract.predecessor_work_item_id.is_none()
        && successor_contract.predecessor_contract_digest.is_none()
        && successor_contract.recovery_decision_path.is_none();
    if legacy_shape {
        validate_legacy_successor_terminal_evidence(
            root,
            successor_id,
            &expected_repository_id,
            &successor_contract_path,
        )?;
        return Ok(true);
    }
    Err(recovery_decision_error(
        successor_contract_path,
        "successor_binding_mismatch",
        "successor Contract does not bind the predecessor repository, identity, and Contract digest",
    ))
}

/// Validate the narrow compatibility boundary for a successor Contract
/// emitted before predecessor fields became mandatory. This is deliberately
/// stricter than merely finding an archive: every terminal projection and its
/// repository/Work Item bindings must be present and content-bound.
fn validate_legacy_successor_terminal_evidence(
    root: &Path,
    successor_id: &str,
    expected_repository_id: &str,
    successor_contract_path: &Path,
) -> Result<(), ObserverError> {
    let archive = root
        .join(".ai/work-items/archive")
        .join(format!("{successor_id}.archive.json"));
    let has_manifest = is_regular_non_symlink(&archive).map_err(|error| {
        recovery_decision_error(&archive, "legacy_successor_evidence_missing", error)
    })?;
    if !has_manifest {
        return Err(recovery_decision_error(
            archive,
            "legacy_successor_evidence_missing",
            "legacy successor has no archive manifest",
        ));
    }
    let manifest = read_json(&archive).map_err(|error| {
        recovery_decision_error(&archive, "legacy_successor_evidence_invalid", error)
    })?;
    if manifest["state"] != serde_json::json!("archived")
        || manifest["closeRequired"] != serde_json::json!(true)
    {
        return Err(recovery_decision_error(
            archive.clone(),
            "legacy_successor_evidence_invalid",
            "legacy successor archive is not a normal close-required archive",
        ));
    }
    verify_archive_manifest(root, successor_id, &manifest).map_err(|error| {
        recovery_decision_error(&archive, "legacy_successor_evidence_invalid", error)
    })?;

    let successor_contract = read_contract(successor_contract_path).map_err(|error| {
        recovery_decision_error(
            successor_contract_path,
            "legacy_successor_evidence_invalid",
            error,
        )
    })?;
    let expected_contract_digest = contract_digest(successor_contract_path).map_err(|error| {
        recovery_decision_error(
            successor_contract_path,
            "legacy_successor_evidence_invalid",
            error,
        )
    })?;
    if successor_contract.work_item_id != successor_id
        || successor_contract.repository_id != expected_repository_id
    {
        return Err(recovery_decision_error(
            successor_contract_path,
            "legacy_successor_evidence_invalid",
            "legacy successor Contract identity is not repository-bound",
        ));
    }

    let summary_path = root
        .join(".ai/work-items/archive")
        .join(format!("{successor_id}.summary.json"));
    let summary = read_json(&summary_path).map_err(|error| {
        recovery_decision_error(&summary_path, "legacy_successor_evidence_invalid", error)
    })?;
    if summary["workItemId"] != serde_json::json!(successor_id)
        || summary["state"] != serde_json::json!("finish_ready")
        || summary["checkpointCount"] != serde_json::json!(1)
        || summary["preflightState"] != serde_json::json!("green")
    {
        return Err(recovery_decision_error(
            summary_path,
            "legacy_successor_evidence_invalid",
            "legacy successor Summary is not a verified terminal projection",
        ));
    }

    let outcome_path = root
        .join(".ai/work-items/archive")
        .join(format!("{successor_id}.outcome.json"));
    let outcome = read_json(&outcome_path).map_err(|error| {
        recovery_decision_error(&outcome_path, "legacy_successor_evidence_invalid", error)
    })?;
    if outcome["workItemId"] != serde_json::json!(successor_id)
        || outcome["verification"]["status"] != serde_json::json!("verified")
    {
        return Err(recovery_decision_error(
            outcome_path,
            "legacy_successor_evidence_invalid",
            "legacy successor Outcome is not verified",
        ));
    }

    let evidence_path = root
        .join(".ai/evidence")
        .join(format!("{successor_id}.verification.json"));
    if !is_regular_non_symlink(&evidence_path).map_err(|error| {
        recovery_decision_error(&evidence_path, "legacy_successor_evidence_missing", error)
    })? {
        return Err(recovery_decision_error(
            evidence_path,
            "legacy_successor_evidence_missing",
            "legacy successor has no verification evidence",
        ));
    }
    let evidence = read_json(&evidence_path).map_err(|error| {
        recovery_decision_error(&evidence_path, "legacy_successor_evidence_invalid", error)
    })?;
    let runtime_version = evidence["runtimeVersion"].as_str().unwrap_or_default();
    let runtime_digest = evidence["runtimeDigest"].as_str().unwrap_or_default();
    let snapshot_digest = evidence["repositorySnapshotDigest"]
        .as_str()
        .unwrap_or_default();
    if evidence["evidenceSchemaVersion"] != serde_json::json!(2)
        || evidence["workItemId"] != serde_json::json!(successor_id)
        || evidence["repositoryId"] != serde_json::json!(expected_repository_id)
        || evidence["passed"] != serde_json::json!(true)
        || evidence["contractDigest"] != serde_json::json!(expected_contract_digest)
        || runtime_version.trim().is_empty()
        || !valid_sha256_digest(runtime_digest)
        || !valid_sha256_digest(snapshot_digest)
    {
        return Err(recovery_decision_error(
            evidence_path,
            "legacy_successor_evidence_invalid",
            "legacy successor verification evidence is missing a strict identity binding",
        ));
    }
    if let Some(receipt) = evidence.get("receipt") {
        let Some(receipt_digest) = evidence["receiptDigest"].as_str() else {
            return Err(recovery_decision_error(
                evidence_path,
                "legacy_successor_evidence_invalid",
                "legacy successor verification receipt digest is missing",
            ));
        };
        let actual = cockpit_protocol::digest_json(receipt).map_err(|error| {
            recovery_decision_error(&evidence_path, "legacy_successor_evidence_invalid", error)
        })?;
        if receipt_digest != actual.to_string() {
            return Err(recovery_decision_error(
                evidence_path,
                "legacy_successor_evidence_invalid",
                "legacy successor verification receipt digest is stale",
            ));
        }
    }
    let close_path = root
        .join(".ai/decisions")
        .join(format!("{successor_id}.close.json"));
    if !close_decision_is_valid_for_status(root, successor_id, expected_repository_id) {
        return Err(recovery_decision_error(
            close_path,
            "legacy_successor_evidence_missing",
            "legacy successor has no valid confirmed close decision",
        ));
    }
    Ok(())
}

/// Record an append-only successor revalidation for an archived Work Item
/// whose reviewed Contract bytes changed after the historical verification.
/// The command derives every repository fact from current archive bytes and
/// requires explicit human authority; it never edits the predecessor archive
/// or evidence.
#[derive(Clone, Debug, Default)]
pub struct ArchivedContractRevalidationRequest {
    pub successor_work_item_id: String,
    pub reason: String,
    pub actor: String,
    pub authority_source: String,
    pub resume_condition: String,
    pub evidence_refs: Vec<String>,
    pub policy_refs: Vec<String>,
}

pub fn revalidate_archived_work_item_with_runtime(
    root: &Path,
    work_item_id: &str,
    request: &ArchivedContractRevalidationRequest,
    runtime: &RuntimeContext,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    validate_work_item_id(&request.successor_work_item_id)?;
    for (field, value) in [
        ("reason", request.reason.as_str()),
        ("actor", request.actor.as_str()),
        ("authoritySource", request.authority_source.as_str()),
        ("resumeCondition", request.resume_condition.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(ObserverError::State {
                path: root.join(".ai/decisions"),
                message: format!("{field} must not be empty for archived revalidation"),
            });
        }
    }
    if work_item_id == request.successor_work_item_id {
        return Err(ObserverError::State {
            path: root.join(".ai/work-items"),
            message: "archived revalidation successor must differ from its predecessor".into(),
        });
    }
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let archive = root.join(".ai/work-items/archive");
    let manifest_path = archive.join(format!("{work_item_id}.archive.json"));
    let manifest_bytes = fs::read(&manifest_path).map_err(|source| ObserverError::Read {
        path: manifest_path.clone(),
        source,
    })?;
    let manifest: serde_json::Value =
        serde_json::from_slice(&manifest_bytes).map_err(|error| ObserverError::State {
            path: manifest_path.clone(),
            message: format!("archived revalidation manifest is invalid: {error}"),
        })?;
    if manifest["state"] != serde_json::json!("archived") {
        return Err(ObserverError::State {
            path: manifest_path,
            message: "archived revalidation requires an archive still awaiting close".into(),
        });
    }
    verify_archive_manifest(&root, work_item_id, &manifest)?;

    let contract_path = archive.join(format!("{work_item_id}.contract.json"));
    let summary_path = archive.join(format!("{work_item_id}.summary.json"));
    let outcome_path = archive.join(format!("{work_item_id}.outcome.json"));
    let summary = read_json(&summary_path)?;
    let outcome = read_json(&outcome_path)?;
    let current_contract_digest = contract_digest(&contract_path)?;
    let evidence_path = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.verification.json"));
    let evidence_bytes = fs::read(&evidence_path).map_err(|source| ObserverError::Read {
        path: evidence_path.clone(),
        source,
    })?;
    reject_duplicate_json_keys(&evidence_bytes).map_err(|message| ObserverError::State {
        path: evidence_path.clone(),
        message: format!("historical verification evidence JSON is invalid: {message}"),
    })?;
    let evidence_value: serde_json::Value =
        serde_json::from_slice(&evidence_bytes).map_err(|error| ObserverError::State {
            path: evidence_path.clone(),
            message: format!("historical verification evidence JSON is invalid: {error}"),
        })?;
    let historical_contract_digest = evidence_value
        .get("contractDigest")
        .and_then(serde_json::Value::as_str)
        .and_then(|value| value.parse::<Digest>().ok())
        .ok_or_else(|| {
            recovery_decision_error(
                &evidence_path,
                "revalidation_evidence_invalid",
                "historical verification evidence has no valid Contract digest",
            )
        })?;
    let evidence_digest = Digest::sha256_bytes(&evidence_bytes);
    validate_archived_revalidation_evidence(
        &root,
        work_item_id,
        &historical_contract_digest,
        &evidence_digest,
    )?;
    if historical_contract_digest == current_contract_digest {
        return Err(recovery_decision_error(
            contract_path,
            "revalidation_binding_invalid",
            "archived Contract has not changed relative to its historical verification evidence",
        ));
    }
    // Resource-finalization receipts historically used a raw Contract-file
    // digest, while verification evidence uses the canonical JSON digest.
    // Preserve that exact predecessor identity when a provider receipt
    // already exists; close/verify can then accept the old receipt without
    // weakening the amended Contract binding or asking a human to rewrite it.
    let predecessor_finalization_contract_digest = {
        let finalization_path = resource_finalization_decision_path(&root, work_item_id);
        if fs::symlink_metadata(&finalization_path).is_ok() {
            let finalization = read_resource_finalization_receipt(&finalization_path)?;
            let current_contract = read_contract(&contract_path)?;
            validate_resource_finalization_receipt_for(
                &finalization,
                &current_contract.repository_id,
                work_item_id,
                None,
                current_contract.resource_context.as_ref(),
            )
            .map_err(|error| ObserverError::State {
                path: finalization_path.clone(),
                message: format!(
                    "existing provider finalization cannot be bound for archived revalidation: {error}"
                ),
            })?;
            ensure_resource_finalization_base_binding(
                &finalization,
                &current_contract,
                &finalization_path,
            )?;
            Some(finalization.contract_digest.ok_or_else(|| {
                recovery_decision_error(
                    &finalization_path,
                    "revalidation_finalization_binding_invalid",
                    "existing provider finalization has no Contract digest",
                )
            })?)
        } else {
            None
        }
    };
    if let Ok(Some(existing)) = load_recovery_decision(&root, work_item_id, None)
        && existing.successor_binding_mode.as_deref() == Some("contract_amendment_revalidation")
        && existing.successor_work_item_id.as_deref() == Some(&request.successor_work_item_id)
        && existing.current_contract_digest.as_ref() == Some(&current_contract_digest)
        && existing.predecessor_contract_digest == historical_contract_digest
        && existing.predecessor_verification_evidence_digest.as_ref() == Some(&evidence_digest)
    {
        let mut result = serde_json::to_value(existing).map_err(|error| ObserverError::State {
            path: root.join(".ai/decisions"),
            message: error.to_string(),
        })?;
        result["state"] = serde_json::json!("idempotent");
        return Ok(result);
    }
    let events_path = archive.join(format!("{work_item_id}.events.jsonl"));
    let predecessor_events_digest = if optional_regular_artifact(&events_path, "archived Events")? {
        Some(Digest::sha256_bytes(&fs::read(&events_path).map_err(
            |source| ObserverError::Read {
                path: events_path.clone(),
                source,
            },
        )?))
    } else {
        None
    };
    let receipt = serde_json::json!({
        "schemaVersion": 1,
        "decisionId": "work-item-recovery",
        "decision": "successor",
        "workItemId": work_item_id,
        "repositoryId": repository_id(&root),
        "predecessorWorkItemId": work_item_id,
        "predecessorContractDigest": historical_contract_digest,
        "currentContractDigest": current_contract_digest,
        "predecessorSummaryDigest": cockpit_protocol::digest_json(&summary).map_err(|error| ObserverError::State {
            path: summary_path.clone(),
            message: error.to_string(),
        })?,
        "predecessorOutcomeDigest": cockpit_protocol::digest_json(&outcome).map_err(|error| ObserverError::State {
            path: outcome_path.clone(),
            message: error.to_string(),
        })?,
        "predecessorEventsDigest": predecessor_events_digest,
        "predecessorArchiveManifestDigest": Digest::sha256_bytes(&manifest_bytes),
        "predecessorVerificationEvidenceDigest": evidence_digest,
        "predecessorFinalizationContractDigest": predecessor_finalization_contract_digest,
        "successorWorkItemId": &request.successor_work_item_id,
        "successorBindingMode": "contract_amendment_revalidation",
        "runtimeVersion": runtime.runtime_version,
        "runtimeDigest": runtime.runtime_digest,
        "actor": request.actor.trim(),
        "authoritySource": request.authority_source.trim(),
        "reason": request.reason.trim(),
        "evidenceRefs": &request.evidence_refs,
        "policyRefs": &request.policy_refs,
        "decidedAt": now(),
        "resumeCondition": request.resume_condition.trim()
    });
    let recorded = record_recovery_decision(&root, work_item_id, &receipt, runtime)?;
    let mut result = recorded;
    result["state"] = serde_json::json!("recorded");
    result["successorWorkItemId"] = serde_json::json!(request.successor_work_item_id);
    result["successorBindingMode"] = serde_json::json!("contract_amendment_revalidation");
    Ok(result)
}

/// Record an immutable, repository-bound retry, successor, or supersession decision. The
/// receipt binds the predecessor's exact Contract/Summary/Outcome/Event
/// digests and the Runtime identity. A second decision is appended under a
/// digest-suffixed filename; no predecessor bytes are replaced.
pub fn record_recovery_decision(
    root: &Path,
    work_item_id: &str,
    receipt: &serde_json::Value,
    runtime: &RuntimeContext,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let _lifecycle_lock = acquire_lifecycle_lock(&root, work_item_id)?;
    let contract_path = work_item_artifact_path(&root, work_item_id, "contract.json")?;
    let summary_path = work_item_artifact_path(&root, work_item_id, "summary.json")?;
    let contract = read_contract(&contract_path)?;
    let mut typed: RecoveryDecisionReceipt =
        serde_json::from_value(receipt.clone()).map_err(|error| ObserverError::State {
            path: root.join(".ai/decisions"),
            message: format!("invalid recovery decision receipt: {error}"),
        })?;
    validate_recovery_predecessor_bindings(
        &root,
        work_item_id,
        &typed,
        Some(runtime),
        &contract_path,
        &summary_path,
        None,
    )?;
    let mut legacy_successor_binding = false;
    if matches!(typed.decision.as_str(), "successor" | "supersede") {
        let Some(successor_id) = typed.successor_work_item_id.as_deref() else {
            return Err(ObserverError::State {
                path: root.join(".ai/decisions"),
                message: "successor recovery decision requires successorWorkItemId".into(),
            });
        };
        validate_work_item_id(successor_id)?;
        if successor_id == work_item_id {
            return Err(ObserverError::State {
                path: root.join(".ai/work-items"),
                message: "successor Work Item equals predecessor".into(),
            });
        }
        // A predecessor may have one active successor lineage only.  Older
        // Runtime versions accepted a second `successor` decision for the
        // same predecessor, leaving the first successor archived/pending and
        // making the recovery graph ambiguous.  Read the current append-only
        // chain before creating another successor and fail closed when the
        // requested target differs.  `supersede` remains valid because it
        // closes an already-selected successor lineage without creating a
        // competing Work Item.
        if typed.decision == "successor"
            && let Ok(Some(existing)) = load_recovery_decision(&root, work_item_id, Some(runtime))
            && existing.decision == "successor"
            && existing.successor_work_item_id.as_deref() != Some(successor_id)
        {
            return Err(recovery_decision_error(
                root.join(".ai/decisions"),
                "competing_successor",
                "predecessor already has a different successor; supersede or continue that lineage instead of creating a competing Work Item",
            ));
        }
        if typed.decision == "successor"
            && work_item_artifact_path_optional(&root, successor_id, "contract.json")?.is_some()
        {
            return Err(ObserverError::State {
                path: root.join(".ai/work-items"),
                message: "successor Work Item already exists".into(),
            });
        }
        if typed.decision == "supersede" {
            legacy_successor_binding =
                validate_recovery_successor_binding(&root, work_item_id, &typed)?;
        }
    }
    if legacy_successor_binding {
        typed.successor_binding_mode = Some("legacy_terminal_evidence".into());
    }
    let value = serde_json::to_value(&typed).map_err(|error| ObserverError::State {
        path: root.join(".ai/decisions"),
        message: error.to_string(),
    })?;
    let decisions_dir = root.join(".ai/decisions");
    fs::create_dir_all(&decisions_dir).map_err(|source| ObserverError::Read {
        path: decisions_dir.clone(),
        source,
    })?;
    let canonical_path = decisions_dir.join(format!("{work_item_id}.recovery.json"));
    let path = if fs::symlink_metadata(&canonical_path).is_ok() {
        let digest =
            cockpit_protocol::digest_json(&value).map_err(|error| ObserverError::State {
                path: canonical_path.clone(),
                message: error.to_string(),
            })?;
        let digest = digest.to_string();
        decisions_dir.join(format!(
            "{work_item_id}.recovery.{}.json",
            digest.strip_prefix("sha256:").unwrap_or(&digest)
        ))
    } else {
        canonical_path
    };
    if fs::symlink_metadata(&path).is_ok() {
        let existing = read_json(&path)?;
        if existing == value {
            return Ok(existing);
        }
        return Err(ObserverError::State {
            path,
            message: "recovery decision receipt already exists with different content".into(),
        });
    }
    let retry_summary_backup = if typed.decision == "retry" {
        Some(prepare_retryable_lifecycle(&root, work_item_id)?)
    } else {
        None
    };
    if let Err(error) = atomic_json(&path, &value) {
        if let Some((summary_path, original_summary)) = retry_summary_backup {
            let _ = atomic_json(&summary_path, &original_summary);
        }
        return Err(error);
    }
    if typed.decision == "retry" {
        let retry_digest =
            cockpit_protocol::digest_json(&value).map_err(|error| ObserverError::State {
                path: path.clone(),
                message: error.to_string(),
            })?;
        let mut summary = read_json(&summary_path)?;
        summary["recoveryRetryDecisionPath"] =
            serde_json::json!(repository_relative_path(&root, &path));
        summary["recoveryRetryDecisionDigest"] = serde_json::json!(retry_digest.to_string());
        if let Err(error) = atomic_json(&summary_path, &summary) {
            if let Some((summary_path, original_summary)) = retry_summary_backup {
                let _ = atomic_json(&summary_path, &original_summary);
            }
            let _ = fs::remove_file(&path);
            return Err(error);
        }
    }
    if typed.decision == "successor" {
        let successor_id = typed
            .successor_work_item_id
            .as_deref()
            .expect("validated successor decision");
        let mode = contract.mode.as_deref().unwrap_or("implementation");
        scaffold_work_item_for_recovery(&root, successor_id, mode)?;
        let successor_contract_path = root
            .join(".ai/work-items/active")
            .join(format!("{successor_id}.contract.json"));
        let mut successor_contract = read_json(&successor_contract_path)?;
        successor_contract["predecessorWorkItemId"] = serde_json::json!(work_item_id);
        successor_contract["predecessorContractDigest"] = serde_json::json!(
            typed
                .current_contract_digest
                .as_ref()
                .unwrap_or(&typed.predecessor_contract_digest)
                .to_string()
        );
        successor_contract["recoveryDecisionPath"] =
            serde_json::json!(repository_relative_path(&root, &path));
        atomic_json(&successor_contract_path, &successor_contract)?;
        let successor_summary_path = root
            .join(".ai/work-items/active")
            .join(format!("{successor_id}.summary.json"));
        let mut successor_summary = read_json(&successor_summary_path)?;
        successor_summary["predecessorWorkItemId"] = serde_json::json!(work_item_id);
        successor_summary["predecessorContractDigest"] = serde_json::json!(
            typed
                .current_contract_digest
                .as_ref()
                .unwrap_or(&typed.predecessor_contract_digest)
                .to_string()
        );
        successor_summary["recoveryDecisionPath"] =
            serde_json::json!(repository_relative_path(&root, &path));
        atomic_json(&successor_summary_path, &successor_summary)?;
    }
    Ok(value)
}

/// Restore the only legal retry point after a lifecycle gate has projected a
/// blocked Outcome.  The failed Outcome remains bound by the recovery receipt;
/// a fresh verify/finish cycle will generate the next current projection.
fn prepare_retryable_lifecycle(
    root: &Path,
    work_item_id: &str,
) -> Result<(PathBuf, serde_json::Value), ObserverError> {
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let mut summary = read_json(&summary_path)?;
    let state = summary["state"].as_str().unwrap_or_default();
    if state == "checkpointed" {
        let original = summary.clone();
        // A failed finish may already have appended a before_finish record
        // against the snapshot that was current at that attempt. A retry is
        // the explicit recovery boundary for a fresh verification cycle, so
        // discard that stale terminal candidate and let finish append a new
        // current before_finish record. The predecessor Summary digest in the
        // recovery receipt preserves the old attempt immutably.
        if let Some(entries) = summary
            .get_mut("checkpointEvidence")
            .and_then(serde_json::Value::as_array_mut)
        {
            entries.retain(|entry| {
                entry.get("stage").and_then(serde_json::Value::as_str) != Some("before_finish")
            });
        }
        // A failed finish can leave its lifecycle marker on a Summary that
        // is still checkpointed (for example when a required finalization
        // plan was missing).  Retry recovery is the explicit correction
        // boundary; clear the stale marker so CI and later lifecycle checks
        // do not reject the freshly recoverable state.
        if let Some(object) = summary.as_object_mut() {
            object.remove("failedGate");
            object.remove("recoveryCondition");
            object.remove("outcomeState");
        }
        summary["recoveryRetryPending"] = serde_json::json!(true);
        atomic_json(&summary_path, &summary)?;
        return Ok((summary_path, original));
    }
    if state != "finish_ready" {
        return Err(ObserverError::State {
            path: summary_path,
            message: format!("retry recovery requires finish_ready state, got {state}"),
        });
    }
    let preflight_state = summary["preflightState"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    // Any failed finish projection is an explicit recovery boundary.  A
    // finish.governance failure can leave the persisted preflight red (for
    // example when controls were missing), so requiring green/yellow here
    // would strand an otherwise recoverable Work Item.  Other lifecycle
    // failures remain bounded by the same one-checkpoint requirement.
    let lifecycle_retry = summary["failedGate"]
        .as_str()
        .is_some_and(|gate| gate.starts_with("finish."));
    if summary["checkpointCount"] != serde_json::json!(1)
        || (!matches!(preflight_state.as_str(), "green" | "yellow") && !lifecycle_retry)
    {
        return Err(ObserverError::State {
            path: summary_path,
            message: "retry recovery requires one checkpoint and either a non-red preflight result or a failed finish.lifecycle transition".into(),
        });
    }
    let original = summary.clone();
    if let Some(entries) = summary
        .get_mut("checkpointEvidence")
        .and_then(serde_json::Value::as_array_mut)
    {
        entries.retain(|entry| {
            entry.get("stage").and_then(serde_json::Value::as_str) != Some("before_finish")
        });
    }
    summary["state"] = serde_json::json!("checkpointed");
    summary["updatedAt"] = serde_json::json!(now());
    if let Some(object) = summary.as_object_mut() {
        object.remove("failedGate");
        object.remove("recoveryCondition");
        object.remove("outcomeState");
    }
    summary["recoveryRetryPending"] = serde_json::json!(true);
    atomic_json(&summary_path, &summary)?;
    Ok((summary_path, original))
}

pub(super) fn work_item_artifact_path(
    root: &Path,
    work_item_id: &str,
    suffix: &str,
) -> Result<PathBuf, ObserverError> {
    work_item_artifact_path_optional(root, work_item_id, suffix)?.ok_or_else(|| {
        ObserverError::State {
            path: root.join(".ai/work-items"),
            message: format!("Work Item artifact not found: {work_item_id}.{suffix}"),
        }
    })
}

fn work_item_artifact_path_optional(
    root: &Path,
    work_item_id: &str,
    suffix: &str,
) -> Result<Option<PathBuf>, ObserverError> {
    for phase in ["active", "archive"] {
        let path = root
            .join(".ai/work-items")
            .join(phase)
            .join(format!("{work_item_id}.{suffix}"));
        if fs::symlink_metadata(&path).is_ok() {
            if !is_regular_non_symlink(&path)? {
                return Err(ObserverError::State {
                    path,
                    message: "Work Item artifact must be a regular non-symlink file".into(),
                });
            }
            return Ok(Some(path));
        }
    }
    Ok(None)
}

pub fn record_verification_with_snapshot(
    root: &Path,
    work_item_id: &str,
    receipt: &serde_json::Value,
    runtime_version: &str,
    runtime_digest: &Digest,
    snapshot: &RepositorySnapshot,
) -> Result<serde_json::Value, ObserverError> {
    record_verification_internal(
        root,
        work_item_id,
        receipt,
        runtime_version,
        runtime_digest,
        snapshot,
        None,
    )
}

/// Record verification evidence while binding it to the Runtime that is
/// executing the request.  The legacy `*_with_snapshot` API remains available
/// for embedders that intentionally own their Runtime identity; all CLI/MCP
/// paths use this function so a foreign but well-formed digest cannot pass a
/// current lifecycle operation.
pub fn record_verification_with_runtime(
    root: &Path,
    work_item_id: &str,
    receipt: &serde_json::Value,
    runtime: &RuntimeContext,
    snapshot: &RepositorySnapshot,
) -> Result<serde_json::Value, ObserverError> {
    record_verification_internal(
        root,
        work_item_id,
        receipt,
        &runtime.runtime_version,
        &runtime.runtime_digest,
        snapshot,
        Some(runtime),
    )
}

fn record_verification_internal(
    root: &Path,
    work_item_id: &str,
    receipt: &serde_json::Value,
    runtime_version: &str,
    runtime_digest: &Digest,
    snapshot: &RepositorySnapshot,
    current_runtime: Option<&RuntimeContext>,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    if fs::canonicalize(&snapshot.root).ok().as_ref() != Some(&root) {
        return Err(ObserverError::SnapshotRootMismatch);
    }
    let active_contract = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    if !active_contract.is_file() {
        return Err(ObserverError::State {
            path: active_contract,
            message: "verification evidence requires an active work item contract".into(),
        });
    }
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let summary: serde_json::Value = read_json(&summary_path)?;
    let evidence_path = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.verification.json"));
    let prior_evidence_present = fs::symlink_metadata(&evidence_path).is_ok();
    if !matches!(
        summary["state"].as_str(),
        Some("checkpointed" | "finish_ready")
    ) || summary["checkpointCount"] != serde_json::json!(1)
    {
        return Err(ObserverError::State {
            path: summary_path.clone(),
            message: "verification requires exactly one completed checkpoint and an active lifecycle state".into(),
        });
    }
    let recovery_retry_pending = summary["recoveryRetryPending"] == serde_json::json!(true);
    let contract_amendment_pending = summary
        .get("verificationInvalidatedByContractAmendment")
        .is_some();
    require_current_retry_recovery_binding(&root, work_item_id, &summary, current_runtime)?;
    if !matches!(summary["preflightState"].as_str(), Some("green" | "yellow"))
        && !recovery_retry_pending
        && !contract_amendment_pending
    {
        return Err(ObserverError::State {
            path: summary_path,
            message: "verification requires a recorded non-red preflight result unless an explicit recovery retry is pending".into(),
        });
    }
    if receipt["passed"] != serde_json::Value::Bool(true) {
        return Err(ObserverError::State {
            path: root.join(".ai/evidence"),
            message: "failed verification cannot be recorded as completion evidence".into(),
        });
    }
    if receipt
        .get("workItemId")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|receipt_id| receipt_id != work_item_id)
    {
        return Err(ObserverError::State {
            path: root.join(".ai/evidence"),
            message: "verification receipt belongs to another work item".into(),
        });
    }
    if let Some(runtime) = current_runtime
        && (runtime.runtime_version != runtime_version || runtime.runtime_digest != *runtime_digest)
    {
        return Err(ObserverError::State {
            path: root.join(".ai/evidence"),
            message:
                "verification receipt Runtime identity arguments do not match the current Runtime"
                    .into(),
        });
    }
    let expected_repository_id = repository_id(&root).to_string();
    let typed_receipt = bind_typed_verification_receipt(
        receipt,
        work_item_id,
        &expected_repository_id,
        runtime_version,
        runtime_digest,
    )?;
    if current_runtime.is_some() && typed_receipt.is_none() {
        return Err(ObserverError::State {
            path: root.join(".ai/evidence"),
            message: "current Runtime requires a strict typed verification receipt".into(),
        });
    }
    let current_contract_digest = contract_digest(&active_contract)?;
    let retention_policy = read_evidence_retention_policy(&root, work_item_id)?;
    let (stored_receipt, capture_mode) = match retention_policy
        .as_ref()
        .map(|policy| &policy.retention.persistence)
    {
        Some(EvidencePersistence::NoPersistence) => {
            return Err(ObserverError::State {
                path: root
                    .join(".ai/evidence")
                    .join(format!("{work_item_id}.retention.json")),
                message:
                    "no_persistence cannot produce completion evidence; use an external evidence owner or change the policy".into(),
            });
        }
        Some(EvidencePersistence::DigestOnly) => (None, VerificationCaptureMode::DigestOnly),
        Some(EvidencePersistence::RedactedCapture) => (
            Some(redact_verification_receipt(
                typed_receipt.as_ref().unwrap_or(receipt),
            )),
            VerificationCaptureMode::RedactedCapture,
        ),
        Some(EvidencePersistence::FullCapture) | None => (
            Some(typed_receipt.clone().unwrap_or_else(|| receipt.clone())),
            if typed_receipt.is_some() {
                VerificationCaptureMode::FullCapture
            } else {
                VerificationCaptureMode::LegacyUntyped
            },
        ),
    };
    let receipt_digest = cockpit_protocol::digest_json(stored_receipt.as_ref().unwrap_or(receipt))
        .map_err(|error| ObserverError::State {
            path: root.join(".ai/evidence"),
            message: error.to_string(),
        })?;
    let mut evidence = serde_json::json!({
        "protocolVersion": 1,
        "evidenceSchemaVersion": 2,
        "workItemId": work_item_id,
        "repositoryId": expected_repository_id,
        "runtimeVersion": runtime_version,
        "runtimeDigest": runtime_digest,
        "contractDigest": current_contract_digest,
        "repositorySnapshotDigest": snapshot_digest(snapshot)?,
        "passed": true,
        "receiptDigest": receipt_digest,
        "captureMode": serde_json::to_value(capture_mode).expect("capture mode serializes"),
        "createdAt": now(),
    });
    if let Some(receipt) = stored_receipt {
        evidence["receipt"] = receipt;
    }
    if let Some(policy) = retention_policy {
        evidence["retention"] =
            serde_json::to_value(policy).map_err(|error| ObserverError::State {
                path: root.join(".ai/evidence"),
                message: error.to_string(),
            })?;
    }
    let path = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.verification.json"));
    atomic_json(&path, &evidence)?;

    // Verification can satisfy a Contract's required evidence.  Refresh the
    // recorded governance result against the same non-.ai snapshot so the
    // canonical lifecycle remains start -> preflight (possibly yellow) ->
    // checkpoint -> verify -> finish, without requiring an otherwise
    // redundant second CLI preflight invocation.
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&contract_path)?;
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let refreshed_snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let raw_decision = governance_decision_for_pre_execution_boundary(
        &root,
        &contract,
        &refreshed_snapshot,
        current_runtime,
        None,
    )?;
    let decision = apply_preflight_review_evidence(
        &root,
        &contract,
        &refreshed_snapshot,
        raw_decision.clone(),
        false,
        None,
    )?;
    let reconcile_blocked_outcome = recovery_retry_pending
        || contract_amendment_pending
        || (!prior_evidence_present && decision.state != DecisionState::Red);
    let decision_value =
        serde_json::to_value(&raw_decision).map_err(|error| ObserverError::State {
            path: contract_path.clone(),
            message: error.to_string(),
        })?;
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let mut summary: serde_json::Value = read_json(&summary_path)?;
    // Preserve the snapshot that actually drove verification in the canonical
    // Summary.  The old path only refreshed governance projections below;
    // consequently a Work Item started from a clean tree reported an empty
    // changedPaths list even after source edits and produced a misleading
    // delivery report.  Do not use `refreshed_snapshot` here: it includes the
    // evidence/projection writes made by this function rather than just the
    // verification input.
    summary["changedPaths"] = serde_json::json!(snapshot.changed_paths);
    summary
        .as_object_mut()
        .expect("Work Item Summary is an object")
        .remove("verificationInvalidatedByContractAmendment");
    let verification_entries = summary
        .as_object_mut()
        .expect("Work Item Summary is an object")
        .entry("verification")
        .or_insert_with(|| serde_json::json!([]));
    let verification_entries =
        verification_entries
            .as_array_mut()
            .ok_or_else(|| ObserverError::State {
                path: summary_path.clone(),
                message: "Summary.verification must be an array".into(),
            })?;
    if let Some(results) = receipt.get("results").and_then(serde_json::Value::as_array) {
        for result in results {
            let Some(node_id) = result.get("nodeId").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let value = serde_json::json!({
                "check": node_id,
                "result": if result.get("passed") == Some(&serde_json::Value::Bool(true)) {
                    "passed"
                } else {
                    "failed"
                },
            });
            verification_entries.retain(|item| {
                item.get("check").and_then(serde_json::Value::as_str) != Some(node_id)
            });
            verification_entries.push(value);
        }
    }
    summary["preflightState"] = decision_state_name(decision.state.clone()).into();
    summary["preflightDecisionDigest"] = cockpit_protocol::digest_json(&decision_value)
        .map_err(|error| ObserverError::State {
            path: contract_path.clone(),
            message: error.to_string(),
        })?
        .to_string()
        .into();
    summary["preflightRepositorySnapshotDigest"] =
        snapshot_digest(&refreshed_snapshot)?.to_string().into();
    summary["preflightContractDigest"] = contract_digest(&contract_path)?.to_string().into();
    // A fresh verification after Contract amendment revalidates the existing
    // checkpoint against the new Contract and snapshot. The immutable
    // before_edit evidence remains in the append-only chain; only this
    // current lifecycle binding advances.
    // A retry recovery marker authorizes exactly one replacement verification.
    // Once that verification and its refreshed governance projection are
    // persisted, consume only the marker projection; the append-only recovery
    // receipt remains immutable history. Leaving the marker set would make a
    // subsequent preflight demand a receipt bound to the already-advanced
    // Summary and strand an otherwise valid retry.
    if reconcile_blocked_outcome {
        let summary_object = summary
            .as_object_mut()
            .expect("Work Item Summary is an object");
        summary_object.remove("recoveryRetryPending");
        summary_object.remove("recoveryRetryDecisionPath");
        summary_object.remove("recoveryRetryDecisionDigest");
        // Bind the replacement verification to the explicit retry boundary.
        // outcome_v2 revalidates this digest against the current evidence and
        // full identity before superseding a blocked Outcome projection.
        summary["verificationRecoveryReconciled"] = cockpit_protocol::digest_json(&evidence)
            .map_err(|error| ObserverError::State {
                path: summary_path.clone(),
                message: error.to_string(),
            })?
            .to_string()
            .into();
    }
    summary["checkpointContractDigest"] = contract_digest(&contract_path)?.to_string().into();
    summary["checkpointRepositorySnapshotDigest"] =
        snapshot_digest(&refreshed_snapshot)?.to_string().into();
    summary["preflightAt"] = now().into();
    atomic_json(&summary_path, &summary)?;
    if summary["state"] == serde_json::json!("finish_ready") {
        refresh_active_outcome_verification_binding(
            &root,
            work_item_id,
            &evidence,
            &snapshot_digest(&refreshed_snapshot)?,
        )?;
    } else if reconcile_blocked_outcome {
        refresh_active_outcome_after_recovery_verification(
            &root,
            work_item_id,
            current_runtime,
            &refreshed_snapshot,
            &snapshot_digest(&refreshed_snapshot)?,
        )?;
    }
    Ok(evidence)
}

/// Refresh the active human Outcome after a verification retry that occurs
/// while the Work Item is already finish-ready.  A hosted PR gate may observe
/// the normal governance-only commits made after the first verification; the
/// retry must update every active Outcome/report binding instead of leaving an
/// old evidence digest that would make archive/close fail later.
fn refresh_active_outcome_verification_binding(
    root: &Path,
    work_item_id: &str,
    evidence: &serde_json::Value,
    current_snapshot_digest: &Digest,
) -> Result<(), ObserverError> {
    let active = root.join(".ai/work-items/active");
    let outcome_path = active.join(format!("{work_item_id}.outcome.json"));
    if !is_regular_non_symlink(&outcome_path)? {
        return Err(ObserverError::State {
            path: outcome_path,
            message: "finish-ready verification retry requires a regular Outcome".into(),
        });
    }
    let mut outcome = read_json(&outcome_path)?;
    if outcome["state"] != serde_json::json!("finish_ready")
        || outcome["verification"]["status"] != serde_json::json!("verified")
    {
        return Err(ObserverError::State {
            path: outcome_path,
            message: "finish-ready verification retry requires a verified Outcome".into(),
        });
    }
    outcome["evidenceDigest"] = cockpit_protocol::digest_json(evidence)
        .map_err(|error| ObserverError::State {
            path: root.join(".ai/evidence"),
            message: error.to_string(),
        })?
        .to_string()
        .into();

    let report_path = active.join(format!("{work_item_id}.task-report.json"));
    if is_regular_non_symlink(&report_path)? {
        let mut report: TaskOutcomeReport = serde_json::from_value(read_json(&report_path)?)
            .map_err(|error| ObserverError::State {
                path: report_path.clone(),
                message: format!("invalid active Task Outcome report: {error}"),
            })?;
        if report.work_item_id != work_item_id {
            return Err(ObserverError::State {
                path: report_path,
                message: "active Task Outcome report belongs to another Work Item".into(),
            });
        }
        report.bindings.repository_snapshot_digest = Some(current_snapshot_digest.clone());
        let report_value = serde_json::to_value(&report).map_err(|error| ObserverError::State {
            path: report_path.clone(),
            message: error.to_string(),
        })?;
        let report_bytes =
            serde_json::to_vec_pretty(&report_value).map_err(|error| ObserverError::State {
                path: report_path.clone(),
                message: error.to_string(),
            })?;
        atomic_write(&report_path, &report_bytes)?;
        let markdown_path = active.join(format!("{work_item_id}.task-report.md"));
        if is_regular_non_symlink(&markdown_path)? {
            atomic_write(&markdown_path, task_outcome_markdown(&report).as_bytes())?;
            outcome["taskReportMarkdownDigest"] =
                Digest::sha256_bytes(task_outcome_markdown(&report).as_bytes())
                    .to_string()
                    .into();
        }
        outcome["taskOutcomeReport"] = report_value;
        outcome["taskReportDigest"] = Digest::sha256_bytes(&report_bytes).to_string().into();
    }
    atomic_json(&outcome_path, &outcome)
}

/// Replace a blocked active Outcome after the explicitly authorized retry has
/// produced a fresh, identity-bound verification.  Failure events remain
/// append-only; this file is only the current projection consumed by
/// `finish`/`archive` and must no longer strand the repaired lifecycle.
fn refresh_active_outcome_after_recovery_verification(
    root: &Path,
    work_item_id: &str,
    current_runtime: Option<&RuntimeContext>,
    snapshot: &RepositorySnapshot,
    snapshot_digest: &Digest,
) -> Result<(), ObserverError> {
    let active = root.join(".ai/work-items/active");
    let outcome_path = active.join(format!("{work_item_id}.outcome.json"));
    if fs::symlink_metadata(&outcome_path).is_err() {
        return Ok(());
    }
    if !is_regular_non_symlink(&outcome_path)? {
        return Err(ObserverError::State {
            path: outcome_path,
            message: "recovery verification requires a regular active Outcome".into(),
        });
    }
    let evidence_path = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.verification.json"));
    let evidence = read_json(&evidence_path)?;
    let outcome = outcome_v2_internal_with_snapshot(
        root,
        work_item_id,
        current_runtime,
        Some((snapshot, snapshot_digest)),
    )?;
    if outcome.state != OutcomeState::Verified
        || outcome.decision_state != Some(DecisionState::Green)
    {
        return Err(ObserverError::State {
            path: evidence_path,
            message: "fresh recovery verification did not produce a green Outcome projection"
                .into(),
        });
    }
    let task_report = outcome
        .task_outcome_report
        .as_ref()
        .ok_or_else(|| ObserverError::State {
            path: outcome_path.clone(),
            message: "green recovery Outcome is missing its task report".into(),
        })?;
    let (task_report_digest, task_report_markdown_digest) =
        write_task_outcome_artifacts(root, work_item_id, task_report, true)?;
    let mut value = serde_json::to_value(outcome).map_err(|error| ObserverError::State {
        path: outcome_path.clone(),
        message: error.to_string(),
    })?;
    value["protocolVersion"] = serde_json::json!(1);
    value["workItemId"] = serde_json::json!(work_item_id);
    value["state"] = serde_json::json!("checkpointed");
    value["verification"] = serde_json::json!({
        "status": "verified",
        "required": true,
        "evidencePath": format!(".ai/evidence/{work_item_id}.verification.json"),
    });
    value["evidenceDigest"] = cockpit_protocol::digest_json(&evidence)
        .map_err(|error| ObserverError::State {
            path: evidence_path,
            message: error.to_string(),
        })?
        .to_string()
        .into();
    value["taskReportDigest"] = task_report_digest.to_string().into();
    value["taskReportMarkdownDigest"] = task_report_markdown_digest.to_string().into();
    atomic_json(&outcome_path, &value)
}

/// Bind a raw execution result to its Work Item/repository/Runtime identity
/// and deserialize it through the strict wire type.  The CLI's raw result has
/// runtime fields at the envelope level; they are removed before adding the
/// required nested identity fields to the persisted receipt.
fn bind_typed_verification_receipt(
    receipt: &serde_json::Value,
    work_item_id: &str,
    repository_id: &str,
    runtime_version: &str,
    runtime_digest: &Digest,
) -> Result<Option<serde_json::Value>, ObserverError> {
    let Some(object) = receipt.as_object() else {
        return Ok(None);
    };
    if object.get("passed") != Some(&serde_json::Value::Bool(true)) {
        return Err(ObserverError::State {
            path: PathBuf::from(".ai/evidence"),
            message: "failed verification cannot be recorded as completion evidence".into(),
        });
    }
    let mut bound = receipt.clone();
    let Some(bound_object) = bound.as_object_mut() else {
        return Ok(None);
    };
    bound_object.remove("runtimeVersion");
    bound_object.remove("runtimeDigest");
    for (key, expected) in [
        ("workItemId", serde_json::Value::String(work_item_id.into())),
        (
            "repositoryId",
            serde_json::Value::String(repository_id.into()),
        ),
        (
            "runtimeVersion",
            serde_json::Value::String(runtime_version.into()),
        ),
        (
            "runtimeDigest",
            serde_json::Value::String(runtime_digest.to_string()),
        ),
    ] {
        if let Some(existing) = bound_object.get(key)
            && existing != &expected
        {
            return Err(ObserverError::State {
                path: PathBuf::from(".ai/evidence"),
                message: format!("verification receipt {key} does not match its binding"),
            });
        }
        bound_object.insert(key.into(), expected);
    }
    let typed: cockpit_verification::VerificationReceipt =
        match serde_json::from_value(bound.clone()) {
            Ok(value) => value,
            Err(_) => return Ok(None),
        };
    if !typed.passed
        || typed.work_item_id.as_deref() != Some(work_item_id)
        || typed.repository_id.as_deref() != Some(repository_id)
        || typed.runtime_version.as_deref() != Some(runtime_version)
        || typed.runtime_digest.as_deref() != Some(runtime_digest.to_string().as_str())
    {
        return Err(ObserverError::State {
            path: PathBuf::from(".ai/evidence"),
            message: "typed verification receipt has missing or mismatched identity".into(),
        });
    }
    Ok(Some(bound))
}

fn redact_verification_receipt(receipt: &serde_json::Value) -> serde_json::Value {
    match receipt {
        serde_json::Value::Object(map) => {
            let mut redacted = serde_json::Map::new();
            for (key, value) in map {
                let lowered = key.to_ascii_lowercase();
                if matches!(lowered.as_str(), "output" | "stdout" | "stderr" | "command")
                    || lowered.contains("log")
                {
                    redacted.insert(key.clone(), serde_json::Value::String("[redacted]".into()));
                } else {
                    redacted.insert(key.clone(), redact_verification_receipt(value));
                }
            }
            serde_json::Value::Object(redacted)
        }
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.iter().map(redact_verification_receipt).collect())
        }
        other => other.clone(),
    }
}

use cockpit_core::{Digest, EvidenceState};
use cockpit_git::RepositorySnapshot;
use cockpit_protocol::{QualityCommand, RepositoryConfig, validate_protocol_version};
use serde::{Deserialize, Serialize};
use sha2::{Digest as ShaDigest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LanguageSignal {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildSystem {
    Cargo,
    Npm,
    Poetry,
    Go,
    Make,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryObservation {
    pub snapshot_digest: Digest,
    pub dependency_fingerprint: Digest,
    pub languages: Vec<LanguageSignal>,
    pub build_systems: Vec<BuildSystem>,
    pub test_roots: Vec<String>,
    pub quality_commands: Vec<QualityCommand>,
    pub ci_surfaces: Vec<String>,
    pub critical_domains: Vec<String>,
    pub files_read: usize,
    #[serde(default)]
    pub cache_hit: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvolutionClass {
    L0,
    L1,
    L2,
    L3,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvolutionEvent {
    pub class: EvolutionClass,
    pub event_type: String,
    pub path: String,
    pub action: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileUpdateProposal {
    pub from_profile_version: u64,
    pub candidate: String,
    pub reason: String,
    pub requires_human_confirmation: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachedProfile {
    pub profile_version: u64,
    pub repository_id: String,
    pub state: String,
    #[serde(default)]
    pub profile_digest: Option<Digest>,
    pub tests: Vec<QualityCommand>,
    pub build_systems: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryStatus {
    pub protocol_version: u32,
    pub repository_id: String,
    pub state: String,
    pub profile_version: u64,
    pub active_work_items: usize,
    pub archived_work_items: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleReceipt {
    pub work_item_id: String,
    pub state: String,
    pub timestamp: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkItemStartOptions {
    pub out_of_scope: Vec<String>,
    pub risk: String,
    pub authority: String,
    pub acceptance_criteria: Vec<String>,
    pub required_evidence_classes: Vec<String>,
}

#[derive(Debug, Error)]
pub enum ObserverError {
    #[error("failed to read repository entry {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("repository snapshot root does not match observer root")]
    SnapshotRootMismatch,
    #[error("repository protocol state error at {path}: {message}")]
    State { path: PathBuf, message: String },
}

pub fn repository_id(root: &Path) -> Digest {
    Digest::sha256_bytes(root.to_string_lossy().as_bytes())
}

pub fn attach(root: &Path) -> Result<AttachedProfile, ObserverError> {
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
    let observation = observe(&root, &snapshot)?;
    let ai = root.join(".ai");
    for directory in [
        ai.join("work-items/active"),
        ai.join("work-items/archive"),
        ai.join("decisions"),
        ai.join("evidence"),
        ai.join("knowledge"),
    ] {
        fs::create_dir_all(&directory).map_err(|source| ObserverError::Read {
            path: directory,
            source,
        })?;
    }
    let id = repository_id(&root).to_string();
    let config = format!("protocol_version = 1\nrepository_id = \"{id}\"\n");
    atomic_write(&ai.join("cockpit.toml"), config.as_bytes())?;
    let profile_digest = cockpit_protocol::digest_json(&cockpit_protocol::ProjectProfile {
        profile_version: 1,
        repository_id: id.clone(),
        tests: observation.quality_commands.clone(),
        build_systems: observation
            .build_systems
            .iter()
            .map(|value| format!("{value:?}"))
            .collect(),
    })
    .map_err(|error| ObserverError::State {
        path: ai.join("project.json"),
        message: error.to_string(),
    })?;
    let profile = AttachedProfile {
        profile_version: 1,
        repository_id: id,
        state: "calibration_required".into(),
        profile_digest: Some(profile_digest.clone()),
        tests: observation.quality_commands,
        build_systems: observation
            .build_systems
            .iter()
            .map(|value| format!("{value:?}"))
            .collect(),
    };
    let encoded = serde_json::to_vec_pretty(&profile).map_err(|error| ObserverError::State {
        path: ai.join("project.json"),
        message: error.to_string(),
    })?;
    atomic_write(&ai.join("project.json"), &encoded)?;
    let proposal = serde_json::json!({
        "kind": "project_profile_initialization",
        "profileVersion": 1,
        "profileDigest": profile_digest,
        "state": "calibration_required",
    });
    atomic_json(&ai.join("decisions/profile-v1.json"), &proposal)?;
    Ok(profile)
}

pub fn status(root: &Path) -> Result<RepositoryStatus, ObserverError> {
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
        || config.repository_id != repository_id(&root).to_string()
    {
        return Err(ObserverError::State {
            path: config_path,
            message: "repository identity does not match protocol state".into(),
        });
    }
    Ok(RepositoryStatus {
        protocol_version: config.protocol_version,
        repository_id: profile.repository_id,
        state: profile.state,
        profile_version: profile.profile_version,
        active_work_items: count_suffix(&ai.join("work-items/active"), ".contract.json"),
        archived_work_items: count_suffix(&ai.join("work-items/archive"), ".archive.json"),
    })
}

fn count_suffix(path: &Path, suffix: &str) -> usize {
    fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(suffix))
        .count()
}

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
    validate_work_item_id(work_item_id)?;
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
    let profile_digest =
        Digest::sha256_bytes(&fs::read(ai.join("project.json")).map_err(|source| {
            ObserverError::Read {
                path: ai.join("project.json"),
                source,
            }
        })?);
    let repository_snapshot_digest = snapshot_digest(&snapshot)?;
    let now = now();
    let contract = serde_json::json!({
        "protocolVersion": 1,
        "repositoryId": repository_id(&root),
        "workItemId": work_item_id,
        "intent": intent,
        "goal": goal,
        "scope": scope,
        "outOfScope": options.out_of_scope.clone(),
        "risk": options.risk.clone(),
        "authority": options.authority.clone(),
        "acceptanceCriteria": options.acceptance_criteria.clone(),
        "requiredEvidenceClasses": options.required_evidence_classes.clone(),
        "baseRevision": snapshot.head.unwrap_or_else(|| "unborn".into()),
        "projectProfileDigest": profile_digest,
        "repositorySnapshotDigest": repository_snapshot_digest,
        "createdAt": now,
    });
    let summary = serde_json::json!({
        "protocolVersion": 1,
        "repositoryId": repository_id(&root),
        "workItemId": work_item_id,
        "state": "implementation_active",
        "changedPaths": snapshot.changed_paths,
        "checkpointCount": 0,
        "createdAt": now,
        "updatedAt": now,
    });
    let active = ai.join("work-items/active");
    let archive = ai.join("work-items/archive");
    if [
        active.join(format!("{work_item_id}.contract.json")),
        active.join(format!("{work_item_id}.summary.json")),
        archive.join(format!("{work_item_id}.archive.json")),
    ]
    .iter()
    .any(|path| path.exists())
    {
        return Err(ObserverError::State {
            path: active.join(format!("{work_item_id}.contract.json")),
            message: "work item already exists".into(),
        });
    }
    atomic_json(
        &active.join(format!("{work_item_id}.contract.json")),
        &contract,
    )?;
    atomic_json(
        &active.join(format!("{work_item_id}.summary.json")),
        &summary,
    )?;
    Ok(LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "implementation_active".into(),
        timestamp: now,
    })
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
    let count = summary["checkpointCount"].as_u64().unwrap_or(0) + 1;
    let timestamp = now();
    summary["checkpointCount"] = count.into();
    summary["updatedAt"] = timestamp.clone().into();
    atomic_json(&path, &summary)?;
    Ok(LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "checkpointed".into(),
        timestamp,
    })
}

pub fn finish_work_item(
    root: &Path,
    work_item_id: &str,
) -> Result<LifecycleReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let active = root.join(".ai/work-items/active");
    let summary_path = active.join(format!("{work_item_id}.summary.json"));
    let mut summary: serde_json::Value = read_json(&summary_path)?;
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
    if evidence["repositorySnapshotDigest"] != serde_json::Value::String(current_digest.to_string())
    {
        return Err(ObserverError::State {
            path: evidence_path,
            message: "verification receipt is stale for the current repository snapshot".into(),
        });
    }
    let timestamp = now();
    summary["state"] = "finish_ready".into();
    summary["updatedAt"] = timestamp.clone().into();
    atomic_json(&summary_path, &summary)?;
    let outcome = serde_json::json!({
        "protocolVersion": 1,
        "workItemId": work_item_id,
        "state": "finish_ready",
        "verification": {"status": "verified", "required": true, "evidencePath": format!(".ai/evidence/{work_item_id}.verification.json")},
        "evidenceDigest": cockpit_protocol::digest_json(&evidence).map_err(|error| ObserverError::State { path: root.join(".ai/evidence"), message: error.to_string() })?,
        "createdAt": timestamp,
    });
    if let Err(error) = atomic_json(
        &active.join(format!("{work_item_id}.outcome.json")),
        &outcome,
    ) {
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

pub fn record_verification_with_snapshot(
    root: &Path,
    work_item_id: &str,
    receipt: &serde_json::Value,
    runtime_version: &str,
    runtime_digest: &Digest,
    snapshot: &RepositorySnapshot,
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
    let evidence = serde_json::json!({
        "protocolVersion": 1,
        "workItemId": work_item_id,
        "runtimeVersion": runtime_version,
        "runtimeDigest": runtime_digest,
        "repositorySnapshotDigest": snapshot_digest(snapshot)?,
        "passed": true,
        "receipt": receipt,
        "createdAt": now(),
    });
    let path = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.verification.json"));
    atomic_json(&path, &evidence)?;
    Ok(evidence)
}

pub fn snapshot_digest(snapshot: &RepositorySnapshot) -> Result<Digest, ObserverError> {
    let mut stable = snapshot.clone();
    stable
        .changed_paths
        .retain(|path| !path.starts_with(".ai/"));
    cockpit_protocol::digest_json(&stable).map_err(|error| ObserverError::State {
        path: snapshot.root.join(".ai"),
        message: error.to_string(),
    })
}

pub fn contract_freshness_findings(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
) -> Result<Vec<String>, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let ai_path = root.join(".ai");
    let config_path = ai_path.join("cockpit.toml");
    let profile_path = ai_path.join("project.json");
    if !ai_path.is_dir() {
        return Ok(Vec::new());
    }
    let mut findings = Vec::new();
    if !config_path.is_file() || !profile_path.is_file() {
        findings.push("stale_contract".into());
        return Ok(findings);
    }
    if contract.repository_id != repository_id(&root).to_string() {
        findings.push("stale_contract".into());
    }
    if contract.base_revision != "unborn"
        && snapshot.head.as_deref() != Some(contract.base_revision.as_str())
    {
        findings.push("stale_contract".into());
    }
    let profile_digest =
        Digest::sha256_bytes(
            &fs::read(&profile_path).map_err(|source| ObserverError::Read {
                path: profile_path.clone(),
                source,
            })?,
        );
    if contract.project_profile_digest != profile_digest {
        findings.push("stale_contract".into());
    }
    if contract.repository_snapshot_digest != snapshot_digest(snapshot)? {
        findings.push("stale_contract".into());
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}

pub fn evidence_state_for_contract(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
) -> Result<EvidenceState, ObserverError> {
    if contract.required_evidence_classes.is_empty() {
        return Ok(EvidenceState::Complete);
    }
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let evidence_path = root
        .join(".ai/evidence")
        .join(format!("{}.verification.json", contract.work_item_id));
    let evidence = match read_json(&evidence_path) {
        Ok(value) => value,
        Err(ObserverError::Read { source, .. })
            if source.kind() == std::io::ErrorKind::NotFound =>
        {
            return Ok(EvidenceState::Missing);
        }
        Err(error) => return Err(error),
    };
    if evidence["workItemId"] != serde_json::Value::String(contract.work_item_id.clone())
        || evidence["passed"] != serde_json::Value::Bool(true)
    {
        return Ok(EvidenceState::Contradictory);
    }
    if evidence["repositorySnapshotDigest"]
        != serde_json::Value::String(snapshot_digest(snapshot)?.to_string())
    {
        return Ok(EvidenceState::Stale);
    }
    if contract.required_evidence_classes.iter().any(|class| {
        !matches!(
            class.to_ascii_lowercase().as_str(),
            "verification" | "verification_receipt" | "verification-receipt"
        )
    }) {
        return Ok(EvidenceState::Missing);
    }
    Ok(EvidenceState::Complete)
}

pub fn archive_work_item(
    root: &Path,
    work_item_id: &str,
) -> Result<LifecycleReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let ai = root.join(".ai");
    let active = ai.join("work-items/active");
    let archive = ai.join("work-items/archive");
    fs::create_dir_all(&archive).map_err(|source| ObserverError::Read {
        path: archive.clone(),
        source,
    })?;
    let manifest_path = archive.join(format!("{work_item_id}.archive.json"));
    if manifest_path.exists() {
        return Err(ObserverError::State {
            path: manifest_path,
            message: "archive manifest already exists".into(),
        });
    }
    let names = ["contract", "summary", "outcome"];
    let mut files = serde_json::Map::new();
    let mut pending = Vec::new();
    for name in names {
        let source_path = active.join(format!("{work_item_id}.{name}.json"));
        let target = archive.join(format!("{work_item_id}.{name}.json"));
        let bytes = fs::read(&source_path).map_err(|error| ObserverError::Read {
            path: source_path.clone(),
            source: error,
        })?;
        files.insert(
            format!("{name}Path"),
            serde_json::Value::String(format!(".ai/work-items/archive/{work_item_id}.{name}.json")),
        );
        files.insert(
            format!("{name}Digest"),
            serde_json::Value::String(Digest::sha256_bytes(&bytes).to_string()),
        );
        if target.exists() {
            return Err(ObserverError::State {
                path: target,
                message: "archive target already exists".into(),
            });
        }
        pending.push((source_path, target));
    }
    let mut moved = Vec::new();
    for (source, target) in &pending {
        if let Err(source_error) = fs::rename(source, target) {
            for (moved_source, moved_target) in moved.into_iter().rev() {
                let _ = fs::rename(moved_target, moved_source);
            }
            return Err(ObserverError::Read {
                path: target.clone(),
                source: source_error,
            });
        }
        moved.push((source.clone(), target.clone()));
    }
    let timestamp = now();
    let manifest = serde_json::json!({
        "protocolVersion": 1,
        "workItemId": work_item_id,
        "state": "archived",
        "files": files,
        "createdAt": timestamp,
    });
    if let Err(error) = atomic_json(&manifest_path, &manifest) {
        for (moved_source, moved_target) in moved.into_iter().rev() {
            let _ = fs::rename(moved_target, moved_source);
        }
        return Err(error);
    }
    let _ = fs::remove_file(ai.join("knowledge/index.json"));
    Ok(LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "archived".into(),
        timestamp,
    })
}

pub fn close_work_item(root: &Path, work_item_id: &str) -> Result<LifecycleReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    Err(ObserverError::State {
        path: root.join(".ai/decisions"),
        message: "close requires an explicit human decision".into(),
    })
}

pub fn close_work_item_with_decision(
    root: &Path,
    work_item_id: &str,
    human_decision: &str,
) -> Result<LifecycleReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    if human_decision.trim().is_empty() {
        return Err(ObserverError::State {
            path: root.join(".ai/decisions"),
            message: "human decision must not be empty".into(),
        });
    }
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let archive = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.archive.json"));
    let manifest = read_json(&archive)?;
    verify_archive_manifest(&root, work_item_id, &manifest)?;
    let outcome = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.outcome.json"));
    let outcome_value = read_json(&outcome)?;
    if outcome_value["verification"]["status"] != "verified" {
        return Err(ObserverError::State {
            path: outcome,
            message: "close requires a verified outcome".into(),
        });
    }
    let timestamp = now();
    let receipt = LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "closed".into(),
        timestamp: timestamp.clone(),
    };
    let receipt_value = serde_json::to_value(&receipt).map_err(|error| ObserverError::State {
        path: root.join(".ai/decisions"),
        message: error.to_string(),
    })?;
    let decision_path = root
        .join(".ai/decisions")
        .join(format!("{work_item_id}.close.json"));
    if decision_path.exists() {
        return Err(ObserverError::State {
            path: decision_path,
            message: "work item is already closed".into(),
        });
    }
    let mut decision = receipt_value;
    decision["humanDecision"] = serde_json::Value::String(human_decision.trim().into());
    decision["decisionState"] = serde_json::Value::String("confirmed".into());
    atomic_json(&decision_path, &decision)?;
    Ok(receipt)
}

fn verify_archive_manifest(
    root: &Path,
    work_item_id: &str,
    manifest: &serde_json::Value,
) -> Result<(), ObserverError> {
    if manifest["workItemId"] != serde_json::Value::String(work_item_id.into())
        || manifest["state"] != serde_json::Value::String("archived".into())
    {
        return Err(ObserverError::State {
            path: root
                .join(".ai/work-items/archive")
                .join(format!("{work_item_id}.archive.json")),
            message: "archive manifest identity or state is invalid".into(),
        });
    }
    let archive = root.join(".ai/work-items/archive");
    for name in ["contract", "summary", "outcome"] {
        let path = archive.join(format!("{work_item_id}.{name}.json"));
        let bytes = fs::read(&path).map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        let expected = manifest["files"][format!("{name}Digest")]
            .as_str()
            .ok_or_else(|| ObserverError::State {
                path: path.clone(),
                message: format!("archive manifest is missing {name} digest"),
            })?;
        let actual = Digest::sha256_bytes(&bytes).to_string();
        if actual != expected {
            return Err(ObserverError::State {
                path,
                message: format!("archived {name} digest does not match manifest"),
            });
        }
    }
    Ok(())
}

pub fn generate_knowledge(root: &Path) -> Result<cockpit_knowledge::KnowledgeIndex, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let archive = root.join(".ai/work-items/archive");
    let knowledge = root.join(".ai/knowledge");
    let index_path = knowledge.join("index.json");
    if index_path.is_file() {
        let cached = read_json(&index_path)?;
        return serde_json::from_value(cached).map_err(|error| ObserverError::State {
            path: index_path,
            message: error.to_string(),
        });
    }
    let mut records = Vec::new();
    for entry in fs::read_dir(&archive).map_err(|source| ObserverError::Read {
        path: archive.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: archive.clone(),
            source,
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(work_item_id) = name.strip_suffix(".archive.json") else {
            continue;
        };
        let contract_path = archive.join(format!("{work_item_id}.contract.json"));
        let contract = read_json(&contract_path)?;
        let intent = contract["intent"].as_str().unwrap_or("unknown");
        records.push(cockpit_knowledge::project_record(
            work_item_id,
            intent,
            "archived",
            &format!(".ai/work-items/archive/{work_item_id}.archive.json"),
        ));
    }
    let index = cockpit_knowledge::KnowledgeIndex::from_records(records);
    fs::create_dir_all(&knowledge).map_err(|source| ObserverError::Read {
        path: knowledge.clone(),
        source,
    })?;
    let encoded = serde_json::to_value(&index).map_err(|error| ObserverError::State {
        path: knowledge.clone(),
        message: error.to_string(),
    })?;
    atomic_json(&knowledge.join("index.json"), &encoded)?;
    for record in &index.records {
        let record_value = serde_json::to_value(record).map_err(|error| ObserverError::State {
            path: knowledge.clone(),
            message: error.to_string(),
        })?;
        atomic_json(&root.join(&record.knowledge_path), &record_value)?;
    }
    Ok(index)
}

fn validate_work_item_id(id: &str) -> Result<(), ObserverError> {
    if id.is_empty()
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(ObserverError::State {
            path: PathBuf::from(id),
            message: "invalid work item id".into(),
        });
    }
    Ok(())
}

fn read_json(path: &Path) -> Result<serde_json::Value, ObserverError> {
    let bytes = fs::read(path).map_err(|source| ObserverError::Read {
        path: path.into(),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|error| ObserverError::State {
        path: path.into(),
        message: error.to_string(),
    })
}

fn atomic_json(path: &Path, value: &serde_json::Value) -> Result<(), ObserverError> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| ObserverError::State {
        path: path.into(),
        message: error.to_string(),
    })?;
    atomic_write(path, &bytes)
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), ObserverError> {
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&temporary, bytes).map_err(|source| ObserverError::Read {
        path: temporary.clone(),
        source,
    })?;
    fs::rename(&temporary, path).map_err(|source| ObserverError::Read {
        path: path.into(),
        source,
    })
}

pub fn observe(
    root: &Path,
    snapshot: &RepositorySnapshot,
) -> Result<RepositoryObservation, ObserverError> {
    let canonical_root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let canonical_snapshot_root =
        fs::canonicalize(&snapshot.root).map_err(|source| ObserverError::Read {
            path: snapshot.root.clone(),
            source,
        })?;
    if canonical_root != canonical_snapshot_root {
        return Err(ObserverError::SnapshotRootMismatch);
    }
    let mut files = Vec::new();
    collect_files(&canonical_root, &canonical_root, &mut files)?;
    let mut languages = Vec::new();
    let mut build_systems = Vec::new();
    let mut test_roots = Vec::new();
    let mut quality_commands = Vec::new();
    let mut ci_surfaces = Vec::new();
    let mut critical_domains = Vec::new();
    for relative in &files {
        match relative
            .extension()
            .and_then(|extension| extension.to_str())
        {
            Some("rs") => languages.push(LanguageSignal::Rust),
            Some("py") => languages.push(LanguageSignal::Python),
            Some("js") | Some("jsx") => languages.push(LanguageSignal::JavaScript),
            Some("ts") | Some("tsx") => languages.push(LanguageSignal::TypeScript),
            Some("go") => languages.push(LanguageSignal::Go),
            _ => {}
        }
        let path = relative.to_string_lossy();
        let path_string = path.to_string();
        if path.starts_with("tests/") || path == "tests" {
            test_roots.push("tests/**".into());
        }
        if path.starts_with(".github/workflows/") {
            ci_surfaces.push(path_string.clone());
        }
        for (needle, domain) in [
            ("security", "security"),
            ("payment", "payment"),
            ("auth", "identity"),
            ("production", "production"),
            ("release", "release"),
        ] {
            if path.contains(needle) {
                critical_domains.push(domain.into());
            }
        }
        if path == "Cargo.toml" {
            build_systems.push(BuildSystem::Cargo);
            quality_commands.push(QualityCommand {
                program: "cargo".into(),
                args: vec!["test".into(), "--workspace".into()],
                state: "detected".into(),
            });
        } else if path == "package.json" {
            build_systems.push(BuildSystem::Npm);
        } else if path == "pyproject.toml" {
            build_systems.push(BuildSystem::Poetry);
        } else if path == "go.mod" {
            build_systems.push(BuildSystem::Go);
        } else if path == "Makefile" {
            build_systems.push(BuildSystem::Make);
        }
    }
    languages.sort_by_key(|value| format!("{value:?}"));
    languages.dedup();
    build_systems.sort_by_key(|value| format!("{value:?}"));
    build_systems.dedup();
    test_roots.sort();
    test_roots.dedup();
    ci_surfaces.sort();
    ci_surfaces.dedup();
    critical_domains.sort();
    critical_domains.dedup();
    let mut hasher = Sha256::new();
    for path in &files {
        hasher.update(path.to_string_lossy().as_bytes());
        hasher.update([0]);
    }
    let snapshot_digest = Digest::sha256_bytes(&hasher.finalize());
    Ok(RepositoryObservation {
        snapshot_digest,
        languages,
        build_systems,
        test_roots,
        quality_commands,
        ci_surfaces,
        critical_domains,
        dependency_fingerprint: snapshot
            .dependency_fingerprint
            .parse()
            .unwrap_or_else(|_| Digest::sha256_bytes(b"invalid-dependency-fingerprint")),
        files_read: files.len() + snapshot.files_read,
        cache_hit: false,
    })
}

pub fn observe_cached(
    root: &Path,
    snapshot: &RepositorySnapshot,
) -> Result<RepositoryObservation, ObserverError> {
    let canonical_root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    if !canonical_root.join(".ai/cockpit.toml").is_file() {
        return observe(&canonical_root, snapshot);
    }
    let cache_path = canonical_root.join(".ai/decisions/observer-snapshot.json");
    if let Ok(cache) = read_json(&cache_path) {
        let matches = cache["treeDigest"] == snapshot.tree_digest
            && cache["diffDigest"] == snapshot.diff_digest
            && cache["dependencyFingerprint"] == snapshot.dependency_fingerprint;
        if matches
            && let Ok(mut observation) =
                serde_json::from_value::<RepositoryObservation>(cache["observation"].clone())
        {
            observation.cache_hit = true;
            return Ok(observation);
        }
    }
    let observation = observe(&canonical_root, snapshot)?;
    let cache = serde_json::json!({
        "treeDigest": snapshot.tree_digest,
        "diffDigest": snapshot.diff_digest,
        "dependencyFingerprint": snapshot.dependency_fingerprint,
        "observation": observation,
    });
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).map_err(|source| ObserverError::Read {
            path: parent.into(),
            source,
        })?;
    }
    atomic_json(&cache_path, &cache)?;
    Ok(observation)
}

pub fn classify_evolution(
    _profile: &cockpit_protocol::ProjectProfile,
    observation: &RepositoryObservation,
    snapshot: &RepositorySnapshot,
) -> Vec<EvolutionEvent> {
    snapshot
        .changed_paths
        .iter()
        .map(|path| {
            let normalized = path.replace('\\', "/");
            let is_governance = normalized.starts_with(".github/")
                || normalized.contains("security")
                || normalized.contains("release")
                || normalized.contains("branch-protection")
                || normalized.contains("production");
            let is_known_test = observation.test_roots.iter().any(|root| {
                root.strip_suffix("/**")
                    .is_some_and(|prefix| normalized.starts_with(&format!("{prefix}/")))
            });
            let is_new_capability = normalized.contains("playwright")
                || normalized.contains("cypress")
                || normalized.contains("nextest")
                || normalized.ends_with("playwright.config.ts")
                || normalized.ends_with("playwright.config.js");
            if is_governance {
                EvolutionEvent {
                    class: EvolutionClass::L3,
                    event_type: "governance_change".into(),
                    path: normalized,
                    action: "needs_human_decision".into(),
                }
            } else if is_new_capability {
                EvolutionEvent {
                    class: EvolutionClass::L2,
                    event_type: "new_test_framework".into(),
                    path: normalized,
                    action: "needs_confirmation".into(),
                }
            } else if is_known_test {
                EvolutionEvent {
                    class: EvolutionClass::L1,
                    event_type: "new_test".into(),
                    path: normalized,
                    action: "auto_absorb".into(),
                }
            } else {
                EvolutionEvent {
                    class: EvolutionClass::L0,
                    event_type: "content_change".into(),
                    path: normalized,
                    action: "auto_absorb".into(),
                }
            }
        })
        .collect()
}

pub fn profile_update_proposal(
    profile: &cockpit_protocol::ProjectProfile,
    events: &[EvolutionEvent],
) -> Option<ProfileUpdateProposal> {
    let candidate = events
        .iter()
        .find(|event| matches!(&event.class, EvolutionClass::L2 | EvolutionClass::L3))?;
    Some(ProfileUpdateProposal {
        from_profile_version: profile.profile_version,
        candidate: candidate.path.clone(),
        reason: candidate.event_type.clone(),
        requires_human_confirmation: true,
    })
}

pub fn confirm_profile_update(
    root: &Path,
    program: &str,
    args: &[String],
) -> Result<AttachedProfile, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let project_path = root.join(".ai/project.json");
    let current: AttachedProfile = read_json(&project_path).and_then(|value| {
        serde_json::from_value(value).map_err(|error| ObserverError::State {
            path: project_path.clone(),
            message: error.to_string(),
        })
    })?;
    let mut tests = current.tests.clone();
    let candidate = QualityCommand {
        program: program.into(),
        args: args.to_vec(),
        state: "verified".into(),
    };
    if !tests.contains(&candidate) {
        tests.push(candidate);
    }
    tests.sort_by(|left, right| {
        (left.program.as_str(), &left.args).cmp(&(right.program.as_str(), &right.args))
    });
    let profile_version = current.profile_version + 1;
    let profile = cockpit_protocol::ProjectProfile {
        profile_version,
        repository_id: current.repository_id.clone(),
        tests: tests.clone(),
        build_systems: current.build_systems.clone(),
    };
    let profile_digest =
        cockpit_protocol::digest_json(&profile).map_err(|error| ObserverError::State {
            path: project_path.clone(),
            message: error.to_string(),
        })?;
    let updated = AttachedProfile {
        profile_version,
        repository_id: current.repository_id,
        state: "calibrated".into(),
        profile_digest: Some(profile_digest.clone()),
        tests,
        build_systems: current.build_systems,
    };
    let value = serde_json::to_value(&updated).map_err(|error| ObserverError::State {
        path: project_path.clone(),
        message: error.to_string(),
    })?;
    atomic_json(&project_path, &value)?;
    let decision = serde_json::json!({
        "kind": "project_profile_confirmation",
        "profileVersion": profile_version,
        "profileDigest": profile_digest,
        "candidate": {"program": program, "args": args},
        "state": "confirmed",
        "createdAt": now(),
    });
    atomic_json(
        &root
            .join(".ai/decisions")
            .join(format!("profile-v{profile_version}.json")),
        &decision,
    )?;
    Ok(updated)
}

fn collect_files(
    root: &Path,
    current: &Path,
    output: &mut Vec<PathBuf>,
) -> Result<(), ObserverError> {
    let entries = fs::read_dir(current).map_err(|source| ObserverError::Read {
        path: current.into(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: current.into(),
            source,
        })?;
        let path = entry.path();
        if path
            .file_name()
            .is_some_and(|name| name == ".git" || name == "target")
        {
            continue;
        }
        if path.is_dir() {
            collect_files(root, &path, output)?;
        } else if path.is_file() {
            output.push(path.strip_prefix(root).unwrap_or(&path).to_path_buf());
        }
    }
    output.sort();
    Ok(())
}

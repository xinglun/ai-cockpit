use super::{
    ObserverError, atomic_json, git_text, knowledge_source_digest, observe, read_contract,
    read_json, repository_id, snapshot_digest, validate_work_item_id,
};
use cockpit_protocol::{FactOrigin, ImplementationApproach};
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn generate_knowledge(root: &Path) -> Result<cockpit_knowledge::KnowledgeIndex, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let archive = root.join(".ai/work-items/archive");
    let knowledge = root.join(".ai/knowledge");
    let index_path = knowledge.join("index.json");
    let current_source_revision = git_text(&root, &["rev-parse", "--verify", "HEAD^{commit}"]);
    if index_path.is_file() {
        // A derived cache is disposable.  An unreadable, malformed, or
        // schema-incompatible index is treated as stale and rebuilt through
        // this explicit query path; authority remains in the archive.
        if let Ok(cached) = read_json(&index_path)
            && let Ok(index) = serde_json::from_value::<cockpit_knowledge::KnowledgeIndex>(cached)
            && !index.source_digest.is_empty()
            && index.is_structurally_valid()
        {
            if let (Some(cached_revision), Some(current_revision)) = (
                index.source_revision.as_deref(),
                current_source_revision.as_deref(),
            ) && cached_revision == current_revision
                && knowledge_archive_is_clean(&root).is_some_and(|clean| clean)
            {
                return Ok(index);
            }
            // A dirty or uncertain source boundary cannot be trusted from
            // metadata alone. Recompute the content digest before reuse;
            // this preserves archive tamper detection for uncommitted and
            // non-Git repositories.
            let source_digest = knowledge_source_digest(&archive)?;
            if index.source_digest == source_digest {
                if index.source_revision != current_source_revision {
                    let mut refreshed = index;
                    refreshed.source_revision = current_source_revision.clone();
                    let encoded =
                        serde_json::to_value(&refreshed).map_err(|error| ObserverError::State {
                            path: index_path.clone(),
                            message: error.to_string(),
                        })?;
                    atomic_json(&index_path, &encoded)?;
                    return Ok(refreshed);
                }
                return Ok(index);
            }
        }
    }
    let source_digest = knowledge_source_digest(&archive)?;
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
        let scope = contract_scope(&contract);
        records.push(cockpit_knowledge::project_record_with_context(
            work_item_id,
            intent,
            &scope,
            "archived",
            &format!(".ai/work-items/archive/{work_item_id}.archive.json"),
        ));
    }
    let index = cockpit_knowledge::KnowledgeIndex::with_source_metadata(
        records,
        source_digest,
        current_source_revision,
    );
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

/// Build and persist a request-scoped, provenance-aware implementation
/// approach.  This explicit command is the write boundary for the approach
/// projection; read-only callers should use [`implementation_approach_read_only`].
pub fn implementation_approach(
    root: &Path,
    work_item_id: &str,
) -> Result<ImplementationApproach, ObserverError> {
    implementation_approach_internal(root, work_item_id, true)
}

/// Build the same implementation approach projection without materializing an
/// `.approach.json` artifact.  `work-item inspect` uses this variant so that
/// inspection remains a truthful read-only operation.
pub fn implementation_approach_read_only(
    root: &Path,
    work_item_id: &str,
) -> Result<ImplementationApproach, ObserverError> {
    implementation_approach_internal(root, work_item_id, false)
}

fn implementation_approach_internal(
    root: &Path,
    work_item_id: &str,
    persist: bool,
) -> Result<ImplementationApproach, ObserverError> {
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
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&contract_path)?;
    let observation = observe(&root, &snapshot)?;
    let snapshot_digest = snapshot_digest(&snapshot)?;
    let evidence_prefix = format!(".ai/work-items/active/{work_item_id}");
    let mut facts = vec![
        cockpit_protocol::TraceableFact {
            key: "repositoryId".into(),
            value: serde_json::Value::String(contract.repository_id.clone()),
            origin: FactOrigin::Observed,
            evidence_refs: vec![".ai/cockpit.toml".into()],
            confidence: "high".into(),
        },
        cockpit_protocol::TraceableFact {
            key: "baseRevision".into(),
            value: serde_json::Value::String(contract.base_revision.clone()),
            origin: FactOrigin::Observed,
            evidence_refs: vec!["git:HEAD".into()],
            confidence: "high".into(),
        },
        cockpit_protocol::TraceableFact {
            key: "languages".into(),
            value: serde_json::to_value(&observation.languages).map_err(|error| {
                ObserverError::State {
                    path: root.clone(),
                    message: error.to_string(),
                }
            })?,
            origin: FactOrigin::Observed,
            evidence_refs: vec!["repository-snapshot".into()],
            confidence: "high".into(),
        },
        cockpit_protocol::TraceableFact {
            key: "buildSystems".into(),
            value: serde_json::to_value(&observation.build_systems).map_err(|error| {
                ObserverError::State {
                    path: root.clone(),
                    message: error.to_string(),
                }
            })?,
            origin: FactOrigin::Observed,
            evidence_refs: vec!["repository-snapshot".into()],
            confidence: "high".into(),
        },
    ];
    facts.sort_by(|left, right| left.key.cmp(&right.key));
    let mut derivations = Vec::new();
    if !observation.quality_commands.is_empty() {
        derivations.push(cockpit_protocol::TraceableDerivation {
            key: "verificationCapability".into(),
            value: serde_json::to_value(&observation.quality_commands).map_err(|error| {
                ObserverError::State {
                    path: root.clone(),
                    message: error.to_string(),
                }
            })?,
            rule: "observer.quality_commands_from_detected_build_system".into(),
            input_fact_keys: vec!["buildSystems".into()],
            evidence_refs: vec!["repository-snapshot".into()],
            confidence: "medium".into(),
        });
    }
    let mut unknowns = Vec::new();
    if contract.intent.is_empty() {
        unknowns.push("intent".into());
    }
    if contract.scope.is_empty() {
        unknowns.push("scope".into());
    }
    if contract.acceptance_criteria.is_empty() {
        unknowns.push("acceptanceCriteria".into());
    }
    if contract.authority.trim().is_empty() || contract.authority == "unknown" {
        unknowns.push("authority".into());
    }
    unknowns.sort();
    unknowns.dedup();
    let mut evidence_refs = vec![evidence_prefix, "repository-snapshot".into()];
    evidence_refs.sort();
    let approach = ImplementationApproach {
        schema_version: 2,
        repository_id: contract.repository_id,
        work_item_id: work_item_id.into(),
        repository_snapshot_digest: snapshot_digest,
        facts,
        derivations,
        unknowns,
        evidence_refs,
    };
    if persist {
        atomic_json(
            &root
                .join(".ai/work-items/active")
                .join(format!("{work_item_id}.approach.json")),
            &serde_json::to_value(&approach).map_err(|error| ObserverError::State {
                path: root.clone(),
                message: error.to_string(),
            })?,
        )?;
    }
    Ok(approach)
}

/// Return knowledge v2 projections without replacing the legacy index.  The
/// projection is derived from archive contracts and bound to one snapshot.
pub fn generate_knowledge_v2(
    root: &Path,
) -> Result<Vec<cockpit_protocol::KnowledgeV2Record>, ObserverError> {
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
    let digest = snapshot_digest(&snapshot)?;
    let repository_id = repository_id(&root).to_string();
    let archive = root.join(".ai/work-items/archive");
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
        let contract: serde_json::Value =
            read_json(&archive.join(format!("{work_item_id}.contract.json")))?;
        let intent = contract["intent"].as_str().unwrap_or("unknown");
        let scope = contract_scope(&contract);
        records.push(cockpit_knowledge::project_record_v2_with_context(
            &repository_id,
            work_item_id,
            intent,
            &scope,
            "archived",
            &format!(".ai/work-items/archive/{work_item_id}.archive.json"),
            digest.clone(),
        ));
    }
    records.sort_by(|left, right| left.work_item_id.cmp(&right.work_item_id));
    let path = root.join(".ai/knowledge/index.v2.json");
    atomic_json(
        &path,
        &serde_json::to_value(&records).map_err(|error| ObserverError::State {
            path: path.clone(),
            message: error.to_string(),
        })?,
    )?;
    Ok(records)
}

fn contract_scope(contract: &serde_json::Value) -> Vec<String> {
    contract["scope"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn knowledge_archive_is_clean(root: &Path) -> Option<bool> {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
            "--",
            ".ai/work-items/archive",
        ])
        .output()
        .ok()?;
    status
        .status
        .success()
        .then(|| String::from_utf8_lossy(&status.stdout).trim().is_empty())
}

//! Rust-native readers for the optional repository capability/profile
//! declarations.  This module is deliberately read-only: declarations are
//! human-owned inputs and the Runtime only validates, binds, and projects
//! them.

use cockpit_core::Digest;
use cockpit_git::RepositorySnapshot;
use cockpit_protocol::{
    Contract, ProjectCapabilityDeclaration, ProjectGovernanceProjection, ProjectProfilePolicy,
    ProjectSuccessCriteriaDeclaration,
};
use serde::{Serialize, de::DeserializeOwned};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path};

use super::{ObserverError, reject_duplicate_json_keys, repository_id, snapshot_digest};

const DECLARATION_MAX_BYTES: u64 = 1024 * 1024;
const SCHEMA_VERSION: u32 = 1;

#[derive(Debug)]
enum Loaded<T> {
    Missing,
    Invalid(String),
    Valid { value: T, digest: Digest },
}

impl<T> Loaded<T> {
    fn unknown(&self, kind: &str) -> Option<String> {
        match self {
            Self::Missing => Some(format!("project_{kind}_missing")),
            Self::Invalid(reason) => Some(format!("project_{kind}_{reason}")),
            Self::Valid { .. } => None,
        }
    }

    fn value(&self) -> Option<&T> {
        match self {
            Self::Valid { value, .. } => Some(value),
            Self::Missing | Self::Invalid(_) => None,
        }
    }

    fn digest(&self) -> Option<Digest> {
        match self {
            Self::Valid { digest, .. } => Some(digest.clone()),
            Self::Missing | Self::Invalid(_) => None,
        }
    }
}

fn load_declaration<T, V>(
    path: &Path,
    kind: &str,
    expected_repository_id: &str,
    expected_snapshot_digest: &Digest,
    validate: V,
) -> Result<Loaded<T>, ObserverError>
where
    T: DeserializeOwned + Serialize,
    V: FnOnce(&T) -> Option<&'static str>,
{
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Loaded::Missing),
        Err(source) => {
            return Err(ObserverError::Read {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    if metadata.file_type().is_symlink() {
        return Ok(Loaded::Invalid("symlink".into()));
    }
    if !metadata.file_type().is_file() {
        return Ok(Loaded::Invalid("invalid".into()));
    }
    if metadata.len() > DECLARATION_MAX_BYTES {
        return Ok(Loaded::Invalid("invalid".into()));
    }
    let bytes = fs::read(path).map_err(|source| ObserverError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    if let Err(_error) = reject_duplicate_json_keys(&bytes) {
        return Ok(Loaded::Invalid("invalid".into()));
    }
    let value = match serde_json::from_slice::<serde_json::Value>(&bytes)
        .ok()
        .and_then(|json| serde_json::from_value::<T>(json).ok())
    {
        Some(value) => value,
        None => return Ok(Loaded::Invalid("invalid".into())),
    };
    if let Some(reason) = validate(&value) {
        return Ok(Loaded::Invalid(reason.into()));
    }

    // The three declaration types expose the common identity/freshness
    // fields.  Keep this check explicit rather than using reflection so the
    // protocol remains strict and compile-time typed.
    let encoded = serde_json::to_value(&value).map_err(|error| ObserverError::State {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    let Some(repository) = encoded.get("repositoryId").and_then(|v| v.as_str()) else {
        return Ok(Loaded::Invalid("invalid".into()));
    };
    if repository != expected_repository_id {
        return Ok(Loaded::Invalid("repository_mismatch".into()));
    }
    let Some(snapshot) = encoded
        .get("repositorySnapshotDigest")
        .and_then(|v| v.as_str())
    else {
        return Ok(Loaded::Invalid("snapshot_digest_missing".into()));
    };
    if snapshot != expected_snapshot_digest.to_string() {
        return Ok(Loaded::Invalid("stale".into()));
    }
    let digest = cockpit_protocol::digest_json(&value).map_err(|error| ObserverError::State {
        path: path.to_path_buf(),
        message: format!("failed to digest {kind} declaration: {error}"),
    })?;
    Ok(Loaded::Valid { value, digest })
}

fn valid_nonempty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn requires_project_capability_mapping(operation: &str) -> bool {
    // These values are historical lifecycle/verification labels, not
    // repository capability names. Preserve their protocol-v1 behavior; a
    // project-owned mapping is required for explicit domain operations such
    // as `documentation.modify` or `release.publish`.
    !matches!(
        operation,
        "code" | "implementation" | "review" | "cleanup" | "investigate" | "modify_source"
    )
}

fn unique_nonempty(values: &[String]) -> bool {
    let mut seen = BTreeSet::new();
    values
        .iter()
        .all(|value| valid_nonempty(value) && seen.insert(value))
}

fn valid_path_pattern(value: &str) -> bool {
    if !valid_nonempty(value) || value.starts_with('/') || value.contains('\\') {
        return false;
    }
    !Path::new(value).components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    })
}

fn validate_capabilities(value: &ProjectCapabilityDeclaration) -> Option<&'static str> {
    if value.schema_version != SCHEMA_VERSION
        || !unique_nonempty(&value.capabilities)
        || !unique_nonempty(&value.non_capabilities)
        || !unique_nonempty(&value.critical_domains)
        || value
            .capabilities
            .iter()
            .any(|capability| value.non_capabilities.contains(capability))
        || value
            .operation_mappings
            .iter()
            .any(|(operation, required)| !valid_nonempty(operation) || !unique_nonempty(required))
    {
        return Some("invalid");
    }
    None
}

fn validate_success_criteria(value: &ProjectSuccessCriteriaDeclaration) -> Option<&'static str> {
    if value.schema_version != SCHEMA_VERSION
        || !valid_nonempty(&value.work_item_id)
        || value.criteria.is_empty()
        || value.criteria.iter().any(|criterion| {
            !valid_nonempty(&criterion.id)
                || !valid_nonempty(&criterion.statement)
                || !unique_nonempty(&criterion.evidence_hints)
        })
        || {
            let ids = value
                .criteria
                .iter()
                .map(|criterion| criterion.id.clone())
                .collect::<Vec<_>>();
            !unique_nonempty(&ids)
        }
    {
        return Some("invalid");
    }
    None
}

fn validate_profile_policy(value: &ProjectProfilePolicy) -> Option<&'static str> {
    let boundaries = &value.approved_boundaries;
    if value.schema_version != SCHEMA_VERSION
        || !unique_nonempty(&value.critical_domains)
        || !unique_nonempty(&value.review_requirements)
        || !unique_nonempty(&value.unknowns)
        || !boundaries
            .production_roots
            .iter()
            .all(|v| valid_path_pattern(v))
        || !boundaries
            .feature_roots
            .iter()
            .all(|v| valid_path_pattern(v))
        || !boundaries.test_roots.iter().all(|v| valid_path_pattern(v))
        || !boundaries
            .generated_paths
            .iter()
            .all(|v| valid_path_pattern(v))
        || !boundaries
            .critical_paths
            .iter()
            .all(|v| valid_path_pattern(v))
    {
        return Some("invalid");
    }
    None
}

struct Declarations {
    capabilities: Loaded<ProjectCapabilityDeclaration>,
    success_criteria: Loaded<ProjectSuccessCriteriaDeclaration>,
    profile_policy: Loaded<ProjectProfilePolicy>,
}

fn load_declarations(
    root: &Path,
    expected_repository_id: &str,
    expected_snapshot_digest: &Digest,
) -> Result<Declarations, ObserverError> {
    let project = root.join(".ai/project");
    Ok(Declarations {
        capabilities: load_declaration(
            &project.join("capabilities.json"),
            "capabilities",
            expected_repository_id,
            expected_snapshot_digest,
            validate_capabilities,
        )?,
        success_criteria: load_declaration(
            &project.join("success_criteria.json"),
            "success_criteria",
            expected_repository_id,
            expected_snapshot_digest,
            validate_success_criteria,
        )?,
        profile_policy: load_declaration(
            &project.join("profile-policy.json"),
            "profile_policy",
            expected_repository_id,
            expected_snapshot_digest,
            validate_profile_policy,
        )?,
    })
}

fn declaration_unknowns(declarations: &Declarations) -> Vec<String> {
    let mut unknowns = Vec::new();
    if let Some(code) = declarations.capabilities.unknown("capabilities") {
        unknowns.push(code);
    }
    if let Some(code) = declarations.success_criteria.unknown("success_criteria") {
        unknowns.push(code);
    }
    if let Some(code) = declarations.profile_policy.unknown("profile_policy") {
        unknowns.push(code);
    }
    unknowns.sort();
    unknowns.dedup();
    unknowns
}

/// Return a no-write, repository-bound projection of optional project
/// declarations. Invalid inputs remain visible as stable unknown codes.
pub fn project_governance_projection(
    root: &Path,
    snapshot: &RepositorySnapshot,
) -> Result<ProjectGovernanceProjection, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.to_path_buf(),
        source,
    })?;
    let snapshot_root = fs::canonicalize(&snapshot.root).map_err(|source| ObserverError::Read {
        path: snapshot.root.clone(),
        source,
    })?;
    if root != snapshot_root {
        return Err(ObserverError::SnapshotRootMismatch);
    }
    let current_snapshot_digest = snapshot_digest(snapshot)?;
    let expected_repository_id = repository_id(&root).to_string();
    let declarations = load_declarations(&root, &expected_repository_id, &current_snapshot_digest)?;
    Ok(ProjectGovernanceProjection {
        schema_version: SCHEMA_VERSION,
        repository_id: expected_repository_id,
        snapshot_digest: current_snapshot_digest,
        capabilities_digest: declarations.capabilities.digest(),
        success_criteria_digest: declarations.success_criteria.digest(),
        success_criteria: declarations.success_criteria.value().cloned(),
        profile_policy_digest: declarations.profile_policy.digest(),
        unknowns: declaration_unknowns(&declarations),
    })
}

/// Bind an explicit Contract operation to the repository declaration. Intent
/// prose, detected files, and model output are never accepted as substitutes.
pub fn project_governance_unknowns(
    root: &Path,
    contract: &Contract,
    snapshot: &RepositorySnapshot,
) -> Result<Vec<String>, ObserverError> {
    let operation = contract
        .requested_operation
        .as_deref()
        .filter(|value| valid_nonempty(value))
        .or_else(|| {
            contract
                .operation
                .as_deref()
                .filter(|value| valid_nonempty(value))
        });
    // Optional project declarations are not a prerequisite for legacy
    // Contracts.  Only an explicit operation opts into capability binding.
    let Some(operation) = operation else {
        return Ok(Vec::new());
    };
    if !requires_project_capability_mapping(operation) {
        return Ok(Vec::new());
    }
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.to_path_buf(),
        source,
    })?;
    let current_snapshot_digest = snapshot_digest(snapshot)?;
    let declarations = load_declarations(
        &root,
        &repository_id(&root).to_string(),
        &current_snapshot_digest,
    )?;
    let mut unknowns = declaration_unknowns(&declarations);
    let Some(capabilities) = declarations.capabilities.value() else {
        unknowns.push("project_capability_mapping_unknown".into());
        unknowns.sort();
        unknowns.dedup();
        return Ok(unknowns);
    };
    let Some(required) = capabilities.operation_mappings.get(operation) else {
        unknowns.push("project_capability_mapping_missing".into());
        unknowns.sort();
        unknowns.dedup();
        return Ok(unknowns);
    };
    if required
        .iter()
        .any(|capability| capabilities.non_capabilities.contains(capability))
    {
        unknowns.push("project_capability_mapping_conflict".into());
    }
    if required
        .iter()
        .any(|capability| !capabilities.capabilities.contains(capability))
    {
        unknowns.push("project_capability_mapping_insufficient".into());
    }
    unknowns.sort();
    unknowns.dedup();
    Ok(unknowns)
}

/// Exposed for tests and future read-only adapters that need to inspect the
/// validated success criteria without granting it governance authority.
pub fn project_success_criteria(
    root: &Path,
    snapshot: &RepositorySnapshot,
) -> Result<Option<ProjectSuccessCriteriaDeclaration>, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.to_path_buf(),
        source,
    })?;
    let current_snapshot_digest = snapshot_digest(snapshot)?;
    let declarations = load_declarations(
        &root,
        &repository_id(&root).to_string(),
        &current_snapshot_digest,
    )?;
    Ok(declarations.success_criteria.value().cloned())
}

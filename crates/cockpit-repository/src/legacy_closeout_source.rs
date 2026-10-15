//! Explicit present-day proof obligations for legacy-v1 closeouts.
//!
//! A canonical name is an obligation even when its content is damaged. Parsing
//! or a foreign identity must never make that Work Item disappear from scope.
use super::{ObserverError, open_cap_directory_nofollow_strict, validate_work_item_id};
use cap_std::fs::Dir;
use std::{collections::BTreeSet, fs, path::Path};

pub(super) const REQUIREMENT_SUFFIX: &str = ".legacy-closeout-source-requirement.json";
pub(super) const REGISTRATION_SUFFIX: &str = ".legacy-closeout-source-registration.json";
pub(super) const EXTENSION_SUFFIX: &str = ".legacy-closeout-source-extension.json";

pub(super) fn has_obligation(root: &Path, id: &str) -> bool {
    [REQUIREMENT_SUFFIX, REGISTRATION_SUFFIX, EXTENSION_SUFFIX]
        .iter()
        .any(|suffix| {
            match fs::symlink_metadata(root.join(".ai/decisions").join(format!("{id}{suffix}"))) {
                Ok(_) => true,
                Err(error) => error.kind() != std::io::ErrorKind::NotFound,
            }
        })
}

pub(super) fn obligation_ids(root: &Path) -> Result<BTreeSet<String>, ObserverError> {
    let path = root.join(".ai/decisions");
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeSet::new()),
        Err(source) => return Err(ObserverError::Read { path, source }),
        Ok(_) => {}
    }
    let ambient = Dir::open_ambient_dir(root, cap_std::ambient_authority()).map_err(|source| {
        ObserverError::Read {
            path: root.into(),
            source,
        }
    })?;
    let ai = open_cap_directory_nofollow_strict(&ambient, ".ai", &root.join(".ai"))?;
    let decisions = open_cap_directory_nofollow_strict(&ai, "decisions", &path)?;
    let entries = decisions.entries().map_err(|source| ObserverError::Read {
        path: path.clone(),
        source,
    })?;
    let mut ids = BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        for suffix in [REQUIREMENT_SUFFIX, REGISTRATION_SUFFIX, EXTENSION_SUFFIX] {
            if let Some(id) = name.strip_suffix(suffix)
                && validate_work_item_id(id).is_ok()
            {
                ids.insert(id.to_owned());
            }
        }
    }
    Ok(ids)
}

use super::{
    acquire_lifecycle_lock, read_cap_file_nofollow_bounded, reject_duplicate_json_keys,
    repository_id, snapshot_digest,
};
use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cockpit_core::Digest;
use cockpit_protocol::{
    LegacyCloseoutSourceAttestation, LegacyCloseoutSourceFileBinding as FileBinding,
    LegacyCloseoutSourceOperation as Operation, LegacyCloseoutSourcePlan as Plan,
    LegacyCloseoutSourceRegistration as Registration, LegacyCloseoutSourceRequest as Request,
    LegacyCloseoutSourceRequirement as Requirement, RuntimeContext, digest_json,
};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    io::Write,
    path::{Component, PathBuf},
    process::Command,
    sync::atomic::Ordering,
};
const MAX_BYTES: u64 = 4 * 1024 * 1024;
const MAX_RECORD_BYTES: u64 = 2 * MAX_BYTES;
const MAX_FILES: usize = 128;

fn error(root: &Path, code: &str) -> ObserverError {
    ObserverError::State {
        path: root.into(),
        message: format!("legacy_closeout_source:{code}"),
    }
}
fn canonical(root: &Path) -> Result<PathBuf, ObserverError> {
    fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })
}
fn encode<T: Serialize>(root: &Path, value: &T) -> Result<Vec<u8>, ObserverError> {
    serde_json::to_vec_pretty(value).map_err(|_| error(root, "serialization_failed"))
}
fn parse<T: DeserializeOwned>(root: &Path, bytes: &[u8]) -> Result<T, ObserverError> {
    if bytes.len() as u64 > MAX_RECORD_BYTES {
        return Err(error(root, "record_budget_exceeded"));
    }
    reject_duplicate_json_keys(bytes).map_err(|_| error(root, "duplicate_json_key"))?;
    serde_json::from_slice(bytes).map_err(|_| error(root, "strict_schema_invalid"))
}
fn digest<T: Serialize>(root: &Path, value: &T) -> Result<Digest, ObserverError> {
    digest_json(value).map_err(|_| error(root, "identity_failed"))
}
fn relative_parts(root: &Path, relative: &str) -> Result<Vec<String>, ObserverError> {
    if relative.contains('\\') || !relative.starts_with(".ai/") {
        return Err(error(root, "path_invalid"));
    }
    Path::new(relative)
        .components()
        .map(|part| match part {
            Component::Normal(name) => name
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| error(root, "path_invalid")),
            _ => Err(error(root, "path_invalid")),
        })
        .collect()
}
/// Read every component through pinned directory handles with no-follow opens.
fn safe_read(root: &Path, relative: &str, maximum: u64) -> Result<Vec<u8>, ObserverError> {
    let parts = relative_parts(root, relative)?;
    let mut directory =
        Dir::open_ambient_dir(root, cap_std::ambient_authority()).map_err(|source| {
            ObserverError::Read {
                path: root.into(),
                source,
            }
        })?;
    let mut display = root.to_path_buf();
    for part in &parts[..parts.len() - 1] {
        display.push(part);
        directory = open_cap_directory_nofollow_strict(&directory, part, &display)?;
    }
    let leaf = parts.last().ok_or_else(|| error(root, "path_invalid"))?;
    read_cap_file_nofollow_bounded(&directory, leaf, &root.join(relative), maximum)
}
fn optional_read(root: &Path, relative: &str) -> Result<Option<Vec<u8>>, ObserverError> {
    match fs::symlink_metadata(root.join(relative)) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(ObserverError::Read {
            path: root.join(relative),
            source,
        }),
        Ok(_) => safe_read(root, relative, MAX_RECORD_BYTES).map(Some),
    }
}
fn binding(root: &Path, path: String, bytes: &[u8]) -> Result<FileBinding, ObserverError> {
    let canonical_digest = if path.ends_with(".json") {
        let value: serde_json::Value = parse(root, bytes)?;
        Some(digest(root, &value)?)
    } else {
        None
    };
    Ok(FileBinding {
        path,
        bytes: bytes.len() as u64,
        raw_digest: Digest::sha256_bytes(bytes),
        canonical_digest,
    })
}
fn receipt_path(id: &str) -> String {
    format!(".ai/decisions/{id}.cross-checkout-closeout-recovery.json")
}
fn requirement_path(id: &str) -> String {
    format!(".ai/decisions/{id}{REQUIREMENT_SUFFIX}")
}
fn registration_path(id: &str) -> String {
    format!(".ai/decisions/{id}{REGISTRATION_SUFFIX}")
}
fn requirement(
    root: &Path,
    registration: &Registration,
    bytes: &str,
) -> Result<Vec<u8>, ObserverError> {
    encode(
        root,
        &Requirement {
            schema_version: 1,
            kind: "legacy_closeout_source_requirement".into(),
            serialization_version: 1,
            repository_id: registration.repository_id.clone(),
            work_item_id: registration.work_item_id.clone(),
            registration_raw_digest: Digest::sha256_bytes(bytes.as_bytes()),
            registration_bytes: bytes.into(),
        },
    )
}

fn scoped_dependency_paths(
    root: &Path,
    id: &str,
    repo: &str,
) -> Result<Vec<String>, ObserverError> {
    let mut paths = BTreeSet::new();
    for (relative, suffix) in [
        (".ai/evidence/external", ".delegated.json"),
        (".ai/evidence/observations", ".observation.json"),
    ] {
        match fs::symlink_metadata(root.join(relative)) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(source) => {
                return Err(ObserverError::Read {
                    path: root.join(relative),
                    source,
                });
            }
            Ok(_) => {}
        }
        let mut dir =
            Dir::open_ambient_dir(root, cap_std::ambient_authority()).map_err(|source| {
                ObserverError::Read {
                    path: root.into(),
                    source,
                }
            })?;
        let mut display = root.to_path_buf();
        for part in relative_parts(root, relative)? {
            display.push(&part);
            dir = open_cap_directory_nofollow_strict(&dir, &part, &display)?;
        }
        let entries = dir.entries().map_err(|source| ObserverError::Read {
            path: display.clone(),
            source,
        })?;
        let mut scanned = 0usize;
        for entry in entries {
            scanned += 1;
            if scanned > 4096 {
                return Err(error(root, "dependency_directory_budget_exceeded"));
            }
            let entry = entry.map_err(|source| ObserverError::Read {
                path: display.clone(),
                source,
            })?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with(&format!("{id}.")) || !name.ends_with(suffix) {
                continue;
            }
            let path = format!("{relative}/{name}");
            let bytes = safe_read(root, &path, MAX_BYTES)?;
            let evidence = if suffix == ".delegated.json" {
                let receipt: cockpit_protocol::DelegatedEvidenceReceipt = parse(root, &bytes)?;
                if receipt.schema_version != 1
                    || receipt.repository_id != repo
                    || receipt.work_item_id != id
                    || receipt.runtime_version.trim().is_empty()
                    || chrono::DateTime::parse_from_rfc3339(&receipt.bound_at).is_err()
                    || receipt
                        .evidence
                        .digest
                        .to_string()
                        .parse::<Digest>()
                        .is_err()
                    || receipt
                        .runtime_digest
                        .to_string()
                        .parse::<Digest>()
                        .is_err()
                    || name
                        != format!(
                            "{id}.{}.delegated.json",
                            &receipt.evidence.digest.to_string()[7..]
                        )
                {
                    return Err(error(root, "delegated_dependency_invalid"));
                }
                receipt.evidence
            } else {
                let receipt: cockpit_protocol::DelegatedEvidenceObservationReceipt =
                    parse(root, &bytes)?;
                if receipt
                    .observation_digest
                    .to_string()
                    .parse::<Digest>()
                    .is_err()
                    || receipt
                        .evidence
                        .digest
                        .to_string()
                        .parse::<Digest>()
                        .is_err()
                    || receipt
                        .runtime_digest
                        .to_string()
                        .parse::<Digest>()
                        .is_err()
                {
                    return Err(error(root, "observation_dependency_invalid"));
                }
                cockpit_protocol::validate_delegated_evidence_observation_receipt(&receipt)
                    .map_err(|_| error(root, "observation_dependency_invalid"))?;
                if receipt.repository_id != repo
                    || receipt.work_item_id != id
                    || chrono::DateTime::parse_from_rfc3339(&receipt.bound_at).is_err()
                    || name
                        != format!(
                            "{id}.{}.observation.json",
                            &receipt.observation_digest.to_string()[7..]
                        )
                {
                    return Err(error(root, "observation_dependency_invalid"));
                }
                receipt.evidence
            };
            // Open raw bytes safely before invoking the existing metadata
            // validator, whose contract lookup must not create a raw directory.
            let raw = safe_read(root, &evidence.raw_evidence_ref, MAX_BYTES)?;
            super::validate_delegated_metadata(root, id, &evidence, &raw)?;
            paths.insert(path);
            paths.insert(evidence.raw_evidence_ref);
            if paths.len() > MAX_FILES {
                return Err(error(root, "dependency_budget_exceeded"));
            }
        }
    }
    Ok(paths.into_iter().collect())
}
fn dependencies(inventory: &[FileBinding]) -> Vec<FileBinding> {
    inventory
        .iter()
        .filter(|file| {
            file.path.starts_with(".ai/evidence/external/")
                || file.path.starts_with(".ai/evidence/observations/")
        })
        .cloned()
        .collect()
}

fn observe_history(
    root: &Path,
    id: &str,
    repo: &str,
) -> Result<(serde_json::Value, Vec<FileBinding>), ObserverError> {
    // Reject invalid old-v1/origin facts before accepting any new obligation.
    let receipt_bytes = safe_read(root, &receipt_path(id), MAX_BYTES)?;
    let receipt: serde_json::Value = parse(root, &receipt_bytes)?;
    if receipt["schemaVersion"] != 1
        || receipt["kind"] != "cross_checkout_closeout_recovery"
        || receipt["workItemId"] != id
        || receipt["destinationRepositoryId"] != repo
        || receipt.get("originBinding").is_some()
        || fs::symlink_metadata(root.join(format!(
            ".ai/decisions/{id}.cross-checkout-closeout-origin.json"
        )))
        .is_ok()
    {
        return Err(error(root, "provenance_unknown"));
    }
    let (facts, files) = super::lifecycle::legacy_source_history_inventory(root, id, repo)?;
    let mut inventory = Vec::new();
    let mut total = 0u64;
    for (path, observed) in files {
        let bytes = safe_read(root, &path, MAX_BYTES)?;
        if bytes != observed {
            return Err(error(root, "history_changed_during_observation"));
        }
        total = total
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| error(root, "inventory_budget_exceeded"))?;
        inventory.push(binding(root, path, &bytes)?);
    }
    if !inventory.iter().any(|file| file.path == receipt_path(id)) {
        total += receipt_bytes.len() as u64;
        inventory.push(binding(root, receipt_path(id), &receipt_bytes)?);
    }
    for path in scoped_dependency_paths(root, id, repo)? {
        let bytes = safe_read(root, &path, MAX_BYTES)?;
        let value = binding(root, path, &bytes)?;
        if let Some(existing) = inventory.iter().find(|entry| entry.path == value.path) {
            if existing != &value {
                return Err(error(root, "dependency_changed_during_observation"));
            }
        } else {
            total += bytes.len() as u64;
            inventory.push(value);
        }
    }
    inventory.sort_by(|a, b| a.path.cmp(&b.path));
    if inventory.is_empty()
        || inventory.len() > MAX_FILES
        || total > MAX_BYTES
        || inventory
            .windows(2)
            .any(|pair| pair[0].path == pair[1].path)
    {
        return Err(error(root, "inventory_budget_exceeded"));
    }
    Ok((facts, inventory))
}
fn git_output(root: &Path, args: &[&str]) -> Result<Vec<u8>, ObserverError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|source| ObserverError::Read {
            path: root.into(),
            source,
        })?;
    if !output.status.success() {
        return Err(error(root, "pinned_source_unavailable"));
    }
    if output.stdout.len() as u64 > MAX_BYTES {
        return Err(error(root, "pinned_source_budget_exceeded"));
    }
    Ok(output.stdout)
}
fn git_text(root: &Path, args: &[&str]) -> Result<String, ObserverError> {
    String::from_utf8(git_output(root, args)?)
        .map(|text| text.trim().into())
        .map_err(|_| error(root, "git_identity_invalid"))
}
fn common_dir(root: &Path) -> Result<PathBuf, ObserverError> {
    let raw = PathBuf::from(git_text(root, &["rev-parse", "--git-common-dir"])?);
    canonical(&if raw.is_absolute() {
        raw
    } else {
        root.join(raw)
    })
}
fn snapshot(root: &Path) -> Result<Digest, ObserverError> {
    let git = cockpit_git::GitRepository::discover(root)
        .map_err(|_| error(root, "repository_unavailable"))?;
    let snapshot = git
        .snapshot()
        .map_err(|_| error(root, "snapshot_unavailable"))?;
    snapshot_digest(&snapshot)
}
fn validate_registration(
    root: &Path,
    id: &str,
    repo: &str,
    bytes: &str,
) -> Result<Registration, ObserverError> {
    let registration: Registration = parse(root, bytes.as_bytes())?;
    if registration.schema_version != 1
        || registration.serialization_version != 1
        || registration.kind != "legacy_closeout_source_registration"
        || registration.repository_id != repo
        || registration.work_item_id != id
        || registration.requirement_path != requirement_path(id)
        || registration.assurance != "explicit_attestation"
        || registration.actor.trim().is_empty()
        || registration.authority_source.trim().is_empty()
        || registration.reason.trim().is_empty()
        || registration.runtime.runtime_version.trim().is_empty()
        || registration.runtime.protocol_version != cockpit_protocol::PROTOCOL_VERSION
        || chrono::DateTime::parse_from_rfc3339(&registration.recorded_at).is_err()
        || encode(root, &registration)? != bytes.as_bytes()
    {
        return Err(error(root, "registration_binding_invalid"));
    }
    let (facts, inventory) = observe_history(root, id, repo)?;
    if facts != registration.history_facts
        || inventory != registration.inventory
        || registration.dependency_bindings != dependencies(&inventory)
        || registration.source.inventory_digest != digest(root, &inventory)?
        || registration.legacy_receipt
            != *inventory
                .iter()
                .find(|f| f.path == receipt_path(id))
                .ok_or_else(|| error(root, "legacy_receipt_missing"))?
    {
        return Err(error(root, "registered_proof_changed"));
    }
    Ok(registration)
}
fn validate_pair(root: &Path, id: &str, repo: &str) -> Result<Registration, ObserverError> {
    let registration = safe_read(root, &registration_path(id), MAX_RECORD_BYTES)?;
    let text = String::from_utf8(registration).map_err(|_| error(root, "registration_invalid"))?;
    let parsed = validate_registration(root, id, repo, &text)?;
    let required = safe_read(root, &requirement_path(id), MAX_RECORD_BYTES)?;
    let _: Requirement = parse(root, &required)?;
    if required != requirement(root, &parsed, &text)? {
        return Err(error(root, "requirement_binding_invalid"));
    }
    Ok(parsed)
}
pub(super) fn integrity_problem(root: &Path, id: &str, repository_id: &str) -> Option<String> {
    if !has_obligation(root, id) {
        return None;
    }
    match optional_read(root, &format!(".ai/decisions/{id}{EXTENSION_SUFFIX}")) {
        Ok(None) => {}
        Ok(Some(_)) => return Some("legacy_closeout_source:extension_pending".into()),
        Err(error) => return Some(error.to_string()),
    }
    validate_pair(root, id, repository_id)
        .err()
        .map(|error| error.to_string())
}

pub(super) fn integrity_allows_close(root: &Path, id: &str, repository_id: &str) -> bool {
    integrity_problem(root, id, repository_id).is_none()
}

fn observation(
    root: &Path,
    id: &str,
    registration: &Registration,
) -> Result<Digest, ObserverError> {
    let (facts, inventory) = observe_history(root, id, &registration.repository_id)?;
    digest(
        root,
        &serde_json::json!({"snapshot":snapshot(root)?,"head":git_text(root,&["rev-parse","HEAD"])? ,"facts":facts,"inventory":inventory,"requirement":optional_read(root,&requirement_path(id))?.map(|bytes|Digest::sha256_bytes(&bytes)),"registration":optional_read(root,&registration_path(id))?.map(|bytes|Digest::sha256_bytes(&bytes))}),
    )
}
fn plan_digest(root: &Path, plan: &Plan) -> Result<Digest, ObserverError> {
    let mut value = serde_json::to_value(plan).map_err(|_| error(root, "plan_invalid"))?;
    value
        .as_object_mut()
        .ok_or_else(|| error(root, "plan_invalid"))?
        .remove("planDigest");
    digest(root, &value)
}

/// Build an exact frozen present-day observation without writing either repo.
/// The caller retains this output independently before invoking record.
pub fn plan_legacy_closeout_source(
    root: &Path,
    source: Option<&Path>,
    id: &str,
    request: &Request,
    runtime: &RuntimeContext,
) -> Result<Plan, ObserverError> {
    validate_work_item_id(id)?;
    let root = canonical(root)?;
    let repo = repository_id(&root).to_string();
    if request.schema_version != 1 {
        return Err(error(&root, "request_schema_invalid"));
    }
    let registration_bytes = match request.operation {
        Operation::Register => {
            if has_obligation(&root, id) {
                return Err(error(&root, "original_binding_required_for_resume"));
            }
            if !request.source_accepted
                || request.original_registration_bytes.is_some()
                || request.original_requirement_raw_digest.is_some()
            {
                return Err(error(&root, "source_acceptance_required"));
            }
            let source = canonical(source.ok_or_else(|| error(&root, "provenance_unknown"))?)?;
            if source == root
                || common_dir(&source)? == common_dir(&root)?
                || repository_id(&source).to_string() != repo
            {
                return Err(error(&root, "independent_source_required"));
            }
            let revision = request
                .source_revision
                .as_deref()
                .ok_or_else(|| error(&root, "pinned_source_required"))?;
            if !super::valid_git_object_id(revision)
                || git_text(&source, &["rev-parse", "HEAD"])? != revision
            {
                return Err(error(&root, "pinned_source_stale"));
            }
            let (history_facts, inventory) = observe_history(&source, id, &repo)?;
            let (destination_facts, destination_inventory) = observe_history(&root, id, &repo)?;
            if history_facts != destination_facts || inventory != destination_inventory {
                return Err(error(&root, "source_payload_mismatch"));
            }
            for file in &inventory {
                let pinned = git_output(&source, &["show", &format!("{revision}:{}", file.path)])?;
                if pinned.len() as u64 != file.bytes
                    || Digest::sha256_bytes(&pinned) != file.raw_digest
                {
                    return Err(error(&root, "pinned_source_payload_mismatch"));
                }
            }
            let text = |value: &Option<String>| {
                value
                    .as_deref()
                    .filter(|s| !s.trim().is_empty())
                    .map(str::to_owned)
                    .ok_or_else(|| error(&root, "explicit_attestation_required"))
            };
            let registration = Registration {
                schema_version: 1,
                kind: "legacy_closeout_source_registration".into(),
                serialization_version: 1,
                repository_id: repo.clone(),
                work_item_id: id.into(),
                requirement_path: requirement_path(id),
                legacy_receipt: inventory
                    .iter()
                    .find(|f| f.path == receipt_path(id))
                    .ok_or_else(|| error(&root, "legacy_receipt_missing"))?
                    .clone(),
                dependency_bindings: dependencies(&inventory),
                source: LegacyCloseoutSourceAttestation {
                    root: source.to_string_lossy().into_owned(),
                    git_common_directory: common_dir(&source)?.to_string_lossy().into_owned(),
                    revision: revision.into(),
                    snapshot_digest: snapshot(&source)?,
                    inventory_digest: digest(&root, &inventory)?,
                },
                inventory,
                history_facts,
                runtime: runtime.clone(),
                actor: text(&request.actor)?,
                authority_source: text(&request.authority_source)?,
                reason: text(&request.reason)?,
                assurance: "explicit_attestation".into(),
                recorded_at: super::now(),
            };
            String::from_utf8(encode(&root, &registration)?)
                .map_err(|_| error(&root, "serialization_failed"))?
        }
        Operation::Resume => {
            if !has_obligation(&root, id) {
                return Err(error(&root, "no_surviving_obligation"));
            }
            let original = request
                .original_registration_bytes
                .as_ref()
                .ok_or_else(|| error(&root, "original_binding_unavailable"))?;
            let registration = validate_registration(&root, id, &repo, original)?;
            let required = requirement(&root, &registration, original)?;
            if request.original_requirement_raw_digest.as_ref()
                != Some(&Digest::sha256_bytes(&required))
            {
                return Err(error(&root, "original_binding_invalid"));
            }
            original.clone()
        }
    };
    let registration = validate_registration(&root, id, &repo, &registration_bytes)?;
    let requirement_bytes =
        String::from_utf8(requirement(&root, &registration, &registration_bytes)?)
            .map_err(|_| error(&root, "serialization_failed"))?;
    for (path, expected) in [
        (registration_path(id), registration_bytes.as_bytes()),
        (requirement_path(id), requirement_bytes.as_bytes()),
    ] {
        if let Some(existing) = optional_read(&root, &path)?
            && existing != expected
        {
            return Err(error(&root, "published_record_changed"));
        }
    }
    let mut plan = Plan {
        schema_version: 1,
        kind: "legacy_closeout_source_plan".into(),
        state: "prepared".into(),
        operation: request.operation.clone(),
        repository_id: repo,
        work_item_id: id.into(),
        destination_root: root.to_string_lossy().into_owned(),
        registration_raw_digest: Digest::sha256_bytes(registration_bytes.as_bytes()),
        requirement_raw_digest: Digest::sha256_bytes(requirement_bytes.as_bytes()),
        observation_digest: observation(&root, id, &registration)?,
        registration_bytes,
        requirement_bytes,
        plan_digest: Digest::sha256_bytes(b"uninitialized"),
    };
    plan.plan_digest = plan_digest(&root, &plan)?;
    Ok(plan)
}
fn publish(root: &Path, dir: &Dir, name: &str, bytes: &[u8]) -> Result<(), ObserverError> {
    if bytes.len() as u64 > MAX_RECORD_BYTES {
        return Err(error(root, "record_budget_exceeded"));
    }
    let sequence = super::NEXT_ATOMIC_WRITE_ID.fetch_add(1, Ordering::Relaxed);
    let temporary = format!(
        ".{name}.legacy-source-tmp-{}-{sequence}",
        std::process::id()
    );
    let mut options = cap_std::fs::OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut file = dir
        .open_with(&temporary, &options)
        .map_err(|source| ObserverError::Read {
            path: root.join(&temporary),
            source,
        })?
        .into_std();
    if let Err(source) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        let _ = dir.remove_file(&temporary);
        return Err(ObserverError::Read {
            path: root.join(&temporary),
            source,
        });
    }
    drop(file);
    let installed = dir.hard_link(&temporary, dir, name);
    let _ = dir.remove_file(&temporary);
    match installed {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(source) => {
            return Err(ObserverError::Read {
                path: root.join(name),
                source,
            });
        }
    }
    #[cfg(not(windows))]
    dir.open(".")
        .and_then(|f| f.into_std().sync_all())
        .map_err(|source| ObserverError::Read {
            path: root.into(),
            source,
        })?;
    if read_cap_file_nofollow_bounded(dir, name, &root.join(name), MAX_RECORD_BYTES)? != bytes {
        return Err(error(root, "published_record_changed"));
    }
    Ok(())
}
/// Publish requirement first, then the exact frozen registration. Canonical
/// files are never unlinked on errors; partial publication remains an obligation.
pub fn record_legacy_closeout_source(
    root: &Path,
    id: &str,
    plan: &Plan,
    runtime: &RuntimeContext,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(id)?;
    let root = canonical(root)?;
    let _lock = acquire_lifecycle_lock(&root, id)?;
    let repo = repository_id(&root).to_string();
    if plan.schema_version != 1
        || plan.kind != "legacy_closeout_source_plan"
        || plan.state != "prepared"
        || plan.repository_id != repo
        || plan.work_item_id != id
        || plan.destination_root != root.to_string_lossy()
        || plan.plan_digest != plan_digest(&root, plan)?
        || plan.registration_raw_digest != Digest::sha256_bytes(plan.registration_bytes.as_bytes())
        || plan.requirement_raw_digest != Digest::sha256_bytes(plan.requirement_bytes.as_bytes())
    {
        return Err(error(&root, "plan_binding_invalid"));
    }
    let registration = validate_registration(&root, id, &repo, &plan.registration_bytes)?;
    if requirement(&root, &registration, &plan.registration_bytes)?
        != plan.requirement_bytes.as_bytes()
    {
        return Err(error(&root, "plan_binding_invalid"));
    }
    if let Ok(existing) = validate_pair(&root, id, &repo) {
        if encode(&root, &existing)? != plan.registration_bytes.as_bytes() {
            return Err(error(&root, "published_record_changed"));
        }
        return Ok(
            serde_json::json!({"state":"committed","workItemId":id,"registrationRawDigest":plan.registration_raw_digest,"requirementRawDigest":plan.requirement_raw_digest,"changedPaths":[]}),
        );
    }
    if plan.operation == Operation::Resume && !has_obligation(&root, id) {
        return Err(error(&root, "no_surviving_obligation"));
    }
    if plan.observation_digest != observation(&root, id, &registration)? {
        return Err(error(&root, "plan_stale"));
    }
    if plan.operation == Operation::Register {
        if has_obligation(&root, id) {
            return Err(error(&root, "original_binding_required_for_resume"));
        }
        let at = chrono::DateTime::parse_from_rfc3339(&registration.recorded_at)
            .map_err(|_| error(&root, "record_time_invalid"))?;
        let age = chrono::Utc::now().signed_duration_since(at);
        if registration.runtime != *runtime || age.num_seconds() < 0 || age.num_seconds() > 300 {
            return Err(error(&root, "prepared_observation_stale"));
        }
        let source = Path::new(&registration.source.root);
        if git_text(source, &["rev-parse", "HEAD"])? != registration.source.revision
            || snapshot(source)? != registration.source.snapshot_digest
            || common_dir(source)?.to_string_lossy() != registration.source.git_common_directory
        {
            return Err(error(&root, "source_stale"));
        }
        let (facts, inventory) = observe_history(source, id, &repo)?;
        if facts != registration.history_facts || inventory != registration.inventory {
            return Err(error(&root, "source_stale"));
        }
        for file in &inventory {
            if Digest::sha256_bytes(&git_output(
                source,
                &[
                    "show",
                    &format!("{}:{}", registration.source.revision, file.path),
                ],
            )?) != file.raw_digest
            {
                return Err(error(&root, "pinned_source_payload_mismatch"));
            }
        }
    }
    let root_dir =
        Dir::open_ambient_dir(&root, cap_std::ambient_authority()).map_err(|source| {
            ObserverError::Read {
                path: root.clone(),
                source,
            }
        })?;
    let ai = open_cap_directory_nofollow_strict(&root_dir, ".ai", &root.join(".ai"))?;
    let dir = open_cap_directory_nofollow_strict(&ai, "decisions", &root.join(".ai/decisions"))?;
    let mut changed = Vec::new();
    for (suffix, bytes) in [
        (REQUIREMENT_SUFFIX, plan.requirement_bytes.as_bytes()),
        (REGISTRATION_SUFFIX, plan.registration_bytes.as_bytes()),
    ] {
        let relative = format!(".ai/decisions/{id}{suffix}");
        if let Some(existing) = optional_read(&root, &relative)? {
            if existing != bytes {
                return Err(error(&root, "published_record_changed"));
            }
        } else {
            publish(
                &root.join(".ai/decisions"),
                &dir,
                &format!("{id}{suffix}"),
                bytes,
            )?;
            changed.push(relative);
        }
    }
    validate_pair(&root, id, &repo).map_err(|_| error(&root, "commit_state_unknown"))?;
    Ok(
        serde_json::json!({"state":"committed","workItemId":id,"registrationRawDigest":plan.registration_raw_digest,"requirementRawDigest":plan.requirement_raw_digest,"changedPaths":changed,"publicationObservedAt":super::now()}),
    )
}

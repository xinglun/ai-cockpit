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
    let inspect = || -> Result<bool, ObserverError> {
        let pinned = PinnedRoot::open(root)?;
        let root_dir = root_directory(root)?;
        let Some(ai) = optional_directory(&root_dir, ".ai", &root.join(".ai"))? else {
            pinned.require_current()?;
            return Ok(false);
        };
        observe_directory(&root.join(".ai"), &ai)?;
        let path = root.join(".ai/decisions");
        let Some(dir) = optional_directory(&ai, "decisions", &path)? else {
            pinned.require_current()?;
            return Ok(false);
        };
        observe_directory(&path, &dir)?;
        let mut found = false;
        for (index, entry) in dir
            .entries()
            .map_err(|source| ObserverError::Read {
                path: path.clone(),
                source,
            })?
            .enumerate()
        {
            if index >= 4096 {
                return Err(error(root, "directory_budget_exceeded"));
            }
            let name = entry
                .map_err(|source| ObserverError::Read {
                    path: path.clone(),
                    source,
                })?
                .file_name();
            if [REQUIREMENT_SUFFIX, REGISTRATION_SUFFIX, EXTENSION_SUFFIX]
                .iter()
                .any(|suffix| {
                    name.to_str()
                        .is_some_and(|name| name.eq_ignore_ascii_case(&format!("{id}{suffix}")))
                })
            {
                found = true;
                break;
            }
        }
        pinned.require_current()?;
        let current_root = pinned.directory()?;
        let current_ai =
            open_cap_directory_nofollow_strict(&current_root, ".ai", &root.join(".ai"))?;
        let current = open_cap_directory_nofollow_strict(&current_ai, "decisions", &path)?;
        if directory_identity(&path, &current)? != directory_identity(&path, &dir)? {
            return Err(error(root, "directory_identity_changed"));
        }
        Ok(found)
    };
    // Enumeration avoids cap-std's Windows missing-leaf ACCESS_DENIED quirk.
    inspect().unwrap_or(true)
}

pub(super) fn obligation_ids(root: &Path) -> Result<BTreeSet<String>, ObserverError> {
    let pinned = PinnedRoot::open(root)?;
    let path = root.join(".ai/decisions");
    let ambient = root_directory(root)?;
    let Some(ai) = optional_directory(&ambient, ".ai", &root.join(".ai"))? else {
        pinned.require_current()?;
        return Ok(BTreeSet::new());
    };
    observe_directory(&root.join(".ai"), &ai)?;
    let Some(decisions) = optional_directory(&ai, "decisions", &path)? else {
        pinned.require_current()?;
        return Ok(BTreeSet::new());
    };
    observe_directory(&path, &decisions)?;
    let entries = decisions.entries().map_err(|source| ObserverError::Read {
        path: path.clone(),
        source,
    })?;
    let mut ids = BTreeSet::new();
    for (index, entry) in entries.enumerate() {
        if index >= 4096 {
            return Err(error(root, "directory_budget_exceeded"));
        }
        let entry = entry.map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        let folded = name.to_ascii_lowercase();
        for suffix in [REQUIREMENT_SUFFIX, REGISTRATION_SUFFIX, EXTENSION_SUFFIX] {
            if let Some(prefix) = folded.strip_suffix(suffix) {
                let id = &name[..prefix.len()];
                if validate_work_item_id(id).is_ok() {
                    ids.insert(id.to_owned());
                }
            }
        }
    }
    pinned.require_current()?;
    let current_root = root_directory(root)?;
    let current_ai = open_cap_directory_nofollow_strict(&current_root, ".ai", &root.join(".ai"))?;
    let current = open_cap_directory_nofollow_strict(&current_ai, "decisions", &path)?;
    if directory_identity(&path, &current)? != directory_identity(&path, &decisions)? {
        return Err(error(root, "directory_identity_changed"));
    }
    Ok(ids)
}

use super::{acquire_lifecycle_lock, reject_duplicate_json_keys, repository_id, snapshot_digest};
use cap_fs_ext::{DirExt, FollowSymlinks, MetadataExt, OpenOptionsFollowExt};
use cockpit_core::Digest;
use cockpit_protocol::{
    LegacyCloseoutSourceAttestation, LegacyCloseoutSourceDirectoryBinding as DirectoryBinding,
    LegacyCloseoutSourceExtension as Extension, LegacyCloseoutSourceFileBinding as FileBinding,
    LegacyCloseoutSourceOperation as Operation, LegacyCloseoutSourcePlan as Plan,
    LegacyCloseoutSourceRegistration as Registration, LegacyCloseoutSourceRequest as Request,
    LegacyCloseoutSourceRequirement as Requirement, RuntimeContext, digest_json,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    io::{Read, Write},
    path::{Component, PathBuf},
    process::Command,
    sync::atomic::Ordering,
};
const MAX_BYTES: u64 = 4 * 1024 * 1024;
const MAX_RECORD_BYTES: u64 = 2 * MAX_BYTES;
const MAX_FILES: usize = 128;

/// Validate the raw MCP envelope before duplicate fields can be discarded by
/// Value deserialization. The envelope allows twice the nested record budget.
pub fn validate_legacy_closeout_source_wire(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() as u64 > 2 * MAX_RECORD_BYTES {
        return Err("legacy_closeout_source:rpc_line_budget_exceeded".into());
    }
    reject_duplicate_json_keys(bytes)
        .map_err(|detail| format!("legacy_closeout_source:duplicate_json_key: {detail}"))
}

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
/// A capability opened from the filesystem anchor, never through a target
/// root or ancestor symlink. Callers canonicalize user aliases once at entry.
pub(crate) struct PinnedRoot {
    path: PathBuf,
    dir: Dir,
    identity: [u64; 2],
}
impl PinnedRoot {
    pub(crate) fn open(path: &Path) -> Result<Self, ObserverError> {
        if !path.is_absolute() {
            return Err(error(path, "root_path_invalid"));
        }
        let mut anchor = PathBuf::new();
        let mut names = Vec::new();
        for component in path.components() {
            match component {
                Component::Prefix(_) | Component::RootDir if names.is_empty() => {
                    anchor.push(component.as_os_str())
                }
                Component::Normal(name) => names.push(name.to_os_string()),
                _ => return Err(error(path, "root_path_invalid")),
            }
        }
        let read_error = |source| ObserverError::Read {
            path: path.into(),
            source,
        };
        let mut dir =
            Dir::open_ambient_dir(&anchor, cap_std::ambient_authority()).map_err(read_error)?;
        for name in names {
            dir = dir.open_dir_nofollow(&name).map_err(read_error)?;
            let metadata = dir.dir_metadata().map_err(read_error)?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err(error(path, "root_path_invalid"));
            }
        }
        let identity = directory_identity(path, &dir)?;
        Ok(Self {
            path: path.into(),
            dir,
            identity,
        })
    }
    pub(crate) fn require_current(&self) -> Result<(), ObserverError> {
        if Self::open(&self.path)?.identity != self.identity {
            return Err(error(&self.path, "root_identity_changed"));
        }
        Ok(())
    }
    pub(crate) fn directory(&self) -> Result<Dir, ObserverError> {
        self.dir.try_clone().map_err(|source| ObserverError::Read {
            path: self.path.clone(),
            source,
        })
    }
}
pub(crate) fn directory_identity(path: &Path, dir: &Dir) -> Result<[u64; 2], ObserverError> {
    // Metadata is obtained from the open handle, also on Windows.
    let metadata = dir.dir_metadata().map_err(|source| ObserverError::Read {
        path: path.into(),
        source,
    })?;
    Ok([metadata.dev(), metadata.ino()])
}
fn root_directory(root: &Path) -> Result<Dir, ObserverError> {
    if let Some(result) = QUERY_BUDGET.with(|state| {
        let state = state.borrow();
        let pinned = state.as_ref()?.roots.get(root)?;
        Some(pinned.require_current().and_then(|()| pinned.directory()))
    }) {
        return result;
    }
    PinnedRoot::open(root)?.directory()
}
fn optional_directory(parent: &Dir, name: &str, path: &Path) -> Result<Option<Dir>, ObserverError> {
    #[cfg(windows)]
    match fs::symlink_metadata(path) {
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(ObserverError::Read {
                path: path.into(),
                source,
            });
        }
        Ok(_) => {}
    }
    match parent.open_dir_nofollow(name) {
        Ok(dir) => {
            let metadata = dir.dir_metadata().map_err(|source| ObserverError::Read {
                path: path.into(),
                source,
            })?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err(error(path, "directory_invalid"));
            }
            Ok(Some(dir))
        }
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(ObserverError::Read {
            path: path.into(),
            source,
        }),
    }
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
    if relative.len() > 4096 || relative.contains('\\') || !relative.starts_with(".ai/") {
        return Err(error(root, "path_invalid"));
    }
    let parts: Vec<String> = Path::new(relative)
        .components()
        .map(|part| match part {
            Component::Normal(name) => name
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| error(root, "path_invalid")),
            _ => Err(error(root, "path_invalid")),
        })
        .collect::<Result<_, _>>()?;
    if parts.len() > 64 || parts.join("/") != relative {
        return Err(error(root, "path_invalid"));
    }
    Ok(parts)
}
/// Open without following a link or waiting for a FIFO peer, verify the open
/// handle is a regular file, then read at most the declared budget plus one.
pub(super) fn read_regular_cap_bounded(
    parent: &Dir,
    name: &str,
    path: &Path,
    maximum: u64,
) -> Result<Vec<u8>, ObserverError> {
    let mut options = cap_std::fs::OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    #[cfg(unix)]
    {
        use cap_fs_ext::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK);
    }
    let mut file = parent
        .open_with(name, &options)
        .map_err(|source| ObserverError::Read {
            path: path.into(),
            source,
        })?;
    if !file
        .metadata()
        .map_err(|source| ObserverError::Read {
            path: path.into(),
            source,
        })?
        .is_file()
    {
        return Err(error(path, "regular_file_required"));
    }
    let mut bytes = Vec::new();
    std::io::Read::by_ref(&mut file)
        .take(maximum.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|source| ObserverError::Read {
            path: path.into(),
            source,
        })?;
    if bytes.len() as u64 > maximum {
        return Err(error(
            path,
            "record_budget_exceeded: entry exceeds the bounded read limit",
        ));
    }
    Ok(bytes)
}
/// Read every component through pinned directory handles with no-follow opens.
fn safe_read(root: &Path, relative: &str, maximum: u64) -> Result<Vec<u8>, ObserverError> {
    let parts = relative_parts(root, relative)?;
    let mut directory = root_directory(root)?;
    let mut display = root.to_path_buf();
    for part in &parts[..parts.len() - 1] {
        display.push(part);
        directory = open_cap_directory_nofollow_strict(&directory, part, &display)?;
        observe_directory(&display, &directory)?;
    }
    let leaf = parts.last().ok_or_else(|| error(root, "path_invalid"))?;
    let maximum = begin_query_read(root, maximum)?;
    let bytes = read_regular_cap_bounded(&directory, leaf, &root.join(relative), maximum)?;
    complete_query_read(root, relative, &bytes)?;
    Ok(bytes)
}
pub(super) fn optional_read(root: &Path, relative: &str) -> Result<Option<Vec<u8>>, ObserverError> {
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
        let mut dir = root_directory(root)?;
        let mut display = root.to_path_buf();
        for part in relative_parts(root, relative)? {
            display.push(&part);
            dir = open_cap_directory_nofollow_strict(&dir, &part, &display)?;
            observe_directory(&display, &dir)?;
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
fn dependencies(inventory: &[FileBinding], id: &str) -> Vec<FileBinding> {
    inventory
        .iter()
        .filter(|file| {
            ![
                format!(".ai/work-items/archive/{id}."),
                format!(".ai/evidence/{id}."),
                format!(".ai/decisions/{id}."),
            ]
            .iter()
            .any(|prefix| file.path.starts_with(prefix))
        })
        .cloned()
        .collect()
}

// Registration requires a complete producer receipt. Ordinary legacy reading
// retains its historical compatibility policy; it does not acquire this schema.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LegacyReceipt {
    schema_version: u32,
    kind: String,
    work_item_id: String,
    source_repository_id: String,
    destination_repository_id: String,
    source_snapshot_digest: Digest,
    destination_snapshot_digest: Digest,
    contract_digest: Digest,
    provider_facts: serde_json::Value,
    runtime_version: String,
    runtime_digest: Digest,
    file_digests: Vec<LegacyFile>,
    created_at: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LegacyFile {
    path: String,
    digest: Digest,
    size_bytes: u64,
}
fn validate_legacy_header(
    root: &Path,
    id: &str,
    repo: &str,
    bytes: &[u8],
) -> Result<(), ObserverError> {
    let receipt: LegacyReceipt =
        parse(root, bytes).map_err(|_| error(root, "legacy_receipt_invalid"))?;
    let mut paths = BTreeSet::new();
    if receipt.schema_version != 1
        || receipt.kind != "cross_checkout_closeout_recovery"
        || receipt.work_item_id != id
        || receipt.source_repository_id != repo
        || receipt.destination_repository_id != repo
        || receipt.runtime_version.trim().is_empty()
        || chrono::DateTime::parse_from_rfc3339(&receipt.created_at).is_err()
        || [
            &receipt.source_snapshot_digest,
            &receipt.destination_snapshot_digest,
            &receipt.contract_digest,
            &receipt.runtime_digest,
        ]
        .iter()
        .any(|value| !super::valid_sha256_digest(&value.to_string()))
        || receipt.file_digests.is_empty()
        || receipt.file_digests.len() > MAX_FILES
        || receipt.file_digests.iter().any(|file| {
            relative_parts(root, &file.path).is_err()
                || !paths.insert(file.path.clone())
                || file.size_bytes > MAX_BYTES
                || !super::valid_sha256_digest(&file.digest.to_string())
        })
    {
        return Err(error(root, "legacy_receipt_invalid"));
    }
    // The existing validator reconstructs the complete applicable payload and
    // checks canonical Contract and lawful historical/current provider facts.
    let value: serde_json::Value = parse(root, bytes)?;
    if value["providerFacts"] != receipt.provider_facts
        || !super::lifecycle::closeout_recovery_payload_matches(root, id, repo, &value)
    {
        return Err(error(root, "legacy_receipt_invalid"));
    }
    Ok(())
}

fn observe_history(
    root: &Path,
    id: &str,
    repo: &str,
) -> Result<(serde_json::Value, Vec<FileBinding>), ObserverError> {
    let _query = QuerySession::start();
    let _visit = QueryVisit::enter(root, id, "observation")?;
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
    validate_legacy_header(root, id, repo, &receipt_bytes)?;
    let (facts, files, selected_ids) =
        super::lifecycle::legacy_source_history_inventory(root, id, repo)?;
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
    let mut dependency_paths = BTreeSet::new();
    for selected in selected_ids {
        dependency_paths.extend(scoped_dependency_paths(root, &selected, repo)?);
    }
    for path in dependency_paths {
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
    // Fixed read-only local commands; never allocate an unbounded Git output
    // before applying the budget. Lazy fetch and stderr disclosure are disabled.
    let mut child = Command::new("git")
        .arg("--no-pager")
        .args(args)
        .env("GIT_NO_LAZY_FETCH", "1")
        .current_dir(root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|source| ObserverError::Read {
            path: root.into(),
            source,
        })?;
    let mut bytes = Vec::new();
    let result = match child.stdout.take() {
        Some(stdout) => stdout.take(MAX_BYTES + 1).read_to_end(&mut bytes),
        None => Err(std::io::Error::other("Git stdout missing")),
    };
    if result.is_err() || bytes.len() as u64 > MAX_BYTES {
        let _ = child.kill();
    }
    let status = child.wait().map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    result.map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(error(root, "pinned_source_budget_exceeded"));
    }
    if !status.success() {
        return Err(error(root, "pinned_source_unavailable"));
    }
    Ok(bytes)
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
fn directory_binding(root: &Path) -> Result<DirectoryBinding, ObserverError> {
    let pinned = PinnedRoot::open(root)?;
    let common = common_dir(root)?;
    let common_pinned = PinnedRoot::open(&common)?;
    pinned.require_current()?;
    Ok(DirectoryBinding {
        root_identity: pinned.identity,
        git_common_directory: common.to_string_lossy().into_owned(),
        git_common_directory_identity: common_pinned.identity,
    })
}
fn require_plan_directory(root: &Path, plan: &Plan) -> Result<(), ObserverError> {
    if plan.destination_root != root.to_string_lossy()
        || plan.destination_binding != directory_binding(root)?
        || (plan.operation == Operation::Register) != plan.source_binding.is_some()
    {
        return Err(error(root, "plan_directory_stale"));
    }
    Ok(())
}
fn snapshot(root: &Path) -> Result<Digest, ObserverError> {
    let git = cockpit_git::GitRepository::discover(root)
        .map_err(|_| error(root, "repository_unavailable"))?;
    let snapshot = git
        .snapshot()
        .map_err(|_| error(root, "snapshot_unavailable"))?;
    snapshot_digest(&snapshot)
}
fn parse_registration(
    root: &Path,
    id: &str,
    repo: &str,
    bytes: &str,
) -> Result<Registration, ObserverError> {
    let registration: Registration = parse(root, bytes.as_bytes())?;
    let mut total = 0u64;
    if registration.inventory.is_empty()
        || registration.inventory.len() > MAX_FILES
        || registration
            .inventory
            .windows(2)
            .any(|pair| pair[0].path >= pair[1].path)
        || registration.inventory.iter().any(|file| {
            total = total.saturating_add(file.bytes);
            relative_parts(root, &file.path).is_err()
                || file.bytes > MAX_BYTES
                || !super::valid_sha256_digest(file.raw_digest.as_str())
                || file
                    .canonical_digest
                    .as_ref()
                    .is_some_and(|digest| !super::valid_sha256_digest(digest.as_str()))
        })
        || total > MAX_BYTES
    {
        return Err(error(root, "inventory_binding_invalid"));
    }
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
        || !super::valid_sha256_digest(registration.runtime.runtime_digest.as_str())
        || !super::valid_sha256_digest(registration.source.snapshot_digest.as_str())
        || !Path::new(&registration.source.root).is_absolute()
        || !Path::new(&registration.source.git_common_directory).is_absolute()
        || !super::valid_git_object_id(&registration.source.revision)
        || chrono::DateTime::parse_from_rfc3339(&registration.recorded_at).is_err()
        || encode(root, &registration)? != bytes.as_bytes()
    {
        return Err(error(root, "registration_binding_invalid"));
    }
    if registration.dependency_bindings != dependencies(&registration.inventory, id)
        || registration.source.inventory_digest != digest(root, &registration.inventory)?
        || registration.legacy_receipt
            != *registration
                .inventory
                .iter()
                .find(|f| f.path == receipt_path(id))
                .ok_or_else(|| error(root, "legacy_receipt_missing"))?
        || registration
            .runtime
            .runtime_digest
            .as_str()
            .parse::<Digest>()
            .is_err()
    {
        return Err(error(root, "registration_binding_invalid"));
    }
    Ok(registration)
}
fn validate_registration(
    root: &Path,
    id: &str,
    repo: &str,
    bytes: &str,
) -> Result<Registration, ObserverError> {
    let registration = parse_registration(root, id, repo, bytes)?;
    if let Some(extension) = optional_read(root, &extension_path(id))? {
        validate_extension(root, id, &registration, bytes.as_bytes(), &extension, true)?;
    } else {
        let (facts, inventory) = observe_history(root, id, repo)?;
        if facts != registration.history_facts || inventory != registration.inventory {
            return Err(error(root, "registered_proof_changed"));
        }
    }
    Ok(registration)
}
/// Repair only the original root pair. New applicable decisions are observed
/// for freshness but gain no registration authority from this operation.
fn validate_root_repair(
    root: &Path,
    id: &str,
    repo: &str,
    bytes: &str,
) -> Result<Registration, ObserverError> {
    let registration = parse_registration(root, id, repo, bytes)?;
    observe_history(root, id, repo)?;
    for file in &registration.inventory {
        let raw = safe_read(root, &file.path, MAX_BYTES)?;
        if binding(root, file.path.clone(), &raw)? != *file {
            return Err(error(root, "registered_proof_changed"));
        }
    }
    let required = requirement(root, &registration, bytes)?;
    let mut root_survivor = false;
    for (path, expected) in [
        (requirement_path(id), required.as_slice()),
        (registration_path(id), bytes.as_bytes()),
    ] {
        if let Some(existing) = optional_read(root, &path)? {
            if existing != expected {
                return Err(error(root, "published_record_changed"));
            }
            root_survivor = true;
        }
    }
    if !root_survivor {
        // An extension can be an independent root anchor only when its strict
        // template really binds these original registration bytes. A transition
        // by itself does not remember the registration and is insufficient.
        let extension = optional_read(root, &extension_path(id))?
            .ok_or_else(|| error(root, "original_root_binding_unavailable"))?;
        validate_extension(root, id, &registration, bytes.as_bytes(), &extension, false)
            .map_err(|_| error(root, "original_root_binding_unavailable"))?;
    }
    Ok(registration)
}
fn validate_root_plan(
    root: &Path,
    id: &str,
    repo: &str,
    bytes: &str,
    operation: &Operation,
) -> Result<Registration, ObserverError> {
    if *operation == Operation::Resume {
        validate_root_repair(root, id, repo, bytes)
    } else {
        validate_registration(root, id, repo, bytes)
    }
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
    let _query = QuerySession::start();
    let _visit = match QueryVisit::enter(root, id, "registration") {
        Ok(visit) => visit,
        Err(error) => return Some(error.to_string()),
    };
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
        &serde_json::json!({"directory":directory_binding(root)?,"snapshot":snapshot(root)?,"head":git_text(root,&["rev-parse","HEAD"])? ,"facts":facts,"inventory":inventory,"requirement":optional_read(root,&requirement_path(id))?.map(|bytes|Digest::sha256_bytes(&bytes)),"registration":optional_read(root,&registration_path(id))?.map(|bytes|Digest::sha256_bytes(&bytes)),"extension":optional_read(root,&extension_path(id))?.map(|bytes|Digest::sha256_bytes(&bytes))}),
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
fn seal_plan(root: &Path, mut plan: Plan) -> Result<Plan, ObserverError> {
    plan.plan_digest = plan_digest(root, &plan)?;
    if encode(root, &plan)?.len() as u64 > MAX_RECORD_BYTES {
        return Err(error(root, "record_budget_exceeded"));
    }
    require_plan_directory(root, &plan)?;
    Ok(plan)
}
fn validate_request_role(
    root: &Path,
    source: Option<&Path>,
    request: &Request,
) -> Result<(), ObserverError> {
    let source_fields =
        source.is_some() || request.source_revision.is_some() || request.source_accepted;
    let attestation =
        request.actor.is_some() || request.authority_source.is_some() || request.reason.is_some();
    let root_resume = request.original_registration_bytes.is_some()
        || request.original_requirement_raw_digest.is_some();
    let extension_resume = request.original_extension_bytes.is_some()
        || request.original_extension_raw_digest.is_some();
    let invalid = match request.operation {
        Operation::Register => {
            root_resume || extension_resume || request.transition_bytes.is_some()
        }
        Operation::Resume => {
            source_fields || attestation || extension_resume || request.transition_bytes.is_some()
        }
        Operation::Extend => source_fields || root_resume || extension_resume,
        Operation::ResumeExtension => {
            source_fields || attestation || root_resume || request.transition_bytes.is_some()
        }
    };
    if invalid {
        Err(error(root, "request_role_invalid"))
    } else {
        Ok(())
    }
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
    if source.is_some_and(|path| !path.is_absolute()) {
        return Err(ObserverError::State {
            path: root.into(),
            message: "sourceRepo must be an absolute path".into(),
        });
    }
    let root = canonical(root)?;
    let _query = QuerySession::start();
    let _visit = QueryVisit::enter(&root, id, "plan")?;
    let destination_binding = directory_binding(&root)?;
    let repo = repository_id(&root).to_string();
    if request.schema_version != 1 {
        return Err(error(&root, "request_schema_invalid"));
    }
    validate_request_role(&root, source, request)?;
    if matches!(
        request.operation,
        Operation::Extend | Operation::ResumeExtension
    ) {
        return plan_extension(&root, id, request, runtime);
    }
    let mut source_binding = None;
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
            let _source_visit = QueryVisit::enter(&source, id, "source")?;
            source_binding = Some(directory_binding(&source)?);
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
            if source_binding.as_ref() != Some(&directory_binding(&source)?) {
                return Err(error(&root, "source_stale"));
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
                dependency_bindings: dependencies(&inventory, id),
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
            let registration = validate_root_repair(&root, id, &repo, original)?;
            let required = requirement(&root, &registration, original)?;
            if request.original_requirement_raw_digest.as_ref()
                != Some(&Digest::sha256_bytes(&required))
            {
                return Err(error(&root, "original_binding_invalid"));
            }
            original.clone()
        }
        Operation::Extend | Operation::ResumeExtension => unreachable!("handled above"),
    };
    let registration =
        validate_root_plan(&root, id, &repo, &registration_bytes, &request.operation)?;
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
    let plan = Plan {
        schema_version: 1,
        kind: "legacy_closeout_source_plan".into(),
        state: "prepared".into(),
        operation: request.operation.clone(),
        repository_id: repo,
        work_item_id: id.into(),
        destination_root: root.to_string_lossy().into_owned(),
        destination_binding,
        source_binding,
        registration_raw_digest: Digest::sha256_bytes(registration_bytes.as_bytes()),
        requirement_raw_digest: Digest::sha256_bytes(requirement_bytes.as_bytes()),
        observation_digest: observation(&root, id, &registration)?,
        registration_bytes,
        requirement_bytes,
        plan_digest: Digest::sha256_bytes(b"uninitialized"),
        extension_bytes: None,
        extension_raw_digest: None,
    };
    seal_plan(&root, plan)
}
fn require_publication_parent(
    root: &Path,
    id: &str,
    guard: &super::LifecycleLockGuard,
    dir: &Dir,
) -> Result<(), ObserverError> {
    guard.require_subject(root, id)?;
    let root_dir = root_directory(root)?;
    let ai = open_cap_directory_nofollow_strict(&root_dir, ".ai", &root.join(".ai"))?;
    observe_directory(&root.join(".ai"), &ai)?;
    let path = root.join(".ai/decisions");
    let current = open_cap_directory_nofollow_strict(&ai, "decisions", &path)?;
    observe_directory(&path, &current)?;
    if directory_identity(&path, &current)? != directory_identity(&path, dir)? {
        return Err(error(root, "publication_parent_changed"));
    }
    Ok(())
}
fn require_publication_binding(root: &Path, id: &str, plan: &Plan) -> Result<(), ObserverError> {
    require_plan_directory(root, plan)?;
    if plan.operation == Operation::Register {
        let registration =
            parse_registration(root, id, &plan.repository_id, &plan.registration_bytes)?;
        let source = Path::new(&registration.source.root);
        let _visit = QueryVisit::enter(source, id, "source_publication")?;
        if plan.source_binding.as_ref() != Some(&directory_binding(source)?)
            || git_text(source, &["rev-parse", "HEAD"])? != registration.source.revision
            || snapshot(source)? != registration.source.snapshot_digest
        {
            return Err(error(root, "source_stale"));
        }
        let (facts, inventory) = observe_history(source, id, &registration.repository_id)?;
        if facts != registration.history_facts || inventory != registration.inventory {
            return Err(error(root, "source_stale"));
        }
    }
    Ok(())
}
fn publish(
    root: &Path,
    id: &str,
    guard: &super::LifecycleLockGuard,
    plan: &Plan,
    dir: &Dir,
    name: &str,
    bytes: &[u8],
) -> Result<(), ObserverError> {
    require_publication_parent(root, id, guard, dir)?;
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
    #[cfg(test)]
    publication_test_point(root, name, "temporary_created");
    if let Err(source) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        let _ = dir.remove_file(&temporary);
        return Err(ObserverError::Read {
            path: root.join(&temporary),
            source,
        });
    }
    #[cfg(test)]
    publication_test_point(root, name, "file_synced");
    drop(file);
    if let Err(error) = require_publication_parent(root, id, guard, dir) {
        let _ = dir.remove_file(&temporary);
        return Err(error);
    }
    if let Err(error) = require_publication_binding(root, id, plan) {
        let _ = dir.remove_file(&temporary);
        return Err(error);
    }
    if let Err(error) = require_publication_parent(root, id, guard, dir) {
        let _ = dir.remove_file(&temporary);
        return Err(error);
    }
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
    let confirmation = (|| -> Result<(), ObserverError> {
        #[cfg(test)]
        publication_test_point(root, name, "canonical_linked");
        require_publication_parent(root, id, guard, dir)?;
        #[cfg(test)]
        publication_test_io(root, name, "parent_sync")?;
        #[cfg(not(windows))]
        dir.open(".")
            .and_then(|f| f.into_std().sync_all())
            .map_err(|source| ObserverError::Read {
                path: root.into(),
                source,
            })?;
        #[cfg(test)]
        publication_test_point(root, name, "parent_sync_step_completed");
        #[cfg(test)]
        publication_test_io(root, name, "readback")?;
        if read_regular_cap_bounded(dir, name, &root.join(name), MAX_RECORD_BYTES)? != bytes {
            return Err(error(root, "published_record_changed"));
        }
        #[cfg(test)]
        publication_test_point(root, name, "read_back");
        #[cfg(test)]
        publication_test_io(root, name, "report")?;
        require_publication_parent(root, id, guard, dir)?;
        Ok(())
    })();
    confirmation.map_err(|cause| {
        error(
            root,
            &format!("commit_state_unknown:publication_confirmation:{cause}"),
        )
    })
}

// Compiled only into the test harness. Production binaries expose no fault
// injection environment switch or alternate publication path.
#[cfg(test)]
fn publication_test_point(root: &Path, name: &str, phase: &str) {
    if std::env::var("COCKPIT_TEST_LEGACY_SOURCE_FAULT")
        .ok()
        .as_deref()
        != Some(&format!("{name}:{phase}"))
    {
        return;
    }
    if std::env::var("COCKPIT_TEST_LEGACY_SOURCE_FAULT_ACTION")
        .ok()
        .as_deref()
        == Some("replace_parent")
    {
        fs::rename(root.join(".ai/decisions"), root.join("retained-decisions")).unwrap();
        fs::create_dir(root.join(".ai/decisions")).unwrap();
    } else {
        std::process::exit(86);
    }
}
#[cfg(test)]
fn publication_test_io(root: &Path, name: &str, phase: &str) -> Result<(), ObserverError> {
    if std::env::var("COCKPIT_TEST_LEGACY_SOURCE_FAULT_ACTION")
        .ok()
        .as_deref()
        == Some("io_failure")
        && std::env::var("COCKPIT_TEST_LEGACY_SOURCE_FAULT")
            .ok()
            .as_deref()
            == Some(&format!("{name}:{phase}"))
    {
        return Err(ObserverError::Read {
            path: root.join(name),
            source: std::io::Error::other(format!("injected_{phase}_failure")),
        });
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
    require_plan_directory(&root, plan)?;
    let _query = QuerySession::start();
    let _visit = QueryVisit::enter(&root, id, "root_write")?;
    let guard = acquire_lifecycle_lock(&root, id)?;
    require_plan_directory(&root, plan)?;
    if matches!(
        plan.operation,
        Operation::Extend | Operation::ResumeExtension
    ) {
        return record_extension_locked(&root, id, plan, runtime, &guard);
    }
    if plan.extension_bytes.is_some() || plan.extension_raw_digest.is_some() {
        return Err(error(&root, "plan_binding_invalid"));
    }
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
    let registration =
        validate_root_plan(&root, id, &repo, &plan.registration_bytes, &plan.operation)?;
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
        let _source_visit = QueryVisit::enter(source, id, "source")?;
        if plan.source_binding.as_ref() != Some(&directory_binding(source)?) {
            return Err(error(&root, "source_stale"));
        }
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
    let root_dir = root_directory(&root)?;
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
                &root,
                id,
                &guard,
                plan,
                &dir,
                &format!("{id}{suffix}"),
                bytes,
            )?;
            changed.push(relative);
        }
    }
    let readback = validate_root_plan(&root, id, &repo, &plan.registration_bytes, &plan.operation)
        .map_err(|cause| error(&root, &format!("commit_state_unknown:{cause}")))?;
    let (_, actual_registration, actual_requirement) = root_pair(&root, id)
        .map_err(|cause| error(&root, &format!("commit_state_unknown:{cause}")))?;
    require_plan_directory(&root, plan)
        .map_err(|cause| error(&root, &format!("commit_state_unknown:{cause}")))?;
    if encode(&root, &readback)? != plan.registration_bytes.as_bytes()
        || actual_registration != plan.registration_bytes
        || actual_requirement != plan.requirement_bytes
    {
        return Err(error(&root, "commit_state_unknown"));
    }
    if let Some(problem) = integrity_problem(&root, id, &repo) {
        return Ok(
            serde_json::json!({"state":"pending","rootState":"committed","workItemId":id,"registrationRawDigest":plan.registration_raw_digest,"requirementRawDigest":plan.requirement_raw_digest,"changedPaths":changed,"integrityProblem":problem,"publicationObservedAt":super::now()}),
        );
    }
    Ok(
        serde_json::json!({"state":"committed","workItemId":id,"registrationRawDigest":plan.registration_raw_digest,"requirementRawDigest":plan.requirement_raw_digest,"changedPaths":changed,"publicationObservedAt":super::now()}),
    )
}

fn extension_path(id: &str) -> String {
    format!(".ai/decisions/{id}{EXTENSION_SUFFIX}")
}

/// Bound every caller input before parsing and reject duplicate keys. No parent
/// component or leaf is followed through a symlink.
pub fn read_legacy_closeout_source_input(path: &Path) -> Result<serde_json::Value, ObserverError> {
    parse(path, &read_input_bytes(path)?)
}
pub(crate) fn read_input_bytes(path: &Path) -> Result<Vec<u8>, ObserverError> {
    // Caller-selected inputs may be outside the repository. Resolve the parent
    // once (including platform temp aliases), then pin it and reject leaf links.
    // Repository evidence uses safe_read instead and never resolves symlinks.
    let parent = canonical(
        path.parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new(".")),
    )?;
    let pinned = PinnedRoot::open(&parent)?;
    let dir = pinned.directory()?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| error(path, "path_invalid"))?;
    let bytes = read_regular_cap_bounded(&dir, name, path, MAX_RECORD_BYTES)?;
    pinned.require_current()?;
    Ok(bytes)
}

fn root_pair(root: &Path, id: &str) -> Result<(Registration, String, String), ObserverError> {
    let repo = repository_id(root).to_string();
    let bytes = String::from_utf8(safe_read(root, &registration_path(id), MAX_RECORD_BYTES)?)
        .map_err(|_| error(root, "registration_invalid"))?;
    let registration = parse_registration(root, id, &repo, &bytes)?;
    let required = String::from_utf8(safe_read(root, &requirement_path(id), MAX_RECORD_BYTES)?)
        .map_err(|_| error(root, "requirement_invalid"))?;
    if required.as_bytes() != requirement(root, &registration, &bytes)? {
        return Err(error(root, "requirement_binding_invalid"));
    }
    Ok((registration, bytes, required))
}
fn transition_value(root: &Path, bytes: &str) -> Result<serde_json::Value, ObserverError> {
    let typed: cockpit_protocol::ResourceFinalizationTransitionReceipt =
        parse(root, bytes.as_bytes())?;
    serde_json::to_value(typed).map_err(|_| error(root, "transition_invalid"))
}
fn extension_result(
    root: &Path,
    extension: &Extension,
) -> Result<serde_json::Value, ObserverError> {
    let value = transition_value(root, &extension.transition_bytes)?;
    Ok(
        serde_json::json!({"workItemId": extension.work_item_id, "state":"appended", "sequence":value["sequence"], "disposition":value["receipt"]["result"]["disposition"], "predecessorPath":extension.predecessor.path,"path":extension.transition.path}),
    )
}
fn validate_extension(
    root: &Path,
    id: &str,
    registration: &Registration,
    registration_bytes: &[u8],
    bytes: &[u8],
    require_transition: bool,
) -> Result<Extension, ObserverError> {
    let extension: Extension = parse(root, bytes)?;
    let value = transition_value(root, &extension.transition_bytes)?;
    let canonical_digest = digest(root, &value)?;
    let transition_path = format!(
        ".ai/decisions/{id}.finalize.{}.json",
        canonical_digest.as_str().trim_start_matches("sha256:")
    );
    let expected_binding = binding(root, transition_path, extension.transition_bytes.as_bytes())?;
    if extension.schema_version != 1
        || extension.serialization_version != 1
        || extension.kind != "legacy_closeout_source_extension"
        || extension.repository_id != registration.repository_id
        || extension.work_item_id != id
        || extension.registration_raw_digest != Digest::sha256_bytes(registration_bytes)
        || extension.legacy_receipt_raw_digest != registration.legacy_receipt.raw_digest
        || extension.transition != expected_binding
        || extension.predecessor_sequence.checked_add(1) != value["sequence"].as_u64()
        || registration.history_facts["finalizationSequence"].as_u64()
            != Some(extension.predecessor_sequence)
        || registration.history_facts["finalizationHeadPath"] != extension.predecessor.path
        || registration.history_facts["finalizationHeadDigest"] != value["predecessorReceiptDigest"]
        || !registration.inventory.contains(&extension.predecessor)
        || extension.actor.trim().is_empty()
        || extension.authority_source.trim().is_empty()
        || extension.reason.trim().is_empty()
        || extension.runtime.runtime_version.trim().is_empty()
        || extension.runtime.protocol_version != cockpit_protocol::PROTOCOL_VERSION
        || !super::valid_sha256_digest(extension.runtime.runtime_digest.as_str())
        || extension
            .runtime
            .runtime_digest
            .as_str()
            .parse::<Digest>()
            .is_err()
        || chrono::DateTime::parse_from_rfc3339(&extension.recorded_at).is_err()
        || encode(root, &extension)? != bytes
        || extension.result != extension_result(root, &extension)?
        || value["receipt"]["result"]["disposition"] != "deleted"
    {
        return Err(error(root, "extension_binding_invalid"));
    }
    let mut expected = registration.inventory.clone();
    let allowed_dependencies = scoped_dependency_paths(root, id, &registration.repository_id)?;
    for file in &extension.dependency_bindings {
        if !allowed_dependencies.contains(&file.path)
            || expected.iter().any(|old| old.path == file.path)
        {
            return Err(error(root, "extension_dependency_invalid"));
        }
        expected.push(file.clone());
    }
    let present = optional_read(root, &extension.transition.path)?;
    if let Some(raw) = &present {
        if raw != extension.transition_bytes.as_bytes() {
            return Err(error(root, "published_transition_changed"));
        }
        expected.push(extension.transition.clone());
    } else if require_transition {
        return Err(error(root, "extension_pending"));
    }
    expected.sort_by(|a, b| a.path.cmp(&b.path));
    if expected.len() > MAX_FILES
        || expected
            .windows(2)
            .any(|files| files[0].path == files[1].path)
    {
        return Err(error(root, "inventory_budget_exceeded"));
    }
    let (mut facts, observed) = observe_history(root, id, &registration.repository_id)?;
    if present.is_some() {
        if facts["finalizationSequence"] != value["sequence"]
            || facts["finalizationHeadPath"] != extension.transition.path
            || facts["finalizationHeadDigest"]
                != serde_json::json!(digest(root, &value["receipt"])?)
        {
            return Err(error(root, "extension_head_invalid"));
        }
        for name in [
            "finalizationSequence",
            "finalizationHeadPath",
            "finalizationHeadDigest",
        ] {
            facts[name] = registration.history_facts[name].clone();
        }
    }
    if expected != observed || facts != registration.history_facts {
        return Err(error(root, "registered_proof_changed"));
    }
    Ok(extension)
}
fn extension_plan_identity(
    root: &Path,
    id: &str,
    plan: &Plan,
) -> Result<(Registration, Extension), ObserverError> {
    require_plan_directory(root, plan)?;
    let (registration, bytes, required) = root_pair(root, id)?;
    if plan.schema_version != 1
        || plan.kind != "legacy_closeout_source_plan"
        || plan.state != "prepared"
        || !matches!(
            plan.operation,
            Operation::Extend | Operation::ResumeExtension
        )
        || plan.repository_id != registration.repository_id
        || plan.work_item_id != id
        || plan.destination_root != root.to_string_lossy()
        || plan.registration_bytes != bytes
        || plan.requirement_bytes != required
        || plan.registration_raw_digest != Digest::sha256_bytes(bytes.as_bytes())
        || plan.requirement_raw_digest != Digest::sha256_bytes(required.as_bytes())
        || plan.plan_digest != plan_digest(root, plan)?
    {
        return Err(error(root, "plan_binding_invalid"));
    }
    let extension_bytes = plan
        .extension_bytes
        .as_ref()
        .ok_or_else(|| error(root, "extension_plan_required"))?;
    if plan.extension_raw_digest.as_ref() != Some(&Digest::sha256_bytes(extension_bytes.as_bytes()))
    {
        return Err(error(root, "plan_binding_invalid"));
    }
    let extension = validate_extension(
        root,
        id,
        &registration,
        bytes.as_bytes(),
        extension_bytes.as_bytes(),
        false,
    )?;
    Ok((registration, extension))
}
fn plan_extension(
    root: &Path,
    id: &str,
    request: &Request,
    runtime: &RuntimeContext,
) -> Result<Plan, ObserverError> {
    let _query = QuerySession::start();
    let _visit = QueryVisit::enter(root, id, "extension_plan")?;
    if request.source_accepted
        || request.source_revision.is_some()
        || request.original_registration_bytes.is_some()
        || request.original_requirement_raw_digest.is_some()
    {
        return Err(error(root, "request_role_invalid"));
    }
    let (registration, registration_bytes, requirement_bytes) = root_pair(root, id)?;
    let extension_bytes = if request.operation == Operation::ResumeExtension {
        let original = request
            .original_extension_bytes
            .as_ref()
            .ok_or_else(|| error(root, "original_binding_unavailable"))?;
        let extension = validate_extension(
            root,
            id,
            &registration,
            registration_bytes.as_bytes(),
            original.as_bytes(),
            false,
        )?;
        if request.original_extension_raw_digest.as_ref()
            != Some(&Digest::sha256_bytes(original.as_bytes()))
        {
            return Err(error(root, "original_binding_invalid"));
        }
        if optional_read(root, &extension_path(id))?.is_none()
            && optional_read(root, &extension.transition.path)?.is_none()
        {
            return Err(error(root, "no_surviving_extension_obligation"));
        }
        original.clone()
    } else {
        if optional_read(root, &extension_path(id))?.is_some() {
            return Err(error(root, "original_binding_required_for_resume"));
        }
        if request.original_extension_bytes.is_some()
            || request.original_extension_raw_digest.is_some()
        {
            return Err(error(root, "request_role_invalid"));
        }
        let transition_bytes = request
            .transition_bytes
            .as_ref()
            .ok_or_else(|| error(root, "transition_required"))?;
        let value = transition_value(root, transition_bytes)?;
        let prepared = super::resource_lifecycle::prepare_resource_finalization(
            root,
            id,
            &value,
            &root.join(".ai/decisions"),
            runtime,
        )?;
        if prepared.value.is_none() {
            return Err(error(root, "original_binding_required_for_resume"));
        }
        let relative = super::repository_relative_path(root, &prepared.path);
        let (_, inventory) = observe_history(root, id, &registration.repository_id)?;
        let added = inventory
            .iter()
            .filter(|file| !registration.inventory.contains(file))
            .cloned()
            .collect::<Vec<_>>();
        let predecessor_path = registration.history_facts["finalizationHeadPath"]
            .as_str()
            .ok_or_else(|| error(root, "retained_predecessor_required"))?;
        let text = |value: &Option<String>| {
            value
                .as_deref()
                .filter(|v| !v.trim().is_empty())
                .map(str::to_owned)
                .ok_or_else(|| error(root, "explicit_attestation_required"))
        };
        let mut extension = Extension {
            schema_version: 1,
            kind: "legacy_closeout_source_extension".into(),
            serialization_version: 1,
            repository_id: registration.repository_id.clone(),
            work_item_id: id.into(),
            registration_raw_digest: Digest::sha256_bytes(registration_bytes.as_bytes()),
            legacy_receipt_raw_digest: registration.legacy_receipt.raw_digest.clone(),
            predecessor: registration
                .inventory
                .iter()
                .find(|file| file.path == predecessor_path)
                .ok_or_else(|| error(root, "retained_predecessor_required"))?
                .clone(),
            predecessor_sequence: registration.history_facts["finalizationSequence"]
                .as_u64()
                .ok_or_else(|| error(root, "retained_predecessor_required"))?,
            transition: binding(root, relative, transition_bytes.as_bytes())?,
            transition_bytes: transition_bytes.clone(),
            dependency_bindings: added,
            runtime: runtime.clone(),
            actor: text(&request.actor)?,
            authority_source: text(&request.authority_source)?,
            reason: text(&request.reason)?,
            recorded_at: super::now(),
            result: serde_json::Value::Null,
        };
        extension.result = extension_result(root, &extension)?;
        String::from_utf8(encode(root, &extension)?)
            .map_err(|_| error(root, "serialization_failed"))?
    };
    let extension = validate_extension(
        root,
        id,
        &registration,
        registration_bytes.as_bytes(),
        extension_bytes.as_bytes(),
        false,
    )?;
    if let Some(existing) = optional_read(root, &extension_path(id))?
        && existing != extension_bytes.as_bytes()
    {
        return Err(error(root, "published_record_changed"));
    }
    if request.operation == Operation::Extend
        && optional_read(root, &extension.transition.path)?.is_some()
    {
        return Err(error(root, "original_binding_required_for_resume"));
    }
    let plan = Plan {
        schema_version: 1,
        kind: "legacy_closeout_source_plan".into(),
        state: "prepared".into(),
        operation: request.operation.clone(),
        repository_id: registration.repository_id.clone(),
        work_item_id: id.into(),
        destination_root: root.to_string_lossy().into_owned(),
        destination_binding: directory_binding(root)?,
        source_binding: None,
        registration_raw_digest: Digest::sha256_bytes(registration_bytes.as_bytes()),
        requirement_raw_digest: Digest::sha256_bytes(requirement_bytes.as_bytes()),
        registration_bytes,
        requirement_bytes,
        observation_digest: observation(root, id, &registration)?,
        plan_digest: Digest::sha256_bytes(b"uninitialized"),
        extension_raw_digest: Some(Digest::sha256_bytes(extension_bytes.as_bytes())),
        extension_bytes: Some(extension_bytes),
    };
    seal_plan(root, plan)
}
/// Sole registered extension publisher; both public transports arrive holding
/// the same unforgeable repository/Work Item lifecycle guard.
fn record_extension_locked(
    root: &Path,
    id: &str,
    plan: &Plan,
    runtime: &RuntimeContext,
    guard: &super::LifecycleLockGuard,
) -> Result<serde_json::Value, ObserverError> {
    guard.require_subject(root, id)?;
    let _query = QuerySession::start();
    let _visit = QueryVisit::enter(root, id, "extension_write")?;
    let (registration, extension) = extension_plan_identity(root, id, plan)?;
    let frozen = plan
        .extension_bytes
        .as_ref()
        .ok_or_else(|| error(root, "extension_plan_required"))?;
    let existing_extension = optional_read(root, &extension_path(id))?;
    let existing_transition = optional_read(root, &extension.transition.path)?;
    if existing_extension
        .as_deref()
        .is_some_and(|bytes| bytes != frozen.as_bytes())
    {
        return Err(error(root, "published_record_changed"));
    }
    // Recognition of the exact original publication precedes next-head/sequence
    // resolution, freshness and current Runtime checks. No bytes are regenerated.
    if existing_extension.is_some() && existing_transition.is_some() {
        validate_pair(root, id, &registration.repository_id)?;
        return Ok(extension.result);
    }
    if plan.operation == Operation::Extend {
        if existing_extension.is_some() || existing_transition.is_some() {
            return Err(error(root, "original_binding_required_for_resume"));
        }
        let at = chrono::DateTime::parse_from_rfc3339(&extension.recorded_at)
            .map_err(|_| error(root, "record_time_invalid"))?;
        let age = chrono::Utc::now().signed_duration_since(at);
        if extension.runtime != *runtime || age.num_seconds() < 0 || age.num_seconds() > 300 {
            return Err(error(root, "prepared_observation_stale"));
        }
    } else if existing_extension.is_none() && existing_transition.is_none() {
        return Err(error(root, "no_surviving_extension_obligation"));
    }
    if plan.observation_digest != observation(root, id, &registration)? {
        return Err(error(root, "plan_stale"));
    }
    if existing_transition.is_none() {
        let value = transition_value(root, &extension.transition_bytes)?;
        let prepared = super::resource_lifecycle::prepare_resource_finalization(
            root,
            id,
            &value,
            &root.join(".ai/decisions"),
            runtime,
        )?;
        if prepared.value.is_none() || prepared.result != extension.result {
            return Err(error(root, "extension_binding_invalid"));
        }
    }
    let root_dir = root_directory(root)?;
    let ai = open_cap_directory_nofollow_strict(&root_dir, ".ai", &root.join(".ai"))?;
    let dir = open_cap_directory_nofollow_strict(&ai, "decisions", &root.join(".ai/decisions"))?;
    for (path, bytes) in [
        (extension_path(id), frozen.as_bytes()),
        (
            extension.transition.path.clone(),
            extension.transition_bytes.as_bytes(),
        ),
    ] {
        let name = Path::new(&path)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| error(root, "path_invalid"))?;
        publish(root, id, guard, plan, &dir, name, bytes)?;
    }
    let readback = validate_pair(root, id, &registration.repository_id)
        .map_err(|cause| error(root, &format!("commit_state_unknown:{cause}")))?;
    require_plan_directory(root, plan)
        .map_err(|cause| error(root, &format!("commit_state_unknown:{cause}")))?;
    if encode(root, &readback)? != plan.registration_bytes.as_bytes()
        || safe_read(root, &extension_path(id), MAX_RECORD_BYTES)? != frozen.as_bytes()
        || safe_read(root, &extension.transition.path, MAX_RECORD_BYTES)?
            != extension.transition_bytes.as_bytes()
    {
        return Err(error(root, "commit_state_unknown"));
    }
    Ok(extension.result)
}

pub(crate) fn record_finalization_locked(
    root: &Path,
    id: &str,
    input: &serde_json::Value,
    input_bytes: &[u8],
    runtime: &RuntimeContext,
    guard: &super::LifecycleLockGuard,
) -> Result<serde_json::Value, ObserverError> {
    guard.require_subject(root, id)?;
    let _query = QuerySession::start();
    let _visit = QueryVisit::enter(root, id, "finalization_write")?;
    if input["kind"] == "legacy_closeout_source_plan" {
        let plan: Plan = serde_json::from_value(input.clone())
            .map_err(|_| error(root, "strict_schema_invalid"))?;
        return record_extension_locked(root, id, &plan, runtime, guard);
    }
    let (registration, bytes, _) = root_pair(root, id)?;
    if let Some(frozen) = optional_read(root, &extension_path(id))? {
        let extension =
            validate_extension(root, id, &registration, bytes.as_bytes(), &frozen, true)?;
        let requested: cockpit_protocol::ResourceFinalizationTransitionReceipt =
            serde_json::from_value(input.clone())
                .map_err(|_| error(root, "extension_plan_required"))?;
        if serde_json::to_value(requested).map_err(|_| error(root, "transition_invalid"))?
            == transition_value(root, &extension.transition_bytes)?
        {
            if input_bytes != extension.transition_bytes.as_bytes() {
                return Err(error(root, "published_transition_changed"));
            }
            validate_pair(root, id, &registration.repository_id)?;
            return Ok(extension.result);
        }
    }
    Err(error(root, "extension_plan_required"))
}

// A per-thread, per-query budget; never held borrowed while invoking lower
// historical validators. No cache of successful external bytes is trusted
// across reads. Repeated work is explicitly bounded as well as recursion.
#[derive(Default)]
struct QueryBudget {
    active: Vec<(PathBuf, String, &'static str)>,
    subjects: BTreeSet<(PathBuf, String)>,
    work: usize,
    read_calls: usize,
    read_bytes: u64,
    observed: std::collections::BTreeMap<PathBuf, (Digest, u64)>,
    roots: std::collections::BTreeMap<PathBuf, PinnedRoot>,
    directories: std::collections::BTreeMap<PathBuf, [u64; 2]>,
}
thread_local! {
    static QUERY_BUDGET: std::cell::RefCell<Option<QueryBudget>> = const { std::cell::RefCell::new(None) };
}
struct QuerySession {
    outer: bool,
}
impl QuerySession {
    fn start() -> Self {
        let outer = QUERY_BUDGET.with(|state| {
            let mut state = state.borrow_mut();
            if state.is_none() {
                *state = Some(QueryBudget::default());
                true
            } else {
                false
            }
        });
        Self { outer }
    }
}
impl Drop for QuerySession {
    fn drop(&mut self) {
        if self.outer {
            QUERY_BUDGET.with(|state| *state.borrow_mut() = None);
        }
    }
}
pub(super) struct QueryVisit {
    key: (PathBuf, String, &'static str),
}
impl QueryVisit {
    fn enter(root: &Path, id: &str, kind: &'static str) -> Result<Self, ObserverError> {
        // The public entry already resolved aliases. Re-canonicalizing here
        // could silently re-key the query to a replacement symlink target.
        let root = root.to_path_buf();
        let key = (root.clone(), id.to_owned(), kind);
        QUERY_BUDGET.with(|state| {
            let mut state = state.borrow_mut();
            let budget = state
                .as_mut()
                .ok_or_else(|| error(&root, "query_session_missing"))?;
            if let Some(pinned) = budget.roots.get(&root) {
                pinned.require_current()?;
            } else {
                budget.roots.insert(root.clone(), PinnedRoot::open(&root)?);
            }
            if budget.active.contains(&key) {
                return Err(error(&root, "dependency_cycle"));
            }
            if budget.active.len() >= 32 || budget.work >= 256 {
                return Err(error(&root, "dependency_work_budget_exceeded"));
            }
            budget.subjects.insert((root.clone(), id.to_owned()));
            if budget.subjects.len() > 32 {
                return Err(error(&root, "dependency_subject_budget_exceeded"));
            }
            budget.work += 1;
            budget.active.push(key.clone());
            Ok(Self { key })
        })
    }
}
impl Drop for QueryVisit {
    fn drop(&mut self) {
        QUERY_BUDGET.with(|state| {
            if let Some(budget) = state.borrow_mut().as_mut() {
                budget.active.retain(|key| key != &self.key);
            }
        });
    }
}
pub(super) fn enter_recovery_query(
    root: &Path,
    id: &str,
) -> Result<Option<QueryVisit>, ObserverError> {
    if QUERY_BUDGET.with(|state| state.borrow().is_some()) {
        QueryVisit::enter(root, id, "recovery").map(Some)
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod query_budget_tests {
    use super::*;
    #[test]
    fn review_regression_ascii_case_alias_remains_a_canonical_obligation() {
        let owner = tempfile::tempdir().unwrap();
        let root = owner.path().canonicalize().unwrap();
        fs::create_dir_all(root.join(".ai/decisions")).unwrap();
        fs::write(
            root.join(".ai/decisions/WI-ALIAS.legacy-closeout-source-requirement.JSON"),
            b"{broken",
        )
        .unwrap();
        assert!(has_obligation(&root, "WI-ALIAS"));
        assert!(obligation_ids(&root).unwrap().contains("WI-ALIAS"));
    }
    fn scope_with_invalid_obligation(named_predecessor: bool) -> Vec<String> {
        let owner = tempfile::tempdir().unwrap();
        let root = owner.path().canonicalize().unwrap();
        let id = "WI-SCOPE-REVIEW";
        fs::create_dir_all(root.join(".ai/decisions")).unwrap();
        fs::create_dir_all(root.join(".ai/work-items/archive")).unwrap();
        let contract = serde_json::to_vec(&serde_json::json!({"workItemId":id,"repositoryId":repository_id(&root),"scope":if named_predecessor {vec!["src/**"]} else {vec!["tests/**"]}})).unwrap();
        fs::write(
            root.join(format!(".ai/work-items/archive/{id}.contract.json")),
            &contract,
        )
        .unwrap();
        fs::write(root.join(format!(".ai/work-items/archive/{id}.archive.json")), serde_json::to_vec(&serde_json::json!({"workItemId":id,"state":"archived","closeRequired":true,"files":{"contractDigest":Digest::sha256_bytes(&contract)}})).unwrap()).unwrap();
        fs::write(
            root.join(format!(".ai/decisions/{id}{REQUIREMENT_SUFFIX}")),
            b"{broken",
        )
        .unwrap();
        let predecessors = if named_predecessor {
            BTreeSet::from([id.to_owned()])
        } else {
            BTreeSet::new()
        };
        super::super::unclosed_archived_scope_conflicts(&root, &["src/**".into()], &predecessors)
            .unwrap()
    }
    #[test]
    fn review_regression_named_predecessor_cannot_waive_invalid_obligation() {
        assert!(
            scope_with_invalid_obligation(true)
                .iter()
                .any(|blocker| blocker == "archived_work_item_scope_untrusted:WI-SCOPE-REVIEW")
        );
    }
    #[test]
    fn review_regression_resealed_foreign_scope_cannot_hide_invalid_obligation() {
        assert!(
            scope_with_invalid_obligation(false)
                .iter()
                .any(|blocker| blocker == "archived_work_item_scope_untrusted:WI-SCOPE-REVIEW")
        );
    }
    #[test]
    fn query_rejects_same_byte_parent_directory_replacement() {
        let owner = tempfile::tempdir().unwrap();
        let root = owner.path().canonicalize().unwrap();
        let parent = root.join(".ai/decisions");
        fs::create_dir_all(&parent).unwrap();
        fs::write(parent.join("proof.json"), b"{}").unwrap();
        let _query = QuerySession::start();
        let _visit = QueryVisit::enter(&root, "WI-ROOT", "observation").unwrap();
        assert_eq!(
            safe_read(&root, ".ai/decisions/proof.json", MAX_BYTES).unwrap(),
            b"{}"
        );
        fs::rename(&parent, root.join("retained-decisions")).unwrap();
        fs::create_dir(&parent).unwrap();
        fs::write(parent.join("proof.json"), b"{}").unwrap();
        assert!(safe_read(&root, ".ai/decisions/proof.json", MAX_BYTES).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn replaced_repository_root_cannot_redirect_a_read() {
        let owner = tempfile::tempdir().unwrap();
        let root = owner.path().canonicalize().unwrap().join("root");
        let outside = owner.path().canonicalize().unwrap().join("outside");
        for path in [&root, &outside] {
            fs::create_dir_all(path.join(".ai/decisions")).unwrap();
        }
        fs::write(outside.join(".ai/decisions/proof.json"), b"outside").unwrap();
        fs::rename(&root, owner.path().canonicalize().unwrap().join("retained")).unwrap();
        std::os::unix::fs::symlink(&outside, &root).unwrap();
        assert!(safe_read(&root, ".ai/decisions/proof.json", MAX_BYTES).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn query_pins_root_identity_before_its_first_payload_read() {
        let owner = tempfile::tempdir().unwrap();
        let root = owner.path().canonicalize().unwrap().join("root");
        fs::create_dir_all(root.join(".ai/decisions")).unwrap();
        let _query = QuerySession::start();
        let _visit = QueryVisit::enter(&root, "WI-ROOT", "observation").unwrap();
        fs::rename(&root, owner.path().canonicalize().unwrap().join("retained")).unwrap();
        fs::create_dir_all(root.join(".ai/decisions")).unwrap();
        fs::write(root.join(".ai/decisions/proof.json"), b"replacement").unwrap();
        assert!(safe_read(&root, ".ai/decisions/proof.json", MAX_BYTES).is_err());
    }
    #[test]
    fn active_query_cannot_fall_back_to_unbound_governance_reads() {
        let bound = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        fs::create_dir_all(other.path().join(".ai")).unwrap();
        let path = other.path().join(".ai/proof.json");
        fs::write(&path, b"{}").unwrap();
        let _query = QuerySession::start();
        let _visit = QueryVisit::enter(
            &bound.path().canonicalize().unwrap(),
            "WI-ROOT",
            "observation",
        )
        .unwrap();
        assert!(super::super::read_json(&path).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn held_guard_rejects_replaced_root_and_canonical_lock() {
        for replace_root in [false, true] {
            let owner = tempfile::tempdir().unwrap();
            let root = owner.path().canonicalize().unwrap().join("root");
            fs::create_dir_all(root.join(".ai")).unwrap();
            fs::write(
                root.join(".ai/cockpit.toml"),
                b"repository_id = 'sha256:fixture'\n",
            )
            .unwrap();
            let guard = acquire_lifecycle_lock(&root, "WI-ROOT").unwrap();
            let path = if replace_root {
                root.clone()
            } else {
                root.join(".ai/locks")
            };
            fs::rename(&path, owner.path().canonicalize().unwrap().join("retained")).unwrap();
            fs::create_dir_all(root.join(".ai/locks")).unwrap();
            fs::write(
                root.join(".ai/cockpit.toml"),
                b"repository_id = 'sha256:fixture'\n",
            )
            .unwrap();
            fs::write(root.join(".ai/locks/WI-ROOT.lifecycle.lock"), b"").unwrap();
            assert!(
                guard.require_subject(&root, "WI-ROOT").is_err(),
                "physical root and canonical lock must remain bound"
            );
        }
    }
    #[test]
    fn relative_paths_reject_aliases_and_explicit_size_budgets() {
        let root = Path::new("/");
        for path in [
            ".ai//decisions/proof.json".to_owned(),
            ".ai/./proof.json".into(),
            format!(".ai/{}", "a".repeat(4096)),
            format!(".ai/{}proof.json", "a/".repeat(64)),
        ] {
            assert!(relative_parts(root, &path).is_err());
        }
    }
    #[test]
    fn obligation_enumeration_has_a_directory_work_budget() {
        let owner = tempfile::tempdir().unwrap();
        fs::create_dir_all(owner.path().canonicalize().unwrap().join(".ai/decisions")).unwrap();
        for index in 0..4097 {
            fs::write(
                owner
                    .path()
                    .join(format!(".ai/decisions/unrelated-{index}")),
                b"",
            )
            .unwrap();
        }
        assert!(obligation_ids(owner.path()).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn one_query_cannot_validate_two_different_bytes_for_the_same_path() {
        let root_owner = tempfile::tempdir().unwrap();
        let root = root_owner.path().canonicalize().unwrap();
        fs::create_dir_all(root.as_path().join(".ai/decisions")).unwrap();
        let relative = ".ai/decisions/WI-A.json";
        fs::write(root.as_path().join(relative), b"first").unwrap();
        let _query = QuerySession::start();
        let _visit = QueryVisit::enter(root.as_path(), "WI-A", "observation").unwrap();
        assert_eq!(
            safe_read(root.as_path(), relative, MAX_BYTES).unwrap(),
            b"first"
        );
        fs::write(root.as_path().join(relative), b"changed").unwrap();
        assert!(
            safe_read(root.as_path(), relative, MAX_BYTES).is_err(),
            "lower reads and final inventory must describe one consistent observation"
        );
    }
    #[test]
    fn lower_historical_json_read_cannot_follow_replaced_parent() {
        let owner = tempfile::tempdir().unwrap();
        let root = owner.path().canonicalize().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.join(".ai/work-items")).unwrap();
        fs::write(outside.path().join("proof.json"), b"{\"outside\":true}").unwrap();
        std::os::unix::fs::symlink(outside.path(), root.join(".ai/work-items/archive")).unwrap();
        let _query = QuerySession::start();
        let _visit = QueryVisit::enter(&root, "WI-BOUND", "observation").unwrap();
        let error = super::super::read_json(&root.join(".ai/work-items/archive/proof.json"))
            .expect_err(
                "lower validators must use capability-bound reads during a legacy assessment",
            );
        assert!(!error.to_string().is_empty());
    }
    #[test]
    fn cycles_fail_closed_and_raii_clears_failed_queries() {
        let root_owner = tempfile::tempdir().unwrap();
        let root = root_owner.path().canonicalize().unwrap();
        {
            let _query = QuerySession::start();
            let _a = QueryVisit::enter(root.as_path(), "WI-A", "registration").unwrap();
            let _b = QueryVisit::enter(root.as_path(), "WI-B", "registration").unwrap();
            assert!(QueryVisit::enter(root.as_path(), "WI-A", "registration").is_err());
        }
        let _query = QuerySession::start();
        assert!(QueryVisit::enter(root.as_path(), "WI-A", "registration").is_ok());
    }
    #[test]
    fn depth_and_repeated_work_have_independent_query_budgets() {
        let root_owner = tempfile::tempdir().unwrap();
        let root = root_owner.path().canonicalize().unwrap();
        {
            let _query = QuerySession::start();
            let mut guards = Vec::new();
            for index in 0..32 {
                guards.push(
                    QueryVisit::enter(root.as_path(), &format!("WI-{index}"), "registration")
                        .unwrap(),
                );
            }
            assert!(QueryVisit::enter(root.as_path(), "WI-OVERFLOW", "registration").is_err());
        }
        {
            let _query = QuerySession::start();
            for _ in 0..256 {
                drop(QueryVisit::enter(root.as_path(), "WI-A", "registration").unwrap());
            }
            assert!(QueryVisit::enter(root.as_path(), "WI-A", "registration").is_err());
        }
        let _query = QuerySession::start();
        assert!(QueryVisit::enter(root.as_path(), "WI-A", "registration").is_ok());
    }
}

fn query_path(path: &Path) -> Option<(PathBuf, String)> {
    QUERY_BUDGET.with(|state| {
        let state = state.borrow();
        let budget = state.as_ref()?;
        budget.subjects.iter().find_map(|(root, _)| {
            let relative = path.strip_prefix(root).ok()?;
            if !relative.starts_with(".ai") {
                return None;
            }
            Some((root.clone(), relative.to_string_lossy().replace('\\', "/")))
        })
    })
}
pub(super) fn read_under_query(path: &Path) -> Option<std::io::Result<Vec<u8>>> {
    if let Some((root, relative)) = query_path(path) {
        Some(safe_read(&root, &relative, MAX_BYTES).map_err(std::io::Error::other))
    } else if unbound_governance_path(path) {
        Some(Err(std::io::Error::other(
            "legacy_closeout_source:query_root_unbound",
        )))
    } else {
        None
    }
}
fn unbound_governance_path(path: &Path) -> bool {
    let _ = path;
    // These hooks are only used for governance reads. In particular, an
    // alternate `.AI` spelling on a case-insensitive filesystem must not
    // escape to an ambient read. Explicit caller inputs use a separate reader.
    QUERY_BUDGET.with(|state| state.borrow().is_some())
}
pub(super) struct QueryDirectoryEntry {
    path: PathBuf,
    name: std::ffi::OsString,
}
impl QueryDirectoryEntry {
    pub(super) fn path(&self) -> PathBuf {
        self.path.clone()
    }
    pub(super) fn file_name(&self) -> std::ffi::OsString {
        self.name.clone()
    }
}
pub(super) fn read_query_directory(
    path: impl AsRef<Path>,
) -> std::io::Result<Box<dyn Iterator<Item = std::io::Result<QueryDirectoryEntry>>>> {
    let path = path.as_ref();
    if let Some((root, relative)) = query_path(path) {
        let parts = relative_parts(&root, &relative).map_err(std::io::Error::other)?;
        let mut dir = root_directory(&root).map_err(std::io::Error::other)?;
        let mut display = root.clone();
        for part in parts {
            display.push(&part);
            dir = open_cap_directory_nofollow_strict(&dir, &part, &display)
                .map_err(std::io::Error::other)?;
            observe_directory(&display, &dir).map_err(std::io::Error::other)?;
        }
        let mut entries = Vec::new();
        for entry in dir.entries()? {
            if entries.len() >= 4096 {
                return Err(std::io::Error::other(
                    "legacy_closeout_source:directory_budget_exceeded",
                ));
            }
            let entry = entry?;
            let name = entry.file_name();
            entries.push(Ok(QueryDirectoryEntry {
                path: path.join(&name),
                name,
            }));
        }
        Ok(Box::new(entries.into_iter()))
    } else if unbound_governance_path(path) {
        Err(std::io::Error::other(
            "legacy_closeout_source:query_root_unbound",
        ))
    } else {
        Ok(Box::new(fs::read_dir(path)?.map(|entry| {
            entry.map(|entry| QueryDirectoryEntry {
                path: entry.path(),
                name: entry.file_name(),
            })
        })))
    }
}

fn observe_directory(path: &Path, dir: &Dir) -> Result<(), ObserverError> {
    let identity = directory_identity(path, dir)?;
    QUERY_BUDGET.with(|state| {
        if let Some(budget) = state.borrow_mut().as_mut() {
            if let Some(previous) = budget.directories.insert(path.into(), identity)
                && previous != identity
            {
                return Err(error(path, "directory_identity_changed"));
            }
            if budget.directories.len() > 4096 {
                return Err(error(path, "directory_budget_exceeded"));
            }
        }
        Ok(())
    })
}

fn begin_query_read(root: &Path, maximum: u64) -> Result<u64, ObserverError> {
    QUERY_BUDGET.with(|state| {
        let mut state = state.borrow_mut();
        if let Some(budget) = state.as_mut() {
            if budget.read_calls >= 4096 || budget.read_bytes >= 64 * MAX_BYTES {
                return Err(error(root, "dependency_read_budget_exceeded"));
            }
            budget.read_calls += 1;
            Ok(maximum.min(64 * MAX_BYTES - budget.read_bytes))
        } else {
            Ok(maximum)
        }
    })
}
fn complete_query_read(root: &Path, relative: &str, bytes: &[u8]) -> Result<(), ObserverError> {
    QUERY_BUDGET.with(|state| {
        let mut state = state.borrow_mut();
        if let Some(budget) = state.as_mut() {
            budget.read_bytes = budget
                .read_bytes
                .checked_add(bytes.len() as u64)
                .ok_or_else(|| error(root, "dependency_read_budget_exceeded"))?;
            let binding = (Digest::sha256_bytes(bytes), bytes.len() as u64);
            if let Some(previous) = budget.observed.insert(root.join(relative), binding.clone())
                && previous != binding
            {
                return Err(error(root, "history_changed_during_observation"));
            }
        }
        Ok(())
    })
}

#[cfg(test)]
mod publication_tests {
    use super::*;
    const ID: &str = "WI-PUBLISH-FAULT";
    fn fixture() -> (tempfile::TempDir, PathBuf) {
        let owner = tempfile::tempdir().unwrap();
        let root = owner.path().canonicalize().unwrap();
        fs::create_dir_all(root.join(".ai/decisions")).unwrap();
        assert!(
            Command::new("git")
                .args(["init", "--quiet"])
                .current_dir(&root)
                .status()
                .unwrap()
                .success()
        );
        (owner, root)
    }
    // Exercise the no-replace publication primitive independently. Complete
    // historical root/extension admission and constrained resumes are covered
    // by the integration fixtures, not asserted by this synthetic plan.
    fn primitive_plan(root: &Path) -> Plan {
        Plan {
            schema_version: 1,
            kind: "legacy_closeout_source_plan".into(),
            state: "prepared".into(),
            operation: Operation::Resume,
            repository_id: repository_id(root).to_string(),
            work_item_id: ID.into(),
            destination_root: root.to_string_lossy().into_owned(),
            destination_binding: directory_binding(root).unwrap(),
            source_binding: None,
            registration_bytes: "{}".into(),
            requirement_bytes: "{}".into(),
            registration_raw_digest: Digest::sha256_bytes(b"{}"),
            requirement_raw_digest: Digest::sha256_bytes(b"{}"),
            observation_digest: Digest::sha256_bytes(b"primitive test"),
            plan_digest: Digest::sha256_bytes(b"primitive test"),
            extension_bytes: None,
            extension_raw_digest: None,
        }
    }
    fn names(extension: bool) -> [String; 2] {
        if extension {
            let transition: serde_json::Value =
                serde_json::from_slice(&transition_bytes()).unwrap();
            let digest = cockpit_protocol::digest_json(&transition).unwrap();
            [
                format!("{ID}{EXTENSION_SUFFIX}"),
                format!(
                    "{ID}.finalize.{}.json",
                    digest.as_str().trim_start_matches("sha256:")
                ),
            ]
        } else {
            [
                format!("{ID}{REQUIREMENT_SUFFIX}"),
                format!("{ID}{REGISTRATION_SUFFIX}"),
            ]
        }
    }
    fn bytes(name: &str) -> Vec<u8> {
        if name.contains(".finalize.") {
            return transition_bytes();
        }
        format!("{{\"name\":\"{name}\",\"actor\":\"original fixture actor\",\"recordedAt\":\"2026-10-01T00:00:00Z\"}}").into_bytes()
    }
    fn transition_bytes() -> Vec<u8> {
        br#"{"transitionId":"original fixture transition","sequence":1,"actor":"original fixture actor","recordedAt":"2026-10-01T00:00:00Z"}"#.to_vec()
    }
    fn decisions(root: &Path) -> Dir {
        let dir = root_directory(root).unwrap();
        let ai = open_cap_directory_nofollow_strict(&dir, ".ai", &root.join(".ai")).unwrap();
        open_cap_directory_nofollow_strict(&ai, "decisions", &root.join(".ai/decisions")).unwrap()
    }
    #[test]
    fn publication_child() {
        let Some(root) = std::env::var_os("COCKPIT_TEST_LEGACY_SOURCE_CHILD_ROOT") else {
            return;
        };
        let root = PathBuf::from(root).canonicalize().unwrap();
        let extension = std::env::var("COCKPIT_TEST_LEGACY_SOURCE_PAIR").unwrap() == "extension";
        let guard = acquire_lifecycle_lock(&root, ID).unwrap();
        let dir = decisions(&root);
        let plan = primitive_plan(&root);
        for name in names(extension) {
            let result = publish(&root, ID, &guard, &plan, &dir, &name, &bytes(&name));
            if std::env::var("COCKPIT_TEST_LEGACY_SOURCE_FAULT_ACTION")
                .ok()
                .as_deref()
                == Some("io_failure")
                && result.is_err()
            {
                let error = result.unwrap_err().to_string();
                assert!(error.contains("commit_state_unknown"), "{error}");
                assert!(
                    error.contains("injected_"),
                    "the original failure cause must remain visible: {error}"
                );
                return;
            }
            if std::env::var("COCKPIT_TEST_LEGACY_SOURCE_FAULT_ACTION")
                .ok()
                .as_deref()
                == Some("replace_parent")
                && result.is_err()
            {
                let error = result.unwrap_err().to_string();
                if std::env::var("COCKPIT_TEST_LEGACY_SOURCE_FAULT")
                    .unwrap()
                    .ends_with(":canonical_linked")
                {
                    assert!(error.contains("commit_state_unknown"), "{error}");
                } else {
                    assert!(error.contains("publication_parent_changed"), "{error}");
                }
                return;
            }
            result.unwrap();
        }
        panic!("the requested publication fault did not fire");
    }
    fn child(
        root: &Path,
        extension: bool,
        target: &str,
        phase: &str,
        replace: bool,
    ) -> std::process::Output {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "legacy_closeout_source::publication_tests::publication_child",
                "--nocapture",
            ])
            .env("COCKPIT_TEST_LEGACY_SOURCE_CHILD_ROOT", root)
            .env(
                "COCKPIT_TEST_LEGACY_SOURCE_PAIR",
                if extension { "extension" } else { "root" },
            )
            .env(
                "COCKPIT_TEST_LEGACY_SOURCE_FAULT",
                format!("{target}:{phase}"),
            );
        if replace {
            command.env("COCKPIT_TEST_LEGACY_SOURCE_FAULT_ACTION", "replace_parent");
        } else {
            command.env_remove("COCKPIT_TEST_LEGACY_SOURCE_FAULT_ACTION");
        }
        command.output().unwrap()
    }
    #[test]
    fn abrupt_exit_matrix_preserves_canonical_bytes_and_exact_primitive_retry() {
        for extension in [false, true] {
            let pair = names(extension);
            for target_index in 0..2 {
                for (phase, linked) in [
                    ("temporary_created", false),
                    ("file_synced", false),
                    ("canonical_linked", true),
                    ("parent_sync_step_completed", true),
                    ("read_back", true),
                ] {
                    let (_owner, root) = fixture();
                    let output = child(&root, extension, &pair[target_index], phase, false);
                    assert_eq!(
                        output.status.code(),
                        Some(86),
                        "{}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    for (index, name) in pair.iter().enumerate() {
                        let canonical = root.join(".ai/decisions").join(name);
                        let expected = index < target_index || (index == target_index && linked);
                        assert_eq!(canonical.exists(), expected, "{name}:{phase}");
                        if expected {
                            assert_eq!(fs::read(&canonical).unwrap(), bytes(name));
                        }
                    }
                    let guard = acquire_lifecycle_lock(&root, ID).unwrap();
                    let dir = decisions(&root);
                    let plan = primitive_plan(&root);
                    for name in &pair {
                        publish(&root, ID, &guard, &plan, &dir, name, &bytes(name)).unwrap();
                    }
                    for name in &pair {
                        assert_eq!(
                            fs::read(root.join(".ai/decisions").join(name)).unwrap(),
                            bytes(name)
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn review_regression_io_failures_after_second_canonical_report_unknown_and_preserve_both() {
        for extension in [false, true] {
            for phase in ["parent_sync", "readback", "report"] {
                let (_owner, root) = fixture();
                let pair = names(extension);
                let output = Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "legacy_closeout_source::publication_tests::publication_child",
                        "--nocapture",
                    ])
                    .env("COCKPIT_TEST_LEGACY_SOURCE_CHILD_ROOT", &root)
                    .env(
                        "COCKPIT_TEST_LEGACY_SOURCE_PAIR",
                        if extension { "extension" } else { "root" },
                    )
                    .env(
                        "COCKPIT_TEST_LEGACY_SOURCE_FAULT",
                        format!("{}:{phase}", pair[1]),
                    )
                    .env("COCKPIT_TEST_LEGACY_SOURCE_FAULT_ACTION", "io_failure")
                    .output()
                    .unwrap();
                assert!(
                    output.status.success(),
                    "{phase}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                for name in &pair {
                    assert_eq!(
                        fs::read(root.join(".ai/decisions").join(name)).unwrap(),
                        bytes(name)
                    );
                }
                let guard = acquire_lifecycle_lock(&root, ID).unwrap();
                let dir = decisions(&root);
                let plan = primitive_plan(&root);
                for name in &pair {
                    publish(&root, ID, &guard, &plan, &dir, name, &bytes(name)).unwrap();
                }
            }
        }
    }
    #[cfg(unix)]
    #[test]
    fn parent_replacement_before_or_after_link_is_rejected_without_rollback() {
        for phase in ["file_synced", "canonical_linked"] {
            let (_owner, root) = fixture();
            let name = names(false)[0].clone();
            let output = child(&root, false, &name, phase, true);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(!root.join(".ai/decisions").join(&name).exists());
            let retained = root.join("retained-decisions").join(&name);
            assert_eq!(retained.exists(), phase == "canonical_linked");
            if retained.exists() {
                assert_eq!(fs::read(retained).unwrap(), bytes(&name));
            }
        }
    }
    #[cfg(unix)]
    #[test]
    fn no_replace_publication_preserves_hardlink_symlink_and_directory_victims() {
        for kind in ["hardlink", "symlink", "directory"] {
            let (_owner, root) = fixture();
            let name = names(false)[0].clone();
            let canonical = root.join(".ai/decisions").join(&name);
            let victim = root.join("victim.json");
            fs::write(&victim, b"victim bytes").unwrap();
            let identity = fs::metadata(&victim).unwrap().ino();
            match kind {
                "hardlink" => fs::hard_link(&victim, &canonical).unwrap(),
                "symlink" => std::os::unix::fs::symlink(&victim, &canonical).unwrap(),
                _ => {
                    fs::create_dir(&canonical).unwrap();
                    fs::write(canonical.join("sentinel"), b"directory bytes").unwrap();
                }
            }
            let guard = acquire_lifecycle_lock(&root, ID).unwrap();
            let dir = decisions(&root);
            let plan = primitive_plan(&root);
            assert!(publish(&root, ID, &guard, &plan, &dir, &name, &bytes(&name)).is_err());
            assert_eq!(fs::read(&victim).unwrap(), b"victim bytes");
            assert_eq!(fs::metadata(&victim).unwrap().ino(), identity);
            if kind == "directory" {
                assert_eq!(
                    fs::read(canonical.join("sentinel")).unwrap(),
                    b"directory bytes"
                );
            } else if kind == "symlink" {
                assert!(
                    fs::symlink_metadata(&canonical)
                        .unwrap()
                        .file_type()
                        .is_symlink()
                );
            } else {
                assert_eq!(fs::metadata(&canonical).unwrap().ino(), identity);
            }
        }
    }
}

#[cfg(all(test, unix))]
mod nonregular_tests {
    use super::*;
    fn fifo(path: &Path) {
        use std::os::unix::ffi::OsStrExt;
        let path = std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
    }
    #[test]
    fn fifo_child() {
        let Some(root) = std::env::var_os("COCKPIT_TEST_LEGACY_FIFO_ROOT") else {
            return;
        };
        let root = PathBuf::from(root).canonicalize().unwrap();
        let mode = std::env::var("COCKPIT_TEST_LEGACY_FIFO_MODE").unwrap();
        if mode == "history" {
            assert!(safe_read(&root, ".ai/decisions/proof.json", MAX_BYTES).is_err());
        } else if mode == "input" {
            assert!(read_input_bytes(&root.join("payload.json")).is_err());
        } else {
            let guard = acquire_lifecycle_lock(&root, "WI-FIFO").unwrap();
            let lock = root.join(".ai/locks/WI-FIFO.lifecycle.lock");
            fs::remove_file(&lock).unwrap();
            fifo(&lock);
            assert!(guard.require_subject(&root, "WI-FIFO").is_err());
        }
    }
    fn rejects_without_blocking(mode: &str) {
        let owner = tempfile::tempdir().unwrap();
        let root = owner.path().canonicalize().unwrap();
        fs::create_dir_all(root.join(".ai/decisions")).unwrap();
        fifo(&root.join(".ai/decisions/proof.json"));
        fifo(&root.join("payload.json"));
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "legacy_closeout_source::nonregular_tests::fifo_child",
                "--nocapture",
            ])
            .env("COCKPIT_TEST_LEGACY_FIFO_ROOT", &root)
            .env("COCKPIT_TEST_LEGACY_FIFO_MODE", mode)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break Some(status);
            }
            if std::time::Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                break None;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        };
        assert!(
            status.is_some(),
            "{mode} must reject a FIFO before a blocking open"
        );
        assert!(status.unwrap().success(), "{mode} child failed");
    }
    #[test]
    fn history_fifo_is_rejected_without_blocking() {
        rejects_without_blocking("history");
    }
    #[test]
    fn caller_input_fifo_is_rejected_without_blocking() {
        rejects_without_blocking("input");
    }
    #[test]
    fn replaced_lock_fifo_is_rejected_without_blocking() {
        rejects_without_blocking("guard");
    }
}

//! Read-only, committed-source material identity. No decision or discharge is
//! performed here; the opt-in is only reported to a future review service.

use super::{
    contains_strong_instruction_injection, derive_governance_signals_with_diagnostics,
    effective_policy_for_contract, repository_id,
};
use crate::rust_material::{MaterialUnknownCause, RustMaterialAssessment};
use cockpit_core::Digest;
use cockpit_git::{ChangeContentState, ChangeKind, GitRepository};
use cockpit_protocol::{Contract, MATERIAL_INSPECTION_REVIEW_CAPABILITY, digest_json};
use serde::Serialize;
use sha2::{Digest as _, Sha256};
use std::{collections::BTreeMap, fs, path::Path, process::Command};
use thiserror::Error;

const SCANNER_SEMANTIC_VERSION: &str = "rust-material-v1";
const ANALYSIS_TARGET_SEMANTIC_PROFILE: &str = "cross-platform-rust-source-v1";
const MAX_MATERIAL_BLOB_BYTES: u64 = 16 * 1024 * 1024;
const MAX_PATCH_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialScannerAssessment {
    Clean,
    Finding,
    Unknown,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialReviewEntry {
    pub path: String,
    pub kind: String,
    pub content_state: String,
    pub scanner_assessment: MaterialScannerAssessment,
    pub unknown_cause: Option<MaterialUnknownCause>,
    pub reviewable: bool,
    pub changed_hunk_digest: Digest,
    pub after_blob_digest: Option<Digest>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialReviewRequest {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub contract_digest: Digest,
    pub immutable_contract_base_revision: String,
    pub source_snapshot_digest: Digest,
    pub material_manifest_digest: Digest,
    pub scanner_semantic_version: String,
    pub analysis_implementation_digest: Digest,
    pub analysis_target_semantic_profile: String,
    pub material_inspection_review_profile_digest: Option<Digest>,
    pub effective_policy_digest: Digest,
    pub entries: Vec<MaterialReviewEntry>,
    pub raw_unknown_codes: Vec<String>,
    pub blocked_by_finding: bool,
    pub review_enabled: bool,
    pub review_diagnostic: Option<String>,
    pub request_digest: Digest,
    /// Provenance only. It is excluded from request_digest so an identical
    /// source manifest in a legitimate descendant remains comparable.
    pub reviewed_source_head: String,
}

#[derive(Debug, Error)]
pub enum MaterialReviewRequestError {
    #[error("Git observation failed: {0}")]
    Git(String),
    #[error("Contract identity is not bound to this repository")]
    ContractIdentity,
    #[error("non-.ai source has staged, unstaged, or untracked changes: {0}")]
    DirtySource(String),
    #[error("material source {path} is unavailable: {reason}")]
    SourceUnavailable { path: String, reason: String },
    #[error("material identity could not be computed: {0}")]
    Identity(String),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestEntry<'a> {
    path: &'a str,
    kind: &'a str,
    content_state: &'a str,
    added_lines: &'a [String],
    added_line_origins: Vec<(usize, usize)>,
    removed_lines: &'a [String],
    changed_hunk_digest: &'a Digest,
    after_blob_digest: &'a Option<Digest>,
}

fn json_digest(value: &impl Serialize) -> Result<Digest, MaterialReviewRequestError> {
    digest_json(value).map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))
}

/// Domain-separated, length-delimited source bytes. The fixed path list is
/// compiled into the binary, so compiler output, host path, and binary SHA do
/// not affect the result. A source-byte change changes the digest even when
/// SCANNER_SEMANTIC_VERSION is accidentally left unchanged.
fn implementation_digest_from_sources(sources: &[(&str, &[u8])]) -> Digest {
    let mut hasher = Sha256::new();
    hasher.update(b"ai-cockpit:material-analysis-implementation:v1\0");
    for (path, bytes) in sources {
        hasher.update((path.len() as u64).to_le_bytes());
        hasher.update(path.as_bytes());
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
    }
    Digest::sha256_bytes(&hasher.finalize())
}

fn analysis_implementation_digest() -> Digest {
    implementation_digest_from_sources(&[
        ("Cargo.lock", include_bytes!("../../../Cargo.lock")),
        (
            "crates/cockpit-core/src/lib.rs",
            include_bytes!("../../cockpit-core/src/lib.rs"),
        ),
        (
            "crates/cockpit-git/src/lib.rs",
            include_bytes!("../../cockpit-git/src/lib.rs"),
        ),
        (
            "crates/cockpit-protocol/src/lib.rs",
            include_bytes!("../../cockpit-protocol/src/lib.rs"),
        ),
        (
            "crates/cockpit-protocol/src/contract_amendment.rs",
            include_bytes!("../../cockpit-protocol/src/contract_amendment.rs"),
        ),
        (
            "crates/cockpit-repository/Cargo.toml",
            include_bytes!("../Cargo.toml"),
        ),
        (
            "crates/cockpit-repository/src/governance_controls.rs",
            include_bytes!("governance_controls.rs"),
        ),
        (
            "crates/cockpit-repository/src/lib.rs",
            include_bytes!("lib.rs"),
        ),
        (
            "crates/cockpit-repository/src/material_review.rs",
            include_bytes!("material_review.rs"),
        ),
        (
            "crates/cockpit-repository/src/rust_material.rs",
            include_bytes!("rust_material.rs"),
        ),
        (
            "crates/cockpit-repository/src/status_projection.rs",
            include_bytes!("status_projection.rs"),
        ),
    ])
}

fn source_patch_digest(
    root: &Path,
    base: &str,
    head: &str,
) -> Result<Digest, MaterialReviewRequestError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "-c",
            "core.quotePath=false",
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "--no-color",
            "--no-renames",
            "--diff-algorithm=myers",
            "--binary",
            "--unified=0",
            base,
            head,
            "--",
            ".",
            ":(exclude).ai/**",
        ])
        .output()
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    if !output.status.success() {
        return Err(MaterialReviewRequestError::Git(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }
    if output.stdout.len() > MAX_PATCH_BYTES {
        return Err(MaterialReviewRequestError::Identity(
            "committed material patch exceeds bounded request budget".into(),
        ));
    }
    Ok(Digest::sha256_bytes(&output.stdout))
}

fn reject_nonstandard_index_flags(root: &Path) -> Result<(), MaterialReviewRequestError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-v", "-z"])
        .output()
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    if !output.status.success() {
        return Err(MaterialReviewRequestError::Git(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }
    for record in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        if record.len() < 2 {
            return Err(MaterialReviewRequestError::Identity(
                "malformed index flags".into(),
            ));
        }
        let (flag, path) = record.split_at(2);
        if path != b".ai" && !path.starts_with(b".ai/") && flag != b"H " {
            return Err(MaterialReviewRequestError::DirtySource(format!(
                "nonstandard index flag for {}",
                String::from_utf8_lossy(path)
            )));
        }
    }
    Ok(())
}

fn committed_blob_ids(root: &Path) -> Result<BTreeMap<String, String>, MaterialReviewRequestError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-tree", "-r", "-z", "HEAD"])
        .output()
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    if !output.status.success() {
        return Err(MaterialReviewRequestError::Git(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }
    let mut objects = BTreeMap::new();
    for record in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let Some(tab) = record.iter().position(|byte| *byte == b'\t') else {
            return Err(MaterialReviewRequestError::Identity(
                "malformed committed tree entry".into(),
            ));
        };
        let metadata = &record[..tab];
        let path = &record[tab + 1..];
        let path = std::str::from_utf8(path)
            .map_err(|_| MaterialReviewRequestError::Identity("non-UTF-8 committed path".into()))?;
        if path == ".ai" || path.starts_with(".ai/") {
            continue;
        }
        let fields = std::str::from_utf8(metadata).map_err(|_| {
            MaterialReviewRequestError::Identity("malformed committed tree metadata".into())
        })?;
        let mut fields = fields.split_whitespace();
        let (Some(_mode), Some(kind), Some(id)) = (fields.next(), fields.next(), fields.next())
        else {
            return Err(MaterialReviewRequestError::Identity(
                "malformed committed tree metadata".into(),
            ));
        };
        if kind != "blob" {
            return Err(MaterialReviewRequestError::Identity(
                "non-blob committed material".into(),
            ));
        }
        objects.insert(path.to_owned(), id.to_owned());
    }
    Ok(objects)
}

fn committed_blob(root: &Path, id: &str) -> Result<Vec<u8>, MaterialReviewRequestError> {
    let size = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "-s", id])
        .output()
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    if !size.status.success() {
        return Err(MaterialReviewRequestError::Git(
            String::from_utf8_lossy(&size.stderr).into_owned(),
        ));
    }
    let size = std::str::from_utf8(&size.stdout)
        .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?
        .trim()
        .parse::<u64>()
        .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?;
    if size > MAX_MATERIAL_BLOB_BYTES {
        return Err(MaterialReviewRequestError::Identity(
            "committed source exceeds bounded request budget".into(),
        ));
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "blob", id])
        .output()
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    if !output.status.success() || output.stdout.len() as u64 != size {
        return Err(MaterialReviewRequestError::Identity(
            "committed blob read is incomplete".into(),
        ));
    }
    Ok(output.stdout)
}

pub fn material_review_request(
    root: &Path,
    contract: &Contract,
) -> Result<MaterialReviewRequest, MaterialReviewRequestError> {
    let git = GitRepository::discover(root)
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    let root = git.root();
    if contract.repository_id != repository_id(root).to_string() {
        return Err(MaterialReviewRequestError::ContractIdentity);
    }
    reject_nonstandard_index_flags(root)?;
    let working = git
        .snapshot()
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    let dirty = working
        .changed_paths
        .iter()
        .filter(|path| *path != ".ai" && !path.starts_with(".ai/"))
        .cloned()
        .collect::<Vec<_>>();
    if !dirty.is_empty() {
        return Err(MaterialReviewRequestError::DirtySource(dirty.join(", ")));
    }
    let mut snapshot = git
        .snapshot_against(&contract.base_revision)
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    let head = snapshot
        .head
        .clone()
        .ok_or_else(|| MaterialReviewRequestError::Identity("committed HEAD is required".into()))?;
    let ancestor = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "merge-base",
            "--is-ancestor",
            &contract.base_revision,
            &head,
        ])
        .status()
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    if !ancestor.success() {
        return Err(MaterialReviewRequestError::Identity(
            "Contract base is not an ancestor of committed HEAD".into(),
        ));
    }
    snapshot
        .change_evidence
        .sort_by(|left, right| left.path.cmp(&right.path));
    let source_snapshot_digest = snapshot
        .source_tree_digest
        .as_deref()
        .ok_or_else(|| {
            MaterialReviewRequestError::Identity("source tree digest is missing".into())
        })?
        .parse::<Digest>()
        .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?;
    let committed_objects = committed_blob_ids(root)?;
    let mut committed_digests = BTreeMap::new();
    for change in snapshot
        .change_evidence
        .iter_mut()
        .filter(|change| change.path != ".ai" && !change.path.starts_with(".ai/"))
    {
        if change.kind == ChangeKind::Deleted {
            continue;
        }
        let id = committed_objects.get(&change.path).ok_or_else(|| {
            MaterialReviewRequestError::SourceUnavailable {
                path: change.path.clone(),
                reason: "source has no committed blob".into(),
            }
        })?;
        let bytes = committed_blob(root, id)?;
        committed_digests.insert(change.path.clone(), Digest::sha256_bytes(&bytes));
        if change.content_state == ChangeContentState::Text {
            if bytes.len() > 4 * 1024 * 1024 {
                change.content_state = ChangeContentState::TooLarge;
                change.after_text = None;
            } else {
                change.after_text = Some(String::from_utf8(bytes).map_err(|error| {
                    MaterialReviewRequestError::SourceUnavailable {
                        path: change.path.clone(),
                        reason: error.to_string(),
                    }
                })?);
            }
        }
    }
    let (signals, rust_diagnostics) = derive_governance_signals_with_diagnostics(&snapshot);
    let mut entries = Vec::new();
    let mut manifest_hasher = Sha256::new();
    manifest_hasher.update(b"ai-cockpit:material-manifest:v1\0");
    for change in snapshot
        .change_evidence
        .iter()
        .filter(|change| change.path != ".ai" && !change.path.starts_with(".ai/"))
    {
        let path = root.join(&change.path);
        let after_blob_digest = if change.kind == ChangeKind::Deleted {
            None
        } else {
            let metadata = fs::symlink_metadata(&path).map_err(|error| {
                MaterialReviewRequestError::SourceUnavailable {
                    path: change.path.clone(),
                    reason: error.to_string(),
                }
            })?;
            if !metadata.file_type().is_file() {
                return Err(MaterialReviewRequestError::SourceUnavailable {
                    path: change.path.clone(),
                    reason: "symlink or non-regular source".into(),
                });
            }
            if metadata.len() > MAX_MATERIAL_BLOB_BYTES {
                return Err(MaterialReviewRequestError::SourceUnavailable {
                    path: change.path.clone(),
                    reason: "source exceeds bounded request budget".into(),
                });
            }
            fs::read(&path).map_err(|error| MaterialReviewRequestError::SourceUnavailable {
                path: change.path.clone(),
                reason: error.to_string(),
            })?;
            committed_digests.get(&change.path).cloned()
        };
        let kind = format!("{:?}", change.kind).to_ascii_lowercase();
        let content_state = format!("{:?}", change.content_state).to_ascii_lowercase();
        let origins = change
            .added_line_origins
            .iter()
            .map(|origin| (origin.after_line, origin.hunk_index))
            .collect::<Vec<_>>();
        let changed_hunk_digest =
            json_digest(&(&change.added_lines, &origins, &change.removed_lines))?;
        let diagnosis = rust_diagnostics.get(&change.path);
        let scanner_assessment = if !matches!(
            change.content_state,
            ChangeContentState::Text | ChangeContentState::Deleted
        ) {
            MaterialScannerAssessment::Unknown
        } else if let Some(diagnosis) = diagnosis {
            match diagnosis.assessment {
                RustMaterialAssessment::Clean => MaterialScannerAssessment::Clean,
                RustMaterialAssessment::Finding => MaterialScannerAssessment::Finding,
                RustMaterialAssessment::Unknown => MaterialScannerAssessment::Unknown,
            }
        } else {
            let added_text = change.added_lines.join("\n");
            let material_text = if change.added_lines.is_empty() && change.kind == ChangeKind::Added
            {
                change.after_text.as_deref().unwrap_or("")
            } else {
                &added_text
            };
            if contains_strong_instruction_injection(material_text) {
                MaterialScannerAssessment::Finding
            } else {
                MaterialScannerAssessment::Clean
            }
        };
        let complete_provenance = change.added_lines.len() == change.added_line_origins.len();
        let unknown_cause = if diagnosis
            .is_some_and(|diagnosis| diagnosis.assessment == RustMaterialAssessment::Unknown)
            && !complete_provenance
        {
            Some(MaterialUnknownCause::InvalidProvenance)
        } else if let Some(diagnosis) = diagnosis {
            diagnosis.unknown_cause
        } else if !matches!(
            change.content_state,
            ChangeContentState::Text | ChangeContentState::Deleted
        ) {
            Some(match change.content_state {
                ChangeContentState::Binary => MaterialUnknownCause::NonTextMaterial,
                ChangeContentState::TooLarge => MaterialUnknownCause::SourceOverBudget,
                _ => MaterialUnknownCause::MissingCompleteSource,
            })
        } else {
            None
        };
        let reviewable =
            unknown_cause == Some(MaterialUnknownCause::ReadableCommittedRustSyntaxUnknown);
        let entry = MaterialReviewEntry {
            path: change.path.clone(),
            kind,
            content_state,
            scanner_assessment,
            unknown_cause,
            reviewable,
            changed_hunk_digest,
            after_blob_digest,
        };
        let manifest_entry = ManifestEntry {
            path: &entry.path,
            kind: &entry.kind,
            content_state: &entry.content_state,
            added_lines: &change.added_lines,
            added_line_origins: origins,
            removed_lines: &change.removed_lines,
            changed_hunk_digest: &entry.changed_hunk_digest,
            after_blob_digest: &entry.after_blob_digest,
        };
        let bytes = serde_json::to_vec(&manifest_entry)
            .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?;
        manifest_hasher.update((bytes.len() as u64).to_le_bytes());
        manifest_hasher.update(bytes);
        entries.push(entry);
    }
    let patch_digest = source_patch_digest(root, &contract.base_revision, &head)?;
    manifest_hasher.update(patch_digest.as_str().as_bytes());
    let material_manifest_digest = Digest::sha256_bytes(&manifest_hasher.finalize());
    let policy = effective_policy_for_contract(root, contract)
        .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?;
    let profile = contract
        .material_inspection_review_profile()
        .map_err(MaterialReviewRequestError::Identity)?;
    let profile_digest = profile.as_ref().map(json_digest).transpose()?;
    let review_enabled = profile.is_some()
        && contract
            .required_runtime_capabilities
            .iter()
            .any(|capability| capability == MATERIAL_INSPECTION_REVIEW_CAPABILITY);
    let mut request = MaterialReviewRequest {
        schema_version: 1,
        repository_id: contract.repository_id.clone(),
        work_item_id: contract.work_item_id.clone(),
        contract_digest: json_digest(contract)?,
        immutable_contract_base_revision: contract.base_revision.clone(),
        source_snapshot_digest,
        material_manifest_digest,
        scanner_semantic_version: SCANNER_SEMANTIC_VERSION.into(),
        analysis_implementation_digest: analysis_implementation_digest(),
        analysis_target_semantic_profile: ANALYSIS_TARGET_SEMANTIC_PROFILE.into(),
        material_inspection_review_profile_digest: profile_digest,
        effective_policy_digest: json_digest(&policy)?,
        entries,
        raw_unknown_codes: signals.unknowns,
        blocked_by_finding: !signals.findings.is_empty(),
        review_enabled,
        review_diagnostic: (!review_enabled).then(|| "material_review_not_enabled".into()),
        request_digest: Digest::sha256_bytes(b"uncomputed"),
        reviewed_source_head: head,
    };
    let mut validity = serde_json::to_value(&request)
        .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?;
    let fields = validity
        .as_object_mut()
        .expect("typed request is an object");
    fields.remove("requestDigest");
    fields.remove("reviewedSourceHead");
    fields.remove("reviewDiagnostic");
    request.request_digest = json_digest(&("ai-cockpit:material-review-request:v1", validity))?;
    let final_snapshot = git
        .snapshot()
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    reject_nonstandard_index_flags(root)?;
    let final_dirty = final_snapshot
        .changed_paths
        .iter()
        .any(|path| path != ".ai" && !path.starts_with(".ai/"));
    if final_dirty
        || final_snapshot.head.as_deref() != Some(request.reviewed_source_head.as_str())
        || final_snapshot.source_tree_digest.as_deref() != working.source_tree_digest.as_deref()
    {
        return Err(MaterialReviewRequestError::Identity(
            "source changed during request observation".into(),
        ));
    }
    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn implementation_digest_is_build_independent_and_source_sensitive() {
        let one = implementation_digest_from_sources(&[("scanner.rs", b"same source")]);
        let two = implementation_digest_from_sources(&[("scanner.rs", b"same source")]);
        let changed = implementation_digest_from_sources(&[("scanner.rs", b"changed source")]);
        assert_eq!(one, two);
        assert_ne!(one, changed);
        assert_ne!(
            one,
            implementation_digest_from_sources(&[("different.rs", b"same source")])
        );
    }
}

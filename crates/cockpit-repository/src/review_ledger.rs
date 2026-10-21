//! Local denial-only audit. This chain is not a protected authority ledger.
//! Complete local rollback cannot be detected without B's independent head.
use crate::evidence_store::{
    open_cap_directory_nofollow_strict, read_cap_file_nofollow_bounded, write_cap_immutable,
};
use crate::review_authority::error;
use crate::{ObserverError, legacy_closeout_source::PinnedRoot};
use cap_std::fs::Dir;
use cockpit_core::Digest;
use cockpit_protocol::review_authority::{ReviewCorrectionEvent, ReviewCorrectionRequest};
use serde_json::Value;
use std::path::{Path, PathBuf};

const MAX_BYTES: u64 = 128 * 1024;
const MAX_EVENTS: usize = 256;
const MAX_DIRECTORY_ENTRIES: usize = 16384;

pub(crate) fn local_hold_head(root: &Path, id: &str) -> Result<Option<Digest>, ObserverError> {
    review_correction_events(root, id)?
        .last()
        .map(cockpit_protocol::digest_json)
        .transpose()
        .map_err(|_| error(root, "review_correction_event_invalid"))
}

/// Publication is internal and never turns arbitrary JSON into authority.
pub(crate) fn publish_authority_mirror(
    root: &Path,
    id: &str,
    kind: &str,
    value: &Value,
) -> Result<Digest, ObserverError> {
    if !matches!(kind, "human-approval" | "human-revocation") {
        return Err(error(root, "review_mirror_kind_invalid"));
    }
    crate::validate_work_item_id(id)?;
    let root = std::fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let guard = crate::acquire_lifecycle_lock(&root, id)?;
    let (pinned, directory, path) = open_decisions(&root)?;
    let bytes =
        serde_json::to_vec_pretty(value).map_err(|_| error(&root, "review_mirror_invalid"))?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(error(&root, "review_mirror_budget_exceeded"));
    }
    let digest = Digest::sha256_bytes(&bytes);
    let name = format!(
        "{id}.{kind}.{}.json",
        digest.as_str().trim_start_matches("sha256:")
    );
    guard.require_subject(&root, id)?;
    pinned.require_current()?;
    require_same_parent(&root, &directory)?;
    write_cap_immutable(&directory, &name, &path.join(&name), &bytes)?;
    guard.require_subject(&root, id)?;
    pinned.require_current()?;
    require_same_parent(&root, &directory)?;
    Ok(digest)
}

pub(crate) fn read_authority_mirror(
    root: &Path,
    id: &str,
    kind: &str,
    digest: &Digest,
) -> Result<Value, ObserverError> {
    if !matches!(kind, "human-approval" | "human-revocation") {
        return Err(error(root, "review_mirror_kind_invalid"));
    }
    crate::validate_work_item_id(id)?;
    let root = std::fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let (pinned, directory, path) = open_decisions(&root)?;
    let name = format!(
        "{id}.{kind}.{}.json",
        digest.as_str().trim_start_matches("sha256:")
    );
    let bytes = read_cap_file_nofollow_bounded(&directory, &name, &path.join(&name), MAX_BYTES)?;
    if Digest::sha256_bytes(&bytes) != *digest {
        return Err(error(&root, "review_mirror_digest_mismatch"));
    }
    crate::validate_review_authority_wire(&bytes).map_err(|code| error(&root, &code))?;
    let value =
        serde_json::from_slice(&bytes).map_err(|_| error(&root, "review_mirror_invalid"))?;
    pinned.require_current()?;
    require_same_parent(&root, &directory)?;
    Ok(value)
}

/// Bounded local evidence enumeration. Each result still requires independent
/// protected-source confirmation; these bytes never grant authority.
pub(crate) fn approval_mirrors(
    root: &Path,
    id: &str,
) -> Result<Vec<(Value, Digest)>, ObserverError> {
    crate::validate_work_item_id(id)?;
    let root = std::fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let (pinned, directory, path) = open_decisions(&root)?;
    let prefix = format!("{id}.human-approval.");
    let mut result = Vec::new();
    for name in names(&directory, &path, &prefix)? {
        let suffix = name
            .strip_prefix(&prefix)
            .and_then(|s| s.strip_suffix(".json"))
            .ok_or_else(|| error(&path, "review_mirror_name_invalid"))?;
        let digest = format!("sha256:{suffix}")
            .parse::<Digest>()
            .map_err(|_| error(&path, "review_mirror_name_invalid"))?;
        if name
            != format!(
                "{prefix}{}.json",
                digest.as_str().trim_start_matches("sha256:")
            )
        {
            return Err(error(&path, "review_mirror_name_invalid"));
        }
        let value = read_authority_mirror(&root, id, "human-approval", &digest)?;
        result.push((value, digest));
    }
    pinned.require_current()?;
    require_same_parent(&root, &directory)?;
    Ok(result)
}

fn open_decisions(root: &Path) -> Result<(PinnedRoot, Dir, PathBuf), ObserverError> {
    let pinned = PinnedRoot::open(root)?;
    let directory = pinned.directory()?;
    let ai = open_cap_directory_nofollow_strict(&directory, ".ai", &root.join(".ai"))?;
    let path = root.join(".ai/decisions");
    let decisions = open_cap_directory_nofollow_strict(&ai, "decisions", &path)?;
    pinned.require_current()?;
    Ok((pinned, decisions, path))
}

fn require_same_parent(root: &Path, original: &Dir) -> Result<(), ObserverError> {
    let (_, current, path) = open_decisions(root)?;
    if crate::legacy_closeout_source::directory_identity(&path, original)?
        != crate::legacy_closeout_source::directory_identity(&path, &current)?
    {
        return Err(error(&path, "review_correction_parent_changed"));
    }
    Ok(())
}

fn correction_digest(request: &ReviewCorrectionRequest) -> Result<Digest, ObserverError> {
    cockpit_protocol::digest_json(&serde_json::json!({
        "domain":"ai-cockpit/review-correction/v1", "request":request
    }))
    .map_err(|_| {
        error(
            Path::new("review-correction"),
            "review_correction_digest_invalid",
        )
    })
}

fn names(directory: &Dir, path: &Path, prefix: &str) -> Result<Vec<String>, ObserverError> {
    let entries = directory.entries().map_err(|source| ObserverError::Read {
        path: path.into(),
        source,
    })?;
    let mut result = Vec::new();
    for (count, entry) in entries.enumerate() {
        if count >= MAX_DIRECTORY_ENTRIES {
            return Err(error(path, "review_directory_budget_exceeded"));
        }
        let entry = entry.map_err(|source| ObserverError::Read {
            path: path.into(),
            source,
        })?;
        let os_name = entry.file_name();
        let Some(name) = os_name.to_str() else {
            continue;
        };
        if name.starts_with(prefix) {
            if result.len() >= MAX_EVENTS {
                return Err(error(path, "review_correction_budget_exceeded"));
            }
            result.push(name.to_owned());
        }
    }
    result.sort();
    Ok(result)
}

fn read_events(
    root: &Path,
    id: &str,
    directory: &Dir,
    path: &Path,
) -> Result<Vec<ReviewCorrectionEvent>, ObserverError> {
    let prefix = format!("{id}.review-correction.");
    let mut result = Vec::new();
    let mut previous = None;
    for name in names(directory, path, &prefix)? {
        let bytes = read_cap_file_nofollow_bounded(directory, &name, &path.join(&name), MAX_BYTES)?;
        crate::validate_review_authority_wire(&bytes).map_err(|code| error(path, &code))?;
        let event: ReviewCorrectionEvent = serde_json::from_slice(&bytes)
            .map_err(|_| error(path, "review_correction_event_invalid"))?;
        let digest = cockpit_protocol::digest_json(&event)
            .map_err(|_| error(path, "review_correction_event_invalid"))?;
        let expected = format!(
            "{prefix}{:08}.{}.json",
            result.len() + 1,
            digest.as_str().trim_start_matches("sha256:")
        );
        if name != expected
            || event.schema_version != 1
            || event.sequence != (result.len() + 1) as u64
            || event.parent_digest != previous
            || event.request_digest != correction_digest(&event.request)?
            || event.request.schema_version != 1
            || event.request.work_item_id != id
            || event.request.repository_id != crate::repository_id(root).to_string()
            || event.request.reason.trim().is_empty()
            || event.human_authority_granted
            || event.assurance != "self_declared_denial_only"
            || event.recorded_by != "unverified_recorder"
            || chrono::DateTime::parse_from_rfc3339(&event.recorded_at).is_err()
        {
            return Err(error(path, "review_correction_chain_invalid"));
        }
        previous = Some(digest);
        result.push(event);
    }
    Ok(result)
}

pub fn review_correction_events(
    root: &Path,
    id: &str,
) -> Result<Vec<ReviewCorrectionEvent>, ObserverError> {
    crate::validate_work_item_id(id)?;
    let root = std::fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let (pinned, directory, path) = open_decisions(&root)?;
    let result = read_events(&root, id, &directory, &path)?;
    pinned.require_current()?;
    require_same_parent(&root, &directory)?;
    Ok(result)
}

/// Append a narrowly bound correction, without changing Summary or any old
/// receipt. It records no authenticated identity and can never unlock work.
pub fn record_review_correction(
    root: &Path,
    id: &str,
    request: &ReviewCorrectionRequest,
) -> Result<Value, ObserverError> {
    crate::validate_work_item_id(id)?;
    let root = std::fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    if request.schema_version != 1
        || request.work_item_id != id
        || request.repository_id != crate::repository_id(&root).to_string()
        || request.reason.trim().is_empty()
        || request.reason.len() > 16384
    {
        return Err(error(&root, "review_correction_binding_invalid"));
    }
    // This is a denial-only action, not a successful continuation based on a
    // prior `allowed`. Acquiring the same WI lock does not approve the WI.
    let guard = crate::acquire_lifecycle_lock(&root, id)?;
    let (pinned, directory, path) = open_decisions(&root)?;
    let events = read_events(&root, id, &directory, &path)?;
    let request_digest = correction_digest(request)?;
    if let Some(existing) = events
        .iter()
        .find(|event| event.request_digest == request_digest)
    {
        guard.require_subject(&root, id)?;
        require_same_parent(&root, &directory)?;
        return serde_json::to_value(existing)
            .map_err(|_| error(&root, "review_correction_event_invalid"));
    }
    if events.len() >= MAX_EVENTS {
        return Err(error(&root, "review_correction_budget_exceeded"));
    }
    let mut targets = 0;
    let mut target_names = names(&directory, &path, &format!("{id}.preflight-review"))?;
    target_names.extend(names(&directory, &path, &format!("{id}.human-approval."))?);
    for name in target_names {
        if !name.ends_with(".json") {
            return Err(error(&root, "review_correction_target_invalid"));
        }
        let bytes =
            read_cap_file_nofollow_bounded(&directory, &name, &path.join(&name), MAX_BYTES)?;
        if Digest::sha256_bytes(&bytes) != request.target_receipt_raw_digest {
            continue;
        }
        crate::validate_review_authority_wire(&bytes).map_err(|code| error(&root, &code))?;
        let (repository_id, work_item_id) = if name.starts_with(&format!("{id}.human-approval.")) {
            let receipt: cockpit_protocol::review_authority::HumanApprovalReceipt =
                serde_json::from_slice(&bytes)
                    .map_err(|_| error(&root, "review_correction_target_invalid"))?;
            (
                receipt.request.binding.repository_id,
                receipt.request.binding.work_item_id,
            )
        } else {
            let receipt: cockpit_protocol::PreflightDecisionEvidence =
                serde_json::from_slice(&bytes)
                    .map_err(|_| error(&root, "review_correction_target_invalid"))?;
            (receipt.repository_id, receipt.work_item_id)
        };
        if repository_id != request.repository_id || work_item_id != id {
            return Err(error(&root, "review_correction_target_identity_mismatch"));
        }
        targets += 1;
    }
    if targets != 1 {
        return Err(error(
            &root,
            "review_correction_target_missing_or_ambiguous",
        ));
    }
    let parent_digest = events
        .last()
        .map(cockpit_protocol::digest_json)
        .transpose()
        .map_err(|_| error(&root, "review_correction_event_invalid"))?;
    let event = ReviewCorrectionEvent {
        schema_version: 1,
        sequence: events.len() as u64 + 1,
        parent_digest,
        request_digest,
        request: request.clone(),
        recorded_at: chrono::Utc::now().to_rfc3339(),
        recorded_by: "unverified_recorder".into(),
        assurance: "self_declared_denial_only".into(),
        human_authority_granted: false,
    };
    let digest = cockpit_protocol::digest_json(&event)
        .map_err(|_| error(&root, "review_correction_event_invalid"))?;
    let name = format!(
        "{id}.review-correction.{:08}.{}.json",
        event.sequence,
        digest.as_str().trim_start_matches("sha256:")
    );
    let bytes = serde_json::to_vec_pretty(&event)
        .map_err(|_| error(&root, "review_correction_event_invalid"))?;
    guard.require_subject(&root, id)?;
    pinned.require_current()?;
    require_same_parent(&root, &directory)?;
    write_cap_immutable(&directory, &name, &path.join(&name), &bytes)?;
    guard.require_subject(&root, id)?;
    pinned.require_current()?;
    require_same_parent(&root, &directory)?;
    // Read the complete chain; incomplete or conflicting publication never
    // becomes an implicit approval. No rollback/delete of an installed event.
    let after = read_events(&root, id, &directory, &path)?;
    if after.last() != Some(&event) {
        return Err(error(&root, "review_correction_publication_changed"));
    }
    serde_json::to_value(event).map_err(|_| error(&root, "review_correction_event_invalid"))
}

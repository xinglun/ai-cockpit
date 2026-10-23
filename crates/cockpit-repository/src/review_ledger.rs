//! Local denial-only audit. This chain is not a protected authority ledger.
//! Complete local rollback cannot be detected without B's independent head.
use crate::evidence_store::{
    open_cap_directory_nofollow_strict, read_cap_file_nofollow_bounded, write_cap_immutable,
};
use crate::{ObserverError, legacy_closeout_source::PinnedRoot};
use cap_std::fs::Dir;
use cockpit_core::Digest;
use cockpit_protocol::review_authority::{ReviewCorrectionEvent, ReviewCorrectionRequest};
use serde_json::Value;
use std::path::{Path, PathBuf};

const MAX_BYTES: u64 = 128 * 1024;
const MAX_EVENTS: usize = 256;
const MAX_DIRECTORY_ENTRIES: usize = 16384;
const MAX_REASON_BYTES: usize = 16384;

fn error(root: &Path, code: &str) -> ObserverError {
    ObserverError::State {
        path: root.into(),
        message: code.into(),
    }
}

// Preserve the frozen candidate's wire guard using the already accepted
// recursive duplicate-key/end-input visitor, without importing authority code.
fn validate_wire(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > 128 * 1024 {
        return Err("review_wire_budget_exceeded".into());
    }
    crate::reject_duplicate_json_keys(bytes).map_err(|e| format!("review_duplicate_json_key: {e}"))
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
        validate_wire(&bytes).map_err(|code| error(path, &code))?;
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
            || event.request.reason.len() > MAX_REASON_BYTES
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
    let guard = crate::acquire_lifecycle_lock(&root, id)?;
    let (pinned, directory, path) = open_decisions(&root)?;
    let result = read_events(&root, id, &directory, &path)?;
    guard.require_subject(&root, id)?;
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
        || request.reason.len() > MAX_REASON_BYTES
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
        validate_wire(&bytes).map_err(|code| error(&root, &code))?;
        let (repository_id, work_item_id) = if name.starts_with(&format!("{id}.human-approval.")) {
            let receipt: cockpit_protocol::review_authority::HumanApprovalReceipt =
                serde_json::from_slice(&bytes)
                    .map_err(|_| error(&root, "review_correction_target_invalid"))?;
            if receipt.schema_version != 2 || receipt.request.schema_version != 2 {
                return Err(error(&root, "review_correction_target_invalid"));
            }
            (
                receipt.request.binding.repository_id,
                receipt.request.binding.work_item_id,
            )
        } else {
            let receipt: cockpit_protocol::PreflightDecisionEvidence =
                serde_json::from_slice(&bytes)
                    .map_err(|_| error(&root, "review_correction_target_invalid"))?;
            if receipt.schema_version != 1 {
                return Err(error(&root, "review_correction_target_invalid"));
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn original_wire_rejects_duplicate_keys_at_every_depth() {
        for bytes in [
            br#"{"a":1,"a":2}"#.as_slice(),
            br#"{"outer":{"a":1,"a":2}}"#.as_slice(),
            br#"[{"a":1,"a":2}]"#.as_slice(),
        ] {
            assert!(
                validate_wire(bytes)
                    .expect_err("duplicate key")
                    .contains("duplicate")
            );
        }
        validate_wire(br#"{"a":1,"outer":{"a":2}}"#).expect("keys in distinct objects");
    }

    #[test]
    fn original_wire_rejects_trailing_and_malformed_input() {
        for bytes in [
            b"{} {}".as_slice(),
            b"{} trailing".as_slice(),
            b"{".as_slice(),
        ] {
            assert!(validate_wire(bytes).is_err());
        }
        validate_wire(b"{} \n\t").expect("JSON whitespace only");
    }

    #[test]
    fn original_wire_budget_is_checked_before_parsing() {
        let mut bytes = b"null".to_vec();
        bytes.resize(MAX_BYTES as usize, b' ');
        validate_wire(&bytes).expect("exact original byte limit");
        bytes.push(b' ');
        assert_eq!(
            validate_wire(&bytes),
            Err("review_wire_budget_exceeded".into())
        );
    }

    fn directory() -> tempfile::TempDir {
        let root = tempfile::tempdir().expect("temporary root");
        fs::create_dir_all(root.path().join(".ai/decisions")).expect("decisions directory");
        root
    }

    #[test]
    fn immutable_writer_preserves_original_on_replay_and_conflict() {
        let root = directory();
        let (_, directory, path) = open_decisions(root.path()).expect("pinned decisions");
        let file = path.join("fixture.json");
        write_cap_immutable(&directory, "fixture.json", &file, b"original")
            .expect("initial immutable publication");
        write_cap_immutable(&directory, "fixture.json", &file, b"original")
            .expect("exact same bytes");
        assert!(write_cap_immutable(&directory, "fixture.json", &file, b"different").is_err());
        assert_eq!(fs::read(file).expect("retained original"), b"original");
    }

    #[cfg(unix)]
    #[test]
    fn captured_parent_replacement_is_rejected_without_deleting_old_bytes() {
        let root = directory();
        let path = root.path().join(".ai/decisions");
        fs::write(path.join("original.json"), b"original evidence").expect("old bytes");
        let (_, original, _) = open_decisions(root.path()).expect("capture real parent handle");
        let retained = root.path().join(".ai/retained-decisions");
        fs::rename(&path, &retained).expect("replace parent after capture");
        fs::create_dir(&path).expect("new parent");
        let error =
            require_same_parent(root.path(), &original).expect_err("parent identity changed");
        assert!(
            error
                .to_string()
                .contains("review_correction_parent_changed")
        );
        assert_eq!(
            fs::read(retained.join("original.json")).expect("old evidence"),
            b"original evidence"
        );
        assert!(!path.join("original.json").exists());
    }
}

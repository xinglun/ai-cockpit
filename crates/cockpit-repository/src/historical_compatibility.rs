use super::{
    ordinary_cleanup_binding_from_decision, read_json, status_projection::discover_worktree_layout,
};
use chrono::DateTime;
use cockpit_protocol::{
    Contract, HumanDecision, ResourceFinalizationDisposition, ResourceFinalizationReceipt,
};
use std::fs;
use std::path::Path;

/// Validate the close receipt before exposing a terminal `closed` status.
/// Merely finding a decision file is not enough: the record must be a regular
/// repository-local file with the same Work Item identity, a confirmed closed
/// state, and a strict structured human decision whose summary agrees with
/// the structured value. Invalid records remain visible as unknowns and can
/// never promote an archived Work Item to `closed`.
pub(crate) fn close_decision_is_valid_for_status(
    root: &Path,
    work_item_id: &str,
    repository_id: &str,
) -> bool {
    let path = root
        .join(".ai/decisions")
        .join(format!("{work_item_id}.close.json"));
    let Ok(metadata) = fs::symlink_metadata(&path) else {
        return false;
    };
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return false;
    }
    let Ok(value) = read_json(&path) else {
        return false;
    };
    if value.get("workItemId").and_then(serde_json::Value::as_str) != Some(work_item_id)
        || value
            .get("repositoryId")
            .and_then(serde_json::Value::as_str)
            != Some(repository_id)
        || value.get("state").and_then(serde_json::Value::as_str) != Some("closed")
        || value
            .get("decisionState")
            .and_then(serde_json::Value::as_str)
            != Some("confirmed")
    {
        return false;
    }
    let Some(structured) = value.get("structuredDecision").cloned() else {
        return false;
    };
    let Ok(decision) = serde_json::from_value::<HumanDecision>(structured) else {
        return false;
    };
    if [
        decision.decision.as_str(),
        decision.actor.as_str(),
        decision.authority_source.as_str(),
        decision.reason.as_str(),
        decision.decided_at.as_str(),
    ]
    .iter()
    .any(|value| value.trim().is_empty())
    {
        return false;
    }
    if value
        .get("humanDecision")
        .and_then(serde_json::Value::as_str)
        != Some(decision.decision.as_str())
    {
        return false;
    }
    if is_canonical_close_decision(&decision.decision) {
        return true;
    }

    historical_legacy_close_decision_is_valid(root, &value, work_item_id, repository_id)
}

/// 旧 Runtime 的 close receipt 仅在完整 Outcome binding 保留时作为
/// historical compatibility 接受。当前 close 仍要求 canonical vocabulary。
fn historical_legacy_close_decision_is_valid(
    root: &Path,
    value: &serde_json::Value,
    work_item_id: &str,
    repository_id: &str,
) -> bool {
    let Some(structured) = value.get("structuredDecision") else {
        return false;
    };
    // A complete report proves the contents were not tampered with, but it
    // does not prove that a non-canonical decision came from an older
    // Runtime. Restrict compatibility to the explicit marker emitted by the
    // legacy CLI; current human decisions with a changed token remain invalid.
    if structured.get("actor").and_then(serde_json::Value::as_str) != Some("legacy-cli")
        || structured
            .get("authoritySource")
            .and_then(serde_json::Value::as_str)
            != Some("explicit-cli")
    {
        return false;
    }
    let Some(final_report) = value.get("finalReport") else {
        return false;
    };
    if final_report
        .get("status")
        .and_then(serde_json::Value::as_str)
        != Some("verified")
        || final_report
            .get("workItemId")
            .and_then(serde_json::Value::as_str)
            != Some(work_item_id)
        || final_report
            .get("humanStatusColor")
            .and_then(serde_json::Value::as_str)
            != Some("green")
        || final_report
            .get("bindings")
            .and_then(|bindings| bindings.get("workItemId"))
            .and_then(serde_json::Value::as_str)
            != Some(work_item_id)
        || final_report
            .get("bindings")
            .and_then(|bindings| bindings.get("repositoryId"))
            .and_then(serde_json::Value::as_str)
            != Some(repository_id)
    {
        return false;
    }
    let Some(expected_digest) = value
        .get("finalReportDigest")
        .and_then(serde_json::Value::as_str)
    else {
        return false;
    };
    let Ok(actual_digest) = cockpit_protocol::digest_json(final_report) else {
        return false;
    };
    if expected_digest != actual_digest.to_string() {
        return false;
    }
    if (value.get("ordinaryCleanupBinding").is_some()
        || value.get("ordinaryCleanupBindingDigest").is_some())
        && ordinary_cleanup_binding_from_decision(root, work_item_id, repository_id, value).is_err()
    {
        return false;
    }
    value
        .get("timestamp")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|timestamp| DateTime::parse_from_rfc3339(timestamp).is_ok())
}

/// Return the finite vocabulary accepted by the close lifecycle boundary.
/// Free-form prose remains in `reason`; the read-only status projection has a
/// separate, evidence-bound compatibility path for complete older receipts.
pub(crate) fn canonical_close_decisions() -> &'static [&'static str] {
    &[
        "approved",
        "confirmed",
        "rejected",
        "superseded",
        "superseded_failed_delivery",
    ]
}

pub(crate) fn is_canonical_close_decision(decision: &str) -> bool {
    canonical_close_decisions().contains(&decision.trim())
}

/// Return true only for a readable, regular legacy evidence file. Malformed
/// v2 JSON, symlinks, and v2 records with missing nested identity remain
/// contradictory/red; current corruption cannot hide behind this projection.
pub(crate) fn legacy_verification_evidence(root: &Path, work_item_id: &str) -> bool {
    let path = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.verification.json"));
    let Ok(metadata) = fs::symlink_metadata(&path) else {
        return false;
    };
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return false;
    }
    let Ok(value) = read_json(&path) else {
        return false;
    };
    let Some(object) = value.as_object() else {
        return false;
    };
    object.get("evidenceSchemaVersion").is_none()
}

/// Infer the narrow compatibility classification for a legacy finalization
/// receipt that predates the explicit `historical` field. A linked worktree
/// or an external provider is never accepted by this projection.
pub(crate) fn infer_legacy_shared_worktree_retained(
    root: &Path,
    receipt: &ResourceFinalizationReceipt,
    contract: &Contract,
) -> bool {
    if receipt.historical.is_some()
        || !matches!(
            receipt.result.disposition,
            ResourceFinalizationDisposition::Retained
        )
        || receipt.provider != "local"
        || !matches!(
            receipt.before.pull_request,
            cockpit_protocol::ResourceFinalizationPullRequestState::Merged
        )
        || !matches!(
            receipt.after.pull_request,
            cockpit_protocol::ResourceFinalizationPullRequestState::Merged
        )
        || receipt.before.branch != receipt.after.branch
        || receipt.before.worktree != receipt.after.worktree
        || !matches!(
            receipt.after.branch,
            cockpit_protocol::ResourceFinalizationBranchState::Present
        )
        || !matches!(
            receipt.after.worktree,
            cockpit_protocol::ResourceFinalizationWorktreeState::Clean
        )
        || receipt.branch.name != receipt.worktree.branch
    {
        return false;
    }
    let Some(context) = receipt.resource_context.as_ref() else {
        return false;
    };
    if context.provider != "local"
        || contract.resource_context.as_ref() != Some(context)
        || context.branch != receipt.branch.name
        || context.worktree != receipt.worktree.path
    {
        return false;
    }
    let Ok(root) = fs::canonicalize(root) else {
        return false;
    };
    let Ok(layout) = discover_worktree_layout(&root) else {
        return false;
    };
    if layout.primary != root {
        return false;
    }
    fs::canonicalize(&context.worktree).ok().as_ref() == Some(&root)
        && fs::canonicalize(&receipt.worktree.path).ok().as_ref() == Some(&root)
}

#[cfg(test)]
mod tests {
    use super::{canonical_close_decisions, legacy_verification_evidence};
    use std::fs;

    #[test]
    fn canonical_close_vocabulary_remains_explicit_and_finite() {
        assert_eq!(
            canonical_close_decisions(),
            &[
                "approved",
                "confirmed",
                "rejected",
                "superseded",
                "superseded_failed_delivery",
            ]
        );
    }

    #[test]
    fn legacy_evidence_projection_rejects_current_schema_and_malformed_records() {
        let directory = tempfile::tempdir().expect("temporary repository");
        let evidence = directory
            .path()
            .join(".ai/evidence/WI-legacy.verification.json");
        fs::create_dir_all(evidence.parent().expect("evidence parent")).expect("evidence dir");

        fs::write(&evidence, br#"{"status":"verified"}"#).expect("legacy evidence");
        assert!(legacy_verification_evidence(directory.path(), "WI-legacy"));

        fs::write(
            &evidence,
            br#"{"evidenceSchemaVersion":2,"status":"verified"}"#,
        )
        .expect("current evidence");
        assert!(!legacy_verification_evidence(directory.path(), "WI-legacy"));

        fs::write(&evidence, b"not-json").expect("malformed evidence");
        assert!(!legacy_verification_evidence(directory.path(), "WI-legacy"));
    }
}

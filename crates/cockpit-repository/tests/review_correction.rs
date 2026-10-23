use cockpit_core::Digest;
use cockpit_protocol::review_authority::ReviewCorrectionRequest;
use cockpit_repository::{
    attach, record_review_correction, repository_id, review_correction_events,
};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const ID: &str = "WI-CORRECTION-AUDIT";

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("temporary repository");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    attach(directory.path()).expect("attach real repository");
    directory
}

fn target(root: &Path) -> (ReviewCorrectionRequest, PathBuf, Vec<u8>) {
    let receipt = json!({
        "schemaVersion": 1,
        "decisionId": "contract-preflight-review",
        "decision": "confirm_review",
        "workItemId": ID,
        "repositoryId": repository_id(root),
        "contractDigest": Digest::sha256_bytes(b"synthetic Contract"),
        "preflightDecisionDigest": Digest::sha256_bytes(b"synthetic preflight"),
        "repositorySnapshotDigest": Digest::sha256_bytes(b"synthetic snapshot"),
        "recordedAt": "2026-10-23T00:00:00Z",
        "recordedBy": "unverified synthetic fixture",
        "reason": "Fixture bytes are not independently authenticated human evidence."
    });
    let path = root.join(format!(".ai/decisions/{ID}.preflight-review.json"));
    fs::create_dir_all(path.parent().expect("decisions directory")).expect("create decisions");
    let bytes = serde_json::to_vec_pretty(&receipt).expect("fixture JSON");
    fs::write(&path, &bytes).expect("write synthetic target");
    (
        ReviewCorrectionRequest {
            schema_version: 1,
            repository_id: repository_id(root).to_string(),
            work_item_id: ID.into(),
            target_receipt_raw_digest: Digest::sha256_bytes(&bytes),
            reason: "Record that this local receipt must not be used as authenticated approval."
                .into(),
        },
        path,
        bytes,
    )
}

#[test]
fn correction_is_denial_only_immutable_and_idempotent() {
    let directory = repository();
    let root = directory.path();
    let (request, path, original) = target(root);
    let summary = root.join(format!(".ai/work-items/active/{ID}.summary.json"));
    fs::create_dir_all(summary.parent().expect("active directory")).expect("create active");
    let summary_bytes = b"{\"sentinel\":\"preserve original Summary bytes\"}\n";
    fs::write(&summary, summary_bytes).expect("old Summary");

    let first = record_review_correction(root, ID, &request).expect("append local correction");
    assert_eq!(first["humanAuthorityGranted"], false);
    assert_eq!(first["assurance"], "self_declared_denial_only");
    assert_eq!(first["recordedBy"], "unverified_recorder");
    assert_eq!(first["sequence"], 1);
    let events = review_correction_events(root, ID).expect("read public correction chain");
    assert_eq!(events.len(), 1);
    assert_eq!(serde_json::to_value(&events[0]).expect("event JSON"), first);
    assert_eq!(events[0].request, request);
    assert_eq!(fs::read(&path).expect("target after append"), original);
    assert_eq!(fs::read(&summary).expect("Summary after append"), summary_bytes);

    let replay = record_review_correction(root, ID, &request).expect("idempotent replay");
    assert_eq!(replay, first);
    assert_eq!(review_correction_events(root, ID).expect("replay chain").len(), 1);
    assert_eq!(fs::read(&path).expect("target after replay"), original);
    assert_eq!(fs::read(&summary).expect("Summary after replay"), summary_bytes);
}

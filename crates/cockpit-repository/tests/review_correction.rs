use cockpit_core::Digest;
use cockpit_protocol::review_authority::{
    HumanApprovalReceipt, HumanReviewBinding, HumanReviewKind, HumanReviewOperation,
    HumanReviewRequest, ReviewCorrectionRequest, review_operation_digest,
};
use cockpit_repository::{
    attach, record_review_correction, repository_id, review_correction_events,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
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

fn inventory(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fs::read_dir(root.join(".ai/decisions"))
        .expect("list decisions")
        .map(|entry| {
            let entry = entry.expect("directory entry");
            let path = entry.path();
            let bytes = if fs::symlink_metadata(&path).expect("entry metadata").is_symlink() {
                fs::read_link(&path).expect("link destination").to_string_lossy().as_bytes().to_vec()
            } else {
                fs::read(&path).expect("decision bytes")
            };
            (PathBuf::from(entry.file_name()), bytes)
        })
        .collect()
}

fn append_is_denied(root: &Path, id: &str, request: &ReviewCorrectionRequest) -> String {
    let before = inventory(root);
    let error = record_review_correction(root, id, request).expect_err("must deny publication");
    assert_eq!(inventory(root), before, "no decision bytes changed by rejection");
    error.to_string()
}

fn human_target(root: &Path, request: &mut ReviewCorrectionRequest) -> PathBuf {
    let operation = HumanReviewOperation::PreflightReview;
    let payload = json!({"decision":"confirm_review","fixtureOnly":true});
    let human_request = HumanReviewRequest {
        schema_version: 2,
        kind: HumanReviewKind::HumanReview,
        issuer: "unverified fixture issuer claim".into(),
        request_id: "synthetic request".into(),
        nonce: "synthetic nonce".into(),
        authority_epoch: 1,
        issued_at_millis: 0,
        expires_at_millis: 1,
        binding: HumanReviewBinding {
            repository_id: repository_id(root).to_string(),
            work_item_id: ID.into(),
            checkout_path: root.to_string_lossy().into_owned(),
            contract_digest: Digest::sha256_bytes(b"fixture Contract"),
            preflight_decision_digest: Some(Digest::sha256_bytes(b"fixture preflight")),
            repository_snapshot_digest: Digest::sha256_bytes(b"fixture snapshot"),
            operation_digest: review_operation_digest(&operation, &payload).expect("model digest"),
            operation,
            scope: vec!["src/**".into()],
            operation_payload: payload,
            operation_context: json!({"fixtureOnly":true}),
            local_hold_digest: None,
            activation_recovery_digest: None,
        },
    };
    let receipt = HumanApprovalReceipt {
        schema_version: 2,
        request_digest: human_request.digest().expect("request digest"),
        request: human_request,
        approval_id: "synthetic approval claim, not an issued grant".into(),
        source_event_id: "synthetic source claim".into(),
        authenticated_subject: "unverified fixture claim, not authentication".into(),
        authority_epoch: 1,
        resolved_local_hold: None,
    };
    let bytes = serde_json::to_vec_pretty(&receipt).expect("model receipt");
    request.target_receipt_raw_digest = Digest::sha256_bytes(&bytes);
    let name = format!("{ID}.human-approval.{}.json", request.target_receipt_raw_digest.as_str().trim_start_matches("sha256:"));
    let path = root.join(".ai/decisions").join(name);
    fs::write(&path, bytes).expect("synthetic model target");
    path
}

#[test]
fn v2_human_receipt_can_be_audited_without_authenticating_or_granting_it() {
    let directory = repository();
    let root = directory.path();
    let (mut request, _, _) = target(root);
    let path = human_target(root, &mut request);
    let original = fs::read(&path).expect("original model bytes");
    let result = record_review_correction(root, ID, &request).expect("v2 model denial-only audit");
    assert_eq!(result["humanAuthorityGranted"], false);
    assert_eq!(result["recordedBy"], "unverified_recorder");
    assert_eq!(result["assurance"], "self_declared_denial_only");
    assert_eq!(review_correction_events(root, ID).expect("chain")[0].request, request);
    assert_eq!(fs::read(path).expect("target after audit"), original);
}

#[test]
fn invalid_request_binding_and_missing_target_publish_nothing() {
    let directory = repository();
    let root = directory.path();
    let (request, _, _) = target(root);
    let mut cases = Vec::new();
    let mut changed = request.clone(); changed.schema_version = 2; cases.push(changed);
    let mut changed = request.clone(); changed.repository_id = Digest::sha256_bytes(b"foreign repository").to_string(); cases.push(changed);
    let mut changed = request.clone(); changed.work_item_id = "WI-FOREIGN".into(); cases.push(changed);
    let mut changed = request.clone(); changed.reason = " \n\t".into(); cases.push(changed);
    let mut changed = request.clone(); changed.reason = "x".repeat(16385); cases.push(changed);
    let mut changed = request.clone(); changed.target_receipt_raw_digest = Digest::sha256_bytes(b"absent target"); cases.push(changed);
    for changed in cases { append_is_denied(root, ID, &changed); }
    append_is_denied(root, "../escape", &request);
    assert!(review_correction_events(root, "../escape").is_err());
    assert!(review_correction_events(root, ID).expect("no events").is_empty());
}

#[test]
fn wrong_target_schema_identity_and_unknown_fields_are_rejected() {
    for field in ["schemaVersion", "repositoryId", "workItemId", "unexpectedField"] {
        let directory = repository();
        let root = directory.path();
        let (mut request, path, original) = target(root);
        let mut value: Value = serde_json::from_slice(&original).expect("fixture value");
        value[field] = match field {
            "schemaVersion" => json!(99),
            "repositoryId" => json!(Digest::sha256_bytes(b"other repository")),
            "workItemId" => json!("WI-OTHER"),
            _ => json!(true),
        };
        let bytes = serde_json::to_vec_pretty(&value).expect("fault bytes");
        request.target_receipt_raw_digest = Digest::sha256_bytes(&bytes);
        fs::write(&path, &bytes).expect("inject target fault");
        append_is_denied(root, ID, &request);
        assert_eq!(fs::read(path).expect("fault retained"), bytes);
    }
}

#[test]
fn duplicate_trailing_malformed_and_over_budget_original_target_bytes_are_rejected() {
    for kind in ["duplicate", "trailing", "malformed", "over-budget"] {
        let directory = repository();
        let root = directory.path();
        let (mut request, path, original) = target(root);
        let bytes = match kind {
            "duplicate" => [b"{\"schemaVersion\":1,".as_slice(), &original[1..]].concat(),
            "trailing" => [original.as_slice(), b" {}".as_slice()].concat(),
            "malformed" => b"{".to_vec(),
            _ => { let mut bytes = original; bytes.resize(128 * 1024 + 1, b' '); bytes },
        };
        request.target_receipt_raw_digest = Digest::sha256_bytes(&bytes);
        fs::write(&path, &bytes).expect("fault bytes");
        append_is_denied(root, ID, &request);
        assert_eq!(fs::read(path).expect("fault retained"), bytes);
    }
}

#[test]
fn exact_original_wire_budget_and_request_raw_digest_are_preserved() {
    let directory = repository();
    let root = directory.path();
    let (mut request, path, mut bytes) = target(root);
    bytes.resize(128 * 1024, b' ');
    request.target_receipt_raw_digest = Digest::sha256_bytes(&bytes);
    fs::write(&path, &bytes).expect("exact budget target");
    record_review_correction(root, ID, &request).expect("exact byte budget");
    assert_eq!(review_correction_events(root, ID).expect("chain")[0].request.target_receipt_raw_digest, Digest::sha256_bytes(&bytes));
    assert_eq!(fs::read(path).expect("original target"), bytes);
}

#[test]
fn equivalent_json_reformat_does_not_satisfy_the_original_raw_binding() {
    let directory = repository();
    let root = directory.path();
    let (mut request, path, original) = target(root);
    let value: Value = serde_json::from_slice(&original).expect("target JSON");
    let compact = serde_json::to_vec(&value).expect("equivalent compact JSON");
    assert_ne!(Digest::sha256_bytes(&compact), request.target_receipt_raw_digest);
    fs::write(&path, &compact).expect("replace target with equivalent JSON");
    append_is_denied(root, ID, &request);
    request.target_receipt_raw_digest = Digest::sha256_bytes(&compact);
    record_review_correction(root, ID, &request).expect("explicit new raw binding");
    assert_eq!(fs::read(path).expect("compact retained"), compact);
}

#[test]
fn reason_limit_counts_utf8_bytes_and_accepts_the_exact_boundary() {
    for reason in ["x".repeat(16384), "é".repeat(8192)] {
        let directory = repository();
        let root = directory.path();
        let (mut request, _, _) = target(root);
        request.reason = reason;
        record_review_correction(root, ID, &request).expect("exact UTF-8 byte limit");
        request.reason.push('é');
        append_is_denied(root, ID, &request);
        assert_eq!(review_correction_events(root, ID).expect("chain").len(), 1);
    }
}

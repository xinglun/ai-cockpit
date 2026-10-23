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
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

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
    assert_eq!(
        fs::read(&summary).expect("Summary after append"),
        summary_bytes
    );

    let replay = record_review_correction(root, ID, &request).expect("idempotent replay");
    assert_eq!(replay, first);
    assert_eq!(
        review_correction_events(root, ID)
            .expect("replay chain")
            .len(),
        1
    );
    assert_eq!(fs::read(&path).expect("target after replay"), original);
    assert_eq!(
        fs::read(&summary).expect("Summary after replay"),
        summary_bytes
    );
}

fn inventory(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fs::read_dir(root.join(".ai/decisions"))
        .expect("list decisions")
        .map(|entry| {
            let entry = entry.expect("directory entry");
            let path = entry.path();
            let bytes = if fs::symlink_metadata(&path)
                .expect("entry metadata")
                .is_symlink()
            {
                fs::read_link(&path)
                    .expect("link destination")
                    .to_string_lossy()
                    .as_bytes()
                    .to_vec()
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
    assert_eq!(
        inventory(root),
        before,
        "no decision bytes changed by rejection"
    );
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
    let name = format!(
        "{ID}.human-approval.{}.json",
        request
            .target_receipt_raw_digest
            .as_str()
            .trim_start_matches("sha256:")
    );
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
    assert_eq!(
        review_correction_events(root, ID).expect("chain")[0].request,
        request
    );
    assert_eq!(fs::read(path).expect("target after audit"), original);
}

#[test]
fn invalid_request_binding_and_missing_target_publish_nothing() {
    let directory = repository();
    let root = directory.path();
    let (request, _, _) = target(root);
    let mut cases = Vec::new();
    let mut changed = request.clone();
    changed.schema_version = 2;
    cases.push(changed);
    let mut changed = request.clone();
    changed.repository_id = Digest::sha256_bytes(b"foreign repository").to_string();
    cases.push(changed);
    let mut changed = request.clone();
    changed.work_item_id = "WI-FOREIGN".into();
    cases.push(changed);
    let mut changed = request.clone();
    changed.reason = " \n\t".into();
    cases.push(changed);
    let mut changed = request.clone();
    changed.reason = "x".repeat(16385);
    cases.push(changed);
    let mut changed = request.clone();
    changed.target_receipt_raw_digest = Digest::sha256_bytes(b"absent target");
    cases.push(changed);
    for changed in cases {
        append_is_denied(root, ID, &changed);
    }
    append_is_denied(root, "../escape", &request);
    assert!(review_correction_events(root, "../escape").is_err());
    assert!(
        review_correction_events(root, ID)
            .expect("no events")
            .is_empty()
    );
}

#[test]
fn wrong_target_schema_identity_and_unknown_fields_are_rejected() {
    for field in [
        "schemaVersion",
        "repositoryId",
        "workItemId",
        "unexpectedField",
    ] {
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
            _ => {
                let mut bytes = original;
                bytes.resize(128 * 1024 + 1, b' ');
                bytes
            }
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
    assert_eq!(
        review_correction_events(root, ID).expect("chain")[0]
            .request
            .target_receipt_raw_digest,
        Digest::sha256_bytes(&bytes)
    );
    assert_eq!(fs::read(path).expect("original target"), bytes);
}

#[test]
fn equivalent_json_reformat_does_not_satisfy_the_original_raw_binding() {
    let directory = repository();
    let root = directory.path();
    let (mut request, path, original) = target(root);
    let value: Value = serde_json::from_slice(&original).expect("target JSON");
    let compact = serde_json::to_vec(&value).expect("equivalent compact JSON");
    assert_ne!(
        Digest::sha256_bytes(&compact),
        request.target_receipt_raw_digest
    );
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

fn event_path(root: &Path) -> PathBuf {
    fs::read_dir(root.join(".ai/decisions"))
        .expect("decisions")
        .map(|entry| entry.expect("entry").path())
        .find(|path| {
            path.file_name()
                .expect("name")
                .to_string_lossy()
                .starts_with(&format!("{ID}.review-correction."))
        })
        .expect("published event")
}

fn damaged_chain_is_denied(root: &Path, request: &ReviewCorrectionRequest) {
    let before = inventory(root);
    assert!(review_correction_events(root, ID).is_err());
    assert_eq!(inventory(root), before);
    append_is_denied(root, ID, request);
}

#[test]
fn unsupported_human_model_versions_do_not_become_targets() {
    for nested in [false, true] {
        let directory = repository();
        let root = directory.path();
        let (mut request, _, _) = target(root);
        let path = human_target(root, &mut request);
        let mut value: Value =
            serde_json::from_slice(&fs::read(&path).expect("model bytes")).expect("model JSON");
        if nested {
            value["request"]["schemaVersion"] = json!(99);
        } else {
            value["schemaVersion"] = json!(99);
        }
        let bytes = serde_json::to_vec_pretty(&value).expect("fault bytes");
        request.target_receipt_raw_digest = Digest::sha256_bytes(&bytes);
        fs::write(&path, &bytes).expect("unsupported model");
        append_is_denied(root, ID, &request);
        assert_eq!(fs::read(path).expect("fault preserved"), bytes);
    }
}

#[test]
fn second_event_binds_parent_sequence_and_domain_digest() {
    let directory = repository();
    let root = directory.path();
    let (mut request, _, _) = target(root);
    record_review_correction(root, ID, &request).expect("first correction");
    let first = review_correction_events(root, ID)
        .expect("first chain")
        .remove(0);
    let expected_request = cockpit_protocol::digest_json(&json!({
        "domain":"ai-cockpit/review-correction/v1", "request":request
    }))
    .expect("domain-bound request");
    assert_eq!(first.request_digest, expected_request);
    request.reason.push_str(" Additional local correction.");
    record_review_correction(root, ID, &request).expect("second correction");
    let events = review_correction_events(root, ID).expect("two event chain");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0], first);
    assert_eq!(events[1].sequence, 2);
    assert_eq!(
        events[1].parent_digest,
        Some(cockpit_protocol::digest_json(&first).expect("parent digest"))
    );
    assert!(!events[1].human_authority_granted);
}

#[test]
fn validated_replay_is_distinct_from_a_new_request_with_a_missing_target() {
    let directory = repository();
    let root = directory.path();
    let (mut request, path, original) = target(root);
    let first = record_review_correction(root, ID, &request).expect("first correction");
    let retained = root.join("retained-fixture-target.json");
    fs::rename(path, &retained).expect("target becomes unavailable to new request");
    assert_eq!(
        record_review_correction(root, ID, &request).expect("validated event replay"),
        first
    );
    request.reason.push_str(" New request.");
    append_is_denied(root, ID, &request);
    assert_eq!(
        review_correction_events(root, ID)
            .expect("old event retained")
            .len(),
        1
    );
    assert_eq!(
        fs::read(retained).expect("target retained outside search"),
        original
    );
}

#[test]
fn duplicate_target_raw_identity_is_ambiguous_without_appending() {
    let directory = repository();
    let root = directory.path();
    let (request, _, original) = target(root);
    fs::write(
        root.join(format!(".ai/decisions/{ID}.preflight-review.copy.json")),
        original,
    )
    .expect("duplicate target identity");
    assert!(append_is_denied(root, ID, &request).contains("target_missing_or_ambiguous"));
}

#[test]
fn chain_tampering_is_denied_even_with_recomputed_filename_and_request_digest() {
    for fault in [
        "schema",
        "sequence",
        "parent",
        "requestDigest",
        "requestSchema",
        "workItem",
        "repository",
        "emptyReason",
        "largeReason",
        "authority",
        "assurance",
        "recorder",
        "time",
        "unknownField",
    ] {
        let directory = repository();
        let root = directory.path();
        let (request, _, _) = target(root);
        let mut value = record_review_correction(root, ID, &request).expect("first correction");
        let old_path = event_path(root);
        match fault {
            "schema" => value["schemaVersion"] = json!(2),
            "sequence" => value["sequence"] = json!(2),
            "parent" => value["parentDigest"] = json!(Digest::sha256_bytes(b"foreign parent")),
            "requestDigest" => {
                value["requestDigest"] = json!(Digest::sha256_bytes(b"wrong request"))
            }
            "requestSchema" => value["request"]["schemaVersion"] = json!(2),
            "workItem" => value["request"]["workItemId"] = json!("WI-FOREIGN"),
            "repository" => {
                value["request"]["repositoryId"] =
                    json!(Digest::sha256_bytes(b"foreign repository"))
            }
            "emptyReason" => value["request"]["reason"] = json!(" \n\t"),
            "largeReason" => value["request"]["reason"] = json!("x".repeat(16385)),
            "authority" => value["humanAuthorityGranted"] = json!(true),
            "assurance" => value["assurance"] = json!("independently_authenticated"),
            "recorder" => value["recordedBy"] = json!("human:forged"),
            "time" => value["recordedAt"] = json!("not-a-timestamp"),
            _ => value["unexpectedField"] = json!(true),
        }
        if matches!(
            fault,
            "requestSchema" | "workItem" | "repository" | "emptyReason" | "largeReason"
        ) {
            value["requestDigest"] = json!(
                cockpit_protocol::digest_json(&json!({
                    "domain":"ai-cockpit/review-correction/v1", "request":value["request"]
                }))
                .expect("recompute request digest")
            );
        }
        let digest = cockpit_protocol::digest_json(&value).expect("recompute event digest");
        let name = format!(
            "{ID}.review-correction.{:08}.{}.json",
            value["sequence"].as_u64().expect("sequence"),
            digest.as_str().trim_start_matches("sha256:")
        );
        fs::remove_file(old_path).expect("replace only synthetic fixture event");
        fs::write(
            root.join(".ai/decisions").join(name),
            serde_json::to_vec_pretty(&value).expect("fault event"),
        )
        .expect("fault with recomputed filename");
        damaged_chain_is_denied(root, &request);
    }
}

#[test]
fn duplicate_trailing_and_over_budget_original_event_bytes_are_rejected() {
    for fault in ["duplicate", "nested-duplicate", "trailing", "over-budget"] {
        let directory = repository();
        let root = directory.path();
        let (request, _, _) = target(root);
        record_review_correction(root, ID, &request).expect("first correction");
        let path = event_path(root);
        let original = fs::read(&path).expect("event bytes");
        let bytes = match fault {
            "duplicate" => [b"{\"schemaVersion\":1,".as_slice(), &original[1..]].concat(),
            "nested-duplicate" => String::from_utf8(original)
                .expect("UTF-8")
                .replacen("\"reason\":", "\"reason\":\"duplicate\",\"reason\":", 1)
                .into_bytes(),
            "trailing" => [original.as_slice(), b" {}".as_slice()].concat(),
            _ => {
                let mut bytes = original;
                bytes.resize(128 * 1024 + 1, b' ');
                bytes
            }
        };
        fs::write(&path, &bytes).expect("event raw fault");
        damaged_chain_is_denied(root, &request);
        assert_eq!(fs::read(path).expect("fault retained"), bytes);
    }
}

#[test]
fn renamed_or_missing_chain_positions_cannot_be_implicitly_skipped() {
    for gap in [false, true] {
        let directory = repository();
        let root = directory.path();
        let (mut request, _, _) = target(root);
        record_review_correction(root, ID, &request).expect("first correction");
        let first_path = event_path(root);
        let first_bytes = fs::read(&first_path).expect("first bytes");
        if gap {
            request.reason.push_str(" Second event.");
            record_review_correction(root, ID, &request).expect("second correction");
            fs::rename(first_path, root.join("retained-first-event.json"))
                .expect("first position missing");
            assert_eq!(
                fs::read(root.join("retained-first-event.json")).expect("old bytes retained"),
                first_bytes
            );
        } else {
            fs::rename(
                first_path,
                root.join(format!(
                    ".ai/decisions/{ID}.review-correction.00000001.{}.json",
                    "0".repeat(64)
                )),
            )
            .expect("wrong canonical filename");
        }
        damaged_chain_is_denied(root, &request);
    }
}

#[test]
fn interrupted_publication_residue_is_preserved_and_denies_continuation() {
    let directory = repository();
    let root = directory.path();
    let (request, _, _) = target(root);
    let residue = root.join(format!(
        ".ai/decisions/{ID}.review-correction.00000001.partial.json.receipt-tmp-fixture"
    ));
    fs::write(&residue, b"{partial publication").expect("interrupted bytes");
    damaged_chain_is_denied(root, &request);
    assert_eq!(
        fs::read(residue).expect("residue retained"),
        b"{partial publication"
    );
}

#[test]
fn event_enumeration_budget_denies_before_parsing_or_publication() {
    let directory = repository();
    let root = directory.path();
    let (request, _, _) = target(root);
    for number in 0..257 {
        fs::write(
            root.join(format!(
                ".ai/decisions/{ID}.review-correction.{number:08}.json"
            )),
            b"{}",
        )
        .expect("bounded enumeration fault");
    }
    let before = inventory(root);
    let error = review_correction_events(root, ID).expect_err("event budget");
    assert!(
        error
            .to_string()
            .contains("review_correction_budget_exceeded")
    );
    assert!(append_is_denied(root, ID, &request).contains("review_correction_budget_exceeded"));
    assert_eq!(inventory(root), before);
}

#[test]
fn total_directory_budget_includes_unrelated_entries() {
    let directory = repository();
    let root = directory.path();
    let (request, _, _) = target(root);
    let initial = inventory(root).len();
    for number in initial..=16384 {
        fs::write(
            root.join(format!(".ai/decisions/unrelated-{number:08}")),
            b"",
        )
        .expect("directory budget fixture");
    }
    let before = inventory(root);
    assert!(
        review_correction_events(root, ID)
            .expect_err("directory budget")
            .to_string()
            .contains("review_directory_budget_exceeded")
    );
    assert!(append_is_denied(root, ID, &request).contains("review_directory_budget_exceeded"));
    assert_eq!(inventory(root), before);
}

#[cfg(unix)]
#[test]
fn target_event_and_parent_symlinks_are_not_followed() {
    use std::os::unix::fs::symlink;
    for kind in ["target", "event", "parent"] {
        let directory = repository();
        let root = directory.path();
        let (request, target_path, target_bytes) = target(root);
        if kind == "target" {
            let retained = root.join("retained-target.json");
            fs::rename(target_path.clone(), &retained).expect("retain original target");
            symlink(&retained, &target_path).expect("symlink target");
            append_is_denied(root, ID, &request);
            assert_eq!(fs::read(retained).expect("retained bytes"), target_bytes);
        } else {
            record_review_correction(root, ID, &request).expect("first correction");
            if kind == "event" {
                let path = event_path(root);
                let retained = root.join("retained-event.json");
                fs::rename(&path, &retained).expect("retain event");
                let bytes = fs::read(&retained).expect("event bytes");
                symlink(&retained, &path).expect("symlink event");
                damaged_chain_is_denied(root, &request);
                assert_eq!(fs::read(retained).expect("event retained"), bytes);
            } else {
                let original = root.join(".ai/decisions");
                let before = inventory(root);
                let retained = root.join("retained-decisions");
                fs::rename(&original, &retained).expect("retain actual parent");
                symlink(&retained, &original).expect("symlink parent");
                assert!(review_correction_events(root, ID).is_err());
                assert!(record_review_correction(root, ID, &request).is_err());
                assert_eq!(inventory(root), before);
            }
        }
    }
}

fn wait_for_child(child: &mut Child, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if child.try_wait().expect("observe own child").is_some() {
            return true;
        }
        if Instant::now() >= deadline {
            child.kill().expect("stop only this bounded child");
            child.wait().expect("reap own child");
            return false;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn public_reader_and_append_share_the_cross_process_work_item_lock() {
    const CHILD: &str = "AI_COCKPIT_CORRECTION_LOCK_CHILD";
    if let Some(action) = std::env::var_os(CHILD) {
        let root =
            PathBuf::from(std::env::var_os("AI_COCKPIT_CORRECTION_LOCK_ROOT").expect("child root"));
        let request: ReviewCorrectionRequest = serde_json::from_slice(
            &fs::read(root.join("child-request.json")).expect("child request bytes"),
        )
        .expect("typed child request");
        let action = action.to_str().expect("child action");
        fs::write(root.join(format!("child-{action}.ready")), b"ready").expect("child ready");
        let result = if action == "read" {
            json!({"events":review_correction_events(&root, ID).expect("actual public reader").len()})
        } else {
            record_review_correction(&root, ID, &request).expect("actual public append")
        };
        fs::write(
            root.join(format!("child-{action}.result")),
            serde_json::to_vec(&result).expect("child result JSON"),
        )
        .expect("child result");
        return;
    }

    let directory = repository();
    let root = directory.path();
    let (request, _, _) = target(root);
    assert!(
        review_correction_events(root, ID)
            .expect("initial public reader creates lock")
            .is_empty()
    );
    fs::write(
        root.join("child-request.json"),
        serde_json::to_vec(&request).expect("child request"),
    )
    .expect("child input file");
    for action in ["read", "append"] {
        let lock = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(root.join(format!(".ai/locks/{ID}.lifecycle.lock")))
            .expect("actual lifecycle lock file");
        lock.lock().expect("parent owns exact work item lock");
        let mut child = Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--exact",
                "public_reader_and_append_share_the_cross_process_work_item_lock",
                "--test-threads=1",
                "--nocapture",
            ])
            .env(CHILD, action)
            .env("AI_COCKPIT_CORRECTION_LOCK_ROOT", root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("bounded own child");
        let ready = root.join(format!("child-{action}.ready"));
        let result = root.join(format!("child-{action}.result"));
        let deadline = Instant::now() + Duration::from_secs(10);
        while !ready.exists() && Instant::now() < deadline {
            if child.try_wait().expect("child status").is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        let observed_ready = ready.exists();
        if observed_ready {
            std::thread::sleep(Duration::from_millis(250));
        }
        let completed_while_locked = result.exists();
        lock.unlock()
            .expect("release exact work item lock before assertions");
        let bounded = wait_for_child(&mut child, Duration::from_secs(10));
        let output = child.wait_with_output().expect("child output");
        assert!(
            observed_ready && bounded && output.status.success(),
            "child did not complete normally: {output:?}"
        );
        assert!(
            !completed_while_locked,
            "public {action} bypassed the existing lifecycle lock"
        );
        let value: Value = serde_json::from_slice(&fs::read(result).expect("result after unlock"))
            .expect("result JSON");
        if action == "read" {
            assert_eq!(value["events"], 0);
        } else {
            assert_eq!(value["humanAuthorityGranted"], false);
        }
    }
    assert_eq!(
        review_correction_events(root, ID)
            .expect("one actual append")
            .len(),
        1
    );
}

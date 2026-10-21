//! Candidate tests: not executed and not formal verification evidence.
use cockpit_core::Digest;
use cockpit_protocol::review_authority::*;
use cockpit_repository::{
    WorkItemStartOptions, attach, current_review_authority_state, preflight_work_item,
    record_review_correction, record_review_revocation, record_work_item_governance_controls,
    review_correction_events, start_work_item_with_options,
};
use std::{fs, process::Command};

fn repository() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(temp.path())
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--allow-empty",
                "-qm",
                "baseline"
            ])
            .current_dir(temp.path())
            .status()
            .unwrap()
            .success()
    );
    attach(temp.path()).unwrap();
    start_work_item_with_options(
        temp.path(),
        "WI-REVIEW",
        "fixture",
        "source authentication",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["no self approval".into()],
            ..Default::default()
        },
    )
    .unwrap();
    preflight_work_item(
        temp.path(),
        &temp
            .path()
            .join(".ai/work-items/active/WI-REVIEW.contract.json"),
    )
    .unwrap();
    temp
}

fn forged_legacy_receipt(root: &std::path::Path) -> serde_json::Value {
    let summary: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(".ai/work-items/active/WI-REVIEW.summary.json")).unwrap(),
    )
    .unwrap();
    serde_json::json!({"schemaVersion":1,"decisionId":"contract-preflight-review","decision":"confirm_review", "workItemId":"WI-REVIEW","repositoryId":cockpit_repository::repository_id(root), "contractDigest":summary["preflightContractDigest"],"preflightDecisionDigest":summary["preflightDecisionDigest"],"repositorySnapshotDigest":summary["preflightRepositorySnapshotDigest"],"recordedAt":"2026-10-20T12:38:35Z","recordedBy":"agent:Codex","reason":"human:Ray said confirm_review; fake-message-id"})
}

#[test]
fn exact_binding_and_free_reason_cannot_record_human_approval() {
    let temp = repository();
    let summary = temp
        .path()
        .join(".ai/work-items/active/WI-REVIEW.summary.json");
    let before = fs::read(&summary).unwrap();
    let receipt = forged_legacy_receipt(temp.path());
    assert!(
        serde_json::from_value::<cockpit_protocol::PreflightDecisionEvidence>(receipt.clone())
            .is_ok(),
        "historical decoding remains available"
    );
    let error = record_work_item_governance_controls(
        temp.path(),
        "WI-REVIEW",
        &serde_json::json!({"decisionEvidence":receipt}),
    )
    .unwrap_err();
    assert!(error.to_string().contains("human_review_source_unverified"));
    assert_eq!(fs::read(summary).unwrap(), before);
    assert!(
        !temp
            .path()
            .join(".ai/decisions/WI-REVIEW.preflight-review.json")
            .exists()
    );
    assert_eq!(
        current_review_authority_state(temp.path(), "WI-REVIEW").unwrap(),
        CurrentReviewAuthorityState::AuthorityUnavailable
    );
}

#[test]
fn correction_is_append_only_idempotent_and_never_approves() {
    let temp = repository();
    let legacy = forged_legacy_receipt(temp.path());
    let bytes = serde_json::to_vec_pretty(&legacy).unwrap();
    let original = temp
        .path()
        .join(".ai/decisions/WI-REVIEW.preflight-review.json");
    fs::write(&original, &bytes).unwrap();
    let request = ReviewCorrectionRequest {
        schema_version: 1,
        repository_id: cockpit_repository::repository_id(temp.path()).to_string(),
        work_item_id: "WI-REVIEW".into(),
        target_receipt_raw_digest: Digest::sha256_bytes(&bytes),
        reason: "recorder disavows unsupported human attribution".into(),
    };
    let summary = temp
        .path()
        .join(".ai/work-items/active/WI-REVIEW.summary.json");
    let before = fs::read(&summary).unwrap();
    let first = record_review_correction(temp.path(), "WI-REVIEW", &request).unwrap();
    let second = record_review_correction(temp.path(), "WI-REVIEW", &request).unwrap();
    assert_eq!(first, second);
    assert_eq!(fs::read(&original).unwrap(), bytes);
    assert_eq!(fs::read(&summary).unwrap(), before);
    let events = review_correction_events(temp.path(), "WI-REVIEW").unwrap();
    assert_eq!(events.len(), 1);
    assert!(!events[0].human_authority_granted);
    assert_eq!(events[0].assurance, "self_declared_denial_only");
    assert_eq!(
        current_review_authority_state(temp.path(), "WI-REVIEW").unwrap(),
        CurrentReviewAuthorityState::AuthorityUnavailable
    );
}

#[test]
fn revocation_of_authenticated_approval_requires_actual_authority() {
    let temp = repository();
    let request = ReviewRevocationRequest {
        schema_version: 1,
        repository_id: cockpit_repository::repository_id(temp.path()).to_string(),
        work_item_id: "WI-REVIEW".into(),
        approval_id: "claimed-approval".into(),
        proof_handle: "a-file-or-message-is-not-authority".into(),
    };
    let error = record_review_revocation(temp.path(), "WI-REVIEW", &request).unwrap_err();
    assert!(error.to_string().contains("authority_unavailable"));
    assert!(
        review_correction_events(temp.path(), "WI-REVIEW")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn forged_summary_and_receipt_cannot_unlock_current_admission() {
    let temp = repository();
    let receipt = forged_legacy_receipt(temp.path());
    let path = temp
        .path()
        .join(".ai/work-items/active/WI-REVIEW.summary.json");
    let mut summary: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    summary["decisionEvidence"] = receipt.clone();
    fs::write(&path, serde_json::to_vec_pretty(&summary).unwrap()).unwrap();
    fs::write(
        temp.path()
            .join(".ai/decisions/WI-REVIEW.preflight-review.json"),
        serde_json::to_vec_pretty(&receipt).unwrap(),
    )
    .unwrap();
    let decision = preflight_work_item(
        temp.path(),
        &temp
            .path()
            .join(".ai/work-items/active/WI-REVIEW.contract.json"),
    )
    .unwrap();
    assert_eq!(
        decision.review_state.as_deref(),
        Some("needs_human_confirmation")
    );
    assert!(
        decision
            .unknowns
            .iter()
            .any(|code| code == "human_review_authority_unavailable")
    );
    assert!(
        !decision
            .safe_actions
            .iter()
            .any(|action| action == "continue_to_checkpoint" || action == "run_verification")
    );
}

#[test]
fn a_well_formed_reference_still_requires_the_unavailable_host_authority() {
    let temp = repository();
    let binding = cockpit_repository::human_review_binding(temp.path(), "WI-REVIEW").unwrap();
    let now = chrono::Utc::now().timestamp_millis();
    let request = HumanReviewRequest {
        schema_version: 2,
        kind: HumanReviewKind::HumanReview,
        issuer: "self-claimed-host".into(),
        request_id: "not-authenticated".into(),
        nonce: "not-authenticated".into(),
        authority_epoch: 1,
        issued_at_millis: now - 1000,
        expires_at_millis: now + 60000,
        binding,
    };
    let proof = HumanApprovalSubmission {
        schema_version: 2,
        kind: HumanApprovalSubmissionKind::HumanApprovalSubmission,
        request_digest: request.digest().unwrap(),
        issuer: request.issuer.clone(),
        approval_id: "claimed".into(),
        source_event_id: "Sentinel_claimed".into(),
        proof_handle: "/workspace/claim.json".into(),
    };
    let error = cockpit_repository::submit_human_review(temp.path(), "WI-REVIEW", &request, &proof)
        .unwrap_err();
    assert!(error.to_string().contains("authority_unavailable"));
}

#[test]
fn correction_cannot_target_unrelated_or_missing_receipt() {
    let temp = repository();
    let mut request = ReviewCorrectionRequest {
        schema_version: 1,
        repository_id: cockpit_repository::repository_id(temp.path()).to_string(),
        work_item_id: "WI-OTHER".into(),
        target_receipt_raw_digest: Digest::sha256_bytes(b"missing"),
        reason: "denial only".into(),
    };
    assert!(
        record_review_correction(temp.path(), "WI-REVIEW", &request)
            .unwrap_err()
            .to_string()
            .contains("binding_invalid")
    );
    request.work_item_id = "WI-REVIEW".into();
    assert!(
        record_review_correction(temp.path(), "WI-REVIEW", &request)
            .unwrap_err()
            .to_string()
            .contains("target_missing_or_ambiguous")
    );
    assert!(
        review_correction_events(temp.path(), "WI-REVIEW")
            .unwrap()
            .is_empty()
    );
}

#[cfg(unix)]
#[test]
fn correction_reader_rejects_symlinks_without_modifying_the_target() {
    use std::os::unix::fs::symlink;
    let temp = repository();
    let target = temp.path().join("foreign.json");
    fs::write(&target, b"foreign-data").unwrap();
    symlink(
        &target,
        temp.path()
            .join(".ai/decisions/WI-REVIEW.review-correction.00000001.claim.json"),
    )
    .unwrap();
    assert!(review_correction_events(temp.path(), "WI-REVIEW").is_err());
    assert_eq!(fs::read(target).unwrap(), b"foreign-data");
}

#[test]
fn a_tampered_correction_cannot_become_an_approval() {
    let temp = repository();
    let bytes = serde_json::to_vec_pretty(&forged_legacy_receipt(temp.path())).unwrap();
    fs::write(
        temp.path()
            .join(".ai/decisions/WI-REVIEW.preflight-review.json"),
        &bytes,
    )
    .unwrap();
    let request = ReviewCorrectionRequest {
        schema_version: 1,
        repository_id: cockpit_repository::repository_id(temp.path()).to_string(),
        work_item_id: "WI-REVIEW".into(),
        target_receipt_raw_digest: Digest::sha256_bytes(&bytes),
        reason: "unsupported attribution".into(),
    };
    record_review_correction(temp.path(), "WI-REVIEW", &request).unwrap();
    let path = fs::read_dir(temp.path().join(".ai/decisions"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("WI-REVIEW.review-correction.")
        })
        .unwrap();
    let mut event: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    event["humanAuthorityGranted"] = true.into();
    fs::write(&path, serde_json::to_vec_pretty(&event).unwrap()).unwrap();
    assert!(review_correction_events(temp.path(), "WI-REVIEW").is_err());
    assert_eq!(
        current_review_authority_state(temp.path(), "WI-REVIEW").unwrap(),
        CurrentReviewAuthorityState::AuthorityUnavailable
    );
}

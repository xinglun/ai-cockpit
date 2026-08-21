use chrono::{Duration, TimeZone, Utc};
use cockpit_release::handoff::{Destination, HandoffDocument, Issuer, ReleaseBinding};

fn parts() -> (Issuer, Destination, ReleaseBinding) {
    (
        Issuer {
            repository: "xinglun/ai-cockpit".into(),
            workflow_ref: "xinglun/ai-cockpit/.github/workflows/release.yml@0000000000000000000000000000000000000000".into(),
            run_id: 42,
        },
        Destination {
            repository: "xinglun/homebrew-tap".into(),
            base_ref: "main".into(),
            path: "Formula/ai-cockpit.rb".into(),
        },
        ReleaseBinding {
            tag: "v0.1.0".into(),
            commit: "0000000000000000000000000000000000000000".into(),
            provider_release_id: 123,
            manifest_sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            formula_sha256: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
        },
    )
}

fn handoff() -> HandoffDocument {
    let (issuer, destination, release) = parts();
    let issued = Utc.with_ymd_and_hms(2026, 8, 21, 0, 0, 0).unwrap();
    HandoffDocument::new(
        issuer,
        destination,
        release,
        "open_pull_request".into(),
        issued,
        issued + Duration::hours(1),
    )
    .unwrap()
}

#[test]
fn request_id_is_canonical_and_validation_accepts_one_hour_window() {
    let handoff = handoff();
    assert_eq!(handoff.request_id, handoff.recompute_request_id().unwrap());
    handoff
        .validate(Utc.with_ymd_and_hms(2026, 8, 21, 0, 30, 0).unwrap())
        .unwrap();
    assert!(handoff.canonical_bytes().unwrap().ends_with(b"\n"));
}

#[test]
fn changed_formula_digest_changes_request_identity() {
    let mut changed = handoff();
    let original = changed.request_id.clone();
    changed.release.formula_sha256 =
        "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".into();
    assert_ne!(original, changed.recompute_request_id().unwrap());
}

#[test]
fn unknown_fields_and_wrong_destination_are_rejected() {
    let json = String::from_utf8(handoff().canonical_bytes().unwrap())
        .unwrap()
        .replace("\"issuer\"", "\"futurePolicy\":true,\"issuer\"");
    let error = HandoffDocument::parse_str(&json).expect_err("unknown field must fail");
    assert!(error.to_string().contains("unknown"));

    let mut wrong = handoff();
    wrong.destination.repository = "someone-else/homebrew-tap".into();
    let error = wrong
        .validate(Utc.with_ymd_and_hms(2026, 8, 21, 0, 30, 0).unwrap())
        .expect_err("wrong destination must fail");
    assert!(error.to_string().contains("destination"));
}

#[test]
fn expiry_and_clock_skew_are_fail_closed() {
    let (issuer, destination, release) = parts();
    let issued = Utc.with_ymd_and_hms(2026, 8, 21, 0, 0, 0).unwrap();
    let too_long = HandoffDocument::new(
        issuer.clone(),
        destination.clone(),
        release.clone(),
        "open_pull_request".into(),
        issued,
        issued + Duration::hours(24) + Duration::seconds(1),
    )
    .expect_err("more than 24 hours must fail");
    assert!(too_long.to_string().contains("24"));

    let expired = HandoffDocument::new(
        issuer,
        destination,
        release,
        "open_pull_request".into(),
        issued,
        issued + Duration::hours(1),
    )
    .unwrap();
    let error = expired
        .validate(issued + Duration::hours(1) + Duration::minutes(6))
        .expect_err("expired handoff must fail");
    assert!(error.to_string().contains("expired"));
}

#[test]
fn workflow_ref_commit_must_match_release_commit() {
    let mut changed = handoff();
    changed.issuer.workflow_ref =
        "xinglun/ai-cockpit/.github/workflows/release.yml@1111111111111111111111111111111111111111"
            .into();
    let error = changed
        .validate(Utc.with_ymd_and_hms(2026, 8, 21, 0, 30, 0).unwrap())
        .expect_err("workflow and release commits must match");
    assert!(error.to_string().contains("commit"));
}

#[test]
fn parse_rejects_expiry_window_over_24_hours() {
    let mut changed = handoff();
    changed.expires_at = "2026-08-23T00:00:00Z".into();
    changed.request_id = changed.recompute_request_id().unwrap();
    let json = String::from_utf8(changed.canonical_bytes().unwrap()).unwrap();
    let error = HandoffDocument::parse_str(&json).expect_err("parsed window must be bounded");
    assert!(error.to_string().contains("24"));
}

//! Candidate transport regressions, not executed in the preparation phase.
use std::process::Command;

fn fixture() -> tempfile::TempDir {
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
    cockpit_repository::attach(temp.path()).unwrap();
    temp
}

#[test]
fn status_exposes_unavailable_without_inventing_a_human_approval() {
    let temp = fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["work-item", "review", "status", "--repo"])
        .arg(temp.path())
        .args(["--id", "WI-FIXTURE"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["state"], "authority_unavailable");
    assert_eq!(value["humanAuthorityGranted"], false);
}

#[test]
fn correction_wire_rejects_duplicate_keys_before_any_governance_write() {
    let temp = fixture();
    let input = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(input.path(), br#"{"schemaVersion":1,"schemaVersion":2}"#).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["work-item", "review", "correct", "--repo"])
        .arg(temp.path())
        .args(["--id", "WI-FIXTURE", "--input"])
        .arg(input.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("review_duplicate_json_key"));
}

#[test]
fn actual_cli_request_requires_protected_issuer_and_has_no_backend_or_actor_flag() {
    let t = fixture();
    cockpit_repository::start_work_item_with_options(
        t.path(),
        "WI-CLI",
        "fixture",
        "request issuance",
        &["src/**".into()],
        &cockpit_repository::WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["protected source".into()],
            ..Default::default()
        },
    )
    .unwrap();
    cockpit_repository::preflight_work_item(
        t.path(),
        &t.path().join(".ai/work-items/active/WI-CLI.contract.json"),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["work-item", "review", "request", "--repo"])
        .arg(t.path())
        .args(["--id", "WI-CLI"])
        .env("AI_COCKPIT_REVIEW_AUTHORITY", "mock")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("authority_unavailable"));
    assert!(output.stdout.is_empty());
    let help = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["work-item", "review", "request", "--help"])
        .output()
        .unwrap();
    assert!(help.status.success());
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("--operation-input"));
    assert!(!text.contains("--actor"));
    assert!(!text.contains("--backend"));
}
#[test]
fn actual_production_cli_binary_does_not_contain_test_authority() {
    let bytes = std::fs::read(env!("CARGO_BIN_EXE_ai-cockpit")).unwrap();
    for marker in [
        b"AI_COCKPIT_TEST_AUTHORITY_SENTINEL_V4".as_slice(),
        b"fixture-protected-issuer".as_slice(),
    ] {
        assert!(!bytes.windows(marker.len()).any(|window| window == marker));
    }
}
#[test]
fn operation_selection_rejects_json_backend_and_actor_claims() {
    let t = fixture();
    let input = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(input.path(),br#"{"operation":"profile_confirmation","payload":{"program":"cargo","args":[]},"backend":"mock","actor":"human:Ray"}"#).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["work-item", "review", "request", "--repo"])
        .arg(t.path())
        .args(["--id", "WI-CLI", "--operation-input"])
        .arg(input.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown field"));
}

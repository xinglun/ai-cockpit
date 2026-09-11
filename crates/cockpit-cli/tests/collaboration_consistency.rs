//! Controlled-repository end-to-end checks for collaboration-language section IV.
//!
//! The fixture deliberately exercises the three facts that are easiest to
//! conflate in a hand-off: the state shown by the Runtime, the option selected
//! by the caller, and the transition the Runtime actually permits.  The
//! synthetic human decision below is test data in a temporary repository; it
//! is never an authorization record in the source repository.

use serde_json::json;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
    thread,
    time::{Duration, Instant},
};

#[cfg(unix)]
use std::{path::PathBuf, process::Child};

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("controlled repository");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    directory
}

fn run(binary: &str, repo: &Path, args: &[&str]) -> Output {
    Command::new(binary)
        .args(args)
        .args(["--repo"])
        .arg(repo)
        .current_dir(repo)
        .output()
        .expect("run ai-cockpit")
}

fn run_json(binary: &str, repo: &Path, args: &[&str]) -> serde_json::Value {
    let output = run(binary, repo, args);
    assert!(
        output.status.success(),
        "args={args:?}, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("machine-readable JSON")
}

fn start_fixture(binary: &str, repo: &Path, id: &str) {
    run_json(binary, repo, &["attach"]);
    let output = run(
        binary,
        repo,
        &[
            "start",
            "--id",
            id,
            "--intent",
            "prove state, option, and Runtime behavior remain aligned",
            "--goal",
            "exercise a bounded controlled-repository consistency scenario",
            "--scope",
            "README.md",
            "--out-of-scope",
            "production behavior",
            "--authority",
            "authorized",
            "--acceptance",
            "test-data option must bind only inside the temporary fixture",
            "--required-evidence",
            "verification",
        ],
    );
    assert!(
        output.status.success(),
        "start stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    // TEST DATA ONLY: mark a human-decision requirement inside the temporary
    // fixture so preflight must display a structured decision request.
    let contract_path = repo
        .join(".ai/work-items/active")
        .join(format!("{id}.contract.json"));
    let mut value = contract(repo, id);
    value["agentCapability"] = serde_json::json!({
        "canImplement": true,
        "canVerify": true,
        "needsHumanDecision": true,
        "blockedReason": "controlled test data requires the option-selection path"
    });
    value["executionDecision"] = serde_json::json!({
        "status": "needs_human_decision",
        "reason": "controlled test data requires the option-selection path"
    });
    fs::write(
        contract_path,
        serde_json::to_vec_pretty(&value).expect("fixture contract JSON"),
    )
    .expect("mutate fixture contract");
}

fn contract(repo: &Path, id: &str) -> serde_json::Value {
    serde_json::from_slice(
        &fs::read(
            repo.join(".ai/work-items/active")
                .join(format!("{id}.contract.json")),
        )
        .expect("contract"),
    )
    .expect("contract JSON")
}

fn summary(repo: &Path, id: &str) -> serde_json::Value {
    serde_json::from_slice(
        &fs::read(
            repo.join(".ai/work-items/active")
                .join(format!("{id}.summary.json")),
        )
        .expect("summary"),
    )
    .expect("summary JSON")
}

fn record_test_data_decision(repo: &Path, id: &str) {
    let contract_path = repo
        .join(".ai/work-items/active")
        .join(format!("{id}.contract.json"));
    let contract = contract(repo, id);
    let summary = summary(repo, id);
    // TEST DATA ONLY: this synthetic choice models the option selected by a
    // human in the controlled fixture. It is not an external authorization.
    let decision = serde_json::json!({
        "schemaVersion": 1,
        "decisionId": "contract-preflight-review",
        "decision": "confirm_review",
        "workItemId": id,
        "repositoryId": cockpit_repository::repository_id(repo),
        "contractDigest": cockpit_protocol::digest_json(&contract).expect("contract digest"),
        "preflightDecisionDigest": summary["preflightDecisionDigest"].clone(),
        "repositorySnapshotDigest": summary["preflightRepositorySnapshotDigest"].clone(),
        "recordedAt": "2026-09-09T00:00:00Z",
        "recordedBy": "test-data:synthetic-human",
        "reason": "test fixture selects the offered confirm_review option"
    });
    let input = tempfile::NamedTempFile::new().expect("decision input");
    fs::write(
        input.path(),
        serde_json::to_vec_pretty(&json!({
            "decisionEvidence": decision,
            "intentAlignment": {
                "state": "resolved",
                "evidence": ["tests/collaboration_consistency.rs"]
            }
        }))
        .expect("decision input JSON"),
    )
    .expect("write decision input");
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let output = Command::new(binary)
        .args(["work-item", "controls", "--id", id, "--input"])
        .arg(input.path())
        .args(["--repo"])
        .arg(repo)
        .output()
        .expect("record test-data decision");
    assert!(
        output.status.success(),
        "controls stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        repo.join(".ai/decisions")
            .join(format!("{id}.preflight-review.json"))
            .is_file(),
        "the synthetic decision must be bound inside the temporary fixture"
    );
    assert!(contract_path.is_file());
}

#[cfg(unix)]
fn wait_for(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !path.is_file() {
        assert!(Instant::now() < deadline, "timed out waiting for {path:?}");
        thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(unix)]
fn blocking_verification_command(repo: &Path) -> (PathBuf, PathBuf, PathBuf) {
    use std::os::unix::fs::PermissionsExt;

    let started = repo.join("verification.started");
    let release = repo.join("verification.release");
    let command = repo.join("verification-blocking.sh");
    fs::write(
        &command,
        format!(
            "#!/bin/sh\ntouch '{}'\nwhile test ! -f '{}'; do sleep 0.01; done\n",
            started.display(),
            release.display()
        ),
    )
    .expect("blocking verification script");
    fs::set_permissions(&command, fs::Permissions::from_mode(0o755))
        .expect("blocking verification permissions");
    (command, started, release)
}

#[cfg(unix)]
fn spawn_interrupted_verify(
    binary: &str,
    repo: &Path,
    id: &str,
) -> (Child, tempfile::TempDir, PathBuf, PathBuf) {
    // The blocking command is execution input, not repository source. Keep it
    // outside the repository so creating/removing the interruption fixture
    // cannot invalidate the Work Item's governance snapshot before verify.
    let command_dir = tempfile::tempdir().expect("verification fixture directory");
    let (command, started, release) = blocking_verification_command(command_dir.path());
    let mut verify = Command::new(binary);
    verify
        .args(["verify", "--work-item", id, "--command"])
        .arg(command)
        .args(["--repo"])
        .arg(repo);
    (
        verify.spawn().expect("spawn verification"),
        command_dir,
        started,
        release,
    )
}

#[test]
fn displayed_option_state_and_runtime_transition_stay_consistent_through_resume() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let id = "WI-COLLAB-CONSISTENCY";
    let repo = repository();
    start_fixture(binary, repo.path(), id);

    let preflight = run_json(
        binary,
        repo.path(),
        &[
            "preflight",
            "--contract",
            &format!(".ai/work-items/active/{id}.contract.json"),
        ],
    );
    assert_eq!(preflight["reviewState"], "needs_human_confirmation");
    let options = preflight["humanDecisionRequest"]["options"]
        .as_array()
        .expect("displayed options");
    assert!(
        options
            .iter()
            .any(|option| option["id"] == "confirm_review")
    );
    assert_eq!(
        preflight["humanDecisionRequest"]["status"],
        "needs_human_confirmation"
    );

    let before_decision = run(binary, repo.path(), &["checkpoint", "--id", id]);
    assert!(!before_decision.status.success());
    assert!(String::from_utf8_lossy(&before_decision.stderr).contains("human confirmation"));

    record_test_data_decision(repo.path(), id);
    let confirmed = run_json(
        binary,
        repo.path(),
        &[
            "preflight",
            "--contract",
            &format!(".ai/work-items/active/{id}.contract.json"),
        ],
    );
    assert_eq!(confirmed["reviewState"], "human_decision_recorded");
    assert!(
        confirmed["safe_actions"]
            .as_array()
            .expect("safe actions")
            .iter()
            .any(|action| action == "continue_to_checkpoint")
    );
    let checkpoint = run_json(binary, repo.path(), &["checkpoint", "--id", id]);
    assert_eq!(checkpoint["state"], "checkpointed");
    let checkpointed = run_json(
        binary,
        repo.path(),
        &["work-item", "status", "--id", id, "--json"],
    );
    assert_eq!(checkpointed["lifecyclePhase"], "checkpointed");
    assert_ne!(checkpointed["verification"], "verified");

    let evidence = repo
        .path()
        .join(".ai/evidence")
        .join(format!("{id}.verification.json"));
    #[cfg(unix)]
    {
        let (mut interrupted, command_dir, started, release) =
            spawn_interrupted_verify(binary, repo.path(), id);
        wait_for(&started);
        interrupted.kill().expect("interrupt verification process");
        let interrupted_status = interrupted.wait().expect("wait interrupted verification");
        assert!(!interrupted_status.success());
        fs::write(&release, b"release controlled child\n").expect("release fixture child");
        thread::sleep(Duration::from_millis(100));

        assert!(!evidence.exists(), "interruption must not publish a pass");
        let interrupted_projection = run_json(
            binary,
            repo.path(),
            &["work-item", "status", "--id", id, "--json"],
        );
        assert_eq!(interrupted_projection["lifecyclePhase"], "checkpointed");
        assert_ne!(
            interrupted_projection["evidenceFreshness"]["state"],
            "fresh"
        );
        let interrupted_outcome = run_json(
            binary,
            repo.path(),
            &["work-item", "outcome", "--id", id, "--json"],
        );
        assert_ne!(interrupted_outcome["state"], "verified");
        fs::remove_file(&started).expect("remove interruption marker");
        fs::remove_file(&release).expect("remove release marker");
        fs::remove_file(command_dir.path().join("verification-blocking.sh"))
            .expect("remove interruption script");
    }

    let resumed = run_json(
        binary,
        repo.path(),
        &["verify", "--work-item", id, "--command", "true"],
    );
    assert_eq!(resumed["passed"], true);
    assert!(evidence.is_file(), "resume must publish current evidence");
    let after_resume_preflight = run_json(
        binary,
        repo.path(),
        &[
            "preflight",
            "--contract",
            &format!(".ai/work-items/active/{id}.contract.json"),
        ],
    );
    assert_eq!(
        after_resume_preflight["reviewState"],
        "needs_human_confirmation"
    );
    assert!(
        after_resume_preflight["unknowns"]
            .as_array()
            .expect("preflight unknowns")
            .iter()
            .any(|unknown| unknown == "preflight_decision_evidence_invalid")
    );

    // A successful verification changes the governance decision projection.
    // The Runtime must reject the old receipt and require a fresh decision for
    // the current evidence before allowing the next lifecycle transition.
    record_test_data_decision(repo.path(), id);
    let current = run_json(
        binary,
        repo.path(),
        &[
            "preflight",
            "--contract",
            &format!(".ai/work-items/active/{id}.contract.json"),
        ],
    );
    assert_eq!(current["reviewState"], "human_decision_recorded");
    let resumed_projection = run_json(
        binary,
        repo.path(),
        &["work-item", "status", "--id", id, "--json"],
    );
    assert_eq!(resumed_projection["lifecyclePhase"], "checkpointed");
    assert_eq!(resumed_projection["verification"], "verified");
    assert_eq!(resumed_projection["evidenceFreshness"]["state"], "fresh");
}

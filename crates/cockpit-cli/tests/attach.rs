use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_REPOSITORY_ID: AtomicU64 = AtomicU64::new(0);

fn fixture_repository() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "cockpit-attach-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    directory
}

#[test]
fn attach_creates_only_protocol_state_and_is_idempotent() {
    let directory = fixture_repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let first = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&directory)
        .output()
        .expect("attach");
    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(directory.join(".ai/cockpit.toml").is_file());
    assert!(directory.join(".ai/project.json").is_file());
    assert!(directory.join(".ai/work-items/active").is_dir());
    assert!(!directory.join("scripts").exists());
    let second = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&directory)
        .output()
        .expect("attach twice");
    assert!(
        second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn attached_repository_identity_survives_a_repository_move() {
    let directory = fixture_repository();
    let moved = directory.with_file_name(format!(
        "{}-moved",
        directory
            .file_name()
            .expect("fixture name")
            .to_string_lossy()
    ));
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let first = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&directory)
        .output()
        .expect("attach");
    assert!(first.status.success());
    let first_json: serde_json::Value = serde_json::from_slice(&first.stdout).expect("profile");
    let first_id = first_json["repositoryId"].clone();

    fs::rename(&directory, &moved).expect("move repository");
    let second = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&moved)
        .output()
        .expect("reattach moved repository");
    assert!(
        second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    let second_json: serde_json::Value = serde_json::from_slice(&second.stdout).expect("profile");
    assert_eq!(second_json["repositoryId"], first_id);
    fs::remove_dir_all(moved).expect("cleanup");
}

#[test]
fn status_reports_calibration_required_before_first_profile_confirmation() {
    let directory = fixture_repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let attach = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&directory)
        .output()
        .expect("attach");
    assert!(attach.status.success());
    let status = Command::new(binary)
        .args(["status", "--repo"])
        .arg(&directory)
        .output()
        .expect("status");
    assert!(
        status.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&status.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&status.stdout).expect("JSON");
    assert_eq!(json["state"], "calibration_required");
    fs::remove_dir_all(directory).expect("cleanup");
}

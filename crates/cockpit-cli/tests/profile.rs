use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn profile_confirmation_creates_a_new_version_and_decision_receipt() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let repo = std::env::temp_dir().join(format!("cockpit-profile-{suffix}"));
    fs::create_dir_all(&repo).expect("repo");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repo)
        .status()
        .expect("git init");
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let attach = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&repo)
        .output()
        .expect("attach");
    assert!(attach.status.success());
    let confirm = Command::new(binary)
        .args(["profile", "confirm", "--repo"])
        .arg(&repo)
        .args(["--program", "cargo", "--args", "test,--workspace"])
        .output()
        .expect("confirm");
    assert!(
        confirm.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&confirm.stderr)
    );
    let profile: serde_json::Value = serde_json::from_slice(&confirm.stdout).expect("JSON");
    assert_eq!(profile["profileVersion"], 2);
    assert_eq!(profile["state"], "calibrated");
    assert!(
        repo.join(".ai/decisions/profile-v2.json").is_file(),
        "confirmation receipt"
    );
    fs::remove_dir_all(repo).expect("cleanup");
}

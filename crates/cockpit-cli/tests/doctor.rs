use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn doctor_reports_protocol_and_runtime_thin_repository_checks() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("cockpit-doctor-{suffix}"));
    fs::create_dir_all(&directory).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let attach = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&directory)
        .output()
        .expect("attach");
    assert!(attach.status.success());
    let output = Command::new(binary)
        .args(["doctor", "--repo"])
        .arg(&directory)
        .output()
        .expect("doctor");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON");
    assert_eq!(json["protocolVersion"], 1);
    assert_eq!(json["runtimeCodeInRepository"], false);
    assert_eq!(json["state"], "ok");
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn status_rejects_unsupported_repository_protocol() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("cockpit-status-protocol-{suffix}"));
    fs::create_dir_all(&directory).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let attach = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&directory)
        .output()
        .expect("attach");
    assert!(attach.status.success());
    fs::write(
        directory.join(".ai/cockpit.toml"),
        "protocol_version = 2\nrepository_id = \"invalid\"\n",
    )
    .expect("mutate protocol");
    let status = Command::new(binary)
        .args(["status", "--repo"])
        .arg(&directory)
        .output()
        .expect("status");
    assert!(!status.status.success());
    let reattach = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&directory)
        .output()
        .expect("reattach");
    assert!(!reattach.status.success());
    fs::remove_dir_all(directory).expect("cleanup");
}

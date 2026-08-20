use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn verify_executes_an_explicit_project_command_with_bounded_telemetry() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("cockpit-verify-{suffix}"));
    fs::create_dir_all(&directory).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["verify", "--repo"])
        .arg(&directory)
        .args(["--command", "true"])
        .output()
        .expect("verify");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON");
    assert_eq!(json["nodesPlanned"], 1);
    assert_eq!(json["nodesExecuted"], 1);
    assert_eq!(json["processesSpawned"], 1);
    assert_eq!(json["passed"], true);
    fs::remove_dir_all(directory).expect("cleanup");
}

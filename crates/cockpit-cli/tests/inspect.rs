use std::process::Command;

#[test]
fn inspect_returns_structured_runtime_and_repository_context() {
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["inspect", "--repo", env!("CARGO_MANIFEST_DIR")])
        .output()
        .expect("run ai-cockpit");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON output");
    assert_eq!(json["protocolVersion"], 1);
    assert!(json["repositoryRoot"].is_string());
    assert!(json["runtimeVersion"].is_string());
}

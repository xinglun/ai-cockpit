use std::process::Command;

#[test]
fn inspect_returns_structured_runtime_and_repository_context() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let output = Command::new(binary)
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
    assert_eq!(json["runtimeVersion"], env!("CARGO_PKG_VERSION"));
    let expected_digest = cockpit_core::Digest::sha256_bytes(
        &std::fs::read(binary).expect("read exact executable under test"),
    )
    .to_string();
    assert_eq!(json["runtimeDigest"], expected_digest);
}

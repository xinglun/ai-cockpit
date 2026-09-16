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

#[test]
fn read_only_diagnostics_accept_explicit_json_flag() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    for command in ["inspect", "status", "doctor"] {
        let output = Command::new(binary)
            .args([command, "--repo"])
            .arg(&repository)
            .arg("--json")
            .output()
            .expect("run ai-cockpit diagnostic");
        assert!(
            output.status.success(),
            "{command} stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice::<serde_json::Value>(&output.stdout)
            .unwrap_or_else(|error| panic!("{command} JSON output: {error}"));
    }
}

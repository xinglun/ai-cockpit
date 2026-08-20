use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn preflight_reports_yellow_when_required_evidence_is_missing() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("cockpit-preflight-{suffix}"));
    fs::create_dir_all(&directory).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    let contract_path = directory.join("contract.json");
    fs::write(&contract_path, r#"{
      "protocolVersion": 1,
      "repositoryId": "fixture",
      "intent": "verify a bounded change",
      "goal": "exercise preflight",
      "scope": ["**"],
      "outOfScope": [".git/**"],
      "risk": "normal",
      "authority": "authorized",
      "acceptanceCriteria": ["verification"],
      "requiredEvidenceClasses": ["verification"],
      "baseRevision": "0123456",
      "projectProfileDigest": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "repositorySnapshotDigest": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    }"#).expect("contract");
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["preflight", "--repo"])
        .arg(&directory)
        .args(["--contract"])
        .arg(&contract_path)
        .output()
        .expect("run preflight");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON");
    assert_eq!(json["state"], "yellow");
    assert_eq!(json["unknowns"][0], "required_evidence_missing");
    fs::remove_dir_all(directory).expect("cleanup");
}

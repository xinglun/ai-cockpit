use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_REPOSITORY_ID: AtomicU64 = AtomicU64::new(0);

#[test]
fn preflight_reports_yellow_when_required_evidence_is_missing() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "cockpit-preflight-{}-{suffix}-{sequence}",
        std::process::id()
    ));
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

#[test]
fn preflight_turns_green_after_matching_verification_evidence() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "cockpit-preflight-green-{}-{suffix}-{sequence}",
        std::process::id()
    ));
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
    let start = Command::new(binary)
        .args(["start", "--repo"])
        .arg(&directory)
        .args([
            "--id",
            "WI-PREFLIGHT",
            "--intent",
            "verify",
            "--goal",
            "green after evidence",
            "--scope",
            "src/**",
            "--required-evidence",
            "verification",
        ])
        .output()
        .expect("start");
    assert!(
        start.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&start.stderr)
    );
    let verify = Command::new(binary)
        .args(["verify", "--repo"])
        .arg(&directory)
        .args(["--work-item", "WI-PREFLIGHT", "--command", "true"])
        .output()
        .expect("verify");
    assert!(verify.status.success());
    let contract = directory.join(".ai/work-items/active/WI-PREFLIGHT.contract.json");
    let output = Command::new(binary)
        .args(["preflight", "--repo"])
        .arg(&directory)
        .args(["--contract"])
        .arg(contract)
        .output()
        .expect("preflight");
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON");
    assert_eq!(json["state"], "green");
    assert!(json["unknowns"].as_array().expect("unknowns").is_empty());
    fs::remove_file(directory.join(".ai/project.json")).expect("remove profile");
    let stale = Command::new(binary)
        .args(["preflight", "--repo"])
        .arg(&directory)
        .args(["--contract"])
        .arg(directory.join(".ai/work-items/active/WI-PREFLIGHT.contract.json"))
        .output()
        .expect("stale preflight");
    assert!(stale.status.success());
    let stale_json: serde_json::Value = serde_json::from_slice(&stale.stdout).expect("JSON");
    assert_eq!(stale_json["state"], "red");
    assert!(
        stale_json["blockers"]
            .as_array()
            .expect("blockers")
            .iter()
            .any(|value| value == "stale_contract")
    );
    fs::remove_dir_all(directory).expect("cleanup");
}

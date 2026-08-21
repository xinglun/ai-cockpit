use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_REPOSITORY_ID: AtomicU64 = AtomicU64::new(0);

fn downgrade_to_schema_one(root: &std::path::Path) {
    for name in ["project.json", "agent-interface.json"] {
        let path = root.join(".ai").join(name);
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).expect("protocol JSON")).expect("JSON");
        value
            .as_object_mut()
            .expect("object")
            .remove("repositorySchemaVersion");
        fs::write(&path, serde_json::to_vec_pretty(&value).expect("JSON")).expect("write JSON");
    }
    let config = root.join(".ai/cockpit.toml");
    let text = fs::read_to_string(&config).expect("config");
    fs::write(
        config,
        text.lines()
            .filter(|line| !line.starts_with("repository_schema_version"))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    )
    .expect("write config");
}

fn set_schema_version(root: &std::path::Path, version: u64) {
    for name in ["project.json", "agent-interface.json"] {
        let path = root.join(".ai").join(name);
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).expect("protocol JSON")).expect("JSON");
        value
            .as_object_mut()
            .expect("object")
            .insert("repositorySchemaVersion".into(), version.into());
        fs::write(&path, serde_json::to_vec_pretty(&value).expect("JSON")).expect("write JSON");
    }
    let config = root.join(".ai/cockpit.toml");
    let text = fs::read_to_string(&config).expect("config");
    fs::write(
        config,
        text.lines()
            .map(|line| {
                if line.starts_with("repository_schema_version") {
                    format!("repository_schema_version = {version}")
                } else {
                    line.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    )
    .expect("write config");
}

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

#[test]
fn preflight_rejects_a_repository_that_requires_migration() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "cockpit-preflight-migration-{}-{suffix}-{sequence}",
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
            "WI-MIGRATION-PREFLIGHT",
            "--intent",
            "verify",
            "--goal",
            "migration gate",
            "--scope",
            "src/**",
            "--authority",
            "authorized",
        ])
        .output()
        .expect("start");
    assert!(start.status.success());
    downgrade_to_schema_one(&directory);
    let contract = directory.join(".ai/work-items/active/WI-MIGRATION-PREFLIGHT.contract.json");
    let output = Command::new(binary)
        .args(["preflight", "--repo"])
        .arg(&directory)
        .args(["--contract"])
        .arg(&contract)
        .output()
        .expect("preflight");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("MIGRATION_REQUIRED"));
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn preflight_rejects_a_repository_with_an_unsupported_future_schema() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "cockpit-preflight-incompatible-{}-{suffix}-{sequence}",
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
    set_schema_version(&directory, 999);
    let output = Command::new(binary)
        .args(["preflight", "--repo"])
        .arg(&directory)
        .args(["--contract", ".ai/work-items/active/missing.contract.json"])
        .output()
        .expect("preflight");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("INCOMPATIBLE"));
    fs::remove_dir_all(directory).expect("cleanup");
}

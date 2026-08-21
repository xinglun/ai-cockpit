use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn repository() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "cockpit-work-item-scaffold-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("repository");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .status()
            .expect("git init")
            .success()
    );
    root
}

#[test]
fn new_work_item_reports_facts_and_keeps_human_decisions_empty() {
    let root = repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let output = Command::new(binary)
        .args(["work-item", "new", "--repo"])
        .arg(&root)
        .args(["--id", "payment-refund-guard", "--mode", "code"])
        .output()
        .expect("work item scaffold");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in [
        "Work Item scaffold created.",
        "repositoryId              resolved",
        "baseRevision              resolved",
        "projectProfileDigest      resolved",
        "repositorySnapshotDigest resolved",
        "intent",
        "scope",
        "acceptanceCriteria",
        "authority",
        "State: not_ready",
    ] {
        assert!(stdout.contains(expected), "missing {expected}: {stdout}");
    }

    let contract_path = root.join(".ai/work-items/active/payment-refund-guard.contract.json");
    let contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract"))
            .expect("contract JSON");
    assert_eq!(contract["repositoryId"].as_str().unwrap().len(), 71);
    assert_eq!(contract["mode"], "code");
    assert_eq!(contract["state"], "not_ready");
    assert_eq!(contract["intent"], "");
    assert_eq!(contract["scope"], serde_json::json!([]));
    assert_eq!(contract["acceptanceCriteria"], serde_json::json!([]));
    assert_eq!(contract["authority"], "unknown");
    for forbidden in ["passed", "approved", "verified", "completed"] {
        assert_ne!(contract["state"], forbidden);
    }

    let preflight = Command::new(binary)
        .args(["preflight", "--repo"])
        .arg(&root)
        .args(["--contract"])
        .arg(&contract_path)
        .output()
        .expect("preflight");
    assert!(preflight.status.success());
    let decision: serde_json::Value = serde_json::from_slice(&preflight.stdout).expect("decision");
    assert_ne!(decision["state"], "Green");
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn work_item_new_requires_an_explicit_repository() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let output = Command::new(binary)
        .args(["work-item", "new", "--id", "missing-repo", "--mode", "code"])
        .output()
        .expect("command");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--repo"));
}

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

#[test]
fn work_item_new_rejects_a_repository_that_requires_migration() {
    let root = repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let attach = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&root)
        .output()
        .expect("attach");
    assert!(attach.status.success());
    downgrade_to_schema_one(&root);

    let output = Command::new(binary)
        .args(["work-item", "new", "--repo"])
        .arg(&root)
        .args(["--id", "legacy", "--mode", "code"])
        .output()
        .expect("work-item scaffold");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("MIGRATION_REQUIRED"));
    assert!(
        !root
            .join(".ai/work-items/active/legacy.contract.json")
            .exists()
    );
    assert!(
        !root
            .join(".ai/work-items/active/legacy.summary.json")
            .exists()
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn work_item_new_rejects_a_repository_with_an_unsupported_future_schema() {
    let root = repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let attach = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&root)
        .output()
        .expect("attach");
    assert!(attach.status.success());
    set_schema_version(&root, 999);
    let output = Command::new(binary)
        .args(["work-item", "new", "--repo"])
        .arg(&root)
        .args(["--id", "future", "--mode", "code"])
        .output()
        .expect("work-item scaffold");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("INCOMPATIBLE"));
    assert!(
        !root
            .join(".ai/work-items/active/future.contract.json")
            .exists()
    );
    fs::remove_dir_all(root).expect("cleanup");
}

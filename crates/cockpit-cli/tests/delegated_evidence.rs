use cockpit_core::Digest;
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
    let path = std::env::temp_dir().join(format!(
        "cockpit-cli-delegated-evidence-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("repository");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&path)
        .status()
        .expect("git init");
    path
}

#[test]
fn cli_imports_and_lists_a_bound_delegated_evidence_receipt() {
    let repo = repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    assert!(
        Command::new(binary)
            .args(["attach", "--repo"])
            .arg(&repo)
            .status()
            .expect("attach")
            .success()
    );
    assert!(
        Command::new(binary)
            .args(["start", "--repo"])
            .arg(&repo)
            .args([
                "--id",
                "WI-EXTERNAL-CLI",
                "--intent",
                "bind provider evidence",
                "--goal",
                "preserve provider proof",
                "--scope",
                "**",
                "--authority",
                "authorized",
                "--required-evidence",
                "delegated:github",
            ])
            .status()
            .expect("start")
            .success()
    );
    let raw = br#"{"run":789,"conclusion":"success"}"#;
    let raw_path = repo.join("provider.json");
    fs::write(&raw_path, raw).expect("raw evidence");
    let metadata_path = repo.join("provider.metadata.json");
    fs::write(
        &metadata_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "provider": "github",
            "subject": "run:789",
            "origin": "https://github.com/example/repo/actions/runs/789",
            "assurance": "provider_verified",
            "collectedAt": "2026-08-21T19:00:00Z",
            "digest": Digest::sha256_bytes(raw),
            "validity": "valid",
            "rawEvidenceRef": ".ai/evidence/external/github-run-789.json"
        }))
        .expect("metadata JSON"),
    )
    .expect("metadata");
    let imported = Command::new(binary)
        .args(["evidence", "import", "--repo"])
        .arg(&repo)
        .args(["--work-item", "WI-EXTERNAL-CLI", "--metadata"])
        .arg(&metadata_path)
        .args(["--raw"])
        .arg(&raw_path)
        .output()
        .expect("import");
    assert!(
        imported.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&imported.stderr)
    );
    let receipt: serde_json::Value = serde_json::from_slice(&imported.stdout).expect("receipt");
    assert_eq!(receipt["workItemId"], "WI-EXTERNAL-CLI");
    assert_eq!(receipt["evidence"]["provider"], "github");

    let listed = Command::new(binary)
        .args(["evidence", "list", "--repo"])
        .arg(&repo)
        .args(["--work-item", "WI-EXTERNAL-CLI"])
        .output()
        .expect("list");
    assert!(listed.status.success());
    let receipts: serde_json::Value = serde_json::from_slice(&listed.stdout).expect("list JSON");
    assert_eq!(receipts.as_array().expect("array").len(), 1);
    assert!(
        repo.join(".ai/evidence/external/github-run-789.json")
            .is_file()
    );
    fs::remove_dir_all(repo).expect("cleanup");
}

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

#[test]
fn cli_observes_corrected_source_for_identical_raw_bytes_without_replacing_legacy_receipt() {
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
                "WI-EXTERNAL-OBSERVATION",
                "--intent",
                "preserve repeated provider observations",
                "--goal",
                "append a corrected source observation",
                "--scope",
                ".ai/**",
                "--authority",
                "authorized",
            ])
            .status()
            .expect("start")
            .success()
    );

    let raw = br#"{"run":789,"conclusion":"success"}"#;
    let raw_path = repo.join("provider.json");
    fs::write(&raw_path, raw).expect("raw evidence");
    let metadata_path = repo.join("provider.metadata.json");
    let metadata = |origin: &str| {
        serde_json::to_vec_pretty(&serde_json::json!({
            "provider": "github",
            "subject": "run:789",
            "origin": origin,
            "assurance": "provider_verified",
            "collectedAt": "2026-10-14T10:00:00Z",
            "digest": Digest::sha256_bytes(raw),
            "validity": "valid",
            "rawEvidenceRef": ".ai/evidence/external/github-run-789.json"
        }))
        .expect("metadata JSON")
    };
    fs::write(
        &metadata_path,
        metadata("https://api.github.com/repos/example/repo/actions/runs/789"),
    )
    .expect("legacy metadata");
    let imported = Command::new(binary)
        .args(["evidence", "import", "--repo"])
        .arg(&repo)
        .args(["--work-item", "WI-EXTERNAL-OBSERVATION", "--metadata"])
        .arg(&metadata_path)
        .args(["--raw"])
        .arg(&raw_path)
        .output()
        .expect("legacy import");
    assert!(
        imported.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&imported.stderr)
    );
    let legacy_receipt_path = repo.join(format!(
        ".ai/evidence/external/WI-EXTERNAL-OBSERVATION.{}.delegated.json",
        Digest::sha256_bytes(raw)
            .as_str()
            .strip_prefix("sha256:")
            .expect("sha256 digest")
    ));
    let legacy_receipt_before = fs::read(&legacy_receipt_path).expect("legacy receipt");
    let raw_before = fs::read(repo.join(".ai/evidence/external/github-run-789.json"))
        .expect("stored raw evidence");

    fs::write(
        &metadata_path,
        metadata("https://api.github.com/repos/example/repo/actions/runs/789?corrected=1"),
    )
    .expect("corrected metadata");
    let observed = Command::new(binary)
        .args(["evidence", "observe", "--repo"])
        .arg(&repo)
        .args(["--work-item", "WI-EXTERNAL-OBSERVATION", "--metadata"])
        .arg(&metadata_path)
        .args(["--raw"])
        .arg(&raw_path)
        .output()
        .expect("record corrected observation");
    assert!(
        observed.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&observed.stderr)
    );
    let first: serde_json::Value =
        serde_json::from_slice(&observed.stdout).expect("observation JSON");
    assert_eq!(first["receipt"]["schemaVersion"], 1);
    assert_eq!(
        first["receipt"]["evidence"]["origin"],
        "https://api.github.com/repos/example/repo/actions/runs/789?corrected=1"
    );
    assert_eq!(first["selector"]["schemaVersion"], 1);
    let selector = first["selector"]["observationDigest"].clone();
    assert_eq!(
        first["receipt"]["evidence"]["digest"],
        Digest::sha256_bytes(raw).to_string()
    );

    let listed = Command::new(binary)
        .args(["evidence", "list-observations", "--repo"])
        .arg(&repo)
        .args(["--work-item", "WI-EXTERNAL-OBSERVATION"])
        .output()
        .expect("list observations");
    assert!(listed.status.success());
    let observations: serde_json::Value =
        serde_json::from_slice(&listed.stdout).expect("observation list JSON");
    assert_eq!(observations.as_array().expect("array").len(), 1);
    assert_eq!(observations[0]["selector"]["observationDigest"], selector);

    let repeated = Command::new(binary)
        .args(["evidence", "observe", "--repo"])
        .arg(&repo)
        .args(["--work-item", "WI-EXTERNAL-OBSERVATION", "--metadata"])
        .arg(&metadata_path)
        .args(["--raw"])
        .arg(&raw_path)
        .output()
        .expect("idempotent observation");
    assert!(repeated.status.success());
    let second: serde_json::Value =
        serde_json::from_slice(&repeated.stdout).expect("repeated observation JSON");
    assert_eq!(second["selector"]["observationDigest"], selector);
    assert_eq!(second["receipt"]["boundAt"], first["receipt"]["boundAt"]);

    assert_eq!(
        fs::read(&legacy_receipt_path).expect("legacy receipt remains"),
        legacy_receipt_before
    );
    assert_eq!(
        fs::read(repo.join(".ai/evidence/external/github-run-789.json"))
            .expect("raw evidence remains"),
        raw_before
    );
    fs::remove_dir_all(repo).expect("cleanup");
}

#[test]
fn legacy_import_rejects_an_observation_selector_instead_of_ignoring_it() {
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
                "WI-EXTERNAL-LEGACY-STRICT",
                "--intent",
                "retain strict legacy parsing",
                "--goal",
                "reject unknown selector fields",
                "--scope",
                ".ai/**",
                "--authority",
                "authorized",
            ])
            .status()
            .expect("start")
            .success()
    );
    let raw = br#"{"run":1}"#;
    let raw_path = repo.join("provider.json");
    fs::write(&raw_path, raw).expect("raw evidence");
    let metadata_path = repo.join("provider.metadata.json");
    fs::write(
        &metadata_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "selector": {"schemaVersion": 1, "observationDigest": Digest::sha256_bytes(b"selector")},
            "provider": "github",
            "subject": "run:1",
            "origin": "https://api.github.com/repos/example/repo/actions/runs/1",
            "assurance": "provider_verified",
            "collectedAt": "2026-10-14T10:00:00Z",
            "digest": Digest::sha256_bytes(raw),
            "validity": "valid",
            "rawEvidenceRef": ".ai/evidence/external/github-run-1.json"
        }))
        .expect("metadata JSON"),
    )
    .expect("selector metadata");
    let rejected = Command::new(binary)
        .args(["evidence", "import", "--repo"])
        .arg(&repo)
        .args(["--work-item", "WI-EXTERNAL-LEGACY-STRICT", "--metadata"])
        .arg(&metadata_path)
        .args(["--raw"])
        .arg(&raw_path)
        .output()
        .expect("legacy import with new selector");
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8_lossy(&rejected.stderr).contains("selector"),
        "stderr: {}",
        String::from_utf8_lossy(&rejected.stderr)
    );
    assert!(
        !repo
            .join(".ai/evidence/external/WI-EXTERNAL-LEGACY-STRICT")
            .exists(),
        "rejected legacy input must not create evidence"
    );
    fs::remove_dir_all(repo).expect("cleanup");
}

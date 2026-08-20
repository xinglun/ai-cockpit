use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn repository() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("cockpit-lifecycle-{suffix}"));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&directory)
        .status()
        .expect("git init");
    directory
}

fn run(binary: &str, args: &[&str], repo: &std::path::Path) -> serde_json::Value {
    let output = Command::new(binary)
        .args(args)
        .args(["--repo"])
        .arg(repo)
        .output()
        .expect("run command");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    if output.stdout.is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_slice(&output.stdout).expect("JSON")
    }
}

#[test]
fn work_item_lifecycle_is_atomic_and_archive_is_content_bound() {
    let repo = repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let attached = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&repo)
        .output()
        .expect("attach");
    assert!(attached.status.success());
    let started = Command::new(binary)
        .args(["start", "--repo"])
        .arg(&repo)
        .args([
            "--id",
            "WI-TEST",
            "--intent",
            "exercise lifecycle",
            "--goal",
            "preserve evidence",
            "--scope",
            "src/**",
        ])
        .output()
        .expect("start");
    assert!(
        started.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&started.stderr)
    );
    assert!(
        repo.join(".ai/work-items/active/WI-TEST.contract.json")
            .is_file()
    );
    assert!(
        repo.join(".ai/work-items/active/WI-TEST.summary.json")
            .is_file()
    );
    run(binary, &["checkpoint", "--id", "WI-TEST"], &repo);
    run(
        binary,
        &["verify", "--work-item", "WI-TEST", "--command", "true"],
        &repo,
    );
    run(binary, &["finish", "--id", "WI-TEST"], &repo);
    assert!(
        repo.join(".ai/work-items/active/WI-TEST.outcome.json")
            .is_file()
    );
    run(binary, &["archive", "--id", "WI-TEST"], &repo);
    assert!(
        !repo
            .join(".ai/work-items/active/WI-TEST.contract.json")
            .exists()
    );
    assert!(
        repo.join(".ai/work-items/archive/WI-TEST.archive.json")
            .is_file()
    );
    run(binary, &["close", "--id", "WI-TEST"], &repo);
    let status = run(binary, &["status"], &repo);
    assert_eq!(status["archivedWorkItems"], 1);
    fs::remove_dir_all(repo).expect("cleanup");
}

#[test]
fn invalid_work_item_id_is_rejected_without_path_traversal() {
    let repo = repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let output = Command::new(binary)
        .args(["start", "--repo"])
        .arg(&repo)
        .args([
            "--id",
            "../escape",
            "--intent",
            "bad",
            "--goal",
            "bad",
            "--scope",
            "**",
        ])
        .output()
        .expect("start");
    assert!(!output.status.success());
    assert!(
        !repo
            .parent()
            .expect("parent")
            .join("escape.contract.json")
            .exists()
    );
    fs::remove_dir_all(repo).expect("cleanup");
}

#[test]
fn archive_failure_keeps_active_files_for_recovery() {
    let repo = repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let attach = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&repo)
        .output()
        .expect("attach");
    assert!(attach.status.success());
    let start = Command::new(binary)
        .args(["start", "--repo"])
        .arg(&repo)
        .args([
            "--id",
            "WI-RECOVER",
            "--intent",
            "recover",
            "--goal",
            "test",
            "--scope",
            "**",
        ])
        .output()
        .expect("start");
    assert!(start.status.success());
    let finish = Command::new(binary)
        .args(["verify", "--repo"])
        .arg(&repo)
        .args(["--work-item", "WI-RECOVER", "--command", "true"])
        .output()
        .expect("verify");
    assert!(finish.status.success());
    let finish = Command::new(binary)
        .args(["finish", "--repo"])
        .arg(&repo)
        .args(["--id", "WI-RECOVER"])
        .output()
        .expect("finish");
    assert!(finish.status.success());
    fs::remove_file(repo.join(".ai/work-items/active/WI-RECOVER.outcome.json"))
        .expect("remove outcome");
    let archive = Command::new(binary)
        .args(["archive", "--repo"])
        .arg(&repo)
        .args(["--id", "WI-RECOVER"])
        .output()
        .expect("archive");
    assert!(!archive.status.success());
    assert!(
        repo.join(".ai/work-items/active/WI-RECOVER.contract.json")
            .is_file()
    );
    assert!(
        repo.join(".ai/work-items/active/WI-RECOVER.summary.json")
            .is_file()
    );
    fs::remove_dir_all(repo).expect("cleanup");
}

#[test]
fn finish_rejects_self_declared_completion_without_receipt() {
    let repo = repository();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let attach = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&repo)
        .output()
        .expect("attach");
    assert!(attach.status.success());
    let start = Command::new(binary)
        .args(["start", "--repo"])
        .arg(&repo)
        .args([
            "--id",
            "WI-NO-RECEIPT",
            "--intent",
            "negative",
            "--goal",
            "must block",
            "--scope",
            "**",
        ])
        .output()
        .expect("start");
    assert!(start.status.success());
    let finish = Command::new(binary)
        .args(["finish", "--repo"])
        .arg(&repo)
        .args(["--id", "WI-NO-RECEIPT"])
        .output()
        .expect("finish");
    assert!(!finish.status.success());
    assert!(
        repo.join(".ai/work-items/active/WI-NO-RECEIPT.summary.json")
            .is_file()
    );
    fs::remove_dir_all(repo).expect("cleanup");
}

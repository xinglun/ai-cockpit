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

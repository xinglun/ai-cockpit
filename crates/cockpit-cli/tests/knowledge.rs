use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn knowledge_query_projects_archived_work_item_records_deterministically() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("cockpit-knowledge-cli-{suffix}"));
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
            "WI-K",
            "--intent",
            "orders projection",
            "--goal",
            "query",
            "--scope",
            "**",
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
        .args(["--work-item", "WI-K", "--command", "true"])
        .output()
        .expect("verify");
    assert!(
        verify.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&verify.stderr)
    );
    for args in [
        vec!["finish", "--id", "WI-K"],
        vec!["archive", "--id", "WI-K"],
    ] {
        let mut command = Command::new(binary);
        command.args(&args).args(["--repo"]).arg(&directory);
        let output = command.output().expect("lifecycle");
        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let output = Command::new(binary)
        .args(["knowledge", "query", "--repo"])
        .arg(&directory)
        .args(["--state", "archived"])
        .output()
        .expect("query");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON");
    assert_eq!(json["matchCount"], 1);
    fs::remove_dir_all(directory).expect("cleanup");
}

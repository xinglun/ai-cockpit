use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_REPOSITORY_ID: AtomicU64 = AtomicU64::new(0);

fn plan(binary: &str, repo: &std::path::Path, work_item_id: &str) {
    let context = tempfile::NamedTempFile::new().expect("resource context");
    fs::write(
        context.path(),
        serde_json::to_vec_pretty(&serde_json::json!({
            "branch": format!("feature/{work_item_id}"),
            "worktree": repo,
            "baseBranch": "main",
            "baseRemote": "origin",
            "provider": "github",
            "pullRequest": format!("https://github.com/example/ai-cockpit/pull/{work_item_id}")
        }))
        .expect("context JSON"),
    )
    .expect("write context");
    let output = Command::new(binary)
        .args([
            "work-item",
            "finalize-plan",
            "--id",
            work_item_id,
            "--input",
        ])
        .arg(context.path())
        .args(["--repo"])
        .arg(repo)
        .output()
        .expect("finalize plan");
    assert!(
        output.status.success(),
        "finalize plan stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn knowledge_query_projects_archived_work_item_records_deterministically() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "cockpit-knowledge-cli-{}-{suffix}-{sequence}",
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
            "WI-K",
            "--intent",
            "orders projection",
            "--goal",
            "query",
            "--scope",
            "**",
            "--out-of-scope",
            "target/**",
            "--acceptance",
            "query passes",
            "--authority",
            "authorized",
        ])
        .output()
        .expect("start");
    assert!(
        start.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&start.stderr)
    );
    plan(binary, &directory, "WI-K");
    let preflight = Command::new(binary)
        .args(["preflight", "--repo"])
        .arg(&directory)
        .args(["--contract", ".ai/work-items/active/WI-K.contract.json"])
        .output()
        .expect("preflight");
    assert!(preflight.status.success());
    let checkpoint = Command::new(binary)
        .args(["checkpoint", "--repo"])
        .arg(&directory)
        .args(["--id", "WI-K"])
        .output()
        .expect("checkpoint");
    assert!(
        checkpoint.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&checkpoint.stderr)
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

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
    let knowledge_dir = directory.join(".ai/knowledge");
    assert!(knowledge_dir.is_dir());
    assert_eq!(
        fs::read_dir(&knowledge_dir).expect("knowledge dir").count(),
        0
    );
    let attach_again = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(&directory)
        .output()
        .expect("idempotent attach");
    assert!(attach_again.status.success());
    assert_eq!(
        fs::read_dir(&knowledge_dir).expect("knowledge dir").count(),
        0
    );
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
    assert!(!knowledge_dir.join("index.json").exists());
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
    assert_eq!(json["projection"]["path"], ".ai/knowledge/index.json");
    assert_eq!(json["projection"]["materialization"], "created");
    assert_eq!(
        json["projection"]["writeBoundary"],
        "repository-local-derived"
    );
    assert_eq!(json["projection"]["authority"], "none");
    assert!(knowledge_dir.join("index.json").is_file());

    let repeated = Command::new(binary)
        .args(["knowledge", "query", "--repo"])
        .arg(&directory)
        .args(["--state", "archived"])
        .output()
        .expect("repeated query");
    assert!(repeated.status.success());
    let repeated_json: serde_json::Value =
        serde_json::from_slice(&repeated.stdout).expect("repeated JSON");
    assert_eq!(repeated_json["projection"]["materialization"], "reused");

    let v2 = Command::new(binary)
        .args(["knowledge", "query", "--repo"])
        .arg(&directory)
        .args(["--v2", "--work-item-id", "WI-K"])
        .output()
        .expect("v2 query");
    assert!(v2.status.success());
    let v2_json: serde_json::Value = serde_json::from_slice(&v2.stdout).expect("v2 JSON");
    assert_eq!(v2_json["schemaVersion"], 2);
    assert_eq!(v2_json["projection"]["path"], ".ai/knowledge/index.v2.json");
    assert_eq!(v2_json["projection"]["materialization"], "created");
    assert_eq!(
        v2_json["projection"]["writeBoundary"],
        "repository-local-derived"
    );
    assert_eq!(v2_json["projection"]["authority"], "none");
    assert!(knowledge_dir.join("index.v2.json").is_file());
    fs::remove_dir_all(directory).expect("cleanup");
}

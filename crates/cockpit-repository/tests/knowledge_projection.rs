use cockpit_core::Digest;
use cockpit_protocol::ResourceFinalizationContext;
use cockpit_repository::{
    archive_work_item, attach, checkpoint_work_item, close_work_item_with_decision,
    finish_work_item, generate_knowledge, generate_knowledge_v2, plan_resource_finalization,
    preflight_work_item, record_verification, start_work_item,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_REPOSITORY_ID: AtomicU64 = AtomicU64::new(0);

fn repository(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "cockpit-knowledge-projection-{name}-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("directory");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(&path)
            .status()
            .expect("git init")
            .success()
    );
    attach(&path).expect("attach");
    path
}

fn archive_one(path: &Path, id: &str) {
    start_work_item(
        path,
        id,
        "projection topic",
        "projection goal",
        &["**".into()],
    )
    .expect("start");
    plan_resource_finalization(
        path,
        id,
        &ResourceFinalizationContext {
            branch: format!("feature/{id}"),
            worktree: path.display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: format!("https://github.com/example/ai-cockpit/pull/{id}"),
        },
    )
    .expect("finalization plan");
    let contract = path
        .join(".ai/work-items/active")
        .join(format!("{id}.contract.json"));
    preflight_work_item(path, &contract).expect("preflight");
    checkpoint_work_item(path, id).expect("checkpoint");
    record_verification(
        path,
        id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.1.0",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(path, id).expect("finish");
    archive_work_item(path, id).expect("archive");
    close_work_item_with_decision(path, id, "approved").expect("close");
}

fn collect_files(root: &Path, directory: &str, files: &mut Vec<(String, Vec<u8>)>) {
    let path = root.join(directory);
    let Ok(entries) = fs::read_dir(&path) else {
        return;
    };
    for entry in entries {
        let entry = entry.expect("directory entry");
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .expect("relative path")
            .to_string_lossy()
            .replace('\\', "/");
        if path.is_dir() {
            collect_files(root, &relative, files);
        } else {
            files.push((relative, fs::read(path).expect("authority file")));
        }
    }
}

fn authority_snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files = Vec::new();
    for directory in [".ai/work-items/archive", ".ai/evidence", ".ai/decisions"] {
        collect_files(root, directory, &mut files);
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

#[test]
fn attach_knowledge_directory_is_idempotent_and_empty() {
    let path = repository("attach");
    let knowledge = path.join(".ai/knowledge");
    assert!(knowledge.is_dir());
    assert_eq!(fs::read_dir(&knowledge).expect("knowledge").count(), 0);
    let protocol_before = ["cockpit.toml", "project.json", "agent-interface.json"]
        .map(|name| fs::read(path.join(".ai").join(name)).expect("protocol"));
    attach(&path).expect("idempotent attach");
    let protocol_after = ["cockpit.toml", "project.json", "agent-interface.json"]
        .map(|name| fs::read(path.join(".ai").join(name)).expect("protocol"));
    assert_eq!(protocol_before, protocol_after);
    assert_eq!(fs::read_dir(&knowledge).expect("knowledge").count(), 0);
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn projections_rebuild_explicitly_without_mutating_authority() {
    let path = repository("authority");
    archive_one(&path, "WI-KNOWLEDGE");
    let authority_before = authority_snapshot(&path);
    let first = generate_knowledge(&path).expect("legacy projection");
    let legacy_bytes = fs::read(path.join(".ai/knowledge/index.json")).expect("legacy index");
    let v2 = generate_knowledge_v2(&path).expect("v2 projection");
    assert_eq!(v2.len(), 1);
    assert_eq!(
        v2[0].repository_id,
        cockpit_repository::repository_id(&path).to_string()
    );
    assert_eq!(authority_before, authority_snapshot(&path));

    fs::write(path.join(".ai/knowledge/index.json"), b"{malformed\n").expect("tamper index");
    let rebuilt = generate_knowledge(&path).expect("rebuild projection");
    assert_eq!(rebuilt, first);
    assert_eq!(
        fs::read(path.join(".ai/knowledge/index.json")).expect("rebuilt index"),
        legacy_bytes
    );
    assert_eq!(authority_before, authority_snapshot(&path));
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn knowledge_v2_projection_is_repository_isolated() {
    let left = repository("left");
    let right = repository("right");
    archive_one(&left, "WI-LEFT");
    archive_one(&right, "WI-RIGHT");
    let left_records = generate_knowledge_v2(&left).expect("left projection");
    let right_records = generate_knowledge_v2(&right).expect("right projection");
    assert_eq!(left_records.len(), 1);
    assert_eq!(right_records.len(), 1);
    assert_eq!(left_records[0].work_item_id, "WI-LEFT");
    assert_eq!(right_records[0].work_item_id, "WI-RIGHT");
    assert_ne!(
        left_records[0].repository_id,
        right_records[0].repository_id
    );
    assert!(!left.join(".ai/knowledge/WI-RIGHT.v2.json").exists());
    assert!(!right.join(".ai/knowledge/WI-LEFT.v2.json").exists());
    fs::remove_dir_all(left).expect("left cleanup");
    fs::remove_dir_all(right).expect("right cleanup");
}

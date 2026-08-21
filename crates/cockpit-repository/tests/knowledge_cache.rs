use cockpit_core::Digest;
use cockpit_repository::{
    archive_work_item, attach, finish_work_item, generate_knowledge, record_verification,
    start_work_item,
};
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_REPOSITORY_ID: AtomicU64 = AtomicU64::new(0);

fn repository() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "cockpit-knowledge-cache-{}-{suffix}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&path)
        .status()
        .expect("git init");
    attach(&path).expect("attach");
    path
}

fn archive_one(path: &std::path::Path, id: &str) {
    start_work_item(path, id, "cache topic", "cache goal", &["**".into()]).expect("start");
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
}

#[test]
fn knowledge_index_is_reused_and_invalidated_by_new_archive() {
    let path = repository();
    archive_one(&path, "WI-CACHE-1");
    let first = generate_knowledge(&path).expect("first projection");
    assert_eq!(first.records.len(), 1);
    assert!(path.join(".ai/knowledge/index.json").is_file());
    let cached = generate_knowledge(&path).expect("cached projection");
    assert_eq!(cached, first);
    archive_one(&path, "WI-CACHE-2");
    let refreshed = generate_knowledge(&path).expect("refreshed projection");
    assert_eq!(refreshed.records.len(), 2);
    fs::remove_dir_all(path).expect("cleanup");
}

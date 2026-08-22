use cockpit_core::Digest;
use cockpit_repository::{
    archive_work_item, attach, checkpoint_work_item, finish_work_item, generate_knowledge,
    preflight_work_item, record_verification, start_work_item,
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
    assert!(cached.source_digest.starts_with("sha256:"));
    archive_one(&path, "WI-CACHE-2");
    let refreshed = generate_knowledge(&path).expect("refreshed projection");
    assert_eq!(refreshed.records.len(), 2);
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn knowledge_index_cache_is_rebuilt_when_an_archived_input_changes() {
    let path = repository();
    archive_one(&path, "WI-CACHE-TAMPER");
    let first = generate_knowledge(&path).expect("first projection");
    let index_path = path.join(".ai/knowledge/index.json");
    let archive_path = path.join(".ai/work-items/archive/WI-CACHE-TAMPER.contract.json");
    let mut contract = fs::read_to_string(&archive_path).expect("contract");
    contract.push('\n');
    fs::write(&archive_path, contract).expect("tamper archive input");
    let rebuilt = generate_knowledge(&path).expect("rebuild projection");
    assert_ne!(rebuilt.source_digest, first.source_digest);
    let persisted: serde_json::Value =
        serde_json::from_slice(&fs::read(index_path).expect("index")).expect("index JSON");
    assert_eq!(persisted["sourceDigest"], rebuilt.source_digest);
    fs::remove_dir_all(path).expect("cleanup");
}

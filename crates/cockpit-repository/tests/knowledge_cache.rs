use cockpit_core::Digest;
use cockpit_repository::{
    archive_work_item, attach, checkpoint_work_item, close_work_item_with_decision,
    finish_work_item, generate_knowledge, preflight_work_item, record_verification,
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
    start_work_item(path, id, "cache topic", "cache goal", &[".ai/**".into()]).expect("start");
    let contract_path = path
        .join(".ai/work-items/active")
        .join(format!("{id}.contract.json"));
    let contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract"))
            .expect("contract JSON");
    assert!(contract.get("resourceContext").is_none());
    preflight_work_item(path, &contract_path).expect("preflight");
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

fn commit_repository(path: &std::path::Path, message: &str) {
    assert!(
        Command::new("git")
            .args(["add", "."])
            .current_dir(path)
            .status()
            .expect("git add")
            .success()
    );
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=knowledge-test",
                "-c",
                "user.email=knowledge-test@example.invalid",
                "commit",
                "-qm",
                message,
            ])
            .current_dir(path)
            .status()
            .expect("git commit")
            .success()
    );
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

#[test]
fn knowledge_index_cache_rejects_tampered_record_even_when_source_is_unchanged() {
    let path = repository();
    archive_one(&path, "WI-CACHE-RECORD-TAMPER");
    let first = generate_knowledge(&path).expect("first projection");
    let index_path = path.join(".ai/knowledge/index.json");
    let mut index: serde_json::Value =
        serde_json::from_slice(&fs::read(&index_path).expect("index")).expect("index JSON");
    index["records"][0]["topic"] = serde_json::Value::String("tampered".into());
    fs::write(
        &index_path,
        serde_json::to_vec_pretty(&index).expect("tampered index"),
    )
    .expect("write tampered index");

    let rebuilt = generate_knowledge(&path).expect("rebuild tampered index");
    assert_eq!(rebuilt, first);
    let persisted: serde_json::Value =
        serde_json::from_slice(&fs::read(index_path).expect("rebuilt index")).expect("JSON");
    assert_ne!(persisted["records"][0]["topic"], "tampered");
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn knowledge_index_cache_rejects_tampered_derived_index_even_when_source_is_unchanged() {
    let path = repository();
    archive_one(&path, "WI-CACHE-DERIVED-TAMPER");
    let first = generate_knowledge(&path).expect("first projection");
    let index_path = path.join(".ai/knowledge/index.json");
    let mut index: serde_json::Value =
        serde_json::from_slice(&fs::read(&index_path).expect("index")).expect("index JSON");
    index["by_topic"]["cache"] = serde_json::json!([]);
    fs::write(
        &index_path,
        serde_json::to_vec_pretty(&index).expect("tampered index"),
    )
    .expect("write tampered index");

    let rebuilt = generate_knowledge(&path).expect("rebuild tampered index");
    assert_eq!(rebuilt, first);
    let persisted: serde_json::Value =
        serde_json::from_slice(&fs::read(index_path).expect("rebuilt index")).expect("JSON");
    assert_eq!(
        persisted["by_topic"]["cache"],
        serde_json::json!(["WI-CACHE-DERIVED-TAMPER"])
    );
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn legacy_index_shape_is_rebuilt_before_reuse() {
    let path = repository();
    archive_one(&path, "WI-CACHE-LEGACY");
    let expected = generate_knowledge(&path).expect("first projection");
    let index_path = path.join(".ai/knowledge/index.json");
    let mut legacy: serde_json::Value =
        serde_json::from_slice(&fs::read(&index_path).expect("index")).expect("index JSON");
    legacy
        .as_object_mut()
        .expect("index object")
        .remove("recordDigest");
    legacy
        .as_object_mut()
        .expect("index object")
        .remove("recordPositions");
    legacy
        .as_object_mut()
        .expect("index object")
        .remove("sourceRevision");
    fs::write(
        &index_path,
        serde_json::to_vec_pretty(&legacy).expect("legacy index"),
    )
    .expect("write legacy index");

    let rebuilt = generate_knowledge(&path).expect("rebuild legacy index");
    assert_eq!(rebuilt, expected);
    assert!(!rebuilt.record_digest.is_empty());
    assert!(!rebuilt.record_positions.is_empty());
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn knowledge_index_records_source_revision_for_clean_fast_path() {
    let path = repository();
    archive_one(&path, "WI-CACHE-CLEAN");
    commit_repository(&path, "archive knowledge fixture");
    let index = generate_knowledge(&path).expect("first projection");
    assert!(index.source_revision.is_some());
    let cached = generate_knowledge(&path).expect("clean cache hit");
    assert_eq!(cached, index);
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn dirty_cache_cannot_be_reused_after_archived_input_is_restored_clean() {
    let path = repository();
    archive_one(&path, "WI-CACHE-DIRTY-CLEAN");
    commit_repository(&path, "archive knowledge baseline");
    let contract_path = path.join(".ai/work-items/archive/WI-CACHE-DIRTY-CLEAN.contract.json");
    let baseline_contract = fs::read(&contract_path).expect("baseline contract");
    let baseline_revision = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&path)
            .output()
            .expect("revision")
            .stdout,
    )
    .expect("revision UTF-8")
    .trim()
    .to_owned();

    let mut dirty_contract: serde_json::Value =
        serde_json::from_slice(&baseline_contract).expect("baseline JSON");
    dirty_contract["intent"] = serde_json::Value::String("release dirty cache".into());
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&dirty_contract).expect("dirty JSON"),
    )
    .expect("write dirty contract");
    let dirty = generate_knowledge(&path).expect("dirty projection");
    assert_eq!(dirty.source_revision, None);
    assert_eq!(dirty.records[0].topic, "release");
    let dirty_source_digest = dirty.source_digest.clone();

    let dirty_again = generate_knowledge(&path).expect("repeated dirty projection");
    assert_eq!(dirty_again.source_revision, None);
    assert_eq!(dirty_again.source_digest, dirty_source_digest);
    assert_eq!(dirty_again.records[0].topic, "release");

    fs::write(&contract_path, &baseline_contract).expect("restore baseline contract");
    let restored = generate_knowledge(&path).expect("restored projection");
    assert_eq!(restored.records[0].topic, "cache");
    assert_ne!(restored.source_digest, dirty_source_digest);
    assert_eq!(
        restored.source_revision.as_deref(),
        Some(baseline_revision.as_str())
    );
    let persisted: serde_json::Value = serde_json::from_slice(
        &fs::read(path.join(".ai/knowledge/index.json")).expect("persisted index"),
    )
    .expect("persisted JSON");
    assert_eq!(persisted["sourceDigest"], restored.source_digest);
    assert_eq!(persisted["records"][0]["topic"], "cache");
    fs::remove_dir_all(path).expect("cleanup");
}

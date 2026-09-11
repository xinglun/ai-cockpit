use cockpit_isolation::{
    ManifestRecord, ScanError, scan_tree, scan_tree_to_jsonl, validate_symlink_containment,
};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use tempfile::tempdir;

fn record<'a>(records: &'a [ManifestRecord], path: &str) -> &'a ManifestRecord {
    records
        .iter()
        .find(|record| record.path == path)
        .unwrap_or_else(|| panic!("missing manifest record for {path:?}"))
}

#[test]
fn emits_deterministic_jsonl_with_metadata_and_streaming_hash_stats() {
    let root = tempdir().expect("temporary root");
    fs::create_dir(root.path().join("nested")).expect("nested directory");
    fs::write(root.path().join("nested/data.txt"), b"alpha\n").expect("data");

    let first = scan_tree(root.path()).expect("first scan");
    let second = scan_tree(root.path()).expect("second scan");
    assert_eq!(first.records, second.records);
    assert_eq!(first.stats, second.stats);

    let data = record(&first.records, "nested/data.txt");
    assert_eq!(data.entry_type, "file");
    assert_eq!(
        data.digest.as_deref(),
        Some("sha256:b6a98d9ce9a2d9149288fa3df42d377c3e42737afdcdaf714e33c0a100b51060")
    );
    assert_eq!(data.target, None);
    assert_eq!(data.resolved_target, None);
    assert!(
        data.mode
            .chars()
            .all(|character| character.is_ascii_digit())
    );
    assert!(data.size.as_deref().unwrap().parse::<u64>().is_ok());
    assert!(data.mtime.as_deref().unwrap().parse::<i64>().is_ok());

    assert_eq!(first.stats.entries, 2);
    assert_eq!(first.stats.file_entries, 1);
    assert_eq!(first.stats.hashed_files, 1);
    assert_eq!(first.stats.bytes_hashed, 6);
    assert_eq!(first.stats.metadata_reads, 2);

    let mut jsonl = Cursor::new(Vec::new());
    let stats = scan_tree_to_jsonl(root.path(), &mut jsonl).expect("JSONL scan");
    assert_eq!(stats, first.stats);
    let output = String::from_utf8(jsonl.into_inner()).expect("UTF-8 JSONL");
    assert!(output.ends_with('\n'));
    assert_eq!(output.lines().count(), 2);
    for line in output.lines() {
        let value = serde_json::from_str::<serde_json::Value>(line).expect("valid JSONL record");
        assert!(value.get("type").is_some());
        assert!(value.get("resolvedTarget").is_some());
        assert!(value.get("entry_type").is_none());
        assert!(value.get("resolved_target").is_none());
        serde_json::from_value::<ManifestRecord>(value).expect("compatible record fields");
    }
}

#[test]
fn detects_file_content_changes_without_mtime_or_size_cache() {
    let root = tempdir().expect("temporary root");
    let path = root.path().join("content.txt");
    fs::write(&path, b"alpha").expect("initial content");
    let before = scan_tree(root.path()).expect("before scan");

    fs::write(&path, b"omega").expect("changed content");
    let after = scan_tree(root.path()).expect("after scan");

    assert_ne!(
        record(&before.records, "content.txt").digest,
        record(&after.records, "content.txt").digest
    );
    assert_eq!(
        record(&before.records, "content.txt").size.as_deref(),
        Some("5")
    );
    assert_eq!(
        record(&after.records, "content.txt").size.as_deref(),
        Some("5")
    );
    assert_eq!(before.stats.bytes_hashed, 5);
    assert_eq!(after.stats.bytes_hashed, 5);
}

#[cfg(unix)]
#[test]
fn records_unix_special_paths_and_rejects_non_utf8_names_explicitly() {
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::net::UnixListener;

    let root = tempdir().expect("temporary root");
    let socket_path = root.path().join("control.sock");
    let _socket = UnixListener::bind(&socket_path).expect("Unix socket");
    fs::write(root.path().join("space and 漢字.txt"), b"special").expect("special name");

    let invalid_name = std::ffi::OsString::from_vec(vec![b'i', b'n', b'v', 0xff]);
    let invalid_path = root.path().join(invalid_name);
    if fs::write(&invalid_path, b"invalid name").is_ok() {
        let error = scan_tree(root.path()).expect_err("non-UTF8 path must be explicit");
        assert!(matches!(error, ScanError::NonUtf8Path { .. }));
    }

    let clean_root = tempdir().expect("clean temporary root");
    let clean_socket_path = clean_root.path().join("control.sock");
    let _clean_socket = UnixListener::bind(&clean_socket_path).expect("clean Unix socket");
    fs::write(clean_root.path().join("space and 漢字.txt"), b"special").expect("special name");
    fs::write(clean_root.path().join("line\nbreak.txt"), b"newline").expect("newline name");
    let result = scan_tree(clean_root.path()).expect("special paths");
    assert_eq!(record(&result.records, "control.sock").entry_type, "other");
    assert_eq!(record(&result.records, "control.sock").digest, None);
    assert_eq!(result.stats.special_entries, 1);
    assert!(record(&result.records, "line\nbreak.txt").digest.is_some());
}

#[cfg(unix)]
#[test]
fn records_symlink_target_resolution_dangling_and_cycles() {
    use std::os::unix::fs::symlink;

    let root = tempdir().expect("temporary root");
    fs::create_dir(root.path().join("nested")).expect("nested directory");
    fs::write(root.path().join("nested/data.txt"), b"payload").expect("data");
    symlink("nested/data.txt", root.path().join("file-link")).expect("file link");
    symlink("missing", root.path().join("dangling-link")).expect("dangling link");
    symlink("cycle-b", root.path().join("cycle-a")).expect("cycle a");
    symlink("cycle-a", root.path().join("cycle-b")).expect("cycle b");

    let result = scan_tree(root.path()).expect("symlink scan");
    let link = record(&result.records, "file-link");
    assert_eq!(link.entry_type, "symlink");
    assert_eq!(link.target.as_deref(), Some("nested/data.txt"));
    assert_eq!(
        link.resolved_target.as_deref(),
        Some(
            fs::canonicalize(root.path().join("nested/data.txt"))
                .expect("canonical target")
                .to_str()
                .expect("UTF-8 target")
        )
    );
    assert_eq!(
        record(&result.records, "dangling-link").resolved_target,
        None
    );
    assert_eq!(record(&result.records, "cycle-a").resolved_target, None);
    assert_eq!(record(&result.records, "cycle-b").resolved_target, None);
    assert_eq!(result.stats.symlink_entries, 4);
    assert_eq!(result.stats.symlink_hops, 82);

    let contained_records = result
        .records
        .iter()
        .filter(|record| record.path == "file-link")
        .cloned()
        .collect::<Vec<_>>();
    validate_symlink_containment(root.path(), &contained_records).expect("contained link");

    let error = validate_symlink_containment(root.path(), &result.records)
        .expect_err("unresolved links must fail containment");
    assert!(matches!(error, ScanError::SymlinkUnresolved { .. }));

    let outside = tempdir().expect("outside root");
    fs::write(outside.path().join("outside.txt"), b"outside").expect("outside data");
    symlink(
        outside.path().join("outside.txt"),
        root.path().join("outside-link"),
    )
    .expect("outside link");
    let result = scan_tree(root.path()).expect("outside-link scan");
    let outside_records = result
        .records
        .iter()
        .filter(|record| record.path == "file-link" || record.path == "outside-link")
        .cloned()
        .collect::<Vec<_>>();
    let error = validate_symlink_containment(root.path(), &outside_records)
        .expect_err("outside link must fail containment");
    assert!(matches!(error, ScanError::SymlinkEscapesRoot { .. }));
}

#[test]
fn missing_or_non_directory_root_is_an_empty_success() {
    let root = tempdir().expect("temporary root");
    let missing = root.path().join("missing");
    assert_eq!(
        scan_tree(&missing).expect("missing root").records,
        Vec::new()
    );

    let file = root.path().join("file");
    fs::write(&file, b"file").expect("file root");
    assert_eq!(scan_tree(&file).expect("file root").records, Vec::new());
}

#[test]
fn scans_selected_paths_once_in_deterministic_order() {
    let root = tempdir().expect("temporary root");
    fs::create_dir(root.path().join(".ai")).expect(".ai");
    fs::write(root.path().join(".ai/project.json"), b"{}").expect("project");
    fs::write(root.path().join("README.md"), b"readme").expect("readme");
    let mut output = Cursor::new(Vec::new());
    let stats = cockpit_isolation::scan_paths_to_jsonl(
        root.path(),
        [
            PathBuf::from("README.md"),
            PathBuf::from(".ai/project.json"),
            PathBuf::from("README.md"),
        ],
        &mut output,
    )
    .expect("selected scan");
    let lines = String::from_utf8(output.into_inner()).expect("UTF-8 JSONL");
    let records = lines
        .lines()
        .map(|line| serde_json::from_str::<ManifestRecord>(line).expect("record"))
        .collect::<Vec<_>>();
    assert_eq!(
        records
            .iter()
            .map(|record| record.path.as_str())
            .collect::<Vec<_>>(),
        [".ai/project.json", "README.md"]
    );
    assert_eq!(stats.entries, 2);
    assert_eq!(stats.file_entries, 2);
}

#[test]
fn selected_missing_paths_emit_shell_compatible_missing_records() {
    let root = tempdir().expect("temporary root");
    fs::write(root.path().join("existing.txt"), b"existing").expect("existing file");
    let mut output = Cursor::new(Vec::new());

    cockpit_isolation::scan_paths_to_jsonl(
        root.path(),
        [PathBuf::from("missing.txt"), PathBuf::from("existing.txt")],
        &mut output,
    )
    .expect("selected scan");

    let records = String::from_utf8(output.into_inner())
        .expect("UTF-8 JSONL")
        .lines()
        .map(|line| serde_json::from_str::<ManifestRecord>(line).expect("record"))
        .collect::<Vec<_>>();
    assert_eq!(
        records
            .iter()
            .map(|record| record.path.as_str())
            .collect::<Vec<_>>(),
        ["existing.txt", "missing.txt"]
    );

    let missing = record(&records, "missing.txt");
    assert_eq!(missing.entry_type, "missing");
    assert_eq!(missing.mode, "0");
    assert_eq!(missing.size.as_deref(), Some("0"));
    assert_eq!(missing.mtime.as_deref(), Some("0"));
    assert_eq!(missing.digest, None);
    assert_eq!(missing.target, None);
    assert_eq!(missing.resolved_target, None);
}

#[test]
fn masks_source_directory_metadata_ancestors_of_excluded_output() {
    let root = tempdir().expect("temporary root");
    fs::create_dir(root.path().join(".ai")).expect(".ai");
    fs::write(root.path().join(".ai/project.json"), b"{}").expect("project");
    let mut output = Cursor::new(Vec::new());
    cockpit_isolation::scan_paths_to_jsonl_with_mask(
        root.path(),
        [PathBuf::from(".ai"), PathBuf::from(".ai/project.json")],
        &mut output,
        Some(std::path::Path::new(".ai/declared-output")),
    )
    .expect("selected scan");
    let records = String::from_utf8(output.into_inner())
        .expect("UTF-8 JSONL")
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("record"))
        .collect::<Vec<_>>();
    let ai = records
        .iter()
        .find(|record| record["path"] == ".ai")
        .expect(".ai record");
    assert!(ai["size"].is_null());
    assert!(ai["mtime"].is_null());
}

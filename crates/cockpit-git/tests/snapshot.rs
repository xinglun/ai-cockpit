use cockpit_git::{IncrementalMerkle, normalize_changed_paths};
use std::fs;
use std::path::Path;

#[test]
fn changed_paths_are_normalized_and_sorted_once() {
    let paths = normalize_changed_paths(["./src/../src/lib.rs", "tests/b.rs", "tests/a.rs"]);
    assert_eq!(paths, vec!["src/lib.rs", "tests/a.rs", "tests/b.rs"]);
}

#[test]
fn incremental_merkle_rehashes_unchanged_files_and_invalidates_changed_content() {
    let root = tempfile::tempdir().expect("tempdir");
    fs::write(root.path().join("a.txt"), "a\n").expect("a");
    fs::write(root.path().join("b.txt"), "b\n").expect("b");
    let mut merkle = IncrementalMerkle::default();
    let first = merkle
        .refresh(root.path(), [Path::new("a.txt"), Path::new("b.txt")])
        .expect("first refresh");
    assert_eq!(first.files_hashed, 2);
    assert_eq!(first.files_reused, 0);

    let second = merkle
        .refresh(root.path(), [Path::new("a.txt"), Path::new("b.txt")])
        .expect("second refresh");
    assert_eq!(second.files_read, 2);
    assert_eq!(second.files_hashed, 2);
    assert_eq!(second.files_reused, 0);
    assert_eq!(first.root_digest, second.root_digest);

    fs::write(root.path().join("b.txt"), "changed\n").expect("change");
    let third = merkle
        .refresh(root.path(), [Path::new("a.txt"), Path::new("b.txt")])
        .expect("third refresh");
    assert_eq!(third.files_read, 2);
    assert_eq!(third.files_hashed, 2);
    assert_eq!(third.files_reused, 0);
    assert_ne!(second.root_digest, third.root_digest);
}

#[test]
fn incremental_merkle_detects_same_length_content_with_restored_mtime() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("same-length.txt");
    fs::write(&path, "old\n").expect("initial content");
    let original_modified = fs::metadata(&path)
        .expect("initial metadata")
        .modified()
        .expect("initial mtime");

    let mut merkle = IncrementalMerkle::default();
    let first = merkle
        .refresh(root.path(), [Path::new("same-length.txt")])
        .expect("first refresh");

    fs::write(&path, "new\n").expect("same-length replacement");
    fs::File::options()
        .write(true)
        .open(&path)
        .expect("open for timestamp restore")
        .set_times(fs::FileTimes::new().set_modified(original_modified))
        .expect("restore mtime");

    let second = merkle
        .refresh(root.path(), [Path::new("same-length.txt")])
        .expect("second refresh");
    assert_eq!(second.files_hashed, 1);
    assert_eq!(second.files_reused, 0);
    assert_ne!(first.root_digest, second.root_digest);
}

#[test]
fn incremental_merkle_handles_replacement_deletion_rename_and_type_change() {
    let root = tempfile::tempdir().expect("tempdir");
    let original = root.path().join("tracked.txt");
    let renamed = root.path().join("renamed.txt");
    fs::write(&original, "old\n").expect("initial content");

    let mut merkle = IncrementalMerkle::default();
    let first = merkle
        .refresh(root.path(), [Path::new("tracked.txt")])
        .expect("first refresh");

    fs::remove_file(&original).expect("remove for replacement");
    fs::write(&original, "replacement\n").expect("replacement content");
    let replaced = merkle
        .refresh(root.path(), [Path::new("tracked.txt")])
        .expect("replacement refresh");
    assert_ne!(first.root_digest, replaced.root_digest);

    fs::rename(&original, &renamed).expect("rename");
    let renamed_result = merkle
        .refresh(root.path(), [Path::new("renamed.txt")])
        .expect("rename refresh");
    assert_ne!(replaced.root_digest, renamed_result.root_digest);

    fs::remove_file(&renamed).expect("delete");
    let deleted = merkle
        .refresh(root.path(), [Path::new("renamed.txt")])
        .expect("delete refresh");
    assert!(deleted.root_digest != renamed_result.root_digest);

    fs::create_dir(&renamed).expect("directory replacement");
    assert!(matches!(
        merkle.refresh(root.path(), [Path::new("renamed.txt")]),
        Err(cockpit_git::ContentIdentityError::NotAFile(_))
    ));
}

#[test]
fn incremental_merkle_rejects_escape_paths_fail_closed() {
    let root = tempfile::tempdir().expect("tempdir");
    let mut merkle = IncrementalMerkle::default();
    assert!(matches!(
        merkle.refresh(root.path(), [Path::new("../outside")]),
        Err(cockpit_git::ContentIdentityError::PathEscape(_))
    ));
}

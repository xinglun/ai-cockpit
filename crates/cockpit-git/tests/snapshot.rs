use cockpit_git::{IncrementalMerkle, normalize_changed_paths};
use std::fs;
use std::path::Path;

#[test]
fn changed_paths_are_normalized_and_sorted_once() {
    let paths = normalize_changed_paths(["./src/../src/lib.rs", "tests/b.rs", "tests/a.rs"]);
    assert_eq!(paths, vec!["src/lib.rs", "tests/a.rs", "tests/b.rs"]);
}

#[test]
fn incremental_merkle_reuses_unchanged_files_and_invalidates_changed_content() {
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
    assert_eq!(second.files_hashed, 0);
    assert_eq!(second.files_reused, 2);
    assert_eq!(first.root_digest, second.root_digest);

    fs::write(root.path().join("b.txt"), "changed\n").expect("change");
    let third = merkle
        .refresh(root.path(), [Path::new("a.txt"), Path::new("b.txt")])
        .expect("third refresh");
    assert_eq!(third.files_hashed, 1);
    assert_eq!(third.files_reused, 1);
    assert_ne!(second.root_digest, third.root_digest);
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

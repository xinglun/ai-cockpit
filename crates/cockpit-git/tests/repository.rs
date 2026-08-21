use cockpit_git::GitRepository;
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn temporary_repository() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("cockpit-git-{suffix}"));
    fs::create_dir_all(&path).expect("temp repo directory");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&path)
        .status()
        .expect("git init");
    Command::new("git")
        .args(["config", "user.email", "test@example.invalid"])
        .current_dir(&path)
        .status()
        .expect("git config");
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(&path)
        .status()
        .expect("git config");
    fs::write(path.join("README.md"), "initial\n").expect("write");
    Command::new("git")
        .args(["add", "."])
        .current_dir(&path)
        .status()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-qm", "initial"])
        .current_dir(&path)
        .status()
        .expect("git commit");
    path
}

#[test]
fn snapshot_observes_head_and_untracked_paths_with_one_snapshot_api() {
    let path = temporary_repository();
    fs::write(path.join("src.txt"), "change\n").expect("write");
    let repository = GitRepository::discover(&path).expect("discover");
    let snapshot = repository.snapshot().expect("snapshot");
    assert!(snapshot.head.is_some());
    assert_eq!(snapshot.changed_paths, vec!["src.txt"]);
    assert_eq!(snapshot.git_calls, 4);
    assert!(snapshot.tree_digest.starts_with("sha256:"));
    assert!(snapshot.diff_digest.starts_with("sha256:"));
    assert!(snapshot.dependency_fingerprint.starts_with("sha256:"));
    fs::remove_dir_all(path).expect("cleanup");
}

#[test]
fn snapshot_hashes_overlapping_changed_and_dependency_paths_once() {
    let path = temporary_repository();
    fs::write(path.join("Cargo.toml"), "[workspace]\nmembers=[]\n").expect("write");
    let snapshot = GitRepository::discover(&path)
        .expect("discover")
        .snapshot()
        .expect("snapshot");
    assert_eq!(snapshot.changed_paths, vec!["Cargo.toml"]);
    assert_eq!(snapshot.files_hashed, 1);
    assert_eq!(snapshot.files_read, 1);
    fs::remove_dir_all(path).expect("cleanup");
}

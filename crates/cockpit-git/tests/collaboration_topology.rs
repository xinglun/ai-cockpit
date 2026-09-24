use cockpit_git::{GitRepository, GitTopologyCompatibility, GitTopologyKind};
use std::fs;
use std::path::Path;
use std::process::Command;

fn run(root: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(root)
        .status()
        .expect("git command");
    assert!(status.success(), "git {:?} failed", args);
}

fn repository() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("tempdir");
    run(root.path(), &["init", "-q"]);
    run(
        root.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    run(root.path(), &["config", "user.name", "Test"]);
    fs::write(root.path().join("README.md"), "initial\n").expect("write");
    run(root.path(), &["add", "."]);
    run(root.path(), &["commit", "-qm", "initial"]);
    root
}

#[test]
fn common_directory_is_shared_by_real_linked_worktrees() {
    let root = repository();
    let linked = tempfile::tempdir().expect("linked parent");
    let linked_path = linked.path().join("linked");
    run(
        root.path(),
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "codex/topology",
            linked_path.to_str().expect("utf8"),
        ],
    );

    let primary = GitRepository::discover(root.path()).expect("primary");
    let worktree = GitRepository::discover(&linked_path).expect("linked");
    let primary_topology = primary.topology().expect("primary topology");
    let linked_topology = worktree.topology().expect("linked topology");
    assert_eq!(primary_topology.common_dir, linked_topology.common_dir);
    assert_eq!(linked_topology.kind, GitTopologyKind::LinkedWorktree);
    assert_eq!(
        primary_topology.compatibility_with(&linked_topology),
        GitTopologyCompatibility::SharedCommonDirectory
    );
}

#[test]
fn independent_clone_is_unsupported_for_shared_coordination() {
    let root = repository();
    let clone = tempfile::tempdir().expect("clone parent");
    let clone_path = clone.path().join("clone");
    let status = Command::new("git")
        .args([
            "clone",
            "-q",
            root.path().to_str().expect("utf8"),
            clone_path.to_str().expect("utf8"),
        ])
        .status()
        .expect("git clone");
    assert!(status.success());
    let primary = GitRepository::discover(root.path()).expect("primary");
    let independent = GitRepository::discover(&clone_path).expect("clone");
    let compatibility = primary
        .topology()
        .expect("primary topology")
        .compatibility_with(&independent.topology().expect("clone topology"));
    assert_eq!(
        compatibility,
        GitTopologyCompatibility::UnsupportedIndependentClone
    );
}

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(["-C"])
        .arg(root)
        .args(args)
        .output()
        .expect("git command");
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_output(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(["-C"])
        .arg(root)
        .args(args)
        .output()
        .expect("git command");
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("git output")
        .trim()
        .to_owned()
}

fn configure_commit(root: &Path, message: &str) {
    git(
        root,
        &[
            "-c",
            "user.email=test@example.com",
            "-c",
            "user.name=test",
            "add",
            "-A",
        ],
    );
    git(
        root,
        &[
            "-c",
            "user.email=test@example.com",
            "-c",
            "user.name=test",
            "commit",
            "-qm",
            message,
        ],
    );
}

fn initialize_repository(root: &Path, with_remote: bool) -> Option<PathBuf> {
    git(root, &["init", "-q"]);
    git(root, &["branch", "-M", "main"]);
    fs::write(root.join("README.md"), "adopter\n").expect("README");
    configure_commit(root, "base");
    if !with_remote {
        return None;
    }
    let origin = root.with_file_name("origin.git");
    fs::create_dir_all(&origin).expect("origin directory");
    git(&origin, &["init", "--bare", "-q"]);
    let origin_path = origin.to_string_lossy().into_owned();
    git(root, &["remote", "add", "origin", origin_path.as_str()]);
    git(root, &["push", "-q", "-u", "origin", "main"]);
    let head = git_output(root, &["rev-parse", "HEAD"]);
    git(root, &["update-ref", "refs/remotes/origin/main", &head]);
    git(
        root,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );
    Some(origin)
}

fn attach_and_commit(root: &Path, binary: &str) {
    let output = Command::new(binary)
        .args(["attach", "--repo"])
        .arg(root)
        .output()
        .expect("attach");
    assert!(
        output.status.success(),
        "attach failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    configure_commit(root, "attach protocol");
    if root.join(".git/config").is_file() {
        // Keep the local default-base ref aligned with the committed protocol
        // files when this fixture has a remote.
        let head = git_output(root, &["rev-parse", "HEAD"]);
        if git_output(root, &["remote"])
            .lines()
            .any(|line| line == "origin")
        {
            git(root, &["push", "-q", "origin", "main"]);
            git(root, &["update-ref", "refs/remotes/origin/main", &head]);
        }
    }
}

fn run_cockpit(binary: &str, repo: &Path, args: &[&str]) -> std::process::Output {
    Command::new(binary)
        .args(args)
        .args(["--repo"])
        .arg(repo)
        .output()
        .expect("ai-cockpit command")
}

#[test]
fn primary_worktree_and_default_branch_are_rejected_but_linked_worktree_is_accepted() {
    let directory = tempfile::tempdir().expect("fixture root");
    let root = directory.path().join("repo");
    fs::create_dir_all(&root).expect("repo");
    let _origin = initialize_repository(&root, true).expect("origin");
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    attach_and_commit(&root, binary);

    let linked = directory.path().join("linked");
    let linked_path = linked.to_string_lossy().into_owned();
    git(
        &root,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "feature/release",
            linked_path.as_str(),
            "main",
        ],
    );

    let primary_new = run_cockpit(
        binary,
        &root,
        &[
            "work-item",
            "new",
            "--id",
            "WI-PRIMARY-NEW",
            "--mode",
            "code",
        ],
    );
    assert!(!primary_new.status.success());
    let primary_new_stderr = String::from_utf8_lossy(&primary_new.stderr);
    assert!(primary_new_stderr.contains("dedicated linked worktree"));

    let primary_start = run_cockpit(
        binary,
        &root,
        &[
            "start",
            "--id",
            "WI-PRIMARY-START",
            "--intent",
            "must stop",
            "--goal",
            "must stop",
            "--scope",
            "README.md",
            "--authority",
            "authorized",
        ],
    );
    assert!(!primary_start.status.success());
    assert!(String::from_utf8_lossy(&primary_start.stderr).contains("dedicated linked worktree"));

    let linked_new = run_cockpit(
        binary,
        &linked,
        &[
            "work-item",
            "new",
            "--id",
            "WI-LINKED-NEW",
            "--mode",
            "code",
        ],
    );
    assert!(
        linked_new.status.success(),
        "linked worktree should be accepted: {}",
        String::from_utf8_lossy(&linked_new.stderr)
    );
    assert!(
        linked
            .join(".ai/work-items/active/WI-LINKED-NEW.contract.json")
            .is_file()
    );

    git(
        &root,
        &["worktree", "remove", "--force", linked_path.as_str()],
    );
    fs::remove_dir_all(directory.path().join("origin.git")).expect("origin cleanup");
}

#[test]
fn linked_worktree_without_unambiguous_default_base_is_rejected() {
    let directory = tempfile::tempdir().expect("fixture root");
    let root = directory.path().join("repo");
    fs::create_dir_all(&root).expect("repo");
    initialize_repository(&root, false);
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    attach_and_commit(&root, binary);
    let linked = directory.path().join("linked");
    let linked_path = linked.to_string_lossy().into_owned();
    git(
        &root,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "feature/no-base",
            linked_path.as_str(),
            "main",
        ],
    );

    let output = run_cockpit(
        binary,
        &linked,
        &[
            "work-item",
            "new",
            "--id",
            "WI-LINKED-NO-BASE",
            "--mode",
            "code",
        ],
    );
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("without an unambiguous discovered remote default base")
    );
    assert!(
        !linked
            .join(".ai/work-items/active/WI-LINKED-NO-BASE.contract.json")
            .exists()
    );

    git(
        &root,
        &["worktree", "remove", "--force", linked_path.as_str()],
    );
}

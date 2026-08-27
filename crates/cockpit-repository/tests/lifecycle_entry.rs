use cockpit_repository::{
    WorkItemStartOptions, attach, scaffold_work_item, start_work_item_with_options, status,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn run(root: &Path, args: &[&str]) {
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

fn output(root: &Path, args: &[&str]) -> String {
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
    String::from_utf8(output.stdout).expect("git output")
}

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("repository");
    run(directory.path(), &["init", "-q"]);
    attach(directory.path()).expect("attach");
    directory
}

fn start_options() -> WorkItemStartOptions {
    WorkItemStartOptions {
        authority: "authorized".into(),
        acceptance_criteria: vec!["entry remains bounded".into()],
        ..Default::default()
    }
}

fn write_unclosed_archive(root: &Path, id: &str) -> (PathBuf, Vec<u8>) {
    let archive = root.join(".ai/work-items/archive");
    fs::create_dir_all(&archive).expect("archive directory");
    let path = archive.join(format!("{id}.archive.json"));
    let bytes = br#"{"schemaVersion":1,"workItemId":"WI-OLD","state":"archived"}
"#
    .to_vec();
    fs::write(&path, &bytes).expect("archive marker");
    (path, bytes)
}

#[test]
fn new_and_start_reject_archived_item_without_close() {
    let directory = repository();
    let (archive_path, archive_bytes) = write_unclosed_archive(directory.path(), "WI-OLD");
    let readiness = status(directory.path()).expect("status").readiness;
    assert!(!readiness.ready_on_base);
    assert_eq!(readiness.state, "blocked");
    assert_eq!(readiness.unclosed_archived_work_items, vec!["WI-OLD"]);

    let scaffold = scaffold_work_item(directory.path(), "WI-NEW", "code")
        .expect_err("new scaffold must stop behind an unclosed archive");
    assert!(scaffold.to_string().contains("archived Work Items"));
    assert!(
        !directory
            .path()
            .join(".ai/work-items/active/WI-NEW.contract.json")
            .exists()
    );

    let start = start_work_item_with_options(
        directory.path(),
        "WI-START",
        "entry gate",
        "stop before unsafe start",
        &["src/**".into()],
        &start_options(),
    )
    .expect_err("start must stop behind an unclosed archive");
    assert!(start.to_string().contains("archived Work Items"));
    assert!(
        !directory
            .path()
            .join(".ai/work-items/active/WI-START.contract.json")
            .exists()
    );
    assert_eq!(
        fs::read(archive_path).expect("archive bytes"),
        archive_bytes
    );
}

#[test]
fn start_rejects_user_changes_that_precede_the_contract() {
    let directory = repository();
    fs::create_dir_all(directory.path().join("src")).expect("src");
    fs::write(directory.path().join("src/main.rs"), "fn main() {}\n").expect("user change");

    let error = start_work_item_with_options(
        directory.path(),
        "WI-DIRTY-START",
        "entry gate",
        "stop before dirty start",
        &["src/**".into()],
        &start_options(),
    )
    .expect_err("dirty pre-start repository must fail closed");
    assert!(error.to_string().contains("before start"));
    assert!(error.to_string().contains("src/main.rs"));
    assert!(
        !directory
            .path()
            .join(".ai/work-items/active/WI-DIRTY-START.contract.json")
            .exists()
    );
}

#[test]
fn start_rejects_clean_branch_ahead_of_discoverable_default_base() {
    let directory = repository();
    fs::write(directory.path().join("README.md"), "base\n").expect("base file");
    run(directory.path(), &["add", "-A"]);
    run(
        directory.path(),
        &[
            "-c",
            "user.email=test@example.com",
            "-c",
            "user.name=test",
            "commit",
            "-qm",
            "base",
        ],
    );
    let base = output(directory.path(), &["rev-parse", "HEAD"])
        .trim()
        .to_owned();
    run(directory.path(), &["branch", "-M", "main"]);
    run(
        directory.path(),
        &[
            "remote",
            "add",
            "origin",
            "https://example.invalid/origin.git",
        ],
    );
    run(
        directory.path(),
        &["update-ref", "refs/remotes/origin/main", &base],
    );
    run(
        directory.path(),
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );
    let readiness = status(directory.path()).expect("status").readiness;
    assert!(readiness.ready_on_base);
    assert_eq!(readiness.state, "ready_on_base");
    assert_eq!(readiness.default_branch.as_deref(), Some("main"));
    fs::write(directory.path().join("README.md"), "ahead\n").expect("ahead change");
    run(directory.path(), &["add", "README.md"]);
    run(
        directory.path(),
        &[
            "-c",
            "user.email=test@example.com",
            "-c",
            "user.name=test",
            "commit",
            "-qm",
            "ahead",
        ],
    );

    let error = start_work_item_with_options(
        directory.path(),
        "WI-AHEAD-START",
        "entry gate",
        "stop ahead branch",
        &["README.md".into()],
        &start_options(),
    )
    .expect_err("branch ahead of default must fail closed");
    assert!(error.to_string().contains("base"));
    assert!(error.to_string().contains("origin/main"));
}

#[test]
fn status_reports_unknown_readiness_without_remote_metadata() {
    let directory = repository();
    let readiness = status(directory.path()).expect("status").readiness;
    assert!(!readiness.ready_on_base);
    assert_eq!(readiness.state, "unknown");
    assert!(
        readiness
            .unknowns
            .iter()
            .any(|value| value == "default_base_unknown")
    );
}

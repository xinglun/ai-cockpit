use cockpit_repository::{
    WorkItemStartOptions, attach, scaffold_work_item, start_work_item_with_options,
};
use std::fs;
use std::path::Path;
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

fn commit(root: &Path) {
    run(root, &["add", "."]);
    run(
        root,
        &[
            "-c",
            "user.email=ai-cockpit@example.invalid",
            "-c",
            "user.name=AI Cockpit Test",
            "commit",
            "-qm",
            "fixture baseline",
        ],
    );
}

fn repository(with_lock: bool) -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("repository");
    run(directory.path(), &["init", "-q"]);
    fs::write(
        directory.path().join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("Cargo.toml");
    if with_lock {
        fs::write(directory.path().join("Cargo.lock"), "# fixture lock\n").expect("Cargo.lock");
    }
    attach(directory.path()).expect("attach");
    commit(directory.path());
    directory
}

fn non_cargo_repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("repository");
    run(directory.path(), &["init", "-q"]);
    fs::write(directory.path().join("README.md"), "fixture\n").expect("README");
    attach(directory.path()).expect("attach");
    commit(directory.path());
    directory
}

fn verification(root: &Path, id: &str) -> serde_json::Value {
    serde_json::from_slice(
        &fs::read(
            root.join(".ai/work-items/active")
                .join(format!("{id}.contract.json")),
        )
        .expect("contract"),
    )
    .expect("contract JSON")
}

fn start(root: &Path, id: &str) {
    scaffold_work_item(root, id, "code").expect("scaffold");
    start_work_item_with_options(
        root,
        id,
        "select the project verification command",
        "keep the declared check executable",
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["verification command is executable".into()],
            ..Default::default()
        },
    )
    .expect("start");
}

#[test]
fn start_declares_unlocked_cargo_when_lockfile_is_absent() {
    let directory = repository(false);
    start(directory.path(), "WI-NO-LOCK");
    assert_eq!(
        verification(directory.path(), "WI-NO-LOCK")["verification"],
        serde_json::json!(["cargo test --workspace"])
    );
}

#[test]
fn start_keeps_locked_cargo_when_lockfile_is_present() {
    let directory = repository(true);
    start(directory.path(), "WI-WITH-LOCK");
    assert_eq!(
        verification(directory.path(), "WI-WITH-LOCK")["verification"],
        serde_json::json!(["cargo test --locked --workspace"])
    );
}

#[test]
fn start_does_not_invent_cargo_for_non_cargo_repository() {
    let directory = non_cargo_repository();
    start(directory.path(), "WI-NON-CARGO");
    assert_eq!(
        verification(directory.path(), "WI-NON-CARGO")["verification"],
        serde_json::json!([])
    );
}

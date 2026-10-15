use cockpit_repository::{WorkItemStartOptions, attach, start_work_item_with_options, status};
use std::{fs, path::Path, process::Command};

const ID: &str = "WI-LEGACY-SOURCE";

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    git(directory.path(), &["init", "--quiet"]);
    attach(directory.path()).unwrap();
    fs::write(directory.path().join("seed.txt"), "baseline\n").unwrap();
    git(directory.path(), &["add", "-A"]);
    git(
        directory.path(),
        &[
            "-c",
            "user.name=Legacy source test",
            "-c",
            "user.email=legacy@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "baseline",
        ],
    );
    directory
}

#[test]
fn malformed_requirement_remains_pending_when_archive_directory_is_missing() {
    // The production change under test is enumeration from canonical proof
    // names before parsing; removing that union must make this assertion fail.
    let directory = repository();
    let root = directory.path();
    let archive = root.join(".ai/work-items/archive");
    if archive.exists() {
        fs::remove_dir_all(&archive).unwrap();
    }
    fs::create_dir_all(root.join(".ai/decisions")).unwrap();
    fs::write(
        root.join(format!(
            ".ai/decisions/{ID}.legacy-closeout-source-requirement.json"
        )),
        b"{broken",
    )
    .unwrap();
    let observed = status(root).unwrap();
    assert!(
        observed
            .readiness
            .unclosed_archived_work_items
            .contains(&ID.to_string()),
        "surviving malformed canonical requirement must retain the WI: {:?}",
        observed.readiness
    );
}

#[test]
fn foreign_registration_cannot_release_untrusted_archived_scope() {
    let directory = repository();
    let root = directory.path();
    fs::create_dir_all(root.join(".ai/decisions")).unwrap();
    fs::write(
        root.join(format!(
            ".ai/decisions/{ID}.legacy-closeout-source-registration.json"
        )),
        br#"{"repositoryId":"foreign","workItemId":"other"}"#,
    )
    .unwrap();
    let result = start_work_item_with_options(
        root,
        "WI-OVERLAPPING",
        "bounded source work",
        "keep lost scope blocked",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["scope remains blocked".into()],
            ..Default::default()
        },
    );
    let error =
        result.expect_err("foreign content must not hide a canonical registration obligation");
    assert!(
        error
            .to_string()
            .contains("archived_work_item_scope_untrusted"),
        "{error}"
    );
}

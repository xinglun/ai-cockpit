use cockpit_core::Digest;
use cockpit_git::GitRepository;
use cockpit_protocol::RuntimeContext;
use cockpit_repository::{
    WorkItemStartOptions, amend_work_item_contract, attach, checkpoint_work_item,
    preflight_work_item, require_verification_preconditions, scaffold_work_item,
    start_work_item_with_options, status,
};
use serde_json::json;
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
    let bytes =
        br#"{"schemaVersion":1,"workItemId":"WI-OLD","state":"archived","closeRequired":true}
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
fn scenario_coverage_can_be_declared_before_the_first_checkpoint() {
    let directory = repository();
    start_work_item_with_options(
        directory.path(),
        "WI-SCENARIO-DECLARATION",
        "declare high-risk scenario coverage",
        "make the preflight boundary explicit",
        &["src/**".into()],
        &WorkItemStartOptions {
            risk: "high".into(),
            ..start_options()
        },
    )
    .expect("start");

    amend_work_item_contract(
        directory.path(),
        "WI-SCENARIO-DECLARATION",
        &json!({
            "scenarioCoverageAppend": [{
                "scenario": "preflight",
                "required": true,
                "status": "unverified",
                "evidence": [],
                "expected": "preflight stops before expensive verification",
                "verificationPlan": "run the preflight regression"
            }]
        }),
        "declare the required high-risk scenario before checkpoint",
    )
    .expect("scenario declaration");

    let contract: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/active/WI-SCENARIO-DECLARATION.contract.json"),
        )
        .expect("contract"),
    )
    .expect("contract JSON");
    assert_eq!(contract["scenarioCoverage"][0]["scenario"], "preflight");
}

#[test]
fn verification_preconditions_reject_missing_governance_controls_before_execution() {
    let directory = repository();
    start_work_item_with_options(
        directory.path(),
        "WI-VERIFY-PRECONDITIONS",
        "check cheap verification gates first",
        "reject missing governance controls before the project command",
        &["src/**".into()],
        &WorkItemStartOptions {
            acceptance_criteria: vec!["A: bounded review remains explicit".into()],
            ..start_options()
        },
    )
    .expect("start");
    let contract = directory
        .path()
        .join(".ai/work-items/active/WI-VERIFY-PRECONDITIONS.contract.json");
    preflight_work_item(directory.path(), &contract).expect("preflight");
    checkpoint_work_item(directory.path(), "WI-VERIFY-PRECONDITIONS").expect("checkpoint");
    let snapshot = GitRepository::discover(directory.path())
        .expect("git repository")
        .snapshot()
        .expect("snapshot");
    let runtime = RuntimeContext {
        runtime_version: "test-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"test-runtime"),
    };
    let error = require_verification_preconditions(
        directory.path(),
        "WI-VERIFY-PRECONDITIONS",
        &runtime,
        &snapshot,
    )
    .expect_err("missing governance controls must stop before execution");
    assert!(
        error
            .to_string()
            .contains("verification preconditions are blocked")
    );
    assert!(error.to_string().contains("acceptance_evidence_missing"));
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
fn recovery_scaffold_may_activate_on_its_existing_ahead_branch() {
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
    run(directory.path(), &["branch", "-M", "main"]);
    let base = output(directory.path(), &["rev-parse", "HEAD"])
        .trim()
        .to_owned();
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
    run(directory.path(), &["checkout", "-qb", "recovery"]);
    scaffold_work_item(directory.path(), "WI-RECOVERY", "implementation")
        .expect("recovery scaffold");
    let contract_path = directory
        .path()
        .join(".ai/work-items/active/WI-RECOVERY.contract.json");
    let mut contract: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract")).expect("json");
    contract["predecessorWorkItemId"] = serde_json::json!("WI-PREDECESSOR");
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract).expect("serialize contract"),
    )
    .expect("write recovery binding");
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
            "reserve recovery continuation",
        ],
    );

    start_work_item_with_options(
        directory.path(),
        "WI-RECOVERY",
        "continue the recovery",
        "activate a bounded recovery continuation",
        &["src/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["recovery remains explicitly bounded".into()],
            ..start_options()
        },
    )
    .expect("recovery continuation should bypass only the ordinary base check");
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

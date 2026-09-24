use cockpit_core::Digest;
use cockpit_git::GitRepository;
use cockpit_protocol::{
    COLLABORATION_CAPABILITY, CollaborationDeclaration, IntegrationResponsibility,
    RuntimeCapabilityBinding, WorktreeRegistration,
};
use serde_json::Value;
use sha2::{Digest as ShaDigest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use cockpit_repository::{
    WorkItemStartOptions, attach, repository_id, start_work_item_with_options,
};

fn run_git(root: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .expect("git command")
            .success()
    );
}

fn repository() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("tempdir");
    run_git(root.path(), &["init", "-q"]);
    run_git(
        root.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    run_git(root.path(), &["config", "user.name", "Test"]);
    fs::write(root.path().join("README.md"), "initial\n").expect("write");
    run_git(root.path(), &["add", "."]);
    run_git(root.path(), &["commit", "-qm", "initial"]);
    root
}

fn candidate_runtime() -> RuntimeCapabilityBinding {
    let executable = PathBuf::from(env!("CARGO_BIN_EXE_ai-cockpit"));
    let bytes = fs::read(executable).expect("read candidate executable");
    let digest = format!("sha256:{:x}", Sha256::digest(bytes))
        .parse::<Digest>()
        .expect("runtime digest");
    RuntimeCapabilityBinding {
        schema_version: 1,
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        runtime_digest: digest,
        capability: COLLABORATION_CAPABILITY.into(),
    }
}

fn registration(root: &Path) -> WorktreeRegistration {
    let topology = GitRepository::discover(root)
        .expect("discover repository")
        .topology()
        .expect("repository topology");
    let contract_path = root.join(".ai/work-items/active/WI-CLI.contract.json");
    let contract: Value = serde_json::from_slice(&fs::read(&contract_path).expect("contract"))
        .expect("contract JSON");
    WorktreeRegistration {
        schema_version: 1,
        repository_id: repository_id(root),
        work_item_id: "WI-CLI".into(),
        contract_digest: cockpit_protocol::digest_json(&contract).expect("contract digest"),
        worktree_path: topology.repository_root.to_string_lossy().into_owned(),
        branch: topology.branch.expect("branch"),
        head: topology.head.expect("head"),
        generation: 1,
        declaration: CollaborationDeclaration {
            integration_responsibility: IntegrationResponsibility {
                responsible_work_item_id: "WI-CLI".into(),
                target_branch: "main".into(),
                composition_order: vec!["WI-CLI".into()],
                rationale: "CLI test".into(),
            },
            ..Default::default()
        },
        runtime: candidate_runtime(),
    }
}

fn invoke(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(args)
        .output()
        .expect("candidate CLI")
}

#[test]
fn coordination_cli_separates_read_only_inspection_from_writes() {
    let root = repository();
    run_git(root.path(), &["checkout", "-qb", "codex/wi-cli"]);
    attach(root.path()).expect("attach repository");
    start_work_item_with_options(
        root.path(),
        "WI-CLI",
        "CLI coordination test",
        "bind registration to observed facts",
        &[".ai/**".into(), "README.md".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["registration is fact-bound".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start Work Item");
    let output = invoke(&[
        "work-item",
        "coordination",
        "inspect",
        "--repo",
        root.path().to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!root.path().join(".git/.ai-cockpit/coordination").exists());
    let first: Value = serde_json::from_slice(&output.stdout).expect("projection JSON");

    let input = root.path().join("registration.json");
    fs::write(
        &input,
        serde_json::to_vec_pretty(&registration(root.path())).unwrap(),
    )
    .unwrap();
    let output = invoke(&[
        "work-item",
        "coordination",
        "register",
        "--repo",
        root.path().to_str().unwrap(),
        "--input",
        input.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let written: Value = serde_json::from_slice(&output.stdout).expect("write result JSON");
    assert!(written.get("projection").is_some());

    let output = invoke(&[
        "work-item",
        "coordination",
        "inspect",
        "--repo",
        root.path().to_str().unwrap(),
    ]);
    assert!(output.status.success());
    let second: Value = serde_json::from_slice(&output.stdout).expect("projection JSON");
    assert_eq!(first["events"], second["events"]);
    assert_eq!(second["registrations"].as_array().unwrap().len(), 1);
}

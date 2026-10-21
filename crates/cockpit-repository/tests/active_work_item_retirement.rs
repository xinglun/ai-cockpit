use cockpit_core::Digest;
use cockpit_protocol::{ActiveWorkItemRetirementRequest, RuntimeContext};
use cockpit_repository::{
    WorkItemStartOptions, attach, retire_active_work_item_with_runtime,
    start_work_item_with_options,
};
use std::{fs, path::Path, process::Command};
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

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("repository");
    run(directory.path(), &["init", "-q"]);
    attach(directory.path()).expect("attach");
    directory
}

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "test-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"test-runtime"),
    }
}

fn start_options() -> WorkItemStartOptions {
    WorkItemStartOptions {
        authority: "authorized".into(),
        acceptance_criteria: vec!["retirement remains explicit".into()],
        ..Default::default()
    }
}

fn request(disposition: &str) -> ActiveWorkItemRetirementRequest {
    ActiveWorkItemRetirementRequest {
        schema_version: 1,
        decision_id: "work-item-retirement".into(),
        disposition: disposition.into(),
        successor_work_item_id: None,
        actor: "human:test".into(),
        authority_source: "explicit-user-authorization".into(),
        reason: "delivery is already represented on the synchronized base".into(),
        repository_id: None,
        contract_digest: None,
        summary_digest: None,
        repository_snapshot_digest: None,
    }
}
#[test]
fn retirement_public_factory_never_treats_a_caller_claim_as_approval() {
    let directory = repository();
    let root = directory.path();
    let id = "WI-RETIRE-PUBLIC";
    start_work_item_with_options(
        root,
        id,
        "bounded public refusal",
        "preserve active bytes",
        &["src/**".into()],
        &start_options(),
    )
    .unwrap();
    let contract = root.join(format!(".ai/work-items/active/{id}.contract.json"));
    let summary = root.join(format!(".ai/work-items/active/{id}.summary.json"));
    let before = (fs::read(&contract).unwrap(), fs::read(&summary).unwrap());
    let error = retire_active_work_item_with_runtime(root, id, &request("integrated"), &runtime())
        .unwrap_err();
    assert!(
        error.to_string().contains("authority_unavailable"),
        "{error}"
    );
    assert_eq!(
        (fs::read(&contract).unwrap(), fs::read(&summary).unwrap()),
        before
    );
    assert!(
        !root
            .join(format!(".ai/work-items/archive/{id}.archive.json"))
            .exists()
    );
}

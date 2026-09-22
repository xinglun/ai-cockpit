use std::fs;

use cockpit_protocol::ResourceFinalizationContext;
use cockpit_repository::{attach, plan_resource_finalization, start_work_item};
use std::process::Command;

#[test]
fn resource_lifecycle_module_uses_explicit_imports() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/resource_lifecycle.rs");
    let source = fs::read_to_string(&path).expect("resource lifecycle module exists");
    assert!(!source.contains("use super::*"));
}

#[test]
fn planning_preserves_the_public_resource_context_binding() {
    let directory = tempfile::tempdir().expect("tempdir");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    attach(directory.path()).expect("attach");
    let work_item_id = "WI-RESOURCE-BOUNDARY-PLAN";
    start_work_item(
        directory.path(),
        work_item_id,
        "resource lifecycle boundary",
        "preserve resource identity",
        &[".ai/**".into()],
    )
    .expect("start");
    let context = ResourceFinalizationContext {
        branch: "feature/resource-boundary".into(),
        worktree: directory.path().display().to_string(),
        base_branch: "main".into(),
        base_remote: "origin".into(),
        provider: "github".into(),
        pull_request: "https://github.com/example/ai-cockpit/pull/985".into(),
    };
    plan_resource_finalization(directory.path(), work_item_id, &context).expect("plan");
    let contract: serde_json::Value = serde_json::from_slice(
        &fs::read(directory.path().join(format!(
            ".ai/work-items/active/{work_item_id}.contract.json"
        )))
        .expect("contract"),
    )
    .expect("contract JSON");
    assert_eq!(
        contract["resourceContext"],
        serde_json::to_value(context).unwrap()
    );
}

#[test]
fn planning_rejects_replacing_a_complete_resource_binding() {
    let directory = tempfile::tempdir().expect("tempdir");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    attach(directory.path()).expect("attach");
    let work_item_id = "WI-RESOURCE-BOUNDARY-REPLAY";
    start_work_item(
        directory.path(),
        work_item_id,
        "resource lifecycle boundary",
        "reject binding replacement",
        &[".ai/**".into()],
    )
    .expect("start");
    let first = ResourceFinalizationContext {
        branch: "feature/first".into(),
        worktree: directory.path().display().to_string(),
        base_branch: "main".into(),
        base_remote: "origin".into(),
        provider: "github".into(),
        pull_request: "https://github.com/example/ai-cockpit/pull/985".into(),
    };
    plan_resource_finalization(directory.path(), work_item_id, &first).expect("first plan");
    let mut replacement = first.clone();
    replacement.branch = "feature/replacement".into();
    let error = plan_resource_finalization(directory.path(), work_item_id, &replacement)
        .expect_err("complete resource context must not be replaced");
    assert!(error.to_string().contains("already bound"));
}

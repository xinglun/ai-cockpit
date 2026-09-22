use std::fs;

use cockpit_core::Digest;
use cockpit_protocol::{ResourceFinalizationContext, RuntimeContext};
use cockpit_repository::{
    archive_work_item, attach, checkpoint_work_item, finish_work_item, plan_resource_finalization,
    plan_resource_finalization_with_runtime, preflight_work_item, record_verification,
    start_work_item, work_item_status_snapshot_with_runtime,
};
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

#[test]
fn archived_finalize_plan_appends_a_resource_binding_without_mutating_archive() {
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
    let work_item_id = "WI-ARCHIVED-RESOURCE-BINDING";
    start_work_item(
        directory.path(),
        work_item_id,
        "bind an archived PR resource",
        "preserve the immutable archive while completing the PR handoff",
        &[".ai/**".into()],
    )
    .expect("start");
    let contract_path = directory.path().join(format!(
        ".ai/work-items/active/{work_item_id}.contract.json"
    ));
    preflight_work_item(directory.path(), &contract_path).expect("preflight");
    checkpoint_work_item(directory.path(), work_item_id).expect("checkpoint");
    record_verification(
        directory.path(),
        work_item_id,
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.105",
        &Digest::sha256_bytes(b"test-runtime"),
    )
    .expect("verification");
    finish_work_item(directory.path(), work_item_id).expect("finish");
    archive_work_item(directory.path(), work_item_id).expect("archive");

    let archived_contract = directory.path().join(format!(
        ".ai/work-items/archive/{work_item_id}.contract.json"
    ));
    let archive_manifest = directory.path().join(format!(
        ".ai/work-items/archive/{work_item_id}.archive.json"
    ));
    let contract_before = fs::read(&archived_contract).expect("archived contract");
    let manifest_before = fs::read(&archive_manifest).expect("archive manifest");
    let context = ResourceFinalizationContext {
        branch: "codex/archived-resource-binding".into(),
        worktree: directory.path().display().to_string(),
        base_branch: "main".into(),
        base_remote: "origin".into(),
        provider: "github".into(),
        pull_request: "https://github.com/xinglun/ai-cockpit/pull/967".into(),
    };
    let runtime = RuntimeContext {
        runtime_version: "0.2.105".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"test-runtime"),
    };

    let first =
        plan_resource_finalization_with_runtime(directory.path(), work_item_id, &context, &runtime)
            .expect("append archived resource binding");
    assert_eq!(first["state"], "recorded");
    assert_eq!(
        fs::read(&archived_contract).expect("contract after"),
        contract_before
    );
    assert_eq!(
        fs::read(&archive_manifest).expect("manifest after"),
        manifest_before
    );
    let binding_before = fs::read(directory.path().join(format!(
        ".ai/decisions/{work_item_id}.resource-context.json"
    )))
    .expect("binding after first plan");

    let second =
        plan_resource_finalization_with_runtime(directory.path(), work_item_id, &context, &runtime)
            .expect("replay archived resource binding");
    assert_eq!(second["state"], "idempotent");
    assert_eq!(
        binding_before,
        fs::read(directory.path().join(format!(
            ".ai/decisions/{work_item_id}.resource-context.json"
        )),)
        .expect("binding record replay"),
    );
    let status = work_item_status_snapshot_with_runtime(directory.path(), work_item_id, &runtime)
        .expect("status uses the appended binding");
    assert!(
        status
            .safe_actions
            .iter()
            .any(|action| action == "finalize_resources")
    );
    assert!(
        !status
            .safe_actions
            .iter()
            .any(|action| action == "close_after_review")
    );
}

use cockpit_core::Digest;
use cockpit_protocol::{ActiveWorkItemRetirementRequest, RuntimeContext};
use cockpit_repository::{
    WorkItemStartOptions, attach, outcome_v2_with_runtime, retire_active_work_item_with_runtime,
    start_work_item_with_options, status_with_runtime,
};
use serde_json::json;
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
fn integrated_retirement_preserves_bytes_without_claiming_verification() {
    let directory = repository();
    let id = "WI-RETIRE-INTEGRATED";
    start_work_item_with_options(
        directory.path(),
        id,
        "retire an already integrated work item",
        "remove stale active state without claiming verification",
        &["src/**".into()],
        &start_options(),
    )
    .expect("start");
    let active_contract = directory
        .path()
        .join(format!(".ai/work-items/active/{id}.contract.json"));
    let original_contract = fs::read(&active_contract).expect("active contract");

    let receipt = retire_active_work_item_with_runtime(
        directory.path(),
        id,
        &request("integrated"),
        &runtime(),
    )
    .expect("retire");

    assert_eq!(receipt["disposition"], json!("integrated"));
    assert_eq!(receipt["verificationClaim"], json!("not_verified"));
    assert!(!active_contract.exists());
    let archived_contract = directory
        .path()
        .join(format!(".ai/work-items/archive/{id}.contract.json"));
    assert_eq!(
        fs::read(archived_contract).expect("archive"),
        original_contract
    );
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(format!(".ai/work-items/archive/{id}.archive.json")),
        )
        .expect("manifest"),
    )
    .expect("manifest JSON");
    assert_eq!(manifest["state"], json!("retired"));
    assert_eq!(manifest["retirementDisposition"], json!("integrated"));
    let outcome = outcome_v2_with_runtime(directory.path(), id, &runtime()).expect("outcome");
    assert_eq!(outcome.historical_status.as_deref(), Some("retired"));
    assert_eq!(
        outcome.decision_state,
        Some(cockpit_core::DecisionState::Yellow)
    );
    let repository_status =
        status_with_runtime(directory.path(), Some(&runtime())).expect("status");
    assert!(
        !repository_status
            .readiness
            .blockers
            .iter()
            .any(|reason| reason == "active_work_items_present")
    );
}

#[test]
fn abandoned_retirement_preserves_bytes_without_a_success_claim() {
    let directory = repository();
    let id = "WI-RETIRE-ABANDONED";
    start_work_item_with_options(
        directory.path(),
        id,
        "retire a displaced active work item",
        "preserve a candidate that is no longer the selected delivery path",
        &["src/**".into()],
        &start_options(),
    )
    .expect("start");
    let active_contract = directory
        .path()
        .join(format!(".ai/work-items/active/{id}.contract.json"));
    let original_contract = fs::read(&active_contract).expect("active contract");

    let mut abandoned = request("abandoned");
    abandoned.reason = "the explicitly authorized delivery path was displaced before this active Work Item could be verified".into();
    let receipt =
        retire_active_work_item_with_runtime(directory.path(), id, &abandoned, &runtime())
            .expect("abandon");

    assert_eq!(receipt["disposition"], json!("abandoned"));
    assert_eq!(receipt["successorWorkItemId"], serde_json::Value::Null);
    assert_eq!(receipt["verificationClaim"], json!("not_verified"));
    assert!(!active_contract.exists());
    assert_eq!(
        fs::read(
            directory
                .path()
                .join(format!(".ai/work-items/archive/{id}.contract.json")),
        )
        .expect("archived contract"),
        original_contract
    );
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(format!(".ai/work-items/archive/{id}.archive.json")),
        )
        .expect("manifest"),
    )
    .expect("manifest JSON");
    assert_eq!(manifest["state"], json!("retired"));
    assert_eq!(manifest["retirementDisposition"], json!("abandoned"));
}

#[test]
fn abandoned_retirement_requires_an_explicit_human_actor() {
    let directory = repository();
    let id = "WI-RETIRE-ABANDONED-AUTHORITY";
    start_work_item_with_options(
        directory.path(),
        id,
        "reject an unowned abandonment",
        "an agent cannot silently discard an active Work Item",
        &["src/**".into()],
        &start_options(),
    )
    .expect("start");
    let mut abandoned = request("abandoned");
    abandoned.actor = "agent:test".into();

    let error = retire_active_work_item_with_runtime(directory.path(), id, &abandoned, &runtime())
        .expect_err("abandonment without a human actor must fail");
    assert!(
        error.to_string().contains("explicit human actor"),
        "{error}"
    );
    assert!(
        directory
            .path()
            .join(format!(".ai/work-items/active/{id}.contract.json"))
            .exists()
    );
}

#[test]
fn stale_binding_is_rejected_before_any_archive_write() {
    let directory = repository();
    let id = "WI-RETIRE-STALE";
    start_work_item_with_options(
        directory.path(),
        id,
        "reject stale retirement input",
        "do not partially retire a stale active item",
        &["src/**".into()],
        &start_options(),
    )
    .expect("start");
    let active = directory.path().join(".ai/work-items/active");
    let before = fs::read_dir(&active).expect("active directory").count();
    let mut stale = request("integrated");
    stale.repository_id = Some("sha256:foreign".into());

    let error = retire_active_work_item_with_runtime(directory.path(), id, &stale, &runtime())
        .expect_err("foreign binding must fail closed");
    assert!(error.to_string().contains("repository identity"));
    assert_eq!(
        fs::read_dir(&active).expect("active directory").count(),
        before
    );
    assert!(
        !directory
            .path()
            .join(format!(".ai/work-items/archive/{id}.archive.json"))
            .exists()
    );
}

#[test]
fn replacement_requires_an_existing_explicit_successor() {
    let directory = repository();
    let id = "WI-RETIRE-REPLACED";
    start_work_item_with_options(
        directory.path(),
        id,
        "require a real replacement",
        "do not silently create a successor while retiring",
        &["src/**".into()],
        &start_options(),
    )
    .expect("start");
    let mut replacement = request("replaced");
    replacement.successor_work_item_id = Some("WI-REPLACEMENT".into());

    let error =
        retire_active_work_item_with_runtime(directory.path(), id, &replacement, &runtime())
            .expect_err("missing successor must fail closed");
    assert!(error.to_string().contains("successor"));
    assert!(
        directory
            .path()
            .join(format!(".ai/work-items/active/{id}.contract.json"))
            .exists()
    );
}

#[test]
fn replacement_rejects_duplicate_active_and_archived_successors() {
    let directory = repository();
    let id = "WI-RETIRE-DUPLICATE";
    let successor_id = "WI-RETIRE-SUCCESSOR";
    start_work_item_with_options(
        directory.path(),
        id,
        "reject duplicate successors",
        "do not choose one successor when active and archived copies exist",
        &["src/**".into()],
        &start_options(),
    )
    .expect("start predecessor");
    start_work_item_with_options(
        directory.path(),
        successor_id,
        "successor",
        "successor is explicitly linked",
        &["src/**".into()],
        &start_options(),
    )
    .expect("start successor");
    let successor_active = directory.path().join(format!(
        ".ai/work-items/active/{successor_id}.contract.json"
    ));
    let mut successor: serde_json::Value =
        serde_json::from_slice(&fs::read(&successor_active).expect("successor contract"))
            .expect("successor JSON");
    successor["predecessorWorkItemId"] = json!(id);
    fs::write(
        &successor_active,
        serde_json::to_vec_pretty(&successor).expect("successor bytes"),
    )
    .expect("rewrite successor");
    let successor_archive = directory.path().join(format!(
        ".ai/work-items/archive/{successor_id}.contract.json"
    ));
    fs::create_dir_all(successor_archive.parent().expect("archive parent"))
        .expect("archive directory");
    fs::copy(&successor_active, &successor_archive).expect("duplicate successor");

    let mut replacement = request("replaced");
    replacement.successor_work_item_id = Some(successor_id.into());
    let error =
        retire_active_work_item_with_runtime(directory.path(), id, &replacement, &runtime())
            .expect_err("duplicate successor must fail closed");
    assert!(error.to_string().contains("duplicate"));
    assert!(
        directory
            .path()
            .join(format!(".ai/work-items/active/{id}.contract.json"))
            .exists()
    );
    assert!(
        !directory
            .path()
            .join(format!(".ai/work-items/archive/{id}.archive.json"))
            .exists()
    );
}

#[cfg(unix)]
#[test]
fn replacement_rejects_a_symlinked_successor_contract() {
    use std::os::unix::fs::symlink;

    let directory = repository();
    let id = "WI-RETIRE-SYMLINK";
    let successor_id = "WI-RETIRE-SYMLINK-SUCCESSOR";
    start_work_item_with_options(
        directory.path(),
        id,
        "reject symlink successor",
        "do not follow a successor Contract outside the repository",
        &["src/**".into()],
        &start_options(),
    )
    .expect("start predecessor");
    let successor_path = directory.path().join(format!(
        ".ai/work-items/active/{successor_id}.contract.json"
    ));
    symlink("/tmp/foreign-successor-contract.json", &successor_path).expect("symlink successor");
    let mut replacement = request("replaced");
    replacement.successor_work_item_id = Some(successor_id.into());

    let error =
        retire_active_work_item_with_runtime(directory.path(), id, &replacement, &runtime())
            .expect_err("symlink successor must fail closed");
    assert!(error.to_string().contains("regular non-symlink"));
    assert!(
        directory
            .path()
            .join(format!(".ai/work-items/active/{id}.contract.json"))
            .exists()
    );
}

#[test]
fn retirement_does_not_move_another_work_items_variant_artifact() {
    let directory = repository();
    let retired_id = "WI-RETIRE-VARIANT-A";
    let retained_id = "WI-RETIRE-VARIANT-B";
    for id in [retired_id, retained_id] {
        start_work_item_with_options(
            directory.path(),
            id,
            "retire an explicitly selected Work Item",
            "preserve unrelated active evidence",
            &["src/**".into()],
            &start_options(),
        )
        .expect("start");
    }
    let unrelated_variant = directory.path().join(format!(
        ".ai/work-items/active/{retained_id}.outcome.attempt.json"
    ));
    fs::write(&unrelated_variant, br#"{"state":"failed"}"#).expect("variant artifact");

    retire_active_work_item_with_runtime(
        directory.path(),
        retired_id,
        &request("integrated"),
        &runtime(),
    )
    .expect("retire selected Work Item");

    assert!(unrelated_variant.is_file());
    assert!(
        !directory
            .path()
            .join(format!(
                ".ai/work-items/archive/{retired_id}.outcome.attempt.json"
            ))
            .exists()
    );
}

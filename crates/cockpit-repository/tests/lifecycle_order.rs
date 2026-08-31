use cockpit_core::{DecisionState, Digest};
use cockpit_protocol::ResourceFinalizationContext;
use cockpit_repository::{
    WorkItemStartOptions, attach, checkpoint_work_item, finish_work_item,
    plan_resource_finalization, preflight_work_item, record_verification,
    revalidate_contract_amendment, start_work_item_with_options,
};
use std::{fs, process::Command};

fn repository() -> tempfile::TempDir {
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
    directory
}

fn start(path: &std::path::Path, id: &str, required: &[&str]) {
    start_work_item_with_options(
        path,
        id,
        "lifecycle ordering",
        "preserve serial governance",
        &["**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            out_of_scope: vec!["target/**".into()],
            acceptance_criteria: vec!["lifecycle remains bounded".into()],
            required_evidence_classes: required.iter().map(|value| (*value).into()).collect(),
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
}

fn contract(path: &std::path::Path, id: &str) -> std::path::PathBuf {
    path.join(".ai/work-items/active")
        .join(format!("{id}.contract.json"))
}

#[test]
fn skipped_preflight_and_checkpoint_fail_closed() {
    let directory = repository();
    start(directory.path(), "WI-ORDER-SKIP", &[]);

    let verify = record_verification(
        directory.path(),
        "WI-ORDER-SKIP",
        &serde_json::json!({"passed": true}),
        "0.2.8",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect_err("verification before checkpoint must fail closed");
    assert!(verify.to_string().contains("checkpoint"));
    let finish = finish_work_item(directory.path(), "WI-ORDER-SKIP")
        .expect_err("finish without preflight/checkpoint must fail closed");
    assert!(finish.to_string().contains("state") || finish.to_string().contains("checkpoint"));
    assert!(
        directory
            .path()
            .join(".ai/work-items/active/WI-ORDER-SKIP.summary.json")
            .is_file()
    );
}

#[test]
fn checkpoint_requires_preflight_and_duplicate_checkpoint_is_rejected() {
    let directory = repository();
    start(directory.path(), "WI-ORDER-DUPLICATE", &[]);
    let summary = directory
        .path()
        .join(".ai/work-items/active/WI-ORDER-DUPLICATE.summary.json");

    let before_preflight = checkpoint_work_item(directory.path(), "WI-ORDER-DUPLICATE")
        .expect_err("checkpoint without preflight must fail closed");
    assert!(before_preflight.to_string().contains("preflight"));

    let decision = preflight_work_item(
        directory.path(),
        &contract(directory.path(), "WI-ORDER-DUPLICATE"),
    )
    .expect("preflight");
    assert_eq!(decision.state, DecisionState::Green);
    checkpoint_work_item(directory.path(), "WI-ORDER-DUPLICATE").expect("checkpoint");
    let duplicate = checkpoint_work_item(directory.path(), "WI-ORDER-DUPLICATE")
        .expect_err("duplicate checkpoint must fail closed");
    assert!(duplicate.to_string().contains("duplicate"));
    let stored: serde_json::Value =
        serde_json::from_slice(&fs::read(summary).expect("summary")).expect("summary JSON");
    assert_eq!(stored["checkpointCount"], 1);
    assert_eq!(stored["state"], "checkpointed");
}

#[test]
fn before_edit_checkpoint_rejects_existing_verification_results() {
    let directory = repository();
    let id = "WI-ORDER-BEFORE-VERIFY";
    start(directory.path(), id, &[]);
    let contract_path = contract(directory.path(), id);
    let mut contract_value: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract"))
            .expect("contract JSON");
    contract_value["checkpointPolicy"] = serde_json::json!({
        "schemaVersion": 1,
        "profile": "strict",
        "requiredBeforeFinish": true,
        "requiredStages": ["before_edit", "before_finish"],
        "requiredChecks": []
    });
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract_value).expect("contract bytes"),
    )
    .expect("contract amendment");
    preflight_work_item(directory.path(), &contract_path).expect("preflight");
    let summary_path = directory
        .path()
        .join(".ai/work-items/active")
        .join(format!("{id}.summary.json"));
    let mut summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("summary")).expect("summary JSON");
    summary["verification"] = serde_json::json!([
        {"check": "quality", "result": "passed"}
    ]);
    fs::write(
        &summary_path,
        serde_json::to_vec_pretty(&summary).expect("summary bytes"),
    )
    .expect("summary amendment");
    let error = checkpoint_work_item(directory.path(), id)
        .expect_err("before_edit checkpoint after verification must fail closed");
    assert!(
        error
            .to_string()
            .contains("before_edit checkpoint must be recorded before required verification"),
        "{error}"
    );
}

#[test]
fn verification_promotes_initial_yellow_preflight_and_allows_recovery() {
    let directory = repository();
    start(directory.path(), "WI-ORDER-RECOVER", &["verification"]);
    plan_resource_finalization(
        directory.path(),
        "WI-ORDER-RECOVER",
        &ResourceFinalizationContext {
            branch: "feature/WI-ORDER-RECOVER".into(),
            worktree: directory.path().display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: "https://github.com/example/ai-cockpit/pull/WI-ORDER-RECOVER".into(),
        },
    )
    .expect("finalization plan");
    let decision = preflight_work_item(
        directory.path(),
        &contract(directory.path(), "WI-ORDER-RECOVER"),
    )
    .expect("preflight");
    assert_eq!(decision.state, DecisionState::Yellow);
    checkpoint_work_item(directory.path(), "WI-ORDER-RECOVER").expect("checkpoint");

    let missing = finish_work_item(directory.path(), "WI-ORDER-RECOVER")
        .expect_err("finish before verification must preserve recovery state");
    assert!(
        missing.to_string().contains("preflight") || missing.to_string().contains("verification")
    );

    record_verification(
        directory.path(),
        "WI-ORDER-RECOVER",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.8",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    let summary: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/active/WI-ORDER-RECOVER.summary.json"),
        )
        .expect("summary"),
    )
    .expect("summary JSON");
    assert_eq!(summary["preflightState"], "green");
    finish_work_item(directory.path(), "WI-ORDER-RECOVER").expect("finish after recovery");
}

#[test]
fn finalize_plan_after_finish_ready_is_rejected_before_it_can_invalidate_evidence() {
    let directory = repository();
    let id = "WI-ORDER-FINALIZE-TOO-LATE";
    start(directory.path(), id, &["verification"]);
    let contract_path = contract(directory.path(), id);
    preflight_work_item(directory.path(), &contract_path).expect("preflight");
    checkpoint_work_item(directory.path(), id).expect("checkpoint");
    let summary_path = directory
        .path()
        .join(format!(".ai/work-items/active/{id}.summary.json"));
    let mut summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("summary")).expect("summary JSON");
    summary["state"] = serde_json::json!("finish_ready");
    fs::write(
        &summary_path,
        serde_json::to_vec_pretty(&summary).expect("summary bytes"),
    )
    .expect("finish-ready fixture");

    let error = plan_resource_finalization(
        directory.path(),
        id,
        &ResourceFinalizationContext {
            branch: format!("feature/{id}"),
            worktree: directory.path().display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: format!("https://github.com/example/ai-cockpit/pull/{id}"),
        },
    )
    .expect_err("finalize-plan after finish_ready must fail closed");
    assert!(error.to_string().contains("before verification"), "{error}");
}

#[test]
fn verification_retry_refreshes_finish_ready_bindings_after_source_change() {
    let directory = repository();
    let id = "WI-ORDER-REVERIFY";
    start(directory.path(), id, &[]);
    let contract_path = contract(directory.path(), id);
    plan_resource_finalization(
        directory.path(),
        id,
        &ResourceFinalizationContext {
            branch: format!("feature/{id}"),
            worktree: directory.path().display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: format!("https://github.com/example/ai-cockpit/pull/{id}"),
        },
    )
    .expect("finalization plan");
    preflight_work_item(directory.path(), &contract_path).expect("preflight");
    checkpoint_work_item(directory.path(), id).expect("checkpoint");
    record_verification(
        directory.path(),
        id,
        &serde_json::json!({"passed": true}),
        "0.2.8",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("initial verification");
    finish_work_item(directory.path(), id).expect("finish");

    fs::write(directory.path().join("source.rs"), "pub fn changed() {}\n").expect("source");
    let evidence = record_verification(
        directory.path(),
        id,
        &serde_json::json!({"passed": true}),
        "0.2.8",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification retry");
    let evidence_digest = cockpit_protocol::digest_json(&evidence).expect("evidence digest");
    let outcome: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(format!(".ai/work-items/active/{id}.outcome.json")),
        )
        .expect("outcome"),
    )
    .expect("outcome JSON");
    assert_eq!(outcome["evidenceDigest"], evidence_digest.to_string());
    assert_eq!(
        outcome["taskOutcomeReport"]["bindings"]["repositorySnapshotDigest"],
        evidence["repositorySnapshotDigest"]
    );
}

#[test]
fn contract_amendment_after_finish_ready_reopens_checkpointed_recovery() {
    let directory = repository();
    let id = "WI-ORDER-AMEND-FINISH-READY";
    start(directory.path(), id, &[]);
    let contract_path = contract(directory.path(), id);
    plan_resource_finalization(
        directory.path(),
        id,
        &ResourceFinalizationContext {
            branch: format!("feature/{id}"),
            worktree: directory.path().display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: format!("https://github.com/example/ai-cockpit/pull/{id}"),
        },
    )
    .expect("finalization plan");
    preflight_work_item(directory.path(), &contract_path).expect("preflight");
    checkpoint_work_item(directory.path(), id).expect("checkpoint");
    record_verification(
        directory.path(),
        id,
        &serde_json::json!({"passed": true}),
        "0.2.33",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(directory.path(), id).expect("finish");

    let mut contract_value: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract"))
            .expect("contract JSON");
    contract_value["title"] = serde_json::json!("amended after finish-ready");
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract_value).expect("serialize contract"),
    )
    .expect("contract amendment");
    revalidate_contract_amendment(
        directory.path(),
        id,
        "record a post-finish amendment and reopen the verification cycle",
    )
    .expect("amendment revalidation");

    let summary_path = directory
        .path()
        .join(".ai/work-items/active")
        .join(format!("{id}.summary.json"));
    let summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("summary")).expect("summary JSON");
    assert_eq!(summary["state"], "checkpointed");
    assert_eq!(summary["preflightState"], "not_run");
    assert!(summary["recoveryRetryPending"].is_null());
}

#[test]
fn before_edit_checkpoint_survives_authorized_edit_and_fresh_preflight() {
    let directory = repository();
    let id = "WI-ORDER-SNAPSHOT-HISTORY";
    start(directory.path(), id, &[]);
    let contract_path = contract(directory.path(), id);
    let mut contract_value: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract"))
            .expect("contract JSON");
    contract_value["checkpointPolicy"] = serde_json::json!({
        "schemaVersion": 1,
        "profile": "standard",
        "requiredBeforeFinish": true,
        "requiredStages": ["before_edit", "before_finish"],
        "requiredChecks": []
    });
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract_value).expect("serialize contract"),
    )
    .expect("contract amendment");
    plan_resource_finalization(
        directory.path(),
        id,
        &ResourceFinalizationContext {
            branch: format!("feature/{id}"),
            worktree: directory.path().display().to_string(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            provider: "github".into(),
            pull_request: format!("https://github.com/example/ai-cockpit/pull/{id}"),
        },
    )
    .expect("finalization plan");
    preflight_work_item(directory.path(), &contract_path).expect("initial preflight");
    checkpoint_work_item(directory.path(), id).expect("before_edit checkpoint");

    fs::write(directory.path().join("source.rs"), "pub fn changed() {}\n").expect("source edit");
    preflight_work_item(directory.path(), &contract_path).expect("fresh preflight");
    record_verification(
        directory.path(),
        id,
        &serde_json::json!({"passed": true}),
        "0.2.33",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");
    finish_work_item(directory.path(), id).expect("finish after fresh snapshot binding");
}

#[test]
fn legacy_command_only_amendment_after_verification_has_no_gate_to_invalidate() {
    let directory = repository();
    let id = "WI-ORDER-LEGACY-AMENDMENT";
    start(directory.path(), id, &[]);
    let contract_path = contract(directory.path(), id);
    let mut contract_value: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract"))
            .expect("contract JSON");
    contract_value["checkpointPolicy"] = serde_json::json!({
        "schemaVersion": 1,
        "profile": "standard",
        "requiredBeforeFinish": true,
        "requiredStages": ["before_edit", "before_finish"],
        "requiredChecks": []
    });
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract_value).expect("serialize contract"),
    )
    .expect("contract amendment");
    preflight_work_item(directory.path(), &contract_path).expect("preflight");
    checkpoint_work_item(directory.path(), id).expect("checkpoint");
    record_verification(
        directory.path(),
        id,
        &serde_json::json!({"passed": true}),
        "0.2.33",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");

    contract_value["title"] = serde_json::json!("amended after verification");
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&contract_value).expect("serialize amended contract"),
    )
    .expect("amended contract");
    let amendment = revalidate_contract_amendment(
        directory.path(),
        id,
        "record the post-verification legacy amendment",
    )
    .expect("legacy amendment with no required gates");
    assert_eq!(amendment["verificationStarted"], serde_json::json!(false));
    assert_eq!(
        amendment["invalidatedRequiredChecks"],
        serde_json::json!([])
    );
}

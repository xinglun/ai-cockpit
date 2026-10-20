use cockpit_core::{DecisionState, Digest};
use cockpit_protocol::{ResourceFinalizationContext, RuntimeContext};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_work_item, attach, checkpoint_work_item, finish_work_item, plan_resource_finalization,
    preflight_work_item, preflight_work_item_with_runtime, record_verification,
    record_verification_with_runtime, revalidate_contract_amendment, run_repository_verification,
    start_work_item_with_options, work_item_status_snapshot_with_runtime,
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
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=AI Cockpit Test",
                "-c",
                "user.email=ai-cockpit-test@example.invalid",
                "commit",
                "--allow-empty",
                "-m",
                "fixture baseline",
            ])
            .current_dir(directory.path())
            .status()
            .expect("git baseline commit")
            .success()
    );
    attach(directory.path()).expect("attach");
    directory
}

fn commit_fixture_source(path: &std::path::Path, source: &str, message: &str) {
    assert!(
        Command::new("git")
            .args(["add", "--", source])
            .current_dir(path)
            .status()
            .expect("git add fixture source")
            .success()
    );
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=AI Cockpit Test",
                "-c",
                "user.email=ai-cockpit-test@example.invalid",
                "commit",
                "-m",
                message,
            ])
            .current_dir(path)
            .status()
            .expect("git commit fixture source")
            .success()
    );
}

fn start(path: &std::path::Path, id: &str, required: &[&str]) {
    start_work_item_with_options(
        path,
        id,
        "lifecycle ordering",
        "preserve serial governance",
        &[".ai/**".into(), "src/**".into(), "source.rs".into()],
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

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "0.2.91-test".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"runtime-bound-lifecycle"),
    }
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

    fs::create_dir_all(directory.path().join("src")).expect("source directory");
    fs::write(directory.path().join("src/main.rs"), "fn main() {}\n").expect("source change");
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
    assert!(
        summary["changedPaths"]
            .as_array()
            .is_some_and(|paths| paths.iter().any(|path| path == "src/main.rs")),
        "verification input paths must be retained in the canonical Summary: {summary}"
    );
    let outcome: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/active/WI-ORDER-RECOVER.outcome.json"),
        )
        .expect("reconciled outcome"),
    )
    .expect("reconciled outcome JSON");
    assert_eq!(
        outcome["state"], "checkpointed",
        "fresh verification must replace the stale blocked projection"
    );
    assert_eq!(outcome["verification"]["status"], "verified");
    assert_eq!(outcome["decisionState"], "green");
    commit_fixture_source(directory.path(), "src/main.rs", "verified recovery source");
    let contract_path = contract(directory.path(), "WI-ORDER-RECOVER");
    preflight_work_item(directory.path(), &contract_path)
        .expect("refresh preflight for committed source");
    record_verification(
        directory.path(),
        "WI-ORDER-RECOVER",
        &serde_json::json!({"passed": true, "nodesPlanned": 1}),
        "0.2.8",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("record current verification");
    finish_work_item(directory.path(), "WI-ORDER-RECOVER").expect("finish after recovery");
    let finished_summary: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".ai/work-items/active/WI-ORDER-RECOVER.summary.json"),
        )
        .expect("finished summary"),
    )
    .expect("finished summary JSON");
    assert!(
        finished_summary
            .get("verificationRecoveryReconciled")
            .is_none(),
        "finish must consume the recovery projection marker"
    );
}

#[test]
fn future_lifecycle_evidence_is_deferred_until_terminal_completion_boundary() {
    let directory = repository();
    let id = "WI-ORDER-FUTURE-EVIDENCE";
    start_work_item_with_options(
        directory.path(),
        id,
        "lifecycle ordering",
        "preserve serial governance",
        &[".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            required_evidence_classes: vec!["delegated:github".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start");
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

    let preflight = preflight_work_item(directory.path(), &contract_path).expect("preflight");
    assert_eq!(preflight.state, DecisionState::Green);
    checkpoint_work_item(directory.path(), id).expect("checkpoint");
    record_verification(
        directory.path(),
        id,
        &serde_json::json!({"passed": true}),
        "0.2.8",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("verification");

    let summary: serde_json::Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(format!(".ai/work-items/active/{id}.summary.json")),
        )
        .expect("summary"),
    )
    .expect("summary JSON");
    assert_eq!(summary["preflightState"], "green");
    finish_work_item(directory.path(), id).expect("finish must not require future evidence");

    archive_work_item(directory.path(), id)
        .expect("archive must defer evidence produced by reviewed provider actions");
}

#[test]
fn future_lifecycle_stage_labels_are_rejected_at_entry_boundary() {
    let directory = repository();
    let id = "WI-ORDER-FUTURE-LABEL";
    let error = start_work_item_with_options(
        directory.path(),
        id,
        "lifecycle ordering",
        "reject future-stage labels before verification",
        &[".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            required_evidence_classes: vec!["public-install".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect_err("future lifecycle labels must fail before verification");
    let message = error.to_string();
    assert!(
        message.contains("unsupported required evidence class"),
        "{message}"
    );
    assert!(message.contains("public-install"), "{message}");
    assert!(
        !directory
            .path()
            .join(format!(".ai/work-items/active/{id}.contract.json"))
            .exists()
    );
}

#[test]
fn runtime_bound_verification_keeps_governance_bound_to_current_runtime() {
    let directory = repository();
    let id = "WI-ORDER-RUNTIME-BOUND";
    start(directory.path(), id, &["verification"]);
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
    let current_runtime = runtime();
    let contract_path = contract(directory.path(), id);
    preflight_work_item_with_runtime(directory.path(), &contract_path, &current_runtime)
        .expect("runtime-bound preflight");
    checkpoint_work_item(directory.path(), id).expect("checkpoint");

    let run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "runtime-bound-check".into(),
            program: "true".into(),
            args: Vec::new(),
            scope: vec![".ai/**".into(), "src/**".into(), "source.rs".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: current_runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: None,
            timeout_seconds: None,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .expect("verification run");
    record_verification_with_runtime(
        directory.path(),
        id,
        &serde_json::to_value(&run.receipt).expect("verification receipt JSON"),
        &current_runtime,
        &run.final_snapshot,
    )
    .expect("runtime-bound verification");

    let summary_path = directory
        .path()
        .join(".ai/work-items/active")
        .join(format!("{id}.summary.json"));
    let summary: serde_json::Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("summary")).expect("summary JSON");
    assert_eq!(summary["preflightState"], "green");
    assert_eq!(
        work_item_status_snapshot_with_runtime(directory.path(), id, &current_runtime)
            .expect("post-verification status")
            .verification,
        "verified",
        "the current Runtime-bound receipt remains valid without a redundant preflight"
    );
    finish_work_item(directory.path(), id).expect("finish after runtime-bound verification");
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
    commit_fixture_source(directory.path(), "source.rs", "authorized source edit");
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
fn legacy_command_only_amendment_after_verification_detects_formal_receipt() {
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
    assert_eq!(amendment["verificationStarted"], serde_json::json!(true));
    assert_eq!(
        amendment["invalidatedRequiredChecks"],
        serde_json::json!([])
    );
}

#[test]
fn contract_amendment_allows_stale_predecessor_verification_to_be_replaced() {
    let directory = repository();
    let id = "WI-ORDER-AMEND-REVERIFY";
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
        "0.2.74",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("initial verification");

    let mut amended: serde_json::Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("contract")).expect("JSON");
    amended["title"] = serde_json::json!("amended after verification");
    fs::write(
        &contract_path,
        serde_json::to_vec_pretty(&amended).expect("serialize contract"),
    )
    .expect("amended contract");
    revalidate_contract_amendment(
        directory.path(),
        id,
        "replace the predecessor verification after an authorized Contract amendment",
    )
    .expect("amendment revalidation");

    let decision = preflight_work_item(directory.path(), &contract_path)
        .expect("stale predecessor must not block a fresh preflight");
    assert_ne!(decision.state, DecisionState::Red);
    record_verification(
        directory.path(),
        id,
        &serde_json::json!({"passed": true}),
        "0.2.74",
        &Digest::sha256_bytes(b"runtime"),
    )
    .expect("replacement verification");
}

const OBS_ID: &str = "WI-OBSERVATION-NATIVE";
const OBS_LEDGERS: [&str; 3] = [
    "docs/reference/reference-parity.md",
    "docs/reference/reference-parity.zh-CN.md",
    "docs/reference/reference-parity.ja.md",
];

fn source_observation_json(path: impl AsRef<std::path::Path>) -> serde_json::Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn source_observation_git(root: &std::path::Path, args: &[&str]) -> String {
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
    String::from_utf8(output.stdout).unwrap().trim().into()
}

fn source_observation_fixture(
    changed: usize,
) -> (tempfile::TempDir, RepositoryVerificationRequest) {
    use cockpit_repository::{amend_work_item_contract, start_work_item_with_options_and_runtime};
    let directory = repository();
    let root = directory.path();
    fs::write(root.join("source.rs"), "fn base() {}\n").unwrap();
    fs::create_dir_all(root.join("docs/reference")).unwrap();
    fs::create_dir_all(root.join("docs/work-items")).unwrap();
    for path in OBS_LEDGERS {
        fs::write(root.join(path), format!("| {OBS_ID} | in_progress |\n")).unwrap();
    }
    for suffix in ["", ".zh-CN", ".ja"] {
        fs::write(root.join(format!("docs/work-items/{OBS_ID}{suffix}.md")), format!("---\nworkItemId: {OBS_ID}\nlastVerifiedBy: {OBS_ID}\nstatus: in_progress\n---\nSynthetic native fixture.\n")).unwrap();
    }
    source_observation_git(root, &["add", "source.rs", "docs"]);
    source_observation_git(
        root,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-qm",
            "repository base B",
        ],
    );
    let base = source_observation_git(root, &["rev-parse", "HEAD"]);
    source_observation_git(root, &["branch", "-M", "main"]);
    source_observation_git(
        root,
        &[
            "remote",
            "add",
            "origin",
            "https://example.invalid/synthetic.git",
        ],
    );
    source_observation_git(root, &["update-ref", "refs/remotes/origin/main", &base]);
    source_observation_git(
        root,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );
    source_observation_git(root, &["checkout", "-qb", "observation-fixture"]);
    // A real synthetic predecessor lifecycle supplies the native recovery
    // exception; no branch-entry check or successor authority is bypassed.
    let predecessor = "WI-OBSERVATION-PREDECESSOR";
    start_work_item_with_options(
        root,
        predecessor,
        "synthetic predecessor",
        "preserve native ancestry",
        &["source.rs".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["bounded predecessor".into()],
            ..Default::default()
        },
    )
    .unwrap();
    preflight_work_item(root, &contract(root, predecessor)).unwrap();
    checkpoint_work_item(root, predecessor).unwrap();
    let run = run_repository_verification(
        root,
        &RepositoryVerificationRequest {
            node_id: "predecessor-check".into(),
            program: "true".into(),
            args: vec![],
            scope: vec!["source.rs".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: runtime().runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: None,
            timeout_seconds: None,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .unwrap();
    record_verification_with_runtime(
        root,
        predecessor,
        &serde_json::to_value(run.receipt).unwrap(),
        &runtime(),
        &run.final_snapshot,
    )
    .unwrap();
    finish_work_item(root, predecessor).unwrap();
    archive_work_item(root, predecessor).unwrap();
    fs::write(
        root.join("source.rs"),
        "fn base() {}\nfn inherited_predecessor() {}\n",
    )
    .unwrap();
    commit_fixture_source(root, "source.rs", "inherited predecessor S");
    let archive = root.join(".ai/work-items/archive");
    let predecessor_contract =
        source_observation_json(archive.join(format!("{predecessor}.contract.json")));
    let predecessor_summary =
        source_observation_json(archive.join(format!("{predecessor}.summary.json")));
    let predecessor_outcome =
        source_observation_json(archive.join(format!("{predecessor}.outcome.json")));
    cockpit_repository::record_recovery_decision(root, predecessor, &serde_json::json!({
        "schemaVersion":1,"decisionId":"work-item-recovery","decision":"successor","workItemId":predecessor,"repositoryId":cockpit_repository::repository_id(root),"predecessorWorkItemId":predecessor,
        "predecessorContractDigest":cockpit_protocol::digest_json(&predecessor_contract).unwrap(),"predecessorSummaryDigest":cockpit_protocol::digest_json(&predecessor_summary).unwrap(),"predecessorOutcomeDigest":cockpit_protocol::digest_json(&predecessor_outcome).unwrap(),"predecessorEventsDigest":Digest::sha256_bytes(&fs::read(archive.join(format!("{predecessor}.events.jsonl"))).unwrap()),
        "successorWorkItemId":OBS_ID,"runtimeVersion":runtime().runtime_version,"runtimeDigest":runtime().runtime_digest,"actor":"human:synthetic-fixture","authoritySource":"synthetic test only","reason":"Exercise native successor source responsibility","evidenceRefs":[format!(".ai/work-items/archive/{predecessor}.outcome.json")],"policyRefs":[],"decidedAt":"2026-10-20T00:00:00Z","resumeCondition":"continue bounded synthetic successor"
    }), &runtime()).unwrap();
    let path = contract(root, OBS_ID);
    let before = source_observation_json(&path);
    let scope = OBS_LEDGERS
        .iter()
        .copied()
        .chain(["source.rs", ".ai/**"])
        .map(str::to_owned)
        .collect::<Vec<_>>();
    amend_work_item_contract(root, OBS_ID, &serde_json::json!({
        "schemaVersion":1,"changeId":"synthetic-native-origin","expectedContractDigest":cockpit_protocol::digest_json(&before).unwrap(),"reason":"Synthetic test fixture only; no real authority.",
        "changes":[
            {"path":"/intent","operation":"set","value":"Observe actual native delta"},
            {"path":"/goal","operation":"set","value":"Checks see committed responsibility"},
            {"path":"/scope","operation":"replace","value":scope},
            {"path":"/risk","operation":"set","value":"normal"},
            {"path":"/authority","operation":"set","value":"authorized"},
            {"path":"/acceptanceCriteria","operation":"replace","value":["A1: native origin remains bounded"]}
        ]
    }), "").unwrap();
    start_work_item_with_options_and_runtime(
        root,
        OBS_ID,
        "Observe actual native delta",
        "Checks see committed responsibility",
        &scope,
        &WorkItemStartOptions {
            risk: "normal".into(),
            authority: "authorized".into(),
            acceptance_criteria: vec!["A1: native origin remains bounded".into()],
            ..Default::default()
        },
        &runtime(),
    )
    .unwrap();
    for path in OBS_LEDGERS.iter().take(changed) {
        let mut text = fs::read_to_string(root.join(path)).unwrap();
        text.push_str("\nsuccessor hunk\n");
        fs::write(root.join(path), text).unwrap();
    }
    if changed != 0 {
        source_observation_git(root, &["add", "docs"]);
        source_observation_git(
            root,
            &[
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "-qm",
                "successor delta H",
            ],
        );
    }
    preflight_work_item_with_runtime(root, &path, &runtime()).unwrap();
    let summary_path = root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"));
    let summary = source_observation_json(&summary_path);
    let current = source_observation_json(&path);
    cockpit_repository::record_work_item_governance_controls_with_runtime(root, OBS_ID, &serde_json::json!({
        "decisionEvidence":{"schemaVersion":1,"decisionId":"contract-preflight-review","decision":"confirm_review","workItemId":OBS_ID,"repositoryId":cockpit_repository::repository_id(root),"contractDigest":cockpit_protocol::digest_json(&current).unwrap(),"preflightDecisionDigest":summary["preflightDecisionDigest"],"repositorySnapshotDigest":summary["preflightRepositorySnapshotDigest"],"recordedAt":"2026-10-20T00:00:00Z","recordedBy":"human:synthetic-fixture","reason":"Synthetic exact plan decision, never real user evidence."},
        "acceptanceEvidence":[{"acceptanceId":"A1","evidence":[{"type":"test","path":"source.rs","locator":"inherited_predecessor","verification":"passed"}]}],
        "intentAlignment":{"state":"resolved","evidence":["source.rs"]}
    }), &runtime()).unwrap();
    checkpoint_work_item(root, OBS_ID).unwrap();
    (
        directory,
        RepositoryVerificationRequest {
            node_id: "project-command-0".into(),
            program: "true".into(),
            args: vec![],
            scope: vec!["**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: runtime().runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: Some(OBS_ID.into()),
            timeout_seconds: None,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
}

fn source_observation_prepare(
    root: &std::path::Path,
    request: &RepositoryVerificationRequest,
) -> cockpit_repository::PreparedWorkItemVerification {
    let binding =
        cockpit_repository::capture_work_item_verification(root, OBS_ID, &runtime()).unwrap();
    cockpit_repository::prepare_work_item_verification(binding, std::slice::from_ref(request))
        .unwrap()
}

fn source_observation_receipt(
    root: &std::path::Path,
    request: &RepositoryVerificationRequest,
) -> (serde_json::Value, cockpit_git::RepositorySnapshot) {
    let run = run_repository_verification(root, request).unwrap();
    let mut receipt = serde_json::to_value(&run.receipt).unwrap();
    receipt["runtimeVersion"] = runtime().runtime_version.into();
    receipt["runtimeDigest"] = runtime().runtime_digest.to_string().into();
    (receipt, run.final_snapshot)
}

#[test]
fn source_change_observation_validated_origin_and_real_gate_ownership() {
    for changed in [0, 2, 3] {
        let (directory, request) = source_observation_fixture(changed);
        let root = directory.path();
        source_observation_prepare(root, &request);
        let summary = source_observation_json(
            root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json")),
        );
        assert!(
            !summary["changedPaths"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p == "source.rs"),
            "inherited predecessor hunk is not ours"
        );
        assert_ne!(
            summary["sourceChangeObservation"]["sourceFacts"]["origin"]["repositoryBaseRevision"],
            summary["sourceChangeObservation"]["sourceFacts"]["origin"]["responsibilityRevision"]
        );
        // Invoke the unchanged real gate on the actual native fixture
        // records. No Contract or Summary ownership facts are hand-filled.
        let gate = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/ci/governance_integrity_gate.py");
        let script = "import runpy,sys,pathlib; m=runpy.run_path(sys.argv[1]); ok=m['recovery_successor_owns_parity_projection'](pathlib.Path(sys.argv[2]),'WI-OBSERVATION-PREDECESSOR',{'successorWorkItemId':'WI-OBSERVATION-NATIVE'}); assert ok==(sys.argv[3]=='3'),ok";
        let output = Command::new("python3")
            .args(["-c", script])
            .arg(gate)
            .arg(root)
            .arg(changed.to_string())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "real gate: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn source_change_observation_same_path_inherited_and_successor_hunks() {
    let (directory, request) = source_observation_fixture(3);
    let root = directory.path();
    fs::write(
        root.join("source.rs"),
        "fn base() {}\nfn inherited_predecessor() {}\nfn successor() {}\n",
    )
    .unwrap();
    commit_fixture_source(root, "source.rs", "same path successor hunk");
    preflight_work_item_with_runtime(root, &contract(root, OBS_ID), &runtime()).unwrap();
    source_observation_prepare(root, &request);
    let summary =
        source_observation_json(root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json")));
    assert!(
        summary["changedPaths"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p == "source.rs")
    );
    assert!(
        !summary["sourceChangeObservation"]["sourceFacts"]["origin"]["activationPreparedDigest"]
            .is_null()
    );
}

#[test]
fn source_change_observation_rejects_pending_foreign_tampered_and_missing_origin() {
    for kind in ["pending", "foreign", "tampered", "missing"] {
        let (directory, request) = source_observation_fixture(3);
        let root = directory.path();
        let binding =
            cockpit_repository::capture_work_item_verification(root, OBS_ID, &runtime()).unwrap();
        let dir = root.join(format!(".ai/evidence/{OBS_ID}.activation-transition"));
        if kind == "pending" {
            fs::remove_file(dir.join("committed.json")).unwrap();
        } else if kind == "missing" {
            fs::remove_dir_all(&dir).unwrap();
        } else {
            let p = dir.join("prepared.json");
            let mut value = source_observation_json(&p);
            if kind == "foreign" {
                value["record"]["repositoryId"] = "sha256:foreign".into();
            } else {
                value["recordDigest"] = "sha256:tampered".into();
            }
            fs::write(p, serde_json::to_vec(&value).unwrap()).unwrap();
        }
        let before =
            fs::read(root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"))).unwrap();
        assert!(
            cockpit_repository::prepare_work_item_verification(
                binding,
                std::slice::from_ref(&request)
            )
            .is_err(),
            "{kind}"
        );
        assert_eq!(
            before,
            fs::read(root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"))).unwrap()
        );
    }
}

#[test]
fn source_change_observation_lock_wait_rejects_head_contract_and_journal_drift() {
    for kind in ["head", "contract", "journal"] {
        let (directory, request) = source_observation_fixture(3);
        let root = directory.path();
        let binding =
            cockpit_repository::capture_work_item_verification(root, OBS_ID, &runtime()).unwrap();
        let file = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(root.join(format!(".ai/locks/{OBS_ID}.lifecycle.lock")))
            .unwrap();
        file.lock().unwrap();
        let before =
            fs::read(root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"))).unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let waiter = std::thread::spawn(move || {
            tx.send(()).unwrap();
            cockpit_repository::prepare_work_item_verification(binding, &[request]).is_err()
        });
        rx.recv().unwrap();
        match kind {
            "head" => {
                source_observation_git(
                    root,
                    &[
                        "-c",
                        "user.name=Test",
                        "-c",
                        "user.email=test@example.invalid",
                        "commit",
                        "--allow-empty",
                        "-qm",
                        "HEAD drift while locked",
                    ],
                );
            }
            "contract" => {
                let p = contract(root, OBS_ID);
                let mut c = source_observation_json(&p);
                c["goal"] = "tamper negative fixture".into();
                fs::write(p, serde_json::to_vec(&c).unwrap()).unwrap();
            }
            _ => {
                fs::write(
                    root.join(format!(
                        ".ai/evidence/{OBS_ID}.contract-amendments/00000002.prepared.json"
                    )),
                    b"{}",
                )
                .unwrap();
            }
        }
        file.unlock().unwrap();
        assert!(waiter.join().unwrap(), "{kind}");
        assert_eq!(
            before,
            fs::read(root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"))).unwrap()
        );
    }
}

#[test]
fn source_change_observation_concurrent_plans_and_legacy_recorders_preserve_facts() {
    let (directory, request) = source_observation_fixture(3);
    let root = directory.path();
    let one = cockpit_repository::capture_work_item_verification(root, OBS_ID, &runtime()).unwrap();
    let two = cockpit_repository::capture_work_item_verification(root, OBS_ID, &runtime()).unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let other = barrier.clone();
    let r = request.clone();
    let thread = std::thread::spawn(move || {
        other.wait();
        cockpit_repository::prepare_work_item_verification(two, &[r])
    });
    barrier.wait();
    let first =
        cockpit_repository::prepare_work_item_verification(one, std::slice::from_ref(&request));
    let second = thread.join().unwrap();
    assert_ne!(first.is_ok(), second.is_ok(), "one stale plan must reject");
    let prepared = first.ok().or(second.ok()).unwrap();
    let path = root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"));
    let before = fs::read(&path).unwrap();
    assert!(
        record_verification(
            root,
            OBS_ID,
            &serde_json::json!({"passed":true}),
            "legacy",
            &Digest::sha256_bytes(b"legacy")
        )
        .is_err()
    );
    let (receipt, snapshot) = source_observation_receipt(root, &request);
    assert!(
        record_verification_with_runtime(root, OBS_ID, &receipt, &runtime(), &snapshot).is_err()
    );
    assert_eq!(before, fs::read(&path).unwrap());
    cockpit_repository::record_prepared_work_item_verification(&prepared, &receipt, &snapshot)
        .unwrap();
    assert!(
        source_observation_json(&path)["changedPaths"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p == OBS_LEDGERS[2])
    );
}

#[test]
fn source_change_observation_execution_drift_and_completion_rollback_keep_start_facts() {
    let (directory, request) = source_observation_fixture(3);
    let root = directory.path();
    let prepared = source_observation_prepare(root, &request);
    let path = root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"));
    let before = fs::read(&path).unwrap();
    let (receipt, snapshot) = source_observation_receipt(root, &request);
    fs::write(root.join("source.rs"), "source drift during execution\n").unwrap();
    assert!(
        cockpit_repository::record_prepared_work_item_verification(&prepared, &receipt, &snapshot)
            .is_err()
    );
    assert_eq!(before, fs::read(&path).unwrap());
    source_observation_git(root, &["checkout", "--", "source.rs"]);
    // Force failure after the evidence write, inside the completion
    // transaction. Its rollback baseline must contain the native start facts.
    let mut malformed = source_observation_json(&path);
    malformed["verification"] = serde_json::json!({});
    fs::write(&path, serde_json::to_vec(&malformed).unwrap()).unwrap();
    let before = fs::read(&path).unwrap();
    let error =
        cockpit_repository::record_prepared_work_item_verification(&prepared, &receipt, &snapshot)
            .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Summary.verification must be an array"),
        "{error}"
    );
    assert_eq!(before, fs::read(&path).unwrap());
    assert!(
        !root
            .join(format!(".ai/evidence/{OBS_ID}.verification.json"))
            .exists()
    );
}

#[test]
fn source_change_observation_failure_success_evidence_commit_finish_archive() {
    let (directory, request) = source_observation_fixture(3);
    let root = directory.path();
    let mut failed = request.clone();
    failed.program = "python3".into();
    failed.args = vec!["-c".into(), "raise SystemExit(7)".into()];
    let _prepared = source_observation_prepare(root, &failed);
    let (failed_receipt, snapshot) = source_observation_receipt(root, &failed);
    assert_eq!(failed_receipt["passed"], false);
    let attempt = cockpit_repository::persist_verification_attempt(
        root,
        OBS_ID,
        &[failed],
        &snapshot,
        &runtime(),
        "execution_failed",
        Some(("verification_execution", "synthetic expected first failure")),
        Some(&failed_receipt),
    )
    .unwrap();
    let attempt_path = root.join(attempt["path"].as_str().unwrap());
    let failure_bytes = fs::read(&attempt_path).unwrap();
    let prepared = source_observation_prepare(root, &request);
    let (receipt, snapshot) = source_observation_receipt(root, &request);
    assert_eq!(receipt["passed"], true);
    cockpit_repository::record_prepared_work_item_verification(&prepared, &receipt, &snapshot)
        .unwrap();
    let path = root.join(format!(".ai/work-items/active/{OBS_ID}.summary.json"));
    let facts = source_observation_json(&path);
    for ledger in OBS_LEDGERS {
        assert!(
            facts["changedPaths"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p == ledger)
        );
    }
    let source_before = cockpit_repository::snapshot_digest(
        &cockpit_git::GitRepository::discover(root)
            .unwrap()
            .snapshot()
            .unwrap(),
    )
    .unwrap();
    source_observation_git(root, &["add", ".ai"]);
    source_observation_git(
        root,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-qm",
            "normal evidence-only commit",
        ],
    );
    let source_after = cockpit_repository::snapshot_digest(
        &cockpit_git::GitRepository::discover(root)
            .unwrap()
            .snapshot()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        source_before, source_after,
        "HEAD is strict only during execution"
    );
    let status = work_item_status_snapshot_with_runtime(root, OBS_ID, &runtime()).unwrap();
    assert!(!status.human_decision_required, "{status:#?}");
    cockpit_repository::finish_work_item_with_runtime(root, OBS_ID, &runtime()).unwrap();
    cockpit_repository::archive_work_item_with_runtime(root, OBS_ID, &runtime()).unwrap();
    assert_eq!(failure_bytes, fs::read(attempt_path).unwrap());
    let archived =
        source_observation_json(root.join(format!(".ai/work-items/archive/{OBS_ID}.summary.json")));
    assert!(
        archived["changedPaths"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p == OBS_LEDGERS[2])
    );
}

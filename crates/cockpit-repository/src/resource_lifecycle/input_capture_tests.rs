// Real Git/archive fixtures for the public recovery entry point.
// Existing integration tests remain unchanged. All fixture Git mutations are
// confined to the TempDir created by direct_merge_repository.
use super::*;
use crate::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_work_item_with_runtime, attach, checkpoint_work_item, finish_work_item_with_runtime,
    preflight_work_item_with_runtime, record_verification_with_runtime,
    run_repository_verification, start_work_item_with_options,
};
use serde_json::{Value, json};
use std::cell::RefCell;

const ID: &str = "WI-INPUT-CAPTURE";

thread_local! {
    static AFTER_CLASSIFICATION: RefCell<Option<Box<dyn FnOnce()>>> = RefCell::new(None);
}

struct RestoreHook(Option<Box<dyn FnOnce()>>);

impl Drop for RestoreHook {
    fn drop(&mut self) {
        AFTER_CLASSIFICATION.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}

fn with_classification_hook<T>(
    callback: impl FnOnce() + 'static,
    operation: impl FnOnce() -> T,
) -> T {
    let previous = AFTER_CLASSIFICATION.with(|slot| slot.borrow_mut().replace(Box::new(callback)));
    let _restore = RestoreHook(previous);
    operation()
}

pub(super) fn after_classification() {
    let callback = AFTER_CLASSIFICATION.with(|slot| slot.borrow_mut().take());
    if let Some(callback) = callback {
        callback();
    }
}

fn runtime() -> RuntimeContext {
    RuntimeContext {
        runtime_version: "test-runtime".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"test-runtime"),
    }
}

fn git(directory: &tempfile::TempDir, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(directory.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}

fn direct_merge_repository() -> (
    tempfile::TempDir,
    ResourceFinalizationContext,
    Digest,
    String,
    String,
    String,
) {
    let directory = tempfile::tempdir().unwrap();
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .unwrap()
            .success()
    );
    git(
        &directory,
        &["config", "user.email", "tests@example.invalid"],
    );
    git(&directory, &["config", "user.name", "AI Cockpit Tests"]);
    git(&directory, &["branch", "-M", "main"]);
    fs::write(directory.path().join("README.md"), "direct merge fixture\n").unwrap();
    git(&directory, &["add", "README.md"]);
    git(&directory, &["commit", "-q", "-m", "base"]);
    let base_revision = git(&directory, &["rev-parse", "HEAD"]);

    git(
        &directory,
        &["checkout", "-q", "-b", "feature/direct-merge"],
    );
    fs::write(
        directory.path().join("direct-merge.txt"),
        "merged history\n",
    )
    .unwrap();
    git(&directory, &["add", "direct-merge.txt"]);
    git(&directory, &["commit", "-q", "-m", "direct merge feature"]);
    let feature_head = git(&directory, &["rev-parse", "HEAD"]);
    git(&directory, &["checkout", "-q", "main"]);
    git(
        &directory,
        &[
            "merge",
            "--no-ff",
            "-q",
            "feature/direct-merge",
            "-m",
            "direct merge",
        ],
    );
    let merge_commit = git(&directory, &["rev-parse", "HEAD"]);
    git(&directory, &["reset", "--hard", "-q", &base_revision]);
    git(&directory, &["branch", "-D", "feature/direct-merge"]);

    attach(directory.path()).unwrap();
    start_work_item_with_options(
        directory.path(),
        ID,
        "record a historical direct merge",
        "preserve direct merge history",
        &[".ai/**".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["direct merge facts remain auditable".into()],
            ..Default::default()
        },
    )
    .unwrap();
    let context = ResourceFinalizationContext {
        branch: "feature/direct-merge".into(),
        worktree: directory.path().to_string_lossy().to_string(),
        base_branch: "main".into(),
        base_remote: "origin".into(),
        // Keep the original Contract binding as an object repository does:
        // the historical receipt changes only its provider/URL identity.
        provider: "local".into(),
        pull_request: "https://github.com/example/project/pull/191".into(),
    };
    plan_resource_finalization(directory.path(), ID, &context).unwrap();
    let current = runtime();
    let contract_path = directory
        .path()
        .join(format!(".ai/work-items/active/{ID}.contract.json"));
    preflight_work_item_with_runtime(directory.path(), &contract_path, &current).unwrap();
    checkpoint_work_item(directory.path(), ID).unwrap();
    let run = run_repository_verification(
        directory.path(),
        &RepositoryVerificationRequest {
            node_id: "project-command-0".into(),
            program: "true".into(),
            args: vec![],
            scope: vec![".ai/**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: current.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: None,
            timeout_seconds: None,
            policy: RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .unwrap();
    let mut evidence = serde_json::to_value(&run.receipt).unwrap();
    evidence["runtimeVersion"] = current.runtime_version.clone().into();
    evidence["runtimeDigest"] = current.runtime_digest.to_string().into();
    record_verification_with_runtime(
        directory.path(),
        ID,
        &evidence,
        &current,
        &run.final_snapshot,
    )
    .unwrap();
    finish_work_item_with_runtime(directory.path(), ID, &current).unwrap();
    archive_work_item_with_runtime(directory.path(), ID, &current).unwrap();
    let contract_path = directory
        .path()
        .join(format!(".ai/work-items/archive/{ID}.contract.json"));
    let contract_digest = Digest::sha256_bytes(&fs::read(contract_path).unwrap());
    (
        directory,
        context,
        contract_digest,
        base_revision,
        feature_head,
        merge_commit,
    )
}

fn write_input(directory: &tempfile::TempDir, name: &str, value: &Value) -> std::path::PathBuf {
    let inputs = directory.path().join(".ai/test-inputs");
    fs::create_dir_all(&inputs).unwrap();
    let path = inputs.join(name);
    fs::write(&path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
    path
}

fn direct_receipt(directory: &tempfile::TempDir, merge_commit: &str) -> Value {
    let plan =
        historical_finalization_recovery_plan(directory.path(), ID, &runtime(), Some(merge_commit))
            .expect("real direct merge plan");
    let mut receipt = plan["suggestedReceipt"].clone();
    receipt["actor"] = "human:captured-A".into();
    receipt["authoritySource"] = "historical-test".into();
    receipt["reason"] = "preserve the exact classified direct input".into();
    receipt["timestamp"] = "2026-09-03T07:00:00Z".into();
    receipt
}

#[test]
fn first_direct_recovery_preserves_classified_input_when_path_is_replaced() {
    let (directory, context, _contract, _base, _head, merge_commit) = direct_merge_repository();
    let original = direct_receipt(&directory, &merge_commit);
    let input = write_input(&directory, "replace.json", &original);
    let mut replacement = original.clone();
    replacement.as_object_mut().unwrap().remove("historical");
    replacement["receiptId"] = "replacement-B".into();
    replacement["operationId"] = "replacement-operation-B".into();
    replacement["actor"] = "human:replacement-B".into();
    replacement["provider"] = context.provider.into();
    replacement["pullRequest"]["number"] = 191.into();
    replacement["pullRequest"]["url"] = context.pull_request.into();
    // Establish both inputs as independently valid before injecting the race.
    // A rejection caused by a broken B fixture would not reproduce the gap.
    prepare_resource_finalization(directory.path(), ID, &original, &input, &runtime())
        .expect("original direct A must independently validate");
    prepare_resource_finalization(directory.path(), ID, &replacement, &input, &runtime())
        .expect("ordinary PR B must independently validate under the same Contract");
    let destination = resource_finalization_decision_path(directory.path(), ID);
    assert!(!destination.exists());
    let path = input.clone();
    let result = with_classification_hook(
        move || fs::write(path, serde_json::to_vec_pretty(&replacement).unwrap()).unwrap(),
        || record_historical_finalization_recovery(directory.path(), ID, &input, &runtime()),
    )
    .expect("public first-record recovery completes");
    assert_eq!(result["state"], "recorded");
    let persisted: Value = serde_json::from_slice(&fs::read(&destination).unwrap()).unwrap();
    assert_eq!(
        persisted, original,
        "the public recovery must publish the exact classified A, not reopen the external path for B"
    );
}

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
use serde_json::Value;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const ID: &str = "WI-INPUT-CAPTURE";

#[derive(Clone, Copy, PartialEq, Eq)]
enum CapturePoint {
    Captured,
    Classified,
}

type InputHook = (CapturePoint, Box<dyn FnOnce()>);

thread_local! {
    static INPUT_HOOK: RefCell<Option<InputHook>> = RefCell::new(None);
}

struct RestoreHook(Option<InputHook>);

impl Drop for RestoreHook {
    fn drop(&mut self) {
        INPUT_HOOK.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}

fn with_classification_hook<T>(
    callback: impl FnOnce() + 'static,
    operation: impl FnOnce() -> T,
) -> T {
    with_hook(CapturePoint::Classified, callback, operation)
}

fn with_hook<T>(
    point: CapturePoint,
    callback: impl FnOnce() + 'static,
    operation: impl FnOnce() -> T,
) -> T {
    let previous = INPUT_HOOK.with(|slot| slot.borrow_mut().replace((point, Box::new(callback))));
    let _restore = RestoreHook(previous);
    operation()
}

pub(super) fn after_classification() {
    hit(CapturePoint::Classified);
}

pub(super) fn after_capture() {
    hit(CapturePoint::Captured);
}

fn hit(point: CapturePoint) {
    let callback = INPUT_HOOK.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot
            .as_ref()
            .is_some_and(|(expected, _)| *expected == point)
        {
            slot.take()
        } else {
            None
        }
    });
    if let Some((_, callback)) = callback {
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
    replacement["resourceContext"] = serde_json::to_value(&context).unwrap();
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

#[test]
fn invalid_captured_input_cannot_be_replaced_with_a_valid_receipt() {
    let (directory, _context, _contract, _base, _head, merge_commit) = direct_merge_repository();
    let valid = direct_receipt(&directory, &merge_commit);
    let input = write_input(&directory, "invalid-to-valid.json", &Value::Null);
    prepare_resource_finalization(directory.path(), ID, &valid, &input, &runtime()).unwrap();
    let path = input.clone();
    let replacement = serde_json::to_vec_pretty(&valid).unwrap();
    let called = Rc::new(Cell::new(false));
    let observed = called.clone();
    let error = with_hook(
        CapturePoint::Captured,
        move || {
            observed.set(true);
            fs::write(path, replacement).unwrap();
        },
        || record_historical_finalization_recovery(directory.path(), ID, &input, &runtime()),
    )
    .unwrap_err();
    assert!(called.get(), "the real read must occur before replacement");
    assert!(
        error
            .to_string()
            .contains("requires an existing predecessor")
    );
    assert!(!resource_finalization_decision_path(directory.path(), ID).exists());
}

#[test]
fn captured_direct_receipt_survives_input_path_deletion() {
    let (directory, _context, _contract, _base, _head, merge_commit) = direct_merge_repository();
    let original = direct_receipt(&directory, &merge_commit);
    let input = write_input(&directory, "delete-after-classification.json", &original);
    let path = input.clone();
    let result = with_classification_hook(
        move || fs::remove_file(path).unwrap(),
        || record_historical_finalization_recovery(directory.path(), ID, &input, &runtime()),
    )
    .unwrap();
    assert_eq!(result["state"], "recorded");
    assert!(!input.exists());
    let persisted: Value = serde_json::from_slice(
        &fs::read(resource_finalization_decision_path(directory.path(), ID)).unwrap(),
    )
    .unwrap();
    assert_eq!(persisted, original);
}

fn assert_classification_rejected(directory: &tempfile::TempDir, input: &Path, bytes: &[u8]) {
    fs::write(input, bytes).unwrap();
    let error = record_historical_finalization_recovery(directory.path(), ID, input, &runtime())
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("requires an existing predecessor")
    );
    assert!(!resource_finalization_decision_path(directory.path(), ID).exists());
}

#[test]
fn strict_classification_rejects_duplicate_keys_unknown_fields_and_malformed_json() {
    let (directory, _context, _contract, _base, _head, merge_commit) = direct_merge_repository();
    let original = direct_receipt(&directory, &merge_commit);
    let input = write_input(&directory, "strict-input.json", &original);
    let bytes = serde_json::to_vec(&original).unwrap();
    let mut duplicate = b"{\"actor\":\"duplicate\",".to_vec();
    duplicate.extend_from_slice(&bytes[1..]);
    assert_classification_rejected(&directory, &input, &duplicate);
    let mut unknown = original;
    unknown["unexpectedInputField"] = true.into();
    assert_classification_rejected(&directory, &input, &serde_json::to_vec(&unknown).unwrap());
    assert_classification_rejected(&directory, &input, b"{\"historical\":");
}

#[test]
fn first_input_read_preserves_the_eight_mib_budget_and_specific_error() {
    let (directory, _context, _contract, _base, _head, merge_commit) = direct_merge_repository();
    let original = direct_receipt(&directory, &merge_commit);
    let input = write_input(&directory, "bounded-input.json", &original);
    let mut bytes = serde_json::to_vec(&original).unwrap();
    bytes.resize(8 * 1024 * 1024 + 1, b' ');
    fs::write(&input, &bytes).unwrap();
    let error = record_historical_finalization_recovery(directory.path(), ID, &input, &runtime())
        .unwrap_err();
    assert!(error.to_string().contains("record_budget_exceeded"));
    assert!(!resource_finalization_decision_path(directory.path(), ID).exists());
    bytes.pop();
    fs::write(&input, bytes).unwrap();
    let result = record_historical_finalization_recovery(directory.path(), ID, &input, &runtime())
        .expect("exactly the existing 8 MiB reader budget remains admissible");
    assert_eq!(result["state"], "recorded");
}

#[test]
fn missing_and_directory_inputs_preserve_reader_errors() {
    let (directory, _context, _contract, _base, _head, _merge_commit) = direct_merge_repository();
    let input = directory.path().join("missing.json");
    let missing = record_historical_finalization_recovery(directory.path(), ID, &input, &runtime())
        .unwrap_err();
    assert!(matches!(missing, ObserverError::Read { .. }));
    fs::create_dir(&input).unwrap();
    let nonregular =
        record_historical_finalization_recovery(directory.path(), ID, &input, &runtime())
            .unwrap_err();
    assert!(
        !nonregular
            .to_string()
            .contains("requires an existing predecessor")
    );
    assert!(!resource_finalization_decision_path(directory.path(), ID).exists());
}

#[cfg(unix)]
#[test]
fn first_recovery_rejects_leaf_symlinks_without_following_them() {
    let (directory, _context, _contract, _base, _head, merge_commit) = direct_merge_repository();
    let original = direct_receipt(&directory, &merge_commit);
    let target = write_input(&directory, "symlink-target.json", &original);
    let original_bytes = fs::read(&target).unwrap();
    let input = target.with_file_name("symlink-input.json");
    std::os::unix::fs::symlink(&target, &input).unwrap();
    let error = record_historical_finalization_recovery(directory.path(), ID, &input, &runtime())
        .unwrap_err();
    assert!(matches!(error, ObserverError::Read { .. }));
    assert_eq!(fs::read(&target).unwrap(), original_bytes);
    assert!(!resource_finalization_decision_path(directory.path(), ID).exists());
}

#[test]
fn captured_input_still_requires_current_identity_and_git_bindings() {
    let (directory, _context, _contract, _base, _head, merge_commit) = direct_merge_repository();
    let original = direct_receipt(&directory, &merge_commit);
    let input = write_input(&directory, "identity-input.json", &original);
    let mut cases = Vec::new();
    for field in [
        "repositoryId",
        "workItemId",
        "contractDigest",
        "runtimeDigest",
    ] {
        let mut invalid = original.clone();
        invalid[field] = if field.ends_with("Digest") || field == "repositoryId" {
            Digest::sha256_bytes(b"foreign identity").to_string().into()
        } else {
            "WI-FOREIGN".into()
        };
        cases.push(invalid);
    }
    let mut invalid_parents = original.clone();
    invalid_parents["historical"]["mergeParents"]
        .as_array_mut()
        .unwrap()
        .reverse();
    cases.push(invalid_parents);
    let mut invalid_base = original;
    // With PR base already equal to Contract base, the optional historical
    // Contract-base hint is redundant. Exercise the actual Git base binding.
    invalid_base["pullRequest"]["baseRevision"] = "foreign-base".into();
    cases.push(invalid_base);
    for invalid in cases {
        fs::write(&input, serde_json::to_vec(&invalid).unwrap()).unwrap();
        let error =
            record_historical_finalization_recovery(directory.path(), ID, &input, &runtime())
                .unwrap_err();
        assert!(
            !error
                .to_string()
                .contains("requires an existing predecessor"),
            "well-shaped direct input must reach the existing binding validation: {error}"
        );
        assert!(!resource_finalization_decision_path(directory.path(), ID).exists());
    }
}

#[test]
fn bytes_writer_rejects_the_wrong_guard_before_parsing_input() {
    let directory = tempfile::tempdir().unwrap();
    git(&directory, &["init", "-q"]);
    attach(directory.path()).unwrap();
    let root = directory.path().canonicalize().unwrap();
    let guard = acquire_lifecycle_lock(&root, "WI-LOCK-OWNER").unwrap();
    let error = record_resource_finalization_bytes_locked(
        &root,
        ID,
        &root.join("missing-input.json"),
        b"not JSON",
        &runtime(),
        &guard,
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("lifecycle_lock_subject_mismatch")
    );
}

#[test]
fn ordinary_finalize_replay_preserves_the_first_canonical_bytes() {
    let (directory, _context, _contract, _base, _head, merge_commit) = direct_merge_repository();
    let original = direct_receipt(&directory, &merge_commit);
    let input = write_input(&directory, "ordinary-replay.json", &original);
    let result =
        record_historical_finalization_recovery(directory.path(), ID, &input, &runtime()).unwrap();
    assert_eq!(result["state"], "recorded");
    let destination = resource_finalization_decision_path(directory.path(), ID);
    let frozen = fs::read(&destination).unwrap();
    fs::write(&input, serde_json::to_vec(&original).unwrap()).unwrap();
    let replay = record_resource_finalization(directory.path(), ID, &input, &runtime()).unwrap();
    assert_eq!(replay["state"], "idempotent");
    assert_eq!(fs::read(destination).unwrap(), frozen);
}

#[test]
fn private_hook_is_one_shot_and_restores_its_predecessor_on_unwind() {
    let count = Rc::new(Cell::new(0));
    let outer = count.clone();
    with_classification_hook(
        move || outer.set(outer.get() + 1),
        || {
            let inner = count.clone();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                with_classification_hook(
                    move || inner.set(inner.get() + 10),
                    || {
                        after_classification();
                        after_classification();
                        panic!("exercise callback state restoration");
                    },
                );
            }));
            assert!(result.is_err());
            assert_eq!(count.get(), 10);
            after_classification();
            after_classification();
            assert_eq!(count.get(), 11);
        },
    );
    after_classification();
    assert_eq!(count.get(), 11);
}

//! Real repository business flows in synthetic local Git fixtures only.
//! The injected source/ledger is simulated, not persistent production B.
use super::*;
pub(super) fn git(root: &Path, args: &[&str]) {
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
}
pub(super) fn commit(root: &Path, message: &str) {
    git(root, &["add", "-A"]);
    git(
        root,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "--allow-empty",
            "-qm",
            message,
        ],
    );
}
pub(super) fn prepare_archive(root: &Path, rt: &cockpit_protocol::RuntimeContext) {
    crate::checkpoint_work_item(root, "WI-PORT").expect("native checkpoint");
    let execution = crate::run_repository_verification(
        root,
        &crate::RepositoryVerificationRequest {
            node_id: "authority-lifecycle-fixture-verification".into(),
            program: "true".into(),
            args: vec![],
            scope: vec!["src/**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: rt.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: Some("WI-PORT".into()),
            timeout_seconds: None,
            policy: crate::RepositoryVerificationPolicy::NeverReuse,
        },
    )
    .expect("execute true in synthetic fixture");
    crate::record_verification_with_runtime(
        root,
        "WI-PORT",
        &serde_json::to_value(&execution.receipt).unwrap(),
        rt,
        &execution.final_snapshot,
    )
    .expect("record actual fixture verification");
    crate::finish_work_item_with_runtime(root, "WI-PORT", rt).expect("native finish");
    crate::archive_work_item_with_runtime(root, "WI-PORT", rt).expect("native archive");
}
pub(super) fn close_decision() -> cockpit_protocol::HumanDecision {
    cockpit_protocol::HumanDecision {
        decision: "approved".into(),
        actor: "fixture-unverified-audit-actor".into(),
        authority_source: "fixture audit metadata; real source is simulated protected port".into(),
        reason: "exact synthetic closeout".into(),
        evidence_refs: vec![".ai/evidence/WI-PORT.verification.json".into()],
        policy_refs: vec![],
        decided_at: "2026-10-20T00:00:00Z".into(),
        resume_condition: None,
    }
}
fn close_exact(port: &Arc<Mock>, root: &Path, rt: &cockpit_protocol::RuntimeContext) -> Value {
    let decision = close_decision();
    approve_operation(
        port,
        root,
        HumanReviewOperation::Close,
        &json!({"decision":decision,"runtime":Some(rt)}),
    );
    serde_json::to_value(
        crate::close_work_item_with_structured_decision_and_runtime(root, "WI-PORT", &decision, rt)
            .expect("native close"),
    )
    .unwrap()
}
#[test]
fn native_retirement_succeeds_preserves_bytes_and_replays_after_active_record_moves() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let rt = runtime();
    let request = retirement();
    let active = t.path().join(".ai/work-items/active/WI-PORT.contract.json");
    let original = std::fs::read(&active).unwrap();
    approve_operation(
        &p,
        t.path(),
        HumanReviewOperation::Retire,
        &json!({"request":request,"runtime":rt}),
    );
    let result = crate::retire_active_work_item_with_runtime(t.path(), "WI-PORT", &request, &rt)
        .expect("strict native retirement success");
    assert_eq!(result["verificationClaim"], "not_verified");
    assert!(!active.exists());
    assert_eq!(
        std::fs::read(
            t.path()
                .join(".ai/work-items/archive/WI-PORT.contract.json")
        )
        .unwrap(),
        original
    );
    assert_eq!(
        crate::retire_active_work_item_with_runtime(t.path(), "WI-PORT", &request, &rt).unwrap(),
        result
    );
    assert_eq!(p.store.lock().unwrap().executions, 1);
}
#[test]
fn native_archive_close_and_exact_terminal_replay_succeed() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let rt = runtime();
    prepare_archive(t.path(), &rt);
    commit(t.path(), "archive before exact close review");
    let closed = close_exact(&p, t.path(), &rt);
    assert_eq!(closed["state"], "closed");
    let again = serde_json::to_value(
        crate::close_work_item_with_structured_decision_and_runtime(
            t.path(),
            "WI-PORT",
            &close_decision(),
            &rt,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(closed, again);
    assert_eq!(p.store.lock().unwrap().executions, 1);
}
pub(super) fn cross_fixture() -> (
    tempfile::TempDir,
    tempfile::TempDir,
    std::path::PathBuf,
    Arc<Mock>,
    Installed,
    cockpit_protocol::RuntimeContext,
) {
    let source = fixture();
    let p = Mock::new();
    let installed = Installed::new(p.clone());
    let rt = runtime();
    prepare_archive(source.path(), &rt);
    commit(source.path(), "archive synthetic closeout");
    let parent = tempfile::tempdir().unwrap();
    let destination = parent.path().join("clone");
    git(
        source.path(),
        &[
            "clone",
            "--quiet",
            "--no-hardlinks",
            source.path().to_str().unwrap(),
            destination.to_str().unwrap(),
        ],
    );
    close_exact(&p, source.path(), &rt);
    (source, parent, destination, p, installed, rt)
}
#[test]
fn completed_cross_checkout_replays_after_source_directory_disappears() {
    let (source, parent, destination, p, _installed, rt) = cross_fixture();
    let source_path = source.path().to_path_buf();
    let payload = json!({"sourceRoot":source_path,"runtime":rt});
    approve_operation(
        &p,
        &destination,
        HumanReviewOperation::CrossCheckoutRecovery,
        &payload,
    );
    let result = crate::recover_cross_checkout_closeout(&destination, &source_path, "WI-PORT", &rt)
        .expect("native cross-checkout import");
    let before = p.store.lock().unwrap().executions;
    std::fs::rename(
        &source_path,
        parent.path().join("source-moved-after-completion"),
    )
    .unwrap();
    assert!(!source_path.exists());
    let repeated =
        crate::recover_cross_checkout_closeout(&destination, &source_path, "WI-PORT", &rt)
            .expect("completed operation can replay without reading its former source directory");
    assert_eq!(repeated, result);
    assert_eq!(p.store.lock().unwrap().executions, before);
}
#[test]
fn unstarted_cross_checkout_rejects_missing_source_without_starting_business() {
    let (source, parent, destination, p, _installed, rt) = cross_fixture();
    let source_path = source.path().to_path_buf();
    approve_operation(
        &p,
        &destination,
        HumanReviewOperation::CrossCheckoutRecovery,
        &json!({"sourceRoot":source_path,"runtime":rt}),
    );
    let before = p.store.lock().unwrap().executions;
    std::fs::rename(
        &source_path,
        parent.path().join("source-moved-before-start"),
    )
    .unwrap();
    assert!(
        crate::recover_cross_checkout_closeout(&destination, &source_path, "WI-PORT", &rt).is_err()
    );
    assert_eq!(p.store.lock().unwrap().executions, before);
    assert!(
        !destination
            .join(".ai/decisions/WI-PORT.close.json")
            .exists()
    );
}
#[test]
fn changed_source_plan_requires_new_exact_review_before_cross_checkout_start() {
    let (source, _parent, destination, p, _installed, rt) = cross_fixture();
    approve_operation(
        &p,
        &destination,
        HumanReviewOperation::CrossCheckoutRecovery,
        &json!({"sourceRoot":source.path(),"runtime":rt}),
    );
    let before = p.store.lock().unwrap().executions;
    std::fs::write(
        source.path().join("changed-source.txt"),
        b"unreviewed input",
    )
    .unwrap();
    assert!(
        crate::recover_cross_checkout_closeout(&destination, source.path(), "WI-PORT", &rt)
            .is_err()
    );
    assert_eq!(p.store.lock().unwrap().executions, before);
    assert!(
        !destination
            .join(".ai/decisions/WI-PORT.close.json")
            .exists()
    );
}
#[test]
fn native_retry_recovery_has_current_source_and_preserves_historical_receipt_after_denial() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let root = t.path();
    let rt = runtime();
    crate::checkpoint_work_item(root, "WI-PORT").unwrap();
    assert!(
        crate::finish_work_item_with_runtime(root, "WI-PORT", &rt).is_err(),
        "retain a real synthetic blocked native finish"
    );
    let active = root.join(".ai/work-items/active");
    let contract = read_json(&active.join("WI-PORT.contract.json")).unwrap();
    let summary = read_json(&active.join("WI-PORT.summary.json")).unwrap();
    let outcome = read_json(&active.join("WI-PORT.outcome.json")).unwrap();
    let events = std::fs::read(active.join("WI-PORT.events.jsonl")).unwrap();
    let receipt = json!({"schemaVersion":1,"decisionId":"work-item-recovery","decision":"retry","workItemId":"WI-PORT","repositoryId":repository_id(root),"predecessorWorkItemId":"WI-PORT","predecessorContractDigest":cockpit_protocol::digest_json(&contract).unwrap(),"predecessorSummaryDigest":cockpit_protocol::digest_json(&summary).unwrap(),"predecessorOutcomeDigest":cockpit_protocol::digest_json(&outcome).unwrap(),"predecessorEventsDigest":Digest::sha256_bytes(&events),"runtimeVersion":rt.runtime_version,"runtimeDigest":rt.runtime_digest,"actor":"fixture-unverified-audit-actor","authoritySource":"fixture port proves the exact source","reason":"retry missing synthetic verification","evidenceRefs":[".ai/work-items/active/WI-PORT.outcome.json"],"policyRefs":[],"decidedAt":"2026-10-20T00:00:00Z","resumeCondition":"fresh actual verification required"});
    let payload = json!({"receipt":receipt,"runtime":rt});
    approve_operation(&p, root, HumanReviewOperation::Recovery, &payload);
    let recorded = crate::record_recovery_decision(root, "WI-PORT", &receipt, &rt)
        .expect("strict native retry recovery success");
    assert_eq!(recorded["decision"], "retry");
    assert!(
        current_recovery_decision(root, "WI-PORT", Some(&rt))
            .unwrap()
            .is_some()
    );
    let path = root.join(".ai/decisions/WI-PORT.recovery.json");
    let original = std::fs::read(&path).unwrap();
    let fresh = p
        .issue(
            &human_operation_binding(root, "WI-PORT", HumanReviewOperation::Recovery, &payload)
                .unwrap(),
        )
        .unwrap();
    let proof = p.approve(&fresh, false);
    p.store
        .lock()
        .unwrap()
        .records
        .get_mut(&proof.approval_id)
        .unwrap()
        .denied = true;
    assert!(current_recovery_decision(root, "WI-PORT", Some(&rt)).is_err());
    assert!(
        crate::load_recovery_decision(root, "WI-PORT", Some(&rt))
            .unwrap()
            .is_some()
    );
    assert_eq!(std::fs::read(path).unwrap(), original);
}

#[test]
fn native_profile_effect_with_unknown_finish_cannot_restart_with_a_new_approval() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let args = vec!["test".into()];
    let payload = json!({"program":"cargo","args":args});
    approve_operation(
        &p,
        t.path(),
        HumanReviewOperation::ProfileConfirmation,
        &payload,
    );
    let before = read_json(&t.path().join(".ai/project.json")).unwrap()["profileVersion"]
        .as_u64()
        .unwrap();
    p.store.lock().unwrap().fail_method = Some("finish_before");
    assert!(crate::confirm_profile_update(t.path(), "cargo", &args).is_err());
    let path = t.path().join(".ai/project.json");
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(read_json(&path).unwrap()["profileVersion"], before + 1);
    let fresh = p
        .issue(
            &human_operation_binding(
                t.path(),
                "WI-PORT",
                HumanReviewOperation::ProfileConfirmation,
                &payload,
            )
            .unwrap(),
        )
        .unwrap();
    let proof = p.approve(&fresh, false);
    let receipt = submit_human_review(t.path(), "WI-PORT", &fresh, &proof).unwrap();
    assert_eq!(receipt["humanAuthorityGranted"], false);
    let error = crate::confirm_profile_update(t.path(), "cargo", &args).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("human_execution_recovery_required"),
        "{error}"
    );
    assert_eq!(std::fs::read(path).unwrap(), bytes);
    assert_eq!(p.store.lock().unwrap().executions, 1);
}
#[test]
fn native_cross_effect_lost_finish_preserves_close_bytes_without_reexecution() {
    for mode in ["finish_before", "finish_after"] {
        let (source, parent, destination, p, _installed, rt) = cross_fixture();
        let source_path = source.path().to_path_buf();
        approve_operation(
            &p,
            &destination,
            HumanReviewOperation::CrossCheckoutRecovery,
            &json!({"sourceRoot":source_path,"runtime":rt}),
        );
        p.store.lock().unwrap().fail_method = Some(mode);
        assert!(
            crate::recover_cross_checkout_closeout(&destination, &source_path, "WI-PORT", &rt)
                .is_err(),
            "{mode}"
        );
        let close = destination.join(".ai/decisions/WI-PORT.close.json");
        let bytes = std::fs::read(&close).expect("native side effect was installed before finish");
        let executions = p.store.lock().unwrap().executions;
        std::fs::rename(
            &source_path,
            parent.path().join("source-moved-after-effect"),
        )
        .unwrap();
        let repeated =
            crate::recover_cross_checkout_closeout(&destination, &source_path, "WI-PORT", &rt);
        assert_eq!(repeated.is_ok(), mode == "finish_after", "{mode}");
        if mode == "finish_before" {
            assert!(
                repeated
                    .unwrap_err()
                    .to_string()
                    .contains("human_execution_recovery_required")
            );
        }
        assert_eq!(std::fs::read(close).unwrap(), bytes);
        assert_eq!(p.store.lock().unwrap().executions, executions, "{mode}");
    }
}
#[test]
fn cross_request_relative_source_is_bound_to_its_absolute_lexical_identity() {
    let (source, _parent, destination, _p, _installed, rt) = cross_fixture();
    let cwd = std::env::current_dir().unwrap();
    let common = cwd
        .ancestors()
        .find(|ancestor| source.path().starts_with(ancestor))
        .expect("fixture and package directory share a filesystem root");
    let mut relative = std::path::PathBuf::new();
    for _ in cwd.strip_prefix(common).unwrap().components() {
        relative.push("..");
    }
    relative.push(source.path().strip_prefix(common).unwrap());
    assert!(!relative.is_absolute());
    let lexical = std::path::absolute(&relative).unwrap();
    let from_relative = human_operation_binding(
        &destination,
        "WI-PORT",
        HumanReviewOperation::CrossCheckoutRecovery,
        &json!({"sourceRoot":relative,"runtime":rt}),
    )
    .unwrap();
    let from_absolute = human_operation_binding(
        &destination,
        "WI-PORT",
        HumanReviewOperation::CrossCheckoutRecovery,
        &json!({"sourceRoot":lexical,"runtime":rt}),
    )
    .unwrap();
    assert_eq!(from_relative, from_absolute);
    assert!(
        std::path::Path::new(
            from_relative.operation_payload["sourceRoot"]
                .as_str()
                .unwrap()
        )
        .is_absolute()
    );
}

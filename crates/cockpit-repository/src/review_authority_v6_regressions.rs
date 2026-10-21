//! V6 reproductions exercise real business functions with the test-only port.
use super::lifecycle_regressions::{cross_fixture, prepare_archive};
use super::*;

fn after_begin(action: impl FnOnce() + 'static) {
    ACTION.with(|v| v.replace(Some(("after_begin_execution", Box::new(action)))));
}

#[test]
fn submit_never_attaches_old_receipt_to_new_approval_current_state() {
    let t = fixture();
    let p = Mock::new();
    let payload = json!({"op":"same-binding-different-approval"});
    let (old, proof) = approve_operation(&p, t.path(), HumanReviewOperation::Retire, &payload);
    let (new, _) = approve_operation(&p, t.path(), HumanReviewOperation::Retire, &payload);
    assert_ne!(old.request_id, new.request_id);
    assert!(
        submit_with_authority(t.path(), "WI-PORT", &old, &proof, p.as_ref()).is_err(),
        "receipt A cannot be reported as current using B's Active state"
    );
}

#[test]
fn completed_effect_hold_blocks_current_permission_preserves_fact_and_bytes() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let payload = json!({"program":"cargo","args":["test"]});
    let (r, _) = approve_operation(
        &p,
        t.path(),
        HumanReviewOperation::ProfileConfirmation,
        &payload,
    );
    let result = crate::confirm_profile_update(t.path(), "cargo", &["test".into()]).unwrap();
    let value = serde_json::to_value(result).unwrap();
    require_current_human_effect(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::ProfileConfirmation,
        &value,
    )
    .unwrap();
    let mirror = match p.current(&operation_key(&r.binding).unwrap()).unwrap() {
        AuthorityState::Consumed(v) => v.mirror_digest,
        _ => panic!("completed"),
    };
    let project = t.path().join(".ai/project.json");
    let bytes = std::fs::read(&project).unwrap();
    crate::record_review_correction(
        t.path(),
        "WI-PORT",
        &ReviewCorrectionRequest {
            schema_version: 1,
            repository_id: repository_id(t.path()).to_string(),
            work_item_id: "WI-PORT".into(),
            target_receipt_raw_digest: mirror,
            reason: "new local disavowal after committed effect".into(),
        },
    )
    .unwrap();
    assert!(
        require_current_human_effect(
            t.path(),
            "WI-PORT",
            HumanReviewOperation::ProfileConfirmation,
            &value
        )
        .is_err(),
        "new unresolved hold denies CURRENT permission even for Completed"
    );
    assert_eq!(std::fs::read(project).unwrap(), bytes);
    assert!(matches!(
        p.store
            .lock()
            .unwrap()
            .operation_runs
            .get(&operation_key(&r.binding).unwrap().to_string()),
        Some(Run::Completed(_))
    ));
}

#[test]
fn profile_rejects_context_replaced_after_begin_before_business_read() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let path = t.path().join(".ai/project.json");
    let mut replacement = read_json(&path).unwrap();
    replacement["profileVersion"] = json!(replacement["profileVersion"].as_u64().unwrap() + 10);
    let replacement_bytes = serde_json::to_vec_pretty(&replacement).unwrap();
    approve_operation(
        &p,
        t.path(),
        HumanReviewOperation::ProfileConfirmation,
        &json!({"program":"cargo","args":["test"]}),
    );
    let target = path.clone();
    let bytes = replacement_bytes.clone();
    after_begin(move || std::fs::write(target, bytes).unwrap());
    assert!(
        crate::confirm_profile_update(t.path(), "cargo", &["test".into()]).is_err(),
        "unapproved project B must not be confirmed under A's binding"
    );
    assert_eq!(std::fs::read(path).unwrap(), replacement_bytes);
}

#[test]
fn cross_rejects_valid_source_plan_replaced_after_begin_before_install() {
    let (source, _parent, destination, p, _installed, rt) = cross_fixture();
    approve_operation(
        &p,
        &destination,
        HumanReviewOperation::CrossCheckoutRecovery,
        &json!({"sourceRoot":source.path(),"runtime":rt}),
    );
    let source_path = source.path().to_path_buf();
    let dest = destination.clone();
    let runtime = rt.clone();
    after_begin(move || {
        std::fs::write(
            source_path.join("unreviewed-after-begin.txt"),
            b"different valid plan snapshot",
        )
        .unwrap();
        super::lifecycle_regressions::commit(
            &source_path,
            "valid later source commit after approval",
        );
        crate::plan_cross_checkout_closeout_recovery(&dest, &source_path, "WI-PORT", &runtime)
            .expect("replacement plan is independently valid");
    });
    assert!(
        crate::recover_cross_checkout_closeout(&destination, source.path(), "WI-PORT", &rt)
            .is_err(),
        "newly rebuilt unapproved plan must not be installed"
    );
    assert!(
        !destination
            .join(".ai/decisions/WI-PORT.close.json")
            .exists()
    );
}

fn recovery_input(
    root: &Path,
    rt: &cockpit_protocol::RuntimeContext,
    successor: Option<&str>,
) -> Value {
    let active = root.join(".ai/work-items/active");
    let c = read_json(&active.join("WI-PORT.contract.json")).unwrap();
    let s = read_json(&active.join("WI-PORT.summary.json")).unwrap();
    let o = read_json(&active.join("WI-PORT.outcome.json")).unwrap();
    let e = std::fs::read(active.join("WI-PORT.events.jsonl")).unwrap();
    json!({"schemaVersion":1,"decisionId":"work-item-recovery","decision":if successor.is_some(){"successor"}else{"retry"},"workItemId":"WI-PORT","repositoryId":repository_id(root),"predecessorWorkItemId":"WI-PORT","predecessorContractDigest":cockpit_protocol::digest_json(&c).unwrap(),"predecessorSummaryDigest":cockpit_protocol::digest_json(&s).unwrap(),"predecessorOutcomeDigest":cockpit_protocol::digest_json(&o).unwrap(),"predecessorEventsDigest":Digest::sha256_bytes(&e),"successorWorkItemId":successor,"runtimeVersion":rt.runtime_version,"runtimeDigest":rt.runtime_digest,"actor":"untrusted-fixture-audit","authoritySource":"simulated protected source","reason":"exact successor choice","evidenceRefs":[".ai/work-items/active/WI-PORT.outcome.json"],"policyRefs":[],"decidedAt":"2026-10-20T00:00:00Z","resumeCondition":"fresh actual verification required"})
}

#[test]
fn successor_uniqueness_survives_revocation_and_global_epoch_change() {
    for revoked in [false, true] {
        let t = fixture();
        let p = Mock::new();
        let _installed = Installed::new(p.clone());
        let rt = runtime();
        crate::checkpoint_work_item(t.path(), "WI-PORT").unwrap();
        assert!(crate::finish_work_item_with_runtime(t.path(), "WI-PORT", &rt).is_err());
        let first = recovery_input(t.path(), &rt, Some("WI-FIRST"));
        let (_, proof) = approve_operation(
            &p,
            t.path(),
            HumanReviewOperation::Recovery,
            &json!({"receipt":first,"runtime":rt}),
        );
        crate::record_recovery_decision(t.path(), "WI-PORT", &first, &rt)
            .expect("native first successor");
        let before = std::fs::read(t.path().join(".ai/decisions/WI-PORT.recovery.json")).unwrap();
        if revoked {
            let revoke = p.revoke_request(t.path(), "WI-PORT", &proof.approval_id);
            p.revoke(&revoke).unwrap();
        } else {
            p.store.lock().unwrap().epoch += 1;
        }
        let competing = recovery_input(t.path(), &rt, Some("WI-SECOND"));
        approve_operation(
            &p,
            t.path(),
            HumanReviewOperation::Recovery,
            &json!({"receipt":competing,"runtime":rt}),
        );
        let err = crate::record_recovery_decision(t.path(), "WI-PORT", &competing, &rt)
            .expect_err("history still forbids competing successor");
        assert!(err.to_string().contains("competing_successor"), "{err}");
        assert!(
            !t.path()
                .join(".ai/work-items/active/WI-SECOND.contract.json")
                .exists()
        );
        assert_eq!(
            std::fs::read(t.path().join(".ai/decisions/WI-PORT.recovery.json")).unwrap(),
            before
        );
    }
}

#[test]
fn historical_finalization_success_authenticates_actual_receipt_not_api_projection() {
    let t = fixture();
    let root = t.path();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let rt = runtime();
    let context = cockpit_protocol::ResourceFinalizationContext {
        branch: "feature/synthetic-resource".into(),
        worktree: "/tmp/fixture-only-already-removed".into(),
        base_branch: "main".into(),
        base_remote: "origin".into(),
        provider: "github".into(),
        pull_request: "https://github.com/example/fixture/pull/191".into(),
    };
    crate::plan_resource_finalization(root, "WI-PORT", &context).unwrap();
    crate::preflight_work_item_with_runtime(
        root,
        &root.join(".ai/work-items/active/WI-PORT.contract.json"),
        &rt,
    )
    .unwrap();
    approve_operation(
        &p,
        root,
        HumanReviewOperation::PreflightReview,
        &preflight_payload(),
    );
    prepare_archive(root, &rt);
    let archived = root.join(".ai/work-items/archive/WI-PORT.contract.json");
    let contract = read_json(&archived).unwrap();
    let legacy = cockpit_protocol::RuntimeContext {
        runtime_version: "0.2.33".into(),
        protocol_version: 1,
        runtime_digest: Digest::sha256_bytes(b"legacy-synthetic"),
    };
    let receipt = json!({"schemaVersion":1,"receiptId":"legacy-finalize","operationId":"legacy-operation","repositoryId":repository_id(root),"workItemId":"WI-PORT","runtimeVersion":legacy.runtime_version,"runtimeDigest":legacy.runtime_digest,"provider":context.provider,"pullRequest":{"number":191,"url":context.pull_request,"headRevision":"head-191","baseBranch":"main","baseRemote":"origin","baseRevision":contract["baseRevision"],"mergeCommit":"merge-191"},"branch":{"name":context.branch,"remote":"origin","headRevision":"head-191"},"worktree":{"worktreeId":"fixture-worktree","path":context.worktree,"branch":context.branch,"headRevision":"head-191"},"before":{"pullRequest":"merged","branch":"present","worktree":"clean"},"after":{"pullRequest":"merged","branch":"deleted","worktree":"removed"},"result":{"disposition":"deleted","failureCodes":[],"unknownCodes":[]},"actor":"untrusted-fixture-audit","authoritySource":"legacy fixture","reason":"immutable historical receipt","timestamp":"2026-08-23T00:00:00Z","contractDigest":Digest::sha256_bytes(&std::fs::read(&archived).unwrap()),"resourceContext":context});
    let input = root.join("legacy-fixture-input.json");
    std::fs::write(&input, serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
    crate::record_resource_finalization(root, "WI-PORT", &input, &legacy)
        .expect("native legacy predecessor");
    let plan = crate::historical_finalization_recovery_plan(root, "WI-PORT", &rt, None).unwrap();
    let mut recovery = plan["suggestedRecovery"].clone();
    recovery["schemaVersion"] = json!(1);
    recovery["recoveryId"] = json!("actual-effect-recovery");
    recovery["kind"] = json!("historical_finalization_recovery");
    recovery["actor"] = json!("untrusted-fixture-audit");
    recovery["authoritySource"] = json!("simulated protected source");
    recovery["reason"] = json!("classify immutable receipt");
    recovery["decidedAt"] = json!("2026-10-20T00:00:00Z");
    let recovery_input = root.join("recovery-fixture-input.json");
    let bytes = serde_json::to_vec_pretty(&recovery).unwrap();
    std::fs::write(&recovery_input, &bytes).unwrap();
    approve_operation(
        &p,
        root,
        HumanReviewOperation::HistoricalFinalizationRecovery,
        &json!({"inputRawDigest":Digest::sha256_bytes(&bytes),"runtime":rt}),
    );
    let result =
        crate::record_historical_finalization_recovery(root, "WI-PORT", &recovery_input, &rt)
            .expect("native recovery success");
    assert_eq!(result["state"], "recorded");
    let recorded = read_json(&root.join(".ai/decisions/WI-PORT.finalize-recovery.json")).unwrap();
    require_current_human_effect(
        root,
        "WI-PORT",
        HumanReviewOperation::HistoricalFinalizationRecovery,
        &recorded,
    )
    .expect("authenticate exact business receipt");
    let verified = crate::verify_resource_finalization(root, "WI-PORT", &rt)
        .expect("native loader authenticates the committed recovery");
    assert_eq!(verified["state"], "verified");
}

#[test]
fn submit_replacement_after_activate_cannot_report_old_receipt_as_current() {
    let t = fixture();
    let p = Mock::new();
    let payload = json!({"op":"replace-during-submit"});
    let binding =
        human_operation_binding(t.path(), "WI-PORT", HumanReviewOperation::Retire, &payload)
            .unwrap();
    let request = p.issue(&binding).unwrap();
    let proof = p.approve(&request, false);
    let root = t.path().to_path_buf();
    let next = p.clone();
    let args = payload.clone();
    ACTION.with(|v| {
        v.replace(Some((
            "after_activate",
            Box::new(move || {
                approve_operation(&next, &root, HumanReviewOperation::Retire, &args);
            }),
        )))
    });
    let err = submit_with_authority(t.path(), "WI-PORT", &request, &proof, p.as_ref()).unwrap_err();
    assert!(
        err.to_string().contains("not_current_exact_receipt"),
        "{err}"
    );
    assert_eq!(
        current_with_authority(
            t.path(),
            "WI-PORT",
            &HumanReviewOperation::Retire,
            &payload,
            p.as_ref()
        )
        .unwrap(),
        CurrentReviewAuthorityState::Active
    );
}

fn append_hold(root: &Path, mirror: Digest) {
    crate::record_review_correction(
        root,
        "WI-PORT",
        &ReviewCorrectionRequest {
            schema_version: 1,
            repository_id: repository_id(root).to_string(),
            work_item_id: "WI-PORT".into(),
            target_receipt_raw_digest: mirror,
            reason: "disavow current permission, preserve completed history".into(),
        },
    )
    .unwrap();
}

#[test]
fn immutable_effect_fact_survives_each_current_permission_rejection() {
    for change in ["hold", "revoke", "epoch", "deny", "replacement"] {
        let t = fixture();
        let p = Mock::new();
        let _installed = Installed::new(p.clone());
        let payload = json!({"program":"cargo","args":["test"]});
        let (request, proof) = approve_operation(
            &p,
            t.path(),
            HumanReviewOperation::ProfileConfirmation,
            &payload,
        );
        let result = crate::confirm_profile_update(t.path(), "cargo", &["test".into()]).unwrap();
        let value = serde_json::to_value(result).unwrap();
        let project = t.path().join(".ai/project.json");
        let original = std::fs::read(&project).unwrap();
        require_current_human_effect(
            t.path(),
            "WI-PORT",
            HumanReviewOperation::ProfileConfirmation,
            &value,
        )
        .unwrap();
        match change {
            "hold" => {
                let mirror = match p
                    .current(&operation_key(&request.binding).unwrap())
                    .unwrap()
                {
                    AuthorityState::Consumed(r) => r.mirror_digest,
                    _ => panic!("completed"),
                };
                append_hold(t.path(), mirror);
            }
            "revoke" => {
                let r = p.revoke_request(t.path(), "WI-PORT", &proof.approval_id);
                p.revoke(&r).unwrap();
            }
            "epoch" => p.store.lock().unwrap().epoch += 1,
            "deny" => {
                let b = human_operation_binding(
                    t.path(),
                    "WI-PORT",
                    HumanReviewOperation::ProfileConfirmation,
                    &payload,
                )
                .unwrap();
                let r = p.issue(&b).unwrap();
                let proof = p.approve(&r, false);
                p.store
                    .lock()
                    .unwrap()
                    .records
                    .get_mut(&proof.approval_id)
                    .unwrap()
                    .denied = true;
            }
            "replacement" => {
                approve_operation(
                    &p,
                    t.path(),
                    HumanReviewOperation::ProfileConfirmation,
                    &payload,
                );
            }
            _ => unreachable!(),
        }
        assert!(
            require_current_human_effect(
                t.path(),
                "WI-PORT",
                HumanReviewOperation::ProfileConfirmation,
                &value
            )
            .is_err(),
            "{change}"
        );
        assert!(
            has_historical_human_effect(
                t.path(),
                "WI-PORT",
                HumanReviewOperation::ProfileConfirmation,
                &value
            )
            .unwrap(),
            "{change}"
        );
        assert_eq!(std::fs::read(project).unwrap(), original, "{change}");
        assert_eq!(p.store.lock().unwrap().executions, 1);
    }
}

#[test]
fn native_retry_hold_denies_current_authority_without_erasing_committed_receipt() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let rt = runtime();
    crate::checkpoint_work_item(t.path(), "WI-PORT").unwrap();
    assert!(crate::finish_work_item_with_runtime(t.path(), "WI-PORT", &rt).is_err());
    let receipt = recovery_input(t.path(), &rt, None);
    let (request, _) = approve_operation(
        &p,
        t.path(),
        HumanReviewOperation::Recovery,
        &json!({"receipt":receipt,"runtime":rt}),
    );
    let recorded = crate::record_recovery_decision(t.path(), "WI-PORT", &receipt, &rt).unwrap();
    assert!(
        current_recovery_decision(t.path(), "WI-PORT", Some(&rt))
            .unwrap()
            .is_some()
    );
    let path = t.path().join(".ai/decisions/WI-PORT.recovery.json");
    let bytes = std::fs::read(&path).unwrap();
    let mirror = match p
        .current(&operation_key(&request.binding).unwrap())
        .unwrap()
    {
        AuthorityState::Consumed(r) => r.mirror_digest,
        _ => panic!("completed"),
    };
    append_hold(t.path(), mirror);
    assert!(current_recovery_decision(t.path(), "WI-PORT", Some(&rt)).is_err());
    assert!(
        crate::load_recovery_decision(t.path(), "WI-PORT", Some(&rt))
            .unwrap()
            .is_some()
    );
    assert!(
        has_historical_human_effect(
            t.path(),
            "WI-PORT",
            HumanReviewOperation::Recovery,
            &recorded
        )
        .unwrap()
    );
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}

#[test]
fn retirement_rejects_contract_changed_by_writer_after_begin_before_lock() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let rt = runtime();
    let request = retirement();
    let path = t.path().join(".ai/work-items/active/WI-PORT.contract.json");
    let mut changed = read_json(&path).unwrap();
    changed["scope"] = json!(["different/**"]);
    let bytes = serde_json::to_vec_pretty(&changed).unwrap();
    approve_operation(
        &p,
        t.path(),
        HumanReviewOperation::Retire,
        &json!({"request":request,"runtime":rt}),
    );
    let root = t.path().to_path_buf();
    let target = path.clone();
    let input = bytes.clone();
    after_begin(move || {
        let _writer = crate::acquire_lifecycle_lock(&root, "WI-PORT").unwrap();
        std::fs::write(target, input).unwrap();
    });
    assert!(
        crate::retire_active_work_item_with_runtime(t.path(), "WI-PORT", &request, &rt).is_err(),
        "approval for C1 cannot retire a cooperating writer's C2"
    );
    assert_eq!(std::fs::read(path).unwrap(), bytes);
    assert!(
        !t.path()
            .join(".ai/decisions/WI-PORT.retirement.json")
            .exists()
    );
}

fn cooperating_empty_commit(root: &Path) {
    let _writer = crate::acquire_lifecycle_lock(root, "WI-PORT").unwrap();
    super::lifecycle_regressions::git(
        root,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "--allow-empty",
            "-qm",
            "cooperating writer advances HEAD after begin",
        ],
    );
}
#[test]
fn cooperating_writer_changes_git_before_successor_business_is_rejected() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let rt = runtime();
    crate::checkpoint_work_item(t.path(), "WI-PORT").unwrap();
    assert!(crate::finish_work_item_with_runtime(t.path(), "WI-PORT", &rt).is_err());
    let receipt = recovery_input(t.path(), &rt, Some("WI-NEW-SUCCESSOR"));
    approve_operation(
        &p,
        t.path(),
        HumanReviewOperation::Recovery,
        &json!({"receipt":receipt,"runtime":rt}),
    );
    let root = t.path().to_path_buf();
    after_begin(move || cooperating_empty_commit(&root));
    assert!(
        crate::record_recovery_decision(t.path(), "WI-PORT", &receipt, &rt).is_err(),
        "R1 approval cannot create a successor on R2"
    );
    assert!(
        !t.path()
            .join(".ai/work-items/active/WI-NEW-SUCCESSOR.contract.json")
            .exists()
    );
}
#[test]
fn cooperating_writer_changes_git_before_close_business_is_rejected() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let rt = runtime();
    prepare_archive(t.path(), &rt);
    super::lifecycle_regressions::commit(t.path(), "archive before bound close");
    let decision = super::lifecycle_regressions::close_decision();
    approve_operation(
        &p,
        t.path(),
        HumanReviewOperation::Close,
        &json!({"decision":decision,"runtime":Some(&rt)}),
    );
    let root = t.path().to_path_buf();
    after_begin(move || cooperating_empty_commit(&root));
    assert!(
        crate::close_work_item_with_structured_decision_and_runtime(
            t.path(),
            "WI-PORT",
            &decision,
            &rt
        )
        .is_err(),
        "R1 approval cannot close with a new HEAD2 cleanup target"
    );
    assert!(!t.path().join(".ai/decisions/WI-PORT.close.json").exists());
}

#[test]
fn cooperating_writer_changes_cleanup_branch_at_same_head_is_rejected() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let rt = runtime();
    prepare_archive(t.path(), &rt);
    super::lifecycle_regressions::commit(t.path(), "archive before exact branch close");
    let decision = super::lifecycle_regressions::close_decision();
    approve_operation(
        &p,
        t.path(),
        HumanReviewOperation::Close,
        &json!({"decision":decision,"runtime":Some(&rt)}),
    );
    let root = t.path().to_path_buf();
    after_begin(move || {
        let _writer = crate::acquire_lifecycle_lock(&root, "WI-PORT").unwrap();
        super::lifecycle_regressions::git(&root, &["switch", "-qc", "unapproved-cleanup-branch"]);
    });
    assert!(
        crate::close_work_item_with_structured_decision_and_runtime(
            t.path(),
            "WI-PORT",
            &decision,
            &rt
        )
        .is_err(),
        "same HEAD cannot authorize a different cleanup branch"
    );
    assert!(!t.path().join(".ai/decisions/WI-PORT.close.json").exists());
}

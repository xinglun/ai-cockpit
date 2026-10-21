//! Test-only candidate regressions. Simulated authority is not deployed B.
use super::*;

fn reference(p: &Mock, r: &HumanReviewRequest) -> AuthorityReference {
    match p.current(&operation_key(&r.binding).unwrap()).unwrap() {
        AuthorityState::Active(reference) | AuthorityState::Consumed(reference) => reference,
        _ => panic!("expected published reference"),
    }
}
fn advance_generation(p: &Arc<Mock>, root: &Path) {
    let (_, proof) = approve_operation(
        p,
        root,
        HumanReviewOperation::ProfileConfirmation,
        &json!({"program":"true","args":[]}),
    );
    let request = p.revoke_request(root, "WI-PORT", &proof.approval_id);
    p.revoke(&request).unwrap();
}
#[test]
fn global_epoch_blocks_stale_pending_grant() {
    let t = fixture();
    let p = Mock::new();
    let payload = json!({"op":"pending-before-generation"});
    let r = p
        .issue(
            &human_operation_binding(t.path(), "WI-PORT", HumanReviewOperation::Retire, &payload)
                .unwrap(),
        )
        .unwrap();
    let proof = p.approve(&r, false);
    advance_generation(&p, t.path());
    let error = submit_with_authority(t.path(), "WI-PORT", &r, &proof, p.as_ref())
        .expect_err("stale source generation cannot publish a fresh grant");
    assert!(
        error.to_string().contains("authority_epoch_stale"),
        "{error}"
    );
    assert_eq!(
        p.store.lock().unwrap().unique_consumptions,
        1,
        "only the unrelated generation-changing operation was consumed"
    );
}
#[test]
fn global_epoch_blocks_new_start_but_keeps_completed_fact() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let pending = json!({"op":"not-yet-started"});
    let (r, _) = approve_operation(&p, t.path(), HumanReviewOperation::Retire, &pending);
    let old = reference(&p, &r);
    let completed = json!({"op":"already-completed"});
    approve_operation(&p, t.path(), HumanReviewOperation::Retire, &completed);
    let value: Value = execute_human_operation(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::Retire,
        &completed,
        || Ok(json!({"historical":"completed"})),
    )
    .unwrap();
    advance_generation(&p, t.path());
    assert!(
        matches!(
            p.begin_execution(
                &operation_key(&r.binding).unwrap(),
                &old.receipt,
                Some(&r.binding)
            ),
            Err("authority_epoch_stale")
        ),
        "current global epoch must be checked atomically before Started"
    );
    assert_eq!(
        current_with_authority(
            t.path(),
            "WI-PORT",
            &HumanReviewOperation::Retire,
            &pending,
            p.as_ref()
        )
        .unwrap(),
        CurrentReviewAuthorityState::Invalid
    );
    assert!(
        require_current_human_effect(t.path(), "WI-PORT", HumanReviewOperation::Retire, &value)
            .is_err(),
        "old completion is not fresh execution authority"
    );
    let replay: Value = execute_human_operation(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::Retire,
        &completed,
        || panic!("historical operation rerun"),
    )
    .unwrap();
    assert_eq!(replay, value);
}
#[test]
fn old_receipt_and_none_cannot_start_and_none_only_replays_a_terminal_result() {
    let t = fixture();
    let p = Mock::new();
    let payload = json!({"op":"receipt-none"});
    let (r, _) = approve_operation(&p, t.path(), HumanReviewOperation::Retire, &payload);
    let reference = reference(&p, &r);
    let key = operation_key(&r.binding).unwrap();
    let mut foreign = reference.receipt.clone();
    foreign.authority_epoch += 1;
    assert!(matches!(
        p.begin_execution(&key, &foreign, Some(&r.binding)),
        Err("review_execution_binding_mismatch")
    ));
    assert!(matches!(
        p.begin_execution(&key, &reference.receipt, None),
        Err("review_execution_current_binding_mismatch")
    ));
    assert_eq!(p.store.lock().unwrap().executions, 0);
    let _: Value = execute_with_authority(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::Retire,
        &payload,
        p.as_ref(),
        || Ok(json!("done")),
    )
    .unwrap();
    assert!(matches!(
        p.begin_execution(&key, &reference.receipt, None),
        Ok(ExecutionBegin::Replay(_))
    ));
    assert_eq!(p.store.lock().unwrap().executions, 1);
}
struct SwitchOnCurrent {
    inner: Arc<Mock>,
    key: String,
    next: String,
    switch_after: usize,
    reads: AtomicUsize,
}
impl ProtectedAuthority for SwitchOnCurrent {
    fn consume(
        &self,
        r: &HumanReviewRequest,
        p: &HumanApprovalSubmission,
    ) -> Result<AuthorityObservation, &'static str> {
        self.inner.consume(r, p)
    }
    fn now_millis(&self) -> Result<i64, &'static str> {
        self.inner.now_millis()
    }
    fn current(&self, key: &Digest) -> Result<AuthorityState, &'static str> {
        let state = self.inner.current(key)?;
        if self.reads.fetch_add(1, Ordering::SeqCst) + 1 == self.switch_after {
            self.inner
                .store
                .lock()
                .unwrap()
                .latest
                .insert(self.key.clone(), self.next.clone());
        }
        Ok(state)
    }
    fn begin_execution(
        &self,
        key: &Digest,
        r: &HumanApprovalReceipt,
        b: Option<&HumanReviewBinding>,
    ) -> Result<ExecutionBegin, &'static str> {
        self.inner.begin_execution(key, r, b)
    }
    fn finish_execution(&self, token: &str, result: &ExecutionResult) -> Result<(), &'static str> {
        self.inner.finish_execution(token, result)
    }
}
#[test]
fn approval_replacement_between_each_current_read_rejects_old_invocation() {
    for boundary in 1..=4 {
        let t = fixture();
        let p = Mock::new();
        let payload = json!({"op":"replace-before-begin"});
        let (a, proof_a) = approve_operation(&p, t.path(), HumanReviewOperation::Retire, &payload);
        let (_, proof_b) = approve_operation(&p, t.path(), HumanReviewOperation::Retire, &payload);
        let key = operation_key(&a.binding).unwrap().to_string();
        p.store
            .lock()
            .unwrap()
            .latest
            .insert(key.clone(), proof_a.approval_id);
        let port = SwitchOnCurrent {
            inner: p.clone(),
            key,
            next: proof_b.approval_id,
            switch_after: boundary,
            reads: AtomicUsize::new(0),
        };
        let result = execute_with_authority::<Value, _>(
            t.path(),
            "WI-PORT",
            HumanReviewOperation::Retire,
            &payload,
            &port,
            || panic!("old receipt executed after replacement"),
        );
        let error = result.unwrap_err();
        assert!(
            error
                .to_string()
                .contains("review_execution_binding_mismatch"),
            "boundary {boundary}: {error}"
        );
        assert_eq!(p.store.lock().unwrap().executions, 0, "boundary {boundary}");
    }
}
#[derive(serde::Deserialize)]
struct CannotSerialize;
impl serde::Serialize for CannotSerialize {
    fn serialize<S: serde::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom("fixture_serialization_failure"))
    }
}
#[test]
fn serializer_failure_after_real_effect_never_reexecutes() {
    let t = fixture();
    let p = Mock::new();
    let payload = json!({"op":"serializer-failure"});
    let (r, _) = approve_operation(&p, t.path(), HumanReviewOperation::Retire, &payload);
    let marker = t.path().join(".ai/serializer-side-effect");
    let first = execute_with_authority::<CannotSerialize, _>(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::Retire,
        &payload,
        p.as_ref(),
        || {
            std::fs::write(&marker, b"effect-applied").unwrap();
            Ok(CannotSerialize)
        },
    );
    assert!(
        first
            .err()
            .unwrap()
            .to_string()
            .contains("human_execution_result_invalid")
    );
    assert_eq!(std::fs::read(&marker).unwrap(), b"effect-applied");
    assert!(matches!(
        p.current(&operation_key(&r.binding).unwrap()).unwrap(),
        AuthorityState::Running(_)
    ));
    assert!(
        execute_with_authority::<CannotSerialize, _>(
            t.path(),
            "WI-PORT",
            HumanReviewOperation::Retire,
            &payload,
            p.as_ref(),
            || panic!("serializer error rerun")
        )
        .is_err()
    );
    assert_eq!(p.store.lock().unwrap().executions, 1);
}
#[test]
fn actual_write_error_after_real_effect_is_terminal_and_never_retryable() {
    let t = fixture();
    let p = Mock::new();
    let payload = json!({"op":"partial-disk-error"});
    let (r, _) = approve_operation(&p, t.path(), HumanReviewOperation::Retire, &payload);
    let marker = t.path().join(".ai/partial-side-effect");
    let failed = t.path().join(".ai/nonexistent-parent/file");
    let first = execute_with_authority::<Value, _>(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::Retire,
        &payload,
        p.as_ref(),
        || {
            std::fs::write(&marker, b"effect-applied").unwrap();
            let source = std::fs::write(&failed, b"never-written").unwrap_err();
            Err(ObserverError::Read {
                path: failed.clone(),
                source,
            })
        },
    );
    assert!(first.is_err());
    assert!(matches!(
        p.current(&operation_key(&r.binding).unwrap()).unwrap(),
        AuthorityState::Consumed(_)
    ));
    assert_eq!(std::fs::read(&marker).unwrap(), b"effect-applied");
    assert!(
        execute_with_authority::<Value, _>(
            t.path(),
            "WI-PORT",
            HumanReviewOperation::Retire,
            &payload,
            p.as_ref(),
            || panic!("partial error retried")
        )
        .is_err()
    );
    assert_eq!(p.store.lock().unwrap().executions, 1);
}
#[test]
fn newer_denial_blocks_current_authority_without_erasing_the_completed_fact() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let args = vec!["test".into()];
    let payload = json!({"program":"cargo","args":args});
    let (r, _) = approve_operation(
        &p,
        t.path(),
        HumanReviewOperation::ProfileConfirmation,
        &payload,
    );
    let first = crate::confirm_profile_update(t.path(), "cargo", &args).unwrap();
    let project = t.path().join(".ai/project.json");
    let bytes = std::fs::read(&project).unwrap();
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
    p.store
        .lock()
        .unwrap()
        .records
        .get_mut(&proof.approval_id)
        .unwrap()
        .denied = true;
    assert_eq!(
        current_with_authority(
            t.path(),
            "WI-PORT",
            &HumanReviewOperation::ProfileConfirmation,
            &payload,
            p.as_ref()
        )
        .unwrap(),
        CurrentReviewAuthorityState::Denied
    );
    assert!(crate::confirm_profile_update(t.path(), "cargo", &args).is_err());
    assert_eq!(std::fs::read(project).unwrap(), bytes);
    let store = p.store.lock().unwrap();
    match store
        .operation_runs
        .get(&operation_key(&r.binding).unwrap().to_string())
        .unwrap()
    {
        Run::Completed(result) => {
            assert!(result.success);
            assert_eq!(result.value, serde_json::to_value(first).unwrap());
        }
        _ => panic!("historical completion disappeared"),
    }
}

#[test]
fn epoch_change_between_consume_and_activate_cannot_publish_current_permission() {
    let t = fixture();
    let p = Mock::new();
    let payload = json!({"op":"consumed-before-generation"});
    let r = p
        .issue(
            &human_operation_binding(t.path(), "WI-PORT", HumanReviewOperation::Retire, &payload)
                .unwrap(),
        )
        .unwrap();
    let proof = p.approve(&r, false);
    p.consume(&r, &proof).unwrap();
    let receipt = p.store.lock().unwrap().records[&proof.approval_id].receipt();
    let bytes = serde_json::to_vec_pretty(&serde_json::to_value(&receipt).unwrap()).unwrap();
    advance_generation(&p, t.path());
    assert!(matches!(
        p.activate(&receipt, &Digest::sha256_bytes(&bytes)),
        Err("authority_epoch_stale")
    ));
    assert!(
        p.store.lock().unwrap().records[&proof.approval_id]
            .mirror
            .is_none()
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
        CurrentReviewAuthorityState::Invalid
    );
}

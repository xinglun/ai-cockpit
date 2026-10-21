//! Compiled only for repository unit tests. This is simulated domain behavior,
//! not authentication of a real human, host authority or deployment boundary.
use super::*;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    process::Command,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
const TEST_SENTINEL: &str = "AI_COCKPIT_TEST_AUTHORITY_SENTINEL_V4";
type TestAction = Option<(&'static str, Box<dyn FnOnce()>)>;
thread_local! {
    static INSTALLED:RefCell<Option<Arc<dyn ProtectedAuthority>>>=RefCell::new(None);
    static ACTION:RefCell<TestAction>=RefCell::new(None);
    static FAILURE:RefCell<Option<String>>=const { RefCell::new(None) };
}
pub(super) fn installed() -> Option<Arc<dyn ProtectedAuthority>> {
    INSTALLED.with(|v| v.borrow().clone())
}
pub(crate) fn hit(stage: &str) -> Result<(), &'static str> {
    let action = ACTION.with(|v| {
        let mut v = v.borrow_mut();
        if v.as_ref().is_some_and(|(name, _)| *name == stage) {
            v.take()
        } else {
            None
        }
    });
    if let Some((_, action)) = action {
        action();
    }
    FAILURE.with(|v| {
        let mut x = v.borrow_mut();
        if x.as_deref() == Some(stage) {
            *x = None;
            Err("injected_candidate_interruption")
        } else {
            Ok(())
        }
    })
}
// Installation and restoration affect this thread's port. The Arc itself may
// be shared, but the lifetime guard must stay on the installing thread.
struct Installed(
    Option<Arc<dyn ProtectedAuthority>>,
    std::marker::PhantomData<std::rc::Rc<()>>,
);
impl Installed {
    fn new(value: Arc<dyn ProtectedAuthority>) -> Self {
        Self(
            INSTALLED.with(|v| v.replace(Some(value))),
            std::marker::PhantomData,
        )
    }
}
impl Drop for Installed {
    fn drop(&mut self) {
        INSTALLED.with(|v| {
            v.replace(self.0.take());
        });
        ACTION.with(|v| {
            v.replace(None);
        });
        FAILURE.with(|v| {
            v.replace(None);
        });
    }
}
#[test]
fn installed_authority_guards_cannot_move_or_be_shared_across_threads() {
    // Ambiguous trait resolution fails to compile if either guard regains
    // Send or Sync; no runtime thread can move its TLS restoration duty.
    trait AmbiguousIfSend<A> {
        fn check() {}
    }
    impl<T: ?Sized> AmbiguousIfSend<()> for T {}
    impl<T: ?Sized + Send> AmbiguousIfSend<u8> for T {}
    trait AmbiguousIfSync<A> {
        fn check() {}
    }
    impl<T: ?Sized> AmbiguousIfSync<()> for T {}
    impl<T: ?Sized + Sync> AmbiguousIfSync<u8> for T {}
    let _ = <Installed as AmbiguousIfSend<_>>::check;
    let _ = <Installed as AmbiguousIfSync<_>>::check;
    let _ = <TestAuthority as AmbiguousIfSend<_>>::check;
    let _ = <TestAuthority as AmbiguousIfSync<_>>::check;
}
/// Explicit test-only source/ledger lifetime for native domain fixtures.
/// It cannot be selected by production, an environment variable or wire JSON.
pub(crate) struct TestAuthority {
    port: Arc<Mock>,
    _installed: Installed,
}
/// Shares only this test's explicit source/ledger. Each receiving thread must
/// install its own !Send/!Sync lifetime guard; sharing grants no approval.
#[derive(Clone)]
pub(crate) struct SharedTestAuthority {
    port: Arc<Mock>,
}
impl SharedTestAuthority {
    pub(crate) fn install(self) -> TestAuthority {
        let installed = Installed::new(self.port.clone());
        TestAuthority {
            port: self.port,
            _installed: installed,
        }
    }
}
impl TestAuthority {
    pub(crate) fn install() -> Self {
        let port = Mock::new();
        let installed = Installed::new(port.clone());
        Self {
            port,
            _installed: installed,
        }
    }
    pub(crate) fn share(&self) -> SharedTestAuthority {
        SharedTestAuthority {
            port: self.port.clone(),
        }
    }
    pub(crate) fn execution_count(&self) -> usize {
        self.port.store.lock().unwrap().executions
    }
    pub(crate) fn on_next_stage(&self, stage: &'static str, action: impl FnOnce() + 'static) {
        ACTION.with(|slot| {
            assert!(slot.borrow().is_none());
            slot.replace(Some((stage, Box::new(action))));
        });
    }
    /// Fault injection affects only this test's protected historical lookup.
    /// Public JSON and production authority construction cannot select it.
    pub(crate) fn set_historical_preflight_fault(&self, mode: Option<&str>, completed_at: &str) {
        self.port.store.lock().unwrap().historical_fault = mode.map(|mode| match mode {
            "missing" => HistoricalFault::Missing,
            "unavailable" => HistoricalFault::Unavailable,
            "late" => HistoricalFault::DecisionTime(
                (chrono::DateTime::parse_from_rfc3339(completed_at).unwrap()
                    + chrono::Duration::nanoseconds(1))
                .to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
            ),
            _ => panic!("unsupported private historical fault"),
        });
    }
    pub(crate) fn revoke_approval(&self, root: &Path, id: &str, request: &HumanReviewRequest) {
        let approval_id = self
            .port
            .store
            .lock()
            .unwrap()
            .records
            .values()
            .find(|record| record.request == *request)
            .unwrap()
            .proof
            .approval_id
            .clone();
        let revocation = self.port.revoke_request(root, id, &approval_id);
        record_review_revocation(root, id, &revocation).unwrap();
    }
    pub(crate) fn approve(
        &self,
        root: &Path,
        id: &str,
        operation: HumanReviewOperation,
        payload: &Value,
    ) -> HumanReviewRequest {
        let binding = human_operation_binding(root, id, operation, payload).unwrap();
        let request = self.port.issue(&binding).unwrap();
        let proof = self.port.approve(&request, false);
        submit_with_authority(root, id, &request, &proof, self.port.as_ref()).unwrap();
        request
    }
    /// Build the legacy transaction's historical shape, separately from its
    /// CURRENT authority, which comes from the explicit private port above.
    /// This synthetic projection never authenticates a real human and does
    /// not claim the production v2-to-v1 material bridge exists.
    pub(crate) fn approve_preflight_with_synthetic_legacy_projection(&self, root: &Path, id: &str) {
        let request = self.approve(
            root,
            id,
            HumanReviewOperation::PreflightReview,
            &preflight_payload(),
        );
        assert_eq!(
            current_review_authority_state(root, id).unwrap(),
            CurrentReviewAuthorityState::Active
        );
        let binding = &request.binding;
        let projection = json!({
            "schemaVersion": 1, "decisionId": "contract-preflight-review", "decision": "confirm_review",
            "repositoryId": binding.repository_id, "workItemId": id,
            "contractDigest": binding.contract_digest, "preflightDecisionDigest": binding.preflight_decision_digest,
            "repositorySnapshotDigest": binding.repository_snapshot_digest,
            "recordedAt": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
            "recordedBy": "synthetic-legacy-transaction-projection",
            "reason": "Test historical structure only; current authority is the separate simulated private port."
        });
        let digest = cockpit_protocol::digest_json(&projection)
            .unwrap()
            .to_string();
        let path = root.join(format!(
            ".ai/decisions/{id}.preflight-review.{}.json",
            digest.strip_prefix("sha256:").unwrap()
        ));
        assert!(!path.exists());
        std::fs::write(path, serde_json::to_vec_pretty(&projection).unwrap()).unwrap();
        let path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
        let mut summary = read_json(&path).unwrap();
        summary["decisionEvidence"] = projection;
        std::fs::write(path, serde_json::to_vec_pretty(&summary).unwrap()).unwrap();
    }
}
#[test]
fn shared_test_authority_requires_explicit_thread_installation() {
    let authority = TestAuthority::install();
    let parent_port = installed().unwrap();
    let shared = authority.share();
    std::thread::spawn(move || {
        assert!(installed().is_none());
        {
            let _guard = shared.install();
            assert!(Arc::ptr_eq(&installed().unwrap(), &parent_port));
        }
        assert!(installed().is_none());
    })
    .join()
    .unwrap();
    assert!(installed().is_some());
}
#[derive(Clone)]
enum Run {
    Running(String),
    Completed(ExecutionResult),
}
#[derive(Clone)]
struct Record {
    request: HumanReviewRequest,
    proof: HumanApprovalSubmission,
    human: bool,
    denied: bool,
    consumed: bool,
    resolve_hold: bool,
    mirror: Option<Digest>,
    revocation: Option<HumanRevocationReceipt>,
    approved_at_utc: String,
}
enum HistoricalFault {
    Missing,
    Unavailable,
    DecisionTime(String),
}
impl Record {
    fn receipt(&self) -> HumanApprovalReceipt {
        HumanApprovalReceipt {
            schema_version: 2,
            request: self.request.clone(),
            request_digest: self.request.digest().unwrap(),
            approval_id: self.proof.approval_id.clone(),
            source_event_id: self.proof.source_event_id.clone(),
            authenticated_subject: "fixture-authenticated-human".into(),
            authority_epoch: self.request.authority_epoch,
            resolved_local_hold: if self.resolve_hold {
                self.request.binding.local_hold_digest.clone()
            } else {
                None
            },
        }
    }
}
#[derive(Default)]
struct Store {
    next: u64,
    epoch: u64,
    issued: BTreeMap<String, HumanReviewRequest>,
    records: BTreeMap<String, Record>,
    latest: BTreeMap<String, String>,
    operation_runs: BTreeMap<String, Run>,
    // Preserve the approval that actually Started this operation; a later
    // approval must never inherit an earlier committed effect as its own.
    operation_approvals: BTreeMap<String, HumanApprovalReceipt>,
    revoke_sources: BTreeMap<String, String>,
    unique_consumptions: usize,
    executions: usize,
    fail_method: Option<&'static str>,
    mutate_after_consume: Option<std::path::PathBuf>,
    historical_fault: Option<HistoricalFault>,
}
struct Mock {
    store: Mutex<Store>,
    now: i64,
}
impl Mock {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            store: Mutex::new(Store {
                epoch: 1,
                ..Default::default()
            }),
            now: chrono::Utc::now().timestamp_millis(),
        })
    }
    fn approve(&self, request: &HumanReviewRequest, resolve_hold: bool) -> HumanApprovalSubmission {
        let mut s = self.store.lock().unwrap();
        assert_eq!(s.issued.get(&request.request_id), Some(request));
        let proof = HumanApprovalSubmission {
            schema_version: 2,
            kind: HumanApprovalSubmissionKind::HumanApprovalSubmission,
            request_digest: request.digest().unwrap(),
            issuer: request.issuer.clone(),
            approval_id: format!("grant-{}", request.request_id),
            source_event_id: format!("source-{}", request.request_id),
            proof_handle: format!("{TEST_SENTINEL}-{}", request.request_id),
        };
        let key = operation_key(&request.binding).unwrap().to_string();
        s.latest.insert(key, proof.approval_id.clone());
        s.records.insert(
            proof.approval_id.clone(),
            Record {
                request: request.clone(),
                proof: proof.clone(),
                human: true,
                denied: false,
                consumed: false,
                resolve_hold,
                mirror: None,
                revocation: None,
                approved_at_utc: chrono::Utc::now()
                    .to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
            },
        );
        proof
    }
    fn revoke_request(&self, root: &Path, id: &str, approval: &str) -> ReviewRevocationRequest {
        let mut s = self.store.lock().unwrap();
        s.next += 1;
        let handle = format!("fixture-revoke-source-{}", s.next);
        s.revoke_sources.insert(handle.clone(), approval.into());
        ReviewRevocationRequest {
            schema_version: 1,
            repository_id: repository_id(root).to_string(),
            work_item_id: id.into(),
            approval_id: approval.into(),
            proof_handle: handle,
        }
    }
}
impl ProtectedAuthority for Mock {
    fn historical_preflight_approval(
        &self,
        receipt: &HumanApprovalReceipt,
        activated_mirror: &Digest,
    ) -> Result<Option<ProtectedHistoricalDecision>, &'static str> {
        let store = self.store.lock().unwrap();
        match &store.historical_fault {
            Some(HistoricalFault::Missing) => return Ok(None),
            Some(HistoricalFault::Unavailable) => return Err("authority_unavailable"),
            _ => {}
        }
        Ok(store
            .records
            .get(&receipt.approval_id)
            .filter(|r| {
                r.human
                    && r.consumed
                    && r.mirror.as_ref() == Some(activated_mirror)
                    && r.receipt() == *receipt
                    && r.request.binding.operation == HumanReviewOperation::PreflightReview
            })
            .map(|r| ProtectedHistoricalDecision {
                decided_at_utc: match &store.historical_fault {
                    Some(HistoricalFault::DecisionTime(time)) => time.clone(),
                    _ => r.approved_at_utc.clone(),
                },
            }))
    }
    fn now_millis(&self) -> Result<i64, &'static str> {
        Ok(self.now)
    }
    fn issue(&self, binding: &HumanReviewBinding) -> Result<HumanReviewRequest, &'static str> {
        let mut s = self.store.lock().unwrap();
        s.next += 1;
        let r = HumanReviewRequest {
            schema_version: 2,
            kind: HumanReviewKind::HumanReview,
            issuer: "fixture-protected-issuer".into(),
            request_id: format!("request-{}", s.next),
            nonce: format!("nonce-{}", s.next),
            authority_epoch: s.epoch,
            issued_at_millis: self.now - 1,
            expires_at_millis: self.now + 60000,
            binding: binding.clone(),
        };
        s.issued.insert(r.request_id.clone(), r.clone());
        Ok(r)
    }
    fn consume(
        &self,
        r: &HumanReviewRequest,
        p: &HumanApprovalSubmission,
    ) -> Result<AuthorityObservation, &'static str> {
        let mut s = self.store.lock().unwrap();
        let current_epoch = s.epoch;
        let key = operation_key(&r.binding).unwrap().to_string();
        if s.latest.get(&key) != Some(&p.approval_id) {
            return Err("review_approval_superseded");
        }
        let record = s
            .records
            .get_mut(&p.approval_id)
            .ok_or("human_source_not_registered")?;
        if &record.request != r || &record.proof != p {
            return Err("review_replayed");
        }
        if record.revocation.is_some() {
            return Err("review_approval_revoked");
        }
        if record.denied {
            return Err("review_approval_denied");
        }
        if !record.human {
            return Err("human_review_source_unverified");
        }
        if record.request.authority_epoch != current_epoch {
            return Err("authority_epoch_stale");
        }
        let newly = !record.consumed;
        record.consumed = true;
        let resolved = if record.resolve_hold {
            r.binding.local_hold_digest.clone()
        } else {
            None
        };
        if newly {
            s.unique_consumptions += 1;
        }
        let mutate = s.mutate_after_consume.take();
        drop(s);
        if let Some(path) = mutate {
            std::fs::write(path, b"changed during consumption\n").unwrap();
        }
        Ok(AuthorityObservation {
            principal: Some(AuthenticatedHumanPrincipal {
                subject: "fixture-authenticated-human".into(),
            }),
            issuer: r.issuer.clone(),
            request_digest: r.digest().unwrap(),
            approval_id: p.approval_id.clone(),
            source_event_id: p.source_event_id.clone(),
            epoch: r.authority_epoch,
            revoked: false,
            resolved_local_hold: resolved,
        })
    }
    fn activate(
        &self,
        receipt: &HumanApprovalReceipt,
        digest: &Digest,
    ) -> Result<(), &'static str> {
        let mut s = self.store.lock().unwrap();
        let current_epoch = s.epoch;
        let key = operation_key(&receipt.request.binding).unwrap().to_string();
        if s.latest.get(&key) != Some(&receipt.approval_id) {
            return Err("review_approval_superseded");
        }
        let fail = s.fail_method.take();
        let r = s
            .records
            .get_mut(&receipt.approval_id)
            .ok_or("human_source_not_registered")?;
        if r.revocation.is_some() {
            return Err("review_approval_revoked");
        }
        if !r.consumed || r.denied || !r.human || r.receipt() != *receipt {
            return Err("review_activation_binding_mismatch");
        }
        if r.request.authority_epoch != current_epoch {
            return Err("authority_epoch_stale");
        }
        if Digest::sha256_bytes(
            serde_json::to_vec_pretty(&serde_json::to_value(receipt).unwrap())
                .unwrap()
                .as_slice(),
        ) != *digest
        {
            return Err("review_activation_digest_mismatch");
        }
        if r.mirror.as_ref().is_some_and(|old| old != digest) {
            return Err("review_activation_conflict");
        }
        r.mirror = Some(digest.clone());
        if fail == Some("activate_after") {
            return Err("fixture_lost_activation_reply");
        }
        Ok(())
    }
    fn current(&self, key: &Digest) -> Result<AuthorityState, &'static str> {
        let s = self.store.lock().unwrap();
        let Some(id) = s.latest.get(&key.to_string()) else {
            return Ok(AuthorityState::Missing);
        };
        let r = &s.records[id];
        if r.revocation.is_some() {
            return Ok(AuthorityState::Revoked);
        }
        if r.denied {
            return Ok(AuthorityState::Denied);
        }
        // A recorded execution is a historical fact even after an unrelated
        // generation change. It may only be replayed, never newly Started.
        let recorded_run = r.consumed
            && r.mirror.is_some()
            && matches!(
                s.operation_runs.get(&key.to_string()),
                Some(Run::Running(_) | Run::Completed(_))
            );
        if !recorded_run && r.request.authority_epoch != s.epoch {
            return Err("authority_epoch_stale");
        }
        if !r.consumed {
            return Ok(AuthorityState::Missing);
        }
        let Some(d) = &r.mirror else {
            return Ok(AuthorityState::Pending);
        };
        let reference = AuthorityReference {
            receipt: r.receipt(),
            mirror_digest: d.clone(),
        };
        Ok(match s.operation_runs.get(&key.to_string()) {
            Some(Run::Running(_)) => AuthorityState::Running(reference),
            Some(Run::Completed(_)) => AuthorityState::Consumed(reference),
            None => AuthorityState::Active(reference),
        })
    }
    fn revoke(
        &self,
        request: &ReviewRevocationRequest,
    ) -> Result<HumanRevocationReceipt, &'static str> {
        let mut s = self.store.lock().unwrap();
        if s.revoke_sources.get(&request.proof_handle) != Some(&request.approval_id) {
            return Err("revocation_source_not_registered");
        }
        let r = s
            .records
            .get(&request.approval_id)
            .ok_or("human_source_not_registered")?;
        if r.request.binding.repository_id != request.repository_id
            || r.request.binding.work_item_id != request.work_item_id
        {
            return Err("review_revocation_binding_invalid");
        }
        if let Some(old) = &r.revocation {
            return Ok(old.clone());
        }
        s.epoch += 1;
        let event = HumanRevocationReceipt {
            schema_version: 1,
            repository_id: request.repository_id.clone(),
            work_item_id: request.work_item_id.clone(),
            approval_id: request.approval_id.clone(),
            revocation_id: format!("revoke-{}", s.epoch),
            source_event_id: request.proof_handle.clone(),
            authenticated_subject: "fixture-authenticated-revoker".into(),
            authority_epoch: s.epoch,
        };
        s.records.get_mut(&request.approval_id).unwrap().revocation = Some(event.clone());
        Ok(event)
    }
    fn begin_execution(
        &self,
        key: &Digest,
        receipt: &HumanApprovalReceipt,
        current_binding: Option<&HumanReviewBinding>,
    ) -> Result<ExecutionBegin, &'static str> {
        let mut s = self.store.lock().unwrap();
        let current_epoch = s.epoch;
        let id = s
            .latest
            .get(&key.to_string())
            .ok_or("human_source_not_registered")?
            .clone();
        let r = s.records.get_mut(&id).unwrap();
        if r.revocation.is_some() {
            return Err("review_approval_revoked");
        }
        if r.denied {
            return Err("review_approval_denied");
        }
        if r.receipt() != *receipt || r.mirror.is_none() {
            return Err("review_execution_binding_mismatch");
        }
        match s.operation_runs.get(&key.to_string()) {
            Some(Run::Running(_)) => return Err("human_execution_recovery_required"),
            Some(Run::Completed(result)) => return Ok(ExecutionBegin::Replay(result.clone())),
            None => {}
        }
        if receipt.authority_epoch != current_epoch {
            return Err("authority_epoch_stale");
        }
        if current_binding != Some(&receipt.request.binding) {
            return Err("review_execution_current_binding_mismatch");
        }
        let token = format!("operation-{}", id);
        s.operation_runs
            .insert(key.to_string(), Run::Running(token.clone()));
        s.operation_approvals
            .insert(key.to_string(), receipt.clone());
        s.executions += 1;
        Ok(ExecutionBegin::Started(token))
    }
    fn historical_effect(
        &self,
        repo: &str,
        checkout: &str,
        id: &str,
        operation: &HumanReviewOperation,
        effect: &Digest,
    ) -> Result<bool, &'static str> {
        let s = self.store.lock().unwrap();
        Ok(s.records.values().any(|r| {
            let b = &r.request.binding;
            b.repository_id == repo
                && b.checkout_path == checkout
                && b.work_item_id == id
                && &b.operation == operation
                && committed_effect_matches(&s, r, effect)
        }))
    }
    fn completed_effect(
        &self,
        repo: &str,
        checkout: &str,
        id: &str,
        operation: &HumanReviewOperation,
        effect: &Digest,
        current_hold: Option<&Digest>,
    ) -> Result<bool, &'static str> {
        let s = self.store.lock().unwrap();
        Ok(s.records.values().any(|r| {
            let b = &r.request.binding;
            if b.repository_id != repo
                || b.checkout_path != checkout
                || b.work_item_id != id
                || &b.operation != operation
                || r.revocation.is_some()
                || r.denied
                || r.request.authority_epoch != s.epoch
            {
                return false;
            }
            if current_hold.is_some() && r.receipt().resolved_local_hold.as_ref() != current_hold {
                return false;
            }
            let key = operation_key(b).unwrap().to_string();
            s.latest.get(&key) == Some(&r.proof.approval_id)
                && committed_effect_matches(&s, r, effect)
        }))
    }
    fn finish_execution(&self, token: &str, result: &ExecutionResult) -> Result<(), &'static str> {
        let mut s = self.store.lock().unwrap();
        let fail = s.fail_method.take();
        if fail == Some("finish_before") {
            return Err("fixture_finish_not_committed");
        }
        if fail == Some("finish_unavailable") {
            return Err("authority_unavailable");
        }
        let r = s
            .operation_runs
            .values_mut()
            .find(|run| matches!(run,Run::Running(t) if t==token))
            .ok_or("execution_token_not_current")?;
        *r = Run::Completed(result.clone());
        if fail == Some("finish_after") {
            return Err("fixture_lost_finish_reply");
        }
        Ok(())
    }
}
fn committed_effect_matches(s: &Store, r: &Record, effect: &Digest) -> bool {
    let key = operation_key(&r.request.binding).unwrap().to_string();
    r.consumed
        && r.mirror.is_some()
        && r.human
        && s.operation_approvals.get(&key) == Some(&r.receipt())
        && matches!(s.operation_runs.get(&key), Some(Run::Completed(result))
            if result.success && result.effect_digest.as_ref() == Some(effect))
}
fn fixture() -> tempfile::TempDir {
    fixture_with_reuse_calibration(false)
}
fn fixture_with_reuse_calibration(calibrate: bool) -> tempfile::TempDir {
    let t = tempfile::tempdir().unwrap();
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(t.path())
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--allow-empty",
                "-qm",
                "baseline"
            ])
            .current_dir(t.path())
            .status()
            .unwrap()
            .success()
    );
    crate::attach(t.path()).unwrap();
    if calibrate {
        // Reuse mechanics require an immutable profile captured by start.
        // Calibration is private test setup, not a simulated public approval.
        // Public authenticated profile transitions have separate port tests.
        let profile = read_json(&t.path().join(".ai/project.json")).unwrap();
        crate::confirm_profile_update_business(t.path(), "true", &[], &profile).unwrap();
    }
    crate::start_work_item_with_options(
        t.path(),
        "WI-PORT",
        "fixture",
        "authority domain behavior",
        &["src/**".into()],
        &crate::WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["authenticated review".into()],
            ..Default::default()
        },
    )
    .unwrap();
    crate::preflight_work_item(
        t.path(),
        &t.path().join(".ai/work-items/active/WI-PORT.contract.json"),
    )
    .unwrap();
    t
}
fn approve_operation(
    port: &Arc<Mock>,
    root: &Path,
    operation: HumanReviewOperation,
    payload: &Value,
) -> (HumanReviewRequest, HumanApprovalSubmission) {
    let binding = human_operation_binding(root, "WI-PORT", operation, payload).unwrap();
    let request = port.issue(&binding).unwrap();
    let proof = port.approve(&request, false);
    submit_with_authority(root, "WI-PORT", &request, &proof, port.as_ref()).unwrap();
    (request, proof)
}
fn assert_unavailable_diagnostic_preserves_late_error(mode: &str) {
    let t = fixture();
    let authority = TestAuthority::install();
    let payload = json!({"fixture": mode});
    authority.approve(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::MaterialReview,
        &payload,
    );
    let calls = std::cell::Cell::new(0);
    let diagnostics = std::cell::Cell::new(0);
    let marker = t.path().join("partial-effect.marker");
    let result: Result<Value, ObserverError> = execute_human_operation_with_unavailable_diagnostic(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::MaterialReview,
        &payload,
        || {
            calls.set(calls.get() + 1);
            std::fs::write(&marker, b"effect already happened").unwrap();
            if mode == "body_error" {
                Err(error(t.path(), "authority_unavailable"))
            } else {
                authority.port.store.lock().unwrap().fail_method = Some("finish_unavailable");
                Ok(json!({"effect":"written"}))
            }
        },
        || {
            diagnostics.set(diagnostics.get() + 1);
            Some(error(t.path(), "incorrect_disabled_profile_diagnostic"))
        },
    );
    let cause = result.unwrap_err();
    assert!(
        matches!(&cause, ObserverError::State { message, .. } if message == "authority_unavailable"),
        "{mode}: {cause}"
    );
    assert_eq!(
        diagnostics.get(),
        0,
        "a late failure is not an initial unavailability diagnostic"
    );
    assert_eq!(calls.get(), 1);
    assert_eq!(std::fs::read(&marker).unwrap(), b"effect already happened");
    let retry: Result<Value, ObserverError> = execute_human_operation_with_unavailable_diagnostic(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::MaterialReview,
        &payload,
        || {
            calls.set(calls.get() + 1);
            Ok(json!("must not rerun"))
        },
        || {
            diagnostics.set(diagnostics.get() + 1);
            None
        },
    );
    assert!(retry.is_err());
    assert_eq!(
        calls.get(),
        1,
        "partial effect and lost finish never reexecute automatically"
    );
    assert_eq!(diagnostics.get(), 0);
}
#[test]
fn unavailable_diagnostic_preserves_partially_effective_body_error() {
    assert_unavailable_diagnostic_preserves_late_error("body_error");
}
#[test]
fn unavailable_diagnostic_preserves_uncommitted_finish_error() {
    assert_unavailable_diagnostic_preserves_late_error("finish_unavailable");
}

#[test]
fn unavailable_diagnostic_can_only_reject_before_any_execution() {
    let t = fixture();
    let diagnostics = std::cell::Cell::new(0);
    let result: Result<Value, ObserverError> = execute_human_operation_with_unavailable_diagnostic(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::MaterialReview,
        &json!({"fixture":"disabled"}),
        || panic!("an unavailable production factory cannot execute the body"),
        || {
            diagnostics.set(diagnostics.get() + 1);
            Some(error(t.path(), "disabled_profile_diagnostic"))
        },
    );
    assert!(
        matches!(result, Err(ObserverError::State { message, .. }) if message == "disabled_profile_diagnostic")
    );
    assert_eq!(diagnostics.get(), 1);
}
#[test]
fn exact_human_approval_publishes_and_current_read_is_active() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let request = issue_human_review_request(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::PreflightReview,
        &preflight_payload(),
    )
    .unwrap();
    let proof = p.approve(&request, false);
    let first = submit_human_review(t.path(), "WI-PORT", &request, &proof).unwrap();
    let second = submit_human_review(t.path(), "WI-PORT", &request, &proof).unwrap();
    assert_eq!(first, second);
    assert_eq!(
        current_review_authority_state(t.path(), "WI-PORT").unwrap(),
        CurrentReviewAuthorityState::Active
    );
    assert_eq!(p.store.lock().unwrap().unique_consumptions, 1);
    let decision = crate::preflight_work_item(
        t.path(),
        &t.path().join(".ai/work-items/active/WI-PORT.contract.json"),
    )
    .unwrap();
    assert!(
        !decision
            .unknowns
            .iter()
            .any(|s| s == "human_review_authority_unavailable")
    );
}
#[test]
fn denial_and_revocation_cannot_be_undone_by_old_proof_or_local_bytes() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let (r, proof) = approve_operation(
        &p,
        t.path(),
        HumanReviewOperation::PreflightReview,
        &preflight_payload(),
    );
    let revoke = p.revoke_request(t.path(), "WI-PORT", &proof.approval_id);
    let first = record_review_revocation(t.path(), "WI-PORT", &revoke).unwrap();
    assert_eq!(
        record_review_revocation(t.path(), "WI-PORT", &revoke).unwrap(),
        first
    );
    assert_eq!(
        current_review_authority_state(t.path(), "WI-PORT").unwrap(),
        CurrentReviewAuthorityState::Revoked
    );
    assert!(submit_human_review(t.path(), "WI-PORT", &r, &proof).is_err());
    let fresh = issue_human_review_request(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::PreflightReview,
        &preflight_payload(),
    )
    .unwrap();
    assert_ne!(fresh.nonce, r.nonce);
    let fresh_proof = p.approve(&fresh, false);
    submit_human_review(t.path(), "WI-PORT", &fresh, &fresh_proof).unwrap();
    assert_eq!(
        current_review_authority_state(t.path(), "WI-PORT").unwrap(),
        CurrentReviewAuthorityState::Active
    );
    let denied = issue_human_review_request(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::PreflightReview,
        &preflight_payload(),
    )
    .unwrap();
    let denial = p.approve(&denied, false);
    p.store
        .lock()
        .unwrap()
        .records
        .get_mut(&denial.approval_id)
        .unwrap()
        .denied = true;
    assert!(submit_human_review(t.path(), "WI-PORT", &denied, &denial).is_err());
    assert_eq!(
        current_review_authority_state(t.path(), "WI-PORT").unwrap(),
        CurrentReviewAuthorityState::Denied
    );
}
#[test]
fn every_consumption_publication_boundary_recovers_without_a_second_grant() {
    for stage in [
        "before_consume",
        "after_consume",
        "before_publish",
        "after_publish",
        "before_activate",
        "after_activate",
    ] {
        let t = fixture();
        let p = Mock::new();
        let _installed = Installed::new(p.clone());
        let binding = human_review_binding(t.path(), "WI-PORT").unwrap();
        let request = p.issue(&binding).unwrap();
        let proof = p.approve(&request, false);
        FAILURE.with(|v| v.replace(Some(stage.into())));
        assert!(
            submit_human_review(t.path(), "WI-PORT", &request, &proof).is_err(),
            "{stage}"
        );
        submit_human_review(t.path(), "WI-PORT", &request, &proof).unwrap();
        assert_eq!(p.store.lock().unwrap().unique_consumptions, 1, "{stage}");
        assert_eq!(
            current_review_authority_state(t.path(), "WI-PORT").unwrap(),
            CurrentReviewAuthorityState::Active
        );
    }
}
#[test]
fn revocation_between_consume_and_publish_prevents_activation() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let r = p
        .issue(&human_review_binding(t.path(), "WI-PORT").unwrap())
        .unwrap();
    let proof = p.approve(&r, false);
    FAILURE.with(|v| v.replace(Some("after_consume".into())));
    assert!(submit_human_review(t.path(), "WI-PORT", &r, &proof).is_err());
    let rev = p.revoke_request(t.path(), "WI-PORT", &proof.approval_id);
    record_review_revocation(t.path(), "WI-PORT", &rev).unwrap();
    assert!(submit_human_review(t.path(), "WI-PORT", &r, &proof).is_err());
    assert_eq!(
        current_review_authority_state(t.path(), "WI-PORT").unwrap(),
        CurrentReviewAuthorityState::Revoked
    );
}
#[test]
fn binding_change_during_consumption_stops_publication() {
    let t = fixture();
    let p = Mock::new();
    let r = p
        .issue(&human_review_binding(t.path(), "WI-PORT").unwrap())
        .unwrap();
    let proof = p.approve(&r, false);
    p.store.lock().unwrap().mutate_after_consume = Some(t.path().join("changed.rs"));
    let error = submit_with_authority(t.path(), "WI-PORT", &r, &proof, p.as_ref()).unwrap_err();
    assert!(error.to_string().contains("binding_changed_after_consume"));
    assert!(matches!(
        p.current(&operation_key(&r.binding).unwrap()).unwrap(),
        AuthorityState::Pending
    ));
}
#[test]
fn local_hold_requires_an_explicit_fresh_authenticated_resolution() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let (r, proof) = approve_operation(
        &p,
        t.path(),
        HumanReviewOperation::PreflightReview,
        &preflight_payload(),
    );
    let value = submit_human_review(t.path(), "WI-PORT", &r, &proof).unwrap();
    let digest: Digest = serde_json::from_value(value["receiptRawDigest"].clone()).unwrap();
    crate::record_review_correction(
        t.path(),
        "WI-PORT",
        &ReviewCorrectionRequest {
            schema_version: 1,
            repository_id: repository_id(t.path()).to_string(),
            work_item_id: "WI-PORT".into(),
            target_receipt_raw_digest: digest,
            reason: "local recorder disavows authority".into(),
        },
    )
    .unwrap();
    assert_eq!(
        current_review_authority_state(t.path(), "WI-PORT").unwrap(),
        CurrentReviewAuthorityState::LocalHold
    );
    assert!(submit_human_review(t.path(), "WI-PORT", &r, &proof).is_err());
    let fresh = issue_human_review_request(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::PreflightReview,
        &preflight_payload(),
    )
    .unwrap();
    let proof = p.approve(&fresh, true);
    submit_human_review(t.path(), "WI-PORT", &fresh, &proof).unwrap();
    assert_eq!(
        current_review_authority_state(t.path(), "WI-PORT").unwrap(),
        CurrentReviewAuthorityState::Active
    );
}
#[test]
fn identical_operation_replays_result_without_a_second_execution() {
    let t = fixture();
    let p = Mock::new();
    let payload = json!({"exact":"operation"});
    approve_operation(&p, t.path(), HumanReviewOperation::Retire, &payload);
    let count = AtomicUsize::new(0);
    for _ in 0..2 {
        let value: Value = execute_with_authority(
            t.path(),
            "WI-PORT",
            HumanReviewOperation::Retire,
            &payload,
            p.as_ref(),
            || {
                count.fetch_add(1, Ordering::SeqCst);
                Ok(json!({"done":1}))
            },
        )
        .unwrap();
        assert_eq!(value, json!({"done":1}));
    }
    assert_eq!(count.load(Ordering::SeqCst), 1);
    assert_eq!(p.store.lock().unwrap().executions, 1);
    assert!(
        execute_with_authority::<Value, _>(
            t.path(),
            "WI-PORT",
            HumanReviewOperation::Retire,
            &json!({"exact":"different"}),
            p.as_ref(),
            || panic!("must not run")
        )
        .is_err()
    );
}
#[test]
fn successful_replay_requires_a_committed_effect_digest_without_reexecution() {
    let t = fixture();
    let p = Mock::new();
    let payload = json!({"replay":"requires-effect-commitment"});
    approve_operation(&p, t.path(), HumanReviewOperation::Retire, &payload);
    let marker = t.path().join("business-effect");
    execute_with_authority::<Value, _>(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::Retire,
        &payload,
        p.as_ref(),
        || {
            std::fs::write(&marker, b"executed-once").unwrap();
            Ok(json!({"done":true}))
        },
    )
    .unwrap();
    let key = bare_key(t.path(), "WI-PORT", &HumanReviewOperation::Retire, &payload).unwrap();
    {
        let mut store = p.store.lock().unwrap();
        let Run::Completed(result) = store.operation_runs.get_mut(&key.to_string()).unwrap() else {
            panic!("business execution must have completed");
        };
        result.effect_digest = None;
    }
    let replay = execute_with_authority::<Value, _>(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::Retire,
        &payload,
        p.as_ref(),
        || panic!("a malformed terminal must never reexecute the body"),
    );
    assert!(
        replay
            .unwrap_err()
            .to_string()
            .contains("human_execution_effect_unverified")
    );
    assert_eq!(std::fs::read(&marker).unwrap(), b"executed-once");
    assert_eq!(p.store.lock().unwrap().executions, 1);
}
#[test]
fn successful_replay_accepts_the_business_effect_digest_separate_from_response() {
    let effect = json!({"actual":"business-receipt"});
    let response = json!({"api":"response"});
    assert_ne!(
        cockpit_protocol::digest_json(&effect).unwrap(),
        cockpit_protocol::digest_json(&response).unwrap()
    );
    let result = ExecutionResult {
        success: true,
        value: response.clone(),
        effect_digest: Some(cockpit_protocol::digest_json(&effect).unwrap()),
    };
    assert_eq!(
        decode_execution::<Value>(Path::new("."), result).unwrap(),
        response
    );
}
#[test]
fn failed_replay_preserves_its_business_error_without_an_effect_digest() {
    let result = ExecutionResult {
        success: false,
        value: json!("original-business-error"),
        effect_digest: None,
    };
    let error = decode_execution::<Value>(Path::new("."), result).unwrap_err();
    assert!(error.to_string().contains("original-business-error"));
}
#[test]
fn missing_finish_blocks_reexecution_and_lost_committed_reply_is_idempotent() {
    for mode in ["finish_before", "finish_after"] {
        let t = fixture();
        let p = Mock::new();
        let payload = json!({"finish":mode});
        approve_operation(&p, t.path(), HumanReviewOperation::Retire, &payload);
        p.store.lock().unwrap().fail_method = Some(mode);
        let count = AtomicUsize::new(0);
        assert!(
            execute_with_authority::<Value, _>(
                t.path(),
                "WI-PORT",
                HumanReviewOperation::Retire,
                &payload,
                p.as_ref(),
                || {
                    count.fetch_add(1, Ordering::SeqCst);
                    Ok(json!(1))
                }
            )
            .is_err()
        );
        let retry = execute_with_authority::<Value, _>(
            t.path(),
            "WI-PORT",
            HumanReviewOperation::Retire,
            &payload,
            p.as_ref(),
            || {
                count.fetch_add(1, Ordering::SeqCst);
                Ok(json!(2))
            },
        );
        assert_eq!(retry.is_ok(), mode == "finish_after");
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}
#[test]
fn concurrent_begin_and_revoke_have_one_execution_and_no_restored_grant() {
    let t = fixture();
    let p = Mock::new();
    let payload = json!({"parallel":1});
    let (_, proof) = approve_operation(&p, t.path(), HumanReviewOperation::Retire, &payload);
    let root = t.path().to_path_buf();
    let port = p.clone();
    let args = payload.clone();
    let (started_tx, started_rx) = std::sync::mpsc::channel();
    let (end_tx, end_rx) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        execute_with_authority::<Value, _>(
            &root,
            "WI-PORT",
            HumanReviewOperation::Retire,
            &args,
            port.as_ref(),
            || {
                started_tx.send(()).unwrap();
                end_rx.recv().unwrap();
                Ok(json!("once"))
            },
        )
    });
    started_rx.recv().unwrap();
    assert!(
        execute_with_authority::<Value, _>(
            t.path(),
            "WI-PORT",
            HumanReviewOperation::Retire,
            &payload,
            p.as_ref(),
            || panic!("second execution")
        )
        .is_err()
    );
    let rev = p.revoke_request(t.path(), "WI-PORT", &proof.approval_id);
    p.revoke(&rev).unwrap();
    end_tx.send(()).unwrap();
    assert!(worker.join().unwrap().is_ok());
    assert!(
        execute_with_authority::<Value, _>(
            t.path(),
            "WI-PORT",
            HumanReviewOperation::Retire,
            &payload,
            p.as_ref(),
            || panic!("revoked execution")
        )
        .is_err()
    );
    assert_eq!(p.store.lock().unwrap().executions, 1);
}
#[test]
fn forged_mirror_and_old_status_cannot_authorize_a_new_operation() {
    let t = fixture();
    let p = Mock::new();
    let payload = json!({"operation":1});
    let (r, proof) = approve_operation(&p, t.path(), HumanReviewOperation::Retire, &payload);
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
    let rev = p.revoke_request(t.path(), "WI-PORT", &proof.approval_id);
    p.revoke(&rev).unwrap();
    assert!(
        execute_with_authority::<Value, _>(
            t.path(),
            "WI-PORT",
            HumanReviewOperation::Retire,
            &payload,
            p.as_ref(),
            || panic!("stale status")
        )
        .is_err()
    );
    assert!(submit_with_authority(t.path(), "WI-PORT", &r, &proof, p.as_ref()).is_err());
}

fn runtime() -> cockpit_protocol::RuntimeContext {
    cockpit_protocol::RuntimeContext {
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        protocol_version: cockpit_protocol::PROTOCOL_VERSION,
        runtime_digest: Digest::sha256_bytes(b"isolated-development-fixture"),
    }
}
fn fake_plan(
    root: &Path,
    op: cockpit_protocol::LegacyCloseoutSourceOperation,
) -> cockpit_protocol::LegacyCloseoutSourcePlan {
    cockpit_protocol::LegacyCloseoutSourcePlan {
        schema_version: 1,
        kind: "legacy_closeout_source_plan".into(),
        state: "prepared".into(),
        operation: op,
        repository_id: repository_id(root).to_string(),
        work_item_id: "WI-PORT".into(),
        destination_root: root.to_string_lossy().into(),
        destination_binding: cockpit_protocol::LegacyCloseoutSourceDirectoryBinding {
            root_identity: [0, 0],
            git_common_directory: "fixture".into(),
            git_common_directory_identity: [0, 0],
        },
        source_binding: None,
        registration_bytes: "{}".into(),
        requirement_bytes: "{}".into(),
        registration_raw_digest: Digest::sha256_bytes(b"{}"),
        requirement_raw_digest: Digest::sha256_bytes(b"{}"),
        observation_digest: Digest::sha256_bytes(b"fixture"),
        plan_digest: Digest::sha256_bytes(b"fixture"),
        extension_bytes: None,
        extension_raw_digest: None,
    }
}
fn retirement() -> cockpit_protocol::ActiveWorkItemRetirementRequest {
    cockpit_protocol::ActiveWorkItemRetirementRequest {
        schema_version: 1,
        decision_id: "work-item-retirement".into(),
        disposition: "integrated".into(),
        successor_work_item_id: None,
        actor: "human:caller-claim".into(),
        authority_source: "bare confirm_review".into(),
        reason: "caller supplied old green status".into(),
        repository_id: None,
        contract_digest: None,
        summary_digest: None,
        repository_snapshot_digest: None,
    }
}
#[test]
fn every_current_human_entry_rejects_unavailable_authority_before_business_writes() {
    let t = fixture();
    let root = t.path();
    let runtime = runtime();
    let human = cockpit_protocol::HumanDecision {
        decision: "confirm_close".into(),
        actor: "human:caller-claim".into(),
        authority_source: "old confirm_review".into(),
        reason: "status was green".into(),
        evidence_refs: vec![],
        policy_refs: vec![],
        decided_at: "2026-10-20T00:00:00Z".into(),
        resume_condition: None,
    };
    macro_rules! denied {
        ($label:expr,$call:expr) => {
            let e = $call.unwrap_err();
            assert!(
                e.to_string().contains("authority_unavailable"),
                "{}: {}",
                $label,
                e
            );
        };
    }
    let material = cockpit_protocol::MaterialInspectionReviewDecisionInput {
        schema_version: 1,
        decision: cockpit_protocol::MaterialInspectionReviewDecision::AcceptPermittedUnknowns,
        request_digest: Digest::sha256_bytes(b"material"),
        request_artifact_digest: None,
        request_artifact_byte_length: None,
        reviewer_actor: "agent:caller".into(),
        authority_source: "old approval".into(),
        assurance: cockpit_protocol::MaterialInspectionReviewAssurance::SelfDeclared,
        evidence_refs: vec![],
        rationale: "caller claim".into(),
        residual_risk: "accept".into(),
    };
    // This witness needs an enabled, schema-valid profile so the public
    // disabled-profile diagnostic cannot mask the authority error. These
    // synthetic bytes are never used as admitted execution or history.
    let material_fixture = fixture();
    let material_root = material_fixture.path();
    let material_contract = material_root.join(".ai/work-items/active/WI-PORT.contract.json");
    let mut value = read_json(&material_contract).unwrap();
    value["governanceProfile"] = json!({"materialInspectionReview": {
        "schemaVersion":1, "permittedUnknown":"repository_material_inspection_unavailable",
        "permittedCause":"readable_committed_rust_syntax_unknown", "assurance":"self_declared",
        "reviewerActor":"agent:caller", "authoritySource":"synthetic private fixture",
        "acceptResidualRisk":true
    }});
    value
        .as_object_mut()
        .unwrap()
        .entry("requiredRuntimeCapabilities")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .unwrap()
        .push(json!(
            cockpit_protocol::MATERIAL_INSPECTION_REVIEW_CAPABILITY
        ));
    let typed: cockpit_protocol::Contract = serde_json::from_value(value.clone()).unwrap();
    typed.validate().unwrap();
    std::fs::write(
        &material_contract,
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
    assert!(
        crate::plan_work_item_material_review(material_root, "WI-PORT")
            .unwrap()
            .review_enabled
    );
    let material_summary = material_root.join(".ai/work-items/active/WI-PORT.summary.json");
    let material_summary_before = std::fs::read(&material_summary).unwrap();
    let material_contract_before = std::fs::read(&material_contract).unwrap();
    denied!(
        "material risk acceptance",
        crate::record_work_item_material_review(material_root, "WI-PORT", &material, &runtime)
    );
    assert_eq!(
        std::fs::read(material_summary).unwrap(),
        material_summary_before
    );
    assert_eq!(
        std::fs::read(material_contract).unwrap(),
        material_contract_before
    );
    let summary = root.join(".ai/work-items/active/WI-PORT.summary.json");
    let before = std::fs::read(&summary).unwrap();
    denied!(
        "close",
        crate::close_work_item_with_structured_decision_and_runtime(
            root, "WI-PORT", &human, &runtime
        )
    );
    denied!(
        "retire",
        crate::retire_active_work_item_with_runtime(root, "WI-PORT", &retirement(), &runtime)
    );
    denied!(
        "profile",
        crate::confirm_profile_update(root, "cargo", &["test".into()])
    );
    for decision in ["retry", "resume", "successor", "supersede"] {
        denied!(
            decision,
            crate::record_recovery_decision(
                root,
                "WI-PORT",
                &json!({"decision":decision,"actor":"human:caller-claim","authoritySource":"old confirm_review"}),
                &runtime
            )
        );
    }
    denied!(
        "selected lineage",
        crate::record_selected_successor_lineage_recovery(
            root,
            "WI-PORT",
            &json!({"actor":"human:caller-claim"}),
            &runtime
        )
    );
    let snapshot = cockpit_git::GitRepository::discover(root)
        .unwrap()
        .snapshot()
        .unwrap();
    denied!(
        "archived verification",
        crate::record_archived_verification_recovery_with_runtime(
            root,
            "WI-PORT",
            &crate::ArchivedVerificationRecoveryRequest::default(),
            &runtime,
            &snapshot
        )
    );
    denied!(
        "cross checkout",
        crate::recover_cross_checkout_closeout(root, root, "WI-PORT", &runtime)
    );
    let decision = cockpit_protocol::ActivationRecoveryDecision {
        schema_version: 1,
        decision_id: "activation-recovery".into(),
        response: "confirm".into(),
        request_digest: Digest::sha256_bytes(b"request"),
        human_actor: "human:caller-claim".into(),
        recorder_actor: "agent:Codex".into(),
        assurance: "self_declared".into(),
        authority_source_path: "source.json".into(),
        authority_source_digest: Digest::sha256_bytes(b"source"),
        responded_at: "2026-10-20T00:00:00Z".into(),
        reason: "old status".into(),
    };
    denied!(
        "activation",
        crate::record_activation_recovery(root, "WI-PORT", &decision, &runtime)
    );
    for operation in [
        cockpit_protocol::LegacyCloseoutSourceOperation::Register,
        cockpit_protocol::LegacyCloseoutSourceOperation::Resume,
        cockpit_protocol::LegacyCloseoutSourceOperation::Extend,
        cockpit_protocol::LegacyCloseoutSourceOperation::ResumeExtension,
    ] {
        denied!(
            "legacy registration/resume/extension",
            crate::record_legacy_closeout_source(
                root,
                "WI-PORT",
                &fake_plan(root, operation),
                &runtime
            )
        );
    }
    let guard = crate::acquire_lifecycle_lock(root, "WI-PORT").unwrap();
    denied!(
        "legacy finalization",
        crate::legacy_closeout_source::record_finalization_locked(
            root,
            "WI-PORT",
            &json!({}),
            b"{}",
            &runtime,
            &guard
        )
    );
    drop(guard);
    denied!(
        "historical finalization",
        crate::record_historical_finalization_recovery(
            root,
            "WI-PORT",
            &root.join("absent.json"),
            &runtime
        )
    );
    assert_eq!(std::fs::read(summary).unwrap(), before);
}
#[test]
fn authenticated_profile_entry_runs_native_business_once_and_replays_after_context_changes() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let args = vec!["test".into()];
    let payload = json!({"program":"cargo","args":args});
    let current = read_json(&t.path().join(".ai/project.json")).unwrap();
    let (request, _) = approve_operation(
        &p,
        t.path(),
        HumanReviewOperation::ProfileConfirmation,
        &payload,
    );
    assert_eq!(request.binding.operation_context, current);
    let first = crate::confirm_profile_update(t.path(), "cargo", &args).unwrap();
    assert_eq!(
        first.profile_version,
        current["profileVersion"].as_u64().unwrap() + 1
    );
    let second = crate::confirm_profile_update(t.path(), "cargo", &args).unwrap();
    assert_eq!(first, second);
    assert_eq!(p.store.lock().unwrap().executions, 1);
    assert!(crate::confirm_profile_update(t.path(), "cargo", &["build".into()]).is_err());
}
#[test]
fn authenticated_retirement_enters_native_business_without_granting_a_different_method() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let rt = runtime();
    let request = retirement();
    let payload = json!({"request":request,"runtime":rt});
    approve_operation(&p, t.path(), HumanReviewOperation::Retire, &payload);
    let first = crate::retire_active_work_item_with_runtime(t.path(), "WI-PORT", &request, &rt);
    assert_eq!(p.store.lock().unwrap().executions, 1);
    if let Err(e) = &first {
        assert!(!e.to_string().contains("human_review_not_active"), "{e}");
    }
    let second = crate::retire_active_work_item_with_runtime(t.path(), "WI-PORT", &request, &rt);
    assert_eq!(first.is_ok(), second.is_ok());
    assert_eq!(p.store.lock().unwrap().executions, 1);
    assert!(crate::confirm_profile_update(t.path(), "cargo", &[]).is_err());
}
#[test]
fn fresh_approval_cannot_reset_a_completed_operation_id() {
    let t = fixture();
    let p = Mock::new();
    let payload = json!({"operationId":"stable"});
    approve_operation(&p, t.path(), HumanReviewOperation::Retire, &payload);
    let first: Value = execute_with_authority(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::Retire,
        &payload,
        p.as_ref(),
        || Ok(json!({"executed":1})),
    )
    .unwrap();
    let fresh = p
        .issue(
            &human_operation_binding(t.path(), "WI-PORT", HumanReviewOperation::Retire, &payload)
                .unwrap(),
        )
        .unwrap();
    let proof = p.approve(&fresh, false);
    // A replacement grant may be published, but cannot erase the terminal operation result.
    let replayed = submit_with_authority(t.path(), "WI-PORT", &fresh, &proof, p.as_ref()).unwrap();
    assert_eq!(replayed["humanAuthorityGranted"], false);
    assert_eq!(replayed["state"], "consumed");
    let repeated: Value = execute_with_authority(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::Retire,
        &payload,
        p.as_ref(),
        || panic!("completed operation rerun"),
    )
    .unwrap();
    assert_eq!(first, repeated);
    assert_eq!(p.store.lock().unwrap().executions, 1);
}
#[test]
fn mirror_loss_tamper_and_whole_local_restore_cannot_override_protected_revocation() {
    let t = fixture();
    let p = Mock::new();
    let (r, proof) = approve_operation(
        &p,
        t.path(),
        HumanReviewOperation::PreflightReview,
        &preflight_payload(),
    );
    let reference = match p.current(&operation_key(&r.binding).unwrap()).unwrap() {
        AuthorityState::Active(r) => r,
        _ => panic!("active"),
    };
    let path = t.path().join(format!(
        ".ai/decisions/WI-PORT.human-approval.{}.json",
        reference
            .mirror_digest
            .as_str()
            .trim_start_matches("sha256:")
    ));
    let original = std::fs::read(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert_eq!(
        current_with_authority(
            t.path(),
            "WI-PORT",
            &HumanReviewOperation::PreflightReview,
            &preflight_payload(),
            p.as_ref()
        )
        .unwrap(),
        CurrentReviewAuthorityState::PublicationIncomplete
    );
    std::fs::write(&path, b"{}").unwrap();
    assert_eq!(
        current_with_authority(
            t.path(),
            "WI-PORT",
            &HumanReviewOperation::PreflightReview,
            &preflight_payload(),
            p.as_ref()
        )
        .unwrap(),
        CurrentReviewAuthorityState::Invalid
    );
    std::fs::write(&path, &original).unwrap();
    let revoke = p.revoke_request(t.path(), "WI-PORT", &proof.approval_id);
    p.revoke(&revoke).unwrap();
    assert_eq!(
        current_with_authority(
            t.path(),
            "WI-PORT",
            &HumanReviewOperation::PreflightReview,
            &preflight_payload(),
            p.as_ref()
        )
        .unwrap(),
        CurrentReviewAuthorityState::Revoked
    );
}
#[test]
fn revocation_commit_survives_missing_publication_and_retry_is_idempotent() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let (_, proof) = approve_operation(
        &p,
        t.path(),
        HumanReviewOperation::PreflightReview,
        &preflight_payload(),
    );
    let revoke = p.revoke_request(t.path(), "WI-PORT", &proof.approval_id);
    FAILURE.with(|v| v.replace(Some("after_revoke".into())));
    assert!(record_review_revocation(t.path(), "WI-PORT", &revoke).is_err());
    assert_eq!(
        current_review_authority_state(t.path(), "WI-PORT").unwrap(),
        CurrentReviewAuthorityState::Revoked
    );
    let result = record_review_revocation(t.path(), "WI-PORT", &revoke).unwrap();
    assert_eq!(
        record_review_revocation(t.path(), "WI-PORT", &revoke).unwrap(),
        result
    );
}
#[test]
fn deterministic_running_reservation_blocks_duplicate_then_replays_known_result() {
    let t = fixture();
    let port = Mock::new();
    let _installed = Installed::new(port.clone());
    let payload = json!({"fixture":"deterministic_running"});
    approve_operation(&port, t.path(), HumanReviewOperation::Retire, &payload);
    let count = Arc::new(AtomicUsize::new(0));
    let hook_port = port.clone();
    let hook_count = count.clone();
    let hook_root = t.path().to_path_buf();
    let hook_payload = payload.clone();
    ACTION.with(|action| {
        action.replace(Some((
            "after_begin_execution",
            Box::new(move || {
                assert_eq!(hook_port.store.lock().unwrap().executions, 1);
                assert_eq!(
                    hook_count.load(Ordering::SeqCst),
                    0,
                    "first body has not begun"
                );
                let error = execute_with_authority::<Value, _>(
                    &hook_root,
                    "WI-PORT",
                    HumanReviewOperation::Retire,
                    &hook_payload,
                    hook_port.as_ref(),
                    || panic!("Running must never execute a second body"),
                )
                .unwrap_err()
                .to_string();
                assert!(
                    error.contains("human_execution_recovery_required"),
                    "{error}"
                );
                assert_eq!(hook_port.store.lock().unwrap().executions, 1);
                assert_eq!(hook_count.load(Ordering::SeqCst), 0);
            }),
        )))
    });
    let marker = t.path().join("exactly-once.marker");
    let first: Value = execute_with_authority(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::Retire,
        &payload,
        port.as_ref(),
        || {
            count.fetch_add(1, Ordering::SeqCst);
            std::fs::write(&marker, b"actual single effect").unwrap();
            Ok(json!({"result":"actual completed effect"}))
        },
    )
    .unwrap();
    assert!(
        ACTION.with(|action| action.borrow().is_none()),
        "the Running hook actually fired"
    );
    let bytes = std::fs::read(&marker).unwrap();
    let replay: Value = execute_with_authority(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::Retire,
        &payload,
        port.as_ref(),
        || panic!("known terminal replay must never rerun the body"),
    )
    .unwrap();
    assert_eq!(replay, first);
    assert_eq!(std::fs::read(&marker).unwrap(), bytes);
    assert_eq!(count.load(Ordering::SeqCst), 1);
    assert_eq!(port.store.lock().unwrap().executions, 1);
}

#[test]
fn panic_and_after_begin_interruption_never_reexecute_an_unknown_operation() {
    for mode in ["panic", "after_begin_execution", "before_finish_execution"] {
        let t = fixture();
        let p = Mock::new();
        let payload = json!({"operationId":mode});
        approve_operation(&p, t.path(), HumanReviewOperation::Retire, &payload);
        let count = AtomicUsize::new(0);
        if mode != "panic" {
            FAILURE.with(|v| v.replace(Some(mode.into())));
        }
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            execute_with_authority::<Value, _>(
                t.path(),
                "WI-PORT",
                HumanReviewOperation::Retire,
                &payload,
                p.as_ref(),
                || {
                    count.fetch_add(1, Ordering::SeqCst);
                    if mode == "panic" {
                        panic!("fixture interrupted side effect")
                    };
                    Ok(json!(1))
                },
            )
        }));
        assert!(
            execute_with_authority::<Value, _>(
                t.path(),
                "WI-PORT",
                HumanReviewOperation::Retire,
                &payload,
                p.as_ref(),
                || panic!("unknown operation rerun")
            )
            .is_err()
        );
        assert_eq!(
            count.load(Ordering::SeqCst),
            if mode == "after_begin_execution" {
                0
            } else {
                1
            }
        );
    }
}

#[test]
fn a_current_recovery_effect_requires_its_protected_completed_result() {
    let t = fixture();
    let p = Mock::new();
    let _installed = Installed::new(p.clone());
    let effect = json!({"decision":"retry","actor":"human:caller-claim"});
    assert!(
        require_current_human_effect(t.path(), "WI-PORT", HumanReviewOperation::Recovery, &effect)
            .is_err()
    );
    let payload = json!({"receipt":effect,"runtime":runtime()});
    let (_, proof) = approve_operation(&p, t.path(), HumanReviewOperation::Recovery, &payload);
    assert!(
        require_current_human_effect(t.path(), "WI-PORT", HumanReviewOperation::Recovery, &effect)
            .is_err()
    );
    let _: Value = execute_human_operation(
        t.path(),
        "WI-PORT",
        HumanReviewOperation::Recovery,
        &payload,
        || Ok(effect.clone()),
    )
    .unwrap();
    require_current_human_effect(t.path(), "WI-PORT", HumanReviewOperation::Recovery, &effect)
        .unwrap();
    assert!(
        require_current_human_effect(
            t.path(),
            "WI-PORT",
            HumanReviewOperation::Recovery,
            &json!({"decision":"resume"})
        )
        .is_err()
    );
    let fresh = p
        .issue(
            &human_operation_binding(
                t.path(),
                "WI-PORT",
                HumanReviewOperation::Recovery,
                &payload,
            )
            .unwrap(),
        )
        .unwrap();
    let denied = p.approve(&fresh, false);
    p.store
        .lock()
        .unwrap()
        .records
        .get_mut(&denied.approval_id)
        .unwrap()
        .denied = true;
    assert!(
        require_current_human_effect(t.path(), "WI-PORT", HumanReviewOperation::Recovery, &effect)
            .is_err(),
        "a later denial cannot be undone by an old completed source"
    );
    let revoke = p.revoke_request(t.path(), "WI-PORT", &proof.approval_id);
    p.revoke(&revoke).unwrap();
    assert!(
        require_current_human_effect(t.path(), "WI-PORT", HumanReviewOperation::Recovery, &effect)
            .is_err()
    );
}
#[test]
fn unknown_provenance_remains_distinct_from_missing_and_invalid() {
    let t = fixture();
    let p = Mock::new();
    assert_eq!(
        current_with_authority(
            t.path(),
            "WI-PORT",
            &HumanReviewOperation::PreflightReview,
            &preflight_payload(),
            p.as_ref()
        )
        .unwrap(),
        CurrentReviewAuthorityState::Missing
    );
    let path = t.path().join(".ai/work-items/active/WI-PORT.summary.json");
    let mut summary = read_json(&path).unwrap();
    summary["decisionEvidence"] = json!({"decision":"confirm_review","recordedBy":"human:caller-claim","reason":"old status"});
    std::fs::write(&path, serde_json::to_vec_pretty(&summary).unwrap()).unwrap();
    assert_eq!(
        current_with_authority(
            t.path(),
            "WI-PORT",
            &HumanReviewOperation::PreflightReview,
            &preflight_payload(),
            p.as_ref()
        )
        .unwrap(),
        CurrentReviewAuthorityState::UnknownProvenance
    );
}

fn historical_preflight_fixture() -> (tempfile::TempDir, Arc<Mock>, Installed, HumanReviewRequest) {
    let t = fixture();
    let port = Mock::new();
    let installed = Installed::new(port.clone());
    let (request, _) = approve_operation(
        &port,
        t.path(),
        HumanReviewOperation::PreflightReview,
        &preflight_payload(),
    );
    (t, port, installed, request)
}
#[test]
fn historical_preflight_requires_activation_not_only_consumption_and_local_publication() {
    let t = fixture();
    let port = Mock::new();
    let _installed = Installed::new(port.clone());
    let binding = human_review_binding(t.path(), "WI-PORT").unwrap();
    let request = port.issue(&binding).unwrap();
    let proof = port.approve(&request, false);
    port.consume(&request, &proof).unwrap();
    let receipt = port.store.lock().unwrap().records[&proof.approval_id].receipt();
    crate::review_ledger::publish_authority_mirror(
        t.path(),
        "WI-PORT",
        "human-approval",
        &serde_json::to_value(receipt).unwrap(),
    )
    .unwrap();
    assert!(
        port.store.lock().unwrap().records[&proof.approval_id]
            .mirror
            .is_none()
    );
    assert!(
        historical_preflight_reviews(t.path(), "WI-PORT")
            .unwrap_err()
            .to_string()
            .contains("review_historical_source_unverified")
    );
}
#[test]
fn historical_preflight_new_active_review_is_not_blocked_by_an_old_pending_mirror() {
    let t = fixture();
    let port = Mock::new();
    let _installed = Installed::new(port.clone());
    let request = port
        .issue(&human_review_binding(t.path(), "WI-PORT").unwrap())
        .unwrap();
    let pending = port.approve(&request, false);
    FAILURE.with(|v| v.replace(Some("after_publish".into())));
    assert!(submit_human_review(t.path(), "WI-PORT", &request, &pending).is_err());
    let old_mirrors = crate::review_ledger::approval_mirrors(t.path(), "WI-PORT").unwrap();
    assert_eq!(old_mirrors.len(), 1);
    assert!(
        port.store.lock().unwrap().records[&pending.approval_id]
            .mirror
            .is_none()
    );
    let (fresh, _) = approve_operation(
        &port,
        t.path(),
        HumanReviewOperation::PreflightReview,
        &preflight_payload(),
    );
    let history = historical_preflight_reviews(t.path(), "WI-PORT").unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].receipt.request, fresh);
    assert_eq!(
        current_preflight_review(t.path(), "WI-PORT")
            .unwrap()
            .receipt
            .request,
        fresh
    );
    assert!(
        port.store.lock().unwrap().records[&pending.approval_id]
            .mirror
            .is_none()
    );
    let after = crate::review_ledger::approval_mirrors(t.path(), "WI-PORT").unwrap();
    assert!(old_mirrors.iter().all(|old| after.contains(old)));
}
#[test]
fn source_observation_completion_records_command_produced_r1_and_preserves_start_r0() {
    let t = fixture();
    let port = Mock::new();
    let _installed = Installed::new(port);
    let rt = runtime();
    crate::checkpoint_work_item(t.path(), "WI-PORT").unwrap();
    let request = crate::RepositoryVerificationRequest {
        node_id: "source-observation-command-side-effect".into(),
        program: "python3".into(),
        args: vec!["-c".into(), "from pathlib import Path; Path('src').mkdir(exist_ok=True); Path('src/generated.rs').write_text('generated by actual command\\n')".into()],
        scope: vec!["src/**".into()], stage: "task".into(), runner: "local".into(),
        runtime_digest: rt.runtime_digest.to_string(), base_commit: None, workers: 1,
        work_item_id: Some("WI-PORT".into()), timeout_seconds: None,
        policy: crate::RepositoryVerificationPolicy::NeverReuse,
    };
    let binding = crate::capture_work_item_verification(t.path(), "WI-PORT", &rt).unwrap();
    let r0 = crate::snapshot_digest(binding.snapshot()).unwrap();
    let prepared =
        crate::prepare_work_item_verification(binding, std::slice::from_ref(&request)).unwrap();
    let summary_path = t.path().join(".ai/work-items/active/WI-PORT.summary.json");
    let before: Value = serde_json::from_slice(&std::fs::read(&summary_path).unwrap()).unwrap();
    let run = crate::run_repository_verification(t.path(), &request).unwrap();
    assert!(run.receipt.passed);
    let r1 = crate::snapshot_digest(&run.final_snapshot).unwrap();
    assert_ne!(r0, r1);
    let receipt = serde_json::to_value(run.receipt).unwrap();
    let evidence =
        crate::record_prepared_work_item_verification(&prepared, &receipt, &run.final_snapshot)
            .unwrap();
    assert_eq!(evidence["repositorySnapshotDigest"], json!(r1));
    let after: Value = serde_json::from_slice(&std::fs::read(&summary_path).unwrap()).unwrap();
    assert_eq!(
        after["sourceChangeObservation"],
        before["sourceChangeObservation"]
    );
    assert_eq!(
        after["sourceChangeObservation"]["sourceSnapshotDigest"],
        json!(r0)
    );
    assert!(
        after["changedPaths"]
            .as_array()
            .unwrap()
            .contains(&json!("src/generated.rs"))
    );
}
#[test]
fn source_observation_completion_rejects_command_produced_out_of_contract_change() {
    let (t, _installed, prepared, run) = source_observation_run(
        "from pathlib import Path; Path('outside.txt').write_text('outside approved src scope')",
    );
    let summary = t.path().join(".ai/work-items/active/WI-PORT.summary.json");
    let before = std::fs::read(&summary).unwrap();
    let error = crate::record_prepared_work_item_verification(
        &prepared,
        &serde_json::to_value(run.receipt).unwrap(),
        &run.final_snapshot,
    )
    .unwrap_err();
    assert!(error.to_string().contains("completion scope"), "{error}");
    assert_eq!(before, std::fs::read(summary).unwrap());
    assert!(
        !t.path()
            .join(".ai/evidence/WI-PORT.verification.json")
            .exists()
    );
    assert!(t.path().join("outside.txt").exists());
}
#[test]
fn source_observation_completion_rejects_forged_reuse_for_never_reuse_preparation() {
    let (t, _installed, prepared, run) = source_observation_run("pass");
    let summary = t.path().join(".ai/work-items/active/WI-PORT.summary.json");
    let before = std::fs::read(&summary).unwrap();
    let mut receipt = serde_json::to_value(run.receipt).unwrap();
    receipt["executionRecords"] = json!([]);
    receipt["results"][0]["reused"] = json!(true);
    receipt["results"][0]["receiptId"] =
        json!(Digest::sha256_bytes(b"not a native reusable receipt"));
    receipt["nodesExecuted"] = json!(0);
    receipt["processesSpawned"] = json!(0);
    receipt["nodesReused"] = json!(1);
    let error =
        crate::record_prepared_work_item_verification(&prepared, &receipt, &run.final_snapshot)
            .unwrap_err();
    assert!(
        error.to_string().contains("completion execution facts"),
        "{error}"
    );
    assert_eq!(before, std::fs::read(summary).unwrap());
    assert!(
        !t.path()
            .join(".ai/evidence/WI-PORT.verification.json")
            .exists()
    );
}
#[test]
fn source_observation_completion_rejects_unplanned_passed_summary_projection() {
    let (t, _installed, prepared, run) = source_observation_run("pass");
    let summary = t.path().join(".ai/work-items/active/WI-PORT.summary.json");
    let mut value: Value = serde_json::from_slice(&std::fs::read(&summary).unwrap()).unwrap();
    value["verification"] = json!([{"check":"unplanned-projected-check","result":"passed"}]);
    std::fs::write(&summary, serde_json::to_vec(&value).unwrap()).unwrap();
    let before = std::fs::read(&summary).unwrap();
    let error = crate::record_prepared_work_item_verification(
        &prepared,
        &serde_json::to_value(run.receipt).unwrap(),
        &run.final_snapshot,
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("verification projection changed"),
        "{error}"
    );
    assert_eq!(before, std::fs::read(summary).unwrap());
    assert!(
        !t.path()
            .join(".ai/evidence/WI-PORT.verification.json")
            .exists()
    );
}
#[test]
fn source_observation_completion_rejects_r2_after_returned_r1() {
    let (t, _installed, prepared, run) = source_observation_run("pass");
    let summary = t.path().join(".ai/work-items/active/WI-PORT.summary.json");
    let before = std::fs::read(&summary).unwrap();
    std::fs::create_dir_all(t.path().join("src")).unwrap();
    std::fs::write(t.path().join("src/after-return.rs"), "after returned R1").unwrap();
    let error = crate::record_prepared_work_item_verification(
        &prepared,
        &serde_json::to_value(run.receipt).unwrap(),
        &run.final_snapshot,
    )
    .unwrap_err();
    assert!(error.to_string().contains("snapshot changed"), "{error}");
    assert_eq!(before, std::fs::read(summary).unwrap());
    assert!(
        !t.path()
            .join(".ai/evidence/WI-PORT.verification.json")
            .exists()
    );
}
#[test]
fn source_observation_completion_rollback_preserves_malformed_summary_and_start_facts() {
    let (t, _installed, prepared, run) = source_observation_run("pass");
    let summary = t.path().join(".ai/work-items/active/WI-PORT.summary.json");
    let mut value: Value = serde_json::from_slice(&std::fs::read(&summary).unwrap()).unwrap();
    let start = value["sourceChangeObservation"].clone();
    value["verification"] = json!({});
    std::fs::write(&summary, serde_json::to_vec(&value).unwrap()).unwrap();
    let before = std::fs::read(&summary).unwrap();
    let error = crate::record_prepared_work_item_verification(
        &prepared,
        &serde_json::to_value(run.receipt).unwrap(),
        &run.final_snapshot,
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Summary.verification must be an array"),
        "{error}"
    );
    assert_eq!(before, std::fs::read(summary).unwrap());
    assert_eq!(value["sourceChangeObservation"], start);
    assert!(
        !t.path()
            .join(".ai/evidence/WI-PORT.verification.json")
            .exists()
    );
}
#[test]
fn source_observation_completion_accepts_only_the_natively_planned_reuse_receipt() {
    let t = fixture_with_reuse_calibration(true);
    let port = Mock::new();
    let _installed = Installed::new(port.clone());
    let rt = runtime();
    crate::checkpoint_work_item(t.path(), "WI-PORT").unwrap();
    let request = crate::RepositoryVerificationRequest {
        node_id: "native-prepared-reuse".into(),
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
        policy: crate::RepositoryVerificationPolicy::ProfileAuthorized,
    };
    let first = crate::run_repository_verification(t.path(), &request).unwrap();
    assert_eq!(first.receipt.nodes_executed, 1);
    let binding = crate::capture_work_item_verification(t.path(), "WI-PORT", &rt).unwrap();
    let prepared =
        crate::prepare_work_item_verification(binding, std::slice::from_ref(&request)).unwrap();
    let run = crate::run_repository_verification(t.path(), &request).unwrap();
    assert_eq!(run.receipt.nodes_reused, 1);
    assert_eq!(run.receipt.processes_spawned, 0);
    let mut substituted = serde_json::to_value(&run.receipt).unwrap();
    substituted["results"][0]["receiptId"] =
        json!(Digest::sha256_bytes(b"different reusable receipt"));
    assert!(
        crate::record_prepared_work_item_verification(&prepared, &substituted, &run.final_snapshot)
            .unwrap_err()
            .to_string()
            .contains("completion execution facts")
    );
    crate::record_prepared_work_item_verification(
        &prepared,
        &serde_json::to_value(run.receipt).unwrap(),
        &run.final_snapshot,
    )
    .unwrap();
}
#[test]
fn source_observation_completion_rejects_batch_reuse_before_a_later_source_change() {
    let t = fixture_with_reuse_calibration(true);
    let port = Mock::new();
    let _installed = Installed::new(port.clone());
    let rt = runtime();
    crate::checkpoint_work_item(t.path(), "WI-PORT").unwrap();
    let a = crate::RepositoryVerificationRequest {
        node_id: "earlier-real-reuse".into(),
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
        policy: crate::RepositoryVerificationPolicy::ProfileAuthorized,
    };
    crate::run_repository_verification(t.path(), &a).unwrap();
    let mut b = a.clone();
    b.node_id = "later-real-source-change".into();
    b.program = "python3".into();
    b.args = vec!["-c".into(), "from pathlib import Path; Path('src').mkdir(exist_ok=True); Path('src/later.rs').write_text('later source')".into()];
    b.policy = crate::RepositoryVerificationPolicy::NeverReuse;
    let binding = crate::capture_work_item_verification(t.path(), "WI-PORT", &rt).unwrap();
    let r0 = crate::snapshot_digest(binding.snapshot()).unwrap();
    let prepared = crate::prepare_work_item_verification(binding, &[a.clone(), b.clone()]).unwrap();
    let mut first = crate::run_repository_verification(t.path(), &a).unwrap();
    assert_eq!(first.receipt.nodes_reused, 1);
    let mut second = crate::run_repository_verification(t.path(), &b).unwrap();
    assert_eq!(second.receipt.nodes_executed, 1);
    assert_ne!(r0, crate::snapshot_digest(&second.final_snapshot).unwrap());
    // Merge actual native results as the CLI's sequential scheduler does.
    first.receipt.results.append(&mut second.receipt.results);
    first
        .receipt
        .execution_records
        .append(&mut second.receipt.execution_records);
    first.receipt.nodes_planned += second.receipt.nodes_planned;
    first.receipt.nodes_executed += second.receipt.nodes_executed;
    first.receipt.processes_spawned += second.receipt.processes_spawned;
    let error = crate::record_prepared_work_item_verification(
        &prepared,
        &serde_json::to_value(first.receipt).unwrap(),
        &second.final_snapshot,
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("multi-node completion changed source"),
        "{error}"
    );
    assert!(
        !t.path()
            .join(".ai/evidence/WI-PORT.verification.json")
            .exists()
    );
}
#[test]
fn source_observation_completion_rejects_deleted_verification_projection() {
    let (t, _installed, prepared, run) = source_observation_run("pass");
    let summary = t.path().join(".ai/work-items/active/WI-PORT.summary.json");
    let mut value: Value = serde_json::from_slice(&std::fs::read(&summary).unwrap()).unwrap();
    assert!(
        value
            .as_object_mut()
            .unwrap()
            .remove("verification")
            .is_some()
    );
    std::fs::write(&summary, serde_json::to_vec(&value).unwrap()).unwrap();
    let error = crate::record_prepared_work_item_verification(
        &prepared,
        &serde_json::to_value(run.receipt).unwrap(),
        &run.final_snapshot,
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("verification projection changed"),
        "{error}"
    );
    assert!(
        !t.path()
            .join(".ai/evidence/WI-PORT.verification.json")
            .exists()
    );
}
#[test]
fn source_observation_completion_rejects_missing_actual_execution_facts() {
    let (t, _installed, prepared, run) = source_observation_run(
        "from pathlib import Path; Path('src').mkdir(exist_ok=True); Path('src/generated.rs').write_text('actual command')",
    );
    let summary = t.path().join(".ai/work-items/active/WI-PORT.summary.json");
    let before = std::fs::read(&summary).unwrap();
    let mut receipt = serde_json::to_value(run.receipt).unwrap();
    receipt["executionRecords"] = json!([]);
    let error =
        crate::record_prepared_work_item_verification(&prepared, &receipt, &run.final_snapshot)
            .unwrap_err();
    assert!(
        error.to_string().contains("completion execution facts"),
        "{error}"
    );
    assert_eq!(before, std::fs::read(summary).unwrap());
    assert!(
        !t.path()
            .join(".ai/evidence/WI-PORT.verification.json")
            .exists()
    );
    assert!(t.path().join("src/generated.rs").exists());
}
fn source_observation_run(
    script: &str,
) -> (
    tempfile::TempDir,
    Installed,
    crate::PreparedWorkItemVerification,
    crate::RepositoryVerificationRun,
) {
    let t = fixture();
    let installed = Installed::new(Mock::new());
    let rt = runtime();
    crate::checkpoint_work_item(t.path(), "WI-PORT").unwrap();
    let request = crate::RepositoryVerificationRequest {
        node_id: "source-observation-completion".into(),
        program: "python3".into(),
        args: vec!["-c".into(), script.into()],
        scope: vec!["src/**".into()],
        stage: "task".into(),
        runner: "local".into(),
        runtime_digest: rt.runtime_digest.to_string(),
        base_commit: None,
        workers: 1,
        work_item_id: Some("WI-PORT".into()),
        timeout_seconds: None,
        policy: crate::RepositoryVerificationPolicy::NeverReuse,
    };
    // Explicit empty baseline exercises removal separately from a Summary
    // that never had verification output; it grants no passed check.
    let summary = t.path().join(".ai/work-items/active/WI-PORT.summary.json");
    let mut baseline: Value = serde_json::from_slice(&std::fs::read(&summary).unwrap()).unwrap();
    baseline["verification"] = json!([]);
    std::fs::write(&summary, serde_json::to_vec(&baseline).unwrap()).unwrap();
    let binding = crate::capture_work_item_verification(t.path(), "WI-PORT", &rt).unwrap();
    let prepared =
        crate::prepare_work_item_verification(binding, std::slice::from_ref(&request)).unwrap();
    let run = crate::run_repository_verification(t.path(), &request).unwrap();
    assert!(run.receipt.passed);
    (t, installed, prepared, run)
}
#[test]
fn historical_preflight_facts_survive_new_denial_epoch_revocation_and_local_hold() {
    let (t, port, _installed, request) = historical_preflight_fixture();
    let before = historical_preflight_reviews(t.path(), "WI-PORT")
        .unwrap()
        .remove(0);
    let raw = before.raw_digest().clone();
    let revoke = port.revoke_request(t.path(), "WI-PORT", &before.receipt().approval_id);
    record_review_revocation(t.path(), "WI-PORT", &revoke).unwrap();
    crate::record_review_correction(
        t.path(),
        "WI-PORT",
        &ReviewCorrectionRequest {
            schema_version: 1,
            repository_id: repository_id(t.path()).to_string(),
            work_item_id: "WI-PORT".into(),
            target_receipt_raw_digest: raw.clone(),
            reason: "synthetic later local hold".into(),
        },
    )
    .unwrap();
    {
        let mut store = port.store.lock().unwrap();
        store.epoch += 1;
        store
            .records
            .get_mut(&before.receipt().approval_id)
            .unwrap()
            .denied = true;
    }
    let after = historical_preflight_reviews(t.path(), "WI-PORT").unwrap();
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].receipt().request, request);
    assert_eq!(after[0].raw_digest(), &raw);
    assert_eq!(
        after[0].proof_digest().unwrap(),
        before.proof_digest().unwrap()
    );
    assert!(
        current_preflight_review(t.path(), "WI-PORT").is_err(),
        "historical existence grants no current permission"
    );
}
#[test]
fn historical_preflight_rejects_untrusted_time_precision_window_and_zone() {
    for timestamp in [
        "2026-10-20T12:00:00+08:00",
        "2026-10-20T12:00:00.1234567891Z",
        "1970-01-01T00:00:00Z",
    ] {
        let (t, port, _installed, _) = historical_preflight_fixture();
        let approval = current_preflight_review(t.path(), "WI-PORT")
            .unwrap()
            .receipt()
            .approval_id
            .clone();
        port.store
            .lock()
            .unwrap()
            .records
            .get_mut(&approval)
            .unwrap()
            .approved_at_utc = timestamp.into();
        let error = historical_preflight_reviews(t.path(), "WI-PORT")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("review_historical_decision_time_invalid")
                || error.contains("review_historical_binding_invalid"),
            "{error}"
        );
    }
}
#[test]
fn historical_preflight_rejects_copied_checkout_and_tampered_raw_mirror() {
    let (t, _port, _installed, _) = historical_preflight_fixture();
    let original = historical_preflight_reviews(t.path(), "WI-PORT")
        .unwrap()
        .remove(0);
    let name = format!(
        "WI-PORT.human-approval.{}.json",
        original.raw_digest().as_str().trim_start_matches("sha256:")
    );
    let bytes = std::fs::read(t.path().join(".ai/decisions").join(&name)).unwrap();
    let other = fixture();
    std::fs::write(other.path().join(".ai/decisions").join(&name), &bytes).unwrap();
    assert!(
        historical_preflight_reviews(other.path(), "WI-PORT")
            .unwrap_err()
            .to_string()
            .contains("review_historical_binding_invalid")
    );
    let mut changed = bytes;
    changed.push(b'\n');
    std::fs::write(t.path().join(".ai/decisions").join(&name), changed).unwrap();
    assert!(
        historical_preflight_reviews(t.path(), "WI-PORT")
            .unwrap_err()
            .to_string()
            .contains("review_mirror_digest_mismatch")
    );
}
#[test]
fn historical_preflight_current_proof_binds_full_approval_when_cpr_are_identical() {
    let (t, port, _installed, _) = historical_preflight_fixture();
    let first = current_preflight_review(t.path(), "WI-PORT").unwrap();
    approve_operation(
        &port,
        t.path(),
        HumanReviewOperation::PreflightReview,
        &preflight_payload(),
    );
    let second = current_preflight_review(t.path(), "WI-PORT").unwrap();
    assert_eq!(
        first.receipt().request.binding,
        second.receipt().request.binding
    );
    assert_ne!(first.approval_identity(), second.approval_identity());
    assert_ne!(
        first.proof_digest().unwrap(),
        second.proof_digest().unwrap()
    );
    assert_eq!(
        historical_preflight_reviews(t.path(), "WI-PORT")
            .unwrap()
            .len(),
        2
    );
}

#[path = "review_authority_regressions.rs"]
mod regressions;

#[path = "review_authority_lifecycle_regressions.rs"]
mod lifecycle_regressions;

#[path = "review_authority_v6_regressions.rs"]
mod v6_regressions;

//! A single protected authority port. Production construction is fixed here;
//! only cfg(test) can install the explicitly simulated authority.
use crate::{ObserverError, contract_digest, read_json, repository_id};
use cockpit_core::Digest;
use cockpit_protocol::review_authority::*;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::{path::Path, sync::Arc};

pub(crate) fn error(root: &Path, code: &str) -> ObserverError {
    ObserverError::State {
        path: root.into(),
        message: code.into(),
    }
}
#[derive(Clone)]
struct AuthenticatedHumanPrincipal {
    subject: String,
}
struct AuthorityObservation {
    principal: Option<AuthenticatedHumanPrincipal>,
    issuer: String,
    request_digest: Digest,
    approval_id: String,
    source_event_id: String,
    epoch: u64,
    revoked: bool,
    resolved_local_hold: Option<Digest>,
}
struct VerifiedHumanReview {
    principal: AuthenticatedHumanPrincipal,
    observation: AuthorityObservation,
}
#[derive(Clone)]
struct AuthorityReference {
    receipt: HumanApprovalReceipt,
    mirror_digest: Digest,
}
#[derive(Clone)]
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "B is absent; only the test port constructs protected states"
    )
)]
enum AuthorityState {
    Missing,
    Pending,
    Denied,
    Active(AuthorityReference),
    Revoked,
    Consumed(AuthorityReference),
    Running(AuthorityReference),
}
#[derive(Clone)]
struct ExecutionResult {
    success: bool,
    value: Value,
    // A business receipt commitment is distinct from its API response projection.
    effect_digest: Option<Digest>,
}
/// Private source attestation, never deserialized from public mirror JSON.
struct ProtectedHistoricalDecision {
    /// Time of the original authenticated decision, not request issuance,
    /// publication mtime, or the time this query happened.
    decided_at_utc: String,
}
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "B is absent; only the test port returns execution grants"
    )
)]
enum ExecutionBegin {
    Started(String),
    Replay(ExecutionResult),
}

/// Implemented only by a protected host integration. Issuance authenticates
/// no decision; consume validates the original immutable human source and
/// atomically records nonce/request/operation replay state. activate compares
/// the exact consumed request and complete mirror bytes before admitting it.
/// current/revoke/begin/finish are linearizable against that same protected
/// ledger. Publication in the repository is never authority.
///
/// begin reserves one exact operation. A revoke before begin prevents it;
/// revoke after begin blocks future operations but cannot undo a started one.
/// A lost finish with no protected terminal result stays recovery-required.
/// authority_epoch is the issuer's global current-authorization generation.
/// consume/activate/new begin validate it atomically. A terminal result is a
/// historical fact; replay never creates current authority. completed_effect
/// is a CURRENT authority predicate, not a historical-existence query.
trait ProtectedAuthority: Send + Sync {
    /// Prove this exact receipt and raw mirror were successfully activated.
    /// A Pending consumption is insufficient. This historical fact survives
    /// later revocation, hold, denial and epoch changes; it grants no CURRENT
    /// permission. The original event time is independently authenticated.
    fn historical_preflight_approval(
        &self,
        _receipt: &HumanApprovalReceipt,
        _activated_mirror: &Digest,
    ) -> Result<Option<ProtectedHistoricalDecision>, &'static str> {
        Err("authority_unavailable")
    }
    fn consume(
        &self,
        request: &HumanReviewRequest,
        proof: &HumanApprovalSubmission,
    ) -> Result<AuthorityObservation, &'static str>;
    fn now_millis(&self) -> Result<i64, &'static str> {
        Err("authority_unavailable")
    }
    fn issue(&self, _binding: &HumanReviewBinding) -> Result<HumanReviewRequest, &'static str> {
        Err("authority_unavailable")
    }
    fn activate(
        &self,
        _receipt: &HumanApprovalReceipt,
        _digest: &Digest,
    ) -> Result<(), &'static str> {
        Err("authority_unavailable")
    }
    fn current(&self, _key: &Digest) -> Result<AuthorityState, &'static str> {
        Err("authority_unavailable")
    }
    fn revoke(
        &self,
        _request: &ReviewRevocationRequest,
    ) -> Result<HumanRevocationReceipt, &'static str> {
        Err("authority_unavailable")
    }
    fn begin_execution(
        &self,
        _key: &Digest,
        _receipt: &HumanApprovalReceipt,
        _current_binding: Option<&HumanReviewBinding>,
    ) -> Result<ExecutionBegin, &'static str> {
        Err("authority_unavailable")
    }
    /// An immutable committed fact, independent of CURRENT epoch, latest approval,
    /// denial, revocation and local hold. This predicate never grants permission.
    fn historical_effect(
        &self,
        _repository_id: &str,
        _checkout: &str,
        _id: &str,
        _operation: &HumanReviewOperation,
        _effect: &Digest,
    ) -> Result<bool, &'static str> {
        Err("authority_unavailable")
    }
    /// CURRENT permission for the exact previously committed business receipt.
    /// The hold is a denial-only local observation, never an authenticated grant.
    fn completed_effect(
        &self,
        _repository_id: &str,
        _checkout: &str,
        _id: &str,
        _operation: &HumanReviewOperation,
        _effect: &Digest,
        _current_hold: Option<&Digest>,
    ) -> Result<bool, &'static str> {
        Err("authority_unavailable")
    }
    fn finish_execution(
        &self,
        _token: &str,
        _result: &ExecutionResult,
    ) -> Result<(), &'static str> {
        Err("authority_unavailable")
    }
}
struct UnavailableAuthority;
impl ProtectedAuthority for UnavailableAuthority {
    fn consume(
        &self,
        _: &HumanReviewRequest,
        _: &HumanApprovalSubmission,
    ) -> Result<AuthorityObservation, &'static str> {
        Err("authority_unavailable")
    }
}
fn authority() -> Arc<dyn ProtectedAuthority> {
    #[cfg(test)]
    if let Some(value) = pipeline_tests::installed() {
        return value;
    }
    Arc::new(UnavailableAuthority)
}
fn authenticate(
    expected: &HumanReviewBinding,
    request: &HumanReviewRequest,
    proof: &HumanApprovalSubmission,
    now: i64,
    port: &dyn ProtectedAuthority,
) -> Result<VerifiedHumanReview, &'static str> {
    request.validate_at(now)?;
    if &request.binding != expected {
        return Err("review_request_binding_mismatch");
    }
    let digest = request
        .digest()
        .map_err(|_| "review_request_digest_invalid")?;
    if proof.schema_version != 2
        || proof.request_digest != digest
        || proof.issuer != request.issuer
        || proof.approval_id.trim().is_empty()
        || proof.source_event_id.trim().is_empty()
        || proof.proof_handle.trim().is_empty()
    {
        return Err("review_proof_binding_mismatch");
    }
    let mut observed = port.consume(request, proof)?;
    if observed.revoked {
        return Err("review_approval_revoked");
    }
    if observed.issuer != request.issuer
        || observed.request_digest != digest
        || observed.approval_id != proof.approval_id
        || observed.source_event_id != proof.source_event_id
        || observed.epoch != request.authority_epoch
    {
        return Err("review_authority_binding_mismatch");
    }
    let principal = observed
        .principal
        .take()
        .ok_or("human_review_source_unverified")?;
    if principal.subject.trim().is_empty() {
        return Err("human_review_source_unverified");
    }
    if expected.local_hold_digest.is_some()
        && observed.resolved_local_hold != expected.local_hold_digest
    {
        return Err("review_local_hold");
    }
    Ok(VerifiedHumanReview {
        principal,
        observation: observed,
    })
}

#[cfg(test)]
fn operation_key(binding: &HumanReviewBinding) -> Result<Digest, ObserverError> {
    cockpit_protocol::digest_json(&json!({"domain":"ai-cockpit/human-operation-key/v1",
        "repo":binding.repository_id,"wi":binding.work_item_id,"checkout":binding.checkout_path,
        "operation":binding.operation,"arguments":binding.operation_digest}))
    .map_err(|_| error(Path::new("review"), "review_operation_digest_invalid"))
}
fn bare_key(
    root: &Path,
    id: &str,
    operation: &HumanReviewOperation,
    payload: &Value,
) -> Result<Digest, ObserverError> {
    let root = std::fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let digest = review_operation_digest(operation, payload)
        .map_err(|_| error(&root, "review_operation_digest_invalid"))?;
    cockpit_protocol::digest_json(&json!({"domain":"ai-cockpit/human-operation-key/v1",
        "repo":repository_id(&root).to_string(),"wi":id,"checkout":root.to_string_lossy(),
        "operation":operation,"arguments":digest}))
    .map_err(|_| error(&root, "review_operation_digest_invalid"))
}
/// Observed binding only. A missing P is explicit for recovery operations;
/// contract preflight review still requires a real current preflight P/R.
pub fn human_operation_binding(
    root: &Path,
    id: &str,
    operation: HumanReviewOperation,
    payload: &Value,
) -> Result<HumanReviewBinding, ObserverError> {
    crate::validate_work_item_id(id)?;
    let root = std::fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    // Operation identity must be recoverable after a completed source has
    // disappeared. Resolve lexical location only here; a NEW execution still
    // recomputes the complete canonical source plan below.
    let mut normalized_payload = payload.clone();
    if operation == HumanReviewOperation::CrossCheckoutRecovery {
        let source = normalized_payload["sourceRoot"]
            .as_str()
            .ok_or_else(|| error(&root, "review_source_subject_missing"))?;
        let absolute = std::path::absolute(source).map_err(|cause| ObserverError::Read {
            path: std::path::PathBuf::from(source),
            source: cause,
        })?;
        normalized_payload["sourceRoot"] = json!(absolute);
    }
    let payload = &normalized_payload;
    let mut record_root = root.clone();
    let active = root.join(format!(".ai/work-items/active/{id}.contract.json"));
    let archived = root.join(format!(".ai/work-items/archive/{id}.contract.json"));
    if !active.exists()
        && !archived.exists()
        && operation == HumanReviewOperation::CrossCheckoutRecovery
    {
        let source_path = payload["sourceRoot"]
            .as_str()
            .ok_or_else(|| error(&root, "review_source_subject_missing"))?;
        record_root = std::fs::canonicalize(source_path).map_err(|source| ObserverError::Read {
            path: std::path::PathBuf::from(source_path),
            source,
        })?;
    }
    let (contract_path, summary_path) = ["active", "archive"]
        .into_iter()
        .map(|part| {
            (
                record_root.join(format!(".ai/work-items/{part}/{id}.contract.json")),
                record_root.join(format!(".ai/work-items/{part}/{id}.summary.json")),
            )
        })
        .find(|(contract, summary)| contract.is_file() && summary.is_file())
        .ok_or_else(|| error(&root, "review_context_missing"))?;
    for path in [&contract_path, &summary_path] {
        if !crate::is_regular_non_symlink(path)? {
            return Err(error(path, "review_context_not_regular"));
        }
    }
    let contract = read_json(&contract_path)?;
    let summary = read_json(&summary_path)?;
    let identity = repository_id(&root).to_string();
    if contract["repositoryId"] != identity
        || summary["repositoryId"] != identity
        || contract["workItemId"] != id
        || summary["workItemId"] != id
    {
        return Err(error(&root, "review_context_identity_mismatch"));
    }
    let c = contract_digest(&contract_path)?;
    let p = summary["preflightDecisionDigest"]
        .as_str()
        .map(str::parse::<Digest>)
        .transpose()
        .map_err(|_| error(&root, "review_context_digest_invalid"))?;
    let git = cockpit_git::GitRepository::discover(&root)
        .map_err(|_| error(&root, "review_snapshot_unavailable"))?;
    let snapshot = git
        .snapshot()
        .map_err(|_| error(&root, "review_snapshot_unavailable"))?;
    let r = crate::snapshot_digest(&snapshot)?;
    if operation == HumanReviewOperation::PreflightReview
        && (p.is_none()
            || summary["preflightContractDigest"] != c.to_string()
            || summary["preflightRepositorySnapshotDigest"] != r.to_string()
            || !matches!(
                summary["preflightState"].as_str(),
                Some("green" | "yellow" | "red")
            ))
    {
        return Err(error(&root, "review_context_preflight_stale"));
    }
    let scope: Vec<String> = serde_json::from_value(contract["scope"].clone())
        .map_err(|_| error(&root, "review_scope_invalid"))?;
    let activation_recovery_digest = if summary["activationRecovery"].is_null() {
        None
    } else {
        Some(
            cockpit_protocol::digest_json(&summary["activationRecovery"])
                .map_err(|_| error(&root, "review_activation_context_invalid"))?,
        )
    };
    let operation_context = match operation {
        HumanReviewOperation::ProfileConfirmation => read_json(&root.join(".ai/project.json"))?,
        // Source verification R deliberately excludes HEAD. Operations that
        // create a baseRevision or cleanup target must bind Git identity too.
        HumanReviewOperation::Recovery
        | HumanReviewOperation::Retire
        | HumanReviewOperation::Close => git_operation_context(&root, &snapshot)?,
        HumanReviewOperation::CrossCheckoutRecovery => {
            let source = payload["sourceRoot"]
                .as_str()
                .ok_or_else(|| error(&root, "review_source_subject_missing"))?;
            let runtime = serde_json::from_value(payload["runtime"].clone())
                .map_err(|_| error(&root, "review_runtime_binding_invalid"))?;
            crate::plan_cross_checkout_closeout_recovery(&root, Path::new(source), id, &runtime)?
        }
        _ => Value::Null,
    };
    Ok(HumanReviewBinding {
        repository_id: identity,
        work_item_id: id.into(),
        checkout_path: root.to_string_lossy().into(),
        contract_digest: c,
        preflight_decision_digest: p,
        repository_snapshot_digest: r,
        operation: operation.clone(),
        scope,
        operation_digest: review_operation_digest(&operation, payload)
            .map_err(|_| error(&root, "review_operation_digest_invalid"))?,
        operation_payload: payload.clone(),
        operation_context,
        local_hold_digest: crate::review_ledger::local_hold_head(&root, id)?,
        activation_recovery_digest,
    })
}
fn git_operation_context(
    root: &Path,
    snapshot: &cockpit_git::RepositorySnapshot,
) -> Result<Value, ObserverError> {
    let git_dir = crate::git_text(root, &["rev-parse", "--absolute-git-dir"])
        .ok_or_else(|| error(root, "review_git_identity_unavailable"))?;
    let common_dir = crate::git_text(
        root,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )
    .ok_or_else(|| error(root, "review_git_identity_unavailable"))?;
    let worktrees = crate::git_text(root, &["worktree", "list", "--porcelain"])
        .ok_or_else(|| error(root, "review_git_identity_unavailable"))?;
    let symbolic_ref = crate::git_text(root, &["symbolic-ref", "--quiet", "HEAD"]);
    let head = crate::git_text(root, &["rev-parse", "--verify", "HEAD^{commit}"]);
    if head != snapshot.head {
        return Err(error(root, "review_git_identity_changed_during_capture"));
    }
    Ok(
        json!({"head":head,"symbolicRef":symbolic_ref,"gitDir":git_dir,
        "commonGitDir":common_dir,"worktreeInventory":worktrees}),
    )
}
fn preflight_payload() -> Value {
    json!({"decision":"confirm_review"})
}
pub fn human_review_binding(root: &Path, id: &str) -> Result<HumanReviewBinding, ObserverError> {
    human_operation_binding(
        root,
        id,
        HumanReviewOperation::PreflightReview,
        &preflight_payload(),
    )
}
pub fn issue_human_review_request(
    root: &Path,
    id: &str,
    operation: HumanReviewOperation,
    payload: &Value,
) -> Result<HumanReviewRequest, ObserverError> {
    let binding = human_operation_binding(root, id, operation, payload)?;
    let port = authority();
    let request = port.issue(&binding).map_err(|code| error(root, code))?;
    if request.binding != binding {
        return Err(error(root, "review_issuer_binding_mismatch"));
    }
    request
        .validate_at(port.now_millis().map_err(|code| error(root, code))?)
        .map_err(|code| error(root, code))?;
    Ok(request)
}
fn hit(_stage: &str) -> Result<(), &'static str> {
    #[cfg(test)]
    pipeline_tests::hit(_stage)?;
    Ok(())
}
pub fn submit_human_review(
    root: &Path,
    id: &str,
    request: &HumanReviewRequest,
    proof: &HumanApprovalSubmission,
) -> Result<Value, ObserverError> {
    let port = authority();
    submit_with_authority(root, id, request, proof, port.as_ref())
}
fn submit_with_authority(
    root: &Path,
    id: &str,
    request: &HumanReviewRequest,
    proof: &HumanApprovalSubmission,
    port: &dyn ProtectedAuthority,
) -> Result<Value, ObserverError> {
    let expected = human_operation_binding(
        root,
        id,
        request.binding.operation.clone(),
        &request.binding.operation_payload,
    )?;
    hit("before_consume").map_err(|code| error(root, code))?;
    let verified = authenticate(
        &expected,
        request,
        proof,
        port.now_millis().map_err(|code| error(root, code))?,
        port,
    )
    .map_err(|code| error(root, code))?;
    hit("after_consume").map_err(|code| error(root, code))?;
    if human_operation_binding(
        root,
        id,
        request.binding.operation.clone(),
        &request.binding.operation_payload,
    )
    .map_err(|cause| {
        error(
            root,
            &format!("review_binding_changed_after_consume:{cause}"),
        )
    })? != expected
    {
        return Err(error(root, "review_binding_changed_after_consume"));
    }
    let receipt = HumanApprovalReceipt {
        schema_version: 2,
        request: request.clone(),
        request_digest: request
            .digest()
            .map_err(|_| error(root, "review_request_digest_invalid"))?,
        approval_id: verified.observation.approval_id,
        source_event_id: verified.observation.source_event_id,
        authenticated_subject: verified.principal.subject,
        authority_epoch: verified.observation.epoch,
        resolved_local_hold: verified.observation.resolved_local_hold,
    };
    hit("before_publish").map_err(|code| error(root, code))?;
    let value = serde_json::to_value(&receipt).map_err(|_| error(root, "review_mirror_invalid"))?;
    let digest =
        crate::review_ledger::publish_authority_mirror(root, id, "human-approval", &value)?;
    hit("after_publish").map_err(|code| error(root, code))?;
    if human_operation_binding(
        root,
        id,
        request.binding.operation.clone(),
        &request.binding.operation_payload,
    )
    .map_err(|cause| {
        error(
            root,
            &format!("review_binding_changed_before_activate:{cause}"),
        )
    })? != expected
    {
        return Err(error(root, "review_binding_changed_before_activate"));
    }
    hit("before_activate").map_err(|code| error(root, code))?;
    port.activate(&receipt, &digest)
        .map_err(|code| error(root, code))?;
    hit("after_activate").map_err(|code| error(root, code))?;
    // One protected CURRENT observation supplies both the reference and state.
    // A separate enum-only lookup could accidentally associate A's receipt with B.
    let key = bare_key(
        root,
        id,
        &request.binding.operation,
        &request.binding.operation_payload,
    )?;
    let observed = port.current(&key).map_err(|code| error(root, code))?;
    match &observed {
        AuthorityState::Active(reference)
        | AuthorityState::Consumed(reference)
        | AuthorityState::Running(reference)
            if reference.receipt == receipt && reference.mirror_digest == digest => {}
        _ => return Err(error(root, "review_activation_not_current_exact_receipt")),
    }
    let current = classify_current_observation(
        root,
        id,
        &request.binding.operation,
        &request.binding.operation_payload,
        port,
        Ok(observed),
    )?;
    if !matches!(
        current,
        CurrentReviewAuthorityState::Active | CurrentReviewAuthorityState::Consumed
    ) {
        return Err(error(root, "review_activation_not_current"));
    }
    Ok(
        json!({"receipt":receipt,"receiptRawDigest":digest,"state":current,"humanAuthorityGranted":current==CurrentReviewAuthorityState::Active}),
    )
}
fn current_with_authority(
    root: &Path,
    id: &str,
    operation: &HumanReviewOperation,
    payload: &Value,
    port: &dyn ProtectedAuthority,
) -> Result<CurrentReviewAuthorityState, ObserverError> {
    crate::validate_work_item_id(id)?;
    let key = bare_key(root, id, operation, payload)?;
    classify_current_observation(root, id, operation, payload, port, port.current(&key))
}
fn classify_current_observation(
    root: &Path,
    id: &str,
    operation: &HumanReviewOperation,
    payload: &Value,
    port: &dyn ProtectedAuthority,
    observed: Result<AuthorityState, &'static str>,
) -> Result<CurrentReviewAuthorityState, ObserverError> {
    use CurrentReviewAuthorityState as S;
    let state = match observed {
        Ok(state) => state,
        Err("authority_unavailable") => return Ok(S::AuthorityUnavailable),
        Err("authority_epoch_stale") => return Ok(S::Invalid),
        Err(code) => return Err(error(root, code)),
    };
    let reference = match state {
        AuthorityState::Missing => {
            if crate::review_ledger::local_hold_head(root, id)?.is_some() {
                return Ok(S::LocalHold);
            }
            let summary = root.join(format!(".ai/work-items/active/{id}.summary.json"));
            return Ok(
                if read_json(&summary).is_ok_and(|v| !v["decisionEvidence"].is_null()) {
                    S::UnknownProvenance
                } else {
                    S::Missing
                },
            );
        }
        AuthorityState::Pending => return Ok(S::PublicationIncomplete),
        AuthorityState::Denied => return Ok(S::Denied),
        AuthorityState::Revoked => return Ok(S::Revoked),
        AuthorityState::Consumed(_) | AuthorityState::Running(_) => return Ok(S::Consumed),
        AuthorityState::Active(reference) => reference,
    };
    let expected = match human_operation_binding(root, id, operation.clone(), payload) {
        Ok(binding) => binding,
        Err(_) => return Ok(S::UnknownProvenance),
    };
    if expected.local_hold_digest.is_some()
        && reference.receipt.resolved_local_hold != expected.local_hold_digest
    {
        return Ok(S::LocalHold);
    }
    if reference.receipt.request.binding != expected {
        return Ok(S::Invalid);
    }
    let now = port.now_millis().map_err(|code| error(root, code))?;
    if reference.receipt.request.validate_at(now).is_err() {
        return Ok(S::Invalid);
    }
    let stored = match crate::review_ledger::read_authority_mirror(
        root,
        id,
        "human-approval",
        &reference.mirror_digest,
    ) {
        Ok(v) => v,
        Err(ObserverError::Read { source, .. })
            if source.kind() == std::io::ErrorKind::NotFound =>
        {
            return Ok(S::PublicationIncomplete);
        }
        Err(_) => return Ok(S::Invalid),
    };
    if serde_json::from_value::<HumanApprovalReceipt>(stored)
        .ok()
        .as_ref()
        != Some(&reference.receipt)
    {
        return Ok(S::Invalid);
    }
    Ok(S::Active)
}
pub fn current_review_authority_state(
    root: &Path,
    id: &str,
) -> Result<CurrentReviewAuthorityState, ObserverError> {
    current_with_authority(
        root,
        id,
        &HumanReviewOperation::PreflightReview,
        &preflight_payload(),
        authority().as_ref(),
    )
}
pub(crate) fn require_preflight_authority(root: &Path, id: &str) -> Result<(), ObserverError> {
    let state = current_review_authority_state(root, id)?;
    if state != CurrentReviewAuthorityState::Active {
        return Err(error(root, &format!("human_review_not_active:{state:?}")));
    }
    Ok(())
}
pub fn record_review_revocation(
    root: &Path,
    id: &str,
    request: &ReviewRevocationRequest,
) -> Result<Value, ObserverError> {
    crate::validate_work_item_id(id)?;
    if request.schema_version != 1
        || request.work_item_id != id
        || request.repository_id != repository_id(root).to_string()
        || request.approval_id.trim().is_empty()
        || request.proof_handle.trim().is_empty()
    {
        return Err(error(root, "review_revocation_binding_invalid"));
    }
    let port = authority();
    let receipt = port.revoke(request).map_err(|code| error(root, code))?;
    if receipt.schema_version != 1
        || receipt.repository_id != request.repository_id
        || receipt.work_item_id != id
        || receipt.approval_id != request.approval_id
        || receipt.authenticated_subject.trim().is_empty()
        || receipt.source_event_id.trim().is_empty()
        || receipt.revocation_id.trim().is_empty()
        || receipt.authority_epoch == 0
    {
        return Err(error(root, "review_revocation_source_invalid"));
    }
    hit("after_revoke").map_err(|code| error(root, code))?;
    let value =
        serde_json::to_value(&receipt).map_err(|_| error(root, "review_revocation_invalid"))?;
    let digest =
        crate::review_ledger::publish_authority_mirror(root, id, "human-revocation", &value)?;
    Ok(
        json!({"receipt":receipt,"receiptRawDigest":digest,"state":"revoked","humanAuthorityGranted":false}),
    )
}
pub(crate) fn require_authority_available(root: &Path) -> Result<(), ObserverError> {
    authority()
        .now_millis()
        .map(|_| ())
        .map_err(|code| error(root, code))
}
pub(crate) fn sole_active_review_subject(root: &Path) -> Result<String, ObserverError> {
    let path = root.join(".ai/work-items/active");
    let mut ids = Vec::new();
    for entry in std::fs::read_dir(&path).map_err(|source| ObserverError::Read {
        path: path.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        let name = entry.file_name();
        if let Some(id) = name.to_str().and_then(|n| n.strip_suffix(".contract.json")) {
            crate::validate_work_item_id(id)?;
            ids.push(id.to_owned());
        }
    }
    if ids.len() != 1 {
        return Err(error(root, "human_review_subject_required"));
    }
    Ok(ids.remove(0))
}
fn decode_execution<T: DeserializeOwned>(
    root: &Path,
    result: ExecutionResult,
) -> Result<T, ObserverError> {
    if !result.success {
        return Err(error(
            root,
            result
                .value
                .as_str()
                .unwrap_or("previous_human_operation_failed"),
        ));
    }
    if result.effect_digest.is_none() {
        return Err(error(root, "human_execution_effect_unverified"));
    }
    serde_json::from_value(result.value).map_err(|_| error(root, "human_execution_result_invalid"))
}
pub(crate) fn execute_human_operation<T, F>(
    root: &Path,
    id: &str,
    operation: HumanReviewOperation,
    payload: &Value,
    body: F,
) -> Result<T, ObserverError>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Result<T, ObserverError>,
{
    execute_with_authority(root, id, operation, payload, authority().as_ref(), body)
}
pub(crate) fn execute_human_operation_with_unavailable_diagnostic<T, F, D>(
    root: &Path,
    id: &str,
    operation: HumanReviewOperation,
    payload: &Value,
    body: F,
    diagnostic: D,
) -> Result<T, ObserverError>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Result<T, ObserverError>,
    D: FnOnce() -> Option<ObserverError>,
{
    let port = authority();
    let key = bare_key(root, id, &operation, payload)?;
    // Only this initial failed read proves this invocation has not begun.
    // The diagnostic can reject but cannot grant execution. All successful
    // observations, including terminal replay, use the unchanged pipeline;
    // its begin still atomically checks the latest exact approval and binding.
    match port.current(&key) {
        Err("authority_unavailable") => {
            Err(diagnostic().unwrap_or_else(|| error(root, "authority_unavailable")))
        }
        Err(code) => Err(error(root, code)),
        Ok(_) => execute_with_authority(root, id, operation, payload, port.as_ref(), body),
    }
}
fn execute_with_authority<T, F>(
    root: &Path,
    id: &str,
    operation: HumanReviewOperation,
    payload: &Value,
    port: &dyn ProtectedAuthority,
    body: F,
) -> Result<T, ObserverError>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Result<T, ObserverError>,
{
    execute_bound_with_authority(root, id, operation, payload, port, |_| {
        body().map(|value| (value, None))
    })
}
/// Only repository-owned code constructs the effect; public JSON is never a capability.
pub(crate) fn execute_bound_human_operation<T, F>(
    root: &Path,
    id: &str,
    operation: HumanReviewOperation,
    payload: &Value,
    body: F,
) -> Result<T, ObserverError>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce(&HumanReviewBinding) -> Result<(T, Option<Value>), ObserverError>,
{
    execute_bound_with_authority(root, id, operation, payload, authority().as_ref(), body)
}
fn execute_bound_with_authority<T, F>(
    root: &Path,
    id: &str,
    operation: HumanReviewOperation,
    payload: &Value,
    port: &dyn ProtectedAuthority,
    body: F,
) -> Result<T, ObserverError>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce(&HumanReviewBinding) -> Result<(T, Option<Value>), ObserverError>,
{
    let key = bare_key(root, id, &operation, payload)?;
    let reference = match port.current(&key).map_err(|code| error(root, code))? {
        AuthorityState::Active(r) | AuthorityState::Consumed(r) => r,
        AuthorityState::Running(_) => return Err(error(root, "human_execution_recovery_required")),
        AuthorityState::Revoked => return Err(error(root, "review_approval_revoked")),
        AuthorityState::Denied => return Err(error(root, "review_approval_denied")),
        _ => return Err(error(root, "human_review_not_active")),
    };
    // The protected begin, not a previously returned status, decides whether
    // this is an exact completed replay or a newly reserved execution.
    if matches!(
        port.current(&key).map_err(|code| error(root, code))?,
        AuthorityState::Active(_)
    ) && current_with_authority(root, id, &operation, payload, port)?
        != CurrentReviewAuthorityState::Active
    {
        if operation == HumanReviewOperation::CrossCheckoutRecovery {
            // A rejected NEW execution may expose its precise read-only
            // source validation failure. No body or protected begin occurs.
            // Consumed replay skips this branch and needs no surviving source.
            human_operation_binding(root, id, operation.clone(), payload)?;
        }
        return Err(error(root, "human_review_not_active"));
    }
    let current_binding = match port.current(&key).map_err(|code| error(root, code))? {
        AuthorityState::Active(_) => Some(human_operation_binding(
            root,
            id,
            operation.clone(),
            payload,
        )?),
        AuthorityState::Consumed(_) => None,
        _ => return Err(error(root, "human_review_not_active")),
    };
    let token = match port
        .begin_execution(&key, &reference.receipt, current_binding.as_ref())
        .map_err(|code| error(root, code))?
    {
        ExecutionBegin::Replay(result) => return decode_execution(root, result),
        ExecutionBegin::Started(token) => token,
    };
    hit("after_begin_execution").map_err(|code| error(root, code))?;
    let binding = current_binding
        .as_ref()
        .ok_or_else(|| error(root, "review_execution_started_without_binding"))?;
    let result = body(binding);
    let cached = match &result {
        Ok((value, effect)) => {
            let value = serde_json::to_value(value)
                .map_err(|_| error(root, "human_execution_result_invalid"))?;
            let effect_digest = Some(
                cockpit_protocol::digest_json(effect.as_ref().unwrap_or(&value))
                    .map_err(|_| error(root, "human_execution_effect_invalid"))?,
            );
            ExecutionResult {
                success: true,
                value,
                effect_digest,
            }
        }
        Err(err) => ExecutionResult {
            success: false,
            value: Value::String(err.to_string()),
            effect_digest: None,
        },
    };
    hit("before_finish_execution").map_err(|code| error(root, code))?;
    port.finish_execution(&token, &cached)
        .map_err(|code| error(root, code))?;
    result.map(|(value, _)| value)
}
pub fn validate_review_authority_wire(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > 128 * 1024 {
        return Err("review_wire_budget_exceeded".into());
    }
    crate::reject_duplicate_json_keys(bytes).map_err(|e| format!("review_duplicate_json_key: {e}"))
}

pub(crate) fn project_current_review(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    mut decision: cockpit_core::GovernanceDecision,
    archived: bool,
) -> Result<cockpit_core::GovernanceDecision, ObserverError> {
    if archived || contract.work_item_id.is_empty() {
        return Ok(decision);
    }
    let summary_path = root.join(format!(
        ".ai/work-items/active/{}.summary.json",
        contract.work_item_id
    ));
    if !summary_path.is_file() {
        return Ok(decision);
    }
    let summary = read_json(&summary_path)?;
    let policy = crate::effective_policy_for_contract(root, contract)?;
    let requires = policy
        .as_ref()
        .and_then(|p| crate::contract_policy_rule(contract, p))
        .is_some_and(|r| {
            r.approval_mode != cockpit_protocol::ApprovalMode::NoHumanApprovalForLowRisk
        })
        || contract
            .destructive_change_policy
            .as_ref()
            .is_some_and(|p| p.requires_human_approval)
        || contract.restricted_write_approval.is_some()
        || !summary["decisionEvidence"].is_null()
        || !summary["activationRecovery"].is_null()
        || matches!(
            decision.review_state.as_deref(),
            Some("needs_human_confirmation" | "human_decision_recorded")
        )
        || crate::contract_amendment::has_sensitive_amendment(root, &contract.work_item_id)?;
    if !requires {
        return Ok(decision);
    }
    let state = current_review_authority_state(root, &contract.work_item_id)?;
    if state == CurrentReviewAuthorityState::Active {
        decision.review_state = Some("human_decision_recorded".into());
        decision.human_decision_request = None;
        if decision.state != cockpit_core::DecisionState::Red
            && decision.blockers.is_empty()
            && decision.outcome_state != "blocked"
        {
            decision.outcome_state = "verification_pending".into();
            decision.safe_actions.push("continue_to_checkpoint".into());
        }
        return Ok(decision);
    }
    if decision.state != cockpit_core::DecisionState::Red
        && decision.blockers.is_empty()
        && decision.outcome_state != "blocked"
    {
        decision.state = cockpit_core::DecisionState::Yellow;
        decision.outcome_state = "needs_human_decision".into();
    }
    decision.authority = "unknown".into();
    decision.review_state = Some("needs_human_confirmation".into());
    let code = if state == CurrentReviewAuthorityState::AuthorityUnavailable {
        "human_review_authority_unavailable".into()
    } else {
        format!("human_review_{state:?}")
    };
    decision.unknowns.push(code);
    decision.unknowns.sort();
    decision.unknowns.dedup();
    decision
        .required_checks
        .push("authenticated_human_review".into());
    decision.required_checks.sort();
    decision.required_checks.dedup();
    decision.safe_actions.retain(|s| {
        !matches!(
            s.as_str(),
            "continue_to_checkpoint" | "run_verification" | "record_fresh_preflight_decision"
        )
    });
    decision.human_decision_request=Some(cockpit_core::HumanDecisionRequest{decision_id:"contract-preflight-review".into(),status:format!("{state:?}"),
        what_happened:"Current authority is not active; local publication and historical structure cannot grant it.".into(),
        why_it_matters:"Only the protected source/ledger may resolve provenance, holds, revocation or an interrupted operation.".into(),
        options:vec![cockpit_core::HumanDecisionOption{id:"provide_authenticated_review".into(),label:"Use the protected exact request".into(),effect:"Authenticate its original source and exact current binding; never submit a bare confirmation.".into()},cockpit_core::HumanDecisionOption{id:"stop_work".into(),label:"Keep work paused".into(),effect:"Preserve the evidence without performing the operation.".into()}],
        recommended_option:"stop_work".into(),recommendation_reason:"Production authority and execution gate remain unavailable until B is deployed.".into(),
        question:"Obtain a new source-authenticated request decision through the protected authority.".into(),resume_condition:"Current protected state and exact local mirror agree; every other Runtime gate remains satisfied.".into()});
    Ok(decision)
}

#[cfg(test)]
#[path = "review_authority_fixture.rs"]
pub(crate) mod pipeline_tests;

/// Constructed only after independent protected confirmation of the exact
/// activated mirror. Neither its public receipt nor a local projection can
/// construct this verified proof outside this module.
#[derive(Debug)]
pub(crate) struct VerifiedPreflightReview {
    receipt: HumanApprovalReceipt,
    mirror_digest: Digest,
    decided_at: chrono::DateTime<chrono::Utc>,
}
impl VerifiedPreflightReview {
    pub(crate) fn receipt(&self) -> &HumanApprovalReceipt {
        &self.receipt
    }
    pub(crate) fn raw_digest(&self) -> &Digest {
        &self.mirror_digest
    }
    pub(crate) fn decided_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.decided_at
    }
    pub(crate) fn approval_identity(&self) -> (String, String) {
        (
            self.receipt.request.issuer.clone(),
            self.receipt.approval_id.clone(),
        )
    }
    pub(crate) fn source_identity(&self) -> (String, String) {
        (
            self.receipt.request.issuer.clone(),
            self.receipt.source_event_id.clone(),
        )
    }
    pub(crate) fn proof_digest(&self) -> Result<Digest, ObserverError> {
        cockpit_protocol::digest_json(&json!({
            "domain":"ai-cockpit/private-verified-preflight/v2",
            "receipt":self.receipt,"activatedMirrorRawDigest":self.mirror_digest,
            "protectedDecisionAtUtc":self.decided_at.to_rfc3339_opts(chrono::SecondsFormat::Nanos,true)
        })).map_err(|_| error(Path::new("review"), "review_historical_proof_invalid"))
    }
}
fn protected_preflight_proof(
    root: &Path,
    id: &str,
    receipt: HumanApprovalReceipt,
    mirror_digest: Digest,
    port: &dyn ProtectedAuthority,
) -> Result<Option<VerifiedPreflightReview>, ObserverError> {
    if receipt.request.binding.checkout_path != root.to_string_lossy() {
        return Err(error(root, "review_historical_binding_invalid"));
    }
    let Some(decided_at) = protected_preflight_event(root, id, &receipt, &mirror_digest, port)?
    else {
        return Ok(None);
    };
    Ok(Some(VerifiedPreflightReview {
        receipt,
        mirror_digest,
        decided_at,
    }))
}

fn protected_preflight_event(
    root: &Path,
    id: &str,
    receipt: &HumanApprovalReceipt,
    mirror_digest: &Digest,
    port: &dyn ProtectedAuthority,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, ObserverError> {
    let Some(attestation) = port
        .historical_preflight_approval(receipt, mirror_digest)
        .map_err(|code| error(root, code))?
    else {
        // Publication can precede activation. Such a mirror proves no
        // historical approval and must not poison a later independent one.
        return Ok(None);
    };
    let timestamp = &attestation.decided_at_utc;
    let fractional_digits = timestamp
        .split_once('.')
        .map(|(_, tail)| tail.strip_suffix('Z').map(str::len));
    if !timestamp.ends_with('Z')
        || fractional_digits.is_some_and(|n| n.is_none_or(|n| n == 0 || n > 9))
    {
        return Err(error(root, "review_historical_decision_time_invalid"));
    }
    let decided_at = chrono::DateTime::parse_from_rfc3339(timestamp)
        .map_err(|_| error(root, "review_historical_decision_time_invalid"))?
        .with_timezone(&chrono::Utc);
    let binding = &receipt.request.binding;
    if receipt.schema_version != 2
        || receipt.authority_epoch != receipt.request.authority_epoch
        || receipt.request_digest
            != receipt
                .request
                .digest()
                .map_err(|_| error(root, "review_request_digest_invalid"))?
        || binding.repository_id != repository_id(root).to_string()
        || binding.work_item_id != id
        || binding.operation != HumanReviewOperation::PreflightReview
        || binding.operation_payload != preflight_payload()
        || [
            &receipt.approval_id,
            &receipt.source_event_id,
            &receipt.authenticated_subject,
        ]
        .iter()
        .any(|v| v.trim().is_empty())
        || receipt
            .request
            .validate_at(decided_at.timestamp_millis())
            .is_err()
    {
        return Err(error(root, "review_historical_binding_invalid"));
    }
    Ok(Some(decided_at))
}

/// A proved original approval event in an owned historical snapshot. This is
/// neither CURRENT authority nor an as-of activation/revocation attestation.
/// Its protected event time does not authenticate the software Runtime; the
/// historical verification/finish envelopes retain those separate checks.
pub(super) struct CapturedPreflightReview<'a> {
    _facts: &'a crate::resource_lifecycle::CapturedHistoricalFacts<'a>,
}

pub(super) fn captured_preflight_review<'a>(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    facts: &'a crate::resource_lifecycle::CapturedHistoricalFacts<'a>,
    completed_at: &str,
) -> Result<Option<CapturedPreflightReview<'a>>, ObserverError> {
    facts.validate_root(root)?;
    let id = &contract.work_item_id;
    let decisions = root.join(".ai/decisions");
    match std::fs::symlink_metadata(&decisions) {
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(ObserverError::Read {
                path: decisions,
                source,
            });
        }
        Ok(_) => {}
    }
    let mut receipts = Vec::new();
    for (value, raw) in crate::review_ledger::approval_mirrors(root, id)? {
        let receipt: HumanApprovalReceipt =
            serde_json::from_value(value).map_err(|_| error(root, "review_mirror_invalid"))?;
        if receipt.request.binding.operation == HumanReviewOperation::PreflightReview {
            receipts.push((receipt, raw));
        }
    }
    if receipts.is_empty() {
        return Ok(None);
    }
    let summary = read_json(&root.join(format!(".ai/work-items/active/{id}.summary.json")))?;
    let c = contract_digest(&root.join(format!(".ai/work-items/active/{id}.contract.json")))?;
    let p: Digest = serde_json::from_value(summary["preflightDecisionDigest"].clone())
        .map_err(|_| error(root, "captured_preflight_review_binding_invalid"))?;
    let r: Digest = serde_json::from_value(summary["preflightRepositorySnapshotDigest"].clone())
        .map_err(|_| error(root, "captured_preflight_review_binding_invalid"))?;
    let activation = if summary["activationRecovery"].is_null() {
        None
    } else {
        Some(
            cockpit_protocol::digest_json(&summary["activationRecovery"])
                .map_err(|_| error(root, "captured_preflight_review_binding_invalid"))?,
        )
    };
    let completed = chrono::DateTime::parse_from_rfc3339(completed_at)
        .map_err(|_| error(root, "captured_preflight_review_time_invalid"))?
        .with_timezone(&chrono::Utc);
    if summary["repositoryId"] != contract.repository_id
        || summary["workItemId"] != *id
        || summary["preflightContractDigest"] != c.to_string()
    {
        return Err(error(root, "captured_preflight_review_binding_invalid"));
    }
    if &r != facts.snapshot_digest() {
        return Err(error(root, "captured_preflight_review_snapshot_mismatch"));
    }
    let port = authority();
    let mut matched = false;
    for (receipt, raw) in receipts {
        let binding = &receipt.request.binding;
        let Some(decided_at) = protected_preflight_event(root, id, &receipt, &raw, port.as_ref())?
        else {
            continue;
        };
        // The protected receipt retains its original checkout identity. A
        // temporary historical clone is a reader, never its new execution site.
        if binding.contract_digest == c
            && binding.preflight_decision_digest.as_ref() == Some(&p)
            && binding.repository_snapshot_digest == r
            && binding.scope == contract.scope
            && binding.operation_context.is_null()
            && binding.activation_recovery_digest == activation
            && decided_at <= completed
        {
            matched = true;
        }
    }
    facts.validate_root(root)?;
    if matched {
        Ok(Some(CapturedPreflightReview { _facts: facts }))
    } else {
        Err(error(root, "captured_preflight_review_source_unverified"))
    }
}
pub(crate) fn historical_preflight_reviews(
    root: &Path,
    id: &str,
) -> Result<Vec<VerifiedPreflightReview>, ObserverError> {
    let root = std::fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let port = authority();
    let mut result = Vec::new();
    let mut unactivated = false;
    for (value, raw) in crate::review_ledger::approval_mirrors(&root, id)? {
        let receipt: HumanApprovalReceipt =
            serde_json::from_value(value).map_err(|_| error(&root, "review_mirror_invalid"))?;
        if receipt.request.binding.operation == HumanReviewOperation::PreflightReview {
            match protected_preflight_proof(&root, id, receipt, raw, port.as_ref())? {
                Some(proof) => result.push(proof),
                None => unactivated = true,
            }
        }
    }
    if unactivated && result.is_empty() {
        return Err(error(&root, "review_historical_source_unverified"));
    }
    Ok(result)
}
pub(crate) fn current_preflight_review(
    root: &Path,
    id: &str,
) -> Result<VerifiedPreflightReview, ObserverError> {
    let root = std::fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let port = authority();
    let operation = HumanReviewOperation::PreflightReview;
    let payload = preflight_payload();
    let key = bare_key(&root, id, &operation, &payload)?;
    let observed = port.current(&key);
    if classify_current_observation(
        &root,
        id,
        &operation,
        &payload,
        port.as_ref(),
        observed.clone(),
    )? != CurrentReviewAuthorityState::Active
    {
        return Err(error(&root, "review_current_preflight_not_active"));
    }
    let Ok(AuthorityState::Active(reference)) = observed else {
        return Err(error(&root, "review_current_preflight_not_active"));
    };
    protected_preflight_proof(
        &root,
        id,
        reference.receipt,
        reference.mirror_digest,
        port.as_ref(),
    )?
    .ok_or_else(|| error(&root, "review_historical_source_unverified"))
}

pub(crate) fn require_current_human_effect(
    root: &Path,
    id: &str,
    operation: HumanReviewOperation,
    value: &Value,
) -> Result<(), ObserverError> {
    let checkout = std::fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let digest = cockpit_protocol::digest_json(value)
        .map_err(|_| error(root, "human_effect_digest_invalid"))?;
    let port = authority();
    if !has_historical_human_effect(root, id, operation.clone(), value)? {
        return Err(error(root, "human_effect_source_unverified"));
    }
    let hold = crate::review_ledger::local_hold_head(root, id)?;
    if !port
        .completed_effect(
            &repository_id(root).to_string(),
            &checkout.to_string_lossy(),
            id,
            &operation,
            &digest,
            hold.as_ref(),
        )
        .map_err(|code| error(root, code))?
    {
        return Err(error(root, "human_effect_source_unverified"));
    }
    Ok(())
}
pub(crate) fn has_historical_human_effect(
    root: &Path,
    id: &str,
    operation: HumanReviewOperation,
    value: &Value,
) -> Result<bool, ObserverError> {
    let checkout = std::fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let digest = cockpit_protocol::digest_json(value)
        .map_err(|_| error(root, "human_effect_digest_invalid"))?;
    authority()
        .historical_effect(
            &repository_id(root).to_string(),
            &checkout.to_string_lossy(),
            id,
            &operation,
            &digest,
        )
        .map_err(|code| error(root, code))
}
pub(crate) fn current_recovery_decision(
    root: &Path,
    id: &str,
    runtime: Option<&cockpit_protocol::RuntimeContext>,
) -> Result<Option<cockpit_protocol::RecoveryDecisionReceipt>, ObserverError> {
    let receipt = crate::load_recovery_decision(root, id, runtime)?;
    if let Some(value) = &receipt {
        require_current_human_effect(
            root,
            id,
            HumanReviewOperation::Recovery,
            &serde_json::to_value(value).map_err(|_| error(root, "human_effect_invalid"))?,
        )?;
    }
    Ok(receipt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // A controlled port substitute, never a selectable production backend.
    struct FixtureAuthority {
        used: Mutex<Option<(String, Digest, String)>>,
        revoked: bool,
        human: bool,
        epoch: u64,
    }
    impl ProtectedAuthority for FixtureAuthority {
        fn consume(
            &self,
            request: &HumanReviewRequest,
            proof: &HumanApprovalSubmission,
        ) -> Result<AuthorityObservation, &'static str> {
            let mut used = self.used.lock().unwrap();
            let binding = (
                request.nonce.clone(),
                request.digest().unwrap(),
                proof.approval_id.clone(),
            );
            if used.as_ref().is_some_and(|previous| previous != &binding) {
                return Err("review_replayed");
            }
            *used = Some(binding);
            Ok(AuthorityObservation {
                principal: self.human.then(|| AuthenticatedHumanPrincipal {
                    subject: "fixture-human-principal".into(),
                }),
                issuer: request.issuer.clone(),
                request_digest: request.digest().unwrap(),
                approval_id: proof.approval_id.clone(),
                source_event_id: proof.source_event_id.clone(),
                epoch: self.epoch,
                revoked: self.revoked,
                resolved_local_hold: None,
            })
        }
    }
    fn inputs() -> (HumanReviewRequest, HumanApprovalSubmission) {
        let request = HumanReviewRequest {
            schema_version: 2,
            kind: HumanReviewKind::HumanReview,
            issuer: "fixture-host".into(),
            request_id: "request-1".into(),
            nonce: "nonce-1".into(),
            authority_epoch: 4,
            issued_at_millis: 1000,
            expires_at_millis: 2000,
            binding: HumanReviewBinding {
                repository_id: "repo".into(),
                work_item_id: "WI-FIXTURE".into(),
                checkout_path: "/fixture".into(),
                contract_digest: Digest::sha256_bytes(b"C"),
                preflight_decision_digest: Some(Digest::sha256_bytes(b"P")),
                repository_snapshot_digest: Digest::sha256_bytes(b"R"),
                operation: HumanReviewOperation::PreflightReview,
                scope: vec!["src/**".into()],
                operation_digest: review_operation_digest(
                    &HumanReviewOperation::PreflightReview,
                    &json!({"decision":"confirm_review"}),
                )
                .unwrap(),
                operation_payload: json!({"decision":"confirm_review"}),
                operation_context: Value::Null,
                local_hold_digest: None,
                activation_recovery_digest: None,
            },
        };
        let proof = HumanApprovalSubmission {
            schema_version: 2,
            kind: HumanApprovalSubmissionKind::HumanApprovalSubmission,
            request_digest: request.digest().unwrap(),
            issuer: request.issuer.clone(),
            approval_id: "approval-1".into(),
            source_event_id: "source-1".into(),
            proof_handle: "opaque-1".into(),
        };
        (request, proof)
    }
    fn authority() -> FixtureAuthority {
        FixtureAuthority {
            used: Mutex::new(None),
            revoked: false,
            human: true,
            epoch: 4,
        }
    }
    #[test]
    fn unavailable_authority_does_not_accept_well_formed_proof() {
        let (r, p) = inputs();
        assert!(matches!(
            authenticate(&r.binding, &r, &p, 1500, &UnavailableAuthority),
            Err("authority_unavailable")
        ));
    }
    #[test]
    fn wrong_context_is_rejected_before_consuming_a_host_decision() {
        let (request, proof) = inputs();
        let a = authority();
        let mut contexts = Vec::new();
        let mut expected = request.binding.clone();
        expected.repository_id = "other-repo".into();
        contexts.push(expected);
        let mut expected = request.binding.clone();
        expected.work_item_id = "WI-OTHER".into();
        contexts.push(expected);
        let mut expected = request.binding.clone();
        expected.contract_digest = Digest::sha256_bytes(b"other-C");
        contexts.push(expected);
        let mut expected = request.binding.clone();
        expected.preflight_decision_digest = Some(Digest::sha256_bytes(b"other-P"));
        contexts.push(expected);
        let mut expected = request.binding.clone();
        expected.repository_snapshot_digest = Digest::sha256_bytes(b"other-R");
        contexts.push(expected);
        let mut expected = request.binding.clone();
        expected.operation = HumanReviewOperation::Close;
        contexts.push(expected);
        let mut expected = request.binding.clone();
        expected.scope.push("extra.rs".into());
        contexts.push(expected);
        for context in contexts {
            assert!(matches!(
                authenticate(&context, &request, &proof, 1500, &a),
                Err("review_request_binding_mismatch")
            ));
        }
        assert!(a.used.lock().unwrap().is_none());
        let mut replay = request.clone();
        replay.nonce = "new-request".into();
        assert!(matches!(
            authenticate(&replay.binding, &replay, &proof, 1500, &a),
            Err("review_proof_binding_mismatch")
        ));
        assert!(a.used.lock().unwrap().is_none());
    }
    #[test]
    fn authenticated_port_is_exact_and_idempotent_but_cannot_rebind() {
        let (mut r, mut p) = inputs();
        let a = authority();
        assert!(authenticate(&r.binding, &r, &p, 1500, &a).is_ok());
        assert!(authenticate(&r.binding, &r, &p, 1500, &a).is_ok());
        r.binding.contract_digest = Digest::sha256_bytes(b"new-C");
        p.request_digest = r.digest().unwrap();
        assert!(matches!(
            authenticate(&r.binding, &r, &p, 1500, &a),
            Err("review_replayed")
        ));
    }
    #[test]
    fn revoked_wrong_epoch_and_technical_source_never_approve() {
        let (r, p) = inputs();
        let a = FixtureAuthority {
            revoked: true,
            ..authority()
        };
        assert!(matches!(
            authenticate(&r.binding, &r, &p, 1500, &a),
            Err("review_approval_revoked")
        ));
        let a = FixtureAuthority {
            epoch: 5,
            ..authority()
        };
        assert!(matches!(
            authenticate(&r.binding, &r, &p, 1500, &a),
            Err("review_authority_binding_mismatch")
        ));
        let a = FixtureAuthority {
            human: false,
            ..authority()
        };
        assert!(matches!(
            authenticate(&r.binding, &r, &p, 1500, &a),
            Err("human_review_source_unverified")
        ));
        // Recovery is a new request/nonce and new protected authority epoch;
        // merely restoring old local receipt bytes has no such effect.
        let mut fresh = r.clone();
        fresh.nonce = "nonce-2".into();
        fresh.authority_epoch = 5;
        let mut proof = p.clone();
        proof.request_digest = fresh.digest().unwrap();
        proof.approval_id = "approval-2".into();
        assert!(
            authenticate(
                &fresh.binding,
                &fresh,
                &proof,
                1500,
                &FixtureAuthority {
                    epoch: 5,
                    ..authority()
                }
            )
            .is_ok()
        );
    }
}

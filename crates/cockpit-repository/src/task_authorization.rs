//! Local native owner provenance and current task authority. Evidence freshness is separate.
use super::{
    ObserverError, acquire_lifecycle_lock, create_and_open_cap_directory,
    legacy_closeout_source::{PinnedRoot, read_regular_cap_bounded},
    open_cap_directory_nofollow_strict,
};
use crate::evidence_store::write_cap_immutable;
use cap_std::fs::Dir;
use chrono::{DateTime, Utc};
use cockpit_protocol::{
    AssuranceLevel, Contract, PreflightDecisionEvidence, RuntimeContext, TaskAuthorizationPolicy,
    TaskAuthorizationProjection, TaskAuthorizationTerminationRequest,
};
use serde_json::{Value, json};
use std::fs;
use std::path::{Component, Path, PathBuf};

const TASK_AUTHORIZATION_BATCH_DIRECTORY: &str = "task-authorization-batches";
const TASK_AUTHORIZATION_BATCH_MAX_BYTES: u64 = 1024 * 1024;
const TASK_AUTHORIZATION_BATCH_MAX_EVENTS: usize = 4096;

/// Prepare a deterministic batch authorization manifest from current native
/// Work Item records. This operation is read-only; it neither records the
/// owner decision nor issues durable child grants.
pub fn prepare_task_authorization_batch_manifest(
    root: &Path,
    request: &cockpit_protocol::TaskAuthorizationBatchRequest,
    runtime: &RuntimeContext,
) -> Result<cockpit_protocol::TaskAuthorizationBatchManifest, ObserverError> {
    request.validate().map_err(|e| {
        error(
            root,
            format!("task_authorization_batch_request_invalid: {e}"),
        )
    })?;
    let root = std::fs::canonicalize(root).map_err(|e| error(root, e.to_string()))?;
    let repository_id = super::repository_id(&root).to_string();
    if request.repository_id != repository_id {
        return Err(error(&root, "task_authorization_batch_repository_mismatch"));
    }

    let mut members = Vec::with_capacity(request.members.len());
    let mut repository_snapshot = None;
    for requested in &request.members {
        super::validate_work_item_id(&requested.work_item_id)?;
        let relative = own_path(&root, &requested.work_item_id, "contract")?;
        if !relative.starts_with(".ai/work-items/active") {
            return Err(error(
                &root,
                format!(
                    "task_authorization_batch_requires_active_member: {}",
                    requested.work_item_id
                ),
            ));
        }
        let contract_value = native_json(&root, &relative)?;
        if contract_value["repositoryId"] != repository_id
            || contract_value["workItemId"] != requested.work_item_id
        {
            return Err(error(
                &root,
                format!(
                    "task_authorization_batch_member_identity_mismatch: {}",
                    requested.work_item_id
                ),
            ));
        }
        let contract: Contract = serde_json::from_value(contract_value.clone())
            .map_err(|e| error(&root, e.to_string()))?;
        let policy = contract
            .task_authorization_policy()
            .map_err(|e| error(&root, e))?
            .ok_or_else(|| {
                error(
                    &root,
                    format!(
                        "task_authorization_batch_member_has_no_owner_policy: {}",
                        requested.work_item_id
                    ),
                )
            })?;
        policy.validate().map_err(|e| error(&root, e))?;
        let projection = assess(&root, &requested.work_item_id, &contract, &policy, runtime)?;
        if projection.state != "active" {
            return Err(error(
                &root,
                format!(
                    "task_authorization_batch_member_not_active: {} ({})",
                    requested.work_item_id, projection.state
                ),
            ));
        }
        let parent_grant_id = projection
            .grant_id
            .ok_or_else(|| error(&root, "task_authorization_batch_parent_grant_missing"))?;
        let scope_digest = projection
            .scope_digest
            .ok_or_else(|| error(&root, "task_authorization_batch_scope_digest_missing"))?;

        let allowed = policy
            .allowed_actions
            .iter()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>();
        let requested_actions = requested
            .actions
            .iter()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>();
        if requested.actions.is_empty()
            || requested_actions.len() != requested.actions.len()
            || requested
                .actions
                .iter()
                .any(|action| !allowed.contains(action.as_str()))
        {
            return Err(error(
                &root,
                format!(
                    "task_authorization_batch_actions_exceed_member_policy: {}",
                    requested.work_item_id
                ),
            ));
        }
        let requested_expiry = chrono::DateTime::parse_from_rfc3339(&requested.expires_at)
            .map_err(|e| error(&root, format!("invalid batch member expiry: {e}")))?;
        if let Some(parent_expiry) = policy.expires_at.as_deref() {
            let parent_expiry = chrono::DateTime::parse_from_rfc3339(parent_expiry)
                .map_err(|e| error(&root, format!("invalid parent task expiry: {e}")))?;
            if requested_expiry > parent_expiry {
                return Err(error(
                    &root,
                    format!(
                        "task_authorization_batch_expiry_exceeds_parent: {}",
                        requested.work_item_id
                    ),
                ));
            }
        }
        requested.budget.validate().map_err(|e| error(&root, e))?;
        let declared_checks = contract
            .verification
            .iter()
            .map(|declaration| match declaration {
                cockpit_protocol::VerificationDeclaration::Legacy(command) => command.as_str(),
                cockpit_protocol::VerificationDeclaration::Check(check) => check.check.as_str(),
            })
            .collect::<std::collections::BTreeSet<_>>();
        if requested
            .budget
            .retry_budget_by_check
            .keys()
            .any(|check| !declared_checks.contains(check.as_str()))
        {
            return Err(error(
                &root,
                format!(
                    "task_authorization_batch_retry_check_not_declared: {}",
                    requested.work_item_id
                ),
            ));
        }

        let status =
            super::work_item_status_snapshot_with_runtime(&root, &requested.work_item_id, runtime)?;
        if status.repository_id != repository_id
            || status.source_digests.get("contract")
                != Some(
                    &cockpit_protocol::digest_json(&contract_value)
                        .map_err(|e| error(&root, e.to_string()))?,
                )
        {
            return Err(error(
                &root,
                format!(
                    "task_authorization_batch_member_changed_during_plan: {}",
                    requested.work_item_id
                ),
            ));
        }
        if repository_snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot != &status.snapshot_digest)
        {
            return Err(error(
                &root,
                "task_authorization_batch_repository_snapshot_changed_during_plan",
            ));
        }
        repository_snapshot = Some(status.snapshot_digest.clone());

        let current_contract_digest = cockpit_protocol::digest_json(&contract_value)
            .map_err(|e| error(&root, e.to_string()))?;
        if native_json(&root, &relative)? != contract_value {
            return Err(error(
                &root,
                format!(
                    "task_authorization_batch_member_changed_during_plan: {}",
                    requested.work_item_id
                ),
            ));
        }
        members.push(cockpit_protocol::TaskAuthorizationBatchMember {
            work_item_id: requested.work_item_id.clone(),
            contract_digest: current_contract_digest,
            snapshot_digest: status.snapshot_digest,
            scope_digest,
            parent_grant_id,
            actor: policy.agent_actor,
            authority_source: policy.authority_source,
            expires_at: requested.expires_at.clone(),
            actions: requested.actions.clone(),
            public_target: policy.public_target,
            budget: requested.budget.clone(),
        });
    }

    cockpit_protocol::TaskAuthorizationBatchManifest::new(repository_id, members)
        .map_err(|e| error(&root, e))
}

/// Persist one exact owner review and its independently derived child grants.
/// The request is re-planned under every member lifecycle lock, in sorted WI
/// order, so a member cannot change between review binding and the atomic
/// receipt publication. Replaying the exact decision is idempotent.
pub fn record_task_authorization_batch_review(
    root: &Path,
    manifest: &cockpit_protocol::TaskAuthorizationBatchManifest,
    review: &cockpit_protocol::TaskAuthorizationBatchReview,
    runtime: &RuntimeContext,
) -> Result<cockpit_protocol::TaskAuthorizationBatchReceipt, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.to_path_buf(),
        source,
    })?;
    manifest.validate().map_err(|source| {
        error(
            &root,
            format!("task_authorization_batch_manifest_invalid: {source}"),
        )
    })?;
    if manifest.repository_id != super::repository_id(&root).to_string() {
        return Err(error(&root, "task_authorization_batch_repository_mismatch"));
    }
    let request = request_from_manifest(manifest);

    // Manifest members are uniquely sorted, so all callers acquire locks in
    // the same order and cannot deadlock while binding a multi-WI decision.
    let mut guards = Vec::with_capacity(manifest.members.len());
    for member in &manifest.members {
        guards.push(acquire_lifecycle_lock(&root, &member.work_item_id)?);
    }
    let current = prepare_task_authorization_batch_manifest(&root, &request, runtime)?;
    if current != *manifest {
        return Err(error(
            &root,
            "task_authorization_batch_manifest_stale_before_owner_decision",
        ));
    }
    let candidate = cockpit_protocol::TaskAuthorizationBatchReceipt::new(
        manifest.clone(),
        review.clone(),
        runtime.runtime_version.clone(),
        runtime.runtime_digest.clone(),
        super::now(),
    )
    .map_err(|source| {
        error(
            &root,
            format!("task_authorization_batch_review_invalid: {source}"),
        )
    })?;

    let (pinned, directory, path) = open_task_authorization_batch_directory(&root, true)?;
    let name = batch_receipt_name(&candidate.manifest.manifest_digest)
        .map_err(|source| error(&path, source))?;
    let receipt_path = path.join(&name);
    if let Some(existing) = read_batch_receipt_file(&root, &directory, &receipt_path, &name)? {
        if existing.manifest == candidate.manifest
            && existing.review == candidate.review
            && existing.runtime_version == candidate.runtime_version
            && existing.runtime_digest == candidate.runtime_digest
        {
            pinned.require_current()?;
            return Ok(existing);
        }
        return Err(error(
            &receipt_path,
            "task_authorization_batch_manifest_already_reviewed_with_different_decision",
        ));
    }
    let bytes = serde_json::to_vec_pretty(&candidate).map_err(|source| {
        error(
            &receipt_path,
            format!("serialize task authorization batch receipt: {source}"),
        )
    })?;
    if bytes.len() as u64 > TASK_AUTHORIZATION_BATCH_MAX_BYTES {
        return Err(error(
            &receipt_path,
            "task_authorization_batch_receipt_too_large",
        ));
    }
    write_cap_immutable(&directory, &name, &receipt_path, &bytes)?;
    pinned.require_current()?;
    let persisted =
        read_batch_receipt_file(&root, &directory, &receipt_path, &name)?.ok_or_else(|| {
            error(
                &receipt_path,
                "task_authorization_batch_receipt_not_persisted",
            )
        })?;
    for (guard, member) in guards.iter().zip(&manifest.members) {
        // The guard subject is already bound at acquisition; explicitly
        // revalidate its pinned root before returning the new authority.
        guard.require_subject(&root, &member.work_item_id)?;
    }
    if persisted != candidate {
        return Err(error(
            &receipt_path,
            "task_authorization_batch_receipt_readback_mismatch",
        ));
    }
    Ok(persisted)
}

/// Atomically reserve one member start. The member lifecycle lock covers the
/// quota read and immutable event publication; exact request replay returns
/// the original reservation instead of spending quota again.
pub fn reserve_task_authorization_batch_start(
    root: &Path,
    request: &cockpit_protocol::TaskAuthorizationBatchStartRequest,
    runtime: &RuntimeContext,
) -> Result<cockpit_protocol::TaskAuthorizationBatchEvent, ObserverError> {
    let (root, receipt, member, guard) = lock_and_load_batch_member(
        root,
        &request.manifest_digest,
        &request.work_item_id,
        runtime,
    )?;
    require_active_batch_member(&root, &member, runtime)?;
    validate_batch_caller(&root, &member, &request.caller)?;
    validate_operation_id(&root, &request.operation_id)?;
    if request.schema_version != cockpit_protocol::TASK_AUTHORIZATION_BATCH_SCHEMA_VERSION
        || !member.actions.contains(&request.action)
    {
        return Err(error(
            &root,
            "task_authorization_batch_start_request_invalid",
        ));
    }
    let events = read_batch_member_events(&root, &receipt, &member, true)?;
    ensure_child_grant_not_terminated(&root, &events)?;
    if let Some(existing) = events.iter().find(|event| {
        event.kind == cockpit_protocol::TaskAuthorizationBatchEventKind::StartReserved
            && event.operation_id == request.operation_id
    }) {
        if existing.action.as_deref() == Some(&request.action) {
            guard.require_subject(&root, &member.work_item_id)?;
            return Ok(existing.clone());
        }
        return Err(error(
            &root,
            "task_authorization_batch_operation_id_conflict",
        ));
    }
    let starts = events
        .iter()
        .filter(|event| {
            event.kind == cockpit_protocol::TaskAuthorizationBatchEventKind::StartReserved
        })
        .count();
    if starts >= member.budget.start_quota as usize {
        return Err(error(
            &root,
            "task_authorization_batch_start_quota_exhausted",
        ));
    }
    let event = new_batch_event(
        &root,
        &receipt,
        &member,
        BatchEventDraft {
            operation_id: request.operation_id.clone(),
            kind: cockpit_protocol::TaskAuthorizationBatchEventKind::StartReserved,
            action: Some(request.action.clone()),
            check_id: None,
            attempt: None,
            outcome: None,
            evidence: None,
            reason: "reserve one start for this exact member operation".into(),
        },
    )?;
    let persisted = append_batch_member_event(&root, &member, event)?;
    guard.require_subject(&root, &member.work_item_id)?;
    Ok(persisted)
}

/// Reserve the first attempt or a permitted retry of one exact declared check.
/// An unresolved or uncertain predecessor must be inspected before another
/// attempt can be reserved.
pub fn reserve_task_authorization_batch_check_attempt(
    root: &Path,
    request: &cockpit_protocol::TaskAuthorizationBatchCheckRequest,
    runtime: &RuntimeContext,
) -> Result<cockpit_protocol::TaskAuthorizationBatchEvent, ObserverError> {
    let (root, receipt, member, guard) = lock_and_load_batch_member(
        root,
        &request.manifest_digest,
        &request.work_item_id,
        runtime,
    )?;
    require_active_batch_member(&root, &member, runtime)?;
    validate_batch_caller(&root, &member, &request.caller)?;
    validate_operation_id(&root, &request.operation_id)?;
    if request.schema_version != cockpit_protocol::TASK_AUTHORIZATION_BATCH_SCHEMA_VERSION
        || !member
            .actions
            .iter()
            .any(|action| action == "run_verification")
        || request.check_id.trim().is_empty()
        || request.check_id.trim() != request.check_id
    {
        return Err(error(
            &root,
            "task_authorization_batch_check_request_invalid",
        ));
    }
    let contract = current_member_contract(&root, &member)?;
    let declared_checks = contract
        .verification
        .iter()
        .map(|declaration| match declaration {
            cockpit_protocol::VerificationDeclaration::Legacy(command) => command.as_str(),
            cockpit_protocol::VerificationDeclaration::Check(check) => check.check.as_str(),
        })
        .collect::<std::collections::BTreeSet<_>>();
    if !declared_checks.contains(request.check_id.as_str()) {
        return Err(error(&root, "task_authorization_batch_check_not_declared"));
    }
    let events = read_batch_member_events(&root, &receipt, &member, true)?;
    ensure_child_grant_not_terminated(&root, &events)?;
    if !events.iter().any(|event| {
        event.kind == cockpit_protocol::TaskAuthorizationBatchEventKind::StartReserved
            && event.operation_id == request.operation_id
    }) {
        return Err(error(&root, "task_authorization_batch_start_required"));
    }
    let reservations = events
        .iter()
        .filter(|event| {
            event.kind == cockpit_protocol::TaskAuthorizationBatchEventKind::CheckAttemptReserved
                && event.operation_id == request.operation_id
                && event.check_id.as_deref() == Some(&request.check_id)
        })
        .collect::<Vec<_>>();
    let next_attempt = (reservations.len() + 1) as u32;
    for (index, reservation) in reservations.iter().enumerate() {
        if reservation.attempt != Some((index + 1) as u32) {
            return Err(error(
                &root,
                "task_authorization_batch_attempt_sequence_invalid",
            ));
        }
    }
    if let Some(previous) = reservations.last() {
        let outcome = latest_check_outcome(
            &events,
            &request.operation_id,
            &request.check_id,
            next_attempt - 1,
        );
        match outcome {
            None => {
                return Err(error(
                    &root,
                    "task_authorization_batch_attempt_outcome_unresolved",
                ));
            }
            Some(cockpit_protocol::TaskAuthorizationBatchAttemptOutcome::Unknown) => {
                return Err(error(
                    &root,
                    "task_authorization_batch_uncertain_outcome_requires_inspection",
                ));
            }
            Some(cockpit_protocol::TaskAuthorizationBatchAttemptOutcome::Passed) => {
                return Err(error(
                    &root,
                    "task_authorization_batch_check_already_passed",
                ));
            }
            Some(
                cockpit_protocol::TaskAuthorizationBatchAttemptOutcome::Failed
                | cockpit_protocol::TaskAuthorizationBatchAttemptOutcome::NotStarted,
            ) => {}
        }
        if previous.attempt.is_none() {
            return Err(error(&root, "task_authorization_batch_attempt_invalid"));
        }
    }
    let additional_attempts = next_attempt.saturating_sub(1) as u8;
    let permitted = member
        .budget
        .retry_budget_by_check
        .get(&request.check_id)
        .copied()
        .unwrap_or_default();
    if additional_attempts > permitted {
        return Err(error(
            &root,
            "task_authorization_batch_retry_budget_exhausted",
        ));
    }
    let event = new_batch_event(
        &root,
        &receipt,
        &member,
        BatchEventDraft {
            operation_id: request.operation_id.clone(),
            kind: cockpit_protocol::TaskAuthorizationBatchEventKind::CheckAttemptReserved,
            action: None,
            check_id: Some(request.check_id.clone()),
            attempt: Some(next_attempt),
            outcome: None,
            evidence: None,
            reason: format!("reserve declared check attempt {next_attempt}"),
        },
    )?;
    let persisted = append_batch_member_event(&root, &member, event)?;
    guard.require_subject(&root, &member.work_item_id)?;
    Ok(persisted)
}

/// Append the result of a reserved check attempt. Failed records stay in the
/// journal when a bounded retry is later admitted.
pub fn record_task_authorization_batch_check_outcome(
    root: &Path,
    request: &cockpit_protocol::TaskAuthorizationBatchOutcomeRequest,
    runtime: &RuntimeContext,
) -> Result<cockpit_protocol::TaskAuthorizationBatchEvent, ObserverError> {
    let (root, receipt, member, guard) = lock_and_load_batch_member(
        root,
        &request.manifest_digest,
        &request.work_item_id,
        runtime,
    )?;
    validate_batch_caller(&root, &member, &request.caller)?;
    validate_operation_id(&root, &request.operation_id)?;
    validate_batch_evidence(&root, &request.evidence_ref, &request.reason)?;
    if request.schema_version != cockpit_protocol::TASK_AUTHORIZATION_BATCH_SCHEMA_VERSION
        || request.attempt == 0
        || request.check_id.trim().is_empty()
        || request.check_id.trim() != request.check_id
        || request.outcome == cockpit_protocol::TaskAuthorizationBatchAttemptOutcome::NotStarted
    {
        return Err(error(
            &root,
            "task_authorization_batch_outcome_request_invalid",
        ));
    }
    let events = read_batch_member_events(&root, &receipt, &member, true)?;
    ensure_reserved_check_attempt(
        &root,
        &events,
        &request.operation_id,
        &request.check_id,
        request.attempt,
    )?;
    if let Some(existing) = find_check_result(
        &events,
        &request.operation_id,
        &request.check_id,
        request.attempt,
    ) {
        if existing.kind == cockpit_protocol::TaskAuthorizationBatchEventKind::CheckOutcomeRecorded
            && existing.outcome == Some(request.outcome)
            && existing.evidence_ref.as_deref() == Some(&request.evidence_ref)
            && existing.evidence_digest.as_ref() == Some(&request.evidence_digest)
            && existing.reason == request.reason
        {
            return Ok(existing.clone());
        }
        return Err(error(
            &root,
            "task_authorization_batch_attempt_result_conflict",
        ));
    }
    let event = new_batch_event(
        &root,
        &receipt,
        &member,
        BatchEventDraft {
            operation_id: request.operation_id.clone(),
            kind: cockpit_protocol::TaskAuthorizationBatchEventKind::CheckOutcomeRecorded,
            action: None,
            check_id: Some(request.check_id.clone()),
            attempt: Some(request.attempt),
            outcome: Some(request.outcome),
            evidence: Some((
                request.evidence_ref.clone(),
                request.evidence_digest.clone(),
            )),
            reason: request.reason.clone(),
        },
    )?;
    let persisted = append_batch_member_event(&root, &member, event)?;
    guard.require_subject(&root, &member.work_item_id)?;
    Ok(persisted)
}

/// Resolve an uncertain result only after inspecting the exact attempt and
/// attaching evidence. The old `unknown` event remains immutable.
pub fn inspect_task_authorization_batch_check_outcome(
    root: &Path,
    request: &cockpit_protocol::TaskAuthorizationBatchInspectionRequest,
    runtime: &RuntimeContext,
) -> Result<cockpit_protocol::TaskAuthorizationBatchEvent, ObserverError> {
    let (root, receipt, member, guard) = lock_and_load_batch_member(
        root,
        &request.manifest_digest,
        &request.work_item_id,
        runtime,
    )?;
    validate_batch_caller(&root, &member, &request.caller)?;
    validate_operation_id(&root, &request.operation_id)?;
    validate_batch_evidence(&root, &request.evidence_ref, &request.reason)?;
    if request.schema_version != cockpit_protocol::TASK_AUTHORIZATION_BATCH_SCHEMA_VERSION
        || request.attempt == 0
        || request.check_id.trim().is_empty()
        || request.check_id.trim() != request.check_id
        || request.inspected_outcome
            == cockpit_protocol::TaskAuthorizationBatchAttemptOutcome::Unknown
    {
        return Err(error(
            &root,
            "task_authorization_batch_inspection_request_invalid",
        ));
    }
    let events = read_batch_member_events(&root, &receipt, &member, true)?;
    ensure_reserved_check_attempt(
        &root,
        &events,
        &request.operation_id,
        &request.check_id,
        request.attempt,
    )?;
    match find_check_result(
        &events,
        &request.operation_id,
        &request.check_id,
        request.attempt,
    ) {
        Some(existing)
            if existing.kind
                == cockpit_protocol::TaskAuthorizationBatchEventKind::CheckOutcomeInspected
                && existing.outcome == Some(request.inspected_outcome)
                && existing.evidence_ref.as_deref() == Some(&request.evidence_ref)
                && existing.evidence_digest.as_ref() == Some(&request.evidence_digest)
                && existing.reason == request.reason =>
        {
            return Ok(existing.clone());
        }
        Some(existing)
            if existing.kind
                == cockpit_protocol::TaskAuthorizationBatchEventKind::CheckOutcomeRecorded
                && existing.outcome
                    == Some(cockpit_protocol::TaskAuthorizationBatchAttemptOutcome::Unknown) =>
        {
            // Inspection is the only route from an unknown outcome to a safe
            // retry or a resolved pass/fail result.
        }
        _ => {
            return Err(error(
                &root,
                "task_authorization_batch_attempt_is_not_uncertain",
            ));
        }
    }
    let event = new_batch_event(
        &root,
        &receipt,
        &member,
        BatchEventDraft {
            operation_id: request.operation_id.clone(),
            kind: cockpit_protocol::TaskAuthorizationBatchEventKind::CheckOutcomeInspected,
            action: None,
            check_id: Some(request.check_id.clone()),
            attempt: Some(request.attempt),
            outcome: Some(request.inspected_outcome),
            evidence: Some((
                request.evidence_ref.clone(),
                request.evidence_digest.clone(),
            )),
            reason: request.reason.clone(),
        },
    )?;
    let persisted = append_batch_member_event(&root, &member, event)?;
    guard.require_subject(&root, &member.work_item_id)?;
    Ok(persisted)
}

/// Terminate only the selected child grant. Expiry remains a derived state;
/// termination appends a receipt and never edits another member's ledger.
pub fn terminate_task_authorization_batch_member(
    root: &Path,
    request: &cockpit_protocol::TaskAuthorizationBatchTerminationRequest,
    runtime: &RuntimeContext,
) -> Result<cockpit_protocol::TaskAuthorizationBatchEvent, ObserverError> {
    let (root, receipt, member, guard) = lock_and_load_batch_member(
        root,
        &request.manifest_digest,
        &request.work_item_id,
        runtime,
    )?;
    validate_batch_caller(&root, &member, &request.caller)?;
    validate_batch_evidence(&root, "native-task-authorization-batch", &request.reason)?;
    if request.schema_version != cockpit_protocol::TASK_AUTHORIZATION_BATCH_SCHEMA_VERSION {
        return Err(error(
            &root,
            "task_authorization_batch_termination_request_invalid",
        ));
    }
    let events = read_batch_member_events(&root, &receipt, &member, true)?;
    if let Some(existing) = events
        .iter()
        .find(|event| event.kind == cockpit_protocol::TaskAuthorizationBatchEventKind::Terminated)
    {
        let decision = format!("{:?}", request.decision).to_lowercase();
        if existing.action.as_deref() == Some(decision.as_str())
            && existing.reason == request.reason
        {
            return Ok(existing.clone());
        }
        return Err(error(
            &root,
            "task_authorization_batch_termination_conflict",
        ));
    }
    let operation_id = format!("terminate-{}", member.work_item_id);
    let decision = format!("{:?}", request.decision).to_lowercase();
    let event = new_batch_event(
        &root,
        &receipt,
        &member,
        BatchEventDraft {
            operation_id,
            kind: cockpit_protocol::TaskAuthorizationBatchEventKind::Terminated,
            action: Some(decision),
            check_id: None,
            attempt: None,
            outcome: None,
            evidence: None,
            reason: request.reason.clone(),
        },
    )?;
    let persisted = append_batch_member_event(&root, &member, event)?;
    guard.require_subject(&root, &member.work_item_id)?;
    Ok(persisted)
}

/// Read the exact child grant, quota usage, and append-only per-member events.
pub fn inspect_task_authorization_batch_member(
    root: &Path,
    manifest_digest: &cockpit_core::Digest,
    work_item_id: &str,
    runtime: &RuntimeContext,
) -> Result<cockpit_protocol::TaskAuthorizationBatchMemberLedger, ObserverError> {
    let (root, receipt, member, guard) =
        lock_and_load_batch_member(root, manifest_digest, work_item_id, runtime)?;
    let events = read_batch_member_events(&root, &receipt, &member, false)?;
    let starts_used = events
        .iter()
        .filter(|event| {
            event.kind == cockpit_protocol::TaskAuthorizationBatchEventKind::StartReserved
        })
        .count() as u32;
    let now = Utc::now();
    let expiry = DateTime::parse_from_rfc3339(&member.expires_at)
        .map_err(|source| error(&root, format!("invalid batch member expiry: {source}")))?;
    let parent_state = current_member_authorization_state(&root, &member, runtime)?;
    let terminal_state = events
        .iter()
        .find(|event| event.kind == cockpit_protocol::TaskAuthorizationBatchEventKind::Terminated)
        .and_then(|event| event.action.as_deref());
    let (authorization_state, next_action) = if expiry <= now {
        ("expired", "do_not_start")
    } else if let Some(state) = terminal_state {
        (state, "do_not_start")
    } else if parent_state != "active" {
        (parent_state.as_str(), "do_not_start")
    } else {
        ("active", "continue_exact_member")
    };
    let mut blockers = Vec::new();
    if authorization_state != "active" {
        blockers.push(format!(
            "task_authorization_batch_member_{authorization_state}"
        ));
    }
    if starts_used >= member.budget.start_quota {
        blockers.push("task_authorization_batch_start_quota_exhausted".into());
    }
    if has_unresolved_attempt(&events) {
        blockers.push("task_authorization_batch_attempt_outcome_unresolved".into());
    }
    guard.require_subject(&root, work_item_id)?;
    Ok(cockpit_protocol::TaskAuthorizationBatchMemberLedger {
        schema_version: cockpit_protocol::TASK_AUTHORIZATION_BATCH_SCHEMA_VERSION,
        repository_id: receipt.repository_id,
        work_item_id: member.work_item_id.clone(),
        manifest_digest: receipt.manifest.manifest_digest,
        grant_id: receipt
            .child_grants
            .iter()
            .find(|grant| grant.work_item_id == member.work_item_id)
            .expect("receipt validates every manifest grant")
            .grant_id
            .clone(),
        authorization_state: authorization_state.into(),
        starts_used,
        start_quota: member.budget.start_quota,
        events,
        blockers,
        next_action: next_action.into(),
    })
}

fn lock_and_load_batch_member(
    root: &Path,
    manifest_digest: &cockpit_core::Digest,
    work_item_id: &str,
    _runtime: &RuntimeContext,
) -> Result<
    (
        PathBuf,
        cockpit_protocol::TaskAuthorizationBatchReceipt,
        cockpit_protocol::TaskAuthorizationBatchMember,
        super::LifecycleLockGuard,
    ),
    ObserverError,
> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    super::validate_work_item_id(work_item_id)?;
    let guard = acquire_lifecycle_lock(&root, work_item_id)?;
    let receipt = read_task_authorization_batch_receipt(&root, manifest_digest)?;
    let member = receipt
        .manifest
        .members
        .iter()
        .find(|member| member.work_item_id == work_item_id)
        .cloned()
        .ok_or_else(|| error(&root, "task_authorization_batch_member_not_selected"))?;
    let grant = receipt
        .child_grants
        .iter()
        .find(|grant| grant.work_item_id == work_item_id)
        .ok_or_else(|| error(&root, "task_authorization_batch_child_grant_missing"))?;
    if grant.member != member || grant.manifest_digest != *manifest_digest {
        return Err(error(
            &root,
            "task_authorization_batch_child_grant_mismatch",
        ));
    }
    guard.require_subject(&root, work_item_id)?;
    Ok((root, receipt, member, guard))
}

fn read_task_authorization_batch_receipt(
    root: &Path,
    digest: &cockpit_core::Digest,
) -> Result<cockpit_protocol::TaskAuthorizationBatchReceipt, ObserverError> {
    let (pinned, directory, path) = open_task_authorization_batch_directory(root, false)?;
    let name = batch_receipt_name(digest).map_err(|source| error(&path, source))?;
    let receipt_path = path.join(&name);
    let receipt = read_batch_receipt_file(root, &directory, &receipt_path, &name)?
        .ok_or_else(|| error(&receipt_path, "task_authorization_batch_receipt_missing"))?;
    if receipt.manifest.manifest_digest != *digest {
        return Err(error(
            &receipt_path,
            "task_authorization_batch_digest_mismatch",
        ));
    }
    pinned.require_current()?;
    Ok(receipt)
}

fn current_member_contract(
    root: &Path,
    member: &cockpit_protocol::TaskAuthorizationBatchMember,
) -> Result<Contract, ObserverError> {
    let relative = own_path(root, &member.work_item_id, "contract")?;
    if !relative.starts_with(".ai/work-items/active") {
        return Err(error(root, "task_authorization_batch_member_not_active"));
    }
    let value = native_json(root, &relative)?;
    let digest =
        cockpit_protocol::digest_json(&value).map_err(|source| error(root, source.to_string()))?;
    if digest != member.contract_digest
        || value["repositoryId"] != super::repository_id(root).to_string()
        || value["workItemId"] != member.work_item_id
    {
        return Err(error(
            root,
            "task_authorization_batch_contract_binding_changed",
        ));
    }
    serde_json::from_value(value).map_err(|source| error(root, source.to_string()))
}

fn current_member_authorization_state(
    root: &Path,
    member: &cockpit_protocol::TaskAuthorizationBatchMember,
    runtime: &RuntimeContext,
) -> Result<String, ObserverError> {
    let relative = own_path(root, &member.work_item_id, "contract")?;
    if !relative.starts_with(".ai/work-items/active") {
        return Ok("archived".into());
    }
    let value = native_json(root, &relative)?;
    let digest =
        cockpit_protocol::digest_json(&value).map_err(|source| error(root, source.to_string()))?;
    if digest != member.contract_digest
        || value["repositoryId"] != super::repository_id(root).to_string()
        || value["workItemId"] != member.work_item_id
    {
        return Ok("contract_changed".into());
    }
    let contract: Contract =
        serde_json::from_value(value).map_err(|source| error(root, source.to_string()))?;
    let policy = contract
        .task_authorization_policy()
        .map_err(|source| error(root, source))?
        .ok_or_else(|| error(root, "task_authorization_batch_parent_policy_missing"))?;
    let projection = assess(root, &member.work_item_id, &contract, &policy, runtime)?;
    if projection.grant_id.as_ref() != Some(&member.parent_grant_id)
        || projection.scope_digest.as_ref() != Some(&member.scope_digest)
        || policy.agent_actor != member.actor
        || policy.authority_source != member.authority_source
    {
        return Ok("parent_grant_changed".into());
    }
    if projection.state != "active" {
        return Ok(format!("parent_{}", projection.state));
    }
    let expiry = DateTime::parse_from_rfc3339(&member.expires_at)
        .map_err(|source| error(root, format!("invalid batch member expiry: {source}")))?;
    if expiry <= Utc::now() {
        return Ok("expired".into());
    }
    if let Some(parent_expiry) = policy.expires_at.as_deref()
        && DateTime::parse_from_rfc3339(parent_expiry)
            .map_err(|source| error(root, format!("invalid parent grant expiry: {source}")))?
            <= Utc::now()
    {
        return Ok("parent_expired".into());
    }
    Ok("active".into())
}

fn require_active_batch_member(
    root: &Path,
    member: &cockpit_protocol::TaskAuthorizationBatchMember,
    runtime: &RuntimeContext,
) -> Result<(), ObserverError> {
    let state = current_member_authorization_state(root, member, runtime)?;
    if state == "active" {
        Ok(())
    } else {
        Err(error(
            root,
            format!("task_authorization_batch_member_{state}"),
        ))
    }
}

fn validate_batch_caller(
    root: &Path,
    member: &cockpit_protocol::TaskAuthorizationBatchMember,
    caller: &cockpit_protocol::TaskAuthorizationCaller,
) -> Result<(), ObserverError> {
    if caller.schema_version != 1
        || caller.actor != member.actor
        || caller.authority_source != member.authority_source
        || caller.assurance != cockpit_protocol::AssuranceLevel::SelfDeclared
    {
        return Err(error(root, "task_authorization_batch_caller_mismatch"));
    }
    Ok(())
}

fn validate_operation_id(root: &Path, operation_id: &str) -> Result<(), ObserverError> {
    if operation_id.trim().is_empty()
        || operation_id.trim() != operation_id
        || operation_id.len() > 512
        || operation_id.chars().any(char::is_control)
    {
        return Err(error(root, "task_authorization_batch_operation_id_invalid"));
    }
    Ok(())
}

fn validate_batch_evidence(
    root: &Path,
    evidence_ref: &str,
    reason: &str,
) -> Result<(), ObserverError> {
    if evidence_ref.trim().is_empty()
        || evidence_ref.trim() != evidence_ref
        || evidence_ref.len() > 4096
        || reason.trim().is_empty()
        || reason.len() > 16 * 1024
    {
        return Err(error(root, "task_authorization_batch_evidence_invalid"));
    }
    Ok(())
}

struct BatchEventDraft {
    operation_id: String,
    kind: cockpit_protocol::TaskAuthorizationBatchEventKind,
    action: Option<String>,
    check_id: Option<String>,
    attempt: Option<u32>,
    outcome: Option<cockpit_protocol::TaskAuthorizationBatchAttemptOutcome>,
    evidence: Option<(String, cockpit_core::Digest)>,
    reason: String,
}

fn new_batch_event(
    root: &Path,
    receipt: &cockpit_protocol::TaskAuthorizationBatchReceipt,
    member: &cockpit_protocol::TaskAuthorizationBatchMember,
    draft: BatchEventDraft,
) -> Result<cockpit_protocol::TaskAuthorizationBatchEvent, ObserverError> {
    let grant_id = receipt
        .child_grants
        .iter()
        .find(|grant| grant.work_item_id == member.work_item_id)
        .ok_or_else(|| error(root, "task_authorization_batch_child_grant_missing"))?
        .grant_id
        .clone();
    let mut event = cockpit_protocol::TaskAuthorizationBatchEvent {
        schema_version: cockpit_protocol::TASK_AUTHORIZATION_BATCH_SCHEMA_VERSION,
        repository_id: receipt.repository_id.clone(),
        work_item_id: member.work_item_id.clone(),
        manifest_digest: receipt.manifest.manifest_digest.clone(),
        grant_id,
        sequence: 0,
        operation_id: draft.operation_id,
        kind: draft.kind,
        action: draft.action,
        check_id: draft.check_id,
        attempt: draft.attempt,
        outcome: draft.outcome,
        evidence_ref: draft
            .evidence
            .as_ref()
            .map(|(reference, _)| reference.clone()),
        evidence_digest: draft.evidence.map(|(_, digest)| digest),
        reason: draft.reason,
        recorded_at: super::now(),
        record_digest: cockpit_core::Digest::sha256_bytes(b""),
    };
    event.record_digest = event
        .computed_digest()
        .map_err(|source| error(root, source))?;
    Ok(event)
}

fn open_batch_member_event_directory(
    root: &Path,
    manifest_digest: &cockpit_core::Digest,
    work_item_id: &str,
    create: bool,
) -> Result<(PinnedRoot, Dir, PathBuf), ObserverError> {
    let (pinned, mut directory, mut path) = open_task_authorization_batch_directory(root, create)?;
    let digest_name = batch_receipt_name(manifest_digest)
        .map_err(|source| error(&path, source))?
        .trim_end_matches(".json")
        .to_owned();
    directory = open_batch_child_directory(&directory, &path, &digest_name, create)?;
    path.push(&digest_name);
    directory = open_batch_child_directory(&directory, &path, "members", create)?;
    path.push("members");
    directory = open_batch_child_directory(&directory, &path, work_item_id, create)?;
    path.push(work_item_id);
    directory = open_batch_child_directory(&directory, &path, "events", create)?;
    path.push("events");
    pinned.require_current()?;
    Ok((pinned, directory, path))
}

fn open_batch_child_directory(
    parent: &Dir,
    parent_path: &Path,
    name: &str,
    create: bool,
) -> Result<Dir, ObserverError> {
    let path = parent_path.join(name);
    if create {
        create_and_open_cap_directory(parent, name, &path)
    } else {
        open_cap_directory_nofollow_strict(parent, name, &path)
    }
}

fn read_batch_member_events(
    root: &Path,
    receipt: &cockpit_protocol::TaskAuthorizationBatchReceipt,
    member: &cockpit_protocol::TaskAuthorizationBatchMember,
    create: bool,
) -> Result<Vec<cockpit_protocol::TaskAuthorizationBatchEvent>, ObserverError> {
    let (_, _, batch_path) = open_task_authorization_batch_directory(root, false)?;
    let event_path = batch_path
        .join(
            batch_receipt_name(&receipt.manifest.manifest_digest)
                .map_err(|source| error(&batch_path, source))?
                .trim_end_matches(".json"),
        )
        .join("members")
        .join(&member.work_item_id)
        .join("events");
    if !create {
        match fs::symlink_metadata(&event_path) {
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(source) => {
                return Err(ObserverError::Read {
                    path: event_path,
                    source,
                });
            }
            Ok(metadata) if !metadata.is_dir() || metadata.file_type().is_symlink() => {
                return Err(error(
                    &event_path,
                    "task_authorization_batch_event_directory_invalid",
                ));
            }
            Ok(_) => {}
        }
    }
    let (pinned, directory, path) = open_batch_member_event_directory(
        root,
        &receipt.manifest.manifest_digest,
        &member.work_item_id,
        create,
    )?;
    let entries = directory.entries().map_err(|source| ObserverError::Read {
        path: path.clone(),
        source,
    })?;
    let mut names = Vec::new();
    for (index, entry) in entries.enumerate() {
        if index >= TASK_AUTHORIZATION_BATCH_MAX_EVENTS {
            return Err(error(
                &path,
                "task_authorization_batch_event_budget_exceeded",
            ));
        }
        let entry = entry.map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        let name = entry.file_name();
        let name = name
            .to_str()
            .ok_or_else(|| error(&path, "task_authorization_batch_event_filename_invalid"))?;
        if !name.ends_with(".json") {
            return Err(error(
                &path,
                "task_authorization_batch_event_filename_invalid",
            ));
        }
        names.push(name.to_owned());
    }
    names.sort();
    let mut events = Vec::with_capacity(names.len());
    for name in names {
        let bytes = read_regular_cap_bounded(
            &directory,
            &name,
            &path.join(&name),
            TASK_AUTHORIZATION_BATCH_MAX_BYTES,
        )?;
        super::reject_duplicate_json_keys(&bytes).map_err(|source| {
            error(
                &path,
                format!("task_authorization_batch_event_invalid: {source}"),
            )
        })?;
        let event: cockpit_protocol::TaskAuthorizationBatchEvent = serde_json::from_slice(&bytes)
            .map_err(|source| {
            error(
                &path,
                format!("task_authorization_batch_event_invalid: {source}"),
            )
        })?;
        event
            .validate_integrity()
            .map_err(|source| error(&path, source))?;
        let expected = batch_event_name(event.sequence, &event.record_digest)
            .map_err(|source| error(&path, source))?;
        if name != expected
            || event.repository_id != receipt.repository_id
            || event.work_item_id != member.work_item_id
            || event.manifest_digest != receipt.manifest.manifest_digest
            || event.grant_id
                != receipt
                    .child_grants
                    .iter()
                    .find(|grant| grant.work_item_id == member.work_item_id)
                    .ok_or_else(|| error(&path, "task_authorization_batch_child_grant_missing"))?
                    .grant_id
            || event.sequence != (events.len() + 1) as u64
        {
            return Err(error(&path, "task_authorization_batch_event_chain_invalid"));
        }
        events.push(event);
    }
    pinned.require_current()?;
    Ok(events)
}

fn append_batch_member_event(
    root: &Path,
    member: &cockpit_protocol::TaskAuthorizationBatchMember,
    mut event: cockpit_protocol::TaskAuthorizationBatchEvent,
) -> Result<cockpit_protocol::TaskAuthorizationBatchEvent, ObserverError> {
    let receipt = read_task_authorization_batch_receipt(root, &event.manifest_digest)?;
    let events = read_batch_member_events(root, &receipt, member, true)?;
    if events.len() >= TASK_AUTHORIZATION_BATCH_MAX_EVENTS {
        return Err(error(
            root,
            "task_authorization_batch_event_budget_exceeded",
        ));
    }
    event.sequence = (events.len() + 1) as u64;
    event.record_digest = event
        .computed_digest()
        .map_err(|source| error(root, source))?;
    let (pinned, directory, path) =
        open_batch_member_event_directory(root, &event.manifest_digest, &event.work_item_id, true)?;
    let name = batch_event_name(event.sequence, &event.record_digest)
        .map_err(|source| error(&path, source))?;
    let bytes = serde_json::to_vec_pretty(&event)
        .map_err(|source| error(&path, format!("serialize batch event: {source}")))?;
    write_cap_immutable(&directory, &name, &path.join(&name), &bytes)?;
    pinned.require_current()?;
    let refreshed = read_batch_member_events(root, &receipt, member, false)?;
    if refreshed.last() != Some(&event) {
        return Err(error(
            &path,
            "task_authorization_batch_event_readback_mismatch",
        ));
    }
    Ok(event)
}

fn batch_event_name(sequence: u64, digest: &cockpit_core::Digest) -> Result<String, String> {
    let digest = digest
        .as_str()
        .strip_prefix("sha256:")
        .ok_or_else(|| "task authorization batch event digest prefix is invalid".to_owned())?;
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("task authorization batch event digest is invalid".into());
    }
    Ok(format!("{sequence:08}-{digest}.json"))
}

fn find_check_result<'a>(
    events: &'a [cockpit_protocol::TaskAuthorizationBatchEvent],
    operation_id: &str,
    check_id: &str,
    attempt: u32,
) -> Option<&'a cockpit_protocol::TaskAuthorizationBatchEvent> {
    events.iter().rev().find(|event| {
        matches!(
            event.kind,
            cockpit_protocol::TaskAuthorizationBatchEventKind::CheckOutcomeRecorded
                | cockpit_protocol::TaskAuthorizationBatchEventKind::CheckOutcomeInspected
        ) && event.operation_id == operation_id
            && event.check_id.as_deref() == Some(check_id)
            && event.attempt == Some(attempt)
    })
}

fn latest_check_outcome(
    events: &[cockpit_protocol::TaskAuthorizationBatchEvent],
    operation_id: &str,
    check_id: &str,
    attempt: u32,
) -> Option<cockpit_protocol::TaskAuthorizationBatchAttemptOutcome> {
    find_check_result(events, operation_id, check_id, attempt).and_then(|event| event.outcome)
}

fn ensure_reserved_check_attempt(
    root: &Path,
    events: &[cockpit_protocol::TaskAuthorizationBatchEvent],
    operation_id: &str,
    check_id: &str,
    attempt: u32,
) -> Result<(), ObserverError> {
    if events.iter().any(|event| {
        event.kind == cockpit_protocol::TaskAuthorizationBatchEventKind::CheckAttemptReserved
            && event.operation_id == operation_id
            && event.check_id.as_deref() == Some(check_id)
            && event.attempt == Some(attempt)
    }) {
        Ok(())
    } else {
        Err(error(root, "task_authorization_batch_attempt_not_reserved"))
    }
}

fn ensure_child_grant_not_terminated(
    root: &Path,
    events: &[cockpit_protocol::TaskAuthorizationBatchEvent],
) -> Result<(), ObserverError> {
    if events
        .iter()
        .any(|event| event.kind == cockpit_protocol::TaskAuthorizationBatchEventKind::Terminated)
    {
        Err(error(root, "task_authorization_batch_member_terminated"))
    } else {
        Ok(())
    }
}

fn has_unresolved_attempt(events: &[cockpit_protocol::TaskAuthorizationBatchEvent]) -> bool {
    events
        .iter()
        .filter(|event| {
            event.kind == cockpit_protocol::TaskAuthorizationBatchEventKind::CheckAttemptReserved
        })
        .any(|reservation| {
            reservation
                .check_id
                .as_deref()
                .zip(reservation.attempt)
                .is_none_or(|(check, attempt)| {
                    matches!(
                        latest_check_outcome(events, &reservation.operation_id, check, attempt),
                        None | Some(
                            cockpit_protocol::TaskAuthorizationBatchAttemptOutcome::Unknown
                        )
                    )
                })
        })
}

fn request_from_manifest(
    manifest: &cockpit_protocol::TaskAuthorizationBatchManifest,
) -> cockpit_protocol::TaskAuthorizationBatchRequest {
    cockpit_protocol::TaskAuthorizationBatchRequest {
        schema_version: cockpit_protocol::TASK_AUTHORIZATION_BATCH_SCHEMA_VERSION,
        repository_id: manifest.repository_id.clone(),
        members: manifest
            .members
            .iter()
            .map(
                |member| cockpit_protocol::TaskAuthorizationBatchMemberRequest {
                    work_item_id: member.work_item_id.clone(),
                    actions: member.actions.clone(),
                    expires_at: member.expires_at.clone(),
                    budget: member.budget.clone(),
                },
            )
            .collect(),
    }
}

fn batch_receipt_name(digest: &cockpit_core::Digest) -> Result<String, String> {
    let value = digest
        .as_str()
        .strip_prefix("sha256:")
        .ok_or_else(|| "task authorization batch digest prefix is invalid".to_owned())?;
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("task authorization batch digest is invalid".into());
    }
    Ok(format!("{value}.json"))
}

fn open_task_authorization_batch_directory(
    root: &Path,
    create: bool,
) -> Result<(PinnedRoot, Dir, PathBuf), ObserverError> {
    let pinned = PinnedRoot::open(root)?;
    let root_directory = pinned.directory()?;
    let ai_path = root.join(".ai");
    let ai = open_cap_directory_nofollow_strict(&root_directory, ".ai", &ai_path)?;
    let evidence_path = ai_path.join("evidence");
    let evidence = open_cap_directory_nofollow_strict(&ai, "evidence", &evidence_path)?;
    let directory_path = evidence_path.join(TASK_AUTHORIZATION_BATCH_DIRECTORY);
    let directory = if create {
        create_and_open_cap_directory(
            &evidence,
            TASK_AUTHORIZATION_BATCH_DIRECTORY,
            &directory_path,
        )?
    } else {
        open_cap_directory_nofollow_strict(
            &evidence,
            TASK_AUTHORIZATION_BATCH_DIRECTORY,
            &directory_path,
        )?
    };
    pinned.require_current()?;
    Ok((pinned, directory, directory_path))
}

fn read_batch_receipt_file(
    root: &Path,
    directory: &Dir,
    path: &Path,
    name: &str,
) -> Result<Option<cockpit_protocol::TaskAuthorizationBatchReceipt>, ObserverError> {
    let bytes =
        match read_regular_cap_bounded(directory, name, path, TASK_AUTHORIZATION_BATCH_MAX_BYTES) {
            Ok(bytes) => bytes,
            Err(ObserverError::Read { source, .. })
                if source.kind() == std::io::ErrorKind::NotFound =>
            {
                return Ok(None);
            }
            Err(error) => return Err(error),
        };
    super::reject_duplicate_json_keys(&bytes).map_err(|source| {
        error(
            path,
            format!("task_authorization_batch_duplicate_key: {source}"),
        )
    })?;
    let receipt: cockpit_protocol::TaskAuthorizationBatchReceipt = serde_json::from_slice(&bytes)
        .map_err(|source| {
        error(
            path,
            format!("task_authorization_batch_receipt_invalid: {source}"),
        )
    })?;
    receipt
        .validate_integrity()
        .map_err(|source| error(path, source))?;
    if receipt.repository_id != super::repository_id(root).to_string()
        || batch_receipt_name(&receipt.manifest.manifest_digest)
            .map_err(|source| error(path, source))?
            != name
    {
        return Err(error(
            path,
            "task_authorization_batch_receipt_identity_mismatch",
        ));
    }
    Ok(Some(receipt))
}

fn error(path: &Path, message: impl Into<String>) -> ObserverError {
    ObserverError::State {
        path: path.into(),
        message: message.into(),
    }
}

fn own_path(root: &Path, wi: &str, suffix: &str) -> Result<PathBuf, ObserverError> {
    let active = PathBuf::from(format!(".ai/work-items/active/{wi}.{suffix}.json"));
    match std::fs::symlink_metadata(root.join(&active)) {
        Ok(_) => Ok(active), // native_json refuses symlinks rather than falling back.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(PathBuf::from(format!(
            ".ai/work-items/archive/{wi}.{suffix}.json"
        ))),
        Err(e) => Err(error(root, e.to_string())),
    }
}

/// All paths originate from the exact own-WI native layout, never caller filenames.
/// Pin and recheck every directory around the bounded, no-follow file read.
fn native_json(root: &Path, relative: &Path) -> Result<Value, ObserverError> {
    let pinned = PinnedRoot::open(root)?;
    let mut directory = pinned.directory()?;
    let mut path = root.to_path_buf();
    let names: Vec<_> = relative
        .components()
        .map(|component| match component {
            Component::Normal(name) => Ok(name.to_owned()),
            _ => Err(error(relative, "invalid task authority native path")),
        })
        .collect::<Result<_, _>>()?;
    let (name, parents) = names
        .split_last()
        .ok_or_else(|| error(relative, "missing native filename"))?;
    let mut identities = Vec::new();
    for parent in parents {
        path.push(parent);
        directory = super::open_cap_directory_nofollow_strict(
            &directory,
            &parent.to_string_lossy(),
            &path,
        )?;
        identities.push(super::legacy_closeout_source::directory_identity(
            &path, &directory,
        )?);
    }
    let file_path = path.join(name);
    let bytes = super::legacy_closeout_source::read_regular_cap_bounded(
        &directory,
        &name.to_string_lossy(),
        &file_path,
        8 * 1024 * 1024,
    )?;
    super::reject_duplicate_json_keys(&bytes).map_err(|e| error(&file_path, e))?;
    pinned.require_current()?;
    let mut current = pinned.directory()?;
    path = root.into();
    for (parent, identity) in parents.iter().zip(identities) {
        path.push(parent);
        current =
            super::open_cap_directory_nofollow_strict(&current, &parent.to_string_lossy(), &path)?;
        if super::legacy_closeout_source::directory_identity(&path, &current)? != identity {
            return Err(error(&path, "task authority parent identity changed"));
        }
    }
    serde_json::from_slice(&bytes).map_err(|e| error(&file_path, e.to_string()))
}

fn projection(
    policy: &TaskAuthorizationPolicy,
    state: &str,
    reason: &str,
) -> TaskAuthorizationProjection {
    TaskAuthorizationProjection {
        state: state.into(),
        assurance: AssuranceLevel::SelfDeclared,
        agent_actor: policy.agent_actor.clone(),
        authority_source: policy.authority_source.clone(),
        grant_id: None,
        scope_digest: None,
        expires_at: policy.expires_at.clone(),
        reason: reason.into(),
    }
}

fn assess(
    root: &Path,
    wi: &str,
    contract: &Contract,
    policy: &TaskAuthorizationPolicy,
    runtime: &RuntimeContext,
) -> Result<TaskAuthorizationProjection, ObserverError> {
    let contract_path = own_path(root, wi, "contract")?;
    let current = native_json(root, &contract_path)?;
    let current_digest =
        cockpit_protocol::digest_json(&current).map_err(|e| error(root, e.to_string()))?;
    if current["repositoryId"] != super::repository_id(root).to_string()
        || current["workItemId"] != wi
    {
        return Err(error(
            root,
            "task authorization repository or Work Item mismatch",
        ));
    }
    let summary = native_json(root, &own_path(root, wi, "summary")?)?;
    let mut baseline = current.clone();
    let mut value = summary.get("decisionEvidence").cloned();
    if value
        .as_ref()
        .is_none_or(|owner| owner["contractDigest"] != json!(current_digest))
    {
        if let Some((owner_contract, owner)) =
            super::contract_amendment::reviewed_task_owner_baseline(
                root,
                wi,
                &current_digest,
                runtime,
                &current,
            )?
        {
            let typed: Contract = serde_json::from_value(owner_contract.clone())
                .map_err(|e| error(root, e.to_string()))?;
            if typed
                .task_authorization_policy()
                .map_err(|e| error(root, e))?
                .as_ref()
                != Some(policy)
            {
                return Err(error(
                    root,
                    "task policy differs from the native owner baseline",
                ));
            }
            baseline = owner_contract;
            value = Some(owner);
        } else {
            return Ok(projection(
                policy,
                "owner_review_required",
                "The current task policy needs its exact native owner decision or validated mechanical lineage.",
            ));
        }
    }
    let value = value.ok_or_else(|| error(root, "native task owner input missing"))?;
    let baseline_digest =
        cockpit_protocol::digest_json(&baseline).map_err(|e| error(root, e.to_string()))?;
    let owner: PreflightDecisionEvidence = serde_json::from_value(value.clone())
        .map_err(|e| error(root, format!("invalid task owner review: {e}")))?;
    if owner.schema_version != 1
        || owner.decision_id != "contract-preflight-review"
        || owner.decision != "confirm_review"
        || owner.work_item_id != wi
        || owner.repository_id != contract.repository_id
        || owner.contract_digest != baseline_digest
        || owner.recorded_by.trim().is_empty()
        || owner.reason.trim().is_empty()
        || DateTime::parse_from_rfc3339(&owner.recorded_at).is_err()
    {
        return Err(error(
            root,
            "task owner review is foreign, stale or malformed",
        ));
    }
    let owner_digest =
        cockpit_protocol::digest_json(&value).map_err(|e| error(root, e.to_string()))?;
    let canonical = PathBuf::from(format!(".ai/decisions/{wi}.preflight-review.json"));
    let stored = native_json(root, &canonical)?;
    if stored != value {
        let indexed = PathBuf::from(format!(
            ".ai/decisions/{wi}.preflight-review.{}.json",
            owner_digest.to_string().trim_start_matches("sha256:")
        ));
        if native_json(root, &indexed)? != value {
            return Err(error(root, "native task owner receipt does not match"));
        }
    }
    let history = super::read_work_item_contract_amendments(root, wi)?;
    super::contract_amendment::validate_transition_chain(
        root,
        wi,
        &history,
        Some(&current_digest),
    )?;
    let scope = json!({
        "repositoryId":baseline["repositoryId"], "workItemId":wi, "goal":baseline["goal"],
        "scope":baseline["scope"], "outOfScope":baseline["outOfScope"],
        "verification":baseline["verification"], "acceptanceCriteria":baseline["acceptanceCriteria"],
    });
    let scope_digest =
        cockpit_protocol::digest_json(&scope).map_err(|e| error(root, e.to_string()))?;
    let mut result = projection(
        policy,
        "active",
        "The native owner-reviewed task boundary remains current; verification freshness is independent.",
    );
    result.grant_id = Some(
        cockpit_protocol::digest_json(&json!({
            "schemaVersion":1, "repositoryId":contract.repository_id, "workItemId":wi,
        "ownerContractDigest":owner.contract_digest, "policy":policy, "scopeDigest":scope_digest,
        }))
        .map_err(|e| error(root, e.to_string()))?,
    );
    result.scope_digest = Some(scope_digest);
    let deadline = policy
        .expires_at
        .as_deref()
        .map(DateTime::parse_from_rfc3339)
        .transpose()
        .map_err(|e| error(root, format!("invalid task authorization deadline: {e}")))?;
    if contract_path.starts_with(".ai/work-items/archive") {
        result.state = "delivery_only".into();
        result.reason = "Archived source is immutable; only the declared remaining delivery and resource lifecycle may continue.".into();
    }
    let terminal = read_termination(root, wi, &result, policy)?;
    let deadline_first = deadline.is_some_and(|deadline| {
        deadline_reached(deadline, Utc::now())
            && terminal.as_ref().is_none_or(|v| {
                DateTime::parse_from_rfc3339(v["recordedAt"].as_str().unwrap_or_default())
                    .is_ok_and(|at| deadline <= at)
            })
    });
    if deadline_first {
        result.state = "expired".into();
        result.reason = "The declared task authorization deadline has been reached.".into();
    } else if let Some(record) = terminal {
        result.state = record["request"]["decision"]
            .as_str()
            .unwrap_or_default()
            .into();
        result.reason =
            "An immutable bound termination ended this grant without changing business completion."
                .into();
    } else if contract_path.starts_with(".ai/work-items/archive")
        && super::close_decision_is_valid_for_status(root, wi, &contract.repository_id)
    {
        result.state = "closeout_only".into();
        result.reason = "The task is closed. Only explicitly preauthorized native closeout publication and auxiliary cleanup may continue; source work cannot reopen.".into();
    }
    Ok(result)
}

fn terminal_path(wi: &str, task: &TaskAuthorizationProjection) -> Result<PathBuf, ObserverError> {
    let grant = task
        .grant_id
        .as_ref()
        .ok_or_else(|| error(Path::new(wi), "task grant is not owner-bound"))?;
    Ok(PathBuf::from(format!(
        ".ai/decisions/{wi}.task-authorization.{}.terminated.json",
        grant.as_str().trim_start_matches("sha256:")
    )))
}

fn check_terminal_request(
    root: &Path,
    policy: &TaskAuthorizationPolicy,
    task: &TaskAuthorizationProjection,
    request: &TaskAuthorizationTerminationRequest,
) -> Result<(), ObserverError> {
    if request.schema_version != 1
        || Some(&request.grant_id) != task.grant_id.as_ref()
        || request.caller.schema_version != 1
        || request.caller.actor != policy.agent_actor
        || request.caller.authority_source != policy.authority_source
        || request.caller.assurance != AssuranceLevel::SelfDeclared
        || request.reason.trim().is_empty()
        || request.reason.len() > 16 * 1024
    {
        return Err(error(
            root,
            "task termination caller, grant or input mismatch",
        ));
    }
    Ok(())
}

fn read_termination(
    root: &Path,
    wi: &str,
    task: &TaskAuthorizationProjection,
    policy: &TaskAuthorizationPolicy,
) -> Result<Option<Value>, ObserverError> {
    let relative = terminal_path(wi, task)?;
    match std::fs::symlink_metadata(root.join(&relative)) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(error(root, e.to_string())),
        Ok(_) => {}
    }
    let mut value = native_json(root, &relative)?;
    let fields = [
        "schemaVersion",
        "repositoryId",
        "workItemId",
        "grantId",
        "scopeDigest",
        "request",
        "recordedAt",
        "runtimeVersion",
        "runtimeDigest",
        "recordDigest",
    ];
    if value.as_object().is_none_or(|object| {
        object.len() != fields.len() || object.keys().any(|key| !fields.contains(&key.as_str()))
    }) || value["runtimeVersion"]
        .as_str()
        .is_none_or(|version| version.trim().is_empty())
        || serde_json::from_value::<cockpit_core::Digest>(value["runtimeDigest"].clone()).is_err()
    {
        return Err(error(root, "invalid task termination record schema"));
    }
    let digest = value
        .as_object_mut()
        .and_then(|v| v.remove("recordDigest"))
        .ok_or_else(|| error(root, "task termination digest missing"))?;
    let actual = cockpit_protocol::digest_json(&value).map_err(|e| error(root, e.to_string()))?;
    let request: TaskAuthorizationTerminationRequest =
        serde_json::from_value(value["request"].clone()).map_err(|e| error(root, e.to_string()))?;
    check_terminal_request(root, policy, task, &request)?;
    if value["schemaVersion"] != 1
        || value["repositoryId"] != super::repository_id(root).to_string()
        || value["workItemId"] != wi
        || value["grantId"] != json!(task.grant_id)
        || value["scopeDigest"] != json!(task.scope_digest)
        || digest != json!(actual)
        || DateTime::parse_from_rfc3339(value["recordedAt"].as_str().unwrap_or_default()).is_err()
    {
        return Err(error(
            root,
            "task termination native binding or digest mismatch",
        ));
    }
    value["recordDigest"] = digest;
    Ok(Some(value))
}

/// Caller holds the own-WI lifecycle lock. This only appends narrowing metadata.
pub(crate) fn record_termination_unlocked(
    root: &Path,
    wi: &str,
    value: &Value,
    runtime: &RuntimeContext,
) -> Result<Value, ObserverError> {
    let request: TaskAuthorizationTerminationRequest = serde_json::from_value(value.clone())
        .map_err(|e| error(root, format!("invalid task termination input: {e}")))?;
    let contract_value = native_json(root, &own_path(root, wi, "contract")?)?;
    let contract: Contract =
        serde_json::from_value(contract_value).map_err(|e| error(root, e.to_string()))?;
    let policy = contract
        .task_authorization_policy()
        .map_err(|e| error(root, e))?
        .ok_or_else(|| error(root, "task termination has no explicit owner policy"))?;
    let task = assess(root, wi, &contract, &policy, runtime)?;
    check_terminal_request(root, &policy, &task, &request)?;
    if let Some(existing) = read_termination(root, wi, &task, &policy)? {
        if existing["request"] != *value {
            return Err(error(
                root,
                "task termination is immutable and conflicts with this request",
            ));
        }
        return Ok(existing);
    }
    let relative = terminal_path(wi, &task)?;
    let mut record = json!({
        "schemaVersion":1, "repositoryId":contract.repository_id, "workItemId":wi,
        "grantId":task.grant_id, "scopeDigest":task.scope_digest, "request":request,
        "recordedAt":Utc::now().to_rfc3339(), "runtimeVersion":runtime.runtime_version,
        "runtimeDigest":runtime.runtime_digest,
    });
    record["recordDigest"] =
        json!(cockpit_protocol::digest_json(&record).map_err(|e| error(root, e.to_string()))?);
    let pinned = PinnedRoot::open(root)?;
    let directory = pinned.directory()?;
    let ai_path = root.join(".ai");
    let ai = super::open_cap_directory_nofollow_strict(&directory, ".ai", &ai_path)?;
    let decisions_path = ai_path.join("decisions");
    let decisions = super::open_cap_directory_nofollow_strict(&ai, "decisions", &decisions_path)?;
    let identities = [
        super::legacy_closeout_source::directory_identity(&ai_path, &ai)?,
        super::legacy_closeout_source::directory_identity(&decisions_path, &decisions)?,
    ];
    let check_parent = || -> Result<(), ObserverError> {
        pinned.require_current()?;
        let current_root = pinned.directory()?;
        let current_ai = super::open_cap_directory_nofollow_strict(&current_root, ".ai", &ai_path)?;
        let current_decisions =
            super::open_cap_directory_nofollow_strict(&current_ai, "decisions", &decisions_path)?;
        if identities
            != [
                super::legacy_closeout_source::directory_identity(&ai_path, &current_ai)?,
                super::legacy_closeout_source::directory_identity(
                    &decisions_path,
                    &current_decisions,
                )?,
            ]
        {
            return Err(error(root, "task termination parent identity changed"));
        }
        Ok(())
    };
    check_parent()?;
    let bytes = serde_json::to_vec_pretty(&record).map_err(|e| error(root, e.to_string()))?;
    super::evidence_store::write_cap_immutable(
        &decisions,
        relative
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| error(root, "invalid termination filename"))?,
        &root.join(&relative),
        &bytes,
    )?;
    check_parent()?;
    let stored = read_termination(root, wi, &task, &policy)?;
    if stored.as_ref() != Some(&record) {
        return Err(error(root, "task termination publication is incomplete"));
    }
    Ok(record)
}

pub(crate) fn task_authorization_projection(
    root: &Path,
    wi: &str,
    contract: &Contract,
    runtime: &RuntimeContext,
) -> Result<Option<TaskAuthorizationProjection>, ObserverError> {
    let Some(policy) = contract
        .task_authorization_policy()
        .map_err(|e| error(root, e))?
    else {
        return Ok(None);
    };
    Ok(Some(
        assess(root, wi, contract, &policy, runtime)
            .unwrap_or_else(|e| projection(&policy, "invalid", &e.to_string())),
    ))
}

/// Explain current authority separately from verification and host admission.
/// Every write must still repeat its own checks; this result is never accepted as input.
pub fn assess_task_authorization(
    root: &Path,
    wi: &str,
    request: &cockpit_protocol::TaskAuthorizationRequest,
    runtime: &RuntimeContext,
) -> Result<cockpit_protocol::TaskAuthorizationAssessment, ObserverError> {
    super::validate_work_item_id(wi)?;
    let status = super::work_item_status_snapshot_with_runtime(root, wi, runtime)?;
    let checked = if request.schema_version != 1 {
        Err(error(root, "task_authorization_request_schema_invalid"))
    } else {
        require_task_caller(root, wi, &request.action, runtime, Some(&request.caller))
            .and_then(|()| check_request_intent(root, wi, request))
    };
    let authorized = checked.is_ok() && status.task_authorization.is_some();
    // Only these operations use the shared safeActions gate. Checkpoint and
    // resource transactions validate their input/phase in their own locked
    // entrypoints; absence from generic status is not a rejection of them.
    let runtime_action = [
        "run_preflight",
        "run_verification",
        "record_governance_controls",
        "record_material_review_decision",
        "finish",
        "archive_when_reviewed",
    ]
    .contains(&request.action.as_str());
    let native_operation = [
        "checkpoint",
        "finalize_resources",
        "record_finalization",
        "finalize_verify",
        "cleanup_auxiliary_resources",
    ]
    .contains(&request.action.as_str());
    let admitted = runtime_action.then(|| status.safe_actions.contains(&request.action));
    let (next, reason) = match checked {
        Err(e) => ("stop", e.to_string()),
        Ok(()) if !authorized => ("owner_review_required", "No explicit owner-bound task authorization is configured.".into()),
        Ok(()) if admitted == Some(false) => ("resolve_current_gate", "The task authority remains valid, but current Runtime admission rejects this operation. Follow its current safeActions; do not request the same task authorization again.".into()),
        Ok(()) if request.action == "run_verification" && status.evidence_freshness.state != "fresh" => ("reverify", "Authority remains valid. Execute current verification because evidence is missing or stale; source changes alone do not require new owner approval.".into()),
        Ok(()) if native_operation => ("check_operation_boundary", "Task authority permits this intent. The supported caller-bearing native operation checks its exact input and lifecycle phase under its own lock; generic safeActions does not project that input-specific admission. No prior query authorizes the effect.".into()),
        Ok(()) if admitted.is_none() => ("check_host_boundary", "The task boundary permits this intent. Host/provider permission and the actual operation's fresh checks remain independent.".into()),
        Ok(()) => ("continue", "The explicit task authority and current Runtime action admission both permit continuation.".into()),
    };
    Ok(cockpit_protocol::TaskAuthorizationAssessment {
        authority: status.task_authorization,
        requested_action: request.action.clone(),
        authorized,
        action_admitted: admitted,
        evidence_freshness: status.evidence_freshness,
        next_action: next.into(),
        reason,
    })
}

pub(crate) fn check_request_intent(
    root: &Path,
    wi: &str,
    request: &cockpit_protocol::TaskAuthorizationRequest,
) -> Result<(), ObserverError> {
    let root = std::fs::canonicalize(root).map_err(|e| error(root, e.to_string()))?;
    let relative = own_path(&root, wi, "contract")?;
    let contract: Contract = serde_json::from_value(native_json(&root, &relative)?)
        .map_err(|e| error(&root, e.to_string()))?;
    let policy = contract
        .task_authorization_policy()
        .map_err(|e| error(&root, e))?
        .ok_or_else(|| error(&root, "task_authorization_not_configured"))?;
    if !cockpit_protocol::TASK_AUTHORIZATION_ACTIONS.contains(&request.action.as_str())
        || (request.action == "modify_source" && request.paths.is_empty())
        || (request.public_target.is_some() && request.public_target != policy.public_target)
        || (cockpit_protocol::TASK_AUTHORIZATION_PUBLIC_ACTIONS.contains(&request.action.as_str())
            && request.public_target != policy.public_target)
    {
        return Err(error(&root, "task_authorization_intent_outside_scope"));
    }
    for path in &request.paths {
        // Require exact relative filenames, not caller-supplied patterns or aliases.
        if path.is_empty()
            || path.contains('\\')
            || path.contains(':')
            || path
                .split('/')
                .any(|part| part.is_empty() || part == "." || part == "..")
            || super::scope_pattern_has_glob(path)
            || !super::declared_scope_allows_path(&contract.scope, &contract.out_of_scope, path)
        {
            return Err(error(
                &root,
                format!("task_authorization_path_outside_scope: {path}"),
            ));
        }
        let mut current = root.clone();
        for part in Path::new(path).components() {
            current.push(part.as_os_str());
            match std::fs::symlink_metadata(&current) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(error(&current, "task_authorization_path_is_symlink"));
                }
                Ok(_) => (),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => break,
                Err(e) => return Err(error(&current, e.to_string())),
            }
        }
    }
    Ok(())
}

pub(crate) fn task_policy_configured(root: &Path, wi: &str) -> Result<bool, ObserverError> {
    let path = own_path(root, wi, "contract")?;
    match std::fs::symlink_metadata(root.join(&path)) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(error(root, e.to_string())),
        Ok(_) => (),
    }
    let contract: Contract = serde_json::from_value(native_json(root, &path)?)
        .map_err(|e| error(root, e.to_string()))?;
    Ok(contract
        .task_authorization_policy()
        .map_err(|e| error(root, e))?
        .is_some())
}

pub(crate) fn reject_legacy_task_write(root: &Path, wi: &str) -> Result<(), ObserverError> {
    if task_policy_configured(root, wi)? {
        return Err(error(
            root,
            "task_authorization_caller_required: legacy write has no Runtime or caller context",
        ));
    }
    Ok(())
}

pub(crate) fn require_task_caller(
    root: &Path,
    wi: &str,
    action: &str,
    runtime: &RuntimeContext,
    caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
) -> Result<(), ObserverError> {
    let canonical_root = std::fs::canonicalize(root).map_err(|e| error(root, e.to_string()))?;
    let root = canonical_root.as_path();
    let relative = own_path(root, wi, "contract")?;
    let contract: Contract = serde_json::from_value(native_json(root, &relative)?)
        .map_err(|e| error(root, e.to_string()))?;
    let Some(policy) = contract
        .task_authorization_policy()
        .map_err(|e| error(root, e))?
    else {
        return Ok(());
    };
    // These are separate owner or narrowing-metadata decisions, never
    // delegated operations. Their existing native evidence checks remain.
    if [
        "close",
        "close_after_review",
        "close_after_cleanup",
        "record_usage",
    ]
    .contains(&action)
    {
        return Ok(());
    }
    let task = assess(root, wi, &contract, &policy, runtime)?;
    if caller.is_none() && action == "run_preflight" && task.state == "owner_review_required" {
        // Generate only the missing exact owner-review observation. This
        // starts no checks and grants no source, verification or policy right.
        return Ok(());
    }
    let caller = caller.ok_or_else(|| {
        error(
            root,
            format!("task_authorization_caller_required: {}", task.state),
        )
    })?;
    if caller.schema_version != 1
        || caller.actor != policy.agent_actor
        || caller.authority_source != policy.authority_source
        || caller.assurance != AssuranceLevel::SelfDeclared
    {
        return Err(error(root, "task_authorization_caller_mismatch"));
    }
    let closeout = task.state == "closeout_only"
        && policy
            .closeout_actions
            .iter()
            .any(|allowed| allowed == action);
    let delivery = task.state == "delivery_only"
        && [
            "publish_branch",
            "create_pull_request",
            "ready_pull_request",
            "merge_pull_request",
            "verify_main",
            "finalize_resources",
            "record_finalization",
            "finalize_verify",
            "cleanup_resources",
            "publish_native_closeout",
            "cleanup_auxiliary_resources",
        ]
        .contains(&action);
    if task.state != "active" && !closeout && !delivery {
        return Err(error(root, format!("task_authorization_{}", task.state)));
    }
    if !policy
        .allowed_actions
        .iter()
        .any(|allowed| allowed == action)
    {
        return Err(error(root, "task_authorization_action_outside_scope"));
    }
    Ok(())
}

/// Reduce an already policy-authorized finite node budget. This function grants
/// no timeout extension or command permission; callers still validate both.
/// Re-evaluate before each child spawn, never cache it for the whole session.
pub fn task_authorization_timeout(
    root: &Path,
    wi: &str,
    runtime: &RuntimeContext,
    caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
    authorized_ceiling: u64,
) -> Result<u64, ObserverError> {
    if !(1..=cockpit_protocol::MAX_VERIFICATION_TIMEOUT_SECONDS).contains(&authorized_ceiling) {
        return Err(error(root, "task_authorization_invalid_execution_ceiling"));
    }
    require_task_caller(root, wi, "run_verification", runtime, caller)?;
    let root = std::fs::canonicalize(root).map_err(|e| error(root, e.to_string()))?;
    let contract: Contract =
        serde_json::from_value(native_json(&root, &own_path(&root, wi, "contract")?)?)
            .map_err(|e| error(&root, e.to_string()))?;
    let Some(deadline) = contract
        .task_authorization_policy()
        .map_err(|e| error(&root, e))?
        .and_then(|policy| policy.expires_at)
    else {
        return Ok(authorized_ceiling);
    };
    let deadline =
        DateTime::parse_from_rfc3339(&deadline).map_err(|e| error(&root, e.to_string()))?;
    let remaining = (deadline.with_timezone(&Utc) - Utc::now()).num_seconds();
    if remaining < 1 {
        return Err(error(
            &root,
            "task_authorization_deadline_before_next_spawn",
        ));
    }
    Ok(authorized_ceiling.min(remaining as u64))
}

/// Check the actual argv against the owner-reviewed declarations, including
/// the existing deterministic Cargo workspace-to-package expansion.
pub(crate) fn require_declared_task_command(
    root: &Path,
    request: &super::RepositoryVerificationRequest,
    runtime: &RuntimeContext,
    caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
) -> Result<(), ObserverError> {
    let Some(wi) = request.work_item_id.as_deref() else {
        return Ok(());
    };
    require_task_caller(root, wi, "run_verification", runtime, caller)?;
    let contract: Contract =
        serde_json::from_value(native_json(root, &own_path(root, wi, "contract")?)?)
            .map_err(|e| error(root, e.to_string()))?;
    if contract
        .task_authorization_policy()
        .map_err(|e| error(root, e))?
        .is_none()
    {
        return Ok(());
    }
    if request.runtime_digest != runtime.runtime_digest.to_string() {
        return Err(error(root, "task_authorization_command_runtime_mismatch"));
    }
    for declaration in &contract.verification {
        let raw = match declaration {
            cockpit_protocol::VerificationDeclaration::Legacy(command) => command,
            cockpit_protocol::VerificationDeclaration::Check(check) => &check.check,
        };
        let (program, args) =
            cockpit_protocol::parse_verification_argv(raw).map_err(|e| error(root, e))?;
        let replace = |token: String| {
            if token == "<repo>" {
                root.to_string_lossy().into_owned()
            } else {
                token
            }
        };
        let mut declared = request.clone();
        declared.program = replace(program);
        declared.args = args.into_iter().map(replace).collect();
        if declared.program == request.program && declared.args == request.args {
            return Ok(());
        }
        if declared.program == "cargo" && declared.args.iter().any(|arg| arg == "--workspace") {
            let plan = super::plan_repository_verification(root, &declared)?;
            if plan
                .requests
                .iter()
                .any(|planned| planned.program == request.program && planned.args == request.args)
            {
                return Ok(());
            }
        }
    }
    Err(error(
        root,
        "task_authorization_command_outside_owner_verification",
    ))
}

// Kept explicit so exact equality, offsets and the nanosecond boundary are
// tested without sleeps or changing the executing process clock.
fn deadline_reached(deadline: DateTime<chrono::FixedOffset>, current: DateTime<Utc>) -> bool {
    current >= deadline
}

#[cfg(test)]
mod deadline_tests {
    #[test]
    fn exact_declared_deadline_stops_authority_at_equality() {
        let deadline = chrono::DateTime::parse_from_rfc3339("2030-01-01T09:00:00+09:00").unwrap();
        let instant = deadline.with_timezone(&chrono::Utc);
        assert!(!super::deadline_reached(
            deadline,
            instant - chrono::Duration::nanoseconds(1)
        ));
        assert!(super::deadline_reached(deadline, instant));
        assert!(super::deadline_reached(
            deadline,
            instant + chrono::Duration::nanoseconds(1)
        ));
    }
}

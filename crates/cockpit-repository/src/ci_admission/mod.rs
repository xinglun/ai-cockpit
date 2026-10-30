//! Current-authority read-only plans. No acceptance or receipt-writing API.

mod github;
#[cfg(test)]
mod tests;

use super::*;
use cockpit_protocol::ci_admission::{
    GITHUB_PLAN_SCHEMA_VERSION, GithubAdmissionPlan, GithubEvidenceSelector, GithubPlanAdmission,
    GithubPlanState,
};

fn plan_error(root: &Path, message: impl Into<String>) -> ObserverError {
    ObserverError::State {
        path: root.to_path_buf(),
        message: message.into(),
    }
}

/// Collect metadata directly from the fixed GitHub HTTPS origin and explain
/// every missing proof. This never accepts evidence, runs verification, writes
/// files, installs trust, or follows an artifact-supplied command or URL.
pub fn plan_github_verification_admission(
    root: &Path,
    work_item_id: &str,
    selector: &GithubEvidenceSelector,
    runtime: &RuntimeContext,
    caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
) -> Result<GithubAdmissionPlan, ObserverError> {
    plan_with_collection(
        root,
        work_item_id,
        selector,
        runtime,
        caller,
        github::collect,
    )
}

// Private injection boundary for deterministic collection/authority tests. The
// public service always uses the fixed-origin HTTPS collector above.
fn plan_with_collection<F>(
    root: &Path,
    work_item_id: &str,
    selector: &GithubEvidenceSelector,
    runtime: &RuntimeContext,
    caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
    collect: F,
) -> Result<GithubAdmissionPlan, ObserverError>
where
    F: FnOnce(&GithubEvidenceSelector) -> github::Collected,
{
    validate_work_item_id(work_item_id)?;
    selector.validate().map_err(|e| plan_error(root, e))?;
    require_current_action_admission_with_caller(
        root,
        work_item_id,
        "run_verification",
        runtime,
        caller,
    )?;
    let before = work_item_status_snapshot_with_runtime(root, work_item_id, runtime)?;
    let contract_digest = before
        .source_digests
        .get("contract")
        .ok_or_else(|| plan_error(root, "current Contract binding is missing"))?
        .clone();
    let snapshot_digest = before
        .source_digests
        .get("repositorySnapshot")
        .ok_or_else(|| plan_error(root, "current snapshot binding is missing"))?
        .clone();
    let observed = collect(selector);
    // Recheck present authority after network I/O. A stale plan cannot become
    // an operation token, and expiry cannot be hidden behind a slow provider.
    task_authorization::require_task_caller(
        root,
        work_item_id,
        "run_verification",
        runtime,
        caller,
    )?;
    let after = work_item_status_snapshot_with_runtime(root, work_item_id, runtime)?;
    let mut gaps = vec![
        "reviewed_trust_policy_unavailable".into(),
        "workflow_execution_closure_unverified".into(),
        "producer_runtime_unverified".into(),
        "artifact_job_attempt_binding_unavailable".into(),
        "native_receipt_not_inspected".into(),
        "current_consumer_command_binding_unavailable".into(),
        "admission_not_implemented".into(),
    ];
    if before.source_digests.get("contract") != after.source_digests.get("contract")
        || before.source_digests.get("repositorySnapshot")
            != after.source_digests.get("repositorySnapshot")
    {
        gaps.push("consumer_bindings_changed_during_collection".into());
    }
    if let Some(failure) = &observed.collection.failure {
        gaps.push(failure.clone());
    }
    gaps.extend(observed.diagnostic_gaps);
    let state = if observed.rejected {
        GithubPlanState::MetadataRejected
    } else if observed.collection.complete {
        GithubPlanState::Gaps
    } else {
        GithubPlanState::ProviderUnavailable
    };
    Ok(GithubAdmissionPlan {
        schema_version: GITHUB_PLAN_SCHEMA_VERSION,
        work_item_id: work_item_id.into(),
        repository_id: before.repository_id,
        contract_digest,
        consumer_snapshot_digest: snapshot_digest,
        consumer_runtime_digest: runtime.runtime_digest.clone(),
        selector: selector.clone(),
        state,
        collection: observed.collection,
        metadata: observed.metadata,
        gaps,
        admission: GithubPlanAdmission::NotAttempted,
        verification_commands_executed: 0,
        writes_performed: 0,
    })
}

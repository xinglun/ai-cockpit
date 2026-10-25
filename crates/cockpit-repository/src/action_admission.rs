use super::{ObserverError, RuntimeContext, WorkItemActionExplanation};
use std::path::Path;

/// Recompute action admission from a fresh status projection at the
/// execution boundary. The caller cannot supply a prior query result, so a
/// Contract or repository snapshot change cannot reuse stale permission.
pub fn require_current_action_admission(
    root: &Path,
    work_item_id: &str,
    requested_action: &str,
    runtime: &RuntimeContext,
) -> Result<WorkItemActionExplanation, ObserverError> {
    require_current_action_admission_with_caller(
        root,
        work_item_id,
        requested_action,
        runtime,
        None,
    )
}

/// Explicit request-scoped identity; callers cannot supply an earlier admission result.
pub fn require_current_action_admission_with_caller(
    root: &Path,
    work_item_id: &str,
    requested_action: &str,
    runtime: &RuntimeContext,
    caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
) -> Result<WorkItemActionExplanation, ObserverError> {
    let explanation = require_base_action_admission(root, work_item_id, requested_action, runtime)?;
    super::task_authorization::require_task_caller(
        root,
        work_item_id,
        requested_action,
        runtime,
        caller,
    )?;
    Ok(explanation)
}

/// Native lifecycle/owner gates remain authoritative and independent of a
/// delegated task grant. Exact owner decision recording uses this base gate.
pub(crate) fn require_base_action_admission(
    root: &Path,
    work_item_id: &str,
    requested_action: &str,
    runtime: &RuntimeContext,
) -> Result<WorkItemActionExplanation, ObserverError> {
    let status = super::work_item_status_snapshot_with_runtime(root, work_item_id, runtime)?;
    let explanation = status
        .action_explanation
        .ok_or_else(|| ObserverError::State {
            path: root
                .join(".ai/work-items/active")
                .join(format!("{work_item_id}.contract.json")),
            message: "current status does not contain action admission explanation".into(),
        })?;
    let requested_is_safe = status
        .safe_actions
        .iter()
        .any(|action| action == requested_action);
    // `admission_state` describes the Work Item as a whole.  A blocked
    // archived item can still admit the explicit action that resolves its
    // blocker, such as `close_after_review`.  The action-level authority is
    // the fresh Runtime `safe_actions` set; do not require the aggregate
    // state to be `Allowed` or the close path would deadlock itself.
    if !requested_is_safe {
        return Err(ObserverError::State {
            path: root
                .join(".ai/work-items/active")
                .join(format!("{work_item_id}.summary.json")),
            message: format!(
                "current action admission rejected requested action {requested_action:?}: admission={:?}, blockers={:?}, safeActions={:?}, nextAction={:?}, admissionDigest={}, issues={:?}",
                explanation.admission_state,
                status.blockers,
                status.safe_actions,
                explanation.recommended_action,
                explanation.admission_digest,
                explanation
                    .issues
                    .iter()
                    .map(|issue| issue.code.as_str())
                    .collect::<Vec<_>>()
            ),
        });
    }
    Ok(explanation)
}

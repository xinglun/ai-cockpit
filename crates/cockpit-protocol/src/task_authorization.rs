use crate::{
    Digest, TASK_AUTHORIZATION_ACTIONS, TASK_AUTHORIZATION_PUBLIC_ACTIONS, TaskAuthorizationTarget,
    TaskAuthorizationTerminationDecision, digest_json,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const TASK_AUTHORIZATION_BATCH_SCHEMA_VERSION: u32 = 1;

/// Caller-supplied limits for one explicitly selected Work Item. A retry key
/// is the exact Contract-declared check command, never a glob or alias.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskAuthorizationBatchBudget {
    pub start_quota: u32,
    #[serde(default)]
    pub retry_budget_by_check: BTreeMap<String, u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskAuthorizationBatchMemberRequest {
    pub work_item_id: String,
    pub actions: Vec<String>,
    pub expires_at: String,
    pub budget: TaskAuthorizationBatchBudget,
}

/// Read-only request for a finite set of same-repository Work Items.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskAuthorizationBatchRequest {
    pub schema_version: u32,
    pub repository_id: String,
    pub members: Vec<TaskAuthorizationBatchMemberRequest>,
}

impl TaskAuthorizationBatchRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != TASK_AUTHORIZATION_BATCH_SCHEMA_VERSION
            || self.repository_id.trim().is_empty()
            || self.members.is_empty()
        {
            return Err(
                "task authorization batch request requires schema1, repository and members".into(),
            );
        }
        let mut seen = BTreeSet::new();
        for member in &self.members {
            if member.work_item_id.trim().is_empty()
                || member.work_item_id.trim() != member.work_item_id
                || member.work_item_id.chars().any(char::is_whitespace)
                || member.work_item_id.contains(['*', '?', '[', ']'])
                || !seen.insert(&member.work_item_id)
                || member.actions.is_empty()
                || member.expires_at.trim().is_empty()
            {
                return Err("task authorization batch members must be explicit and unique".into());
            }
            let actions = member.actions.iter().collect::<BTreeSet<_>>();
            if actions.len() != member.actions.len()
                || member
                    .actions
                    .iter()
                    .any(|action| !TASK_AUTHORIZATION_ACTIONS.contains(&action.as_str()))
            {
                return Err(
                    "task authorization batch actions must be unique supported actions".into(),
                );
            }
            member.budget.validate()?;
        }
        Ok(())
    }
}

/// A manifest member binds the existing per-WI native owner grant instead of
/// inheriting authority from another member.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskAuthorizationBatchMember {
    pub work_item_id: String,
    pub contract_digest: Digest,
    pub snapshot_digest: Digest,
    pub scope_digest: Digest,
    pub parent_grant_id: Digest,
    pub actor: String,
    pub authority_source: String,
    pub expires_at: String,
    pub actions: Vec<String>,
    pub public_target: Option<TaskAuthorizationTarget>,
    pub budget: TaskAuthorizationBatchBudget,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskAuthorizationBatchDigestPayload<'a> {
    schema_version: u32,
    repository_id: &'a str,
    members: &'a [TaskAuthorizationBatchMember],
}

/// Deterministic, read-only batch plan. `manifest_digest` excludes itself.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskAuthorizationBatchManifest {
    pub schema_version: u32,
    pub repository_id: String,
    pub members: Vec<TaskAuthorizationBatchMember>,
    pub manifest_digest: Digest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskAuthorizationBatchDecision {
    ConfirmReview,
    Reject,
}

/// A supplied owner-decision claim over the exact manifest digest. It cannot
/// add or edit a member; a changed manifest requires a new review. This value
/// is not authenticated or a durable native decision receipt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskAuthorizationBatchReview {
    pub schema_version: u32,
    pub repository_id: String,
    pub manifest_digest: Digest,
    pub decision: TaskAuthorizationBatchDecision,
    pub reviewer_actor: String,
    pub authority_source: String,
    pub recorded_at: String,
    pub reason: String,
}

/// A separately derived child-grant candidate for one manifest member. This
/// is a projection only; it grants no authority until a native writer stores
/// and revalidates the exact owner decision.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskAuthorizationChildGrant {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub manifest_digest: Digest,
    pub review_digest: Digest,
    pub member_digest: Digest,
    pub grant_id: Digest,
    pub member: TaskAuthorizationBatchMember,
}

/// Immutable record of the owner's decision over one exact batch manifest.
/// The embedded reviewer identity is a claim; `recorded_by` binds the native
/// Runtime that persisted the record and does not authenticate that claim.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskAuthorizationBatchReceipt {
    pub schema_version: u32,
    pub repository_id: String,
    pub manifest: TaskAuthorizationBatchManifest,
    pub review: TaskAuthorizationBatchReview,
    pub child_grants: Vec<TaskAuthorizationChildGrant>,
    pub runtime_version: String,
    pub runtime_digest: Digest,
    pub recorded_at: String,
    pub recorded_by: String,
    pub receipt_digest: Digest,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskAuthorizationBatchReceiptDigestPayload<'a> {
    schema_version: u32,
    repository_id: &'a str,
    manifest: &'a TaskAuthorizationBatchManifest,
    review: &'a TaskAuthorizationBatchReview,
    child_grants: &'a [TaskAuthorizationChildGrant],
    runtime_version: &'a str,
    runtime_digest: &'a Digest,
    recorded_at: &'a str,
    recorded_by: &'a str,
}

/// Append-only per-member journal event. State changes are represented by
/// later events; an earlier failure or uncertain outcome is never rewritten.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskAuthorizationBatchEvent {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub manifest_digest: Digest,
    pub grant_id: Digest,
    pub sequence: u64,
    pub operation_id: String,
    pub kind: TaskAuthorizationBatchEventKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<TaskAuthorizationBatchAttemptOutcome>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_digest: Option<Digest>,
    pub reason: String,
    pub recorded_at: String,
    pub record_digest: Digest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskAuthorizationBatchEventKind {
    StartReserved,
    CheckAttemptReserved,
    CheckOutcomeRecorded,
    CheckOutcomeInspected,
    Terminated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskAuthorizationBatchAttemptOutcome {
    Passed,
    Failed,
    Unknown,
    NotStarted,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskAuthorizationBatchEventDigestPayload<'a> {
    schema_version: u32,
    repository_id: &'a str,
    work_item_id: &'a str,
    manifest_digest: &'a Digest,
    grant_id: &'a Digest,
    sequence: u64,
    operation_id: &'a str,
    kind: TaskAuthorizationBatchEventKind,
    action: &'a Option<String>,
    check_id: &'a Option<String>,
    attempt: &'a Option<u32>,
    outcome: &'a Option<TaskAuthorizationBatchAttemptOutcome>,
    evidence_ref: &'a Option<String>,
    evidence_digest: &'a Option<Digest>,
    reason: &'a str,
    recorded_at: &'a str,
}

/// Request-scoped caller input for reserving one member start.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskAuthorizationBatchStartRequest {
    pub schema_version: u32,
    pub manifest_digest: Digest,
    pub work_item_id: String,
    pub operation_id: String,
    pub action: String,
    pub caller: crate::TaskAuthorizationCaller,
}

/// Request to reserve an initial declared check or an allowed retry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskAuthorizationBatchCheckRequest {
    pub schema_version: u32,
    pub manifest_digest: Digest,
    pub work_item_id: String,
    pub operation_id: String,
    pub check_id: String,
    pub caller: crate::TaskAuthorizationCaller,
}

/// Record the result of an already reserved check attempt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskAuthorizationBatchOutcomeRequest {
    pub schema_version: u32,
    pub manifest_digest: Digest,
    pub work_item_id: String,
    pub operation_id: String,
    pub check_id: String,
    pub attempt: u32,
    pub outcome: TaskAuthorizationBatchAttemptOutcome,
    pub evidence_ref: String,
    pub evidence_digest: Digest,
    pub reason: String,
    pub caller: crate::TaskAuthorizationCaller,
}

/// Inspect an uncertain result and append a resolution only with concrete
/// evidence. `NotStarted` permits a new attempt; pass/fail remain results.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskAuthorizationBatchInspectionRequest {
    pub schema_version: u32,
    pub manifest_digest: Digest,
    pub work_item_id: String,
    pub operation_id: String,
    pub check_id: String,
    pub attempt: u32,
    pub inspected_outcome: TaskAuthorizationBatchAttemptOutcome,
    pub evidence_ref: String,
    pub evidence_digest: Digest,
    pub reason: String,
    pub caller: crate::TaskAuthorizationCaller,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskAuthorizationBatchTerminationRequest {
    pub schema_version: u32,
    pub manifest_digest: Digest,
    pub work_item_id: String,
    pub decision: TaskAuthorizationTerminationDecision,
    pub caller: crate::TaskAuthorizationCaller,
    pub reason: String,
}

/// Read-only per-member execution projection. It preserves every journal
/// event while making quota, unresolved outcomes, and next action visible.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskAuthorizationBatchMemberLedger {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub manifest_digest: Digest,
    pub grant_id: Digest,
    pub authorization_state: String,
    pub starts_used: u32,
    pub start_quota: u32,
    pub events: Vec<TaskAuthorizationBatchEvent>,
    pub blockers: Vec<String>,
    pub next_action: String,
}

impl TaskAuthorizationBatchReceipt {
    pub fn new(
        manifest: TaskAuthorizationBatchManifest,
        review: TaskAuthorizationBatchReview,
        runtime_version: impl Into<String>,
        runtime_digest: Digest,
        recorded_at: impl Into<String>,
    ) -> Result<Self, String> {
        manifest.validate()?;
        let child_grants = manifest.derive_child_grant_candidates(&review)?;
        let runtime_version = runtime_version.into();
        let recorded_at = recorded_at.into();
        if runtime_version.trim().is_empty() || recorded_at.trim().is_empty() {
            return Err("task authorization batch receipt runtime and time are required".into());
        }
        let mut receipt = Self {
            schema_version: TASK_AUTHORIZATION_BATCH_SCHEMA_VERSION,
            repository_id: manifest.repository_id.clone(),
            manifest,
            review,
            child_grants,
            runtime_version: runtime_version.clone(),
            runtime_digest: runtime_digest.clone(),
            recorded_at: recorded_at.clone(),
            recorded_by: format!("runtime:{runtime_version}:{runtime_digest}"),
            receipt_digest: Digest::sha256_bytes(b""),
        };
        receipt.receipt_digest = receipt.computed_digest()?;
        Ok(receipt)
    }

    pub fn computed_digest(&self) -> Result<Digest, String> {
        digest_json(&TaskAuthorizationBatchReceiptDigestPayload {
            schema_version: self.schema_version,
            repository_id: &self.repository_id,
            manifest: &self.manifest,
            review: &self.review,
            child_grants: &self.child_grants,
            runtime_version: &self.runtime_version,
            runtime_digest: &self.runtime_digest,
            recorded_at: &self.recorded_at,
            recorded_by: &self.recorded_by,
        })
        .map_err(|error| error.to_string())
    }

    pub fn validate_integrity(&self) -> Result<(), String> {
        if self.schema_version != TASK_AUTHORIZATION_BATCH_SCHEMA_VERSION
            || self.repository_id != self.manifest.repository_id
            || self.review.repository_id != self.repository_id
            || self.review.manifest_digest != self.manifest.manifest_digest
            || self.child_grants != self.manifest.derive_child_grant_candidates(&self.review)?
            || self.runtime_version.trim().is_empty()
            || self.recorded_at.trim().is_empty()
            || self.recorded_by
                != format!("runtime:{}:{}", self.runtime_version, self.runtime_digest)
            || self.computed_digest()? != self.receipt_digest
        {
            return Err("task authorization batch receipt identity or digest mismatch".into());
        }
        Ok(())
    }
}

impl TaskAuthorizationBatchEvent {
    pub fn computed_digest(&self) -> Result<Digest, String> {
        digest_json(&TaskAuthorizationBatchEventDigestPayload {
            schema_version: self.schema_version,
            repository_id: &self.repository_id,
            work_item_id: &self.work_item_id,
            manifest_digest: &self.manifest_digest,
            grant_id: &self.grant_id,
            sequence: self.sequence,
            operation_id: &self.operation_id,
            kind: self.kind,
            action: &self.action,
            check_id: &self.check_id,
            attempt: &self.attempt,
            outcome: &self.outcome,
            evidence_ref: &self.evidence_ref,
            evidence_digest: &self.evidence_digest,
            reason: &self.reason,
            recorded_at: &self.recorded_at,
        })
        .map_err(|error| error.to_string())
    }

    pub fn validate_integrity(&self) -> Result<(), String> {
        if self.schema_version != TASK_AUTHORIZATION_BATCH_SCHEMA_VERSION
            || self.work_item_id.trim().is_empty()
            || self.operation_id.trim().is_empty()
            || self.reason.trim().is_empty()
            || self.recorded_at.trim().is_empty()
            || self.sequence == 0
            || self.computed_digest()? != self.record_digest
        {
            return Err("task authorization batch event identity or digest mismatch".into());
        }
        Ok(())
    }
}

impl TaskAuthorizationBatchBudget {
    pub fn validate(&self) -> Result<(), String> {
        if self.start_quota == 0 {
            return Err("task authorization batch start quota must be positive".into());
        }
        for (check, retries) in &self.retry_budget_by_check {
            if check.trim().is_empty() || check.trim() != check || *retries > 2 {
                return Err(
                    "task authorization batch retry budgets require exact checks and at most two additional attempts".into(),
                );
            }
        }
        Ok(())
    }
}

impl TaskAuthorizationBatchManifest {
    pub fn new(
        repository_id: impl Into<String>,
        mut members: Vec<TaskAuthorizationBatchMember>,
    ) -> Result<Self, String> {
        let repository_id = repository_id.into();
        if repository_id.trim().is_empty() || members.is_empty() {
            return Err(
                "task authorization batch requires a repository and explicit members".into(),
            );
        }
        members.sort_by(|left, right| left.work_item_id.cmp(&right.work_item_id));
        let mut seen = BTreeSet::new();
        for member in &members {
            validate_member(member)?;
            if !seen.insert(&member.work_item_id) {
                return Err(format!(
                    "duplicate task authorization batch member: {}",
                    member.work_item_id
                ));
            }
        }
        let mut manifest = Self {
            schema_version: TASK_AUTHORIZATION_BATCH_SCHEMA_VERSION,
            repository_id,
            members,
            manifest_digest: Digest::sha256_bytes(b""),
        };
        manifest.manifest_digest = manifest.computed_digest()?;
        Ok(manifest)
    }

    pub fn computed_digest(&self) -> Result<Digest, String> {
        digest_json(&TaskAuthorizationBatchDigestPayload {
            schema_version: self.schema_version,
            repository_id: &self.repository_id,
            members: &self.members,
        })
        .map_err(|error| error.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != TASK_AUTHORIZATION_BATCH_SCHEMA_VERSION
            || self.repository_id.trim().is_empty()
            || self.members.is_empty()
        {
            return Err("invalid task authorization batch manifest identity".into());
        }
        let mut previous: Option<&str> = None;
        for member in &self.members {
            validate_member(member)?;
            if previous.is_some_and(|value| value >= member.work_item_id.as_str()) {
                return Err("task authorization batch members must be uniquely sorted".into());
            }
            previous = Some(&member.work_item_id);
        }
        if self.computed_digest()? != self.manifest_digest {
            return Err("task authorization batch manifest digest mismatch".into());
        }
        Ok(())
    }

    pub fn derive_child_grant_candidates(
        &self,
        review: &TaskAuthorizationBatchReview,
    ) -> Result<Vec<TaskAuthorizationChildGrant>, String> {
        self.validate()?;
        if review.schema_version != TASK_AUTHORIZATION_BATCH_SCHEMA_VERSION
            || review.repository_id != self.repository_id
            || review.manifest_digest != self.manifest_digest
            || review.decision != TaskAuthorizationBatchDecision::ConfirmReview
            || !review.reviewer_actor.starts_with("human:")
            || review.reviewer_actor.trim() != review.reviewer_actor
            || review.reviewer_actor.len() <= 6
            || review.authority_source.trim().is_empty()
            || review.recorded_at.trim().is_empty()
            || review.reason.trim().is_empty()
        {
            return Err(
                "task authorization batch review claim does not confirm this exact manifest".into(),
            );
        }
        let review_digest = digest_json(review).map_err(|error| error.to_string())?;
        self.members
            .iter()
            .map(|member| {
                let member_digest = digest_json(member).map_err(|error| error.to_string())?;
                let grant_id = digest_json(&(
                    self.repository_id.as_str(),
                    self.manifest_digest.clone(),
                    member.work_item_id.as_str(),
                    member_digest.clone(),
                    review_digest.clone(),
                ))
                .map_err(|error| error.to_string())?;
                Ok(TaskAuthorizationChildGrant {
                    schema_version: TASK_AUTHORIZATION_BATCH_SCHEMA_VERSION,
                    repository_id: self.repository_id.clone(),
                    work_item_id: member.work_item_id.clone(),
                    manifest_digest: self.manifest_digest.clone(),
                    review_digest: review_digest.clone(),
                    member_digest,
                    grant_id,
                    member: member.clone(),
                })
            })
            .collect()
    }
}

fn validate_member(member: &TaskAuthorizationBatchMember) -> Result<(), String> {
    if member.work_item_id.trim().is_empty()
        || member.work_item_id.trim() != member.work_item_id
        || member.work_item_id.chars().any(char::is_whitespace)
        || member.work_item_id.contains(['*', '?', '[', ']'])
        || !member.actor.starts_with("agent:")
        || member.actor.trim() != member.actor
        || member.authority_source.trim().is_empty()
        || member.expires_at.trim().is_empty()
        || member.actions.is_empty()
    {
        return Err("invalid task authorization batch member identity".into());
    }
    let actions = member.actions.iter().collect::<BTreeSet<_>>();
    if actions.len() != member.actions.len()
        || member
            .actions
            .iter()
            .any(|action| !TASK_AUTHORIZATION_ACTIONS.contains(&action.as_str()))
    {
        return Err("task authorization batch actions must be unique supported actions".into());
    }
    if member.public_target.is_none()
        && member
            .actions
            .iter()
            .any(|action| TASK_AUTHORIZATION_PUBLIC_ACTIONS.contains(&action.as_str()))
    {
        return Err("task authorization batch public actions require an explicit target".into());
    }
    member.budget.validate()
}

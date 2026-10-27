use crate::{Digest, TASK_AUTHORIZATION_ACTIONS, TASK_AUTHORIZATION_PUBLIC_ACTIONS, TaskAuthorizationTarget, digest_json};
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
            return Err("task authorization batch request requires schema1, repository and members".into());
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
                return Err("task authorization batch actions must be unique supported actions".into());
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
            return Err("task authorization batch requires a repository and explicit members".into());
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
            return Err("task authorization batch review claim does not confirm this exact manifest".into());
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

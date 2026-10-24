//! Local native owner provenance and current task authority. Evidence freshness is separate.
use super::{ObserverError, legacy_closeout_source::PinnedRoot};
use chrono::{DateTime, Utc};
use cockpit_protocol::{
    AssuranceLevel, Contract, PreflightDecisionEvidence, TaskAuthorizationPolicy,
    TaskAuthorizationProjection,
};
use serde_json::{Value, json};
use std::path::{Component, Path, PathBuf};

fn error(path: &Path, message: impl Into<String>) -> ObserverError {
    ObserverError::State {
        path: path.into(),
        message: message.into(),
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
) -> Result<TaskAuthorizationProjection, ObserverError> {
    let contract_path = PathBuf::from(format!(".ai/work-items/active/{wi}.contract.json"));
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
    let summary = native_json(
        root,
        Path::new(&format!(".ai/work-items/active/{wi}.summary.json")),
    )?;
    let Some(value) = summary.get("decisionEvidence") else {
        return Ok(projection(
            policy,
            "owner_review_required",
            "The current task policy has no native owner decision.",
        ));
    };
    let owner: PreflightDecisionEvidence = serde_json::from_value(value.clone())
        .map_err(|e| error(root, format!("invalid task owner review: {e}")))?;
    if owner.schema_version != 1
        || owner.decision_id != "contract-preflight-review"
        || owner.decision != "confirm_review"
        || owner.work_item_id != wi
        || owner.repository_id != contract.repository_id
        || owner.contract_digest != current_digest
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
        cockpit_protocol::digest_json(value).map_err(|e| error(root, e.to_string()))?;
    let canonical = PathBuf::from(format!(".ai/decisions/{wi}.preflight-review.json"));
    let stored = native_json(root, &canonical)?;
    if stored != *value {
        let indexed = PathBuf::from(format!(
            ".ai/decisions/{wi}.preflight-review.{}.json",
            owner_digest.to_string().trim_start_matches("sha256:")
        ));
        if native_json(root, &indexed)? != *value {
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
        "repositoryId":contract.repository_id, "workItemId":wi, "goal":contract.goal,
        "scope":contract.scope, "outOfScope":contract.out_of_scope,
        "verification":contract.verification, "acceptanceCriteria":contract.acceptance_criteria,
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
            "ownerReceiptDigest":owner_digest, "policy":policy, "scopeDigest":scope_digest,
        }))
        .map_err(|e| error(root, e.to_string()))?,
    );
    result.scope_digest = Some(scope_digest);
    if let Some(deadline) = &policy.expires_at {
        let deadline = DateTime::parse_from_rfc3339(deadline)
            .map_err(|e| error(root, format!("invalid task authorization deadline: {e}")))?;
        if Utc::now() >= deadline {
            result.state = "expired".into();
            result.reason = "The declared task authorization deadline has been reached.".into();
        }
    }
    Ok(result)
}

pub(crate) fn task_authorization_projection(
    root: &Path,
    wi: &str,
    contract: &Contract,
) -> Result<Option<TaskAuthorizationProjection>, ObserverError> {
    let Some(policy) = contract
        .task_authorization_policy()
        .map_err(|e| error(root, e))?
    else {
        return Ok(None);
    };
    Ok(Some(assess(root, wi, contract, &policy).unwrap_or_else(
        |e| projection(&policy, "invalid", &e.to_string()),
    )))
}

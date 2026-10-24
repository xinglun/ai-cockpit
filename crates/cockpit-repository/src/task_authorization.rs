//! Local native owner provenance and current task authority. Evidence freshness is separate.
use super::{ObserverError, legacy_closeout_source::PinnedRoot};
use chrono::{DateTime, Utc};
use cockpit_protocol::{
    AssuranceLevel, Contract, PreflightDecisionEvidence, RuntimeContext, TaskAuthorizationPolicy,
    TaskAuthorizationProjection, TaskAuthorizationTerminationRequest,
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
    let deadline = policy
        .expires_at
        .as_deref()
        .map(DateTime::parse_from_rfc3339)
        .transpose()
        .map_err(|e| error(root, format!("invalid task authorization deadline: {e}")))?;
    let terminal = read_termination(root, wi, &result, policy)?;
    let deadline_first = deadline.is_some_and(|deadline| {
        Utc::now() >= deadline
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
    let contract_value = native_json(
        root,
        Path::new(&format!(".ai/work-items/active/{wi}.contract.json")),
    )?;
    let contract: Contract =
        serde_json::from_value(contract_value).map_err(|e| error(root, e.to_string()))?;
    let policy = contract
        .task_authorization_policy()
        .map_err(|e| error(root, e))?
        .ok_or_else(|| error(root, "task termination has no explicit owner policy"))?;
    let task = assess(root, wi, &contract, &policy)?;
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

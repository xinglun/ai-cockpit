use crate::{CoordinationError, CoordinationStore};
use cockpit_git::GitRepository;
use cockpit_protocol::{
    CoordinationEvent, CoordinationIntent, CoordinationRecovery, CoordinationRequest,
    CoordinationRequestState, RuntimeCapabilityBinding, RuntimeContext, WorktreeRegistration,
};
use cockpit_verification::{
    CompositionAttempt, CompositionError, CompositionInput, CompositionPrecondition,
    composition_commands_digest, run_composition,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationProjection {
    pub registrations: Vec<WorktreeRegistration>,
    pub events: Vec<CoordinationEvent>,
    pub recoveries: Vec<CoordinationRecovery>,
    pub requests: Vec<CoordinationRequest>,
    pub affected_work_items: Vec<String>,
    pub blockers: BTreeMap<String, Vec<String>>,
    pub cycles: Vec<Vec<String>>,
    pub unknowns: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationAdmission {
    pub work_item_id: String,
    pub generation: u64,
    pub allowed: bool,
    pub affected: bool,
    pub blockers: Vec<String>,
    pub unknowns: Vec<String>,
    pub refreshed_events: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CollaborationActionKind {
    Composition,
    Verification,
    ResourceReservation,
    ResourceRelease,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationAction {
    pub kind: CollaborationActionKind,
    pub consumer_work_item_id: String,
    #[serde(default)]
    pub outcome_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationOutcomeProjection {
    pub schema_version: u32,
    pub work_item_id: String,
    pub state: String,
    pub providers: Vec<String>,
    pub consumers: Vec<String>,
    pub waiting_edges: Vec<String>,
    pub invalidated_event_ids: Vec<String>,
    pub unhandled_requests: Vec<CoordinationRequest>,
    pub integration_owner: Option<String>,
    pub composition_order: Vec<String>,
    pub implementation_state: String,
    pub composition_state: String,
    pub target_merge_state: String,
    pub cleanup_state: String,
    pub revalidation: String,
    pub reusable_checks: Vec<String>,
    pub blockers: Vec<String>,
    pub unknowns: Vec<String>,
    pub human_decision_required: bool,
    pub next_action: String,
}

#[derive(Debug, Error)]
pub enum CollaborationExecutionError {
    #[error(transparent)]
    Coordination(#[from] CoordinationError),
    #[error("collaboration action is blocked for {work_item_id}: {blockers:?}")]
    Blocked {
        work_item_id: String,
        blockers: Vec<String>,
        unknowns: Vec<String>,
    },
    #[error(transparent)]
    Composition(#[from] CompositionError),
}

pub fn collaboration_projection(
    store: &CoordinationStore,
) -> Result<CollaborationProjection, CoordinationError> {
    let inspection = store.inspect()?;
    let registrations = inspection.registrations;
    let events = inspection.events;
    let recoveries = inspection.recoveries;
    let requests = inspection.requests;
    let mut projection = CollaborationProjection {
        registrations,
        events,
        recoveries,
        requests,
        unknowns: inspection.unknowns,
        ..CollaborationProjection::default()
    };

    let registration_by_id = projection
        .registrations
        .iter()
        .map(|registration| (registration.work_item_id.as_str(), registration))
        .collect::<BTreeMap<_, _>>();
    let mut affected = BTreeSet::new();
    for event in &projection.events {
        for registration in &projection.registrations {
            if registration.work_item_id == event.work_item_id {
                continue;
            }
            if !event_invalidates(event) || recovery_consumed(&projection, event, registration) {
                continue;
            }
            if registration
                .declaration
                .consumed_outcomes
                .iter()
                .any(|dependency| dependency.provider_work_item_id == event.work_item_id)
            {
                affected.insert(registration.work_item_id.clone());
            }
        }
    }
    projection.affected_work_items = affected.into_iter().collect();

    let mut provider_stages = BTreeMap::new();
    for registration in &projection.registrations {
        for outcome in &registration.declaration.provided_outcomes {
            provider_stages.insert(
                (
                    registration.work_item_id.as_str(),
                    outcome.outcome_id.as_str(),
                ),
                outcome.stage,
            );
        }
    }
    for registration in &projection.registrations {
        let mut blockers = Vec::new();
        for dependency in &registration.declaration.consumed_outcomes {
            let Some(provider) = registration_by_id.get(dependency.provider_work_item_id.as_str())
            else {
                blockers.push(format!(
                    "dependency_missing:{}",
                    dependency.provider_work_item_id
                ));
                continue;
            };
            let Some(stage) = provider_stages.get(&(
                provider.work_item_id.as_str(),
                dependency.outcome_id.as_str(),
            )) else {
                blockers.push(format!(
                    "outcome_missing:{}:{}",
                    dependency.provider_work_item_id, dependency.outcome_id
                ));
                continue;
            };
            if !stage.satisfies(dependency.minimum_stage) {
                blockers.push(format!(
                    "dependency_stage:{}:{stage:?}",
                    dependency.provider_work_item_id
                ));
            }
            if projection.events.iter().any(|event| {
                event.work_item_id == dependency.provider_work_item_id
                    && event_invalidates(event)
                    && !recovery_consumed(&projection, event, registration)
            }) {
                blockers.push(format!(
                    "dependency_impact:{}",
                    dependency.provider_work_item_id
                ));
            }
            if dependency.verification_required {
                let provider_outcome = provider
                    .declaration
                    .provided_outcomes
                    .iter()
                    .find(|outcome| outcome.outcome_id == dependency.outcome_id);
                if provider_outcome.is_none_or(|outcome| {
                    outcome.evidence_refs.is_empty()
                        || outcome.evidence_refs.iter().any(|reference| {
                            let path = Path::new(&provider.worktree_path).join(reference);
                            fs::symlink_metadata(path)
                                .map(|metadata| {
                                    metadata.file_type().is_symlink() || !metadata.is_file()
                                })
                                .unwrap_or(true)
                        })
                }) {
                    blockers.push(format!(
                        "dependency_evidence_missing:{}:{}",
                        dependency.provider_work_item_id, dependency.outcome_id
                    ));
                }
            }
        }
        blockers.sort();
        blockers.dedup();
        if !blockers.is_empty() {
            projection
                .blockers
                .insert(registration.work_item_id.clone(), blockers);
        }
    }
    projection.cycles = find_cycles(&projection.registrations);
    for cycle in &projection.cycles {
        for work_item_id in cycle {
            projection
                .blockers
                .entry(work_item_id.clone())
                .or_default()
                .push(format!("dependency_cycle:{}", cycle.join("->")));
        }
    }
    for blockers in projection.blockers.values_mut() {
        blockers.sort();
        blockers.dedup();
    }
    Ok(projection)
}

pub fn collaboration_outcome_projection(
    root: &std::path::Path,
    work_item_id: &str,
    runtime: &RuntimeContext,
) -> CollaborationOutcomeProjection {
    let fallback = |state: &str, reason: String| CollaborationOutcomeProjection {
        schema_version: 1,
        work_item_id: work_item_id.into(),
        state: state.into(),
        providers: Vec::new(),
        consumers: Vec::new(),
        waiting_edges: Vec::new(),
        invalidated_event_ids: Vec::new(),
        unhandled_requests: Vec::new(),
        integration_owner: None,
        composition_order: Vec::new(),
        implementation_state: "separate_lifecycle_outcome".into(),
        composition_state: "not_observed".into(),
        target_merge_state: "not_observed".into(),
        cleanup_state: "not_observed".into(),
        revalidation: "unknown".into(),
        reusable_checks: Vec::new(),
        blockers: Vec::new(),
        unknowns: vec![reason],
        human_decision_required: false,
        next_action: "resolve collaboration capability or recovery unknowns".into(),
    };
    let binding = RuntimeCapabilityBinding {
        schema_version: cockpit_protocol::COLLABORATION_SCHEMA_VERSION,
        runtime_version: runtime.runtime_version.clone(),
        runtime_digest: runtime.runtime_digest.clone(),
        capability: cockpit_protocol::COLLABORATION_CAPABILITY.into(),
    };
    let git = match GitRepository::discover(root) {
        Ok(git) => git,
        Err(error) => return fallback("unknown", format!("repository_topology:{error}")),
    };
    let store = match CoordinationStore::open_read_only(&git, binding) {
        Ok(store) => store,
        Err(error) => return fallback("unsupported", format!("collaboration_store:{error}")),
    };
    let projection = match collaboration_projection(&store) {
        Ok(projection) => projection,
        Err(error) => return fallback("unknown", format!("collaboration_projection:{error}")),
    };
    let current = projection
        .registrations
        .iter()
        .find(|registration| registration.work_item_id == work_item_id);
    let providers = current
        .map(|registration| {
            registration
                .declaration
                .consumed_outcomes
                .iter()
                .map(|dependency| dependency.provider_work_item_id.clone())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default()
        .into_iter()
        .collect::<Vec<_>>();
    let consumers = projection
        .registrations
        .iter()
        .filter(|registration| {
            registration
                .declaration
                .consumed_outcomes
                .iter()
                .any(|dependency| dependency.provider_work_item_id == work_item_id)
        })
        .map(|registration| registration.work_item_id.clone())
        .collect::<Vec<_>>();
    let waiting_edges = current
        .map(|registration| {
            registration
                .declaration
                .consumed_outcomes
                .iter()
                .map(|dependency| format!("{}->{}", work_item_id, dependency.provider_work_item_id))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let invalidated_event_ids = projection
        .events
        .iter()
        .filter(|event| {
            providers
                .iter()
                .any(|provider| provider == &event.work_item_id)
                && event_invalidates(event)
                && current.is_some_and(|consumer| !recovery_consumed(&projection, event, consumer))
        })
        .map(|event| event.event_id.clone())
        .collect::<Vec<_>>();
    let unhandled_requests = projection
        .requests
        .iter()
        .filter(|request| {
            request.target_work_item_id == work_item_id
                && current.is_some_and(|registration| {
                    registration.generation == request.target_generation
                })
                && matches!(
                    request.state,
                    CoordinationRequestState::Requested | CoordinationRequestState::Acknowledged
                )
        })
        .cloned()
        .collect::<Vec<_>>();
    let blockers = projection
        .blockers
        .get(work_item_id)
        .cloned()
        .unwrap_or_default();
    let mut unknowns = projection.unknowns.clone();
    let (composition_state, target_merge_state, cleanup_state, reusable_checks) =
        match latest_composition_attempt(store.root(), work_item_id) {
            Ok(Some(attempt)) => composition_facts(&attempt),
            Ok(None) => (
                "not_observed".into(),
                "not_observed".into(),
                "not_observed".into(),
                Vec::new(),
            ),
            Err(error) => {
                unknowns.push(format!("composition_projection:{error}"));
                (
                    "unknown".into(),
                    "unknown".into(),
                    "unknown".into(),
                    Vec::new(),
                )
            }
        };
    let state = if !unknowns.is_empty() {
        "unknown"
    } else if !blockers.is_empty() {
        "blocked"
    } else {
        "allowed"
    };
    let next_action = if !unknowns.is_empty() {
        "resolve collaboration recovery unknowns before dependent action"
    } else if !blockers.is_empty() {
        "refresh affected dependency and composition evidence"
    } else if !unhandled_requests.is_empty() {
        "acknowledge or safely pause at the next boundary"
    } else {
        "refresh dependency state before the next dependent action"
    };
    CollaborationOutcomeProjection {
        schema_version: 1,
        work_item_id: work_item_id.into(),
        state: state.into(),
        providers,
        consumers,
        waiting_edges,
        invalidated_event_ids: invalidated_event_ids.clone(),
        unhandled_requests: unhandled_requests.clone(),
        integration_owner: current.map(|registration| {
            registration
                .declaration
                .integration_responsibility
                .responsible_work_item_id
                .clone()
        }),
        composition_order: current
            .map(|registration| {
                registration
                    .declaration
                    .integration_responsibility
                    .composition_order
                    .clone()
            })
            .unwrap_or_default(),
        implementation_state: "separate_lifecycle_outcome".into(),
        composition_state,
        target_merge_state,
        cleanup_state,
        revalidation: if invalidated_event_ids.is_empty() {
            "not_required"
        } else {
            "required"
        }
        .into(),
        reusable_checks,
        blockers,
        unknowns,
        // A coordination request is an agent-to-agent protocol state. It is
        // surfaced in `unhandled_requests` and the next action, but it is not
        // a human decision until an explicit human decision record exists.
        human_decision_required: false,
        next_action: next_action.into(),
    }
}

fn latest_composition_attempt(
    coordination_root: &Path,
    work_item_id: &str,
) -> Result<Option<CompositionAttempt>, String> {
    let directory = coordination_root.join("compositions");
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let mut latest = None;
    for entry in entries {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if !metadata.file_type().is_file() {
            return Err(format!(
                "composition record is not a regular file: {}",
                path.display()
            ));
        }
        let attempt: CompositionAttempt =
            serde_json::from_slice(&fs::read(&path).map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())?;
        if !attempt
            .binding
            .participant_work_items
            .iter()
            .any(|participant| participant == work_item_id)
        {
            continue;
        }
        if latest.as_ref().is_none_or(|current: &CompositionAttempt| {
            (attempt.recorded_at_unix_nanos, &attempt.attempt_id)
                > (current.recorded_at_unix_nanos, &current.attempt_id)
        }) {
            latest = Some(attempt);
        }
    }
    Ok(latest)
}

fn composition_facts(attempt: &CompositionAttempt) -> (String, String, String, Vec<String>) {
    let composition_state = if attempt.passed {
        "passed"
    } else if attempt.failure.as_deref() == Some("in_progress") {
        "in_progress"
    } else {
        "failed"
    };
    let target_merge_state = if attempt.isolated_worktree.is_empty() {
        "not_observed"
    } else if attempt.text_conflicts.is_empty() {
        "passed"
    } else {
        "failed"
    };
    let cleanup_state = match &attempt.cleanup {
        Some(cleanup) if cleanup.removed => "cleaned",
        Some(cleanup) if cleanup.attempted => "failed",
        Some(_) => "not_observed",
        None => "unknown",
    };
    let reusable_checks = attempt
        .execution_records
        .iter()
        .filter(|record| record.reused)
        .map(|record| record.node_id.clone())
        .collect();
    (
        composition_state.into(),
        target_merge_state.into(),
        cleanup_state.into(),
        reusable_checks,
    )
}

fn recovery_consumed(
    projection: &CollaborationProjection,
    event: &CoordinationEvent,
    consumer: &WorktreeRegistration,
) -> bool {
    projection.recoveries.iter().any(|recovery| {
        recovery.event_id == event.event_id
            && recovery.provider_work_item_id == event.work_item_id
            && recovery.provider_generation == event.generation
            && recovery.consumer_work_item_id == consumer.work_item_id
            && recovery.consumer_generation == consumer.generation
    })
}

pub fn refresh_dependency_state(
    store: &CoordinationStore,
    _work_item_id: &str,
    _generation: u64,
) -> Result<CollaborationProjection, CoordinationError> {
    collaboration_projection(store)
}

pub fn admit_collaboration_action(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    action: CollaborationAction,
) -> Result<CollaborationAdmission, CoordinationError> {
    let projection = refresh_dependency_state(store, work_item_id, generation)?;
    let registration = projection
        .registrations
        .iter()
        .find(|registration| registration.work_item_id == work_item_id);
    let mut blockers = projection
        .blockers
        .get(work_item_id)
        .cloned()
        .unwrap_or_default();
    let mut unknowns = projection.unknowns.clone();
    if registration.is_none() {
        unknowns.push(format!("registration_missing:{work_item_id}"));
    } else if registration.is_some_and(|registration| registration.generation != generation) {
        blockers.push(format!("generation_mismatch:{work_item_id}"));
    } else if action.consumer_work_item_id != work_item_id {
        blockers.push(format!(
            "action_consumer_mismatch:{}",
            action.consumer_work_item_id
        ));
    } else if let Some(registration) = registration {
        if !action.outcome_ids.is_empty() {
            for outcome_id in &action.outcome_ids {
                if !registration
                    .declaration
                    .consumed_outcomes
                    .iter()
                    .any(|dependency| dependency.outcome_id == *outcome_id)
                {
                    blockers.push(format!("action_outcome_not_declared:{outcome_id}"));
                }
            }
        }
        for request in &projection.requests {
            if request.target_work_item_id == work_item_id
                && request.target_generation == generation
                && request.state == CoordinationRequestState::SafelyPaused
            {
                blockers.push(format!("coordination_safely_paused:{}", request.request_id));
            }
        }
    }
    let affected = projection
        .affected_work_items
        .iter()
        .any(|candidate| candidate == work_item_id);
    if affected
        && !blockers
            .iter()
            .any(|blocker| blocker.starts_with("dependency_impact:"))
    {
        blockers.push("dependency_impact:current_events".into());
    }
    blockers.sort();
    blockers.dedup();
    unknowns.sort();
    unknowns.dedup();
    Ok(CollaborationAdmission {
        work_item_id: work_item_id.into(),
        generation,
        allowed: blockers.is_empty() && unknowns.is_empty(),
        affected,
        blockers,
        unknowns,
        refreshed_events: projection.events.len(),
    })
}

pub fn run_admitted_composition(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    input: CompositionInput,
) -> Result<CompositionAttempt, CollaborationExecutionError> {
    let outcome_ids = store
        .inspect()?
        .registrations
        .into_iter()
        .find(|registration| registration.work_item_id == work_item_id)
        .map(|registration| {
            registration
                .declaration
                .consumed_outcomes
                .into_iter()
                .map(|dependency| dependency.outcome_id)
                .collect()
        })
        .unwrap_or_default();
    let admission = admit_collaboration_action(
        store,
        work_item_id,
        generation,
        CollaborationAction {
            kind: CollaborationActionKind::Composition,
            consumer_work_item_id: work_item_id.into(),
            outcome_ids,
        },
    )?;
    if !admission.allowed {
        return Err(CollaborationExecutionError::Blocked {
            work_item_id: work_item_id.into(),
            blockers: admission.blockers,
            unknowns: admission.unknowns,
        });
    }
    let mut input = input;
    let identity_blockers =
        verify_composition_identity(store, work_item_id, generation, &mut input)?;
    if !identity_blockers.is_empty() {
        return Err(CollaborationExecutionError::Blocked {
            work_item_id: work_item_id.into(),
            blockers: identity_blockers,
            unknowns: Vec::new(),
        });
    }
    // The caller may be in a linked worktree. Composition facts are shared
    // coordination evidence, so they must be written below the Git common
    // directory rather than the caller's private checkout.
    input.state_dir = store.root().join("compositions");
    Ok(run_composition(input)?)
}

fn verify_composition_identity(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
    input: &mut CompositionInput,
) -> Result<Vec<String>, CoordinationError> {
    let inspection = store.inspect()?;
    let mut blockers = inspection
        .unknowns
        .into_iter()
        .map(|unknown| format!("coordination_unknown:{unknown}"))
        .collect::<Vec<_>>();
    let registrations = inspection
        .registrations
        .into_iter()
        .map(|registration| (registration.work_item_id.clone(), registration))
        .collect::<BTreeMap<_, _>>();
    let Some(target) = registrations.get(work_item_id) else {
        blockers.push(format!("registration_missing:{work_item_id}"));
        return Ok(blockers);
    };
    if target.generation != generation {
        blockers.push(format!("generation_mismatch:{work_item_id}"));
    }
    let topology = GitRepository::discover(&input.repository_root)
        .and_then(|git| git.topology())
        .map_err(|error| {
            CoordinationError::RecoveryRequired(format!("composition topology: {error}"))
        })?;
    if input.binding.repository_id != target.repository_id {
        blockers.push("composition_repository_identity_mismatch".into());
    }
    if input.binding.target_branch != topology.branch.clone().unwrap_or_default() {
        blockers.push("composition_target_branch_mismatch".into());
    }
    if input.binding.target_sha != topology.head.clone().unwrap_or_default() {
        blockers.push("composition_target_head_mismatch".into());
    }
    if input.binding.participant_work_items.is_empty()
        || input.binding.participant_work_items.len() != input.binding.participant_heads.len()
        || input.binding.participant_work_items.len() != input.binding.contract_digests.len()
    {
        blockers.push("composition_participant_coverage_incomplete".into());
    }
    let mut seen_participants = BTreeSet::new();
    for (index, participant_id) in input.binding.participant_work_items.iter().enumerate() {
        if !seen_participants.insert(participant_id) {
            blockers.push(format!(
                "composition_participant_duplicate:{participant_id}"
            ));
            continue;
        }
        let Some(registration) = registrations.get(participant_id) else {
            blockers.push(format!("composition_participant_missing:{participant_id}"));
            continue;
        };
        if input.binding.participant_heads.get(index) != Some(&registration.head) {
            blockers.push(format!(
                "composition_participant_head_mismatch:{participant_id}"
            ));
        }
        if input.binding.contract_digests.get(index) != Some(&registration.contract_digest) {
            blockers.push(format!(
                "composition_participant_contract_mismatch:{participant_id}"
            ));
        }
    }
    let mut node_ids = BTreeSet::new();
    if input.commands.is_empty() {
        blockers.push("required_checks_empty".into());
    }
    for command in &input.commands {
        if command.node_id.trim().is_empty() || !node_ids.insert(command.node_id.clone()) {
            blockers.push(format!(
                "required_check_identity_invalid:{}",
                command.node_id
            ));
        }
        if command.program.trim().is_empty() {
            blockers.push(format!(
                "required_check_program_missing:{}",
                command.node_id
            ));
        }
    }
    if input.identity.command_digest != composition_commands_digest(&input.commands) {
        blockers.push("composition_command_identity_mismatch".into());
    }
    blockers.sort();
    blockers.dedup();
    if blockers.is_empty() {
        // The caller's boolean preconditions are not an authorization source.
        // Reconstruct them only after observing registrations, Git topology,
        // participant Contracts, and complete required-check coverage.
        input.preconditions = vec![
            CompositionPrecondition::satisfied("registration-identity-observed"),
            CompositionPrecondition::satisfied("composition-target-observed"),
            CompositionPrecondition::satisfied("required-checks-covered"),
        ];
    }
    Ok(blockers)
}

pub fn report_impact(
    store: &CoordinationStore,
    event: CoordinationEvent,
) -> Result<CoordinationEvent, CoordinationError> {
    store.publish_event(event)
}

pub fn recover_impact(
    store: &CoordinationStore,
    event_id: &str,
    consumer_work_item_id: &str,
    consumer_generation: u64,
) -> Result<CoordinationRecovery, CoordinationError> {
    store.recover_event(event_id, consumer_work_item_id, consumer_generation)
}

pub fn request_safe_pause(
    store: &CoordinationStore,
    request: CoordinationRequest,
) -> Result<CoordinationRequest, CoordinationError> {
    if request.intent != CoordinationIntent::RequestSafePause {
        return Err(CoordinationError::RecoveryRequired(
            "request_safe_pause requires request_safe_pause intent".into(),
        ));
    }
    store.request_coordination(request)
}

pub fn acknowledge_pause(
    store: &CoordinationStore,
    request_id: &str,
    state: CoordinationRequestState,
) -> Result<CoordinationRequest, CoordinationError> {
    if !matches!(
        state,
        CoordinationRequestState::Acknowledged
            | CoordinationRequestState::SafelyPaused
            | CoordinationRequestState::Unavailable
            | CoordinationRequestState::Expired
    ) {
        return Err(CoordinationError::RecoveryRequired(
            "pause acknowledgement must be an explicit safe-boundary state".into(),
        ));
    }
    store.transition_request(request_id, state)
}

pub fn resume_and_re_evaluate(
    store: &CoordinationStore,
    work_item_id: &str,
    generation: u64,
) -> Result<CollaborationAdmission, CoordinationError> {
    let projection = refresh_dependency_state(store, work_item_id, generation)?;
    let request_ids = projection
        .requests
        .iter()
        .filter(|request| {
            request.target_work_item_id == work_item_id
                && request.target_generation == generation
                && request.state == CoordinationRequestState::SafelyPaused
        })
        .map(|request| request.request_id.clone())
        .collect::<Vec<_>>();
    for request_id in request_ids {
        store.transition_request(&request_id, CoordinationRequestState::Resumed)?;
    }
    admit_collaboration_action(
        store,
        work_item_id,
        generation,
        CollaborationAction {
            kind: CollaborationActionKind::Verification,
            consumer_work_item_id: work_item_id.into(),
            outcome_ids: Vec::new(),
        },
    )
}

fn event_invalidates(event: &CoordinationEvent) -> bool {
    !matches!(
        event.kind,
        cockpit_protocol::CoordinationEventKind::OutcomePublished
    )
}

fn find_cycles(registrations: &[WorktreeRegistration]) -> Vec<Vec<String>> {
    let graph = registrations
        .iter()
        .map(|registration| {
            (
                registration.work_item_id.clone(),
                registration
                    .declaration
                    .consumed_outcomes
                    .iter()
                    .map(|dependency| dependency.provider_work_item_id.clone())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut cycles = Vec::new();
    for node in graph.keys() {
        collect_cycles(node, &graph, &mut Vec::new(), &mut cycles);
    }
    cycles
}

fn collect_cycles(
    node: &str,
    graph: &BTreeMap<String, Vec<String>>,
    stack: &mut Vec<String>,
    cycles: &mut Vec<Vec<String>>,
) {
    if let Some(position) = stack.iter().position(|item| item == node) {
        let mut cycle = stack[position..].to_vec();
        cycle.push(node.into());
        let key = cycle.join("->");
        if !cycles.iter().any(|candidate| candidate.join("->") == key) {
            cycles.push(cycle);
        }
        return;
    }
    stack.push(node.into());
    if let Some(dependencies) = graph.get(node) {
        for dependency in dependencies {
            if graph.contains_key(dependency) {
                collect_cycles(dependency, graph, stack, cycles);
            }
        }
    }
    stack.pop();
}

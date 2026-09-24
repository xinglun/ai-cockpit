use crate::{CoordinationError, CoordinationStore};
use cockpit_protocol::{
    CoordinationEvent, CoordinationIntent, CoordinationRequest, CoordinationRequestState,
    WorktreeRegistration,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationProjection {
    pub registrations: Vec<WorktreeRegistration>,
    pub events: Vec<CoordinationEvent>,
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

pub fn collaboration_projection(
    store: &CoordinationStore,
) -> Result<CollaborationProjection, CoordinationError> {
    let inspection = store.inspect()?;
    let registrations = inspection.registrations;
    let events = inspection.events;
    let requests = inspection.requests;
    let mut projection = CollaborationProjection {
        registrations,
        events,
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
            if projection
                .events
                .iter()
                .any(|event| event.work_item_id == dependency.provider_work_item_id)
            {
                blockers.push(format!(
                    "dependency_impact:{}",
                    dependency.provider_work_item_id
                ));
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

pub fn report_impact(
    store: &CoordinationStore,
    event: CoordinationEvent,
) -> Result<CoordinationEvent, CoordinationError> {
    store.publish_event(event)
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
    admit_collaboration_action(store, work_item_id, generation)
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

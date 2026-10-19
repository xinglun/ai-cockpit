//! P1 normalization and P2a offline assessment of supplied authority facts.
//!
//! Conditional eligibility is not an action permit. This module performs no
//! storage/provider I/O, issuer or actor authentication, live source resolution,
//! decision recording, or production admission. A caller-supplied observation
//! is not proof that a trusted source adapter or writer/consumer fence exists.

use cockpit_core::{
    Digest,
    clock::{ClockError, ClockSample, FiniteWindow},
};
use cockpit_protocol::{GoalAuthorityDeadline, GoalAuthorityReference, GoalAuthoritySource};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormalizedGoalAuthoritySource {
    pub source_digest: Digest,
    pub validity: FiniteWindow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NormalizedGoalAuthorityDeadline {
    InheritSource,
    CapAt(i64),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormalizedGoalAuthorityReference {
    pub reference_digest: Digest,
    pub not_before_utc_millis: Option<i64>,
    pub deadline: NormalizedGoalAuthorityDeadline,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum GoalAuthorityNormalizationError {
    #[error("{0}")]
    Model(#[from] cockpit_protocol::GoalAuthorityModelError),
    #[error("{0} must be an explicit supported RFC 3339 instant")]
    Timestamp(&'static str),
    #[error("{0} must be exactly representable in milliseconds; no boundary rounding is allowed")]
    Precision(&'static str),
    #[error("{0}")]
    Window(#[from] ClockError),
    #[error("cannot digest descriptive authority bytes: {0}")]
    Serialization(String),
}

pub fn normalize_goal_authority_source(
    source: &GoalAuthoritySource,
) -> Result<NormalizedGoalAuthoritySource, GoalAuthorityNormalizationError> {
    source.validate_shape()?;
    let start = exact_utc_millis(&source.validity.not_before, "validity.notBefore")?;
    let end = exact_utc_millis(&source.validity.expires_at, "validity.expiresAt")?;
    let validity = FiniteWindow::new(start, end)?;
    let source_digest = cockpit_protocol::digest_json(source)
        .map_err(|error| GoalAuthorityNormalizationError::Serialization(error.to_string()))?;
    Ok(NormalizedGoalAuthoritySource {
        source_digest,
        validity,
    })
}

pub fn normalize_goal_authority_reference(
    reference: &GoalAuthorityReference,
) -> Result<NormalizedGoalAuthorityReference, GoalAuthorityNormalizationError> {
    reference.validate_shape()?;
    let not_before_utc_millis = reference
        .not_before
        .as_deref()
        .map(|value| exact_utc_millis(value, "notBefore"))
        .transpose()?;
    let deadline = match &reference.deadline {
        GoalAuthorityDeadline::InheritSource {} => NormalizedGoalAuthorityDeadline::InheritSource,
        GoalAuthorityDeadline::CapAt { expires_at } => {
            let cap = exact_utc_millis(expires_at, "deadline.expiresAt")?;
            if let Some(start) = not_before_utc_millis {
                FiniteWindow::new(start, cap)?;
            }
            NormalizedGoalAuthorityDeadline::CapAt(cap)
        }
    };
    let reference_digest = cockpit_protocol::digest_json(reference)
        .map_err(|error| GoalAuthorityNormalizationError::Serialization(error.to_string()))?;
    Ok(NormalizedGoalAuthorityReference {
        reference_digest,
        not_before_utc_millis,
        deadline,
    })
}

/// An exact supplied binding, never natural-language goal similarity or authentication.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoalAuthoritySubjectKey {
    pub repository_id: String,
    pub grant_id: String,
    pub subject_ref: String,
}

/// The caller's observation claim. `Observed` does not authenticate its producer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuppliedFactObservation {
    Observed,
    Unknown,
    Conflicting,
}

/// Stable supplied termination state, independent of ordinary source-version replacement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuppliedGoalAuthorityState {
    Active,
    Completed,
    Revoked,
    Replaced,
    Unknown,
    Conflicting,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoalAuthorityHeadObservation {
    pub observation: SuppliedFactObservation,
    pub source_version: u64,
    pub source_digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoalAuthorityNarrowing {
    pub work_item_id: String,
    pub parent_work_item_id: Option<String>,
    pub state: SuppliedGoalAuthorityState,
    pub actors: Vec<String>,
    pub operations: Vec<String>,
    pub resources: Vec<String>,
    pub constraints: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoalAuthorityParentFacts {
    pub subject: GoalAuthoritySubjectKey,
    pub reference: GoalAuthorityReference,
    pub work_item: GoalAuthorityNarrowing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedGoalAuthorityFacts {
    pub source: GoalAuthoritySource,
    pub current_head: GoalAuthorityHeadObservation,
    /// Complete, contiguous versions from one through the supplied current head, inclusive.
    pub version_history: Vec<GoalAuthoritySource>,
    pub issuer_observation: SuppliedFactObservation,
    pub actor_observation: SuppliedFactObservation,
    pub subject_state: SuppliedGoalAuthorityState,
    pub parent_chain_observation: SuppliedFactObservation,
    /// Root first, direct parent last. Empty is valid only for a root Work Item.
    pub parent_chain: Vec<GoalAuthorityParentFacts>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoalAuthorityActionFacts {
    pub subject: GoalAuthoritySubjectKey,
    pub actor: String,
    pub operation: String,
    pub resource: String,
    pub deliverable_refs: Vec<String>,
    pub work_item: GoalAuthorityNarrowing,
}

/// Offline conditional result. It is not a Runtime action permit or an admission receipt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GoalAuthorityAssessment {
    EligibleUnderSuppliedFacts {
        source_digest: Digest,
        reference_digest: Digest,
        effective_window: FiniteWindow,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum GoalAuthorityAssessmentError {
    #[error("{0}")]
    Normalization(#[from] GoalAuthorityNormalizationError),
    #[error("invalid supplied fact: {0}")]
    InvalidFact(&'static str),
    #[error("supplied identity mismatch: {0}")]
    IdentityMismatch(&'static str),
    #[error("required supplied observation is unknown or conflicting: {0}")]
    ObservationUnavailable(&'static str),
    #[error("supplied source is not the observed current head")]
    SourceHeadMismatch,
    #[error("supplied version chain is incomplete")]
    VersionChainIncomplete,
    #[error("supplied version chain is forked or inconsistent")]
    VersionChainForked,
    #[error("supplied subject or Work Item is terminal: {0}")]
    Terminated(&'static str),
    #[error("explicit narrowing widens its parent: {0}")]
    ScopeWidening(&'static str),
    #[error("requested action is outside the supplied scope: {0}")]
    ActionOutsideScope(&'static str),
    #[error("constraint semantics are unsupported")]
    UnsupportedConstraint,
    #[error("resource semantics are unsupported: {0}")]
    UnsupportedResource(String),
    #[error("{0}")]
    Clock(#[from] ClockError),
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum GoalAuthorityResolutionError {
    #[error("current goal-authority resolution is unsupported")]
    Unsupported,
    #[error("current goal-authority facts are missing")]
    Missing,
    #[error("current goal-authority observation is unavailable")]
    Unavailable,
}

/// A future adapter must independently bind repository/subject, actual issuer/actor,
/// complete current version and stable termination observations, and writer/consumer
/// ordering. Strings, timestamps and Runtime binary identity do not establish these.
/// P2a supplies no production adapter, persistence, fencing or configuration selector.
pub trait CurrentGoalAuthorityResolver {
    fn resolve_current(
        &self,
        key: &GoalAuthoritySubjectKey,
    ) -> Result<ResolvedGoalAuthorityFacts, GoalAuthorityResolutionError>;
}

#[derive(Default)]
pub struct UnsupportedCurrentGoalAuthorityResolver;

impl CurrentGoalAuthorityResolver for UnsupportedCurrentGoalAuthorityResolver {
    fn resolve_current(
        &self,
        _key: &GoalAuthoritySubjectKey,
    ) -> Result<ResolvedGoalAuthorityFacts, GoalAuthorityResolutionError> {
        Err(GoalAuthorityResolutionError::Unsupported)
    }
}

/// Assess only supplied facts and an already captured sample, with no I/O or time acquisition.
/// The caller must obtain fresh facts/time for each decision; this value does not fence
/// source writers, preserve a monotonic budget, authenticate anyone, or admit an action.
pub fn assess_goal_authority(
    facts: &ResolvedGoalAuthorityFacts,
    reference: &GoalAuthorityReference,
    action: &GoalAuthorityActionFacts,
    sample: ClockSample,
) -> Result<GoalAuthorityAssessment, GoalAuthorityAssessmentError> {
    let source = normalize_goal_authority_source(&facts.source)?;
    let normalized_reference = normalize_goal_authority_reference(reference)?;
    require_source_identity(&facts.source, &action.subject)?;
    require_observed(facts.issuer_observation, "issuer")?;
    require_observed(facts.actor_observation, "actor")?;
    require_observed(facts.current_head.observation, "current head")?;
    require_observed(facts.parent_chain_observation, "complete parent chain")?;
    require_active(facts.subject_state, "stable subject")?;
    if facts.current_head.source_version != facts.source.source_version
        || facts.current_head.source_digest != source.source_digest
    {
        return Err(GoalAuthorityAssessmentError::SourceHeadMismatch);
    }
    validate_version_chain(facts, &action.subject, &source.source_digest)?;
    require_supported_scope(
        &facts.source.actors,
        &facts.source.operations,
        &facts.source.resources,
        &facts.source.constraints,
    )?;
    require_subset(
        &facts.source.operations,
        &facts.source.issuer_evidence.operations,
        "issuer operations",
    )?;

    let mut actors = facts.source.actors.as_slice();
    let mut operations = facts.source.operations.as_slice();
    let mut resources = facts.source.resources.as_slice();
    let mut deliverables = facts.source.deliverable_refs.as_slice();
    let mut window = source.validity;
    let mut parent_id = None;
    let mut visited = std::collections::BTreeSet::new();
    for parent in &facts.parent_chain {
        if parent.subject != action.subject {
            return Err(GoalAuthorityAssessmentError::IdentityMismatch(
                "parent subject",
            ));
        }
        require_work_item_link(&parent.work_item, parent_id, &mut visited)?;
        require_narrow_scope(&parent.work_item, actors, operations, resources)?;
        let normalized = normalize_goal_authority_reference(&parent.reference)?;
        require_reference_scope(&parent.reference, &action.subject, deliverables)?;
        window = narrow_window(window, &normalized)?;
        actors = &parent.work_item.actors;
        operations = &parent.work_item.operations;
        resources = &parent.work_item.resources;
        deliverables = &parent.reference.deliverable_refs;
        parent_id = Some(parent.work_item.work_item_id.as_str());
    }
    require_work_item_link(&action.work_item, parent_id, &mut visited)?;
    require_narrow_scope(&action.work_item, actors, operations, resources)?;
    require_reference_scope(reference, &action.subject, deliverables)?;
    window = narrow_window(window, &normalized_reference)?;
    require_identifier(&action.actor, "action actor")?;
    require_identifier(&action.operation, "action operation")?;
    require_resource(&action.resource)?;
    require_identifiers(&action.deliverable_refs, "action deliverables")?;
    for (value, allowed, field) in [
        (&action.actor, &action.work_item.actors, "actor"),
        (&action.operation, &action.work_item.operations, "operation"),
        (&action.resource, &action.work_item.resources, "resource"),
    ] {
        if !allowed.contains(value) {
            return Err(GoalAuthorityAssessmentError::ActionOutsideScope(field));
        }
    }
    if !action
        .deliverable_refs
        .iter()
        .all(|part| reference.deliverable_refs.contains(part))
    {
        return Err(GoalAuthorityAssessmentError::ActionOutsideScope(
            "deliverables",
        ));
    }
    window.remaining_at(sample.utc_millis)?;
    Ok(GoalAuthorityAssessment::EligibleUnderSuppliedFacts {
        source_digest: source.source_digest,
        reference_digest: normalized_reference.reference_digest,
        effective_window: window,
    })
}

fn require_observed(
    observation: SuppliedFactObservation,
    field: &'static str,
) -> Result<(), GoalAuthorityAssessmentError> {
    if observation != SuppliedFactObservation::Observed {
        return Err(GoalAuthorityAssessmentError::ObservationUnavailable(field));
    }
    Ok(())
}

fn require_active(
    state: SuppliedGoalAuthorityState,
    field: &'static str,
) -> Result<(), GoalAuthorityAssessmentError> {
    match state {
        SuppliedGoalAuthorityState::Active => Ok(()),
        SuppliedGoalAuthorityState::Unknown | SuppliedGoalAuthorityState::Conflicting => {
            Err(GoalAuthorityAssessmentError::ObservationUnavailable(field))
        }
        _ => Err(GoalAuthorityAssessmentError::Terminated(field)),
    }
}

fn require_identifier(
    value: &str,
    field: &'static str,
) -> Result<(), GoalAuthorityAssessmentError> {
    if value.is_empty()
        || value.trim() != value
        || value
            .chars()
            .any(|c| c.is_control() || matches!(c, '*' | '?' | '[' | ']'))
    {
        return Err(GoalAuthorityAssessmentError::InvalidFact(field));
    }
    Ok(())
}

fn require_identifiers(
    values: &[String],
    field: &'static str,
) -> Result<(), GoalAuthorityAssessmentError> {
    let mut seen = std::collections::BTreeSet::new();
    if values.is_empty() {
        return Err(GoalAuthorityAssessmentError::InvalidFact(field));
    }
    for value in values {
        require_identifier(value, field)?;
        if !seen.insert(value) {
            return Err(GoalAuthorityAssessmentError::InvalidFact(field));
        }
    }
    Ok(())
}

/// P2a supports only exact lexical repo-relative identifiers, with no glob,
/// path alias, provider URI, filesystem resolution, or implicit subtree coverage.
fn require_resource(value: &str) -> Result<(), GoalAuthorityAssessmentError> {
    let valid = value.strip_prefix("repo:").is_some_and(|path| {
        !path.is_empty()
            && !path.chars().any(|c| {
                c.is_whitespace()
                    || c.is_control()
                    || matches!(c, '\\' | ':' | '*' | '?' | '[' | ']')
            })
            && path.split('/').all(|part| !matches!(part, "" | "." | ".."))
    });
    if !valid {
        return Err(GoalAuthorityAssessmentError::UnsupportedResource(
            value.into(),
        ));
    }
    Ok(())
}

fn require_source_identity(
    source: &GoalAuthoritySource,
    key: &GoalAuthoritySubjectKey,
) -> Result<(), GoalAuthorityAssessmentError> {
    if source.repository_id != key.repository_id
        || source.grant_id != key.grant_id
        || source.subject_ref != key.subject_ref
    {
        return Err(GoalAuthorityAssessmentError::IdentityMismatch(
            "repository/grant/subject",
        ));
    }
    Ok(())
}

fn validate_version_chain(
    facts: &ResolvedGoalAuthorityFacts,
    key: &GoalAuthoritySubjectKey,
    current_digest: &Digest,
) -> Result<(), GoalAuthorityAssessmentError> {
    if u64::try_from(facts.version_history.len()).ok() != Some(facts.source.source_version) {
        return Err(GoalAuthorityAssessmentError::VersionChainIncomplete);
    }
    let mut previous = None;
    for (index, version) in facts.version_history.iter().enumerate() {
        require_source_identity(version, key)?;
        let normalized = normalize_goal_authority_source(version)?;
        if version.source_version != index as u64 + 1 {
            return Err(GoalAuthorityAssessmentError::VersionChainIncomplete);
        }
        if version.predecessor_version_digest.as_ref() != previous.as_ref() {
            return Err(GoalAuthorityAssessmentError::VersionChainForked);
        }
        if version.termination.is_some() {
            return Err(GoalAuthorityAssessmentError::Terminated(
                "source version history",
            ));
        }
        if !version.constraints.is_empty() {
            return Err(GoalAuthorityAssessmentError::UnsupportedConstraint);
        }
        previous = Some(normalized.source_digest);
    }
    if previous.as_ref() != Some(current_digest) {
        return Err(GoalAuthorityAssessmentError::VersionChainForked);
    }
    Ok(())
}

fn require_supported_scope(
    actors: &[String],
    operations: &[String],
    resources: &[String],
    constraints: &[String],
) -> Result<(), GoalAuthorityAssessmentError> {
    if !constraints.is_empty() {
        return Err(GoalAuthorityAssessmentError::UnsupportedConstraint);
    }
    require_identifiers(actors, "actors")?;
    require_identifiers(operations, "operations")?;
    // Resource errors retain their unsupported classification rather than a generic glob error.
    if resources.is_empty() {
        return Err(GoalAuthorityAssessmentError::InvalidFact("resources"));
    }
    for resource in resources {
        require_resource(resource)?;
    }
    require_identifiers(resources, "resources")
}

fn require_subset(
    child: &[String],
    parent: &[String],
    field: &'static str,
) -> Result<(), GoalAuthorityAssessmentError> {
    if !child.iter().all(|value| parent.contains(value)) {
        return Err(GoalAuthorityAssessmentError::ScopeWidening(field));
    }
    Ok(())
}

fn require_narrow_scope(
    child: &GoalAuthorityNarrowing,
    actors: &[String],
    operations: &[String],
    resources: &[String],
) -> Result<(), GoalAuthorityAssessmentError> {
    require_supported_scope(
        &child.actors,
        &child.operations,
        &child.resources,
        &child.constraints,
    )?;
    require_subset(&child.actors, actors, "actors")?;
    require_subset(&child.operations, operations, "operations")?;
    require_subset(&child.resources, resources, "resources")
}

fn require_reference_scope(
    reference: &GoalAuthorityReference,
    key: &GoalAuthoritySubjectKey,
    deliverables: &[String],
) -> Result<(), GoalAuthorityAssessmentError> {
    if reference.grant_id != key.grant_id || reference.subject_ref != key.subject_ref {
        return Err(GoalAuthorityAssessmentError::IdentityMismatch(
            "reference grant/subject",
        ));
    }
    require_subset(&reference.deliverable_refs, deliverables, "deliverables")
}

fn require_work_item_link<'a>(
    work_item: &'a GoalAuthorityNarrowing,
    expected_parent: Option<&str>,
    visited: &mut std::collections::BTreeSet<&'a str>,
) -> Result<(), GoalAuthorityAssessmentError> {
    require_active(work_item.state, "Work Item")?;
    require_identifier(&work_item.work_item_id, "Work Item id")?;
    if work_item.parent_work_item_id.as_deref() != expected_parent
        || !visited.insert(&work_item.work_item_id)
    {
        return Err(GoalAuthorityAssessmentError::InvalidFact(
            "complete parent chain",
        ));
    }
    Ok(())
}

fn narrow_window(
    parent: FiniteWindow,
    reference: &NormalizedGoalAuthorityReference,
) -> Result<FiniteWindow, GoalAuthorityAssessmentError> {
    let start = reference
        .not_before_utc_millis
        .unwrap_or(parent.not_before_utc_millis());
    let end = match reference.deadline {
        NormalizedGoalAuthorityDeadline::InheritSource => parent.expires_at_utc_millis(),
        NormalizedGoalAuthorityDeadline::CapAt(cap) => cap,
    };
    if start < parent.not_before_utc_millis() || end > parent.expires_at_utc_millis() {
        return Err(GoalAuthorityAssessmentError::ScopeWidening("validity"));
    }
    Ok(FiniteWindow::new(start, end)?)
}

fn exact_utc_millis(
    value: &str,
    field: &'static str,
) -> Result<i64, GoalAuthorityNormalizationError> {
    // RFC 3339's -00:00 marks an unknown local offset. It is not evidence of
    // a known UTC boundary. Leap seconds also have no unique epoch-ms form.
    if value.ends_with("-00:00") {
        return Err(GoalAuthorityNormalizationError::Timestamp(field));
    }
    let instant = chrono::DateTime::parse_from_rfc3339(value)
        .map_err(|_| GoalAuthorityNormalizationError::Timestamp(field))?;
    let nanos = instant.timestamp_subsec_nanos();
    if nanos >= 1_000_000_000 {
        return Err(GoalAuthorityNormalizationError::Timestamp(field));
    }
    // Chrono skips fractional digits beyond nanoseconds. Inspect all original
    // digits so a nonzero tail cannot disappear before this precision check.
    if value.split_once('.').is_some_and(|(_, fraction)| {
        fraction
            .bytes()
            .take_while(u8::is_ascii_digit)
            .skip(3)
            .any(|digit| digit != b'0')
    }) {
        return Err(GoalAuthorityNormalizationError::Precision(field));
    }
    Ok(instant.timestamp_millis())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn source() -> GoalAuthoritySource {
        serde_json::from_value(json!({
            "schemaVersion":1, "repositoryId":Digest::sha256_bytes(b"normalization-fixture").to_string(),
            "grantId":"grant-a", "subjectRef":"goal-a", "sourceVersion":1,
            "goal":"fixture only", "deliverableRefs":["part-a"], "actors":["agent:fixture"],
            "operations":["source.modify"], "resources":["repo:src/example.rs"], "constraints":[],
            "issuerEvidence":{"assurance":"self_declared","actor":"human:fixture","authoritySource":"fixture-only",
                "operations":["source.modify"],"policyRefs":[],"evidenceRefs":[]},
            "validity":{"notBefore":"2030-01-01T00:00:00Z","expiresAt":"2030-01-01T01:00:00Z"}
        })).unwrap()
    }

    fn reference() -> GoalAuthorityReference {
        serde_json::from_value(
            json!({"grantId":"grant-a","subjectRef":"goal-a","versionResolution":"current",
            "deliverableRefs":["part-a"],"deadline":{"mode":"inherit_source"}}),
        )
        .unwrap()
    }

    #[test]
    fn source_normalizes_exact_utc_milliseconds_and_binds_descriptive_bytes() {
        let input = source();
        let normalized = normalize_goal_authority_source(&input).unwrap();
        assert_eq!(
            normalized.validity.not_before_utc_millis(),
            1_893_456_000_000
        );
        assert_eq!(
            normalized.validity.expires_at_utc_millis(),
            1_893_459_600_000
        );
        assert_eq!(
            normalized.source_digest,
            cockpit_protocol::digest_json(&input).unwrap()
        );
    }

    #[test]
    fn offset_equivalence_does_not_rewrite_original_source_identity() {
        let input = source();
        let first = normalize_goal_authority_source(&input).unwrap();
        let mut offset = input.clone();
        offset.validity.not_before = "2030-01-01T09:00:00+09:00".into();
        offset.validity.expires_at = "2030-01-01T10:00:00+09:00".into();
        let second = normalize_goal_authority_source(&offset).unwrap();
        assert_eq!(first.validity, second.validity);
        assert_ne!(first.source_digest, second.source_digest);
        assert_eq!(offset.validity.not_before, "2030-01-01T09:00:00+09:00");
    }

    #[test]
    fn source_rejects_equal_or_reversed_end_without_a_grace_period() {
        for end in ["2030-01-01T00:00:00Z", "2029-12-31T23:59:59Z"] {
            let mut input = source();
            input.validity.expires_at = end.into();
            assert_eq!(
                normalize_goal_authority_source(&input).unwrap_err(),
                GoalAuthorityNormalizationError::Window(ClockError::InvalidWindow)
            );
        }
    }

    #[test]
    fn non_finite_or_malformed_deadline_text_is_rejected() {
        for end in [
            "infinite",
            "until_done",
            "999999999999999999999999",
            "2030-02-30T00:00:00Z",
        ] {
            let mut input = source();
            input.validity.expires_at = end.into();
            assert_eq!(
                normalize_goal_authority_source(&input).unwrap_err(),
                GoalAuthorityNormalizationError::Timestamp("validity.expiresAt")
            );
        }
    }

    #[test]
    fn submillisecond_precision_is_rejected_instead_of_rounding_a_boundary() {
        let mut input = source();
        input.validity.not_before = "2030-01-01T00:00:00.000001Z".into();
        assert_eq!(
            normalize_goal_authority_source(&input).unwrap_err(),
            GoalAuthorityNormalizationError::Precision("validity.notBefore")
        );
        let mut input = source();
        input.validity.expires_at = "2030-01-01T01:00:00.999999Z".into();
        assert_eq!(
            normalize_goal_authority_source(&input).unwrap_err(),
            GoalAuthorityNormalizationError::Precision("validity.expiresAt")
        );
    }

    #[test]
    fn regression_source_start_rejects_nonzero_digits_beyond_nanoseconds() {
        for fraction in [
            "1230000001",
            "000000000000000000000000001",
            "9990000000000000001",
        ] {
            let mut input = source();
            input.validity.not_before = format!("2030-01-01T00:00:00.{fraction}Z");
            assert_eq!(
                normalize_goal_authority_source(&input).map(|_| ()),
                Err(GoalAuthorityNormalizationError::Precision(
                    "validity.notBefore"
                )),
                "{}",
                input.validity.not_before
            );
        }
    }

    #[test]
    fn regression_source_expiry_rejects_nonzero_digits_beyond_nanoseconds() {
        for fraction in [
            "1230000001",
            "000000000000000000000000001",
            "9990000000000000001",
        ] {
            let mut input = source();
            input.validity.expires_at = format!("2030-01-01T01:00:00.{fraction}+00:00");
            assert_eq!(
                normalize_goal_authority_source(&input).map(|_| ()),
                Err(GoalAuthorityNormalizationError::Precision(
                    "validity.expiresAt"
                )),
                "{}",
                input.validity.expires_at
            );
        }
    }

    #[test]
    fn regression_reference_start_rejects_nonzero_digits_beyond_nanoseconds() {
        for fraction in [
            "1230000001",
            "000000000000000000000000001",
            "9990000000000000001",
        ] {
            let mut input = reference();
            input.not_before = Some(format!("2030-01-01T00:00:00.{fraction}Z"));
            assert_eq!(
                normalize_goal_authority_reference(&input).map(|_| ()),
                Err(GoalAuthorityNormalizationError::Precision("notBefore")),
                "{:?}",
                input.not_before
            );
        }
    }

    #[test]
    fn regression_reference_cap_rejects_nonzero_digits_beyond_nanoseconds() {
        for fraction in [
            "1230000001",
            "000000000000000000000000001",
            "9990000000000000001",
        ] {
            let mut input = reference();
            input.deadline = GoalAuthorityDeadline::CapAt {
                expires_at: format!("2030-01-01T00:30:00.{fraction}-01:00"),
            };
            assert_eq!(
                normalize_goal_authority_reference(&input).map(|_| ()),
                Err(GoalAuthorityNormalizationError::Precision(
                    "deadline.expiresAt"
                )),
                "{:?}",
                input.deadline
            );
        }
    }

    #[test]
    fn arbitrarily_long_zero_padding_preserves_exact_millisecond_boundaries() {
        for fraction in ["123", "1230", "123000000", "123000000000000000000000000000"] {
            let mut input = source();
            input.validity.not_before = format!("2030-01-01T00:00:00.{fraction}+00:00");
            let value = normalize_goal_authority_source(&input).unwrap();
            assert_eq!(value.validity.not_before_utc_millis(), 1_893_456_000_123);
            let mut input = reference();
            input.not_before = Some(format!("2030-01-01T00:00:00.{fraction}+01:00"));
            input.deadline = GoalAuthorityDeadline::CapAt {
                expires_at: format!("2030-01-01T00:30:00.{fraction}Z"),
            };
            let value = normalize_goal_authority_reference(&input).unwrap();
            assert_eq!(value.not_before_utc_millis, Some(1_893_452_400_123));
            assert_eq!(
                value.deadline,
                NormalizedGoalAuthorityDeadline::CapAt(1_893_457_800_123)
            );
        }
    }

    #[test]
    fn exact_millisecond_fraction_is_preserved() {
        let mut input = source();
        input.validity.not_before = "2030-01-01T00:00:00.123Z".into();
        input.validity.expires_at = "2030-01-01T00:00:00.124Z".into();
        let value = normalize_goal_authority_source(&input).unwrap();
        assert_eq!(value.validity.remaining_at(1_893_456_000_123), Ok(1));
        assert_eq!(
            value.validity.remaining_at(1_893_456_000_124),
            Err(ClockError::Expired)
        );
    }

    #[test]
    fn unknown_offset_and_leap_second_cannot_be_silently_normalized() {
        for start in ["2030-01-01T00:00:00-00:00", "2016-12-31T23:59:60Z"] {
            let mut input = source();
            input.validity.not_before = start.into();
            assert_eq!(
                normalize_goal_authority_source(&input).unwrap_err(),
                GoalAuthorityNormalizationError::Timestamp("validity.notBefore")
            );
        }
    }

    #[test]
    fn reference_inherits_current_source_without_copying_a_fixed_cap() {
        let input = reference();
        let value = normalize_goal_authority_reference(&input).unwrap();
        assert_eq!(
            value.deadline,
            NormalizedGoalAuthorityDeadline::InheritSource
        );
        assert_eq!(value.not_before_utc_millis, None);
        assert_eq!(
            value.reference_digest,
            cockpit_protocol::digest_json(&input).unwrap()
        );
    }

    #[test]
    fn explicit_reference_cap_and_later_start_remain_exact() {
        let mut input = reference();
        input.deadline = GoalAuthorityDeadline::CapAt {
            expires_at: "2030-01-01T00:30:00Z".into(),
        };
        input.not_before = Some("2030-01-01T00:10:00Z".into());
        let value = normalize_goal_authority_reference(&input).unwrap();
        assert_eq!(
            value.deadline,
            NormalizedGoalAuthorityDeadline::CapAt(1_893_457_800_000)
        );
        assert_eq!(value.not_before_utc_millis, Some(1_893_456_600_000));
    }

    #[test]
    fn malformed_cap_and_reversed_reference_bounds_are_rejected() {
        let mut input = reference();
        input.deadline = GoalAuthorityDeadline::CapAt {
            expires_at: "until_done".into(),
        };
        assert_eq!(
            normalize_goal_authority_reference(&input).unwrap_err(),
            GoalAuthorityNormalizationError::Timestamp("deadline.expiresAt")
        );
        input.deadline = GoalAuthorityDeadline::CapAt {
            expires_at: "2030-01-01T00:30:00Z".into(),
        };
        input.not_before = Some("2030-01-01T00:30:00Z".into());
        assert_eq!(
            normalize_goal_authority_reference(&input).unwrap_err(),
            GoalAuthorityNormalizationError::Window(ClockError::InvalidWindow)
        );
    }

    #[test]
    fn invalid_shape_is_not_disguised_as_timestamp_validation() {
        let mut input = source();
        input.actors.clear();
        assert_eq!(
            normalize_goal_authority_source(&input).unwrap_err(),
            GoalAuthorityNormalizationError::Model(
                cockpit_protocol::GoalAuthorityModelError::InvalidField("actors")
            )
        );
    }

    #[test]
    fn historical_terminated_source_is_readable_without_becoming_authority() {
        let mut input = source();
        input.validity.not_before = "1960-01-01T00:00:00Z".into();
        input.validity.expires_at = "1960-01-01T01:00:00Z".into();
        input.termination = Some(cockpit_protocol::GoalAuthorityTermination {
            kind: cockpit_protocol::GoalAuthorityTerminationKind::Revoked,
            evidence_refs: vec!["decision:historic-fixture".into()],
        });
        let value = normalize_goal_authority_source(&input).unwrap();
        assert!(value.validity.expires_at_utc_millis() < 0);
        assert_eq!(
            input.termination.unwrap().kind,
            cockpit_protocol::GoalAuthorityTerminationKind::Revoked
        );
    }
}

//! Pure P1 normalization of descriptive authority metadata.
//!
//! This module does not read a live grant, authenticate an issuer, resolve a
//! current version, prove goal coverage, or admit an operation. It performs no
//! storage or provider I/O. Those are separate, unimplemented P2+ boundaries.

use cockpit_core::{
    Digest,
    clock::{ClockError, FiniteWindow},
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
    if nanos % 1_000_000 != 0 {
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

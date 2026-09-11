//! Pure planning for release acceptance recovery.
//!
//! This module does not execute a command or mutate a release. It classifies
//! persisted phase results so an executor can retry the first unsafe phase,
//! reuse verified receipts, and stop all dependent phases after a failure.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::acceptance::{
    AcceptanceScope, CleanupState, EvidenceReference, PhaseResult, PhaseStatus, ReleaseIdentity,
    ReleasePhase,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureKind {
    Interruption,
    Timeout,
    Network,
    Runner,
    Cleanup,
    InputChanged,
    Validation,
    IdentityMismatch,
    ScopeChanged,
    AuthorityChanged,
    BaseChanged,
    AlreadyPublished,
    Unknown,
}

impl FailureKind {
    pub const fn recommended_strategy(self) -> RetryStrategy {
        match self {
            Self::Interruption | Self::Timeout | Self::Network | Self::Runner => {
                RetryStrategy::RetryCurrentPhase
            }
            Self::Cleanup => RetryStrategy::RetryCleanupOnly,
            Self::InputChanged => RetryStrategy::RestartFromPhase,
            Self::ScopeChanged | Self::AuthorityChanged | Self::BaseChanged => {
                RetryStrategy::RequireSuccessor
            }
            Self::Validation | Self::IdentityMismatch | Self::AlreadyPublished | Self::Unknown => {
                RetryStrategy::Block
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryStrategy {
    RetryCurrentPhase,
    RetryCleanupOnly,
    RestartFromPhase,
    RequireSuccessor,
    Block,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PhaseFailure {
    pub kind: FailureKind,
    pub strategy: RetryStrategy,
    pub code: String,
    pub diagnostic: String,
}

impl PhaseFailure {
    pub fn new(kind: FailureKind, code: impl Into<String>, diagnostic: impl Into<String>) -> Self {
        Self {
            strategy: kind.recommended_strategy(),
            kind,
            code: code.into(),
            diagnostic: diagnostic.into(),
        }
    }

    pub fn is_consistent(&self) -> bool {
        self.strategy == self.kind.recommended_strategy()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReuseReason {
    IdentityAndEvidenceMatch,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryReason {
    NonTerminalResult,
    CleanupIncomplete,
    RetryableFailure,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InvalidationReason {
    InputChanged,
    CorruptIdentityDigest,
    MissingEvidence,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockReason {
    FailedPrerequisite { phase: ReleasePhase },
    SuccessorRequired,
    IdentityMismatch,
    CorruptResult,
    MissingFailure,
    InconsistentFailure,
    AlreadyPublishedConflict,
    StoredBlocked,
    ValidationFailure,
    UnknownFailure,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "decision")]
pub enum ReuseDecision {
    Reuse {
        reason: ReuseReason,
        evidence: EvidenceReference,
    },
    Retry {
        strategy: RetryStrategy,
        reason: RetryReason,
    },
    Invalidated {
        from: ReleasePhase,
        reason: InvalidationReason,
    },
    Blocked {
        reason: BlockReason,
    },
}

/// Classify one persisted result against the current operation identity.
pub fn evaluate_reuse(result: &PhaseResult, expected: &ReleaseIdentity) -> ReuseDecision {
    if result.identity_digest != result.identity.digest() {
        return ReuseDecision::Invalidated {
            from: result.phase,
            reason: InvalidationReason::CorruptIdentityDigest,
        };
    }
    if result.identity != *expected {
        return ReuseDecision::Invalidated {
            from: result.phase,
            reason: InvalidationReason::InputChanged,
        };
    }

    match result.status {
        PhaseStatus::Succeeded => match result.cleanup {
            CleanupState::NotRequired | CleanupState::Succeeded => {
                let Some(evidence) = result.evidence.clone() else {
                    return ReuseDecision::Invalidated {
                        from: result.phase,
                        reason: InvalidationReason::MissingEvidence,
                    };
                };
                ReuseDecision::Reuse {
                    reason: ReuseReason::IdentityAndEvidenceMatch,
                    evidence,
                }
            }
            CleanupState::Pending | CleanupState::Failed => ReuseDecision::Retry {
                strategy: RetryStrategy::RetryCleanupOnly,
                reason: RetryReason::CleanupIncomplete,
            },
        },
        PhaseStatus::Pending | PhaseStatus::Running => ReuseDecision::Retry {
            strategy: RetryStrategy::RetryCurrentPhase,
            reason: RetryReason::NonTerminalResult,
        },
        PhaseStatus::Failed | PhaseStatus::Interrupted | PhaseStatus::TimedOut => {
            let Some(failure) = result.failure.as_ref() else {
                return ReuseDecision::Blocked {
                    reason: BlockReason::MissingFailure,
                };
            };
            if !failure.is_consistent() {
                return ReuseDecision::Blocked {
                    reason: BlockReason::InconsistentFailure,
                };
            }
            match failure.strategy {
                RetryStrategy::RetryCurrentPhase => ReuseDecision::Retry {
                    strategy: RetryStrategy::RetryCurrentPhase,
                    reason: RetryReason::RetryableFailure,
                },
                RetryStrategy::RetryCleanupOnly => ReuseDecision::Retry {
                    strategy: RetryStrategy::RetryCleanupOnly,
                    reason: RetryReason::CleanupIncomplete,
                },
                RetryStrategy::RestartFromPhase => ReuseDecision::Invalidated {
                    from: result.phase,
                    reason: InvalidationReason::InputChanged,
                },
                RetryStrategy::RequireSuccessor => ReuseDecision::Blocked {
                    reason: BlockReason::SuccessorRequired,
                },
                RetryStrategy::Block => ReuseDecision::Blocked {
                    reason: block_reason_for(failure.kind),
                },
            }
        }
        PhaseStatus::Blocked => ReuseDecision::Blocked {
            reason: BlockReason::StoredBlocked,
        },
    }
}

fn block_reason_for(kind: FailureKind) -> BlockReason {
    match kind {
        FailureKind::AlreadyPublished => BlockReason::AlreadyPublishedConflict,
        FailureKind::IdentityMismatch => BlockReason::IdentityMismatch,
        FailureKind::Validation => BlockReason::ValidationFailure,
        FailureKind::Unknown => BlockReason::UnknownFailure,
        _ => BlockReason::UnknownFailure,
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum PhaseAction {
    NotApplicable {
        phase: ReleasePhase,
    },
    Reuse {
        phase: ReleasePhase,
        evidence: EvidenceReference,
    },
    Run {
        phase: ReleasePhase,
    },
    Retry {
        phase: ReleasePhase,
        strategy: RetryStrategy,
    },
    Blocked {
        phase: ReleasePhase,
        reason: BlockReason,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryPlanError {
    DuplicatePhase { phase: ReleasePhase },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecoveryPlan {
    pub actions: Vec<PhaseAction>,
}

impl RecoveryPlan {
    /// Build a deterministic resume plan. Once a phase is not reusable, all
    /// dependent phases are blocked until that first phase succeeds.
    pub fn from_results(
        expected: &ReleaseIdentity,
        results: &[PhaseResult],
    ) -> Result<Self, RecoveryPlanError> {
        Self::from_results_for_scope(expected, results, AcceptanceScope::Full)
    }

    /// Build a plan for the graph owned by one acceptance executor. Candidate
    /// acceptance has no publication prerequisite, while public acceptance
    /// begins at the already-published Release. Results outside the selected
    /// scope are ignored, but duplicate phases remain invalid.
    pub fn from_results_for_scope(
        expected: &ReleaseIdentity,
        results: &[PhaseResult],
        scope: AcceptanceScope,
    ) -> Result<Self, RecoveryPlanError> {
        let mut by_phase = BTreeMap::new();
        for result in results {
            if by_phase.insert(result.phase, result).is_some() {
                return Err(RecoveryPlanError::DuplicatePhase {
                    phase: result.phase,
                });
            }
        }

        let phases = scope.phases();
        let mut actions = Vec::with_capacity(ReleasePhase::all().len());
        let mut blocked_by = None;
        for &phase in ReleasePhase::all() {
            if !phases.contains(&phase) {
                actions.push(PhaseAction::NotApplicable { phase });
                continue;
            }
            if let Some(prerequisite) = blocked_by {
                actions.push(PhaseAction::Blocked {
                    phase,
                    reason: BlockReason::FailedPrerequisite {
                        phase: prerequisite,
                    },
                });
                continue;
            }

            let Some(result) = by_phase.get(&phase) else {
                actions.push(PhaseAction::Run { phase });
                blocked_by = Some(phase);
                continue;
            };

            let decision = evaluate_reuse(result, expected);
            match decision {
                ReuseDecision::Reuse { evidence, .. } => {
                    actions.push(PhaseAction::Reuse { phase, evidence });
                }
                ReuseDecision::Retry { strategy, .. } => {
                    actions.push(PhaseAction::Retry { phase, strategy });
                    blocked_by = Some(phase);
                }
                ReuseDecision::Invalidated { from, .. } => {
                    actions.push(PhaseAction::Retry {
                        phase: from,
                        strategy: RetryStrategy::RestartFromPhase,
                    });
                    blocked_by = Some(from);
                }
                ReuseDecision::Blocked { reason } => {
                    actions.push(PhaseAction::Blocked { phase, reason });
                    blocked_by = Some(phase);
                }
            }
        }
        Ok(Self { actions })
    }

    pub fn first_execution(&self) -> Option<&PhaseAction> {
        self.actions
            .iter()
            .find(|action| matches!(action, PhaseAction::Run { .. } | PhaseAction::Retry { .. }))
    }
}

use serde::{Deserialize, Serialize};
use sha2::{Digest as ShaDigest, Sha256};
use std::{fmt, str::FromStr};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkItemState {
    Created,
    PreflightReady,
    ImplementationActive,
    VerificationPending,
    FinishReady,
    Archived,
    Closed,
    Paused,
    Blocked,
    Stale,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvolutionClass {
    L0,
    L1,
    L2,
    L3,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blocker {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafeAction {
    pub code: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanDecisionRequirement {
    pub question: String,
    pub options: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionState {
    Green,
    Yellow,
    Red,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    ReadOnly,
    Write,
    Destructive,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityState {
    Authorized,
    Missing,
    NotEvaluated,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceState {
    Complete,
    Missing,
    Stale,
    Contradictory,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceInput {
    pub scope: Vec<String>,
    pub out_of_scope: Vec<String>,
    pub changed_paths: Vec<String>,
    pub action: ActionKind,
    pub authority: AuthorityState,
    pub evidence: EvidenceState,
    pub untrusted_material: bool,
    pub test_weakening: bool,
    pub coverage_weakening: bool,
    #[serde(default)]
    pub explicit_blockers: Vec<String>,
    #[serde(default)]
    pub explicit_unknowns: Vec<String>,
    #[serde(default)]
    pub outcome_state_override: Option<String>,
    #[serde(default)]
    pub authority_override: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceDecision {
    pub state: DecisionState,
    pub blockers: Vec<String>,
    pub unknowns: Vec<String>,
    pub safe_actions: Vec<String>,
    pub required_checks: Vec<String>,
    pub authority: String,
    pub outcome_state: String,
}

fn matches_pattern(path: &str, pattern: &str) -> bool {
    if pattern == "**" || pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix("/**") {
        return path == prefix || path.starts_with(&format!("{prefix}/"));
    }
    path == pattern
}

pub fn evaluate(input: GovernanceInput) -> GovernanceDecision {
    let mut blockers = Vec::new();
    let mut unknowns = Vec::new();
    let mut safe_actions = Vec::new();
    let mut required_checks = Vec::new();

    for finding in &input.explicit_blockers {
        blockers.push(finding.clone());
        match finding.as_str() {
            "scope_exceeded" => {
                safe_actions.push("stop_and_request_new_contract".into());
                required_checks.push("scope".into());
            }
            "destructive_change_without_authority" => {
                safe_actions.push("stop_and_request_human_authority".into());
                required_checks.push("authority".into());
            }
            "unsafe_deletion_request" => {
                safe_actions.push("stop_and_request_human_authority".into());
                required_checks.push("destructive_operation".into());
            }
            "unsupported_completion_claim" => {
                safe_actions.push("remove_claim_or_provide_evidence".into());
                required_checks.push("completion_evidence".into());
            }
            "human_authority_missing" => {
                safe_actions.push("request_human_decision".into());
                required_checks.push("authority".into());
            }
            "archive_invalid" => {
                safe_actions.push("preserve_active_work_item".into());
                safe_actions.push("repair_archive_evidence".into());
                required_checks.push("archive_integrity".into());
            }
            "stale_contract" => {
                safe_actions.push("stop_and_refresh_contract".into());
                required_checks.push("contract_freshness".into());
            }
            "cross_work_item_evidence" => {
                safe_actions.push("rerun_evidence_for_current_work_item".into());
                required_checks.push("evidence_binding".into());
            }
            "test_weakening" => {
                safe_actions.push("restore_verification_strength".into());
                safe_actions.push("request_human_decision".into());
                required_checks.push("test_integrity".into());
            }
            "coverage_weakening" => {
                safe_actions.push("restore_coverage_requirement".into());
                safe_actions.push("request_human_decision".into());
                required_checks.push("coverage_integrity".into());
            }
            "repository_material_inspection_unavailable" => {
                safe_actions.push("inspect_repository_material".into());
                safe_actions.push("rerun_preflight".into());
                required_checks.push("input_trust".into());
            }
            "test_weakening_inspection_unavailable" => {
                safe_actions.push("inspect_test_change".into());
                safe_actions.push("rerun_preflight".into());
                required_checks.push("test_integrity".into());
            }
            "coverage_weakening_inspection_unavailable" => {
                safe_actions.push("inspect_coverage_change".into());
                safe_actions.push("rerun_preflight".into());
                required_checks.push("coverage_integrity".into());
            }
            "evidence_contradictory" => {
                safe_actions.push("stop_and_reconcile_evidence".into());
                required_checks.push("evidence_consistency".into());
            }
            _ => safe_actions.push("stop_and_request_human_decision".into()),
        }
    }
    unknowns.extend(input.explicit_unknowns.iter().cloned());
    for unknown in &input.explicit_unknowns {
        match unknown.as_str() {
            "required_evidence_missing" => {
                safe_actions.push("collect_required_evidence".into());
                safe_actions.push("rerun_preflight".into());
                required_checks.push("verification".into());
            }
            "evidence_stale" => {
                safe_actions.push("rerun_affected_checks".into());
                safe_actions.push("rerun_preflight".into());
                required_checks.push("evidence_freshness".into());
            }
            "repository_material_untrusted" => {
                safe_actions.push("treat_material_as_data".into());
                safe_actions.push("continue_with_explicit_policy".into());
                required_checks.push("input_trust".into());
            }
            "provider_result_unknown" => {
                safe_actions.push("obtain_provider_receipt".into());
                safe_actions.push("rerun_preflight".into());
                required_checks.push("external_evidence".into());
            }
            "destructive_change_without_authority" => {
                safe_actions.push("stop_and_request_human_authority".into());
                required_checks.push("authority".into());
                required_checks.push("scope".into());
            }
            "human_authority_missing" => {
                safe_actions.push("request_human_decision".into());
                required_checks.push("authority".into());
            }
            "coverage_weakening" => {
                safe_actions.push("restore_coverage_requirement".into());
                safe_actions.push("request_human_decision".into());
                required_checks.push("coverage_integrity".into());
            }
            _ => safe_actions.push("collect_missing_evidence".into()),
        }
    }

    if input.changed_paths.iter().any(|path| {
        !input
            .scope
            .iter()
            .any(|pattern| matches_pattern(path, pattern))
    }) {
        blockers.push("scope_exceeded".into());
        safe_actions.push("stop_and_request_new_contract".into());
        required_checks.push("scope".into());
    }
    if input.changed_paths.iter().any(|path| {
        input
            .out_of_scope
            .iter()
            .any(|pattern| matches_pattern(path, pattern))
    }) {
        blockers.push("out_of_scope_changed".into());
        safe_actions.push("restore_out_of_scope_boundary".into());
        required_checks.push("scope".into());
    }
    if input.action == ActionKind::Destructive && input.authority != AuthorityState::Authorized {
        unknowns.push("destructive_change_without_authority".into());
        safe_actions.push("stop_and_request_human_authority".into());
        required_checks.push("authority".into());
        required_checks.push("scope".into());
    }
    if input.test_weakening {
        blockers.push("test_weakening".into());
        safe_actions.push("restore_verification_strength".into());
        required_checks.push("test_integrity".into());
    }
    if input.coverage_weakening {
        unknowns.push("coverage_weakening".into());
        safe_actions.push("restore_coverage_requirement".into());
        safe_actions.push("request_human_decision".into());
        required_checks.push("coverage_integrity".into());
    }

    match input.evidence {
        EvidenceState::Contradictory => {
            blockers.push("evidence_contradictory".into());
            safe_actions.push("stop_and_reconcile_evidence".into());
            required_checks.push("evidence_consistency".into());
        }
        EvidenceState::Missing => {
            unknowns.push("required_evidence_missing".into());
            safe_actions.push("collect_required_evidence".into());
            required_checks.push("verification".into());
        }
        EvidenceState::Stale => {
            unknowns.push("evidence_stale".into());
            safe_actions.push("rerun_affected_checks".into());
            required_checks.push("evidence_freshness".into());
        }
        EvidenceState::Unknown => {
            unknowns.push("evidence_unknown".into());
            safe_actions.push("rerun_affected_checks".into());
            required_checks.push("verification".into());
        }
        EvidenceState::Complete => {}
    }
    if input.untrusted_material {
        unknowns.push("repository_material_untrusted".into());
        safe_actions.push("treat_material_as_data".into());
        required_checks.push("input_trust".into());
    }

    blockers.sort();
    blockers.dedup();
    unknowns.sort();
    unknowns.dedup();
    safe_actions.sort();
    safe_actions.dedup();
    required_checks.sort();
    required_checks.dedup();

    let state = if !blockers.is_empty() {
        DecisionState::Red
    } else if !unknowns.is_empty() {
        DecisionState::Yellow
    } else {
        DecisionState::Green
    };
    let outcome_state = input.outcome_state_override.unwrap_or_else(|| match state {
        DecisionState::Green => "ready".into(),
        DecisionState::Yellow
            if unknowns.iter().any(|unknown| {
                matches!(
                    unknown.as_str(),
                    "destructive_change_without_authority"
                        | "human_authority_missing"
                        | "coverage_weakening"
                )
            }) =>
        {
            "needs_human_decision".into()
        }
        DecisionState::Yellow => "verification_pending".into(),
        DecisionState::Red => "blocked".into(),
    });
    let authority = input.authority_override.unwrap_or_else(|| {
        match input.authority {
            AuthorityState::Authorized => "authorized",
            AuthorityState::Missing => "missing",
            AuthorityState::NotEvaluated => "not_evaluated",
        }
        .into()
    });
    GovernanceDecision {
        state,
        blockers,
        unknowns,
        safe_actions,
        required_checks,
        authority,
        outcome_state,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Digest(String);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DigestError {
    #[error("digest must be sha256:<64 lowercase hexadecimal characters>")]
    InvalidFormat,
}

impl Digest {
    pub fn sha256_bytes(bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        Self(format!("sha256:{}", hex::encode(hasher.finalize())))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Digest {
    type Err = DigestError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let Some(hex_part) = value.strip_prefix("sha256:") else {
            return Err(DigestError::InvalidFormat);
        };
        if hex_part.len() != 64
            || !hex_part
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(DigestError::InvalidFormat);
        }
        Ok(Self(value.into()))
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl TryFrom<String> for Digest {
    type Error = DigestError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

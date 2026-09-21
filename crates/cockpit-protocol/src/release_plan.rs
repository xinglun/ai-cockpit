use cockpit_core::Digest;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const RELEASE_PLAN_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseMode {
    NormalRelease,
    HistoricalTagRecovery,
    PostReleaseAcceptance,
    CloseOnly,
    IndependentPublicAcceptance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseStage {
    Resolve,
    Preflight,
    Build,
    CandidateAcceptance,
    Publish,
    PublicAcceptance,
    Close,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReleaseEvidenceBinding {
    pub class: String,
    pub path: String,
    pub digest: Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReleaseRequest {
    pub schema_version: u32,
    pub repository_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_item_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_work_item_id: Option<String>,
    pub mode: ReleaseMode,
    pub head_revision: String,
    pub source_revision: String,
    pub base_revision: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_digest: Option<Digest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handoff_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reuse_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reuse_acceptance_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_contract_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_contract_digest: Option<Digest>,
    #[serde(default)]
    pub recovery_evidence: Vec<ReleaseEvidenceBinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_stage: Option<ReleaseStage>,
}

/// Raw workflow-dispatch input. Mode selection is resolved in Rust; shell and
/// workflow code only transport these fields and perform cheap syntax checks.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReleaseRequestInput {
    pub schema_version: u32,
    pub repository_id: String,
    pub event: String,
    pub head_revision: String,
    pub base_revision: String,
    pub source_revision: String,
    pub version: Option<String>,
    pub from_tag: Option<String>,
    pub to_tag: Option<String>,
    #[serde(default)]
    pub publish_existing_tag: bool,
    #[serde(default)]
    pub publish_candidate: bool,
    #[serde(default)]
    pub post_release_acceptance: bool,
    #[serde(default)]
    pub close_only: bool,
    pub work_item_id: Option<String>,
    pub source_work_item_id: Option<String>,
    pub contract_path: Option<String>,
    pub contract_digest: Option<Digest>,
    pub source_contract_path: Option<String>,
    pub source_contract_digest: Option<Digest>,
    #[serde(default)]
    pub recovery_evidence: Vec<ReleaseEvidenceBinding>,
    pub reuse_run_id: Option<String>,
    pub reuse_acceptance_run_id: Option<String>,
    pub handoff_run_id: Option<String>,
    #[serde(default)]
    pub requested_mode: Option<ReleaseMode>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReleaseStageTransition {
    pub from: ReleaseStage,
    pub to: ReleaseStage,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReleasePlan {
    pub schema_version: u32,
    pub plan_id: String,
    pub request: ReleaseRequest,
    pub allowed_stages: Vec<ReleaseStage>,
    pub transitions: Vec<ReleaseStageTransition>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReleasePlanEnvelope {
    pub schema_version: u32,
    pub plan: ReleasePlan,
    pub plan_digest: Digest,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ReleasePlanError {
    #[error("unsupported ReleasePlan schema version {0}")]
    UnsupportedSchema(u32),
    #[error("ReleasePlan field is missing or empty: {0}")]
    MissingField(&'static str),
    #[error("historical tag recovery requires archived recovery evidence")]
    MissingRecoveryEvidence,
    #[error("ReleasePlan field conflicts with mode: {0}")]
    ConflictingField(String),
    #[error("requested stage is not allowed by the selected release mode")]
    RequestedStageNotAllowed,
    #[error("ReleasePlan stage transition list is invalid")]
    InvalidTransitions,
    #[error("ReleasePlan digest does not match its contents")]
    PlanDigestMismatch,
    #[error("could not serialize ReleasePlan: {0}")]
    Serialization(String),
}

impl ReleaseRequest {
    fn validate(&self) -> Result<(), ReleasePlanError> {
        if self.schema_version != RELEASE_PLAN_SCHEMA_VERSION {
            return Err(ReleasePlanError::UnsupportedSchema(self.schema_version));
        }
        for (value, field) in [
            (&self.repository_id, "repository_id"),
            (&self.head_revision, "head_revision"),
            (&self.source_revision, "source_revision"),
            (&self.base_revision, "base_revision"),
        ] {
            if value.trim().is_empty() {
                return Err(ReleasePlanError::MissingField(field));
            }
        }
        if matches!(
            self.mode,
            ReleaseMode::NormalRelease
                | ReleaseMode::HistoricalTagRecovery
                | ReleaseMode::PostReleaseAcceptance
                | ReleaseMode::CloseOnly
        ) && self.work_item_id.as_deref().is_none_or(str::is_empty)
        {
            return Err(ReleasePlanError::MissingField("work_item_id"));
        }
        if matches!(self.mode, ReleaseMode::IndependentPublicAcceptance)
            && self.work_item_id.as_deref().is_none_or(str::is_empty)
            && self.handoff_run_id.as_deref().is_none_or(str::is_empty)
        {
            return Err(ReleasePlanError::MissingField(
                "work_item_id_or_handoff_run_id",
            ));
        }
        if self.source_contract_path.is_some() != self.source_contract_digest.is_some() {
            return Err(ReleasePlanError::ConflictingField(
                "source_contract_path and source_contract_digest must be supplied together".into(),
            ));
        }
        if self
            .source_work_item_id
            .as_deref()
            .is_some_and(str::is_empty)
        {
            return Err(ReleasePlanError::ConflictingField(
                "source_work_item_id must not be empty".into(),
            ));
        }
        if self.source_work_item_id.is_some() && self.source_contract_path.is_none() {
            return Err(ReleasePlanError::ConflictingField(
                "source_work_item_id requires source_contract_path".into(),
            ));
        }
        if self.contract_path.is_some() != self.contract_digest.is_some() {
            return Err(ReleasePlanError::ConflictingField(
                "contract_path and contract_digest must be supplied together".into(),
            ));
        }
        if self.reuse_run_id.as_deref().is_some_and(str::is_empty)
            || self
                .reuse_acceptance_run_id
                .as_deref()
                .is_some_and(str::is_empty)
        {
            return Err(ReleasePlanError::ConflictingField(
                "reuse run identities must not be empty".into(),
            ));
        }
        for evidence in &self.recovery_evidence {
            if evidence.class.trim().is_empty() {
                return Err(ReleasePlanError::MissingField("recovery_evidence.class"));
            }
            if evidence.path.trim().is_empty() {
                return Err(ReleasePlanError::MissingField("recovery_evidence.path"));
            }
        }

        match self.mode {
            ReleaseMode::NormalRelease => {
                require_version_and_tag(self)?;
                if !self.recovery_evidence.is_empty() {
                    return Err(ReleasePlanError::ConflictingField(
                        "normal_release cannot carry recovery_evidence".into(),
                    ));
                }
            }
            ReleaseMode::HistoricalTagRecovery => {
                require_version_and_tag(self)?;
                if self.recovery_evidence.is_empty() {
                    return Err(ReleasePlanError::MissingRecoveryEvidence);
                }
            }
            ReleaseMode::PostReleaseAcceptance | ReleaseMode::IndependentPublicAcceptance => {
                require_version_and_tag(self)?;
                if self.mode == ReleaseMode::PostReleaseAcceptance && self.reuse_run_id.is_none() {
                    return Err(ReleasePlanError::MissingField("reuse_run_id"));
                }
                if !self.recovery_evidence.is_empty() {
                    return Err(ReleasePlanError::ConflictingField(
                        "public acceptance modes cannot carry recovery_evidence".into(),
                    ));
                }
            }
            ReleaseMode::CloseOnly => {
                if self.reuse_acceptance_run_id.is_none() {
                    return Err(ReleasePlanError::MissingField("reuse_acceptance_run_id"));
                }
                if !self.recovery_evidence.is_empty() {
                    return Err(ReleasePlanError::ConflictingField(
                        "close_only cannot carry recovery_evidence".into(),
                    ));
                }
            }
        }
        Ok(())
    }
}

fn require_version_and_tag(request: &ReleaseRequest) -> Result<(), ReleasePlanError> {
    if request.version.as_deref().is_none_or(str::is_empty) {
        return Err(ReleasePlanError::MissingField("version"));
    }
    if request.tag.as_deref().is_none_or(str::is_empty) {
        return Err(ReleasePlanError::MissingField("tag"));
    }
    Ok(())
}

impl ReleasePlan {
    pub fn resolve_input(input: ReleaseRequestInput) -> Result<Self, ReleasePlanError> {
        if input.schema_version != RELEASE_PLAN_SCHEMA_VERSION {
            return Err(ReleasePlanError::UnsupportedSchema(input.schema_version));
        }
        if input.event != "workflow_dispatch" {
            return Err(ReleasePlanError::ConflictingField(
                "ReleasePlan input requires workflow_dispatch".into(),
            ));
        }
        if input.close_only && !input.post_release_acceptance {
            return Err(ReleasePlanError::ConflictingField(
                "close_only requires post_release_acceptance".into(),
            ));
        }
        if input.publish_existing_tag && input.post_release_acceptance {
            return Err(ReleasePlanError::ConflictingField(
                "publish_existing_tag and post_release_acceptance cannot both be true".into(),
            ));
        }
        if input.publish_existing_tag && input.publish_candidate {
            return Err(ReleasePlanError::ConflictingField(
                "publish_existing_tag and publish_candidate cannot both be true".into(),
            ));
        }
        let mode = input.requested_mode.unwrap_or_else(|| {
            if input.close_only {
                ReleaseMode::CloseOnly
            } else if input.post_release_acceptance {
                ReleaseMode::PostReleaseAcceptance
            } else if input.publish_existing_tag {
                ReleaseMode::HistoricalTagRecovery
            } else if input.publish_candidate {
                ReleaseMode::NormalRelease
            } else {
                ReleaseMode::IndependentPublicAcceptance
            }
        });
        if (input.close_only && mode != ReleaseMode::CloseOnly)
            || (input.post_release_acceptance
                && !input.close_only
                && mode != ReleaseMode::PostReleaseAcceptance)
            || (input.publish_existing_tag && mode != ReleaseMode::HistoricalTagRecovery)
            || (input.publish_candidate && mode != ReleaseMode::NormalRelease)
        {
            return Err(ReleasePlanError::ConflictingField(
                "requested_mode conflicts with workflow mode flags".into(),
            ));
        }
        let request = ReleaseRequest {
            schema_version: input.schema_version,
            repository_id: input.repository_id,
            work_item_id: input.work_item_id,
            source_work_item_id: input.source_work_item_id,
            mode,
            head_revision: input.head_revision,
            source_revision: input.source_revision,
            base_revision: input.base_revision,
            contract_path: input.contract_path,
            contract_digest: input.contract_digest,
            from_tag: input.from_tag,
            version: input.version.or_else(|| {
                input
                    .to_tag
                    .as_deref()
                    .map(|tag| tag.trim_start_matches('v').to_owned())
            }),
            tag: input.to_tag,
            handoff_run_id: input.handoff_run_id,
            reuse_run_id: input.reuse_run_id,
            reuse_acceptance_run_id: input.reuse_acceptance_run_id,
            source_contract_path: input.source_contract_path,
            source_contract_digest: input.source_contract_digest,
            recovery_evidence: input.recovery_evidence,
            requested_stage: None,
        };
        Self::resolve(request)
    }

    pub fn resolve(request: ReleaseRequest) -> Result<Self, ReleasePlanError> {
        request.validate()?;
        let allowed_stages = stages_for_mode(request.mode);
        if let Some(requested) = request.requested_stage
            && !allowed_stages.contains(&requested)
        {
            return Err(ReleasePlanError::RequestedStageNotAllowed);
        }
        let plan_id = digest_json(&request)?.to_string();
        let transitions = allowed_stages
            .windows(2)
            .map(|window| ReleaseStageTransition {
                from: window[0],
                to: window[1],
            })
            .collect();
        Ok(Self {
            schema_version: RELEASE_PLAN_SCHEMA_VERSION,
            plan_id,
            request,
            allowed_stages: allowed_stages.to_vec(),
            transitions,
        })
    }

    pub fn mode(&self) -> ReleaseMode {
        self.request.mode
    }

    pub fn allowed_stages(&self) -> &[ReleaseStage] {
        &self.allowed_stages
    }

    pub fn next_stage(&self, current: ReleaseStage) -> Option<ReleaseStage> {
        self.transitions
            .iter()
            .find(|transition| transition.from == current)
            .map(|transition| transition.to)
    }

    pub fn digest(&self) -> Result<Digest, ReleasePlanError> {
        digest_json(self)
    }

    fn validate_structure(&self) -> Result<(), ReleasePlanError> {
        if self.schema_version != RELEASE_PLAN_SCHEMA_VERSION {
            return Err(ReleasePlanError::UnsupportedSchema(self.schema_version));
        }
        self.request.validate()?;
        let expected = stages_for_mode(self.request.mode);
        if self.allowed_stages != expected {
            return Err(ReleasePlanError::InvalidTransitions);
        }
        let expected_transitions: Vec<_> = expected
            .windows(2)
            .map(|window| ReleaseStageTransition {
                from: window[0],
                to: window[1],
            })
            .collect();
        if self.transitions != expected_transitions {
            return Err(ReleasePlanError::InvalidTransitions);
        }
        if self.plan_id != digest_json(&self.request)?.to_string() {
            return Err(ReleasePlanError::InvalidTransitions);
        }
        Ok(())
    }
}

impl ReleasePlanEnvelope {
    pub fn new(plan: ReleasePlan) -> Result<Self, ReleasePlanError> {
        plan.validate_structure()?;
        let plan_digest = plan.digest()?;
        Ok(Self {
            schema_version: RELEASE_PLAN_SCHEMA_VERSION,
            plan,
            plan_digest,
        })
    }

    pub fn verify(&self) -> Result<(), ReleasePlanError> {
        if self.schema_version != RELEASE_PLAN_SCHEMA_VERSION {
            return Err(ReleasePlanError::UnsupportedSchema(self.schema_version));
        }
        if self.plan_digest != self.plan.digest()? {
            return Err(ReleasePlanError::PlanDigestMismatch);
        }
        self.plan.validate_structure()?;
        Ok(())
    }
}

fn stages_for_mode(mode: ReleaseMode) -> &'static [ReleaseStage] {
    match mode {
        ReleaseMode::NormalRelease => &[
            ReleaseStage::Resolve,
            ReleaseStage::Preflight,
            ReleaseStage::Build,
            ReleaseStage::CandidateAcceptance,
            ReleaseStage::Publish,
            ReleaseStage::PublicAcceptance,
            ReleaseStage::Close,
        ],
        ReleaseMode::HistoricalTagRecovery => &[
            ReleaseStage::Resolve,
            ReleaseStage::Preflight,
            ReleaseStage::Publish,
            ReleaseStage::PublicAcceptance,
            ReleaseStage::Close,
        ],
        ReleaseMode::PostReleaseAcceptance | ReleaseMode::IndependentPublicAcceptance => &[
            ReleaseStage::Resolve,
            ReleaseStage::Preflight,
            ReleaseStage::PublicAcceptance,
            ReleaseStage::Close,
        ],
        ReleaseMode::CloseOnly => &[
            ReleaseStage::Resolve,
            ReleaseStage::Preflight,
            ReleaseStage::Close,
        ],
    }
}

fn digest_json<T: Serialize>(value: &T) -> Result<Digest, ReleasePlanError> {
    serde_json::to_vec(value)
        .map(|bytes| Digest::sha256_bytes(&bytes))
        .map_err(|error| ReleasePlanError::Serialization(error.to_string()))
}

use cockpit_core::{DecisionState, Digest};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::{path::PathBuf, str::FromStr};
use thiserror::Error;

pub const PROTOCOL_VERSION: u32 = 1;
pub const AGENT_INTERFACE_VERSION: u32 = 1;
pub const REPOSITORY_SCHEMA_VERSION: u32 = 2;

/// The repository schema migration graph is intentionally explicit.  A
/// Runtime may only apply one adjacent edge at a time; adding a future schema
/// requires adding another reviewed edge instead of changing a direct
/// `from -> latest` conversion.
pub const REPOSITORY_SCHEMA_MIGRATIONS: &[(u32, u32)] = &[(1, 2)];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaMigrationStep {
    pub from_schema: u32,
    pub to_schema: u32,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SchemaMigrationError {
    #[error("repository schema {0} is already current")]
    AlreadyCurrent(u32),
    #[error("repository schema {0} is newer than the Runtime target {1}")]
    FutureSchema(u32, u32),
    #[error("no reviewed adjacent migration from repository schema {0}")]
    MissingStep(u32),
}

/// Resolve the reviewed adjacent migration chain.  The function refuses a
/// future schema and never returns a step that skips an intermediate schema.
pub fn repository_schema_migration_chain(
    from_schema: u32,
    target_schema: u32,
) -> Result<Vec<SchemaMigrationStep>, SchemaMigrationError> {
    if from_schema == target_schema {
        return Err(SchemaMigrationError::AlreadyCurrent(from_schema));
    }
    if from_schema > target_schema {
        return Err(SchemaMigrationError::FutureSchema(
            from_schema,
            target_schema,
        ));
    }
    let mut current = from_schema;
    let mut chain = Vec::new();
    while current < target_schema {
        let Some((from, to)) = REPOSITORY_SCHEMA_MIGRATIONS
            .iter()
            .copied()
            .find(|(from, _)| *from == current)
        else {
            return Err(SchemaMigrationError::MissingStep(current));
        };
        if to > target_schema || to <= from {
            return Err(SchemaMigrationError::MissingStep(current));
        }
        chain.push(SchemaMigrationStep {
            from_schema: from,
            to_schema: to,
        });
        current = to;
    }
    Ok(chain)
}

pub fn default_repository_schema_version() -> u32 {
    1
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepositoryConfig {
    #[serde(rename = "protocol_version")]
    pub protocol_version: u32,
    #[serde(default = "default_repository_schema_version")]
    pub repository_schema_version: u32,
    pub repository_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeContext {
    pub runtime_version: String,
    pub protocol_version: u32,
    pub runtime_digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryContext {
    pub root: PathBuf,
    pub git_root: PathBuf,
    pub config: RepositoryConfig,
}

/// Repository-local discovery facts written by `attach`.
///
/// This manifest deliberately contains no provider prompt, command template,
/// authorization, or global configuration. It only lets an explicitly
/// attached repository advertise the shared Runtime's stable identity and
/// capabilities to an adapter that is already pointed at this directory.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentInterfaceManifest {
    pub schema_version: u32,
    pub protocol_version: u32,
    #[serde(default = "default_repository_schema_version")]
    pub repository_schema_version: u32,
    pub interface_version: u32,
    pub repository_id: String,
    pub root_binding: AgentRootBinding,
    pub capabilities: Vec<String>,
    pub interfaces: AgentInterfaces,
    pub adapter: AgentAdapterCompatibility,
    pub adapter_state: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentRootBinding {
    #[serde(rename = "type")]
    pub binding_type: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentInterfaces {
    pub cli: AgentInterfaceAvailability,
    pub mcp: AgentInterfaceAvailability,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentInterfaceAvailability {
    pub available: bool,
    #[serde(default)]
    pub transport: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentAdapterCompatibility {
    pub required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentProvider {
    GenericAgentsMd,
    Codex,
    Claude,
    Gemini,
    Cursor,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManagedAdapterRecord {
    pub provider: AgentProvider,
    pub adapter_version: u32,
    pub target: String,
    pub mode: String,
    pub repository_id: String,
    pub installed_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentDoctorCheck {
    pub state: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentDoctorAdapter {
    pub provider: AgentProvider,
    pub state: String,
    pub target: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentDoctorInterfaces {
    pub cli: String,
    pub mcp: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentDoctorReport {
    pub schema_version: u32,
    pub state: String,
    pub repository_id: Option<String>,
    pub attachment: AgentDoctorCheck,
    pub manifest: AgentDoctorCheck,
    pub adapters: Vec<AgentDoctorAdapter>,
    pub interfaces: AgentDoctorInterfaces,
    pub problems: Vec<String>,
    pub safe_actions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectProfile {
    pub profile_version: u64,
    pub repository_id: String,
    pub tests: Vec<QualityCommand>,
    pub build_systems: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct QualityCommand {
    pub program: String,
    pub args: Vec<String>,
    pub state: String,
}

/// Verification strength is deliberately separate from evidence provenance.
/// A T3 requirement asks for authoritative verification; it does not decide
/// whether the resulting evidence is provider- or enterprise-verified.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationTier {
    #[serde(rename = "T0")]
    T0,
    #[serde(rename = "T1")]
    T1,
    #[serde(rename = "T2")]
    T2,
    #[serde(rename = "T3")]
    T3,
}

impl VerificationTier {
    pub const fn rank(self) -> u8 {
        match self {
            Self::T0 => 0,
            Self::T1 => 1,
            Self::T2 => 2,
            Self::T3 => 3,
        }
    }
}

/// The execution stage is a governance boundary, not an arbitrary display
/// label.  `pre_ci` is local pre-CI feedback; `pr`, `merge`, and `release`
/// remain independent provider or protected-gate stages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStage {
    Task,
    #[serde(rename = "pre_ci")]
    PreCi,
    #[serde(rename = "pr")]
    PullRequest,
    Merge,
    Release,
}

impl VerificationStage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::PreCi => "pre_ci",
            Self::PullRequest => "pr",
            Self::Merge => "merge",
            Self::Release => "release",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "task" => Ok(Self::Task),
            "pre_ci" => Ok(Self::PreCi),
            "pr" => Ok(Self::PullRequest),
            "merge" => Ok(Self::Merge),
            "release" => Ok(Self::Release),
            other => Err(format!("unsupported verification stage: {other}")),
        }
    }

    pub const fn requires_base_revision(self) -> bool {
        !matches!(self, Self::Task)
    }
}

/// The assurance source for an authority or delegated evidence claim. These
/// labels describe provenance, not an automatic approval.  The wire spelling
/// remains compatible with the pre-existing `AssuranceLevel` vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceAssurance {
    SelfDeclared,
    RepositoryVerified,
    ProviderVerified,
    EnterpriseVerified,
}

/// Compatibility alias retained for existing protocol consumers.  New code
/// should use `EvidenceAssurance` to make the orthogonal dimension explicit.
pub use EvidenceAssurance as AssuranceLevel;

impl EvidenceAssurance {
    pub const fn rank(self) -> u8 {
        match self {
            Self::SelfDeclared => 0,
            Self::RepositoryVerified => 1,
            Self::ProviderVerified => 2,
            Self::EnterpriseVerified => 3,
        }
    }
}

pub const VERIFICATION_SEMANTICS_SCHEMA_VERSION: u32 = 1;

/// A requirement is policy-traceable input to a future planner.  Tier and
/// assurance are independent: every pair is representable, and an unmet
/// assurance requirement is a yellow/unknown governance result rather than a
/// silent upgrade of the evidence provenance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VerificationRequirement {
    pub schema_version: u32,
    pub required_tier: VerificationTier,
    pub required_assurance: EvidenceAssurance,
    pub policy_refs: Vec<String>,
    pub stage_refs: Vec<String>,
    pub gate_refs: Vec<String>,
    pub reason: String,
}

impl VerificationRequirement {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != VERIFICATION_SEMANTICS_SCHEMA_VERSION {
            return Err(format!(
                "unsupported verification semantics schema {}",
                self.schema_version
            ));
        }
        if self.reason.trim().is_empty() {
            return Err("verification requirement reason must not be empty".into());
        }
        let mut refs = std::collections::BTreeSet::new();
        for reference in self
            .policy_refs
            .iter()
            .chain(self.stage_refs.iter())
            .chain(self.gate_refs.iter())
        {
            if reference.trim().is_empty() {
                return Err("verification requirement references must not be empty".into());
            }
            if !refs.insert(reference) {
                return Err(format!(
                    "duplicate verification requirement reference {reference}"
                ));
            }
        }
        if refs.is_empty() {
            return Err(
                "verification requirement must name a policy stage or gate reference".into(),
            );
        }
        Ok(())
    }

    pub const fn is_satisfied_by(
        &self,
        actual_tier: VerificationTier,
        actual_assurance: EvidenceAssurance,
    ) -> bool {
        actual_tier.rank() >= self.required_tier.rank()
            && actual_assurance.rank() >= self.required_assurance.rank()
    }

    pub const fn is_at_least_as_strict_as(&self, parent: &Self) -> bool {
        self.required_tier.rank() >= parent.required_tier.rank()
            && self.required_assurance.rank() >= parent.required_assurance.rank()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthorityEvidence {
    pub assurance: AssuranceLevel,
    pub actor: String,
    pub authority_source: String,
    pub operations: Vec<String>,
    pub policy_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuthorityEvidenceError {
    #[error("authority actor must be explicit")]
    MissingActor,
    #[error("authority source must be explicit")]
    MissingSource,
    #[error("provider or enterprise assurance requires evidence references")]
    MissingEvidence,
    #[error("enterprise assurance requires policy references")]
    MissingPolicy,
    #[error("authority evidence must name at least one operation")]
    MissingOperation,
}

/// Validate provenance metadata without treating a self-declared claim as an
/// externally verified approval. Higher assurance levels add binding
/// requirements; they do not by themselves authorize an operation.
pub fn validate_authority_evidence(
    evidence: &AuthorityEvidence,
) -> Result<(), AuthorityEvidenceError> {
    if evidence.actor.trim().is_empty() {
        return Err(AuthorityEvidenceError::MissingActor);
    }
    if evidence.authority_source.trim().is_empty() {
        return Err(AuthorityEvidenceError::MissingSource);
    }
    if evidence.operations.is_empty() {
        return Err(AuthorityEvidenceError::MissingOperation);
    }
    if matches!(
        evidence.assurance,
        AssuranceLevel::ProviderVerified | AssuranceLevel::EnterpriseVerified
    ) && evidence.evidence_refs.is_empty()
    {
        return Err(AuthorityEvidenceError::MissingEvidence);
    }
    if matches!(evidence.assurance, AssuranceLevel::EnterpriseVerified)
        && evidence.policy_refs.is_empty()
    {
        return Err(AuthorityEvidenceError::MissingPolicy);
    }
    Ok(())
}

/// A structured decision keeps responsibility and recovery conditions visible
/// without imposing a fixed number of approvers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HumanDecision {
    pub decision: String,
    pub actor: String,
    pub authority_source: String,
    pub reason: String,
    pub evidence_refs: Vec<String>,
    pub policy_refs: Vec<String>,
    pub decided_at: String,
    pub resume_condition: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyLayer {
    Organization,
    Project,
    WorkItem,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalMode {
    NoHumanApprovalForLowRisk,
    SingleAuthorizedHuman,
    MultiPartyApproval,
    ExternalProviderApproval,
}

impl ApprovalMode {
    fn strength(&self) -> u8 {
        match self {
            Self::NoHumanApprovalForLowRisk => 0,
            Self::SingleAuthorizedHuman => 1,
            Self::MultiPartyApproval | Self::ExternalProviderApproval => 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PolicyRule {
    pub operation: String,
    pub approval_mode: ApprovalMode,
    pub required_evidence: Vec<String>,
    #[serde(default)]
    pub verification_requirement: Option<VerificationRequirement>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GovernancePolicy {
    pub policy_id: String,
    pub layer: PolicyLayer,
    pub rules: Vec<PolicyRule>,
}

/// Repository-owned policy input. A missing file means that the repository has
/// not opted into an organization/project policy yet; an existing file is
/// parsed strictly and cannot contain adapter instructions or unknown policy
/// fields.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GovernancePolicyDocument {
    pub schema_version: u32,
    #[serde(default)]
    pub organization: Option<GovernancePolicy>,
    #[serde(default)]
    pub project: Option<GovernancePolicy>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PolicyError {
    #[error("lower policy weakens {operation}: {field}")]
    Weakening { operation: String, field: String },
    #[error("policy layer is out of order: expected {expected}, got {actual}")]
    InvalidLayer { expected: String, actual: String },
}

fn policy_layer_name(layer: &PolicyLayer) -> &'static str {
    match layer {
        PolicyLayer::Organization => "organization",
        PolicyLayer::Project => "project",
        PolicyLayer::WorkItem => "work_item",
    }
}

fn policy_layers_in_order(layers: &[&GovernancePolicy]) -> bool {
    layers.windows(2).all(|pair| {
        let rank = |layer: &PolicyLayer| match layer {
            PolicyLayer::Organization => 0,
            PolicyLayer::Project => 1,
            PolicyLayer::WorkItem => 2,
        };
        rank(&pair[0].layer) <= rank(&pair[1].layer)
    })
}

/// Validate an overlay without forcing every lower layer to repeat inherited
/// rules. An explicitly weaker rule is rejected; omission leaves the parent
/// rule in force.
pub fn validate_policy_overlay(
    parent: &GovernancePolicy,
    child: &GovernancePolicy,
) -> Result<(), PolicyError> {
    for parent_rule in &parent.rules {
        let Some(child_rule) = child
            .rules
            .iter()
            .find(|rule| rule.operation == parent_rule.operation)
        else {
            continue;
        };
        if child_rule.approval_mode.strength() < parent_rule.approval_mode.strength() {
            return Err(PolicyError::Weakening {
                operation: parent_rule.operation.clone(),
                field: "approvalMode".into(),
            });
        }
        let parent_evidence = parent_rule
            .required_evidence
            .iter()
            .collect::<std::collections::BTreeSet<_>>();
        let child_evidence = child_rule
            .required_evidence
            .iter()
            .collect::<std::collections::BTreeSet<_>>();
        if !parent_evidence.is_subset(&child_evidence) {
            return Err(PolicyError::Weakening {
                operation: parent_rule.operation.clone(),
                field: "requiredEvidence".into(),
            });
        }
        if let (Some(parent_requirement), Some(child_requirement)) = (
            &parent_rule.verification_requirement,
            &child_rule.verification_requirement,
        ) && !child_requirement.is_at_least_as_strict_as(parent_requirement)
        {
            return Err(PolicyError::Weakening {
                operation: parent_rule.operation.clone(),
                field: "verificationRequirement".into(),
            });
        }
    }
    Ok(())
}

/// Merge organization, project, and optional Work Item policies into one
/// effective policy. Child rules may add requirements or strengthen approval,
/// but they may never weaken a parent rule. Rule order is deterministic.
pub fn merge_policy_layers(layers: &[&GovernancePolicy]) -> Result<GovernancePolicy, PolicyError> {
    if !policy_layers_in_order(layers)
        && let Some(policy) = layers.first()
    {
        return Err(PolicyError::InvalidLayer {
            expected: "ascending organization/project/work_item".into(),
            actual: policy_layer_name(&policy.layer).into(),
        });
    }
    let mut effective = GovernancePolicy {
        policy_id: "effective:none".into(),
        layer: PolicyLayer::Organization,
        rules: Vec::new(),
    };
    let mut ids = Vec::new();
    for (index, policy) in layers.iter().enumerate() {
        let expected = match index {
            0 => None,
            _ => Some(match policy.layer {
                PolicyLayer::Organization => "project or work_item",
                PolicyLayer::Project => "work_item",
                PolicyLayer::WorkItem => "no lower layer",
            }),
        };
        if let Some(expected) = expected {
            let previous = layers[index - 1].layer.clone();
            let valid = match previous {
                PolicyLayer::Organization => {
                    matches!(
                        policy.layer,
                        PolicyLayer::Organization | PolicyLayer::Project
                    )
                }
                PolicyLayer::Project => {
                    matches!(policy.layer, PolicyLayer::Project | PolicyLayer::WorkItem)
                }
                PolicyLayer::WorkItem => matches!(policy.layer, PolicyLayer::WorkItem),
            };
            if !valid {
                return Err(PolicyError::InvalidLayer {
                    expected: expected.into(),
                    actual: policy_layer_name(&policy.layer).into(),
                });
            }
        }
        if index > 0 {
            validate_policy_overlay(&effective, policy)?;
        }
        for rule in &policy.rules {
            if let Some(existing) = effective
                .rules
                .iter_mut()
                .find(|existing| existing.operation == rule.operation)
            {
                let mut evidence = existing.required_evidence.clone();
                for item in &rule.required_evidence {
                    if !evidence.contains(item) {
                        evidence.push(item.clone());
                    }
                }
                existing.approval_mode = rule.approval_mode.clone();
                existing.required_evidence = evidence;
                if let Some(child_requirement) = &rule.verification_requirement {
                    existing.verification_requirement = Some(
                        existing
                            .verification_requirement
                            .as_ref()
                            .map(|parent_requirement| {
                                merge_verification_requirements(
                                    parent_requirement,
                                    child_requirement,
                                )
                            })
                            .unwrap_or_else(|| child_requirement.clone()),
                    );
                }
            } else {
                effective.rules.push(rule.clone());
            }
        }
        effective.layer = policy.layer.clone();
        ids.push(policy.policy_id.clone());
    }
    if !ids.is_empty() {
        effective.policy_id = format!("effective:{}", ids.join(":"));
    }
    Ok(effective)
}

fn merge_verification_requirements(
    parent: &VerificationRequirement,
    child: &VerificationRequirement,
) -> VerificationRequirement {
    let mut merged = parent.clone();
    if child.required_tier.rank() > merged.required_tier.rank() {
        merged.required_tier = child.required_tier;
    }
    if child.required_assurance.rank() > merged.required_assurance.rank() {
        merged.required_assurance = child.required_assurance;
    }
    for reference in &child.policy_refs {
        if !merged.policy_refs.contains(reference) {
            merged.policy_refs.push(reference.clone());
        }
    }
    for reference in &child.stage_refs {
        if !merged.stage_refs.contains(reference) {
            merged.stage_refs.push(reference.clone());
        }
    }
    for reference in &child.gate_refs {
        if !merged.gate_refs.contains(reference) {
            merged.gate_refs.push(reference.clone());
        }
    }
    merged.reason = format!("{}; {}", parent.reason, child.reason);
    merged
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
    SecretProhibited,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidencePersistence {
    FullCapture,
    RedactedCapture,
    DigestOnly,
    NoPersistence,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceRetention {
    pub classification: DataClassification,
    pub persistence: EvidencePersistence,
    pub retention_days: Option<u64>,
    pub expires_at: Option<String>,
    pub disposal_action: String,
}

/// Repository/work-item binding for sensitive evidence handling.  The policy
/// is metadata only: it does not manufacture approval or external assurance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceRetentionPolicy {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub retention: EvidenceRetention,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceDisposition {
    Retain,
    Expired,
    PurgePlanned,
    Purged,
    Blocked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceDispositionItem {
    pub path: String,
    pub digest: Digest,
    pub classification: DataClassification,
    pub persistence: EvidencePersistence,
    pub disposition: EvidenceDisposition,
    pub reason: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EvidenceRetentionError {
    #[error("secret_prohibited evidence cannot use full_capture or redacted_capture")]
    SecretCaptureForbidden,
    #[error("disposal_action must be explicit")]
    MissingDisposalAction,
    #[error("retention policy must declare retention_days or expires_at")]
    MissingExpiry,
    #[error("retention_days and expires_at cannot both be set")]
    ConflictingExpiry,
}

pub fn validate_evidence_retention(
    retention: &EvidenceRetention,
) -> Result<(), EvidenceRetentionError> {
    if retention.disposal_action.trim().is_empty() {
        return Err(EvidenceRetentionError::MissingDisposalAction);
    }
    if retention.retention_days.is_none() && retention.expires_at.is_none() {
        return Err(EvidenceRetentionError::MissingExpiry);
    }
    if retention.retention_days.is_some() && retention.expires_at.is_some() {
        return Err(EvidenceRetentionError::ConflictingExpiry);
    }
    if matches!(
        retention.classification,
        DataClassification::SecretProhibited
    ) && matches!(
        retention.persistence,
        EvidencePersistence::FullCapture | EvidencePersistence::RedactedCapture
    ) {
        return Err(EvidenceRetentionError::SecretCaptureForbidden);
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceValidity {
    Valid,
    Expired,
    Revoked,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DelegatedEvidence {
    pub provider: String,
    pub subject: String,
    pub origin: String,
    pub assurance: AssuranceLevel,
    pub collected_at: String,
    pub digest: Digest,
    pub validity: EvidenceValidity,
    pub raw_evidence_ref: String,
}

/// A Runtime binding receipt for provider-produced evidence. The Runtime
/// records the binding and digest; it does not assert that the provider's
/// underlying claim is true.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DelegatedEvidenceReceipt {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub evidence: DelegatedEvidence,
    pub runtime_version: String,
    pub runtime_digest: Digest,
    pub bound_at: String,
}

/// The result of the resource-finalization boundary that follows a reviewed
/// PR merge.  This protocol is deliberately separate from the Work Item
/// lifecycle: `closed` is not evidence that an external branch or worktree
/// was deleted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceFinalizationDisposition {
    Deleted,
    Retained,
    Blocked,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceFinalizationPullRequestState {
    Merged,
    Unmerged,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceFinalizationBranchState {
    Present,
    Deleted,
    Protected,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceFinalizationWorktreeState {
    Clean,
    Dirty,
    Removed,
    Unknown,
}

/// Provider-side identity needed to prove that a finalization result belongs
/// to the reviewed pull request and immutable base revision.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceFinalizationPullRequestIdentity {
    pub number: u64,
    pub url: String,
    pub head_revision: String,
    pub base_branch: String,
    pub base_remote: String,
    pub base_revision: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_commit: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceFinalizationBranchIdentity {
    pub name: String,
    pub remote: String,
    pub head_revision: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceFinalizationWorktreeIdentity {
    pub worktree_id: String,
    pub path: String,
    pub branch: String,
    pub head_revision: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceFinalizationState {
    pub pull_request: ResourceFinalizationPullRequestState,
    pub branch: ResourceFinalizationBranchState,
    pub worktree: ResourceFinalizationWorktreeState,
}

/// Optional context copied from a Work Item Contract.  It is optional so a
/// Runtime can read historical receipts, but when present it is complete and
/// must agree with the receipt's provider/PR/branch/worktree identities.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceFinalizationContext {
    pub branch: String,
    pub worktree: String,
    pub base_branch: String,
    pub base_remote: String,
    pub provider: String,
    pub pull_request: String,
}

impl ResourceFinalizationContext {
    /// `start` records local branch/worktree facts before a provider-side
    /// finalization plan exists.  The remaining identity fields are explicit
    /// sentinels until `finalize-plan` binds them to the reviewed resource.
    pub fn is_provisional(&self) -> bool {
        [
            self.base_branch.as_str(),
            self.base_remote.as_str(),
            self.provider.as_str(),
            self.pull_request.as_str(),
        ]
        .contains(&"unknown")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceFinalizationResult {
    pub disposition: ResourceFinalizationDisposition,
    #[serde(default)]
    pub failure_codes: Vec<String>,
    #[serde(default)]
    pub unknown_codes: Vec<String>,
}

/// Identity-bound receipt for post-merge branch/worktree finalization.  It is
/// a pure protocol record: it does not perform provider calls or filesystem
/// deletion.  Consumers must validate it before treating a Work Item as
/// resource-finalized.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceFinalizationReceipt {
    pub schema_version: u32,
    pub receipt_id: String,
    pub operation_id: String,
    pub repository_id: String,
    pub work_item_id: String,
    pub runtime_version: String,
    pub runtime_digest: Digest,
    pub provider: String,
    pub pull_request: ResourceFinalizationPullRequestIdentity,
    pub branch: ResourceFinalizationBranchIdentity,
    pub worktree: ResourceFinalizationWorktreeIdentity,
    pub before: ResourceFinalizationState,
    pub after: ResourceFinalizationState,
    pub result: ResourceFinalizationResult,
    pub actor: String,
    pub authority_source: String,
    pub reason: String,
    pub timestamp: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_digest: Option<Digest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_context: Option<ResourceFinalizationContext>,
}

/// Append-only successor to an immutable canonical resource-finalization
/// receipt. The predecessor digest and sequence make stale or forked writes
/// detectable without rewriting historical evidence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceFinalizationTransitionReceipt {
    pub schema_version: u32,
    pub transition_id: String,
    pub sequence: u64,
    pub predecessor_receipt_digest: Digest,
    pub receipt: ResourceFinalizationReceipt,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ResourceFinalizationError {
    #[error("resource finalization schema version must be 1")]
    UnsupportedSchema,
    #[error("resource finalization field is empty: {0}")]
    EmptyField(&'static str),
    #[error("resource finalization digest is invalid: {0}")]
    InvalidDigest(&'static str),
    #[error("resource finalization code is invalid: {0}")]
    InvalidCode(String),
    #[error("resource finalization identity mismatch: {0}")]
    IdentityMismatch(&'static str),
    #[error("resource finalization state is invalid: {0}")]
    InvalidState(&'static str),
    #[error("resource finalization disposition is invalid: {0}")]
    InvalidDisposition(&'static str),
    #[error("resource finalization replay mismatch: {0}")]
    ReplayMismatch(&'static str),
}

pub const RESOURCE_FINALIZATION_SCHEMA_VERSION: u32 = 1;

pub const RESOURCE_FINALIZATION_CODE_DIRTY_WORKTREE: &str = "dirty_worktree";
pub const RESOURCE_FINALIZATION_CODE_UNMERGED_PULL_REQUEST: &str = "unmerged_pull_request";
pub const RESOURCE_FINALIZATION_CODE_PROTECTED_BRANCH: &str = "protected_branch";
pub const RESOURCE_FINALIZATION_CODE_AMBIGUOUS_STATE: &str = "ambiguous_state";
pub const RESOURCE_FINALIZATION_CODE_PROVIDER_UNAVAILABLE: &str = "provider_unavailable";
pub const RESOURCE_FINALIZATION_CODE_PARTIAL_CLEANUP: &str = "partial_cleanup";

fn validate_resource_finalization_text(
    value: &str,
    field: &'static str,
) -> Result<(), ResourceFinalizationError> {
    if value.trim().is_empty() {
        Err(ResourceFinalizationError::EmptyField(field))
    } else {
        Ok(())
    }
}

fn validate_resource_finalization_revision(
    value: &str,
    field: &'static str,
) -> Result<(), ResourceFinalizationError> {
    validate_resource_finalization_text(value, field)
}

fn validate_resource_finalization_code(code: &str) -> Result<(), ResourceFinalizationError> {
    if code.is_empty()
        || !code.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
        })
    {
        return Err(ResourceFinalizationError::InvalidCode(code.into()));
    }
    Ok(())
}

fn validate_resource_finalization_codes(codes: &[String]) -> Result<(), ResourceFinalizationError> {
    for code in codes {
        validate_resource_finalization_code(code)?;
    }
    Ok(())
}

fn validate_resource_finalization_state(
    state: &ResourceFinalizationState,
) -> Result<(), ResourceFinalizationError> {
    if matches!(
        state.pull_request,
        ResourceFinalizationPullRequestState::Unknown
    ) && !matches!(state.branch, ResourceFinalizationBranchState::Unknown)
        && !matches!(state.worktree, ResourceFinalizationWorktreeState::Unknown)
    {
        return Err(ResourceFinalizationError::InvalidState(
            "unknown pull request state must carry an unknown branch or worktree state",
        ));
    }
    Ok(())
}

fn validate_resource_finalization_identity(
    receipt: &ResourceFinalizationReceipt,
) -> Result<(), ResourceFinalizationError> {
    for (value, field) in [
        (receipt.receipt_id.as_str(), "receiptId"),
        (receipt.operation_id.as_str(), "operationId"),
        (receipt.repository_id.as_str(), "repositoryId"),
        (receipt.work_item_id.as_str(), "workItemId"),
        (receipt.runtime_version.as_str(), "runtimeVersion"),
        (receipt.provider.as_str(), "provider"),
        (receipt.actor.as_str(), "actor"),
        (receipt.authority_source.as_str(), "authoritySource"),
        (receipt.reason.as_str(), "reason"),
        (receipt.timestamp.as_str(), "timestamp"),
    ] {
        validate_resource_finalization_text(value, field)?;
    }
    if Digest::from_str(receipt.runtime_digest.as_str()).is_err() {
        return Err(ResourceFinalizationError::InvalidDigest("runtimeDigest"));
    }
    if let Some(contract_digest) = &receipt.contract_digest
        && Digest::from_str(contract_digest.as_str()).is_err()
    {
        return Err(ResourceFinalizationError::InvalidDigest("contractDigest"));
    }
    if receipt.pull_request.number == 0 {
        return Err(ResourceFinalizationError::InvalidState(
            "pull request number must be positive",
        ));
    }
    for (value, field) in [
        (receipt.pull_request.url.as_str(), "pullRequest.url"),
        (
            receipt.pull_request.head_revision.as_str(),
            "pullRequest.headRevision",
        ),
        (
            receipt.pull_request.base_branch.as_str(),
            "pullRequest.baseBranch",
        ),
        (
            receipt.pull_request.base_remote.as_str(),
            "pullRequest.baseRemote",
        ),
        (
            receipt.pull_request.base_revision.as_str(),
            "pullRequest.baseRevision",
        ),
        (receipt.branch.name.as_str(), "branch.name"),
        (receipt.branch.remote.as_str(), "branch.remote"),
        (receipt.branch.head_revision.as_str(), "branch.headRevision"),
        (receipt.worktree.worktree_id.as_str(), "worktree.worktreeId"),
        (receipt.worktree.path.as_str(), "worktree.path"),
        (receipt.worktree.branch.as_str(), "worktree.branch"),
        (
            receipt.worktree.head_revision.as_str(),
            "worktree.headRevision",
        ),
    ] {
        validate_resource_finalization_revision(value, field)?;
    }
    if let Some(merge_commit) = &receipt.pull_request.merge_commit {
        validate_resource_finalization_revision(merge_commit, "pullRequest.mergeCommit")?;
    }
    if receipt.pull_request.head_revision != receipt.branch.head_revision
        || receipt.branch.head_revision != receipt.worktree.head_revision
    {
        return Err(ResourceFinalizationError::IdentityMismatch(
            "pull request, branch, and worktree head revisions differ",
        ));
    }
    if receipt.branch.name != receipt.worktree.branch {
        return Err(ResourceFinalizationError::IdentityMismatch(
            "branch and worktree branch differ",
        ));
    }
    if let Some(context) = &receipt.resource_context {
        validate_resource_finalization_context(context)?;
        if context.branch != receipt.branch.name
            || context.worktree != receipt.worktree.path
            || context.base_branch != receipt.pull_request.base_branch
            || context.base_remote != receipt.pull_request.base_remote
            || context.provider != receipt.provider
            || context.pull_request != receipt.pull_request.url
        {
            return Err(ResourceFinalizationError::IdentityMismatch(
                "resource context does not match receipt identity",
            ));
        }
    }
    Ok(())
}

/// Validate an optional Contract resource context without requiring a
/// provider receipt.  Contract deserialization keeps this field optional for
/// historical compatibility; callers that declare it must provide all
/// non-empty identity values.
pub fn validate_resource_finalization_context(
    context: &ResourceFinalizationContext,
) -> Result<(), ResourceFinalizationError> {
    for (value, field) in [
        (context.branch.as_str(), "resourceContext.branch"),
        (context.worktree.as_str(), "resourceContext.worktree"),
        (context.base_branch.as_str(), "resourceContext.baseBranch"),
        (context.base_remote.as_str(), "resourceContext.baseRemote"),
        (context.provider.as_str(), "resourceContext.provider"),
        (context.pull_request.as_str(), "resourceContext.pullRequest"),
    ] {
        validate_resource_finalization_text(value, field)?;
    }
    Ok(())
}

/// Validate the intrinsic receipt shape and fail-closed disposition rules.
pub fn validate_resource_finalization_receipt(
    receipt: &ResourceFinalizationReceipt,
) -> Result<(), ResourceFinalizationError> {
    if receipt.schema_version != RESOURCE_FINALIZATION_SCHEMA_VERSION {
        return Err(ResourceFinalizationError::UnsupportedSchema);
    }
    validate_resource_finalization_identity(receipt)?;
    validate_resource_finalization_state(&receipt.before)?;
    validate_resource_finalization_state(&receipt.after)?;
    validate_resource_finalization_codes(&receipt.result.failure_codes)?;
    validate_resource_finalization_codes(&receipt.result.unknown_codes)?;

    if matches!(
        receipt.before.pull_request,
        ResourceFinalizationPullRequestState::Merged
    ) && receipt.pull_request.merge_commit.is_none()
    {
        return Err(ResourceFinalizationError::InvalidState(
            "merged pull request requires merge commit",
        ));
    }
    if matches!(
        receipt.before.pull_request,
        ResourceFinalizationPullRequestState::Unmerged
    ) && receipt.pull_request.merge_commit.is_some()
    {
        return Err(ResourceFinalizationError::InvalidState(
            "unmerged pull request cannot carry merge commit",
        ));
    }

    let fully_deleted = |state: &ResourceFinalizationState| {
        matches!(
            state.pull_request,
            ResourceFinalizationPullRequestState::Merged
        ) && matches!(state.branch, ResourceFinalizationBranchState::Deleted)
            && matches!(state.worktree, ResourceFinalizationWorktreeState::Removed)
    };
    match receipt.result.disposition {
        ResourceFinalizationDisposition::Deleted => {
            if !receipt.result.failure_codes.is_empty() || !receipt.result.unknown_codes.is_empty()
            {
                return Err(ResourceFinalizationError::InvalidDisposition(
                    "deleted result cannot carry failure or unknown codes",
                ));
            }
            let fresh_cleanup = matches!(
                receipt.before.pull_request,
                ResourceFinalizationPullRequestState::Merged
            ) && matches!(
                receipt.before.branch,
                ResourceFinalizationBranchState::Present
            ) && matches!(
                receipt.before.worktree,
                ResourceFinalizationWorktreeState::Clean
            );
            if !fully_deleted(&receipt.after) || (!fresh_cleanup && !fully_deleted(&receipt.before))
            {
                return Err(ResourceFinalizationError::InvalidDisposition(
                    "deleted result requires clean merged resources or an idempotent deleted replay",
                ));
            }
        }
        ResourceFinalizationDisposition::Retained => {
            if !matches!(
                receipt.before.pull_request,
                ResourceFinalizationPullRequestState::Merged
            ) || !matches!(
                receipt.after.pull_request,
                ResourceFinalizationPullRequestState::Merged
            ) {
                return Err(ResourceFinalizationError::InvalidDisposition(
                    "retained result requires a merged pull request",
                ));
            }
            if fully_deleted(&receipt.after) {
                return Err(ResourceFinalizationError::InvalidDisposition(
                    "retained result cannot claim complete deletion",
                ));
            }
        }
        ResourceFinalizationDisposition::Blocked => {
            if receipt.result.failure_codes.is_empty() && receipt.result.unknown_codes.is_empty() {
                return Err(ResourceFinalizationError::InvalidDisposition(
                    "blocked result requires failure or unknown codes",
                ));
            }
            if fully_deleted(&receipt.after) {
                return Err(ResourceFinalizationError::InvalidDisposition(
                    "blocked result cannot claim complete deletion",
                ));
            }
        }
        ResourceFinalizationDisposition::Unknown => {
            if receipt.result.unknown_codes.is_empty() {
                return Err(ResourceFinalizationError::InvalidDisposition(
                    "unknown result requires unknown codes",
                ));
            }
            if fully_deleted(&receipt.after) {
                return Err(ResourceFinalizationError::InvalidDisposition(
                    "unknown result cannot claim complete deletion",
                ));
            }
        }
    }
    Ok(())
}

/// Validate repository, Work Item, Contract, and optional ResourceContext
/// bindings supplied by the caller.  This is the hook Runtime finalization
/// code can use after `start` records context in a Contract.
pub fn validate_resource_finalization_receipt_for(
    receipt: &ResourceFinalizationReceipt,
    repository_id: &str,
    work_item_id: &str,
    contract_digest: Option<&Digest>,
    resource_context: Option<&ResourceFinalizationContext>,
) -> Result<(), ResourceFinalizationError> {
    validate_resource_finalization_receipt(receipt)?;
    validate_resource_finalization_text(repository_id, "expectedRepositoryId")?;
    validate_resource_finalization_text(work_item_id, "expectedWorkItemId")?;
    if receipt.repository_id != repository_id {
        return Err(ResourceFinalizationError::IdentityMismatch("repositoryId"));
    }
    if receipt.work_item_id != work_item_id {
        return Err(ResourceFinalizationError::IdentityMismatch("workItemId"));
    }
    if let Some(expected_digest) = contract_digest
        && receipt.contract_digest.as_ref() != Some(expected_digest)
    {
        return Err(ResourceFinalizationError::IdentityMismatch(
            "contractDigest",
        ));
    }
    if let Some(expected_context) = resource_context
        && receipt.resource_context.as_ref() != Some(expected_context)
    {
        return Err(ResourceFinalizationError::IdentityMismatch(
            "resourceContext",
        ));
    }
    Ok(())
}

/// A retry may safely replay an already completed finalization only when the
/// operation identity and final state are identical.  This helper compares
/// the immutable operation binding and intentionally ignores receipt ID and
/// timestamp, which are event metadata rather than mutation identity.
pub fn validate_resource_finalization_replay(
    original: &ResourceFinalizationReceipt,
    replay: &ResourceFinalizationReceipt,
) -> Result<(), ResourceFinalizationError> {
    validate_resource_finalization_receipt(original)?;
    validate_resource_finalization_receipt(replay)?;
    if original.operation_id != replay.operation_id {
        return Err(ResourceFinalizationError::ReplayMismatch("operationId"));
    }
    if original.repository_id != replay.repository_id {
        return Err(ResourceFinalizationError::ReplayMismatch("repositoryId"));
    }
    if original.work_item_id != replay.work_item_id {
        return Err(ResourceFinalizationError::ReplayMismatch("workItemId"));
    }
    if original.provider != replay.provider
        || original.pull_request != replay.pull_request
        || original.branch != replay.branch
        || original.worktree != replay.worktree
        || original.before != replay.before
        || original.after != replay.after
        || original.result != replay.result
        || original.runtime_version != replay.runtime_version
        || original.runtime_digest != replay.runtime_digest
        || original.contract_digest != replay.contract_digest
        || original.resource_context != replay.resource_context
    {
        return Err(ResourceFinalizationError::ReplayMismatch(
            "finalization identity or result",
        ));
    }
    Ok(())
}

pub fn validate_resource_finalization_transition(
    previous: &ResourceFinalizationReceipt,
    transition: &ResourceFinalizationTransitionReceipt,
    expected_sequence: u64,
) -> Result<(), ResourceFinalizationError> {
    let validate_member = |receipt: &ResourceFinalizationReceipt| {
        let merge_observation = matches!(
            receipt.result.disposition,
            ResourceFinalizationDisposition::Retained
        ) && matches!(
            receipt.before.pull_request,
            ResourceFinalizationPullRequestState::Unmerged
        ) && matches!(
            receipt.after.pull_request,
            ResourceFinalizationPullRequestState::Merged
        ) && receipt.before.branch == receipt.after.branch
            && receipt.before.worktree == receipt.after.worktree;
        if merge_observation {
            let mut normalized = receipt.clone();
            normalized.before.pull_request = ResourceFinalizationPullRequestState::Merged;
            validate_resource_finalization_receipt(&normalized)
        } else {
            validate_resource_finalization_receipt(receipt)
        }
    };
    validate_member(previous)?;
    let next = &transition.receipt;
    if previous.after != next.before {
        return Err(ResourceFinalizationError::InvalidState(
            "transition before state must equal predecessor after state",
        ));
    }
    // A merge observation retains the still-present clean resources. This is
    // valid only inside a linked transition; standalone retained receipts keep
    // the stricter merged-before rule.
    validate_member(next)?;
    if transition.schema_version != 1 {
        return Err(ResourceFinalizationError::UnsupportedSchema);
    }
    validate_resource_finalization_text(&transition.transition_id, "transitionId")?;
    if transition.sequence != expected_sequence {
        return Err(ResourceFinalizationError::ReplayMismatch(
            "transition sequence",
        ));
    }
    let previous_value = serde_json::to_value(previous)
        .map_err(|_| ResourceFinalizationError::ReplayMismatch("predecessor digest"))?;
    let previous_digest = digest_json(&previous_value)
        .map_err(|_| ResourceFinalizationError::ReplayMismatch("predecessor digest"))?;
    if transition.predecessor_receipt_digest != previous_digest {
        return Err(ResourceFinalizationError::ReplayMismatch(
            "predecessor digest",
        ));
    }
    if previous.repository_id != next.repository_id
        || previous.work_item_id != next.work_item_id
        || previous.provider != next.provider
        || previous.pull_request.number != next.pull_request.number
        || previous.pull_request.url != next.pull_request.url
        || previous.pull_request.head_revision != next.pull_request.head_revision
        || previous.pull_request.base_branch != next.pull_request.base_branch
        || previous.pull_request.base_remote != next.pull_request.base_remote
        || previous.pull_request.base_revision != next.pull_request.base_revision
        || previous.branch != next.branch
        || previous.worktree != next.worktree
        || previous.contract_digest != next.contract_digest
        || previous.resource_context != next.resource_context
    {
        return Err(ResourceFinalizationError::IdentityMismatch(
            "finalization transition identity",
        ));
    }
    if previous.pull_request.merge_commit.is_some()
        && previous.pull_request.merge_commit != next.pull_request.merge_commit
    {
        return Err(ResourceFinalizationError::InvalidState(
            "merge commit cannot change after it is observed",
        ));
    }
    if matches!(
        previous.after.pull_request,
        ResourceFinalizationPullRequestState::Merged
    ) && !matches!(
        next.before.pull_request,
        ResourceFinalizationPullRequestState::Merged
    ) {
        return Err(ResourceFinalizationError::InvalidState(
            "merged pull request cannot regress",
        ));
    }
    if matches!(
        previous.result.disposition,
        ResourceFinalizationDisposition::Deleted
    ) {
        return Err(ResourceFinalizationError::InvalidDisposition(
            "deleted finalization is terminal",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuditEvent {
    pub event_id: String,
    pub repository_id: String,
    pub work_item_id: Option<String>,
    pub runtime_version: String,
    pub runtime_digest: Digest,
    pub timestamp: String,
    pub event_type: String,
    pub digest: Digest,
    pub evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuditExportManifest {
    pub schema_version: u32,
    pub repository_id: String,
    pub runtime_version: String,
    pub runtime_digest: Digest,
    pub export_digest: Digest,
    pub external_retention_required: bool,
    pub events: Vec<AuditEvent>,
}

/// The reference Contract accepts both the legacy one-line intent and the
/// structured V2 intent.  Keeping the two representations in one tagged
/// value lets old repository bytes remain readable while preventing the
/// Runtime from silently discarding a structured declaration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContractIntent {
    Text(String),
    Structured(ContractIntentDetails),
}

impl Default for ContractIntent {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

impl From<String> for ContractIntent {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for ContractIntent {
    fn from(value: &str) -> Self {
        Self::Text(value.into())
    }
}

impl ContractIntent {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Text(value) => value.trim().is_empty(),
            Self::Structured(value) => value.is_empty(),
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value),
            Self::Structured(_) => None,
        }
    }

    pub fn structured(&self) -> Option<&ContractIntentDetails> {
        match self {
            Self::Text(_) => None,
            Self::Structured(value) => Some(value),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractIntentDetails {
    #[serde(default)]
    pub business_goal: Option<String>,
    #[serde(default)]
    pub user_goal: Option<String>,
    #[serde(default)]
    pub problem: Option<String>,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub non_goals: Vec<String>,
    #[serde(default)]
    pub rationale: Option<String>,
}

impl ContractIntentDetails {
    fn is_empty(&self) -> bool {
        self.business_goal
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
            && self
                .user_goal
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            && self
                .problem
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            && self.constraints.is_empty()
            && self.non_goals.is_empty()
            && self
                .rationale
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
    }
}

/// Source references may be legacy strings or explicit path/reason objects.
/// New V2 authors should use the structured representation; the legacy variant
/// is retained so archived protocol-v1 Contracts are not rewritten.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContractSource {
    Legacy(String),
    Structured(ContractSourceDetails),
}

impl From<String> for ContractSource {
    fn from(value: String) -> Self {
        Self::Legacy(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractSourceDetails {
    pub path: String,
    pub reason: String,
}

/// Verification declarations are descriptive inputs. They never authorize
/// skipping execution and can be represented as a legacy command/capability
/// string or a typed V2 check.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VerificationDeclaration {
    Legacy(String),
    Check(VerificationCheck),
}

impl From<String> for VerificationDeclaration {
    fn from(value: String) -> Self {
        Self::Legacy(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VerificationCheck {
    pub check: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractRiskAssessment {
    pub level: String,
    #[serde(default)]
    pub risk_types: Vec<String>,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractAgentCapability {
    pub can_implement: bool,
    pub can_verify: bool,
    pub needs_human_decision: bool,
    #[serde(default)]
    pub blocked_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractExecutionDecision {
    pub status: String,
    pub reason: String,
}

/// Repository-bound evidence that a human selected the bounded preflight
/// review option. This receipt authorizes only the workflow transition from
/// `needs_human_confirmation` to implementation; it never asserts that an
/// unverified scenario, test, or release requirement has passed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreflightDecisionEvidence {
    pub schema_version: u32,
    pub decision_id: String,
    pub decision: String,
    pub work_item_id: String,
    pub repository_id: String,
    pub contract_digest: Digest,
    pub preflight_decision_digest: Digest,
    pub repository_snapshot_digest: Digest,
    pub recorded_at: String,
    pub recorded_by: String,
    pub reason: String,
}

/// A file that was already dirty when a Work Item started.  The fingerprint
/// is intentionally an opaque digest string: the Contract records the
/// repository's observed baseline without claiming a particular Git hash
/// algorithm.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BaselineDirtyPath {
    pub path: String,
    pub status: String,
    pub fingerprint: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResumePredecessorClosure {
    pub status_closed: bool,
    pub pr_merged: bool,
    pub closure_succeeded: bool,
    pub local_branch_deleted: bool,
    pub remote_branch_deleted: bool,
    pub base_synchronized: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResumeHistoryEntry {
    pub resume_version: u32,
    pub from_base_commit: String,
    pub to_base_commit: String,
    pub base_remote: String,
    pub base_branch: String,
    pub work_branch: String,
    pub recorded_at: String,
    pub prior_contract_digest: Digest,
    pub predecessor_work_item_id: String,
    pub predecessor_merge_commit: String,
    pub predecessor_manifest_path: String,
    pub predecessor_closure: ResumePredecessorClosure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SynchronizationCheckpoint {
    pub authorized: bool,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SynchronizationHistoryEntry {
    pub synchronization_version: u32,
    pub from_base_commit: String,
    pub to_base_commit: String,
    pub base_remote: String,
    pub base_branch: String,
    pub work_branch: String,
    pub recorded_at: String,
    pub prior_contract_digest: Digest,
    pub prior_summary_digest: Digest,
    pub rebase_head_before: String,
    pub rebase_head_after: String,
    #[serde(default)]
    pub checkpoint_paths: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApprovalIdentityEvidencePayload {
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub pull_request: Option<u64>,
    #[serde(default)]
    pub review_id: Option<u64>,
    #[serde(default)]
    pub commit_sha: Option<String>,
    #[serde(default)]
    pub direct_user_instruction_ref: Option<String>,
    #[serde(default)]
    pub direct_user_instruction_digest: Option<Digest>,
    #[serde(default)]
    pub authorized_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApprovalIdentityEvidence {
    pub schema_version: u32,
    pub approval_type: String,
    pub identity_level: String,
    pub actor: String,
    #[serde(default)]
    pub provider: Option<String>,
    pub evidence: ApprovalIdentityEvidencePayload,
    pub scope: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApprovalEvidence {
    pub approved: bool,
    pub approved_by: String,
    pub reason: String,
    #[serde(default)]
    pub identity_evidence: Option<ApprovalIdentityEvidence>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RestrictedWriteApproval {
    pub approved: bool,
    pub approved_by: String,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DestructiveChangePolicy {
    pub allowed: bool,
    pub requires_human_approval: bool,
    #[serde(default)]
    pub allow_patterns: Vec<String>,
    #[serde(default)]
    /// Kept as raw JSON for protocol-v1 compatibility. Contract V2 validation
    /// parses this value as the strict `ApprovalEvidence` type below.
    pub approval_evidence: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Contract {
    pub protocol_version: u32,
    #[serde(default)]
    pub contract_version: Option<u32>,
    pub repository_id: String,
    #[serde(default)]
    pub work_item_id: String,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    pub intent: ContractIntent,
    pub goal: String,
    pub scope: Vec<String>,
    pub out_of_scope: Vec<String>,
    pub risk: String,
    pub authority: String,
    pub acceptance_criteria: Vec<String>,
    pub required_evidence_classes: Vec<String>,
    #[serde(default)]
    pub sources: Vec<ContractSource>,
    #[serde(default)]
    pub verification: Vec<VerificationDeclaration>,
    pub base_revision: String,
    pub project_profile_digest: Digest,
    pub repository_snapshot_digest: Digest,
    /// Optional external resource identity copied from the Work Item's
    /// provider/branch context. Historical Contracts omit it; when present,
    /// Runtime finalization can require a matching receipt context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_context: Option<ResourceFinalizationContext>,
    /// Reference V2 calls this field `baseCommit`; `baseRevision` remains the
    /// protocol-v1 spelling and is never rewritten.
    #[serde(default)]
    pub base_commit: Option<String>,
    #[serde(default)]
    pub baseline_dirty_paths: Vec<BaselineDirtyPath>,
    #[serde(default)]
    pub archive_sequence: Option<u64>,
    #[serde(default)]
    pub resume_history: Vec<ResumeHistoryEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predecessor_work_item_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predecessor_contract_digest: Option<Digest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_decision_path: Option<String>,
    #[serde(default)]
    pub synchronization_checkpoint: Option<SynchronizationCheckpoint>,
    #[serde(default)]
    pub synchronization_history: Vec<SynchronizationHistoryEntry>,
    #[serde(default)]
    pub guidelines: Vec<String>,
    #[serde(default)]
    pub acceptance: Option<Vec<String>>,
    #[serde(default)]
    pub pre_review_warnings: Vec<String>,
    #[serde(default)]
    pub authority_evidence: Option<AuthorityEvidence>,
    #[serde(default)]
    pub operation: Option<String>,
    #[serde(default)]
    pub governance_policy: Option<GovernancePolicy>,
    #[serde(default)]
    pub problem_statement: Option<String>,
    #[serde(default)]
    pub risk_assessment: Option<ContractRiskAssessment>,
    #[serde(default)]
    pub agent_capability: Option<ContractAgentCapability>,
    #[serde(default)]
    pub execution_decision: Option<ContractExecutionDecision>,
    #[serde(default)]
    pub destructive_change_policy: Option<DestructiveChangePolicy>,
    #[serde(default)]
    pub rollback_note: Option<String>,
    #[serde(default)]
    pub rollback_plan: Option<serde_json::Value>,
    #[serde(default)]
    pub unknowns: Vec<String>,
    #[serde(default)]
    pub not_codable: Option<bool>,
    /// Reserved as typed protocol fields by WI-122 and WI-123.  Keeping the
    /// outer fields explicit now prevents silent top-level drops while their
    /// domain validators evolve independently.
    #[serde(default)]
    pub scenario_coverage: Option<serde_json::Value>,
    #[serde(default)]
    /// Explicit paths and serialized projections that govern whether this
    /// Work Item may share a parallel execution slot with another item.
    /// `None` preserves protocol-v1 Contract compatibility; callers must use
    /// the legacy intelligence/scope projection in that case.
    pub concurrency_boundary: Option<ConcurrencyBoundary>,
    #[serde(default)]
    pub checkpoint_policy: Option<serde_json::Value>,
    #[serde(default)]
    pub human_decision_points: Option<serde_json::Value>,
    #[serde(default)]
    pub documentation_impact: Option<serde_json::Value>,
    #[serde(default)]
    pub performance_impact: Option<serde_json::Value>,
    #[serde(default)]
    pub residual_risk_expectation: Option<String>,
    #[serde(default)]
    pub governance_profile: Option<serde_json::Value>,
    #[serde(default)]
    pub requested_operation: Option<String>,
    #[serde(default)]
    pub implementation_surface: Option<serde_json::Value>,
    #[serde(default)]
    /// Kept as raw JSON so historical repositories with provider-specific
    /// fields remain readable. Contract V2 validates it as
    /// `RestrictedWriteApproval` and rejects unknown fields.
    pub restricted_write_approval: Option<serde_json::Value>,
    #[serde(default)]
    pub adoption_bootstrap_paths: Vec<String>,
}

impl Contract {
    /// Validate Contract-owned schema and cross-field invariants.
    ///
    /// Deserialization rejects unknown fields and malformed typed nested
    /// records.  This second pass rejects combinations that are individually
    /// well-typed but would widen authority, lose lineage, or make a `code`
    /// Work Item appear ready while it still has unresolved unknowns.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let is_v2 = self.contract_version == Some(2);

        if let Some(resource_context) = &self.resource_context
            && let Err(error) = validate_resource_finalization_context(resource_context)
        {
            errors.push(error.to_string());
        }

        if let Some(version) = self.contract_version
            && version != 2
        {
            errors.push(format!("unsupported contractVersion {version}"));
        }

        if is_v2 {
            match self.mode.as_deref() {
                Some("investigate" | "author_todo" | "code" | "review" | "cleanup") => {}
                Some(other) => {
                    errors.push(format!("mode {other} is not a supported Contract V2 mode"))
                }
                None => errors.push("mode is required for Contract V2".into()),
            }
            if self
                .title
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            {
                errors.push("title is required for Contract V2".into());
            }
            if self.not_codable.is_none() {
                errors.push("notCodable is required for Contract V2".into());
            }
        }

        if let Some(base_commit) = &self.base_commit
            && base_commit != &self.base_revision
        {
            errors.push("baseCommit must match baseRevision when both are present".into());
        }
        if let Some(base_commit) = &self.base_commit
            && base_commit.trim().is_empty()
        {
            errors.push("baseCommit must be non-empty when present".into());
        }
        if let Some(acceptance) = &self.acceptance
            && acceptance != &self.acceptance_criteria
        {
            errors.push("acceptance must match acceptanceCriteria when both are present".into());
        }

        if self.mode.as_deref() == Some("code") {
            if !self.unknowns.is_empty() {
                errors.push("code mode requires unknowns to be empty".into());
            }
            if self.not_codable == Some(true) {
                errors.push("code mode requires notCodable to be false".into());
            }
        }

        for (index, item) in self.baseline_dirty_paths.iter().enumerate() {
            if item.path.trim().is_empty() || item.path.starts_with('/') || item.path.contains("..")
            {
                errors.push(format!(
                    "baselineDirtyPaths[{index}].path must be repository-relative"
                ));
            }
            if item.status.trim().is_empty() {
                errors.push(format!("baselineDirtyPaths[{index}].status is required"));
            }
            if item.fingerprint.trim().is_empty() {
                errors.push(format!(
                    "baselineDirtyPaths[{index}].fingerprint is required"
                ));
            }
        }

        if self.archive_sequence == Some(0) {
            errors.push("archiveSequence must be a positive integer".into());
        }

        if self.predecessor_work_item_id.is_some() {
            if self
                .predecessor_work_item_id
                .as_deref()
                .is_none_or(|value| value.is_empty())
                || self.predecessor_contract_digest.is_none()
                || self
                    .recovery_decision_path
                    .as_deref()
                    .is_none_or(|value| value.is_empty())
            {
                errors.push(
                    "successor Contract requires predecessorWorkItemId, predecessorContractDigest, and recoveryDecisionPath".into(),
                );
            }
            if self.predecessor_work_item_id.as_deref() == Some(self.work_item_id.as_str()) {
                errors.push(
                    "successor Contract predecessorWorkItemId must differ from workItemId".into(),
                );
            }
        } else if self.predecessor_contract_digest.is_some()
            || self.recovery_decision_path.is_some()
        {
            errors.push(
                "predecessorContractDigest and recoveryDecisionPath require predecessorWorkItemId"
                    .into(),
            );
        }

        for (index, item) in self.resume_history.iter().enumerate() {
            if item.resume_version != (index as u32 + 1) {
                errors.push(format!(
                    "resumeHistory[{index}].resumeVersion must be contiguous"
                ));
            }
            if item.from_base_commit.trim().is_empty()
                || item.to_base_commit.trim().is_empty()
                || item.base_remote.trim().is_empty()
                || item.base_branch.trim().is_empty()
                || item.work_branch.trim().is_empty()
                || item.recorded_at.trim().is_empty()
                || item.predecessor_work_item_id.trim().is_empty()
                || item.predecessor_merge_commit.trim().is_empty()
                || item.predecessor_manifest_path.trim().is_empty()
            {
                errors.push(format!("resumeHistory[{index}] has an empty lineage field"));
            }
            let closure = &item.predecessor_closure;
            if !(closure.status_closed
                && closure.pr_merged
                && closure.closure_succeeded
                && closure.local_branch_deleted
                && closure.remote_branch_deleted
                && closure.base_synchronized)
            {
                errors.push(format!(
                    "resumeHistory[{index}].predecessorClosure must prove closed predecessor"
                ));
            }
            if let Some(base_commit) = &self.base_commit
                && index + 1 == self.resume_history.len()
                && &item.to_base_commit != base_commit
            {
                errors.push("resumeHistory final toBaseCommit must match baseCommit".into());
            }
        }

        if let Some(checkpoint) = &self.synchronization_checkpoint {
            if !checkpoint.authorized {
                errors
                    .push("synchronizationCheckpoint.authorized must be true when present".into());
            }
            if checkpoint.reason.trim().is_empty() {
                errors.push("synchronizationCheckpoint.reason is required".into());
            }
        }
        if !self.synchronization_history.is_empty() && self.synchronization_checkpoint.is_none() {
            errors.push("synchronizationHistory requires synchronizationCheckpoint".into());
        }
        for (index, item) in self.synchronization_history.iter().enumerate() {
            if item.synchronization_version != (index as u32 + 1) {
                errors.push(format!(
                    "synchronizationHistory[{index}].synchronizationVersion must be contiguous"
                ));
            }
            if item.from_base_commit.trim().is_empty()
                || item.to_base_commit.trim().is_empty()
                || item.base_remote.trim().is_empty()
                || item.base_branch.trim().is_empty()
                || item.work_branch.trim().is_empty()
                || item.recorded_at.trim().is_empty()
                || item.rebase_head_before.trim().is_empty()
                || item.rebase_head_after.trim().is_empty()
            {
                errors.push(format!(
                    "synchronizationHistory[{index}] has an empty lineage field"
                ));
            }
            if item
                .checkpoint_paths
                .iter()
                .any(|path| path.trim().is_empty())
            {
                errors.push(format!(
                    "synchronizationHistory[{index}].checkpointPaths contains an empty path"
                ));
            }
            if is_v2 && item.checkpoint_paths.is_empty() {
                errors.push(format!(
                    "synchronizationHistory[{index}].checkpointPaths must be non-empty for Contract V2"
                ));
            }
        }

        if self.guidelines.iter().any(|value| value.trim().is_empty()) {
            errors.push("guidelines must contain only non-empty strings".into());
        }
        if let Some(acceptance) = &self.acceptance {
            for (index, criterion) in acceptance.iter().enumerate() {
                if criterion.trim().is_empty() {
                    errors.push(format!("acceptance[{index}] must be non-empty"));
                }
                if is_v2 {
                    let Some((prefix, _)) = criterion.split_once(':') else {
                        errors.push(format!(
                            "acceptance[{index}] must use a stable A<n>: prefix"
                        ));
                        continue;
                    };
                    if !prefix.strip_prefix('A').is_some_and(|suffix| {
                        !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
                    }) {
                        errors.push(format!(
                            "acceptance[{index}] has an invalid stable identifier"
                        ));
                    }
                }
            }
        }
        if let Some(policy) = &self.destructive_change_policy
            && policy.allowed
            && policy.requires_human_approval
        {
            match policy.approval_evidence.as_ref() {
                Some(value) if is_v2 => match serde_json::from_value::<ApprovalEvidence>(value.clone()) {
                    Ok(evidence) => validate_approval_evidence(&evidence, &policy.allow_patterns, &mut errors),
                    Err(error) => errors.push(format!("invalid destructiveChangePolicy.approvalEvidence: {error}")),
                },
                Some(_) => {}
                None => errors.push("destructiveChangePolicy.approvalEvidence is required for approved destructive changes".into()),
            }
        }
        if let Some(value) = &self.restricted_write_approval
            && is_v2
        {
            match serde_json::from_value::<RestrictedWriteApproval>(value.clone()) {
                Ok(approval)
                    if approval.approved
                        && (approval.approved_by.trim().is_empty()
                            || approval.reason.trim().is_empty()) =>
                {
                    errors.push(
                        "restrictedWriteApproval approved records require approvedBy and reason"
                            .into(),
                    );
                }
                Ok(_) => {}
                Err(error) => errors.push(format!("invalid restrictedWriteApproval: {error}")),
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn validate_approval_evidence(
    evidence: &ApprovalEvidence,
    allow_patterns: &[String],
    errors: &mut Vec<String>,
) {
    if !evidence.approved
        || evidence.approved_by.trim().is_empty()
        || evidence.reason.trim().is_empty()
    {
        errors.push("approvalEvidence requires approved=true, approvedBy, and reason".into());
    }
    let Some(identity) = evidence.identity_evidence.as_ref() else {
        errors.push("approvalEvidence.identityEvidence is required".into());
        return;
    };
    if identity.schema_version != 1 {
        errors.push("approvalEvidence.identityEvidence.schemaVersion must be 1".into());
    }
    if !matches!(
        identity.identity_level.as_str(),
        "provider_verified" | "enterprise_verified" | "direct_user_authorized"
    ) {
        errors.push(
            "approvalEvidence.identityEvidence.identityLevel is not sufficiently assured".into(),
        );
    }
    if identity.actor.trim().is_empty() || identity.approval_type.trim().is_empty() {
        errors.push("approvalEvidence.identityEvidence requires approvalType and actor".into());
    }
    if identity.scope != allow_patterns {
        errors.push(
            "approvalEvidence.identityEvidence.scope must exactly match allowPatterns".into(),
        );
    }
    if identity.identity_level == "direct_user_authorized" {
        if identity.provider.is_some() {
            errors.push("direct_user_authorized identity must not name a provider".into());
        }
        let payload = &identity.evidence;
        if payload
            .direct_user_instruction_ref
            .as_deref()
            .is_none_or(str::is_empty)
            || payload.direct_user_instruction_digest.is_none()
            || payload.authorized_at.as_deref().is_none_or(str::is_empty)
        {
            errors.push("direct_user_authorized evidence requires direct instruction reference, digest, and time".into());
        }
    } else if identity.provider.as_deref().is_none_or(str::is_empty) {
        errors.push("provider or enterprise identity requires provider".into());
    }
}

pub const CONCURRENCY_BOUNDARY_SCHEMA_VERSION: u32 = 1;
pub const PARALLEL_SLOT_LEASE_SCHEMA_VERSION: u32 = 1;

/// A Contract-owned parallelism boundary.  The path classes are deliberately
/// separate so an Agent can explain why two Work Items must serialize rather
/// than treating `scope` as an opaque permission to run concurrently.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConcurrencyBoundary {
    #[serde(default = "default_concurrency_boundary_schema_version")]
    pub schema_version: u32,
    pub implementation_paths: Vec<String>,
    pub generated_evidence_paths: Vec<String>,
    pub verification_output_paths: Vec<String>,
    pub serialized_projection_paths: Vec<String>,
    #[serde(default = "default_parallel_max_workers")]
    pub max_workers: u32,
    pub reason: String,
}

fn default_concurrency_boundary_schema_version() -> u32 {
    CONCURRENCY_BOUNDARY_SCHEMA_VERSION
}

fn default_parallel_max_workers() -> u32 {
    1
}

impl ConcurrencyBoundary {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != CONCURRENCY_BOUNDARY_SCHEMA_VERSION {
            return Err(format!(
                "unsupported concurrency boundary schema {}",
                self.schema_version
            ));
        }
        if self.max_workers == 0 {
            return Err("concurrency boundary maxWorkers must be positive".into());
        }
        if self.reason.trim().is_empty() {
            return Err("concurrency boundary reason must not be empty".into());
        }
        if self.all_paths().is_empty() {
            return Err("concurrency boundary must declare at least one path".into());
        }
        Ok(())
    }

    pub fn all_paths(&self) -> Vec<(&'static str, &str)> {
        self.implementation_paths
            .iter()
            .map(|path| ("implementationPaths", path.as_str()))
            .chain(
                self.generated_evidence_paths
                    .iter()
                    .map(|path| ("generatedEvidencePaths", path.as_str())),
            )
            .chain(
                self.verification_output_paths
                    .iter()
                    .map(|path| ("verificationOutputPaths", path.as_str())),
            )
            .chain(
                self.serialized_projection_paths
                    .iter()
                    .map(|path| ("serializedProjectionPaths", path.as_str())),
            )
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ParallelSlotLease {
    #[serde(default = "default_parallel_slot_lease_schema_version")]
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub slot_id: u32,
    pub lease_id: String,
    pub max_workers: u32,
    pub acquired_at: String,
}

fn default_parallel_slot_lease_schema_version() -> u32 {
    PARALLEL_SLOT_LEASE_SCHEMA_VERSION
}

/// Provenance for a repository fact.  The Runtime never promotes a derived
/// interpretation to an observed fact; consumers can therefore distinguish
/// what the Observer saw from what an implementation approach inferred.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactOrigin {
    Observed,
    Declared,
    Derived,
    External,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TraceableFact {
    pub key: String,
    pub value: serde_json::Value,
    pub origin: FactOrigin,
    pub evidence_refs: Vec<String>,
    pub confidence: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TraceableDerivation {
    pub key: String,
    pub value: serde_json::Value,
    pub rule: String,
    pub input_fact_keys: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub confidence: String,
}

/// A request-scoped implementation approach. It is an auditable projection,
/// not a new authority source: observed facts and derivations remain separate
/// and unresolved questions stay visible in `unknowns`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImplementationApproach {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub repository_snapshot_digest: Digest,
    pub facts: Vec<TraceableFact>,
    pub derivations: Vec<TraceableDerivation>,
    pub unknowns: Vec<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TruthState {
    Observed,
    Declared,
    Verified,
    Derived,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgeV2Record {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub topic: String,
    pub component: String,
    pub state: String,
    pub truth_state: TruthState,
    pub confidence: String,
    pub knowledge_path: String,
    pub evidence_refs: Vec<String>,
    pub unknowns: Vec<String>,
    pub source_snapshot_digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeState {
    Verified,
    Partial,
    NotReady,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HumanBenefitReport {
    pub state: OutcomeState,
    pub user_visible_changes: Vec<String>,
    pub affected_users: Vec<String>,
    pub unknowns: Vec<String>,
    pub evidence_refs: Vec<String>,
}

/// An auditable claim used by the Task Outcome projection.  A claim is either
/// backed by repository-local evidence references or explicitly marked as an
/// inference; the Runtime never silently turns prose into a fact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutcomeClaim {
    pub text: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub inference: bool,
}

/// A deterministic, typed section set aligned with the reference Task Outcome
/// report. Empty sections are intentional and render as `None`; they do not
/// imply that a check was performed or that a benefit exists.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutcomeReportSections {
    pub outcome_summary: Vec<OutcomeClaim>,
    pub task_overview: Vec<OutcomeClaim>,
    pub delivered_changes: Vec<OutcomeClaim>,
    pub findings: Vec<OutcomeClaim>,
    pub risks: Vec<OutcomeClaim>,
    pub warnings: Vec<OutcomeClaim>,
    pub limitations: Vec<OutcomeClaim>,
    pub non_risk_explanations: Vec<OutcomeClaim>,
    pub forbidden_claims: Vec<String>,
    pub interventions: Vec<OutcomeClaim>,
    pub forced_stops: Vec<OutcomeClaim>,
    pub resolutions: Vec<OutcomeClaim>,
    pub recurrence_prevention: Vec<OutcomeClaim>,
    pub avoided_impact: Vec<OutcomeClaim>,
    pub residual_risks: Vec<OutcomeClaim>,
    pub human_decisions: Vec<OutcomeClaim>,
    pub evidence: Vec<OutcomeClaim>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation_approach: Option<ImplementationApproach>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutcomeReportBindings {
    pub repository_id: String,
    pub work_item_id: String,
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository_snapshot_digest: Option<Digest>,
}

/// The machine-readable report source for the human handoff.  It is additive
/// on OutcomeV2 so archived pre-report records remain readable, while every
/// newly generated OutcomeV2 contains this projection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskOutcomeReport {
    pub format: String,
    pub schema_version: u32,
    pub work_item_id: String,
    pub status: OutcomeState,
    pub human_status_color: DecisionState,
    pub bindings: OutcomeReportBindings,
    pub sections: OutcomeReportSections,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed_gate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_condition: Option<String>,
}

/// One append-only Task Outcome event.  Events are evidence inputs to the
/// report projection; they never grant authority or replace lifecycle receipts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskOutcomeEvent {
    pub schema_version: u32,
    pub event_id: String,
    pub repository_id: String,
    pub work_item_id: String,
    pub event_type: String,
    pub timestamp: String,
    pub detail: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub related_event_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correction_of: Option<String>,
}

/// Repository-bound human decision that authorizes a retry or creates an
/// explicitly linked successor. The receipt never rewrites the predecessor
/// and never asserts that the successor has passed verification.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecoveryDecisionReceipt {
    pub schema_version: u32,
    pub decision_id: String,
    pub decision: String,
    pub work_item_id: String,
    pub repository_id: String,
    pub predecessor_work_item_id: String,
    pub predecessor_contract_digest: Digest,
    pub predecessor_summary_digest: Digest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predecessor_outcome_digest: Option<Digest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predecessor_events_digest: Option<Digest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub successor_work_item_id: Option<String>,
    pub runtime_version: String,
    pub runtime_digest: Digest,
    pub actor: String,
    pub authority_source: String,
    pub reason: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub policy_refs: Vec<String>,
    pub decided_at: String,
    pub resume_condition: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutcomeV2 {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub state: OutcomeState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_state: Option<DecisionState>,
    pub summary: String,
    pub acceptance_results: Vec<String>,
    pub unknowns: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub human_benefit_report: HumanBenefitReport,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_outcome_report: Option<TaskOutcomeReport>,
    /// Explicit lifecycle failure metadata. These fields are additive so
    /// archived protocol-v1/v2 records remain readable; they never imply
    /// that a failed gate was repaired or that completion is authorized.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed_gate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_condition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_decision: Option<RecoveryDecisionReceipt>,
    /// A historical terminal projection distinct from a current verification
    /// failure. `superseded` means the predecessor was closed by an explicit,
    /// identity-bound decision while its original evidence bytes were kept.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub historical_status: Option<String>,
}

/// A read-only, evidence-bound Work Item status projection.  Counts are
/// deliberately facts; the Runtime never turns them into a percentage or a
/// completion promise.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkItemStatusSnapshot {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub lifecycle_phase: String,
    pub governance_state: String,
    pub activity_health: String,
    pub progress_facts: BTreeMap<String, u64>,
    pub blockers: Vec<String>,
    pub missing_evidence: Vec<String>,
    pub dependencies: Vec<String>,
    pub human_decisions: Vec<String>,
    pub risks: Vec<String>,
    pub verification: String,
    pub completion_domains: BTreeMap<String, String>,
    pub governance_permissions: Vec<String>,
    pub source_digests: BTreeMap<String, Digest>,
    pub unknowns: Vec<String>,
    pub diagnostics: Vec<String>,
    pub snapshot_digest: Digest,
    pub historical: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityConfidence {
    Unknown,
    Low,
    Medium,
    High,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityTruth {
    pub capability: String,
    pub state: TruthState,
    pub confidence: CapabilityConfidence,
    pub source: FactOrigin,
    pub evidence_refs: Vec<String>,
    pub verification: Option<String>,
    pub unknowns: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityTruthRegistry {
    pub schema_version: u32,
    pub repository_id: String,
    pub snapshot_digest: Digest,
    pub capabilities: Vec<CapabilityTruth>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GovernanceCost {
    pub snapshot_git_calls: usize,
    pub snapshot_files_read: usize,
    pub snapshot_files_hashed: usize,
    pub verification_runs: usize,
    pub verification_nodes_executed: usize,
    pub verification_nodes_reused: usize,
    pub elapsed_ms: u128,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosisState {
    Known,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PerformanceDiagnosis {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: Option<String>,
    pub state: DiagnosisState,
    pub cost: GovernanceCost,
    pub bottlenecks: Vec<String>,
    pub unknowns: Vec<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkItemIntelligence {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub depends_on: Vec<String>,
    pub conflicts_with: Vec<String>,
    pub parallelizable: bool,
    pub unknowns: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkItemCompatibility {
    pub repository_id: String,
    pub work_item_id: String,
    pub compatible: bool,
    pub dependencies_satisfied: bool,
    pub conflicts: Vec<String>,
    pub reasons: Vec<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("unsupported protocol major version {0}")]
    UnsupportedMajor(u32),
    #[error("unsupported agent interface version {0}")]
    UnsupportedAgentInterface(u32),
    #[error("malformed protocol version")]
    Malformed,
}

pub fn validate_protocol_version(version: u32) -> Result<(), ProtocolError> {
    if version == PROTOCOL_VERSION {
        Ok(())
    } else {
        Err(ProtocolError::UnsupportedMajor(version))
    }
}

pub fn validate_agent_interface_version(version: u32) -> Result<(), ProtocolError> {
    if version == AGENT_INTERFACE_VERSION {
        Ok(())
    } else {
        Err(ProtocolError::UnsupportedAgentInterface(version))
    }
}

pub fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(value)
}

pub fn digest_json<T: Serialize>(value: &T) -> Result<Digest, serde_json::Error> {
    Ok(Digest::sha256_bytes(&canonical_json(value)?))
}

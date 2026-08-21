use cockpit_core::Digest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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

/// The assurance source for an authority or delegated evidence claim. These
/// labels describe provenance, not an automatic approval.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssuranceLevel {
    SelfDeclared,
    RepositoryVerified,
    ProviderVerified,
    EnterpriseVerified,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contract {
    pub protocol_version: u32,
    pub repository_id: String,
    #[serde(default)]
    pub work_item_id: String,
    pub intent: String,
    pub goal: String,
    pub scope: Vec<String>,
    pub out_of_scope: Vec<String>,
    pub risk: String,
    pub authority: String,
    pub acceptance_criteria: Vec<String>,
    pub required_evidence_classes: Vec<String>,
    pub base_revision: String,
    pub project_profile_digest: Digest,
    pub repository_snapshot_digest: Digest,
    #[serde(default)]
    pub operation: Option<String>,
    #[serde(default)]
    pub governance_policy: Option<GovernancePolicy>,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutcomeV2 {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub state: OutcomeState,
    pub summary: String,
    pub acceptance_results: Vec<String>,
    pub unknowns: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub human_benefit_report: HumanBenefitReport,
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

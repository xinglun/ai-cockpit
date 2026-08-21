use cockpit_core::Digest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

pub const PROTOCOL_VERSION: u32 = 1;
pub const AGENT_INTERFACE_VERSION: u32 = 1;
pub const REPOSITORY_SCHEMA_VERSION: u32 = 2;

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

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EvidenceRetentionError {
    #[error("secret_prohibited evidence cannot use full_capture or redacted_capture")]
    SecretCaptureForbidden,
    #[error("disposal_action must be explicit")]
    MissingDisposalAction,
}

pub fn validate_evidence_retention(
    retention: &EvidenceRetention,
) -> Result<(), EvidenceRetentionError> {
    if retention.disposal_action.trim().is_empty() {
        return Err(EvidenceRetentionError::MissingDisposalAction);
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
    pub evidence_refs: Vec<String>,
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

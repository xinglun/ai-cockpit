use cockpit_core::Digest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

pub const PROTOCOL_VERSION: u32 = 1;
pub const AGENT_INTERFACE_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepositoryConfig {
    #[serde(rename = "protocol_version")]
    pub protocol_version: u32,
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

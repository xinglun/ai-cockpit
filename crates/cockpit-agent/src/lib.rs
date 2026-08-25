use cockpit_git::GitRepository;
use cockpit_protocol::{
    AgentInterfaceManifest, AgentProvider, RepositoryConfig, validate_agent_interface_version,
    validate_protocol_version,
};
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest as ShaDigest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use thiserror::Error;

const MAX_AGENT_METADATA_BYTES: u64 = 1024 * 1024;
const ADAPTER_BEGIN_MARKER: &str = "<!-- AI_COCKPIT_ADAPTER_BEGIN";
const ADAPTER_END_MARKER: &str = "<!-- AI_COCKPIT_ADAPTER_END -->";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentRepositoryContext {
    pub root: PathBuf,
    pub manifest_path: PathBuf,
    pub repository_id: String,
    pub manifest: AgentInterfaceManifest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DetectionResult {
    pub provider: AgentProvider,
    pub target: PathBuf,
    pub state: String,
    pub current_digest: Option<String>,
    pub conflict: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AdapterPlan {
    pub provider: AgentProvider,
    pub target: PathBuf,
    pub repository_id: String,
    pub current_digest: Option<String>,
    pub conflict: Option<String>,
    pub executable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AdapterReceipt {
    pub provider: AgentProvider,
    pub target: PathBuf,
    pub ownership_path: PathBuf,
    pub repository_id: String,
    pub installed_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentState {
    Unattached,
    Attached,
    DiscoveryAvailable,
    AdapterInstalled,
    Connected,
    Verified,
    Degraded,
    Conflict,
}

impl AgentState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unattached => "UNATTACHED",
            Self::Attached => "ATTACHED",
            Self::DiscoveryAvailable => "DISCOVERY_AVAILABLE",
            Self::AdapterInstalled => "ADAPTER_INSTALLED",
            Self::Connected => "CONNECTED",
            Self::Verified => "VERIFIED",
            Self::Degraded => "DEGRADED",
            Self::Conflict => "CONFLICT",
        }
    }

    pub fn exit_code(self) -> AgentExitCode {
        match self {
            Self::Verified | Self::Connected => AgentExitCode::Ready,
            Self::Degraded | Self::Attached | Self::DiscoveryAvailable | Self::AdapterInstalled => {
                AgentExitCode::Degraded
            }
            Self::Unattached => AgentExitCode::ConfigurationError,
            Self::Conflict => AgentExitCode::InterventionRequired,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentExitCode {
    Ready = 0,
    Degraded = 1,
    ConfigurationError = 2,
    InterventionRequired = 3,
    IncompatibleProtocol = 4,
}

impl AgentExitCode {
    pub const fn code(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("repository is not attached: {0}")]
    Unattached(PathBuf),
    #[error("cannot read {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("invalid agent repository state at {path}: {message}")]
    State { path: PathBuf, message: String },
    #[error("repository discovery failed: {0}")]
    Git(String),
}

pub fn canonical_manifest_path(root: &Path) -> PathBuf {
    root.join(".ai/agent-interface.json")
}

pub fn load_agent_context(root: &Path) -> Result<AgentRepositoryContext, AgentError> {
    let requested_root = fs::canonicalize(root).map_err(|source| AgentError::Read {
        path: root.into(),
        source,
    })?;
    let git = GitRepository::discover(&requested_root)
        .map_err(|error| AgentError::Git(error.to_string()))?;
    let repository_root = fs::canonicalize(git.root()).map_err(|source| AgentError::Read {
        path: git.root().to_path_buf(),
        source,
    })?;
    let manifest_path = canonical_manifest_path(&repository_root);
    reject_symlink(&repository_root.join(".ai"))?;
    reject_symlink(&manifest_path)?;
    let manifest: AgentInterfaceManifest = read_bounded_json(&manifest_path)?;
    validate_protocol_version(manifest.protocol_version).map_err(|error| AgentError::State {
        path: manifest_path.clone(),
        message: error.to_string(),
    })?;
    validate_agent_interface_version(manifest.interface_version).map_err(|error| {
        AgentError::State {
            path: manifest_path.clone(),
            message: error.to_string(),
        }
    })?;
    if manifest.schema_version != 1 {
        return Err(AgentError::State {
            path: manifest_path.clone(),
            message: format!(
                "unsupported manifest schema version {}",
                manifest.schema_version
            ),
        });
    }
    if manifest.root_binding.binding_type != "manifest-parent" {
        return Err(AgentError::State {
            path: manifest_path.clone(),
            message: format!(
                "unsupported root binding type {}",
                manifest.root_binding.binding_type
            ),
        });
    }
    let config_path = repository_root.join(".ai/cockpit.toml");
    reject_symlink(&config_path)?;
    let config: RepositoryConfig =
        toml::from_str(&read_bounded_text(&config_path)?).map_err(|error| AgentError::State {
            path: config_path.clone(),
            message: error.to_string(),
        })?;
    validate_protocol_version(config.protocol_version).map_err(|error| AgentError::State {
        path: config_path.clone(),
        message: error.to_string(),
    })?;
    if config.repository_id != manifest.repository_id {
        return Err(AgentError::State {
            path: manifest_path,
            message: "manifest repositoryId does not match cockpit.toml".into(),
        });
    }
    if config.repository_schema_version != manifest.repository_schema_version {
        return Err(AgentError::State {
            path: manifest_path,
            message:
                "repository schema versions disagree between cockpit.toml and agent-interface.json"
                    .into(),
        });
    }
    Ok(AgentRepositoryContext {
        root: repository_root,
        manifest_path,
        repository_id: config.repository_id,
        manifest,
    })
}

pub fn repository_id_from_manifest(root: &Path) -> Result<String, AgentError> {
    Ok(load_agent_context(root)?.repository_id)
}

pub fn detect_providers(root: &Path) -> Result<Vec<DetectionResult>, AgentError> {
    let context = load_agent_context(root)?;
    let mut results = Vec::new();
    for provider in all_providers() {
        let target = resolve_provider_target(&context.root, &provider)?;
        if !surface_is_discoverable(&context.root, &provider, &target) {
            continue;
        }
        results.push(inspect_target(&provider, target)?);
    }
    results.sort_by(|left, right| left.target.cmp(&right.target));
    Ok(results)
}

pub fn plan_install(root: &Path, provider: AgentProvider) -> Result<AdapterPlan, AgentError> {
    let context = load_agent_context(root)?;
    let target = resolve_provider_target(&context.root, &provider)?;
    let inspection = inspect_target(&provider, target.clone())?;
    Ok(AdapterPlan {
        provider,
        target,
        repository_id: context.repository_id,
        current_digest: inspection.current_digest,
        executable: inspection.conflict.is_none(),
        conflict: inspection.conflict,
    })
}

pub fn install_adapter(root: &Path, provider: AgentProvider) -> Result<AdapterReceipt, AgentError> {
    let context = load_agent_context(root)?;
    let target = resolve_provider_target(&context.root, &provider)?;
    if !target.starts_with(&context.root) {
        return Err(AgentError::State {
            path: target,
            message: "adapter target escaped repository root".into(),
        });
    }
    let ownership_dir = context.root.join(".ai/adapters");
    reject_symlink(&context.root.join(".ai"))?;
    reject_existing_components(&context.root, &ownership_dir)?;
    if ownership_dir.exists() {
        reject_symlink(&ownership_dir)?;
    } else {
        fs::create_dir_all(&ownership_dir).map_err(|source| AgentError::Read {
            path: ownership_dir.clone(),
            source,
        })?;
    }
    let ownership_path = ownership_dir.join(format!("{}.json", provider_name(&provider)));
    if ownership_path.exists() {
        reject_symlink(&ownership_path)?;
    }

    let existing = if target.exists() {
        reject_symlink(&target)?;
        Some(read_bounded_text(&target)?)
    } else {
        None
    };
    let block = managed_block(&provider, &context.repository_id);
    let (new_content, installed_digest) = match existing {
        None => (block.clone(), sha256_bytes(block.as_bytes())),
        Some(text) => {
            let section = managed_section(&text, &target)?;
            if let Some(section) = section {
                let digest = sha256_bytes(section.as_bytes());
                let record: cockpit_protocol::ManagedAdapterRecord =
                    read_bounded_json(&ownership_path).map_err(|_| AgentError::State {
                        path: ownership_path.clone(),
                        message: "managed adapter exists without a valid ownership record".into(),
                    })?;
                let relative_target = relative_target(&context.root, &target)?;
                if record.provider != provider
                    || record.adapter_version != 1
                    || record.mode != "managed-section"
                    || record.repository_id != context.repository_id
                    || record.target != relative_target
                    || record.installed_digest != digest
                {
                    return Err(AgentError::State {
                        path: ownership_path,
                        message: "managed adapter ownership does not match current content".into(),
                    });
                }
                return Ok(AdapterReceipt {
                    provider,
                    target,
                    ownership_path,
                    repository_id: context.repository_id,
                    installed_digest: digest,
                });
            }
            let mut content = text.into_bytes();
            if !content.is_empty() && !content.ends_with(b"\n") {
                content.push(b'\n');
            }
            content.extend_from_slice(block.as_bytes());
            let content = String::from_utf8(content).map_err(|_| AgentError::State {
                path: target.clone(),
                message: "adapter target is not valid UTF-8".into(),
            })?;
            (content, sha256_bytes(block.as_bytes()))
        }
    };
    let relative_target = relative_target(&context.root, &target)?;
    let record = cockpit_protocol::ManagedAdapterRecord {
        provider: provider.clone(),
        adapter_version: 1,
        target: relative_target,
        mode: "managed-section".into(),
        repository_id: context.repository_id.clone(),
        installed_digest: installed_digest.clone(),
    };
    let previous_target = if target.exists() {
        Some(read_bounded_bytes(&target)?)
    } else {
        None
    };
    atomic_write(&target, new_content.as_bytes())?;
    let record_bytes = serde_json::to_vec_pretty(&record).map_err(|error| AgentError::State {
        path: ownership_path.clone(),
        message: error.to_string(),
    })?;
    if let Err(error) = atomic_write(&ownership_path, &record_bytes) {
        if let Some(previous) = previous_target {
            let _ = atomic_write(&target, &previous);
        } else {
            let _ = fs::remove_file(&target);
        }
        return Err(error);
    }
    Ok(AdapterReceipt {
        provider,
        target,
        ownership_path,
        repository_id: context.repository_id,
        installed_digest,
    })
}

/// Derive the current adapter state from repository facts. Cached state values
/// in manifests or ownership records are never trusted as the state machine.
pub fn doctor(root: &Path) -> Result<cockpit_protocol::AgentDoctorReport, AgentError> {
    let requested_root = fs::canonicalize(root).map_err(|source| AgentError::Read {
        path: root.into(),
        source,
    })?;
    let manifest_path = canonical_manifest_path(&requested_root);
    let config_path = requested_root.join(".ai/cockpit.toml");
    if !config_path.is_file() || !manifest_path.is_file() {
        return Ok(cockpit_protocol::AgentDoctorReport {
            schema_version: 1,
            state: AgentState::Unattached.as_str().into(),
            repository_id: None,
            attachment: cockpit_protocol::AgentDoctorCheck {
                state: "missing".into(),
            },
            manifest: cockpit_protocol::AgentDoctorCheck {
                state: "missing".into(),
            },
            adapters: Vec::new(),
            interfaces: cockpit_protocol::AgentDoctorInterfaces {
                cli: "unavailable".into(),
                mcp: "unavailable".into(),
            },
            problems: vec!["repository is not attached".into()],
            safe_actions: vec!["run ai-cockpit attach --repo <path>".into()],
        });
    }

    let context = match load_agent_context(&requested_root) {
        Ok(context) => context,
        Err(error) => {
            let message = error.to_string();
            return Ok(cockpit_protocol::AgentDoctorReport {
                schema_version: 1,
                state: AgentState::Conflict.as_str().into(),
                repository_id: None,
                attachment: cockpit_protocol::AgentDoctorCheck {
                    state: "invalid".into(),
                },
                manifest: cockpit_protocol::AgentDoctorCheck {
                    state: "invalid".into(),
                },
                adapters: Vec::new(),
                interfaces: cockpit_protocol::AgentDoctorInterfaces {
                    cli: "unknown".into(),
                    mcp: "unknown".into(),
                },
                problems: vec![message],
                safe_actions: vec!["repair the attached repository facts and rerun doctor".into()],
            });
        }
    };

    let mut adapters = Vec::new();
    let mut problems = Vec::new();
    let mut installed_count = 0_usize;
    for provider in all_providers() {
        let target = match resolve_provider_target(&context.root, &provider) {
            Ok(target) => target,
            Err(error) => {
                let target = canonical_provider_target(&context.root, &provider);
                let target_name = relative_target(&context.root, &target)?;
                adapters.push(cockpit_protocol::AgentDoctorAdapter {
                    provider,
                    state: "conflict".into(),
                    target: target_name.clone(),
                });
                problems.push(format!("{target_name}: {error}"));
                continue;
            }
        };
        let target_name = relative_target(&context.root, &target)?;
        let ownership_path = context
            .root
            .join(".ai/adapters")
            .join(format!("{}.json", provider_name(&provider)));
        let target_exists = target.exists();
        let record_exists = ownership_path.exists();
        let mut state = "not_installed";
        if target_exists {
            match inspect_target(&provider, target.clone()) {
                Ok(inspection) if inspection.conflict.is_some() => {
                    state = "conflict";
                    problems.push(format!(
                        "{} has duplicate or incomplete managed markers",
                        target_name
                    ));
                }
                Ok(inspection) if inspection.state == "installed" => {
                    let owned_by_provider = read_bounded_text(&target)
                        .ok()
                        .and_then(|text| {
                            managed_section(&text, &target)
                                .ok()
                                .flatten()
                                .map(String::from)
                        })
                        .and_then(|section| {
                            managed_section_provider(&section)
                                .map(|name| name == provider_name(&provider))
                        })
                        .unwrap_or(false);
                    if !owned_by_provider {
                        // AGENTS.md is a shared safe surface for both the
                        // generic adapter and Codex. A managed section owned
                        // by one provider is not a conflict for the other.
                        continue;
                    }
                    match read_managed_record(&ownership_path) {
                        Ok(record)
                            if record.provider == provider
                                && record.repository_id == context.repository_id
                                && record.target == target_name
                                && record.adapter_version == 1
                                && record.mode == "managed-section"
                                && inspection.current_digest.as_deref()
                                    == Some(record.installed_digest.as_str()) =>
                        {
                            state = "installed";
                            installed_count += 1;
                        }
                        Ok(_) | Err(_) => {
                            state = "conflict";
                            problems.push(format!(
                                "{} has no matching repository-owned adapter record",
                                target_name
                            ));
                        }
                    }
                }
                Ok(_) => {
                    if record_exists {
                        state = "conflict";
                        problems.push(format!(
                            "{} has an ownership record without a managed section",
                            target_name
                        ));
                    } else {
                        state = "available";
                    }
                }
                Err(error) => {
                    state = "conflict";
                    problems.push(error.to_string());
                }
            }
        } else if record_exists {
            state = "conflict";
            problems.push(format!(
                "{} ownership record exists but its target is missing",
                target_name
            ));
        } else if surface_is_discoverable(&context.root, &provider, &target) {
            state = "available";
        }
        if state != "not_installed" {
            adapters.push(cockpit_protocol::AgentDoctorAdapter {
                provider,
                state: state.into(),
                target: target_name,
            });
        }
    }

    let cli = if context.manifest.interfaces.cli.available {
        "available"
    } else {
        "unavailable"
    };
    let mcp = if context.manifest.interfaces.mcp.available {
        "available"
    } else {
        "unavailable"
    };
    let has_conflict = adapters.iter().any(|adapter| adapter.state == "conflict");
    let has_available = adapters.iter().any(|adapter| adapter.state == "available");
    let state = if has_conflict {
        AgentState::Conflict
    } else if installed_count > 0 && context.manifest.interfaces.cli.available {
        // The successful context load is the repository-bound probe. MCP is an
        // optional transport; its absence degrades the adapter but does not
        // invalidate CLI connectivity.
        if context.manifest.interfaces.mcp.available {
            AgentState::Verified
        } else {
            AgentState::Degraded
        }
    } else if has_available {
        AgentState::DiscoveryAvailable
    } else {
        AgentState::Attached
    };
    let mut safe_actions = Vec::new();
    if has_available {
        safe_actions
            .push("run ai-cockpit agent install --repo <path> --provider <provider>".into());
    }
    if has_conflict {
        safe_actions.push("review the conflicting adapter surface before repair or detach".into());
    }
    if installed_count > 0 && !context.manifest.interfaces.mcp.available {
        safe_actions
            .push("use the repository-bound CLI; configure MCP separately if desired".into());
    }
    Ok(cockpit_protocol::AgentDoctorReport {
        schema_version: 1,
        state: state.as_str().into(),
        repository_id: Some(context.repository_id),
        attachment: cockpit_protocol::AgentDoctorCheck {
            state: "attached".into(),
        },
        manifest: cockpit_protocol::AgentDoctorCheck {
            state: "valid".into(),
        },
        adapters,
        interfaces: cockpit_protocol::AgentDoctorInterfaces {
            cli: cli.into(),
            mcp: mcp.into(),
        },
        problems,
        safe_actions,
    })
}

pub fn detach_adapter(root: &Path, provider: AgentProvider) -> Result<(), AgentError> {
    let context = load_agent_context(root)?;
    let target = resolve_provider_target(&context.root, &provider)?;
    let ownership_path = context
        .root
        .join(".ai/adapters")
        .join(format!("{}.json", provider_name(&provider)));
    let record = read_managed_record(&ownership_path)?;
    let text = read_bounded_text(&target)?;
    let section = managed_section(&text, &target)?.ok_or_else(|| AgentError::State {
        path: target.clone(),
        message: "managed adapter section is missing".into(),
    })?;
    verify_record(&record, &provider, &context, &target, section)?;
    let new_text = text.replacen(section, "", 1);
    atomic_write(&target, new_text.as_bytes())?;
    if let Err(error) = remove_owned_record(&ownership_path) {
        // Best-effort rollback prevents a failed record removal from leaving a
        // target that no longer has an auditable ownership record.
        let _ = atomic_write(&target, text.as_bytes());
        return Err(error);
    }
    Ok(())
}

pub fn repair_adapter(root: &Path, provider: AgentProvider) -> Result<AdapterReceipt, AgentError> {
    let context = load_agent_context(root)?;
    let target = resolve_provider_target(&context.root, &provider)?;
    let ownership_path = context
        .root
        .join(".ai/adapters")
        .join(format!("{}.json", provider_name(&provider)));
    let record = read_managed_record(&ownership_path)?;
    let text = read_bounded_text(&target)?;
    let section = managed_section(&text, &target)?.ok_or_else(|| AgentError::State {
        path: target.clone(),
        message: "managed adapter section is missing; refusing automatic repair".into(),
    })?;
    verify_record(&record, &provider, &context, &target, section)?;
    install_adapter(&context.root, provider)
}

fn read_managed_record(path: &Path) -> Result<cockpit_protocol::ManagedAdapterRecord, AgentError> {
    read_bounded_json(path).map_err(|error| AgentError::State {
        path: path.into(),
        message: format!("invalid or missing managed adapter ownership record: {error}"),
    })
}

fn verify_record(
    record: &cockpit_protocol::ManagedAdapterRecord,
    provider: &AgentProvider,
    context: &AgentRepositoryContext,
    target: &Path,
    section: &str,
) -> Result<(), AgentError> {
    let target_name = relative_target(&context.root, target)?;
    let digest = sha256_bytes(section.as_bytes());
    if record.provider != *provider
        || record.adapter_version != 1
        || record.mode != "managed-section"
        || record.repository_id != context.repository_id
        || record.target != target_name
        || record.installed_digest != digest
    {
        return Err(AgentError::State {
            path: target.into(),
            message: "managed adapter content or ownership does not match; refusing operation"
                .into(),
        });
    }
    Ok(())
}

fn remove_owned_record(path: &Path) -> Result<(), AgentError> {
    reject_symlink(path)?;
    fs::remove_file(path).map_err(|source| AgentError::Read {
        path: path.into(),
        source,
    })
}

pub fn sha256_file(path: &Path) -> Result<String, AgentError> {
    reject_symlink(path)?;
    let mut file = File::open(path).map_err(|source| AgentError::Read {
        path: path.into(),
        source,
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    let mut total = 0_u64;
    loop {
        let count = file.read(&mut buffer).map_err(|source| AgentError::Read {
            path: path.into(),
            source,
        })?;
        if count == 0 {
            break;
        }
        total = total.saturating_add(count as u64);
        if total > MAX_AGENT_METADATA_BYTES {
            return Err(AgentError::State {
                path: path.into(),
                message: format!("file exceeds {MAX_AGENT_METADATA_BYTES} byte bound"),
            });
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("sha256:{}", hex::encode(hasher.finalize())))
}

fn read_bounded_json<T: DeserializeOwned>(path: &Path) -> Result<T, AgentError> {
    let bytes = read_bounded_bytes(path)?;
    serde_json::from_slice(&bytes).map_err(|error| AgentError::State {
        path: path.into(),
        message: error.to_string(),
    })
}

fn reject_symlink(path: &Path) -> Result<(), AgentError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| AgentError::Read {
        path: path.into(),
        source,
    })?;
    #[cfg(windows)]
    let is_reparse = {
        use std::os::windows::fs::MetadataExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
        metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    };
    #[cfg(not(windows))]
    let is_reparse = false;
    if metadata.file_type().is_symlink() || is_reparse {
        return Err(AgentError::State {
            path: path.into(),
            message: "symlink/reparse target is not an accepted repository surface".into(),
        });
    }
    Ok(())
}

fn reject_existing_components(root: &Path, path: &Path) -> Result<(), AgentError> {
    let relative = path.strip_prefix(root).map_err(|_| AgentError::State {
        path: path.into(),
        message: "path is outside repository root".into(),
    })?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        if current.exists() {
            reject_symlink(&current)?;
        }
    }
    Ok(())
}

fn all_providers() -> [AgentProvider; 5] {
    [
        AgentProvider::GenericAgentsMd,
        AgentProvider::Codex,
        AgentProvider::Claude,
        AgentProvider::Gemini,
        AgentProvider::Cursor,
    ]
}

fn canonical_provider_target(root: &Path, provider: &AgentProvider) -> PathBuf {
    match provider {
        AgentProvider::GenericAgentsMd | AgentProvider::Codex => root.join("AGENTS.md"),
        AgentProvider::Claude => root.join("CLAUDE.md"),
        AgentProvider::Gemini => root.join("GEMINI.md"),
        AgentProvider::Cursor => root.join(".cursor/rules/ai-cockpit.mdc"),
    }
}

fn resolve_provider_target(root: &Path, provider: &AgentProvider) -> Result<PathBuf, AgentError> {
    let canonical = canonical_provider_target(root, provider);
    if !matches!(provider, AgentProvider::Cursor) {
        return Ok(canonical);
    }

    // A repository that already has an owned Cursor adapter keeps its recorded
    // target, so upgrading the Runtime never silently renames a legacy .md
    // surface. The record is validated before it can influence any path.
    let ownership_path = root
        .join(".ai/adapters")
        .join(format!("{}.json", provider_name(provider)));
    if ownership_path.exists() {
        reject_symlink(&ownership_path)?;
        let record = read_managed_record(&ownership_path)?;
        if record.provider != *provider
            || record.adapter_version != 1
            || record.mode != "managed-section"
            || record.repository_id.is_empty()
        {
            return Err(AgentError::State {
                path: ownership_path,
                message: "invalid Cursor adapter ownership record".into(),
            });
        }
        let target = root.join(&record.target);
        validate_cursor_target(root, &target)?;
        return Ok(target);
    }

    if canonical.exists() {
        return Ok(canonical);
    }

    // Legacy .md is selected only when it contains an adapter marker. A
    // user-owned .md file is left untouched and a new install uses .mdc.
    let legacy = root.join(".cursor/rules/ai-cockpit.md");
    if legacy.exists() {
        reject_symlink(&legacy)?;
        let text = read_bounded_text(&legacy)?;
        if text.contains(ADAPTER_BEGIN_MARKER) || text.contains(ADAPTER_END_MARKER) {
            return Ok(legacy);
        }
    }
    Ok(canonical)
}

fn validate_cursor_target(root: &Path, target: &Path) -> Result<(), AgentError> {
    let relative = target.strip_prefix(root).map_err(|_| AgentError::State {
        path: target.into(),
        message: "Cursor adapter target escaped repository root".into(),
    })?;
    if relative.components().any(|component| {
        matches!(
            component,
            std::path::Component::Prefix(_)
                | std::path::Component::RootDir
                | std::path::Component::ParentDir
        )
    }) {
        return Err(AgentError::State {
            path: target.into(),
            message: "Cursor adapter target must be repository-relative".into(),
        });
    }
    let canonical = root.join(".cursor/rules/ai-cockpit.mdc");
    let legacy = root.join(".cursor/rules/ai-cockpit.md");
    if target != canonical && target != legacy {
        return Err(AgentError::State {
            path: target.into(),
            message: "Cursor adapter target is not a supported provider surface".into(),
        });
    }
    Ok(())
}

fn surface_is_discoverable(root: &Path, provider: &AgentProvider, target: &Path) -> bool {
    match provider {
        AgentProvider::Cursor => root.join(".cursor").is_dir() || target.is_file(),
        _ => target.is_file(),
    }
}

fn inspect_target(
    provider: &AgentProvider,
    target: PathBuf,
) -> Result<DetectionResult, AgentError> {
    let (state, current_digest, conflict) = if !target.exists() {
        ("not_installed".into(), None, None)
    } else {
        reject_symlink(&target)?;
        let bytes = read_bounded_bytes(&target)?;
        let text = String::from_utf8_lossy(&bytes);
        let begin_count = text.matches(ADAPTER_BEGIN_MARKER).count();
        let end_count = text.matches(ADAPTER_END_MARKER).count();
        let conflict = if begin_count > 1 || end_count > 1 {
            Some("duplicate managed adapter markers".into())
        } else if begin_count != end_count {
            Some("managed adapter markers are incomplete".into())
        } else {
            None
        };
        let state = if conflict.is_some() {
            "conflict"
        } else if begin_count == 1 {
            "installed"
        } else {
            "available"
        };
        let digest = if state == "installed" {
            managed_section(&text, &target)?.map(|section| sha256_bytes(section.as_bytes()))
        } else {
            Some(sha256_file(&target)?)
        };
        (state.into(), digest, conflict)
    };
    Ok(DetectionResult {
        provider: provider.clone(),
        target,
        state,
        current_digest,
        conflict,
    })
}

fn read_bounded_bytes(path: &Path) -> Result<Vec<u8>, AgentError> {
    reject_symlink(path)?;
    let mut file = File::open(path).map_err(|source| AgentError::Read {
        path: path.into(),
        source,
    })?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(MAX_AGENT_METADATA_BYTES.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|source| AgentError::Read {
            path: path.into(),
            source,
        })?;
    if bytes.len() as u64 > MAX_AGENT_METADATA_BYTES {
        return Err(AgentError::State {
            path: path.into(),
            message: format!("file exceeds {MAX_AGENT_METADATA_BYTES} byte bound"),
        });
    }
    Ok(bytes)
}

fn read_bounded_text(path: &Path) -> Result<String, AgentError> {
    let bytes = read_bounded_bytes(path)?;
    String::from_utf8(bytes).map_err(|_| AgentError::State {
        path: path.into(),
        message: "adapter target is not valid UTF-8".into(),
    })
}

fn managed_section<'a>(text: &'a str, path: &Path) -> Result<Option<&'a str>, AgentError> {
    let begins = text.match_indices(ADAPTER_BEGIN_MARKER).collect::<Vec<_>>();
    let ends = text.match_indices(ADAPTER_END_MARKER).collect::<Vec<_>>();
    if begins.len() > 1 || ends.len() > 1 {
        return Err(AgentError::State {
            path: path.into(),
            message: "duplicate managed adapter markers".into(),
        });
    }
    match (begins.first(), ends.first()) {
        (None, None) => Ok(None),
        (Some((begin, _)), Some((end, end_marker))) if begin < end => {
            let mut end = end + end_marker.len();
            // The managed block is emitted with one trailing line ending. Include
            // that line ending in the owned section digest so a second install is
            // byte-stable while still detecting any user edits around the block.
            if text.as_bytes().get(end) == Some(&b'\r') {
                end += 1;
            }
            if text.as_bytes().get(end) == Some(&b'\n') {
                end += 1;
            }
            Ok(Some(&text[*begin..end]))
        }
        _ => Err(AgentError::State {
            path: path.into(),
            message: "managed adapter markers are incomplete".into(),
        }),
    }
}

fn managed_block(provider: &AgentProvider, repository_id: &str) -> String {
    let mut block = format!(
        "<!-- AI_COCKPIT_ADAPTER_BEGIN provider={} adapterVersion=1 repositoryId={} -->\n\nThis repository is attached to AI Cockpit.\n\nCanonical interface: .ai/agent-interface.json\nRead .ai/README.md before acting; read .ai/glossary.md for the repository-local Agent route and vocabulary.\n\nUse the installed shared Rust Runtime as the repository-governance interface.\nEvery repository-bound command must include an explicit --repo <path>.\nPrefer MCP when available; CLI remains the fallback. Do not infer AI Cockpit state from this file. Query the Runtime for current governance state.\n\nBefore editing, query inspect, status, doctor, and agent doctor. Use one bounded Work Item, branch, and worktree. Keep all edits inside the Contract scope; amend and re-run preflight before expanding it.\n\nContract first: intent, scope, outOfScope, sources, unknowns, acceptance criteria, verification, and authority are human-owned. For code mode, unresolved unknowns or notCodable conditions stop implementation. Do not invent intent, approval, evidence, or completion.\n\nA preflight result of not_ready or needs_human_confirmation is a mandatory human pause. Show the humanDecisionRequest and resume condition; a successful command or yellow result is not authorization.\n\nFor authorized changes use: start or work-item new → preflight → checkpoint → verify → finish → archive → close. Keep the Summary current with changed paths and reasons, sources, verification commands/results, guideline compliance, unknowns, risk, generated/destructive changes, and observed issues.\n\nBefore archive, present a visible human Outcome with 🟢/🟡/🔴, facts, unknowns, evidence, human decision, and next action. A raw MCP record or folded-only output is not a human handoff. Close only after the merged PR, archive, decision, default-branch synchronization, clean worktrees, and exact branch removal are verified.\n\nNever edit global Agent or MCP configuration, secrets, or credentials. Do not copy V1 runtime code, Python modules, Make commands, installers, or schemas into this repository.\n\n{}\n",
        provider_name(provider),
        repository_id,
        ADAPTER_END_MARKER
    );
    let added_guidance = "\n\nCanonical delivery order is latest remote default base → dedicated branch/worktree → implement → finish/archive → push → reviewed PR → merge → close → synchronize and clean. Never merge a feature branch into local main before PR review, delete its branch before merge, or let a provider auto-delete it to bypass finalization. If a remote step fails, preserve the retry checkout and identity until recovery is complete.\n\nA terminal green Outcome is the Rust equivalent of status=completed plus humanStatusColor=green: it requires state=Verified, decisionState=green, current Contract/Summary/evidence bindings, and direct human-visible delivery. Include issue count, blockers/stopping reason, resolved issues, risks, unknowns, verification, impact, human decision, and next action; every factual claim needs evidence, and unproven benefit is an inference.\n\nWhen a defect is found in the current Work Item, repair it there by amending and revalidating its Contract before opening another Work Item or Issue. A successor is allowed only for a genuinely different scope, authority, or base, an independent compatible change, an unsafe in-scope repair, immutable failed delivery, or explicit human direction.";
    block = block.replace(
        "\n\nNever edit global Agent or MCP configuration, secrets, or credentials.",
        &format!("{added_guidance}\n\nNever edit global Agent or MCP configuration, secrets, or credentials."),
    );
    block
}

fn provider_name(provider: &AgentProvider) -> &'static str {
    match provider {
        AgentProvider::GenericAgentsMd => "generic-agents-md",
        AgentProvider::Codex => "codex",
        AgentProvider::Claude => "claude",
        AgentProvider::Gemini => "gemini",
        AgentProvider::Cursor => "cursor",
    }
}

fn managed_section_provider(section: &str) -> Option<&str> {
    let header = section.lines().next()?;
    let value = header.strip_prefix("<!-- AI_COCKPIT_ADAPTER_BEGIN ")?;
    value
        .split_whitespace()
        .find_map(|part| part.strip_prefix("provider="))
}

fn relative_target(root: &Path, target: &Path) -> Result<String, AgentError> {
    target
        .strip_prefix(root)
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .map_err(|_| AgentError::State {
            path: target.into(),
            message: "adapter target escaped repository root".into(),
        })
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), AgentError> {
    if path.exists() {
        reject_symlink(path)?;
    }
    let parent = path.parent().ok_or_else(|| AgentError::State {
        path: path.into(),
        message: "adapter path has no parent".into(),
    })?;
    if let Some(root) = path
        .ancestors()
        .find(|candidate| candidate.join(".ai").is_dir())
    {
        reject_existing_components(root, parent)?;
    }
    fs::create_dir_all(parent).map_err(|source| AgentError::Read {
        path: parent.into(),
        source,
    })?;
    reject_symlink(parent)?;
    let mut temporary = NamedTempFile::new_in(parent).map_err(|source| AgentError::Read {
        path: parent.into(),
        source,
    })?;
    temporary
        .write_all(bytes)
        .and_then(|_| temporary.as_file().sync_all())
        .map_err(|source| AgentError::Read {
            path: parent.into(),
            source,
        })?;
    let temporary_path = temporary.into_temp_path();
    replace_file(temporary_path.as_ref(), path)?;
    Ok(())
}

#[cfg(unix)]
fn replace_file(temporary: &Path, destination: &Path) -> Result<(), AgentError> {
    fs::rename(temporary, destination).map_err(|source| AgentError::Read {
        path: destination.into(),
        source,
    })
}

#[cfg(windows)]
fn replace_file(temporary: &Path, destination: &Path) -> Result<(), AgentError> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };
    let source = temporary
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let destination_path = destination.to_path_buf();
    let destination = destination
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let result = unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if result == 0 {
        Err(AgentError::Read {
            path: destination_path,
            source: io::Error::last_os_error(),
        })
    } else {
        Ok(())
    }
}

use cockpit_git::GitRepository;
use cockpit_protocol::{
    AgentInterfaceManifest, AgentProvider, RepositoryConfig, validate_agent_interface_version,
    validate_protocol_version,
};
use serde::de::DeserializeOwned;
use sha2::{Digest as ShaDigest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectionResult {
    pub provider: AgentProvider,
    pub target: PathBuf,
    pub state: String,
    pub current_digest: Option<String>,
    pub conflict: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdapterPlan {
    pub provider: AgentProvider,
    pub target: PathBuf,
    pub repository_id: String,
    pub current_digest: Option<String>,
    pub conflict: Option<String>,
    pub executable: bool,
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
    let config: RepositoryConfig = toml::from_str(&fs::read_to_string(&config_path).map_err(
        |source| AgentError::Read {
            path: config_path.clone(),
            source,
        },
    )?)
    .map_err(|error| AgentError::State {
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
        let target = provider_target(&context.root, &provider);
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
    let target = provider_target(&context.root, &provider);
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

pub fn sha256_file(path: &Path) -> Result<String, AgentError> {
    reject_symlink(path)?;
    let metadata = fs::metadata(path).map_err(|source| AgentError::Read {
        path: path.into(),
        source,
    })?;
    if metadata.len() > MAX_AGENT_METADATA_BYTES {
        return Err(AgentError::State {
            path: path.into(),
            message: format!("file exceeds {MAX_AGENT_METADATA_BYTES} byte bound"),
        });
    }
    let mut file = File::open(path).map_err(|source| AgentError::Read {
        path: path.into(),
        source,
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let count = file.read(&mut buffer).map_err(|source| AgentError::Read {
            path: path.into(),
            source,
        })?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("sha256:{}", hex::encode(hasher.finalize())))
}

fn read_bounded_json<T: DeserializeOwned>(path: &Path) -> Result<T, AgentError> {
    reject_symlink(path)?;
    let metadata = fs::metadata(path).map_err(|source| AgentError::Read {
        path: path.into(),
        source,
    })?;
    if metadata.len() > MAX_AGENT_METADATA_BYTES {
        return Err(AgentError::State {
            path: path.into(),
            message: format!("file exceeds {MAX_AGENT_METADATA_BYTES} byte bound"),
        });
    }
    let bytes = fs::read(path).map_err(|source| AgentError::Read {
        path: path.into(),
        source,
    })?;
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
    if metadata.file_type().is_symlink() {
        return Err(AgentError::State {
            path: path.into(),
            message: "symlink/reparse target is not an accepted repository surface".into(),
        });
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

fn provider_target(root: &Path, provider: &AgentProvider) -> PathBuf {
    match provider {
        AgentProvider::GenericAgentsMd | AgentProvider::Codex => root.join("AGENTS.md"),
        AgentProvider::Claude => root.join("CLAUDE.md"),
        AgentProvider::Gemini => root.join("GEMINI.md"),
        AgentProvider::Cursor => root.join(".cursor/rules/ai-cockpit.md"),
    }
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
        (state.into(), Some(sha256_file(&target)?), conflict)
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
    let metadata = fs::metadata(path).map_err(|source| AgentError::Read {
        path: path.into(),
        source,
    })?;
    if metadata.len() > MAX_AGENT_METADATA_BYTES {
        return Err(AgentError::State {
            path: path.into(),
            message: format!("file exceeds {MAX_AGENT_METADATA_BYTES} byte bound"),
        });
    }
    fs::read(path).map_err(|source| AgentError::Read {
        path: path.into(),
        source,
    })
}

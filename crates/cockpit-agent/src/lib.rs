use cockpit_git::GitRepository;
use cockpit_protocol::{
    AgentInterfaceManifest, RepositoryConfig, validate_agent_interface_version,
    validate_protocol_version,
};
use serde::de::DeserializeOwned;
use sha2::{Digest as ShaDigest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use thiserror::Error;

const MAX_AGENT_METADATA_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentRepositoryContext {
    pub root: PathBuf,
    pub manifest_path: PathBuf,
    pub repository_id: String,
    pub manifest: AgentInterfaceManifest,
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

use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions as CapOpenOptions};
use cockpit_core::{
    ActionKind, AuthorityState, DecisionState, Digest, EvidenceState, GovernanceDecision,
    GovernanceInput, evaluate,
};
use cockpit_git::{ChangeContentState, ChangeKind, RepositorySnapshot};
use cockpit_protocol::{QualityCommand, RepositoryConfig, validate_protocol_version};
use serde::{Deserialize, Serialize};
use sha2::{Digest as ShaDigest, Sha256};
use std::collections::BTreeMap;
#[cfg(unix)]
use std::collections::BTreeSet;
#[cfg(unix)]
use std::ffi::OsStr;
use std::fs;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use thiserror::Error;

static NEXT_ATOMIC_WRITE_ID: AtomicU64 = AtomicU64::new(0);
const MAX_RECEIPT_INDEX_BYTES: u64 = 8 * 1024 * 1024;
const MAX_REUSABLE_RECEIPT_BYTES: u64 = 1024 * 1024;
const MAX_VERIFICATION_IDENTITY_FILE_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LanguageSignal {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildSystem {
    Cargo,
    Npm,
    Poetry,
    Go,
    Make,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryObservation {
    pub snapshot_digest: Digest,
    pub dependency_fingerprint: Digest,
    pub languages: Vec<LanguageSignal>,
    pub build_systems: Vec<BuildSystem>,
    pub test_roots: Vec<String>,
    pub quality_commands: Vec<QualityCommand>,
    pub ci_surfaces: Vec<String>,
    pub critical_domains: Vec<String>,
    pub files_read: usize,
    #[serde(default)]
    pub cache_hit: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvolutionClass {
    L0,
    L1,
    L2,
    L3,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvolutionEvent {
    pub class: EvolutionClass,
    pub event_type: String,
    pub path: String,
    pub action: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileUpdateProposal {
    pub from_profile_version: u64,
    pub candidate: String,
    pub reason: String,
    pub requires_human_confirmation: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AttachedProfile {
    pub profile_version: u64,
    pub repository_id: String,
    pub state: String,
    #[serde(default)]
    pub profile_digest: Option<Digest>,
    pub tests: Vec<QualityCommand>,
    pub build_systems: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationContextInput {
    pub program: String,
    pub args: Vec<String>,
    pub command_digest: String,
    pub scope: Vec<String>,
    pub stage: String,
    pub runner: String,
    pub runtime_digest: String,
    pub base_commit: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerificationReuseAssessment {
    Authorized(Box<VerificationReuseAuthorization>),
    Denied { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationReuseAuthorization {
    pub context: cockpit_evidence::EvidenceContext,
    config_file_digest: String,
    profile_file_digest: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct VerificationIdentityCost {
    files_read: usize,
    files_hashed: usize,
}

struct ResolvedExecutableIdentity {
    path: String,
    digest: String,
    components: Vec<ExecutableComponent>,
    #[cfg(unix)]
    _staging: tempfile::TempDir,
}

struct ExecutableComponent {
    path: String,
    #[cfg(unix)]
    execution_path: String,
    digest: String,
    #[cfg(unix)]
    first_line: Option<Vec<u8>>,
    _file: fs::File,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReceiptStoreBinding {
    pub repository_id: String,
    pub profile_digest: String,
    pub node_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReceiptStoreLoad {
    Candidate {
        receipt: Box<cockpit_evidence::ReusableReceipt>,
        files_read: usize,
    },
    Unavailable {
        reason: String,
        files_read: usize,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReceiptStoreWrite {
    pub files_read: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepositoryVerificationPolicy {
    ProfileAuthorized,
    NeverReuse,
    Protected(cockpit_verification::ProtectedGateClass),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryVerificationRequest {
    pub node_id: String,
    pub program: String,
    pub args: Vec<String>,
    pub scope: Vec<String>,
    pub stage: String,
    pub runner: String,
    pub runtime_digest: String,
    pub base_commit: Option<String>,
    pub workers: usize,
    pub policy: RepositoryVerificationPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryVerificationRun {
    pub receipt: cockpit_verification::VerificationReceipt,
    pub final_snapshot: RepositorySnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReceiptStoreIndex {
    schema_version: u32,
    repository_id: String,
    profile_digest: String,
    receipts: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryStatus {
    pub protocol_version: u32,
    pub repository_id: String,
    pub state: String,
    pub profile_version: u64,
    pub active_work_items: usize,
    pub archived_work_items: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GovernanceSignalAssessment {
    pub untrusted_material: bool,
    pub test_weakening: bool,
    pub coverage_weakening: bool,
    pub unknowns: Vec<String>,
    pub findings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleReceipt {
    pub work_item_id: String,
    pub state: String,
    pub timestamp: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkItemStartOptions {
    pub out_of_scope: Vec<String>,
    pub risk: String,
    pub authority: String,
    pub acceptance_criteria: Vec<String>,
    pub required_evidence_classes: Vec<String>,
}

#[derive(Debug, Error)]
pub enum ObserverError {
    #[error("failed to read repository entry {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("repository snapshot root does not match observer root")]
    SnapshotRootMismatch,
    #[error("repository protocol state error at {path}: {message}")]
    State { path: PathBuf, message: String },
}

pub fn repository_id(root: &Path) -> Digest {
    Digest::sha256_bytes(root.to_string_lossy().as_bytes())
}

pub fn assess_verification_reuse(
    root: &Path,
    snapshot: &RepositorySnapshot,
    input: &VerificationContextInput,
) -> Result<VerificationReuseAssessment, ObserverError> {
    assess_verification_reuse_measured(
        root,
        snapshot,
        input,
        None,
        &mut VerificationIdentityCost::default(),
    )
}

fn assess_verification_reuse_measured(
    root: &Path,
    snapshot: &RepositorySnapshot,
    input: &VerificationContextInput,
    pre_resolved: Option<&ResolvedExecutableIdentity>,
    cost: &mut VerificationIdentityCost,
) -> Result<VerificationReuseAssessment, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let snapshot_root = fs::canonicalize(&snapshot.root).map_err(|source| ObserverError::Read {
        path: snapshot.root.clone(),
        source,
    })?;
    if root != snapshot_root {
        return Err(ObserverError::SnapshotRootMismatch);
    }

    let Some(head) = snapshot
        .head
        .as_ref()
        .filter(|head| valid_git_object_id(head))
    else {
        return Ok(denied_reuse("source_revision_unknown"));
    };
    if input.program.is_empty()
        || input.scope.is_empty()
        || !matches!(input.stage.as_str(), "task" | "pr" | "release")
        || !matches!(input.runner.as_str(), "local" | "hosted")
    {
        return Ok(denied_reuse("verification_context_invalid"));
    }
    let base_commit = if input.stage == "task" {
        head.clone()
    } else {
        let Some(base) = input
            .base_commit
            .as_ref()
            .filter(|base| valid_git_object_id(base))
        else {
            return Ok(denied_reuse("base_revision_unknown"));
        };
        base.clone()
    };
    let owned_executable_identity;
    let executable_identity = if let Some(identity) = pre_resolved {
        identity
    } else {
        let Some(identity) = resolved_executable_identity(&root, &input.program) else {
            return Ok(denied_reuse("toolchain_identity_unknown"));
        };
        owned_executable_identity = identity;
        &owned_executable_identity
    };
    if pre_resolved.is_none() {
        cost.files_read = cost
            .files_read
            .saturating_add(executable_identity.components.len());
        cost.files_hashed = cost
            .files_hashed
            .saturating_add(executable_identity.components.len());
    }
    let execution_environment = execution_environment_digest(&snapshot.root)?;

    let ai = root.join(".ai");
    let config_path = ai.join("cockpit.toml");
    let profile_path = ai.join("project.json");
    let (config_bytes, profile_bytes) = read_verification_identity_files(&root)?;
    cost.files_read = cost.files_read.saturating_add(2);
    cost.files_hashed = cost.files_hashed.saturating_add(2);
    let config_text = std::str::from_utf8(&config_bytes).map_err(|error| ObserverError::State {
        path: config_path.clone(),
        message: error.to_string(),
    })?;
    let config: RepositoryConfig =
        toml::from_str(config_text).map_err(|error| ObserverError::State {
            path: config_path.clone(),
            message: error.to_string(),
        })?;
    validate_protocol_version(config.protocol_version).map_err(|error| ObserverError::State {
        path: config_path,
        message: error.to_string(),
    })?;
    let expected_repository_id = repository_id(&root).to_string();
    if config.repository_id != expected_repository_id {
        return Ok(denied_reuse("repository_identity_mismatch"));
    }

    let profile: AttachedProfile =
        match serde_json::from_slice(&profile_bytes).map_err(|error| ObserverError::State {
            path: profile_path.clone(),
            message: error.to_string(),
        }) {
            Ok(profile) => profile,
            Err(_) => return Ok(denied_reuse("profile_untrusted")),
        };
    if profile.repository_id != expected_repository_id
        || profile.repository_id != config.repository_id
    {
        return Ok(denied_reuse("repository_identity_mismatch"));
    }
    let Some(stored_profile_digest) = profile.profile_digest.as_ref() else {
        return Ok(denied_reuse("profile_digest_missing"));
    };
    let computed_profile_digest = digest_value(
        &cockpit_protocol::ProjectProfile {
            profile_version: profile.profile_version,
            repository_id: profile.repository_id.clone(),
            tests: profile.tests.clone(),
            build_systems: profile.build_systems.clone(),
        },
        &profile_path,
    )?;
    if stored_profile_digest.to_string() != computed_profile_digest {
        return Ok(denied_reuse("profile_digest_mismatch"));
    }
    if profile.state != "calibrated" {
        return Ok(denied_reuse("profile_not_calibrated"));
    }
    if !profile.tests.iter().any(|command| {
        command.program == input.program
            && command.args == input.args
            && command.state == "verified"
    }) {
        return Ok(denied_reuse("command_not_profile_verified"));
    }

    let mut changed_paths: Vec<String> = snapshot
        .changed_paths
        .iter()
        .map(|path| path.replace('\\', "/"))
        .filter(|path| path != ".ai" && !path.starts_with(".ai/"))
        .collect();
    changed_paths.sort();
    changed_paths.dedup();
    let mut scope = input.scope.clone();
    scope.sort();
    scope.dedup();

    let context = cockpit_evidence::EvidenceContext {
        content_digest: digest_value(
            &(snapshot.tree_digest.as_str(), snapshot.diff_digest.as_str()),
            &snapshot.root,
        )?,
        diff: cockpit_evidence::DiffIdentity {
            base_commit,
            head_commit: head.clone(),
            changed_paths_digest: digest_value(&changed_paths, &snapshot.root)?,
        },
        environment_digest: digest_value(
            &(
                input.runtime_digest.as_str(),
                std::env::consts::OS,
                std::env::consts::ARCH,
                input.runner.as_str(),
                execution_environment.as_str(),
            ),
            &snapshot.root,
        )?,
        command_digest: input.command_digest.clone(),
        scope_digest: digest_value(&scope, &snapshot.root)?,
        governance_digest: digest_value(
            &(config.protocol_version, config.repository_id.as_str()),
            &snapshot.root,
        )?,
        toolchain_digest: digest_value(
            &(
                input.program.as_str(),
                &input.args,
                executable_identity.path.as_str(),
                executable_identity.digest.as_str(),
                snapshot.dependency_fingerprint.as_str(),
            ),
            &snapshot.root,
        )?,
        policy_digest: digest_value(
            &(
                "verification-reuse-v1",
                input.stage.as_str(),
                profile.state.as_str(),
            ),
            &snapshot.root,
        )?,
        profile_digest: stored_profile_digest.to_string(),
        stage: input.stage.clone(),
        runner: input.runner.clone(),
    };
    if context.validate().is_err() {
        return Ok(denied_reuse("verification_context_invalid"));
    }
    Ok(VerificationReuseAssessment::Authorized(Box::new(
        VerificationReuseAuthorization {
            context,
            config_file_digest: digest_raw_bytes(&config_bytes),
            profile_file_digest: digest_raw_bytes(&profile_bytes),
        },
    )))
}

fn refresh_verification_context(
    root: &Path,
    snapshot: &RepositorySnapshot,
    input: &VerificationContextInput,
    executable_identity: Option<&ResolvedExecutableIdentity>,
    authorization: &VerificationReuseAuthorization,
    cost: &mut VerificationIdentityCost,
) -> Result<Option<cockpit_evidence::EvidenceContext>, ObserverError> {
    let snapshot_root = fs::canonicalize(&snapshot.root).map_err(|source| ObserverError::Read {
        path: snapshot.root.clone(),
        source,
    })?;
    if root != snapshot_root {
        return Err(ObserverError::SnapshotRootMismatch);
    }
    let Some(head) = snapshot
        .head
        .as_ref()
        .filter(|head| valid_git_object_id(head))
    else {
        return Ok(None);
    };
    let base_commit = if input.stage == "task" {
        head.clone()
    } else {
        let Some(base) = input
            .base_commit
            .as_ref()
            .filter(|base| valid_git_object_id(base))
        else {
            return Ok(None);
        };
        base.clone()
    };
    let Some(executable_identity) = executable_identity else {
        return Ok(None);
    };
    let execution_environment = execution_environment_digest(&snapshot.root)?;
    let mut changed_paths = snapshot
        .changed_paths
        .iter()
        .map(|path| path.replace('\\', "/"))
        .filter(|path| path != ".ai" && !path.starts_with(".ai/"))
        .collect::<Vec<_>>();
    changed_paths.sort();
    changed_paths.dedup();
    let mut scope = input.scope.clone();
    scope.sort();
    scope.dedup();
    let Ok((config_bytes, profile_bytes)) = read_verification_identity_files(root) else {
        return Ok(None);
    };
    cost.files_read = cost.files_read.saturating_add(2);
    cost.files_hashed = cost.files_hashed.saturating_add(2);
    if digest_raw_bytes(&config_bytes) != authorization.config_file_digest
        || digest_raw_bytes(&profile_bytes) != authorization.profile_file_digest
    {
        return Ok(None);
    }
    let mut context = authorization.context.clone();
    context.content_digest = digest_value(
        &(snapshot.tree_digest.as_str(), snapshot.diff_digest.as_str()),
        &snapshot.root,
    )?;
    context.diff = cockpit_evidence::DiffIdentity {
        base_commit,
        head_commit: head.clone(),
        changed_paths_digest: digest_value(&changed_paths, &snapshot.root)?,
    };
    context.environment_digest = digest_value(
        &(
            input.runtime_digest.as_str(),
            std::env::consts::OS,
            std::env::consts::ARCH,
            input.runner.as_str(),
            execution_environment.as_str(),
        ),
        &snapshot.root,
    )?;
    context.command_digest = input.command_digest.clone();
    context.scope_digest = digest_value(&scope, &snapshot.root)?;
    context.toolchain_digest = digest_value(
        &(
            input.program.as_str(),
            &input.args,
            executable_identity.path.as_str(),
            executable_identity.digest.as_str(),
            snapshot.dependency_fingerprint.as_str(),
        ),
        &snapshot.root,
    )?;
    context.stage = input.stage.clone();
    context.runner = input.runner.clone();
    Ok(context.validate().is_ok().then_some(context))
}

fn denied_reuse(reason: &str) -> VerificationReuseAssessment {
    VerificationReuseAssessment::Denied {
        reason: reason.into(),
    }
}

fn valid_git_object_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn digest_value<T: Serialize>(value: &T, path: &Path) -> Result<String, ObserverError> {
    cockpit_protocol::digest_json(value)
        .map(|digest| digest.to_string())
        .map_err(|error| ObserverError::State {
            path: path.to_path_buf(),
            message: error.to_string(),
        })
}

fn digest_raw_bytes(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

fn read_verification_identity_files(root: &Path) -> Result<(Vec<u8>, Vec<u8>), ObserverError> {
    let root_dir = Dir::open_ambient_dir(root, cap_std::ambient_authority()).map_err(|source| {
        ObserverError::Read {
            path: root.to_path_buf(),
            source,
        }
    })?;
    let ai_path = root.join(".ai");
    let ai = open_cap_directory_nofollow_strict(&root_dir, ".ai", &ai_path)?;
    let config_path = ai_path.join("cockpit.toml");
    let profile_path = ai_path.join("project.json");
    Ok((
        read_cap_file_nofollow_bounded(
            &ai,
            "cockpit.toml",
            &config_path,
            MAX_VERIFICATION_IDENTITY_FILE_BYTES,
        )?,
        read_cap_file_nofollow_bounded(
            &ai,
            "project.json",
            &profile_path,
            MAX_VERIFICATION_IDENTITY_FILE_BYTES,
        )?,
    ))
}

fn resolved_executable_identity(root: &Path, program: &str) -> Option<ResolvedExecutableIdentity> {
    let executable = resolve_executable(root, program)?;
    #[cfg(windows)]
    if executable
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("bat") || extension.eq_ignore_ascii_case("cmd")
        })
    {
        // Command delegates batch files through cmd.exe, whose identity (and any
        // further runtime selected by the batch file) is not represented here.
        return None;
    }
    #[cfg(unix)]
    let staging = tempfile::Builder::new()
        .prefix("ai-cockpit-executable-")
        .tempdir()
        .ok()?;
    #[cfg(unix)]
    let staging_path = Some(staging.path());
    #[cfg(not(unix))]
    let staging_path = None;
    let mut components = Vec::new();
    collect_executable_identities(root, &executable, &mut components, staging_path, 0)?;
    #[cfg(unix)]
    if !supports_pinned_execution(&components) {
        return None;
    }
    let identities = components
        .iter()
        .map(|component| (&component.path, &component.digest))
        .collect::<Vec<_>>();
    let identity_bytes = serde_json::to_vec(&identities).ok()?;
    Some(ResolvedExecutableIdentity {
        path: executable.to_string_lossy().into_owned(),
        digest: digest_raw_bytes(&identity_bytes),
        components,
        #[cfg(unix)]
        _staging: staging,
    })
}

#[cfg(unix)]
fn supports_pinned_execution(components: &[ExecutableComponent]) -> bool {
    let Some(primary) = components.first() else {
        return false;
    };
    let Some(first_line) = primary.first_line.as_deref() else {
        return true;
    };
    let Some(shebang) = first_line.strip_prefix(b"#!") else {
        return true;
    };
    let Ok(shebang) = std::str::from_utf8(shebang) else {
        return false;
    };
    let words = shebang.split_whitespace().collect::<Vec<_>>();
    let Some(interpreter) = components.get(1) else {
        return false;
    };
    let is_env = Path::new(&interpreter.path)
        .file_name()
        .is_some_and(|name| name == "env" || name == "env.exe");
    let effective_index = if is_env { 2 } else { 1 };
    !components
        .get(effective_index)
        .and_then(|component| component.first_line.as_deref())
        .is_some_and(|line| line.starts_with(b"#!"))
        && (!is_env || words.len() >= 2)
}

impl ResolvedExecutableIdentity {
    #[cfg(target_os = "macos")]
    fn execution_environment(&self) -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
        let mut library_paths = self
            .components
            .iter()
            .filter_map(|component| Path::new(&component.path).parent())
            .flat_map(|parent| {
                let prefix = parent.parent().unwrap_or(parent);
                [
                    parent.join("lib"),
                    prefix.join("lib"),
                    prefix.join("Frameworks"),
                ]
            })
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
        if let Some(existing) = std::env::var_os("DYLD_LIBRARY_PATH") {
            library_paths.extend(std::env::split_paths(&existing));
        }
        library_paths.sort();
        library_paths.dedup();
        std::env::join_paths(library_paths)
            .ok()
            .map(|value| vec![("DYLD_LIBRARY_PATH".into(), value)])
            .unwrap_or_default()
    }

    #[cfg(not(target_os = "macos"))]
    fn execution_environment(&self) -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
        Vec::new()
    }

    fn execution(&self, original_args: &[String]) -> Option<(String, Vec<String>)> {
        #[cfg(windows)]
        {
            return Some((self.path.clone(), original_args.to_vec()));
        }
        #[cfg(unix)]
        {
            let primary = self.components.first()?;
            let primary_path = primary.execution_path.clone();
            let Some(first_line) = primary.first_line.as_deref() else {
                return Some((primary_path, original_args.to_vec()));
            };
            let Some(shebang) = first_line.strip_prefix(b"#!") else {
                return Some((primary_path, original_args.to_vec()));
            };
            let shebang = std::str::from_utf8(shebang).ok()?.trim();
            let words = shebang.split_whitespace().collect::<Vec<_>>();
            let interpreter = self.components.get(1)?;
            let is_env = Path::new(&interpreter.path)
                .file_name()
                .is_some_and(|name| name == "env" || name == "env.exe");
            let executable_index = if is_env { 2 } else { 1 };
            self.components.get(executable_index)?;
            let mut args = Vec::new();
            if !is_env && words.len() > 1 {
                args.push(words[1..].join(" "));
            }
            args.push(primary_path);
            args.extend_from_slice(original_args);
            Some((
                self.components[executable_index].execution_path.clone(),
                args,
            ))
        }
    }
}

fn build_repository_verification_command(
    root: &Path,
    request: &RepositoryVerificationRequest,
    execution_identity: Option<&ResolvedExecutableIdentity>,
    policy: cockpit_verification::VerificationReusePolicy,
) -> cockpit_verification::VerificationCommand {
    let (execution_program, execution_args) = execution_identity
        .and_then(|identity| identity.execution(&request.args))
        .unwrap_or_else(|| (request.program.clone(), request.args.clone()));
    let command = if execution_identity.is_some() {
        cockpit_verification::VerificationCommand::new_pinned(
            &request.node_id,
            &execution_program,
            execution_args,
            &request.program,
            request.args.clone(),
            policy,
        )
    } else {
        cockpit_verification::VerificationCommand::new(
            &request.node_id,
            &execution_program,
            execution_args,
            policy,
        )
    };
    command.with_current_dir(root).with_environment(
        execution_identity.map_or_else(Vec::new, ResolvedExecutableIdentity::execution_environment),
    )
}

fn resolve_executable(root: &Path, program: &str) -> Option<PathBuf> {
    let requested = Path::new(program);
    let executable = if requested.is_absolute() {
        resolve_platform_executable(requested)?
    } else if requested.components().count() > 1 {
        resolve_platform_executable(&root.join(requested))?
    } else {
        std::env::var_os("PATH")
            .into_iter()
            .flat_map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
            .find_map(|directory| resolve_platform_executable(&directory.join(requested)))?
    };
    let executable_name = executable.file_name()?.to_owned();
    let executable_parent = fs::canonicalize(executable.parent()?).ok()?;
    let executable = executable_parent.join(executable_name);
    if !is_executable_file(&executable) {
        return None;
    }
    Some(executable)
}

#[cfg(not(windows))]
fn resolve_platform_executable(requested: &Path) -> Option<PathBuf> {
    is_executable_file(requested).then(|| requested.to_path_buf())
}

#[cfg(windows)]
fn resolve_platform_executable(requested: &Path) -> Option<PathBuf> {
    if requested.extension().is_some() && is_executable_file(requested) {
        return Some(requested.to_path_buf());
    }
    let path_ext = std::env::var_os("PATHEXT").unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".into());
    path_ext
        .to_string_lossy()
        .split(';')
        .filter(|extension| !extension.is_empty())
        .map(|extension| {
            let extension = extension.trim_start_matches('.');
            requested.with_extension(extension)
        })
        .find(|candidate| is_executable_file(candidate))
        .or_else(|| is_executable_file(requested).then(|| requested.to_path_buf()))
}

fn collect_executable_identities(
    root: &Path,
    executable: &Path,
    components: &mut Vec<ExecutableComponent>,
    staging: Option<&Path>,
    depth: usize,
) -> Option<()> {
    if depth > 4 {
        return None;
    }
    let key = executable.to_string_lossy().into_owned();
    if components.iter().any(|component| component.path == key) {
        return None;
    }
    let file = open_pinned_executable(executable)?;
    let (mut file, resolved_execution_path) =
        if staging.is_some_and(|_| should_stage_executable(executable)) {
            stage_pinned_executable(file, executable, staging?, components.len())?
        } else {
            (file, executable.to_string_lossy().into_owned())
        };
    #[cfg(not(unix))]
    let _ = &resolved_execution_path;
    let (digest, first_line) = hash_executable_and_first_line(&mut file)?;
    components.push(ExecutableComponent {
        path: key,
        #[cfg(unix)]
        execution_path: resolved_execution_path,
        digest,
        #[cfg(unix)]
        first_line: first_line.clone(),
        _file: file,
    });
    let Some(first_line) = first_line.as_deref() else {
        return Some(());
    };
    let Some(shebang) = first_line.strip_prefix(b"#!") else {
        return Some(());
    };
    let shebang = std::str::from_utf8(shebang).ok()?.trim();
    if shebang.is_empty() {
        return Some(());
    }
    let words = shebang.split_whitespace().collect::<Vec<_>>();
    let interpreter_program = *words.first()?;
    let interpreter = resolve_executable(root, interpreter_program)?;
    collect_executable_identities(root, &interpreter, components, staging, depth + 1)?;
    if interpreter
        .file_name()
        .is_some_and(|name| name == "env" || name == "env.exe")
        && let Some(command) = simple_env_command(&words[1..])
    {
        let effective = resolve_executable(root, command)?;
        collect_executable_identities(root, &effective, components, staging, depth + 1)?;
    } else if interpreter
        .file_name()
        .is_some_and(|name| name == "env" || name == "env.exe")
    {
        return None;
    }
    Some(())
}

fn stage_pinned_executable(
    mut source: fs::File,
    original: &Path,
    staging: &Path,
    index: usize,
) -> Option<(fs::File, String)> {
    let original_parent = original.parent()?;
    let prefix = original_parent.parent()?;
    let logical_relative = original.strip_prefix(prefix).ok()?;
    let canonical_target = fs::canonicalize(original).ok()?;
    let target_relative = canonical_target.strip_prefix(prefix).ok()?;
    let component_root = staging.join(format!("component-{index}"));
    let staged_target = component_root.join(target_relative);
    fs::create_dir_all(staged_target.parent()?).ok()?;
    let mut destination = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&staged_target)
        .ok()?;
    source.seek(std::io::SeekFrom::Start(0)).ok()?;
    std::io::copy(&mut source, &mut destination).ok()?;
    destination.sync_all().ok()?;
    fs::set_permissions(&staged_target, source.metadata().ok()?.permissions()).ok()?;
    // Some Linux filesystems reject execve while a writable descriptor for
    // the executable is still open (ETXTBSY). Close the copy-on-write handle
    // and retain only a read-only pin for the staged bytes.
    drop(destination);
    let pinned = fs::File::open(&staged_target).ok()?;
    let staged_logical = component_root.join(logical_relative);
    if staged_logical != staged_target {
        fs::create_dir_all(staged_logical.parent()?).ok()?;
        #[cfg(unix)]
        std::os::unix::fs::symlink(
            relative_path(staged_logical.parent()?, &staged_target)?,
            &staged_logical,
        )
        .ok()?;
    }
    #[cfg(unix)]
    mirror_relative_runtime_layout(
        prefix,
        &component_root,
        logical_relative.parent()?,
        target_relative.parent()?,
    )?;
    let execution_path = staged_logical.to_string_lossy().into_owned();
    Some((pinned, execution_path))
}

#[cfg(unix)]
fn mirror_relative_runtime_layout(
    prefix: &Path,
    component_root: &Path,
    logical_parent: &Path,
    target_parent: &Path,
) -> Option<()> {
    use std::os::unix::fs::symlink;

    let mut directories = BTreeSet::from([PathBuf::new()]);
    for parent in [logical_parent, target_parent] {
        let mut relative = PathBuf::new();
        for component in parent.components() {
            relative.push(component.as_os_str());
            directories.insert(relative.clone());
        }
    }
    for relative in directories {
        let original_directory = prefix.join(&relative);
        let staged_directory = component_root.join(&relative);
        for entry in fs::read_dir(original_directory).ok()?.flatten() {
            let destination = staged_directory.join(entry.file_name());
            if fs::symlink_metadata(&destination).is_ok() {
                continue;
            }
            symlink(entry.path(), destination).ok()?;
        }
    }
    Some(())
}

#[cfg(unix)]
fn relative_path(from: &Path, to: &Path) -> Option<PathBuf> {
    let from = from.components().collect::<Vec<_>>();
    let to = to.components().collect::<Vec<_>>();
    let common = from
        .iter()
        .zip(&to)
        .take_while(|(left, right)| left == right)
        .count();
    let mut relative = PathBuf::new();
    for _ in common..from.len() {
        relative.push(OsStr::new(".."));
    }
    for component in &to[common..] {
        relative.push(component.as_os_str());
    }
    (!relative.as_os_str().is_empty()).then_some(relative)
}

#[cfg(target_os = "macos")]
fn should_stage_executable(executable: &Path) -> bool {
    ![
        Path::new("/System"),
        Path::new("/usr/bin"),
        Path::new("/usr/sbin"),
        Path::new("/bin"),
        Path::new("/sbin"),
    ]
    .iter()
    .any(|protected| executable.starts_with(protected))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn should_stage_executable(_executable: &Path) -> bool {
    true
}

#[cfg(not(unix))]
fn should_stage_executable(_executable: &Path) -> bool {
    false
}

fn simple_env_command<'a>(arguments: &'a [&'a str]) -> Option<&'a str> {
    let command = *arguments.first()?;
    (!command.starts_with('-') && !command.contains('=')).then_some(command)
}

fn hash_executable_and_first_line(file: &mut fs::File) -> Option<(String, Option<Vec<u8>>)> {
    const MAX_SHEBANG_BYTES: usize = 4096;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut first_line = Vec::with_capacity(MAX_SHEBANG_BYTES);
    let mut first_line_complete = false;
    loop {
        let count = file.read(&mut buffer).ok()?;
        if count == 0 {
            break;
        }
        let chunk = &buffer[..count];
        hasher.update(chunk);
        if !first_line_complete && first_line.len() < MAX_SHEBANG_BYTES {
            let remaining = MAX_SHEBANG_BYTES - first_line.len();
            let prefix = &chunk[..chunk.len().min(remaining)];
            if let Some(newline) = prefix.iter().position(|byte| *byte == b'\n') {
                first_line.extend_from_slice(&prefix[..newline]);
                first_line_complete = true;
            } else {
                first_line.extend_from_slice(prefix);
            }
        }
    }
    let first_line = if first_line.starts_with(b"#!") && !first_line_complete {
        return None;
    } else if first_line.is_empty() {
        None
    } else {
        Some(first_line)
    };
    file.seek(std::io::SeekFrom::Start(0)).ok()?;
    Some((
        format!("sha256:{}", hex::encode(hasher.finalize())),
        first_line,
    ))
}

#[cfg(unix)]
fn open_pinned_executable(executable: &Path) -> Option<fs::File> {
    fs::File::open(executable).ok()
}

#[cfg(windows)]
fn open_pinned_executable(executable: &Path) -> Option<fs::File> {
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ,
    };

    let file = fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(executable)
        .ok()?;
    let metadata = file.metadata().ok()?;
    if !metadata.is_file() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return None;
    }
    Some(file)
}

fn is_executable_file(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn execution_environment_digest(path: &Path) -> Result<String, ObserverError> {
    let mut values = std::env::vars_os()
        .map(|(name, value)| {
            (
                name.as_encoded_bytes().to_vec(),
                value.as_encoded_bytes().to_vec(),
            )
        })
        .collect::<Vec<_>>();
    values.sort();
    digest_value(&values, path)
}

pub fn load_reusable_receipt(
    root: &Path,
    binding: &ReceiptStoreBinding,
) -> Result<ReceiptStoreLoad, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    if binding.repository_id != repository_id(&root).to_string() {
        return Ok(unavailable_receipt("repository_identity_mismatch", 0));
    }
    if !valid_sha256_digest(&binding.profile_digest) || !valid_node_id(&binding.node_id) {
        return Ok(unavailable_receipt("store_binding_invalid", 0));
    }

    let root_dir =
        Dir::open_ambient_dir(&root, cap_std::ambient_authority()).map_err(|source| {
            ObserverError::Read {
                path: root.clone(),
                source,
            }
        })?;
    let ai = match open_cap_directory_nofollow(&root_dir, ".ai", &root.join(".ai")) {
        Ok(directory) => directory,
        Err(reason) => return Ok(unavailable_receipt(reason, 0)),
    };
    let evidence = match open_cap_directory_nofollow(&ai, "evidence", &root.join(".ai/evidence")) {
        Ok(directory) => directory,
        Err(reason) => return Ok(unavailable_receipt(reason, 0)),
    };
    let reuse_path = root.join(".ai/evidence/reuse");
    let reuse = match open_cap_directory_nofollow(&evidence, "reuse", &reuse_path) {
        Ok(directory) => directory,
        Err(reason) => return Ok(unavailable_receipt(reason, 0)),
    };
    let lock_path = reuse_path.join("index.lock");
    let lock = match open_cap_existing_nofollow(&reuse, "index.lock", &lock_path) {
        Ok(lock) => lock,
        Err(_) => return Ok(unavailable_receipt("index_invalid", 0)),
    };
    if lock.lock_shared().is_err() {
        return Ok(unavailable_receipt("index_unreadable", 0));
    }
    #[cfg(windows)]
    match read_optional_cap_file_nofollow_bounded(
        &reuse,
        "index.pending",
        &reuse_path.join("index.pending"),
        1024,
    ) {
        Ok(Some(_)) => return Ok(unavailable_receipt("index_commit_uncertain", 0)),
        Ok(None) => {}
        Err(ObserverError::State { message, .. }) if message.contains("symlink") => {
            return Ok(unavailable_receipt("symlink_rejected", 0));
        }
        Err(_) => return Ok(unavailable_receipt("index_unreadable", 0)),
    }
    #[cfg(not(windows))]
    match reuse.symlink_metadata("index.pending") {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            return Ok(unavailable_receipt("index_commit_uncertain", 0));
        }
        Ok(_) => return Ok(unavailable_receipt("index_invalid", 0)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => return Ok(unavailable_receipt("index_unreadable", 0)),
    }
    let index_path = reuse_path.join("index.json");
    #[cfg(windows)]
    let index_bytes = match read_optional_cap_file_nofollow_bounded(
        &reuse,
        "index.json",
        &index_path,
        MAX_RECEIPT_INDEX_BYTES,
    ) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Ok(unavailable_receipt("evidence_missing", 0)),
        Err(ObserverError::State { message, .. }) if message.contains("symlink") => {
            return Ok(unavailable_receipt("symlink_rejected", 0));
        }
        Err(_) => return Ok(unavailable_receipt("index_unreadable", 0)),
    };
    #[cfg(not(windows))]
    let index_bytes = {
        let index_metadata = match reuse.symlink_metadata("index.json") {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(unavailable_receipt("evidence_missing", 0));
            }
            Err(_) => return Ok(unavailable_receipt("index_unreadable", 0)),
        };
        if index_metadata.file_type().is_symlink() {
            return Ok(unavailable_receipt("symlink_rejected", 0));
        }
        if !index_metadata.is_file() {
            return Ok(unavailable_receipt("index_invalid", 0));
        }
        match read_cap_file_nofollow_bounded(
            &reuse,
            "index.json",
            &index_path,
            MAX_RECEIPT_INDEX_BYTES,
        ) {
            Ok(bytes) => bytes,
            Err(_) => return Ok(unavailable_receipt("index_unreadable", 0)),
        }
    };
    let index: ReceiptStoreIndex = match serde_json::from_slice(&index_bytes) {
        Ok(index) => index,
        Err(_) => return Ok(unavailable_receipt("index_invalid", 1)),
    };
    if index.schema_version != 1 {
        return Ok(unavailable_receipt("index_invalid", 1));
    }
    if index.repository_id != binding.repository_id {
        return Ok(unavailable_receipt("repository_identity_mismatch", 1));
    }
    if index.profile_digest != binding.profile_digest {
        return Ok(unavailable_receipt("profile_identity_mismatch", 1));
    }
    let Some(receipt_id) = index.receipts.get(&binding.node_id) else {
        return Ok(unavailable_receipt("evidence_missing", 1));
    };
    if !valid_sha256_digest(receipt_id) {
        return Ok(unavailable_receipt("index_invalid", 1));
    }

    let receipts_path = reuse_path.join("receipts");
    let receipts = match open_cap_directory_nofollow(&reuse, "receipts", &receipts_path) {
        Ok(directory) => directory,
        Err(reason) => return Ok(unavailable_receipt(reason, 1)),
    };
    let receipt_name = receipt_file_name(receipt_id);
    let receipt_path = receipts_path.join(&receipt_name);
    #[cfg(windows)]
    let receipt_bytes = match read_optional_cap_file_nofollow_bounded(
        &receipts,
        &receipt_name,
        &receipt_path,
        MAX_REUSABLE_RECEIPT_BYTES,
    ) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Ok(unavailable_receipt("receipt_missing", 1)),
        Err(ObserverError::State { message, .. }) if message.contains("symlink") => {
            return Ok(unavailable_receipt("symlink_rejected", 1));
        }
        Err(_) => return Ok(unavailable_receipt("receipt_unreadable", 1)),
    };
    #[cfg(not(windows))]
    let receipt_bytes = {
        let receipt_metadata = match receipts.symlink_metadata(&receipt_name) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(unavailable_receipt("receipt_missing", 1));
            }
            Err(_) => return Ok(unavailable_receipt("receipt_unreadable", 1)),
        };
        if receipt_metadata.file_type().is_symlink() {
            return Ok(unavailable_receipt("symlink_rejected", 1));
        }
        if !receipt_metadata.is_file() {
            return Ok(unavailable_receipt("receipt_invalid", 1));
        }
        match read_cap_file_nofollow_bounded(
            &receipts,
            &receipt_name,
            &receipt_path,
            MAX_REUSABLE_RECEIPT_BYTES,
        ) {
            Ok(bytes) => bytes,
            Err(_) => return Ok(unavailable_receipt("receipt_unreadable", 1)),
        }
    };
    let receipt: cockpit_evidence::ReusableReceipt = match serde_json::from_slice(&receipt_bytes) {
        Ok(receipt) => receipt,
        Err(_) => return Ok(unavailable_receipt("receipt_invalid", 2)),
    };
    if receipt.validate().is_err()
        || receipt.receipt_id != *receipt_id
        || receipt.node_id != binding.node_id
        || receipt.context.profile_digest != binding.profile_digest
    {
        return Ok(unavailable_receipt("receipt_invalid", 2));
    }
    Ok(ReceiptStoreLoad::Candidate {
        receipt: Box::new(receipt),
        files_read: 2,
    })
}

pub fn persist_reusable_receipt(
    root: &Path,
    binding: &ReceiptStoreBinding,
    receipt: &cockpit_evidence::ReusableReceipt,
) -> Result<ReceiptStoreWrite, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let expected_repository_id = repository_id(&root).to_string();
    if binding.repository_id != expected_repository_id
        || !valid_sha256_digest(&binding.profile_digest)
        || !valid_node_id(&binding.node_id)
        || receipt.validate().is_err()
        || !receipt.passed
        || receipt.node_id != binding.node_id
        || receipt.context.profile_digest != binding.profile_digest
    {
        return Err(ObserverError::State {
            path: root.join(".ai/evidence/reuse"),
            message: "invalid reusable receipt store binding".into(),
        });
    }

    let root_dir =
        Dir::open_ambient_dir(&root, cap_std::ambient_authority()).map_err(|source| {
            ObserverError::Read {
                path: root.clone(),
                source,
            }
        })?;
    let ai_path = root.join(".ai");
    let ai = open_cap_directory_nofollow_strict(&root_dir, ".ai", &ai_path)?;
    let evidence_path = ai_path.join("evidence");
    let evidence = create_and_open_cap_directory(&ai, "evidence", &evidence_path)?;
    let reuse_path = evidence_path.join("reuse");
    let reuse = create_and_open_cap_directory(&evidence, "reuse", &reuse_path)?;
    let receipts_path = reuse_path.join("receipts");
    let receipts = create_and_open_cap_directory(&reuse, "receipts", &receipts_path)?;

    let lock_path = reuse_path.join("index.lock");
    let lock = open_or_create_cap_nofollow(&reuse, "index.lock", &lock_path)?;
    lock.lock().map_err(|source| ObserverError::Read {
        path: lock_path,
        source,
    })?;

    let receipt_name = receipt_file_name(&receipt.receipt_id);
    let receipt_path = receipts_path.join(&receipt_name);
    let receipt_bytes =
        serde_json::to_vec_pretty(receipt).map_err(|error| ObserverError::State {
            path: receipt_path.clone(),
            message: error.to_string(),
        })?;
    let mut files_read =
        write_cap_immutable(&receipts, &receipt_name, &receipt_path, &receipt_bytes)?;

    let index_path = reuse_path.join("index.json");
    let mut index = if let Some(bytes) = read_optional_cap_file_nofollow_bounded(
        &reuse,
        "index.json",
        &index_path,
        MAX_RECEIPT_INDEX_BYTES,
    )? {
        files_read += 1;
        let existing: ReceiptStoreIndex =
            serde_json::from_slice(&bytes).map_err(|error| ObserverError::State {
                path: index_path.clone(),
                message: format!("invalid reusable receipt index: {error}"),
            })?;
        if existing.schema_version != 1 || existing.repository_id != binding.repository_id {
            return Err(ObserverError::State {
                path: index_path,
                message: "reusable receipt index binding mismatch".into(),
            });
        }
        if existing.profile_digest == binding.profile_digest {
            existing
        } else {
            ReceiptStoreIndex {
                schema_version: 1,
                repository_id: binding.repository_id.clone(),
                profile_digest: binding.profile_digest.clone(),
                receipts: BTreeMap::new(),
            }
        }
    } else {
        ReceiptStoreIndex {
            schema_version: 1,
            repository_id: binding.repository_id.clone(),
            profile_digest: binding.profile_digest.clone(),
            receipts: BTreeMap::new(),
        }
    };
    index
        .receipts
        .insert(binding.node_id.clone(), receipt.receipt_id.clone());
    let index_bytes = serde_json::to_vec_pretty(&index).map_err(|error| ObserverError::State {
        path: index_path.clone(),
        message: error.to_string(),
    })?;
    write_cap_commit_marker(&reuse, &reuse_path)?;
    atomic_replace_cap_strict(&reuse, "index.json", &index_path, &index_bytes)?;
    reuse
        .remove_file("index.pending")
        .map_err(|source| ObserverError::Read {
            path: reuse_path.join("index.pending"),
            source,
        })?;
    // The index rename is the logical commit. If syncing marker removal fails,
    // a crash can only resurrect the marker, which makes future reads fail closed.
    let _ = sync_cap_directory(&reuse, &reuse_path.join("index.pending"));
    Ok(ReceiptStoreWrite { files_read })
}

pub fn run_repository_verification(
    root: &Path,
    request: &RepositoryVerificationRequest,
) -> Result<RepositoryVerificationRun, ObserverError> {
    let service_started = Instant::now();
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let mut store_files_read = 0;
    let mut store_unavailable_reason = None;
    let execution_identity = resolved_executable_identity(&root, &request.program);
    let mut executable_files_read = execution_identity
        .as_ref()
        .map_or(0, |identity| identity.components.len());
    let mut executable_files_hashed = executable_files_read;
    let base_command = build_repository_verification_command(
        &root,
        request,
        None,
        match &request.policy {
            RepositoryVerificationPolicy::ProfileAuthorized => {
                cockpit_verification::VerificationReusePolicy::Reusable
            }
            RepositoryVerificationPolicy::NeverReuse => {
                cockpit_verification::VerificationReusePolicy::NeverReuse
            }
            RepositoryVerificationPolicy::Protected(class) => {
                cockpit_verification::VerificationReusePolicy::Protected(class.clone())
            }
        },
    );
    let context_input = VerificationContextInput {
        program: request.program.clone(),
        args: request.args.clone(),
        command_digest: base_command.command_digest(),
        scope: request.scope.clone(),
        stage: request.stage.clone(),
        runner: request.runner.clone(),
        runtime_digest: request.runtime_digest.clone(),
        base_commit: request.base_commit.clone(),
    };
    let mut authorized_binding = None;
    let command = if request.policy == RepositoryVerificationPolicy::ProfileAuthorized {
        let mut assessment_cost = VerificationIdentityCost::default();
        let assessment = assess_verification_reuse_measured(
            &root,
            &snapshot,
            &context_input,
            execution_identity.as_ref(),
            &mut assessment_cost,
        );
        executable_files_read = executable_files_read.saturating_add(assessment_cost.files_read);
        executable_files_hashed =
            executable_files_hashed.saturating_add(assessment_cost.files_hashed);
        match assessment {
            Ok(VerificationReuseAssessment::Authorized(authorization)) => {
                let authorization = *authorization;
                let context = authorization.context.clone();
                let binding = ReceiptStoreBinding {
                    repository_id: repository_id(&root).to_string(),
                    profile_digest: context.profile_digest.clone(),
                    node_id: request.node_id.clone(),
                };
                let candidate = match load_reusable_receipt(&root, &binding)? {
                    ReceiptStoreLoad::Candidate {
                        receipt,
                        files_read,
                    } => {
                        store_files_read += files_read;
                        Some(*receipt)
                    }
                    ReceiptStoreLoad::Unavailable { reason, files_read } => {
                        store_files_read += files_read;
                        store_unavailable_reason = Some(reason);
                        None
                    }
                };
                authorized_binding = Some((binding, authorization));
                build_repository_verification_command(
                    &root,
                    request,
                    execution_identity.as_ref(),
                    cockpit_verification::VerificationReusePolicy::Reusable,
                )
                .with_reuse_candidate(candidate, context)
            }
            Ok(VerificationReuseAssessment::Denied { .. }) | Err(_) => {
                build_repository_verification_command(
                    &root,
                    request,
                    None,
                    cockpit_verification::VerificationReusePolicy::NeverReuse,
                )
            }
        }
    } else {
        base_command
    };
    let mut receipt = cockpit_verification::execute_bounded(vec![command], request.workers)
        .map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    if let Some(reason) = store_unavailable_reason.take()
        && let Some(result) = receipt
            .results
            .iter_mut()
            .find(|result| result.reason == "evidence_missing")
    {
        result.reason = reason;
    }
    let mut final_snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let mut snapshot_git_calls = snapshot.git_calls.saturating_add(final_snapshot.git_calls);
    let mut snapshot_files_read = snapshot
        .files_read
        .saturating_add(final_snapshot.files_read);
    let mut snapshot_files_hashed = snapshot
        .files_hashed
        .saturating_add(final_snapshot.files_hashed);

    let mut post_execution_identity = None;
    let mut post_context = if let Some((_, authorization)) = &authorized_binding {
        post_execution_identity = resolved_executable_identity(&root, &request.program);
        if let Some(identity) = &post_execution_identity {
            executable_files_read = executable_files_read.saturating_add(identity.components.len());
            executable_files_hashed =
                executable_files_hashed.saturating_add(identity.components.len());
        }
        let mut refresh_cost = VerificationIdentityCost::default();
        let refreshed = refresh_verification_context(
            &root,
            &final_snapshot,
            &context_input,
            post_execution_identity.as_ref(),
            authorization,
            &mut refresh_cost,
        )?;
        executable_files_read = executable_files_read.saturating_add(refresh_cost.files_read);
        executable_files_hashed = executable_files_hashed.saturating_add(refresh_cost.files_hashed);
        refreshed
    } else {
        None
    };
    let post_stable = authorized_binding
        .as_ref()
        .zip(post_context.as_ref())
        .is_some_and(|((_, before), after)| before.context == *after);
    if receipt.nodes_reused > 0 && !post_stable {
        let command = if let Some(context) = post_context.clone() {
            let mut authorization = authorized_binding
                .as_ref()
                .expect("post context requires authorization")
                .1
                .clone();
            authorization.context = context.clone();
            authorized_binding = Some((
                ReceiptStoreBinding {
                    repository_id: repository_id(&root).to_string(),
                    profile_digest: context.profile_digest.clone(),
                    node_id: request.node_id.clone(),
                },
                authorization,
            ));
            build_repository_verification_command(
                &root,
                request,
                post_execution_identity.as_ref(),
                cockpit_verification::VerificationReusePolicy::Reusable,
            )
            .with_reuse_candidate(None, context)
        } else {
            authorized_binding = None;
            build_repository_verification_command(
                &root,
                request,
                None,
                cockpit_verification::VerificationReusePolicy::NeverReuse,
            )
        };
        receipt = cockpit_verification::execute_bounded(vec![command], request.workers).map_err(
            |error| ObserverError::State {
                path: root.clone(),
                message: error.to_string(),
            },
        )?;
        if let Some(result) = receipt.results.first_mut() {
            result.reason = "post_planning_binding_drift".into();
        }
        final_snapshot = git.snapshot().map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
        snapshot_git_calls = snapshot_git_calls.saturating_add(final_snapshot.git_calls);
        snapshot_files_read = snapshot_files_read.saturating_add(final_snapshot.files_read);
        snapshot_files_hashed = snapshot_files_hashed.saturating_add(final_snapshot.files_hashed);
        let final_execution_identity = if authorized_binding.is_some() {
            resolved_executable_identity(&root, &request.program)
        } else {
            None
        };
        if let Some(identity) = &final_execution_identity {
            executable_files_read = executable_files_read.saturating_add(identity.components.len());
            executable_files_hashed =
                executable_files_hashed.saturating_add(identity.components.len());
        }
        post_context = if let Some((_, authorization)) = &authorized_binding {
            let mut refresh_cost = VerificationIdentityCost::default();
            let refreshed = refresh_verification_context(
                &root,
                &final_snapshot,
                &context_input,
                final_execution_identity.as_ref(),
                authorization,
                &mut refresh_cost,
            )?;
            executable_files_read = executable_files_read.saturating_add(refresh_cost.files_read);
            executable_files_hashed =
                executable_files_hashed.saturating_add(refresh_cost.files_hashed);
            refreshed
        } else {
            None
        };
    }

    if !receipt.receipt_candidates.is_empty() {
        let stable = authorized_binding
            .as_ref()
            .zip(post_context.as_ref())
            .is_some_and(|((_, before), after)| before.context == *after);
        if stable {
            let (binding, _) = authorized_binding.as_ref().expect("stable binding exists");
            for candidate in &receipt.receipt_candidates {
                store_files_read += persist_reusable_receipt(&root, binding, candidate)?.files_read;
            }
        } else {
            receipt.receipt_candidates.clear();
            for result in &mut receipt.results {
                if !result.reused {
                    result.receipt_id = None;
                    result.reason = "post_execution_binding_drift".into();
                }
            }
        }
    }
    receipt.git_calls = snapshot_git_calls;
    receipt.files_read = receipt
        .files_read
        .saturating_add(snapshot_files_read)
        .saturating_add(store_files_read)
        .saturating_add(executable_files_read);
    receipt.files_hashed = receipt
        .files_hashed
        .saturating_add(snapshot_files_hashed)
        .saturating_add(executable_files_hashed);
    receipt.elapsed_ms = service_started.elapsed().as_millis();
    Ok(RepositoryVerificationRun {
        receipt,
        final_snapshot,
    })
}

fn unavailable_receipt(reason: &str, files_read: usize) -> ReceiptStoreLoad {
    ReceiptStoreLoad::Unavailable {
        reason: reason.into(),
        files_read,
    }
}

fn valid_sha256_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn receipt_file_name(receipt_id: &str) -> String {
    format!("{}.json", &receipt_id["sha256:".len()..])
}

fn valid_node_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn open_cap_directory_nofollow(
    parent: &Dir,
    name: &str,
    _display_path: &Path,
) -> Result<Dir, &'static str> {
    #[cfg(windows)]
    {
        // cap-std's Windows `symlink_metadata` implementation asks
        // `CreateFileAtW` for a zero-access handle.  On the hosted Windows
        // runner that relative metadata probe returns `ERROR_ACCESS_DENIED`
        // even for an ordinary child directory.  Open the directory handle
        // directly instead, then inspect the handle metadata; this also
        // avoids a metadata/open TOCTOU window.
        let directory = match parent.open_dir_nofollow(name) {
            Ok(directory) => directory,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err("evidence_missing");
            }
            Err(_) => return Err("store_path_unreadable"),
        };
        let metadata = directory
            .dir_metadata()
            .map_err(|_| "store_path_unreadable")?;
        if metadata.file_type().is_symlink() {
            return Err("symlink_rejected");
        }
        if !metadata.is_dir() {
            return Err("store_path_invalid");
        }
        return Ok(directory);
    }

    #[cfg(not(windows))]
    match parent.symlink_metadata(name) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err("symlink_rejected"),
        Ok(metadata) if !metadata.is_dir() => Err("store_path_invalid"),
        Ok(_) => parent
            .open_dir_nofollow(name)
            .map_err(|_| "store_path_unreadable"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err("evidence_missing"),
        Err(_) => Err("store_path_unreadable"),
    }
}

fn open_cap_directory_nofollow_strict(
    parent: &Dir,
    name: &str,
    display_path: &Path,
) -> Result<Dir, ObserverError> {
    open_cap_directory_nofollow(parent, name, display_path).map_err(|reason| ObserverError::State {
        path: display_path.to_path_buf(),
        message: format!("receipt store directory rejected: {reason}"),
    })
}

fn create_and_open_cap_directory(
    parent: &Dir,
    name: &str,
    display_path: &Path,
) -> Result<Dir, ObserverError> {
    match parent.create_dir(name) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(source) => {
            return Err(ObserverError::Read {
                path: display_path.to_path_buf(),
                source,
            });
        }
    }
    open_cap_directory_nofollow_strict(parent, name, display_path)
}

fn cap_read_options() -> CapOpenOptions {
    let mut options = CapOpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    options
}

fn open_cap_existing_nofollow(
    parent: &Dir,
    name: &str,
    display_path: &Path,
) -> Result<fs::File, ObserverError> {
    let file = parent
        .open_with(name, &cap_read_options())
        .map_err(|source| ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        })?
        .into_std();
    if !file
        .metadata()
        .map_err(|source| ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        })?
        .is_file()
    {
        return Err(ObserverError::State {
            path: display_path.to_path_buf(),
            message: "receipt store entry must be a real file".into(),
        });
    }
    Ok(file)
}

fn open_or_create_cap_nofollow(
    parent: &Dir,
    name: &str,
    display_path: &Path,
) -> Result<fs::File, ObserverError> {
    let mut existing = CapOpenOptions::new();
    existing.read(true).write(true).follow(FollowSymlinks::No);
    let file = match parent.open_with(name, &existing) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut create = CapOpenOptions::new();
            create
                .read(true)
                .write(true)
                .create_new(true)
                .follow(FollowSymlinks::No);
            match parent.open_with(name, &create) {
                Ok(file) => file,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => parent
                    .open_with(name, &existing)
                    .map_err(|source| ObserverError::Read {
                        path: display_path.to_path_buf(),
                        source,
                    })?,
                Err(source) => {
                    return Err(ObserverError::Read {
                        path: display_path.to_path_buf(),
                        source,
                    });
                }
            }
        }
        Err(source) => {
            return Err(ObserverError::Read {
                path: display_path.to_path_buf(),
                source,
            });
        }
    }
    .into_std();
    if !file
        .metadata()
        .map_err(|source| ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        })?
        .is_file()
    {
        return Err(ObserverError::State {
            path: display_path.to_path_buf(),
            message: "receipt store lock must be a real file".into(),
        });
    }
    Ok(file)
}

fn read_cap_file_nofollow_bounded(
    parent: &Dir,
    name: &str,
    display_path: &Path,
    maximum_bytes: u64,
) -> Result<Vec<u8>, ObserverError> {
    let mut file = open_cap_existing_nofollow(parent, name, display_path)?;
    let mut bytes = Vec::new();
    std::io::Read::by_ref(&mut file)
        .take(maximum_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|source| ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        })?;
    if bytes.len() as u64 > maximum_bytes {
        return Err(ObserverError::State {
            path: display_path.to_path_buf(),
            message: "receipt store entry exceeds the bounded read limit".into(),
        });
    }
    Ok(bytes)
}

fn read_optional_cap_file_nofollow_bounded(
    parent: &Dir,
    name: &str,
    display_path: &Path,
    maximum_bytes: u64,
) -> Result<Option<Vec<u8>>, ObserverError> {
    #[cfg(windows)]
    {
        // See `open_cap_directory_nofollow`: probing a child with
        // `symlink_metadata` is not usable with the Windows capability
        // handle implementation on the hosted runner.  Open once with
        // no-follow semantics and read from that pinned handle.
        //
        // cap-std currently reports `ERROR_ACCESS_DENIED` for a missing leaf
        // when it performs that relative open.  Use the canonical display
        // path only to distinguish the absent case; any existing entry is
        // still opened and read through the capability handle below, so the
        // ambient probe cannot authorize a substituted file.
        match fs::symlink_metadata(display_path) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(source) => {
                return Err(ObserverError::Read {
                    path: display_path.to_path_buf(),
                    source,
                });
            }
        }
        let mut file = match parent.open_with(name, &cap_read_options()) {
            Ok(file) => file.into_std(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(source) => {
                return Err(ObserverError::Read {
                    path: display_path.to_path_buf(),
                    source,
                });
            }
        };
        let metadata = file.metadata().map_err(|source| ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        })?;
        if metadata.file_type().is_symlink() {
            return Err(ObserverError::State {
                path: display_path.to_path_buf(),
                message: "receipt store entry must not be a symlink".into(),
            });
        }
        if !metadata.is_file() {
            return Err(ObserverError::State {
                path: display_path.to_path_buf(),
                message: "receipt store entry must be a real file".into(),
            });
        }
        let mut bytes = Vec::new();
        Read::by_ref(&mut file)
            .take(maximum_bytes.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|source| ObserverError::Read {
                path: display_path.to_path_buf(),
                source,
            })?;
        if bytes.len() as u64 > maximum_bytes {
            return Err(ObserverError::State {
                path: display_path.to_path_buf(),
                message: "receipt store entry exceeds the bounded read limit".into(),
            });
        }
        return Ok(Some(bytes));
    }

    #[cfg(not(windows))]
    match parent.symlink_metadata(name) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(ObserverError::State {
                path: display_path.to_path_buf(),
                message: "receipt store entry must be a real file".into(),
            })
        }
        Ok(_) => {
            read_cap_file_nofollow_bounded(parent, name, display_path, maximum_bytes).map(Some)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        }),
    }
}

fn write_cap_immutable(
    parent: &Dir,
    name: &str,
    display_path: &Path,
    bytes: &[u8],
) -> Result<usize, ObserverError> {
    let sequence = NEXT_ATOMIC_WRITE_ID.fetch_add(1, Ordering::Relaxed);
    let temporary = format!("{name}.receipt-tmp-{}-{sequence}", std::process::id());
    let mut options = CapOpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut file = parent
        .open_with(&temporary, &options)
        .map_err(|source| ObserverError::Read {
            path: display_path.with_file_name(&temporary),
            source,
        })?
        .into_std();
    if let Err(source) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        let _ = parent.remove_file(&temporary);
        return Err(ObserverError::Read {
            path: display_path.with_file_name(&temporary),
            source,
        });
    }
    drop(file);
    let installed = match parent.hard_link(&temporary, parent, name) {
        Ok(()) => Ok(0),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            if read_cap_file_nofollow_bounded(parent, name, display_path, bytes.len() as u64)?
                == bytes
            {
                Ok(1)
            } else {
                Err(ObserverError::State {
                    path: display_path.to_path_buf(),
                    message: "immutable receipt already exists with different content".into(),
                })
            }
        }
        Err(source) => Err(ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        }),
    };
    let _ = parent.remove_file(&temporary);
    let files_read = installed?;
    sync_cap_directory(parent, display_path)?;
    Ok(files_read)
}

fn write_cap_commit_marker(parent: &Dir, display_path: &Path) -> Result<(), ObserverError> {
    let marker_path = display_path.join("index.pending");
    let mut options = CapOpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut marker = parent
        .open_with("index.pending", &options)
        .map_err(|source| ObserverError::Read {
            path: marker_path.clone(),
            source,
        })?
        .into_std();
    marker
        .write_all(b"pending-index-commit-v1\n")
        .and_then(|()| marker.sync_all())
        .map_err(|source| ObserverError::Read {
            path: marker_path.clone(),
            source,
        })?;
    sync_cap_directory(parent, &marker_path)
}

fn atomic_replace_cap_strict(
    parent: &Dir,
    name: &str,
    display_path: &Path,
    bytes: &[u8],
) -> Result<(), ObserverError> {
    if let Ok(metadata) = parent.symlink_metadata(name)
        && (metadata.file_type().is_symlink() || !metadata.is_file())
    {
        return Err(ObserverError::State {
            path: display_path.to_path_buf(),
            message: "receipt store entry must be a real file".into(),
        });
    }
    let sequence = NEXT_ATOMIC_WRITE_ID.fetch_add(1, Ordering::Relaxed);
    let temporary = format!("{name}.tmp-{}-{sequence}", std::process::id());
    let mut options = CapOpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    #[cfg(windows)]
    {
        use cap_std::fs::OpenOptionsExt;
        use windows_sys::Win32::{
            Foundation::{GENERIC_READ, GENERIC_WRITE},
            Storage::FileSystem::DELETE,
        };
        // FileRenameInfoEx requires DELETE access on the source handle.  A
        // write-only temporary file is sufficient on Unix but is rejected by
        // Windows with ERROR_ACCESS_DENIED during the first index publish.
        options.access_mode(GENERIC_READ | GENERIC_WRITE | DELETE);
    }
    let mut file = parent
        .open_with(&temporary, &options)
        .map_err(|source| ObserverError::Read {
            path: display_path.with_file_name(&temporary),
            source,
        })?
        .into_std();
    if let Err(source) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        let _ = parent.remove_file(&temporary);
        return Err(ObserverError::Read {
            path: display_path.with_file_name(&temporary),
            source,
        });
    }
    let replace_result = replace_cap_entry(parent, &file, &temporary, name, display_path);
    drop(file);
    if let Err(source) = replace_result {
        let _ = parent.remove_file(&temporary);
        return Err(ObserverError::Read {
            path: display_path.to_path_buf(),
            source,
        });
    }
    sync_cap_directory(parent, display_path)?;
    Ok(())
}

fn sync_cap_directory(parent: &Dir, display_path: &Path) -> Result<(), ObserverError> {
    #[cfg(windows)]
    {
        let _ = (parent, display_path);
        return Ok(());
    }
    #[cfg(not(windows))]
    parent
        .try_clone()
        .and_then(|directory| directory.into_std_file().sync_all())
        .map_err(|source| ObserverError::Read {
            path: display_path.parent().unwrap_or(display_path).to_path_buf(),
            source,
        })
}

#[cfg(not(windows))]
fn replace_cap_entry(
    parent: &Dir,
    _temporary_file: &std::fs::File,
    temporary: &str,
    name: &str,
    _display_path: &Path,
) -> std::io::Result<()> {
    parent.rename(temporary, parent, name)
}

#[cfg(windows)]
fn replace_cap_entry(
    _parent: &Dir,
    temporary_file: &std::fs::File,
    _temporary: &str,
    _name: &str,
    display_path: &Path,
) -> std::io::Result<()> {
    use std::os::windows::{ffi::OsStrExt, io::AsRawHandle};
    use windows_sys::Win32::{
        Storage::FileSystem::{
            FILE_RENAME_INFO, FILE_RENAME_INFO_0, FileRenameInfoEx, SetFileInformationByHandle,
        },
        System::WindowsProgramming::{
            FILE_RENAME_FLAG_POSIX_SEMANTICS, FILE_RENAME_FLAG_REPLACE_IF_EXISTS,
        },
    };

    // FileRenameInfoEx rejects a relative name paired with RootDirectory on
    // the Windows runners (ERROR_INVALID_PARAMETER). The parent capability
    // remains open with delete sharing disabled, so its absolute path cannot
    // be redirected while this operation is in flight. Keeping the Ex class
    // preserves replace/POSIX semantics for an existing target.
    let display_path = display_path.to_string_lossy();
    // cap-std canonical paths on Windows use the extended-length `\\?\\`
    // prefix. FileRenameInfoEx accepts the ordinary DOS spelling for these
    // short local paths, while treating the extended spelling as an invalid
    // rename target on some runner images.
    let display_path = display_path.strip_prefix(r"\\?\").unwrap_or(&display_path);
    let name = std::ffi::OsStr::new(display_path)
        .encode_wide()
        .collect::<Vec<_>>();
    let header_size = std::mem::offset_of!(FILE_RENAME_INFO, FileName);
    let byte_len = header_size + name.len() * std::mem::size_of::<u16>();
    let word_len = byte_len.div_ceil(std::mem::size_of::<usize>());
    let mut storage = vec![0_usize; word_len];
    let information = storage.as_mut_ptr().cast::<FILE_RENAME_INFO>();
    unsafe {
        (*information).Anonymous = FILE_RENAME_INFO_0 {
            Flags: FILE_RENAME_FLAG_REPLACE_IF_EXISTS | FILE_RENAME_FLAG_POSIX_SEMANTICS,
        };
        (*information).RootDirectory = std::ptr::null_mut();
        (*information).FileNameLength = (name.len() * std::mem::size_of::<u16>()) as u32;
        std::ptr::copy_nonoverlapping(
            name.as_ptr(),
            (*information).FileName.as_mut_ptr(),
            name.len(),
        );
    }
    let result = unsafe {
        SetFileInformationByHandle(
            temporary_file.as_raw_handle().cast(),
            FileRenameInfoEx,
            information.cast(),
            byte_len as u32,
        )
    };
    if result == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub fn attach(root: &Path) -> Result<AttachedProfile, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let observation = observe(&root, &snapshot)?;
    let ai = root.join(".ai");
    let config_path = ai.join("cockpit.toml");
    let id = repository_id(&root).to_string();
    if config_path.is_file() {
        let existing = fs::read_to_string(&config_path).map_err(|source| ObserverError::Read {
            path: config_path.clone(),
            source,
        })?;
        let config: RepositoryConfig =
            toml::from_str(&existing).map_err(|error| ObserverError::State {
                path: config_path.clone(),
                message: error.to_string(),
            })?;
        validate_protocol_version(config.protocol_version).map_err(|error| {
            ObserverError::State {
                path: config_path.clone(),
                message: error.to_string(),
            }
        })?;
        if config.repository_id != id {
            return Err(ObserverError::State {
                path: config_path,
                message: "repository identity does not match attach target".into(),
            });
        }
    }
    for directory in [
        ai.join("work-items/active"),
        ai.join("work-items/archive"),
        ai.join("decisions"),
        ai.join("evidence"),
        ai.join("knowledge"),
    ] {
        fs::create_dir_all(&directory).map_err(|source| ObserverError::Read {
            path: directory,
            source,
        })?;
    }
    let config = format!("protocol_version = 1\nrepository_id = \"{id}\"\n");
    atomic_write(&ai.join("cockpit.toml"), config.as_bytes())?;
    let profile_digest = cockpit_protocol::digest_json(&cockpit_protocol::ProjectProfile {
        profile_version: 1,
        repository_id: id.clone(),
        tests: observation.quality_commands.clone(),
        build_systems: observation
            .build_systems
            .iter()
            .map(|value| format!("{value:?}"))
            .collect(),
    })
    .map_err(|error| ObserverError::State {
        path: ai.join("project.json"),
        message: error.to_string(),
    })?;
    let profile = AttachedProfile {
        profile_version: 1,
        repository_id: id,
        state: "calibration_required".into(),
        profile_digest: Some(profile_digest.clone()),
        tests: observation.quality_commands,
        build_systems: observation
            .build_systems
            .iter()
            .map(|value| format!("{value:?}"))
            .collect(),
    };
    let encoded = serde_json::to_vec_pretty(&profile).map_err(|error| ObserverError::State {
        path: ai.join("project.json"),
        message: error.to_string(),
    })?;
    atomic_write(&ai.join("project.json"), &encoded)?;
    let proposal = serde_json::json!({
        "kind": "project_profile_initialization",
        "profileVersion": 1,
        "profileDigest": profile_digest,
        "state": "calibration_required",
    });
    atomic_json(&ai.join("decisions/profile-v1.json"), &proposal)?;
    Ok(profile)
}

pub fn status(root: &Path) -> Result<RepositoryStatus, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let ai = root.join(".ai");
    let config_path = ai.join("cockpit.toml");
    let config_text = fs::read_to_string(&config_path).map_err(|source| ObserverError::Read {
        path: config_path.clone(),
        source,
    })?;
    let config: RepositoryConfig =
        toml::from_str(&config_text).map_err(|error| ObserverError::State {
            path: config_path.clone(),
            message: error.to_string(),
        })?;
    validate_protocol_version(config.protocol_version).map_err(|error| ObserverError::State {
        path: config_path.clone(),
        message: error.to_string(),
    })?;
    let profile_bytes =
        fs::read(ai.join("project.json")).map_err(|source| ObserverError::Read {
            path: ai.join("project.json"),
            source,
        })?;
    let profile: AttachedProfile =
        serde_json::from_slice(&profile_bytes).map_err(|error| ObserverError::State {
            path: ai.join("project.json"),
            message: error.to_string(),
        })?;
    if profile.repository_id != config.repository_id
        || config.repository_id != repository_id(&root).to_string()
    {
        return Err(ObserverError::State {
            path: config_path,
            message: "repository identity does not match protocol state".into(),
        });
    }
    Ok(RepositoryStatus {
        protocol_version: config.protocol_version,
        repository_id: profile.repository_id,
        state: profile.state,
        profile_version: profile.profile_version,
        active_work_items: count_suffix(&ai.join("work-items/active"), ".contract.json"),
        archived_work_items: count_suffix(&ai.join("work-items/archive"), ".archive.json"),
    })
}

fn count_suffix(path: &Path, suffix: &str) -> usize {
    fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(suffix))
        .count()
}

pub fn start_work_item(
    root: &Path,
    work_item_id: &str,
    intent: &str,
    goal: &str,
    scope: &[String],
) -> Result<LifecycleReceipt, ObserverError> {
    start_work_item_with_options(
        root,
        work_item_id,
        intent,
        goal,
        scope,
        &WorkItemStartOptions {
            risk: "normal".into(),
            authority: "missing".into(),
            ..WorkItemStartOptions::default()
        },
    )
}

pub fn start_work_item_with_options(
    root: &Path,
    work_item_id: &str,
    intent: &str,
    goal: &str,
    scope: &[String],
    options: &WorkItemStartOptions,
) -> Result<LifecycleReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let ai = root.join(".ai");
    if !ai.join("cockpit.toml").exists() {
        attach(&root)?;
    }
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let profile_digest =
        Digest::sha256_bytes(&fs::read(ai.join("project.json")).map_err(|source| {
            ObserverError::Read {
                path: ai.join("project.json"),
                source,
            }
        })?);
    let repository_snapshot_digest = snapshot_digest(&snapshot)?;
    let now = now();
    let contract = serde_json::json!({
        "protocolVersion": 1,
        "repositoryId": repository_id(&root),
        "workItemId": work_item_id,
        "intent": intent,
        "goal": goal,
        "scope": scope,
        "outOfScope": options.out_of_scope.clone(),
        "risk": options.risk.clone(),
        "authority": options.authority.clone(),
        "acceptanceCriteria": options.acceptance_criteria.clone(),
        "requiredEvidenceClasses": options.required_evidence_classes.clone(),
        "baseRevision": snapshot.head.unwrap_or_else(|| "unborn".into()),
        "projectProfileDigest": profile_digest,
        "repositorySnapshotDigest": repository_snapshot_digest,
        "createdAt": now,
    });
    let summary = serde_json::json!({
        "protocolVersion": 1,
        "repositoryId": repository_id(&root),
        "workItemId": work_item_id,
        "state": "implementation_active",
        "changedPaths": snapshot.changed_paths,
        "checkpointCount": 0,
        "createdAt": now,
        "updatedAt": now,
    });
    let active = ai.join("work-items/active");
    let archive = ai.join("work-items/archive");
    if [
        active.join(format!("{work_item_id}.contract.json")),
        active.join(format!("{work_item_id}.summary.json")),
        archive.join(format!("{work_item_id}.archive.json")),
    ]
    .iter()
    .any(|path| path.exists())
    {
        return Err(ObserverError::State {
            path: active.join(format!("{work_item_id}.contract.json")),
            message: "work item already exists".into(),
        });
    }
    atomic_json(
        &active.join(format!("{work_item_id}.contract.json")),
        &contract,
    )?;
    atomic_json(
        &active.join(format!("{work_item_id}.summary.json")),
        &summary,
    )?;
    Ok(LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "implementation_active".into(),
        timestamp: now,
    })
}

pub fn checkpoint_work_item(
    root: &Path,
    work_item_id: &str,
) -> Result<LifecycleReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let mut summary: serde_json::Value = read_json(&path)?;
    let count = summary["checkpointCount"].as_u64().unwrap_or(0) + 1;
    let timestamp = now();
    summary["checkpointCount"] = count.into();
    summary["updatedAt"] = timestamp.clone().into();
    atomic_json(&path, &summary)?;
    Ok(LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "checkpointed".into(),
        timestamp,
    })
}

pub fn finish_work_item(
    root: &Path,
    work_item_id: &str,
) -> Result<LifecycleReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let active = root.join(".ai/work-items/active");
    let summary_path = active.join(format!("{work_item_id}.summary.json"));
    let mut summary: serde_json::Value = read_json(&summary_path)?;
    let original_summary = summary.clone();
    let evidence_path = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.verification.json"));
    let evidence = read_json(&evidence_path).map_err(|_| ObserverError::State {
        path: evidence_path.clone(),
        message: "finish requires a recorded verification receipt".into(),
    })?;
    if evidence["workItemId"].as_str() != Some(work_item_id)
        || evidence["passed"] != serde_json::Value::Bool(true)
    {
        return Err(ObserverError::State {
            path: evidence_path,
            message: "verification receipt is not a passed receipt for this work item".into(),
        });
    }
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    let current_digest = snapshot_digest(&snapshot)?;
    if evidence["repositorySnapshotDigest"] != serde_json::Value::String(current_digest.to_string())
    {
        return Err(ObserverError::State {
            path: evidence_path,
            message: "verification receipt is stale for the current repository snapshot".into(),
        });
    }
    let contract_path = active.join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&contract_path)?;
    require_green_governance(&root, &contract_path, &contract, &snapshot, "finish")?;
    let timestamp = now();
    summary["state"] = "finish_ready".into();
    summary["updatedAt"] = timestamp.clone().into();
    atomic_json(&summary_path, &summary)?;
    let outcome = serde_json::json!({
        "protocolVersion": 1,
        "workItemId": work_item_id,
        "state": "finish_ready",
        "verification": {"status": "verified", "required": true, "evidencePath": format!(".ai/evidence/{work_item_id}.verification.json")},
        "evidenceDigest": cockpit_protocol::digest_json(&evidence).map_err(|error| ObserverError::State { path: root.join(".ai/evidence"), message: error.to_string() })?,
        "createdAt": timestamp,
    });
    if let Err(error) = atomic_json(
        &active.join(format!("{work_item_id}.outcome.json")),
        &outcome,
    ) {
        let _ = atomic_json(&summary_path, &original_summary);
        return Err(error);
    }
    Ok(LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "finish_ready".into(),
        timestamp,
    })
}

pub fn record_verification(
    root: &Path,
    work_item_id: &str,
    receipt: &serde_json::Value,
    runtime_version: &str,
    runtime_digest: &Digest,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    record_verification_with_snapshot(
        &root,
        work_item_id,
        receipt,
        runtime_version,
        runtime_digest,
        &snapshot,
    )
}

pub fn record_verification_with_snapshot(
    root: &Path,
    work_item_id: &str,
    receipt: &serde_json::Value,
    runtime_version: &str,
    runtime_digest: &Digest,
    snapshot: &RepositorySnapshot,
) -> Result<serde_json::Value, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    if fs::canonicalize(&snapshot.root).ok().as_ref() != Some(&root) {
        return Err(ObserverError::SnapshotRootMismatch);
    }
    let active_contract = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    if !active_contract.is_file() {
        return Err(ObserverError::State {
            path: active_contract,
            message: "verification evidence requires an active work item contract".into(),
        });
    }
    if receipt["passed"] != serde_json::Value::Bool(true) {
        return Err(ObserverError::State {
            path: root.join(".ai/evidence"),
            message: "failed verification cannot be recorded as completion evidence".into(),
        });
    }
    if receipt
        .get("workItemId")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|receipt_id| receipt_id != work_item_id)
    {
        return Err(ObserverError::State {
            path: root.join(".ai/evidence"),
            message: "verification receipt belongs to another work item".into(),
        });
    }
    let evidence = serde_json::json!({
        "protocolVersion": 1,
        "workItemId": work_item_id,
        "runtimeVersion": runtime_version,
        "runtimeDigest": runtime_digest,
        "repositorySnapshotDigest": snapshot_digest(snapshot)?,
        "passed": true,
        "receipt": receipt,
        "createdAt": now(),
    });
    let path = root
        .join(".ai/evidence")
        .join(format!("{work_item_id}.verification.json"));
    atomic_json(&path, &evidence)?;
    Ok(evidence)
}

pub fn snapshot_digest(snapshot: &RepositorySnapshot) -> Result<Digest, ObserverError> {
    let mut stable = snapshot.clone();
    stable
        .changed_paths
        .retain(|path| !path.starts_with(".ai/"));
    cockpit_protocol::digest_json(&stable).map_err(|error| ObserverError::State {
        path: snapshot.root.join(".ai"),
        message: error.to_string(),
    })
}

fn is_test_path(path: &str) -> bool {
    let normalized = path.to_ascii_lowercase();
    normalized.starts_with("tests/")
        || normalized.starts_with("test/")
        || normalized.contains("/tests/")
        || normalized.contains("/test/")
        || normalized.contains("/spec/")
        || normalized
            .rsplit('/')
            .next()
            .is_some_and(|name| name.contains("test") || name.contains("spec"))
}

fn is_coverage_path(path: &str) -> bool {
    let normalized = path.to_ascii_lowercase();
    let name = normalized.rsplit('/').next().unwrap_or(&normalized);
    matches!(
        name,
        ".coveragerc"
            | "coverage.json"
            | "coverage.yaml"
            | "coverage.yml"
            | "pyproject.toml"
            | "tox.ini"
            | "package.json"
    ) || name.starts_with("jest.config.")
}

fn is_textual_material_path(path: &str) -> bool {
    let normalized = path.to_ascii_lowercase();
    [
        ".md", ".txt", ".json", ".toml", ".yaml", ".yml", ".rs", ".py", ".js", ".ts", ".tsx",
        ".jsx", ".java", ".kt", ".swift", ".go", ".sh",
    ]
    .iter()
    .any(|extension| normalized.ends_with(extension))
}

fn contains_strong_instruction_injection(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    let instruction = [
        "ignore previous instructions",
        "ignore all previous instructions",
        "ignore the contract",
        "override policy",
        "bypass policy",
        "disable governance",
        "system message",
    ]
    .iter()
    .any(|pattern| text.contains(pattern));
    let risky_operation = [
        "delete",
        "rm -rf",
        "execute",
        "run ",
        "curl ",
        "secret",
        "token",
        "upload",
        "exfil",
        "disable test",
        "skip test",
        "publish",
        "push main",
    ]
    .iter()
    .any(|pattern| text.contains(pattern));
    instruction && risky_operation
}

fn contains_skip_marker(lines: &[String]) -> bool {
    lines.iter().any(|line| {
        let line = line.to_ascii_lowercase();
        [
            "pytest.mark.skip",
            ".skip(",
            "#[ignore]",
            "@disabled",
            "@ignore",
            "disabled_",
            "xit(",
            "xdescribe(",
        ]
        .iter()
        .any(|marker| line.contains(marker))
    })
}

fn assertion_count(lines: &[String]) -> usize {
    lines
        .iter()
        .filter(|line| {
            let line = line.to_ascii_lowercase();
            ["assert", "expect(", "xctassert", "tothrow("]
                .iter()
                .any(|marker| line.contains(marker))
        })
        .count()
}

fn contains_test_bypass(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    let removes_tests = (text.contains("delete")
        || text.contains("remove")
        || text.contains("disable")
        || text.contains("skip"))
        && text.contains("test");
    let claims_success = text.contains("pass") || text.contains("green") || text.contains("ci");
    removes_tests && claims_success
        || text.contains("continue-on-error: true")
        || text.contains("allow_failure: true")
        || text.contains("|| true")
}

fn assignment(line: &str) -> Option<(String, String)> {
    let (name, value) = line.split_once('=').or_else(|| line.split_once(':'))?;
    Some((
        name.trim().to_ascii_lowercase(),
        value.trim().trim_matches(['\'', '"']).to_ascii_lowercase(),
    ))
}

fn first_number(value: &str) -> Option<f64> {
    let start = value.find(|character: char| character.is_ascii_digit())?;
    let number = value[start..]
        .chars()
        .take_while(|character| character.is_ascii_digit() || *character == '.')
        .collect::<String>();
    number.parse().ok()
}

fn coverage_weakened(removed: &[String], added: &[String]) -> bool {
    let removed = removed
        .iter()
        .filter_map(|line| assignment(line))
        .collect::<Vec<_>>();
    let added = added
        .iter()
        .filter_map(|line| assignment(line))
        .collect::<Vec<_>>();
    for (name, before) in &removed {
        if matches!(name.as_str(), "fail_under" | "threshold" | "minimum")
            && let Some((_, after)) = added.iter().find(|(candidate, _)| candidate == name)
            && let (Some(before), Some(after)) = (first_number(before), first_number(after))
            && after < before
        {
            return true;
        }
        if matches!(name.as_str(), "source" | "source_pkgs")
            && let Some((_, after)) = added.iter().find(|(candidate, _)| candidate == name)
        {
            let before = before
                .split([',', ' '])
                .filter(|item| !item.is_empty())
                .count();
            let after = after
                .split([',', ' '])
                .filter(|item| !item.is_empty())
                .count();
            if after < before {
                return true;
            }
        }
    }
    added.iter().any(|(name, value)| {
        matches!(
            name.as_str(),
            "omit" | "exclude" | "exclude_lines" | "coveragepathignorepatterns"
        ) && !removed
            .iter()
            .any(|(old_name, old_value)| old_name == name && old_value == value)
    })
}

pub fn derive_governance_signals(snapshot: &RepositorySnapshot) -> GovernanceSignalAssessment {
    let mut result = GovernanceSignalAssessment::default();
    for change in &snapshot.change_evidence {
        if change.path.starts_with(".ai/") {
            continue;
        }
        let test_path = is_test_path(&change.path);
        let coverage_path = is_coverage_path(&change.path);
        let inspectable = matches!(
            change.content_state,
            ChangeContentState::Text | ChangeContentState::Deleted
        );
        if !inspectable {
            if test_path {
                result
                    .unknowns
                    .push("test_weakening_inspection_unavailable".into());
            }
            if coverage_path {
                result
                    .unknowns
                    .push("coverage_weakening_inspection_unavailable".into());
            }
            if is_textual_material_path(&change.path) {
                result
                    .unknowns
                    .push("repository_material_inspection_unavailable".into());
            }
            continue;
        }

        let changed_text = change.after_text.as_deref().unwrap_or("").to_owned()
            + "\n"
            + &change.added_lines.join("\n");
        if contains_strong_instruction_injection(&changed_text) {
            result.untrusted_material = true;
            result.findings.push("repository_prompt_injection".into());
        }
        if test_path
            && (change.kind == ChangeKind::Deleted
                || contains_skip_marker(&change.added_lines)
                || assertion_count(&change.removed_lines) > assertion_count(&change.added_lines)
                || contains_test_bypass(&changed_text))
        {
            result.test_weakening = true;
            result.findings.push("test_weakening".into());
        }
        if coverage_path && coverage_weakened(&change.removed_lines, &change.added_lines) {
            result.coverage_weakening = true;
            result.findings.push("coverage_weakening".into());
        }
    }
    result.unknowns.sort();
    result.unknowns.dedup();
    result.findings.sort();
    result.findings.dedup();
    result
}

pub fn contract_freshness_findings(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    _snapshot: &RepositorySnapshot,
) -> Result<Vec<String>, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let ai_path = root.join(".ai");
    let config_path = ai_path.join("cockpit.toml");
    let profile_path = ai_path.join("project.json");
    if !ai_path.is_dir() {
        return Ok(Vec::new());
    }
    let mut findings = Vec::new();
    if !config_path.is_file() || !profile_path.is_file() {
        findings.push("stale_contract".into());
        return Ok(findings);
    }
    if contract.repository_id != repository_id(&root).to_string() {
        findings.push("stale_contract".into());
    }
    let profile_digest =
        Digest::sha256_bytes(
            &fs::read(&profile_path).map_err(|source| ObserverError::Read {
                path: profile_path.clone(),
                source,
            })?,
        );
    if contract.project_profile_digest != profile_digest {
        findings.push("stale_contract".into());
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}

pub fn governance_decision_for_contract(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
) -> Result<GovernanceDecision, ObserverError> {
    let explicit_blockers = contract_freshness_findings(root, contract, snapshot)?;
    let signals = derive_governance_signals(snapshot);
    let changed_paths = snapshot
        .changed_paths
        .iter()
        .filter(|path| !path.starts_with(".ai/"))
        .cloned()
        .collect();
    let action = if contract.risk.to_ascii_lowercase().contains("destructive") {
        ActionKind::Destructive
    } else {
        ActionKind::Write
    };
    let authority = if contract.authority == "authorized" {
        AuthorityState::Authorized
    } else {
        AuthorityState::Missing
    };
    let evidence = evidence_state_for_contract(root, contract, snapshot)?;
    Ok(evaluate(GovernanceInput {
        scope: contract.scope.clone(),
        out_of_scope: contract.out_of_scope.clone(),
        changed_paths,
        action,
        authority,
        evidence,
        untrusted_material: signals.untrusted_material,
        test_weakening: signals.test_weakening,
        coverage_weakening: signals.coverage_weakening,
        explicit_blockers,
        explicit_unknowns: signals.unknowns,
        outcome_state_override: None,
        authority_override: None,
    }))
}

fn read_contract(path: &Path) -> Result<cockpit_protocol::Contract, ObserverError> {
    let value: serde_json::Value = read_json(path)?;
    serde_json::from_value(value).map_err(|error| ObserverError::State {
        path: path.to_path_buf(),
        message: format!("invalid work item contract: {error}"),
    })
}

fn require_green_governance(
    root: &Path,
    contract_path: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
    operation: &str,
) -> Result<(), ObserverError> {
    let decision = governance_decision_for_contract(root, contract, snapshot)?;
    if decision.state != DecisionState::Green {
        return Err(ObserverError::State {
            path: contract_path.to_path_buf(),
            message: format!(
                "{operation} requires a green governance decision (state={:?}, blockers={:?}, unknowns={:?})",
                decision.state, decision.blockers, decision.unknowns
            ),
        });
    }
    Ok(())
}

pub fn evidence_state_for_contract(
    root: &Path,
    contract: &cockpit_protocol::Contract,
    snapshot: &RepositorySnapshot,
) -> Result<EvidenceState, ObserverError> {
    if contract.required_evidence_classes.is_empty() {
        return Ok(EvidenceState::Complete);
    }
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let evidence_path = root
        .join(".ai/evidence")
        .join(format!("{}.verification.json", contract.work_item_id));
    let evidence = match read_json(&evidence_path) {
        Ok(value) => value,
        Err(ObserverError::Read { source, .. })
            if source.kind() == std::io::ErrorKind::NotFound =>
        {
            return Ok(EvidenceState::Missing);
        }
        Err(error) => return Err(error),
    };
    if evidence["workItemId"] != serde_json::Value::String(contract.work_item_id.clone())
        || evidence["passed"] != serde_json::Value::Bool(true)
    {
        return Ok(EvidenceState::Contradictory);
    }
    if evidence["repositorySnapshotDigest"]
        != serde_json::Value::String(snapshot_digest(snapshot)?.to_string())
    {
        return Ok(EvidenceState::Stale);
    }
    if contract.required_evidence_classes.iter().any(|class| {
        !matches!(
            class.to_ascii_lowercase().as_str(),
            "verification" | "verification_receipt" | "verification-receipt"
        )
    }) {
        return Ok(EvidenceState::Missing);
    }
    Ok(EvidenceState::Complete)
}

pub fn archive_work_item(
    root: &Path,
    work_item_id: &str,
) -> Result<LifecycleReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let ai = root.join(".ai");
    let active = ai.join("work-items/active");
    let archive = ai.join("work-items/archive");
    let contract_path = active.join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&contract_path)?;
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    require_green_governance(&root, &contract_path, &contract, &snapshot, "archive")?;
    let outcome_path = active.join(format!("{work_item_id}.outcome.json"));
    let outcome = read_json(&outcome_path)?;
    if outcome["verification"]["status"] != "verified" {
        return Err(ObserverError::State {
            path: outcome_path,
            message: "archive requires a verified outcome".into(),
        });
    }
    fs::create_dir_all(&archive).map_err(|source| ObserverError::Read {
        path: archive.clone(),
        source,
    })?;
    let manifest_path = archive.join(format!("{work_item_id}.archive.json"));
    if manifest_path.exists() {
        return Err(ObserverError::State {
            path: manifest_path,
            message: "archive manifest already exists".into(),
        });
    }
    let names = ["contract", "summary", "outcome"];
    let mut files = serde_json::Map::new();
    let mut pending = Vec::new();
    for name in names {
        let source_path = active.join(format!("{work_item_id}.{name}.json"));
        let target = archive.join(format!("{work_item_id}.{name}.json"));
        let bytes = fs::read(&source_path).map_err(|error| ObserverError::Read {
            path: source_path.clone(),
            source: error,
        })?;
        files.insert(
            format!("{name}Path"),
            serde_json::Value::String(format!(".ai/work-items/archive/{work_item_id}.{name}.json")),
        );
        files.insert(
            format!("{name}Digest"),
            serde_json::Value::String(Digest::sha256_bytes(&bytes).to_string()),
        );
        if target.exists() {
            return Err(ObserverError::State {
                path: target,
                message: "archive target already exists".into(),
            });
        }
        pending.push((source_path, target));
    }
    let mut moved = Vec::new();
    for (source, target) in &pending {
        if let Err(source_error) = fs::rename(source, target) {
            for (moved_source, moved_target) in moved.into_iter().rev() {
                let _ = fs::rename(moved_target, moved_source);
            }
            return Err(ObserverError::Read {
                path: target.clone(),
                source: source_error,
            });
        }
        moved.push((source.clone(), target.clone()));
    }
    let timestamp = now();
    let manifest = serde_json::json!({
        "protocolVersion": 1,
        "workItemId": work_item_id,
        "state": "archived",
        "files": files,
        "createdAt": timestamp,
    });
    if let Err(error) = atomic_json(&manifest_path, &manifest) {
        for (moved_source, moved_target) in moved.into_iter().rev() {
            let _ = fs::rename(moved_target, moved_source);
        }
        return Err(error);
    }
    let _ = fs::remove_file(ai.join("knowledge/index.json"));
    Ok(LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "archived".into(),
        timestamp,
    })
}

pub fn close_work_item(root: &Path, work_item_id: &str) -> Result<LifecycleReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    Err(ObserverError::State {
        path: root.join(".ai/decisions"),
        message: "close requires an explicit human decision".into(),
    })
}

pub fn close_work_item_with_decision(
    root: &Path,
    work_item_id: &str,
    human_decision: &str,
) -> Result<LifecycleReceipt, ObserverError> {
    validate_work_item_id(work_item_id)?;
    if human_decision.trim().is_empty() {
        return Err(ObserverError::State {
            path: root.join(".ai/decisions"),
            message: "human decision must not be empty".into(),
        });
    }
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let archive = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.archive.json"));
    let manifest = read_json(&archive)?;
    verify_archive_manifest(&root, work_item_id, &manifest)?;
    let contract_path = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.contract.json"));
    let contract = read_contract(&contract_path)?;
    let git =
        cockpit_git::GitRepository::discover(&root).map_err(|error| ObserverError::State {
            path: root.clone(),
            message: error.to_string(),
        })?;
    let snapshot = git.snapshot().map_err(|error| ObserverError::State {
        path: root.clone(),
        message: error.to_string(),
    })?;
    require_green_governance(&root, &contract_path, &contract, &snapshot, "close")?;
    let outcome = root
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.outcome.json"));
    let outcome_value = read_json(&outcome)?;
    if outcome_value["verification"]["status"] != "verified" {
        return Err(ObserverError::State {
            path: outcome,
            message: "close requires a verified outcome".into(),
        });
    }
    let timestamp = now();
    let receipt = LifecycleReceipt {
        work_item_id: work_item_id.into(),
        state: "closed".into(),
        timestamp: timestamp.clone(),
    };
    let receipt_value = serde_json::to_value(&receipt).map_err(|error| ObserverError::State {
        path: root.join(".ai/decisions"),
        message: error.to_string(),
    })?;
    let decision_path = root
        .join(".ai/decisions")
        .join(format!("{work_item_id}.close.json"));
    if decision_path.exists() {
        return Err(ObserverError::State {
            path: decision_path,
            message: "work item is already closed".into(),
        });
    }
    let mut decision = receipt_value;
    decision["humanDecision"] = serde_json::Value::String(human_decision.trim().into());
    decision["decisionState"] = serde_json::Value::String("confirmed".into());
    atomic_json(&decision_path, &decision)?;
    Ok(receipt)
}

fn verify_archive_manifest(
    root: &Path,
    work_item_id: &str,
    manifest: &serde_json::Value,
) -> Result<(), ObserverError> {
    if manifest["workItemId"] != serde_json::Value::String(work_item_id.into())
        || manifest["state"] != serde_json::Value::String("archived".into())
    {
        return Err(ObserverError::State {
            path: root
                .join(".ai/work-items/archive")
                .join(format!("{work_item_id}.archive.json")),
            message: "archive manifest identity or state is invalid".into(),
        });
    }
    let archive = root.join(".ai/work-items/archive");
    for name in ["contract", "summary", "outcome"] {
        let path = archive.join(format!("{work_item_id}.{name}.json"));
        let bytes = fs::read(&path).map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?;
        let expected = manifest["files"][format!("{name}Digest")]
            .as_str()
            .ok_or_else(|| ObserverError::State {
                path: path.clone(),
                message: format!("archive manifest is missing {name} digest"),
            })?;
        let actual = Digest::sha256_bytes(&bytes).to_string();
        if actual != expected {
            return Err(ObserverError::State {
                path,
                message: format!("archived {name} digest does not match manifest"),
            });
        }
    }
    Ok(())
}

pub fn generate_knowledge(root: &Path) -> Result<cockpit_knowledge::KnowledgeIndex, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let archive = root.join(".ai/work-items/archive");
    let knowledge = root.join(".ai/knowledge");
    let index_path = knowledge.join("index.json");
    if index_path.is_file() {
        let cached = read_json(&index_path)?;
        return serde_json::from_value(cached).map_err(|error| ObserverError::State {
            path: index_path,
            message: error.to_string(),
        });
    }
    let mut records = Vec::new();
    for entry in fs::read_dir(&archive).map_err(|source| ObserverError::Read {
        path: archive.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: archive.clone(),
            source,
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(work_item_id) = name.strip_suffix(".archive.json") else {
            continue;
        };
        let contract_path = archive.join(format!("{work_item_id}.contract.json"));
        let contract = read_json(&contract_path)?;
        let intent = contract["intent"].as_str().unwrap_or("unknown");
        records.push(cockpit_knowledge::project_record(
            work_item_id,
            intent,
            "archived",
            &format!(".ai/work-items/archive/{work_item_id}.archive.json"),
        ));
    }
    let index = cockpit_knowledge::KnowledgeIndex::from_records(records);
    fs::create_dir_all(&knowledge).map_err(|source| ObserverError::Read {
        path: knowledge.clone(),
        source,
    })?;
    let encoded = serde_json::to_value(&index).map_err(|error| ObserverError::State {
        path: knowledge.clone(),
        message: error.to_string(),
    })?;
    atomic_json(&knowledge.join("index.json"), &encoded)?;
    for record in &index.records {
        let record_value = serde_json::to_value(record).map_err(|error| ObserverError::State {
            path: knowledge.clone(),
            message: error.to_string(),
        })?;
        atomic_json(&root.join(&record.knowledge_path), &record_value)?;
    }
    Ok(index)
}

fn validate_work_item_id(id: &str) -> Result<(), ObserverError> {
    if id.is_empty()
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(ObserverError::State {
            path: PathBuf::from(id),
            message: "invalid work item id".into(),
        });
    }
    Ok(())
}

fn read_json(path: &Path) -> Result<serde_json::Value, ObserverError> {
    let bytes = fs::read(path).map_err(|source| ObserverError::Read {
        path: path.into(),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|error| ObserverError::State {
        path: path.into(),
        message: error.to_string(),
    })
}

fn atomic_json(path: &Path, value: &serde_json::Value) -> Result<(), ObserverError> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| ObserverError::State {
        path: path.into(),
        message: error.to_string(),
    })?;
    atomic_write(path, &bytes)
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), ObserverError> {
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&temporary, bytes).map_err(|source| ObserverError::Read {
        path: temporary.clone(),
        source,
    })?;
    fs::rename(&temporary, path).map_err(|source| ObserverError::Read {
        path: path.into(),
        source,
    })
}

pub fn observe(
    root: &Path,
    snapshot: &RepositorySnapshot,
) -> Result<RepositoryObservation, ObserverError> {
    let canonical_root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let canonical_snapshot_root =
        fs::canonicalize(&snapshot.root).map_err(|source| ObserverError::Read {
            path: snapshot.root.clone(),
            source,
        })?;
    if canonical_root != canonical_snapshot_root {
        return Err(ObserverError::SnapshotRootMismatch);
    }
    let mut files = Vec::new();
    collect_files(&canonical_root, &canonical_root, &mut files)?;
    let mut languages = Vec::new();
    let mut build_systems = Vec::new();
    let mut test_roots = Vec::new();
    let mut quality_commands = Vec::new();
    let mut ci_surfaces = Vec::new();
    let mut critical_domains = Vec::new();
    for relative in &files {
        match relative
            .extension()
            .and_then(|extension| extension.to_str())
        {
            Some("rs") => languages.push(LanguageSignal::Rust),
            Some("py") => languages.push(LanguageSignal::Python),
            Some("js") | Some("jsx") => languages.push(LanguageSignal::JavaScript),
            Some("ts") | Some("tsx") => languages.push(LanguageSignal::TypeScript),
            Some("go") => languages.push(LanguageSignal::Go),
            _ => {}
        }
        let path = relative.to_string_lossy();
        let path_string = path.to_string();
        if path.starts_with("tests/") || path == "tests" {
            test_roots.push("tests/**".into());
        }
        if path.starts_with(".github/workflows/") {
            ci_surfaces.push(path_string.clone());
        }
        for (needle, domain) in [
            ("security", "security"),
            ("payment", "payment"),
            ("auth", "identity"),
            ("production", "production"),
            ("release", "release"),
        ] {
            if path.contains(needle) {
                critical_domains.push(domain.into());
            }
        }
        if path == "Cargo.toml" {
            build_systems.push(BuildSystem::Cargo);
            quality_commands.push(QualityCommand {
                program: "cargo".into(),
                args: vec!["test".into(), "--workspace".into()],
                state: "detected".into(),
            });
        } else if path == "package.json" {
            build_systems.push(BuildSystem::Npm);
        } else if path == "pyproject.toml" {
            build_systems.push(BuildSystem::Poetry);
        } else if path == "go.mod" {
            build_systems.push(BuildSystem::Go);
        } else if path == "Makefile" {
            build_systems.push(BuildSystem::Make);
        }
    }
    languages.sort_by_key(|value| format!("{value:?}"));
    languages.dedup();
    build_systems.sort_by_key(|value| format!("{value:?}"));
    build_systems.dedup();
    test_roots.sort();
    test_roots.dedup();
    ci_surfaces.sort();
    ci_surfaces.dedup();
    critical_domains.sort();
    critical_domains.dedup();
    let mut hasher = Sha256::new();
    for path in &files {
        hasher.update(path.to_string_lossy().as_bytes());
        hasher.update([0]);
    }
    let snapshot_digest = Digest::sha256_bytes(&hasher.finalize());
    Ok(RepositoryObservation {
        snapshot_digest,
        languages,
        build_systems,
        test_roots,
        quality_commands,
        ci_surfaces,
        critical_domains,
        dependency_fingerprint: snapshot
            .dependency_fingerprint
            .parse()
            .unwrap_or_else(|_| Digest::sha256_bytes(b"invalid-dependency-fingerprint")),
        files_read: files.len() + snapshot.files_read,
        cache_hit: false,
    })
}

pub fn observe_cached(
    root: &Path,
    snapshot: &RepositorySnapshot,
) -> Result<RepositoryObservation, ObserverError> {
    let canonical_root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    if !canonical_root.join(".ai/cockpit.toml").is_file() {
        return observe(&canonical_root, snapshot);
    }
    let cache_path = canonical_root.join(".ai/decisions/observer-snapshot.json");
    if let Ok(cache) = read_json(&cache_path) {
        let matches = cache["treeDigest"] == snapshot.tree_digest
            && cache["diffDigest"] == snapshot.diff_digest
            && cache["dependencyFingerprint"] == snapshot.dependency_fingerprint;
        if matches
            && let Ok(mut observation) =
                serde_json::from_value::<RepositoryObservation>(cache["observation"].clone())
        {
            observation.cache_hit = true;
            return Ok(observation);
        }
    }
    let observation = observe(&canonical_root, snapshot)?;
    let cache = serde_json::json!({
        "treeDigest": snapshot.tree_digest,
        "diffDigest": snapshot.diff_digest,
        "dependencyFingerprint": snapshot.dependency_fingerprint,
        "observation": observation,
    });
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).map_err(|source| ObserverError::Read {
            path: parent.into(),
            source,
        })?;
    }
    atomic_json(&cache_path, &cache)?;
    Ok(observation)
}

pub fn classify_evolution(
    _profile: &cockpit_protocol::ProjectProfile,
    observation: &RepositoryObservation,
    snapshot: &RepositorySnapshot,
) -> Vec<EvolutionEvent> {
    snapshot
        .changed_paths
        .iter()
        .map(|path| {
            let normalized = path.replace('\\', "/");
            let is_governance = normalized.starts_with(".github/")
                || normalized.contains("security")
                || normalized.contains("release")
                || normalized.contains("branch-protection")
                || normalized.contains("production");
            let is_known_test = observation.test_roots.iter().any(|root| {
                root.strip_suffix("/**")
                    .is_some_and(|prefix| normalized.starts_with(&format!("{prefix}/")))
            });
            let is_new_capability = normalized.contains("playwright")
                || normalized.contains("cypress")
                || normalized.contains("nextest")
                || normalized.ends_with("playwright.config.ts")
                || normalized.ends_with("playwright.config.js");
            if is_governance {
                EvolutionEvent {
                    class: EvolutionClass::L3,
                    event_type: "governance_change".into(),
                    path: normalized,
                    action: "needs_human_decision".into(),
                }
            } else if is_new_capability {
                EvolutionEvent {
                    class: EvolutionClass::L2,
                    event_type: "new_test_framework".into(),
                    path: normalized,
                    action: "needs_confirmation".into(),
                }
            } else if is_known_test {
                EvolutionEvent {
                    class: EvolutionClass::L1,
                    event_type: "new_test".into(),
                    path: normalized,
                    action: "auto_absorb".into(),
                }
            } else {
                EvolutionEvent {
                    class: EvolutionClass::L0,
                    event_type: "content_change".into(),
                    path: normalized,
                    action: "auto_absorb".into(),
                }
            }
        })
        .collect()
}

pub fn profile_update_proposal(
    profile: &cockpit_protocol::ProjectProfile,
    events: &[EvolutionEvent],
) -> Option<ProfileUpdateProposal> {
    let candidate = events
        .iter()
        .find(|event| matches!(&event.class, EvolutionClass::L2 | EvolutionClass::L3))?;
    Some(ProfileUpdateProposal {
        from_profile_version: profile.profile_version,
        candidate: candidate.path.clone(),
        reason: candidate.event_type.clone(),
        requires_human_confirmation: true,
    })
}

pub fn confirm_profile_update(
    root: &Path,
    program: &str,
    args: &[String],
) -> Result<AttachedProfile, ObserverError> {
    let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let project_path = root.join(".ai/project.json");
    let current: AttachedProfile = read_json(&project_path).and_then(|value| {
        serde_json::from_value(value).map_err(|error| ObserverError::State {
            path: project_path.clone(),
            message: error.to_string(),
        })
    })?;
    let mut tests = current.tests.clone();
    let candidate = QualityCommand {
        program: program.into(),
        args: args.to_vec(),
        state: "verified".into(),
    };
    if !tests.contains(&candidate) {
        tests.push(candidate);
    }
    tests.sort_by(|left, right| {
        (left.program.as_str(), &left.args).cmp(&(right.program.as_str(), &right.args))
    });
    let profile_version = current.profile_version + 1;
    let profile = cockpit_protocol::ProjectProfile {
        profile_version,
        repository_id: current.repository_id.clone(),
        tests: tests.clone(),
        build_systems: current.build_systems.clone(),
    };
    let profile_digest =
        cockpit_protocol::digest_json(&profile).map_err(|error| ObserverError::State {
            path: project_path.clone(),
            message: error.to_string(),
        })?;
    let updated = AttachedProfile {
        profile_version,
        repository_id: current.repository_id,
        state: "calibrated".into(),
        profile_digest: Some(profile_digest.clone()),
        tests,
        build_systems: current.build_systems,
    };
    let value = serde_json::to_value(&updated).map_err(|error| ObserverError::State {
        path: project_path.clone(),
        message: error.to_string(),
    })?;
    atomic_json(&project_path, &value)?;
    let decision = serde_json::json!({
        "kind": "project_profile_confirmation",
        "profileVersion": profile_version,
        "profileDigest": profile_digest,
        "candidate": {"program": program, "args": args},
        "state": "confirmed",
        "createdAt": now(),
    });
    atomic_json(
        &root
            .join(".ai/decisions")
            .join(format!("profile-v{profile_version}.json")),
        &decision,
    )?;
    Ok(updated)
}

fn collect_files(
    root: &Path,
    current: &Path,
    output: &mut Vec<PathBuf>,
) -> Result<(), ObserverError> {
    let entries = fs::read_dir(current).map_err(|source| ObserverError::Read {
        path: current.into(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| ObserverError::Read {
            path: current.into(),
            source,
        })?;
        let path = entry.path();
        if path
            .file_name()
            .is_some_and(|name| name == ".git" || name == "target")
        {
            continue;
        }
        if path.is_dir() {
            collect_files(root, &path, output)?;
        } else if path.is_file() {
            output.push(path.strip_prefix(root).unwrap_or(&path).to_path_buf());
        }
    }
    output.sort();
    Ok(())
}

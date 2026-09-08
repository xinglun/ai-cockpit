use super::project_governance::{ProjectGovernanceFacts, observe_project_governance};
use super::*;

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

pub(super) fn assess_verification_reuse_measured(
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
    let Ok(stage) = VerificationStage::parse(&input.stage) else {
        return Ok(denied_reuse("verification_context_invalid"));
    };
    if input.program.is_empty()
        || input.scope.is_empty()
        || !matches!(input.runner.as_str(), "local" | "hosted")
    {
        return Ok(denied_reuse("verification_context_invalid"));
    }
    let base_commit = if !stage.requires_base_revision() {
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
        // Reuse is bound to source state, not to the governance receipts that
        // this request itself writes.  The raw diff digest includes `.ai/`
        // records, so using it here would make every successful verification
        // invalidate its own reusable receipt.  `snapshot_digest` filters
        // governance-only paths while retaining source and working-tree
        // identity and remains request-scoped.
        content_digest: snapshot_digest(snapshot)?.to_string(),
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

pub(super) fn refresh_verification_context(
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
    let stage = VerificationStage::parse(&input.stage).ok();
    let base_commit = if stage.is_some_and(|value| !value.requires_base_revision()) {
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
    context.content_digest = snapshot_digest(snapshot)?.to_string();
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

pub(super) fn denied_reuse(reason: &str) -> VerificationReuseAssessment {
    VerificationReuseAssessment::Denied {
        reason: reason.into(),
    }
}

pub(super) fn valid_git_object_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

pub(super) fn digest_value<T: Serialize>(value: &T, path: &Path) -> Result<String, ObserverError> {
    cockpit_protocol::digest_json(value)
        .map(|digest| digest.to_string())
        .map_err(|error| ObserverError::State {
            path: path.to_path_buf(),
            message: error.to_string(),
        })
}

pub(super) fn digest_raw_bytes(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

pub(super) fn read_verification_identity_files(
    root: &Path,
) -> Result<(Vec<u8>, Vec<u8>), ObserverError> {
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

pub(super) fn resolved_executable_identity(
    root: &Path,
    program: &str,
) -> Option<ResolvedExecutableIdentity> {
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
pub(super) fn supports_pinned_execution(components: &[ExecutableComponent]) -> bool {
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
    pub(super) fn execution_environment(&self) -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
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
    pub(super) fn execution_environment(&self) -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
        Vec::new()
    }

    pub(super) fn execution(&self, original_args: &[String]) -> Option<(String, Vec<String>)> {
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

pub(super) fn build_repository_verification_command(
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

pub(super) fn resolve_executable(root: &Path, program: &str) -> Option<PathBuf> {
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
pub(super) fn resolve_platform_executable(requested: &Path) -> Option<PathBuf> {
    is_executable_file(requested).then(|| requested.to_path_buf())
}

#[cfg(windows)]
pub(super) fn resolve_platform_executable(requested: &Path) -> Option<PathBuf> {
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

pub(super) fn collect_executable_identities(
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

pub(super) fn stage_pinned_executable(
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
pub(super) fn mirror_relative_runtime_layout(
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
pub(super) fn relative_path(from: &Path, to: &Path) -> Option<PathBuf> {
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
pub(super) fn should_stage_executable(executable: &Path) -> bool {
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
pub(super) fn should_stage_executable(_executable: &Path) -> bool {
    true
}

#[cfg(not(unix))]
pub(super) fn should_stage_executable(_executable: &Path) -> bool {
    false
}

pub(super) fn simple_env_command<'a>(arguments: &'a [&'a str]) -> Option<&'a str> {
    let command = *arguments.first()?;
    (!command.starts_with('-') && !command.contains('=')).then_some(command)
}

pub(super) fn hash_executable_and_first_line(
    file: &mut fs::File,
) -> Option<(String, Option<Vec<u8>>)> {
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
pub(super) fn open_pinned_executable(executable: &Path) -> Option<fs::File> {
    fs::File::open(executable).ok()
}

#[cfg(windows)]
pub(super) fn open_pinned_executable(executable: &Path) -> Option<fs::File> {
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

pub(super) fn is_executable_file(path: &Path) -> bool {
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

pub(super) fn execution_environment_digest(path: &Path) -> Result<String, ObserverError> {
    execution_environment_digest_from_values(std::env::vars_os(), path)
}

pub(super) fn execution_environment_digest_from_values<I>(
    values: I,
    path: &Path,
) -> Result<String, ObserverError>
where
    I: IntoIterator<Item = (std::ffi::OsString, std::ffi::OsString)>,
{
    let mut values = values
        .into_iter()
        // Shell/mise and Agent host session bookkeeping values do not describe
        // the command's execution inputs. Including them makes an otherwise
        // exact receipt stale whenever a new shell or Agent turn is entered.
        // Keep all actual command/toolchain variables (PATH, PWD, TMPDIR,
        // CARGO_HOME, RUSTFLAGS, etc.) in the identity below.
        .filter(|(name, _)| !volatile_environment_key(name.as_encoded_bytes()))
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

pub(super) fn volatile_environment_key(name: &[u8]) -> bool {
    // `_`, `OLDPWD`, and `SHLVL` are shell bookkeeping rather than stable
    // command inputs. mise's entire `__MISE_*` namespace and the Agent's
    // `CODEX_*` namespace are session state. PATH and explicit toolchain
    // identities remain authoritative.
    name == b"_"
        || name == b"OLDPWD"
        || name == b"SHLVL"
        || name.starts_with(b"__MISE_")
        || name.starts_with(b"CODEX_")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerificationReuseAssessment {
    Authorized(Box<VerificationReuseAuthorization>),
    Denied { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationReuseAuthorization {
    pub context: cockpit_evidence::EvidenceContext,
    pub(super) config_file_digest: String,
    pub(super) profile_file_digest: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct VerificationIdentityCost {
    pub(super) files_read: usize,
    pub(super) files_hashed: usize,
}

pub(super) struct ResolvedExecutableIdentity {
    pub(super) path: String,
    pub(super) digest: String,
    pub(super) components: Vec<ExecutableComponent>,
    #[cfg(unix)]
    pub(super) _staging: tempfile::TempDir,
}

pub(super) struct ExecutableComponent {
    pub(super) path: String,
    #[cfg(unix)]
    pub(super) execution_path: String,
    pub(super) digest: String,
    #[cfg(unix)]
    pub(super) first_line: Option<Vec<u8>>,
    pub(super) _file: fs::File,
}

/// A lifecycle boundary at which repository facts are allowed to be reused.
/// A context captured for one phase must not silently cross an execution or
/// persistence mutation boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservationPhase {
    BeforeGovernance,
    AfterExecution,
    BeforePersistence,
}

impl ObservationPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BeforeGovernance => "before_governance",
            Self::AfterExecution => "after_execution",
            Self::BeforePersistence => "before_persistence",
        }
    }
}

/// The consistency state of an explicitly captured observation phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservationConsistency {
    Stable,
}

/// Validated, request-scoped facts for one observation phase.  This is a
/// value object rather than a cache: `validate_current` must succeed before a
/// caller uses it after any possible repository or execution mutation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationContext {
    repository_id: Digest,
    runtime: Option<RuntimeContext>,
    phase: ObservationPhase,
    snapshot: RepositorySnapshot,
    snapshot_digest: Digest,
    configuration_digest: Digest,
    policy_digest: Option<Digest>,
    contract_digest: Option<Digest>,
    contract_path: Option<PathBuf>,
    project_governance: ProjectGovernanceFacts,
    observation: RepositoryObservation,
    unknowns: Vec<String>,
    consistency: ObservationConsistency,
    root: PathBuf,
}

impl ObservationContext {
    pub fn repository_id(&self) -> &Digest {
        &self.repository_id
    }

    pub fn runtime(&self) -> Option<&RuntimeContext> {
        self.runtime.as_ref()
    }

    pub const fn phase(&self) -> ObservationPhase {
        self.phase
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn snapshot(&self) -> &RepositorySnapshot {
        &self.snapshot
    }

    pub fn snapshot_digest(&self) -> &Digest {
        &self.snapshot_digest
    }

    pub fn configuration_digest(&self) -> &Digest {
        &self.configuration_digest
    }

    pub fn policy_digest(&self) -> Option<&Digest> {
        self.policy_digest.as_ref()
    }

    pub fn contract_digest(&self) -> Option<&Digest> {
        self.contract_digest.as_ref()
    }

    pub fn observation(&self) -> &RepositoryObservation {
        &self.observation
    }

    pub fn project_governance(&self) -> &ProjectGovernanceProjection {
        self.project_governance.projection()
    }

    pub(crate) fn project_governance_facts(&self) -> &ProjectGovernanceFacts {
        &self.project_governance
    }

    pub fn unknowns(&self) -> &[String] {
        &self.unknowns
    }

    pub const fn consistency(&self) -> ObservationConsistency {
        self.consistency
    }

    /// Confirm that the captured facts still describe the current repository.
    /// This deliberately takes a fresh observation boundary; a struct holding
    /// an old snapshot is not treated as an atomic multi-file transaction.
    pub fn validate_current(&self) -> Result<(), ObserverError> {
        let git = cockpit_git::GitRepository::discover(&self.root).map_err(|error| {
            ObserverError::State {
                path: self.root.clone(),
                message: error.to_string(),
            }
        })?;
        let current_snapshot = git.snapshot().map_err(|error| ObserverError::State {
            path: self.root.clone(),
            message: error.to_string(),
        })?;
        let current_snapshot_digest = snapshot_digest(&current_snapshot)?;
        let current_repository_id = repository_id(&self.root);
        let current_configuration_digest = governance_configuration_digest(&self.root)?;
        if current_repository_id != self.repository_id {
            return Err(ObserverError::State {
                path: self.root.join(".ai/cockpit.toml"),
                message: "observation phase repository identity changed".into(),
            });
        }
        if current_configuration_digest != self.configuration_digest {
            return Err(ObserverError::State {
                path: self.root.join(".ai"),
                message: "observation phase governance configuration changed".into(),
            });
        }
        if let (Some(path), Some(expected)) = (&self.contract_path, &self.contract_digest) {
            let current = contract_identity_digest(path)?;
            if &current != expected {
                return Err(ObserverError::State {
                    path: path.clone(),
                    message: "observation phase Contract identity changed".into(),
                });
            }
        }
        if current_snapshot_digest != self.snapshot_digest {
            return Err(ObserverError::State {
                path: self.root.clone(),
                message: "observation phase repository snapshot changed".into(),
            });
        }
        Ok(())
    }

    pub fn require_phase(&self, expected: ObservationPhase) -> Result<(), ObserverError> {
        if self.phase != expected {
            return Err(ObserverError::State {
                path: self.root.join(".ai"),
                message: format!(
                    "observation phase cannot be reused: captured={}, required={}",
                    self.phase.as_str(),
                    expected.as_str()
                ),
            });
        }
        self.validate_current()
    }
}

const GOVERNANCE_INPUT_PATHS: [&str; 6] = [
    ".ai/cockpit.toml",
    ".ai/project.json",
    ".ai/policy.json",
    ".ai/project/capabilities.json",
    ".ai/project/success_criteria.json",
    ".ai/project/profile-policy.json",
];

fn governance_configuration_digest(root: &Path) -> Result<Digest, ObserverError> {
    let mut bytes = Vec::new();
    for relative in GOVERNANCE_INPUT_PATHS {
        bytes.extend_from_slice(relative.as_bytes());
        bytes.push(0);
        let path = root.join(relative);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                bytes.extend_from_slice(b"symlink");
            }
            Ok(metadata) if metadata.file_type().is_file() => {
                bytes.extend_from_slice(&fs::read(&path).map_err(|source| {
                    ObserverError::Read {
                        path: path.clone(),
                        source,
                    }
                })?);
            }
            Ok(_) => bytes.extend_from_slice(b"invalid"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                bytes.extend_from_slice(b"missing");
            }
            Err(source) => {
                return Err(ObserverError::Read { path, source });
            }
        }
        bytes.push(0);
    }
    Ok(Digest::sha256_bytes(&bytes))
}

fn governance_policy_digest(root: &Path) -> Result<Option<Digest>, ObserverError> {
    let path = root.join(".ai/policy.json");
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(ObserverError::Read { path, source }),
    };
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Ok(Some(Digest::sha256_bytes(b"invalid-policy-file")));
    }
    let bytes = fs::read(&path).map_err(|source| ObserverError::Read { path, source })?;
    Ok(Some(Digest::sha256_bytes(&bytes)))
}

fn contract_identity_digest(path: &Path) -> Result<Digest, ObserverError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| ObserverError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(ObserverError::State {
            path: path.to_path_buf(),
            message: "observation Contract must be a regular non-symlink file".into(),
        });
    }
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(path).map_err(|source| ObserverError::Read {
            path: path.to_path_buf(),
            source,
        })?)
        .map_err(|error| ObserverError::State {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
    cockpit_protocol::digest_json(&value).map_err(|error| ObserverError::State {
        path: path.to_path_buf(),
        message: error.to_string(),
    })
}

/// Request-scoped repository state.  A context captures one immutable Git
/// snapshot and memoizes the derived observation for the lifetime of the
/// request.  Callers that need fresh facts must create a new context instead
/// of mutating or globally replacing this one.
pub struct RepositoryExecutionContext {
    root: PathBuf,
    repository_id: Digest,
    snapshot: RepositorySnapshot,
    observation: OnceLock<RepositoryObservation>,
    observation_guard: Mutex<()>,
}

impl std::fmt::Debug for RepositoryExecutionContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RepositoryExecutionContext")
            .field("root", &self.root)
            .field("repository_id", &self.repository_id)
            .field("snapshot", &self.snapshot)
            .finish_non_exhaustive()
    }
}

impl RepositoryExecutionContext {
    pub fn capture(root: &Path) -> Result<Self, ObserverError> {
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
        let repository_id = repository_id(&root);
        Ok(Self {
            root,
            repository_id,
            snapshot,
            observation: OnceLock::new(),
            observation_guard: Mutex::new(()),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn repository_id(&self) -> &Digest {
        &self.repository_id
    }

    pub fn snapshot(&self) -> &RepositorySnapshot {
        &self.snapshot
    }

    pub fn observe(&self) -> Result<&RepositoryObservation, ObserverError> {
        if let Some(observation) = self.observation.get() {
            return Ok(observation);
        }
        let _guard = self
            .observation_guard
            .lock()
            .map_err(|_| ObserverError::State {
                path: self.root.join(".ai"),
                message: "repository observation mutex was poisoned".into(),
            })?;
        if let Some(observation) = self.observation.get() {
            return Ok(observation);
        }
        let observation = observe_cached(&self.root, &self.snapshot)?;
        let _ = self.observation.set(observation);
        self.observation.get().ok_or_else(|| ObserverError::State {
            path: self.root.join(".ai"),
            message: "repository observation was not initialized".into(),
        })
    }

    /// Capture one validated set of facts for a lifecycle phase.  The
    /// before/after checks cover source state, attached repository identity,
    /// and governance configuration without widening the snapshot lifetime.
    pub fn observe_phase(
        &self,
        phase: ObservationPhase,
        runtime: Option<&RuntimeContext>,
        contract_digest: Option<Digest>,
    ) -> Result<ObservationContext, ObserverError> {
        self.observe_phase_with_bindings(phase, runtime, contract_digest, None)
    }

    pub fn observe_phase_with_contract(
        &self,
        phase: ObservationPhase,
        runtime: Option<&RuntimeContext>,
        contract_path: &Path,
    ) -> Result<ObservationContext, ObserverError> {
        let contract_path =
            fs::canonicalize(contract_path).map_err(|source| ObserverError::Read {
                path: contract_path.to_path_buf(),
                source,
            })?;
        if !contract_path.starts_with(&self.root) {
            return Err(ObserverError::State {
                path: contract_path,
                message: "observation Contract escapes repository root".into(),
            });
        }
        let contract_digest = contract_identity_digest(&contract_path)?;
        self.observe_phase_with_bindings(phase, runtime, Some(contract_digest), Some(contract_path))
    }

    fn observe_phase_with_bindings(
        &self,
        phase: ObservationPhase,
        runtime: Option<&RuntimeContext>,
        contract_digest: Option<Digest>,
        contract_path: Option<PathBuf>,
    ) -> Result<ObservationContext, ObserverError> {
        let observation = self.observe()?.clone();
        let captured_snapshot_digest = snapshot_digest(&self.snapshot)?;
        let project_governance =
            observe_project_governance(&self.root, &self.repository_id, &captured_snapshot_digest)?;
        let context = ObservationContext {
            repository_id: self.repository_id.clone(),
            runtime: runtime.cloned(),
            phase,
            snapshot: self.snapshot.clone(),
            snapshot_digest: captured_snapshot_digest,
            configuration_digest: governance_configuration_digest(&self.root)?,
            policy_digest: governance_policy_digest(&self.root)?,
            contract_digest,
            contract_path,
            project_governance: project_governance.clone(),
            observation,
            unknowns: project_governance.projection().unknowns.clone(),
            consistency: ObservationConsistency::Stable,
            root: self.root.clone(),
        };
        context.validate_current()?;
        Ok(context)
    }
}

/// Explicitly owned process session for repeated requests. It is not a
/// global current-repository slot: every lookup receives an explicit path,
/// and each entry contains an isolated request context. The caller chooses
/// when to refresh a context after a repository mutation.
#[derive(Default)]
pub struct RuntimeSession {
    contexts: Mutex<BTreeMap<PathBuf, Arc<RepositoryExecutionContext>>>,
}

impl std::fmt::Debug for RuntimeSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeSession")
            .field(
                "active_repositories",
                &self.active_repositories().unwrap_or_default(),
            )
            .finish_non_exhaustive()
    }
}

impl RuntimeSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(&self, root: &Path) -> Result<Arc<RepositoryExecutionContext>, ObserverError> {
        let canonical = fs::canonicalize(root).map_err(|source| ObserverError::Read {
            path: root.into(),
            source,
        })?;
        let mut contexts = self.contexts.lock().map_err(|_| ObserverError::State {
            path: canonical.clone(),
            message: "runtime session mutex was poisoned".into(),
        })?;
        if let Some(context) = contexts.get(&canonical) {
            return Ok(Arc::clone(context));
        }
        let context = Arc::new(RepositoryExecutionContext::capture(&canonical)?);
        contexts.insert(canonical, Arc::clone(&context));
        Ok(context)
    }

    pub fn refresh(&self, root: &Path) -> Result<Arc<RepositoryExecutionContext>, ObserverError> {
        let canonical = fs::canonicalize(root).map_err(|source| ObserverError::Read {
            path: root.into(),
            source,
        })?;
        let context = Arc::new(RepositoryExecutionContext::capture(&canonical)?);
        let mut contexts = self.contexts.lock().map_err(|_| ObserverError::State {
            path: canonical.clone(),
            message: "runtime session mutex was poisoned".into(),
        })?;
        contexts.insert(canonical, Arc::clone(&context));
        Ok(context)
    }

    pub fn unbind(&self, root: &Path) -> Result<bool, ObserverError> {
        let canonical = fs::canonicalize(root).map_err(|source| ObserverError::Read {
            path: root.into(),
            source,
        })?;
        let mut contexts = self.contexts.lock().map_err(|_| ObserverError::State {
            path: canonical.clone(),
            message: "runtime session mutex was poisoned".into(),
        })?;
        Ok(contexts.remove(&canonical).is_some())
    }

    pub fn active_repositories(&self) -> Result<Vec<PathBuf>, ObserverError> {
        self.contexts
            .lock()
            .map(|contexts| contexts.keys().cloned().collect())
            .map_err(|_| ObserverError::State {
                path: PathBuf::from(".ai"),
                message: "runtime session mutex was poisoned".into(),
            })
    }
}

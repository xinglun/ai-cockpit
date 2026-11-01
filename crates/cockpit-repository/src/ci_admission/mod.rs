//! Read-only evidence inspection and current-authority admission boundaries.

/// Host-selected source reader; never deserialize a program or trust decision
/// from imported evidence. The embedding host controls this descriptor.
pub struct PortableSourceTool {
    executable: std::path::PathBuf,
    executable_digest: cockpit_core::Digest,
}

pub struct PortableSourceRequest<'a> {
    pub input: &'a std::path::Path,
    pub expected_plan_digest: &'a cockpit_core::Digest,
    pub parameters: &'a serde_json::Value,
}

/// A live controlled invocation, never reconstructed from saved JSON/markers.
pub struct VerifiedPortableSource {
    _private: (),
}

pub struct AcceptedPortableEvidence {
    report: cockpit_protocol::ci_admission::PortableEvidenceAcceptance,
    _private: (),
}

impl PortableSourceTool {
    pub fn new(executable: std::path::PathBuf, executable_digest: cockpit_core::Digest) -> Self {
        Self {
            executable,
            executable_digest,
        }
    }

    pub fn observe(
        &self,
        root: &std::path::Path,
        work_item_id: &str,
        request: PortableSourceRequest<'_>,
        runtime: &cockpit_protocol::RuntimeContext,
        caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
    ) -> Result<VerifiedPortableSource, crate::ObserverError> {
        let _ = (
            &self.executable,
            &self.executable_digest,
            work_item_id,
            request.input,
            request.expected_plan_digest,
            request.parameters,
            runtime,
            caller,
        );
        Err(plan_error(
            root,
            "independent_source_observation_not_implemented",
        ))
    }
}

impl VerifiedPortableSource {
    pub fn accept(
        self,
        root: &std::path::Path,
        request: &crate::RepositoryVerificationRequest,
        runtime: &cockpit_protocol::RuntimeContext,
        caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
    ) -> Result<AcceptedPortableEvidence, crate::ObserverError> {
        let _ = (request, runtime, caller);
        Err(plan_error(
            root,
            "verified_source_acceptance_not_implemented",
        ))
    }
}

impl AcceptedPortableEvidence {
    pub fn report(&self) -> &cockpit_protocol::ci_admission::PortableEvidenceAcceptance {
        &self.report
    }

    pub fn consume(
        self,
        root: &std::path::Path,
        request: &crate::RepositoryVerificationRequest,
        runtime: &cockpit_protocol::RuntimeContext,
        caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
    ) -> Result<crate::RepositoryVerificationRun, crate::ObserverError> {
        let _ = (request, runtime, caller);
        Err(plan_error(root, "verified_source_consumer_not_implemented"))
    }
}

pub(super) mod archive;
mod github;
mod store;
#[cfg(test)]
mod tests;

use super::*;
use cockpit_protocol::ci_admission::{
    GITHUB_PLAN_SCHEMA_VERSION, GithubAdmissionPlan, GithubEvidenceSelector, GithubPlanAdmission,
    GithubPlanState,
};

fn plan_error(root: &Path, message: impl Into<String>) -> ObserverError {
    ObserverError::State {
        path: root.to_path_buf(),
        message: message.into(),
    }
}

/// Collect metadata directly from the fixed GitHub HTTPS origin and explain
/// every missing proof. This never accepts evidence, runs verification, writes
/// files, installs trust, or follows an artifact-supplied command or URL.
pub fn plan_github_verification_admission(
    root: &Path,
    work_item_id: &str,
    selector: &GithubEvidenceSelector,
    runtime: &RuntimeContext,
    caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
) -> Result<GithubAdmissionPlan, ObserverError> {
    plan_with_collection(
        root,
        work_item_id,
        selector,
        runtime,
        caller,
        github::collect,
    )
}

// Private injection boundary for deterministic collection/authority tests. The
// public service always uses the fixed-origin HTTPS collector above.
fn plan_with_collection<F>(
    root: &Path,
    work_item_id: &str,
    selector: &GithubEvidenceSelector,
    runtime: &RuntimeContext,
    caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
    collect: F,
) -> Result<GithubAdmissionPlan, ObserverError>
where
    F: FnOnce(&GithubEvidenceSelector) -> github::Collected,
{
    validate_work_item_id(work_item_id)?;
    selector.validate().map_err(|e| plan_error(root, e))?;
    require_current_action_admission_with_caller(
        root,
        work_item_id,
        "run_verification",
        runtime,
        caller,
    )?;
    let before = work_item_status_snapshot_with_runtime(root, work_item_id, runtime)?;
    let contract_digest = before
        .source_digests
        .get("contract")
        .ok_or_else(|| plan_error(root, "current Contract binding is missing"))?
        .clone();
    let snapshot_digest = before
        .source_digests
        .get("repositorySnapshot")
        .ok_or_else(|| plan_error(root, "current snapshot binding is missing"))?
        .clone();
    let observed = collect(selector);
    // Recheck present authority after network I/O. A stale plan cannot become
    // an operation token, and expiry cannot be hidden behind a slow provider.
    task_authorization::require_task_caller(
        root,
        work_item_id,
        "run_verification",
        runtime,
        caller,
    )?;
    let after = work_item_status_snapshot_with_runtime(root, work_item_id, runtime)?;
    let mut gaps = vec![
        "reviewed_trust_policy_unavailable".into(),
        "workflow_execution_closure_unverified".into(),
        "producer_runtime_unverified".into(),
        "artifact_job_attempt_binding_unavailable".into(),
        "native_receipt_not_inspected".into(),
        "current_consumer_command_binding_unavailable".into(),
        "admission_not_implemented".into(),
    ];
    if before.source_digests.get("contract") != after.source_digests.get("contract")
        || before.source_digests.get("repositorySnapshot")
            != after.source_digests.get("repositorySnapshot")
    {
        gaps.push("consumer_bindings_changed_during_collection".into());
    }
    if let Some(failure) = &observed.collection.failure {
        gaps.push(failure.clone());
    }
    gaps.extend(observed.diagnostic_gaps);
    let state = if observed.rejected {
        GithubPlanState::MetadataRejected
    } else if observed.collection.complete {
        GithubPlanState::Gaps
    } else {
        GithubPlanState::ProviderUnavailable
    };
    Ok(GithubAdmissionPlan {
        schema_version: GITHUB_PLAN_SCHEMA_VERSION,
        work_item_id: work_item_id.into(),
        repository_id: before.repository_id,
        contract_digest,
        consumer_snapshot_digest: snapshot_digest,
        consumer_runtime_digest: runtime.runtime_digest.clone(),
        selector: selector.clone(),
        state,
        collection: observed.collection,
        metadata: observed.metadata,
        gaps,
        admission: GithubPlanAdmission::NotAttempted,
        verification_commands_executed: 0,
        writes_performed: 0,
    })
}

/// Read a caller-selected local file under a fixed byte limit. Opening a Unix
/// FIFO must not block even if the path changes after the initial metadata read.
#[cfg(any(unix, windows))]
fn portable_input_bytes(input: &Path) -> Result<Vec<u8>, ObserverError> {
    use std::io::Read as _;
    let read_error = |source| ObserverError::Read {
        path: input.to_path_buf(),
        source,
    };
    #[cfg(windows)]
    {
        use std::path::{Component, Prefix};
        if matches!(input.components().next(), Some(Component::Prefix(prefix))
            if !matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)))
        {
            return Err(plan_error(input, "portable_input_not_local_file_path"));
        }
    }
    let before = fs::symlink_metadata(input).map_err(read_error)?;
    if !before.is_file()
        || before.file_type().is_symlink()
        || before.len() > store::MAX_PORTABLE_INPUT_BYTES
    {
        return Err(plan_error(input, "portable_input_not_bounded_regular_file"));
    }
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt as _;
        options.custom_flags(windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let file = options.open(input).map_err(read_error)?;
    let opened = file.metadata().map_err(read_error)?;
    if !portable_metadata_matches(&before, &opened) {
        return Err(plan_error(input, "portable_input_changed_before_read"));
    }
    let expected_size = usize::try_from(opened.len())
        .map_err(|_| plan_error(input, "portable_input_size_not_addressable"))?;
    let mut bytes = Vec::with_capacity(expected_size);
    let mut reader = (&file).take(opened.len() + 1);
    let mut chunk = [0_u8; 8192];
    loop {
        let count = reader.read(&mut chunk).map_err(read_error)?;
        if count == 0 {
            break;
        }
        if count > expected_size - bytes.len() {
            return Err(plan_error(
                input,
                "portable_input_changed_or_exceeded_limit",
            ));
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
    let after = file.metadata().map_err(read_error)?;
    let path_after = fs::symlink_metadata(input).map_err(read_error)?;
    if bytes.len() as u64 > store::MAX_PORTABLE_INPUT_BYTES
        || bytes.len() as u64 != opened.len()
        || !portable_metadata_matches(&opened, &after)
        || !portable_metadata_matches(&after, &path_after)
    {
        return Err(plan_error(
            input,
            "portable_input_changed_or_exceeded_limit",
        ));
    }
    Ok(bytes)
}

#[cfg(any(unix, windows))]
fn portable_metadata_matches(before: &fs::Metadata, after: &fs::Metadata) -> bool {
    if !before.is_file()
        || !after.is_file()
        || before.file_type().is_symlink()
        || after.file_type().is_symlink()
        || before.len() != after.len()
        || before.modified().ok().is_none()
        || before.modified().ok() != after.modified().ok()
    {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        before.dev() == after.dev()
            && before.ino() == after.ino()
            && before.ctime() == after.ctime()
            && before.ctime_nsec() == after.ctime_nsec()
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt as _;
        let reparse = windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
        before.file_attributes() & reparse == 0
            && after.file_attributes() & reparse == 0
            && before.creation_time() == after.creation_time()
            && before.last_write_time() == after.last_write_time()
            && before.file_attributes() == after.file_attributes()
    }
}

#[cfg(not(any(unix, windows)))]
fn portable_input_bytes(input: &Path) -> Result<Vec<u8>, ObserverError> {
    Err(plan_error(
        input,
        "portable_input_file_observation_unsupported_on_platform",
    ))
}

/// Inspect supplied data without requiring permission to execute a command.
/// Even a complete cold graph and every caller-supplied source flag remain unverified.
pub fn plan_portable_evidence(
    root: &Path,
    work_item_id: Option<&str>,
    input: &Path,
    runtime: &RuntimeContext,
    caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
) -> Result<cockpit_protocol::ci_admission::PortableEvidencePlan, ObserverError> {
    use cockpit_protocol::ci_admission::{
        PORTABLE_EVIDENCE_SCHEMA_VERSION, PortableEvidencePlan, PortableSourceVerification,
    };
    let _ = caller; // Inspection is not an execution-authority request.
    let raw = portable_input_bytes(input)?;
    let input_digest = Digest::sha256_bytes(&raw);
    let view = archive::inspect_untrusted_portable_evidence(&raw)
        .map_err(|error| plan_error(input, format!("portable_evidence_invalid: {error}")))?;
    let (consumer_repository_id, contract_digest, snapshot_digest) = match work_item_id {
        Some(id) => {
            validate_work_item_id(id)?;
            let status = work_item_status_snapshot_with_runtime(root, id, runtime)?;
            let contract = status
                .source_digests
                .get("contract")
                .ok_or_else(|| plan_error(root, "current Contract binding is missing"))?
                .clone();
            let snapshot = status
                .source_digests
                .get("repositorySnapshot")
                .ok_or_else(|| plan_error(root, "current snapshot binding is missing"))?
                .clone();
            (Some(status.repository_id), Some(contract), Some(snapshot))
        }
        None => (None, None, None),
    };
    let mut gaps = vec!["independent_execution_source_unavailable".to_string()];
    gaps.extend(view.gaps);
    match work_item_id {
        Some(id)
            if view.claims["workItemId"] != id
                || view.claims["repositoryId"] != serde_json::json!(consumer_repository_id)
                || view.claims["contractDigest"] != serde_json::json!(contract_digest)
                || view.claims["repositorySnapshotDigest"]
                    != serde_json::json!(snapshot_digest)
                || view.claims["runtimeVersion"] != runtime.runtime_version
                || view.claims["runtimeDigest"] != runtime.runtime_digest.as_str() =>
        {
            gaps.push("native_consumer_binding_mismatch".into());
        }
        None => gaps.push("consumer_work_item_not_bound".into()),
        _ => {}
    }
    // The exact consumer and executing Runtime are part of the plan. A plan
    // digest is an optimistic-concurrency check, never a source credential.
    let plan_digest = cockpit_protocol::digest_json(&serde_json::json!({
        "domain":"portable-evidence-plan-v1", "schemaVersion":PORTABLE_EVIDENCE_SCHEMA_VERSION,
        "inputDigest":input_digest, "workItemId":work_item_id,
        "consumerRepositoryId":consumer_repository_id,
        "consumerContractDigest":contract_digest, "consumerSnapshotDigest":snapshot_digest,
        "consumerRuntimeVersion":runtime.runtime_version,
        "consumerRuntimeDigest":runtime.runtime_digest,
        "consumerProtocolVersion":runtime.protocol_version,
        "integrityValid":view.integrity_valid, "claims":view.claims, "gaps":gaps,
    }))
    .map_err(|error| plan_error(root, error.to_string()))?;
    Ok(PortableEvidencePlan {
        schema_version: PORTABLE_EVIDENCE_SCHEMA_VERSION,
        input_digest,
        plan_digest,
        work_item_id: work_item_id.map(str::to_string),
        consumer_contract_digest: contract_digest,
        consumer_snapshot_digest: snapshot_digest,
        source_verification: PortableSourceVerification::Unverified,
        may_replace_execution: false,
        integrity_valid: view.integrity_valid,
        claims: view.claims,
        gaps,
        verification_commands_executed: 0,
        writes_performed: 0,
    })
}

/// Cold import has no independent invocation witness. Current authority and a
/// matching plan are necessary checks, and cannot turn that absence into proof.
pub fn accept_portable_evidence(
    root: &Path,
    work_item_id: &str,
    input: &Path,
    expected_plan_digest: &Digest,
    runtime: &RuntimeContext,
    caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
) -> Result<cockpit_protocol::ci_admission::PortableEvidenceAcceptance, ObserverError> {
    use cockpit_protocol::ci_admission::{
        PORTABLE_EVIDENCE_SCHEMA_VERSION, PortableEvidenceAcceptance, PortableSourceVerification,
    };
    validate_work_item_id(work_item_id)?;
    require_current_action_admission_with_caller(
        root,
        work_item_id,
        "run_verification",
        runtime,
        caller,
    )?;
    let plan = plan_portable_evidence(root, Some(work_item_id), input, runtime, caller)?;
    if &plan.plan_digest != expected_plan_digest {
        return Err(plan_error(root, "portable_evidence_plan_changed"));
    }
    // Recheck after all parsing/observation. No approval, marker, archive or
    // external path can supply the private process-local source witness.
    require_current_action_admission_with_caller(
        root,
        work_item_id,
        "run_verification",
        runtime,
        caller,
    )?;
    let current = plan_portable_evidence(root, Some(work_item_id), input, runtime, caller)?;
    if current.plan_digest != plan.plan_digest || &current.plan_digest != expected_plan_digest {
        return Err(plan_error(root, "portable_evidence_plan_changed"));
    }
    require_current_action_admission_with_caller(
        root,
        work_item_id,
        "run_verification",
        runtime,
        caller,
    )?;
    Ok(PortableEvidenceAcceptance {
        schema_version: PORTABLE_EVIDENCE_SCHEMA_VERSION,
        input_digest: current.input_digest,
        plan_digest: current.plan_digest,
        accepted: false,
        source_verification: PortableSourceVerification::Unverified,
        admission_ref: None,
        gaps: current.gaps,
        verification_commands_executed: 0,
        writes_performed: 0,
    })
}

/// Internal storage bridge for the root's opaque native invocation publisher.
/// This function returns a record reference, never a source/acceptance capability.
pub(super) fn persist_portable_native_observation(
    root: &Path,
    wi: &str,
    record_id: &str,
    input: &[u8],
    marker: &[u8],
) -> Result<String, ObserverError> {
    let published = store::publish_portable_record(root, wi, record_id, input, marker)?;
    // Read through the cold boundary after the publishing lock is released.
    // This checks the installed record; it never reconstructs a source witness.
    let loaded = store::load_portable_record(root, wi, record_id)?
        .ok_or_else(|| plan_error(root, "portable_publication_readback_missing"))?;
    if loaded != published {
        return Err(plan_error(root, "portable_publication_readback_changed"));
    }
    Ok(loaded.record_ref)
}

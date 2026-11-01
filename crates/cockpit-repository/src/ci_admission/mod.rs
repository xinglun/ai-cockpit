//! Portable evidence inspection and source-checked admission boundaries.

/// Selected by the trusted embedding host, never from imported packet fields.
/// A malicious embedding host is outside this application trust boundary.
#[derive(Clone)]
pub struct PortableSourceTool {
    executable: std::path::PathBuf,
    executable_digest: cockpit_core::Digest,
}

pub struct PortableSourceRequest<'a> {
    pub input: &'a std::path::Path,
    pub expected_plan_digest: &'a cockpit_core::Digest,
    pub parameters: &'a serde_json::Value,
}

/// Not Clone/Deserialize: a saved response or marker cannot recreate a session.
pub struct VerifiedPortableSource {
    tool: PortableSourceTool,
    root: std::path::PathBuf,
    work_item_id: String,
    input: std::path::PathBuf,
    raw: Vec<u8>,
    plan: cockpit_protocol::ci_admission::PortableEvidencePlan,
    parameters: serde_json::Value,
    source_root: cockpit_core::Digest,
    runtime: cockpit_protocol::RuntimeContext,
    caller: Option<cockpit_protocol::TaskAuthorizationCaller>,
    observed: std::time::Instant,
    started: std::time::Instant,
    trace_ref: String,
}

pub struct AcceptedPortableEvidence {
    report: cockpit_protocol::ci_admission::PortableEvidenceAcceptance,
    source: VerifiedPortableSource,
    request: crate::RepositoryVerificationRequest,
    record_id: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SourceReply {
    schema_version: u32,
    nonce: String,
    input_digest: cockpit_core::Digest,
    source_root: cockpit_core::Digest,
}

fn source_error(root: &Path, code: &str) -> ObserverError {
    plan_error(
        root,
        format!(
            "{code}; evidence remains available for inspection; check its source or run the declared check fresh under current authority"
        ),
    )
}

impl PortableSourceTool {
    /// A production adapter may select only this held, current Runtime.
    /// No imported packet or public request chooses executable bytes.
    pub fn for_current_runtime(
        root: &Path,
        runtime: &RuntimeContext,
    ) -> Result<Self, ObserverError> {
        let executable = std::env::current_exe()
            .map_err(|_| source_error(root, "current_source_tool_unavailable"))?;
        let mut held = fs::File::open(&executable).map_err(|source| ObserverError::Read {
            path: executable.clone(),
            source,
        })?;
        if execution_context::hash_executable_and_first_line(&mut held)
            .is_none_or(|(digest, _)| digest != runtime.runtime_digest.as_str())
        {
            return Err(source_error(root, "current_source_tool_runtime_mismatch"));
        }
        Ok(Self::new(executable, runtime.runtime_digest.clone()))
    }

    pub fn new(executable: std::path::PathBuf, executable_digest: cockpit_core::Digest) -> Self {
        Self {
            executable,
            executable_digest,
        }
    }

    pub fn observe(
        &self,
        root: &Path,
        work_item_id: &str,
        request: PortableSourceRequest<'_>,
        runtime: &RuntimeContext,
        caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
    ) -> Result<VerifiedPortableSource, ObserverError> {
        let started = std::time::Instant::now();
        use cockpit_verification::{VerificationCommand, VerificationReusePolicy};
        use std::sync::atomic::{AtomicU64, Ordering};
        static SESSION: AtomicU64 = AtomicU64::new(1);
        require_current_action_admission_with_caller(
            root,
            work_item_id,
            "run_verification",
            runtime,
            caller,
        )?;
        let root = fs::canonicalize(root).map_err(|source| ObserverError::Read {
            path: root.into(),
            source,
        })?;
        let plan =
            plan_portable_evidence(&root, Some(work_item_id), request.input, runtime, caller)?;
        if &plan.plan_digest != request.expected_plan_digest {
            return Err(source_error(&root, "portable_evidence_plan_changed"));
        }
        let raw = portable_input_bytes(request.input)?;
        if Digest::sha256_bytes(&raw) != plan.input_digest {
            return Err(source_error(&root, "portable_input_changed"));
        }
        let metadata =
            fs::symlink_metadata(&self.executable).map_err(|source| ObserverError::Read {
                path: self.executable.clone(),
                source,
            })?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(source_error(&root, "source_tool_not_regular"));
        }
        let executable = self
            .executable
            .to_str()
            .ok_or_else(|| source_error(&root, "source_tool_path_invalid"))?;
        let pin = resolved_executable_identity(&root, executable)
            .ok_or_else(|| source_error(&root, "source_tool_cannot_be_pinned"))?;
        if !pin.components.iter().any(|component| {
            component.path == pin.path && component.digest == self.executable_digest.as_str()
        }) {
            return Err(source_error(&root, "source_tool_digest_changed"));
        }
        let nonce = cockpit_protocol::digest_json(&serde_json::json!([
            "portable-source-session-v1",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            SESSION.fetch_add(1, Ordering::Relaxed),
        ]))
        .map_err(|_| source_error(&root, "source_session_serialization"))?
        .to_string();
        let wire = serde_json::to_string(&serde_json::json!({
            "schemaVersion":1, "nonce":nonce, "inputDigest":plan.input_digest,
            "parameters":request.parameters,
            "repositoryRoot":root,"workItemId":work_item_id,
            "taskCaller":caller,"observerToolDigest":self.executable_digest,
        }))
        .map_err(|_| source_error(&root, "source_request_serialization"))?;
        if wire.len() > 16 * 1024 {
            return Err(source_error(&root, "source_request_byte_limit"));
        }
        let arguments = vec!["__portable-source-observe".into(), wire];
        let (program, args) = pin
            .execution(&arguments)
            .ok_or_else(|| source_error(&root, "source_tool_execution_unavailable"))?;
        let command = VerificationCommand::new_pinned(
            "portable-source-reader",
            &program,
            args,
            executable,
            arguments,
            VerificationReusePolicy::NeverReuse,
        )
        .with_current_dir(&root)
        .with_timeout_seconds(120);
        let result = cockpit_verification::execute_bounded(vec![command], 1)
            .map_err(|_| source_error(&root, "source_tool_execution_failed"))?;
        let trace_ref = persist_source_data(
            &root,
            work_item_id,
            "portable-source-reader-observation",
            &serde_json::json!({
                "inputDigest":plan.input_digest,"toolDigest":self.executable_digest,
                "parametersDigest":cockpit_protocol::digest_json(request.parameters).map_err(|_|source_error(&root,"source_parameters_serialization"))?,
                "receipt":result,
            }),
        )?;
        if !result.passed
            || result.processes_spawned != 1
            || result.nodes_executed != 1
            || result.nodes_reused != 0
            || result.execution_records.len() != 1
        {
            return Err(source_error(&root, "source_tool_incomplete_or_failed"));
        }
        let record = &result.execution_records[0];
        if !record.spawned
            || !record.passed
            || record.exit_code != Some(0)
            || record.termination_signal.is_some()
            || record.timed_out
            || record.stdout_truncated
            || record.stderr_truncated
        {
            return Err(source_error(&root, "source_tool_incomplete_or_failed"));
        }
        // Rehash the held original executable/interpreter pins after execution.
        for component in &pin.components {
            let mut file = component
                ._file
                .try_clone()
                .map_err(|source| ObserverError::Read {
                    path: self.executable.clone(),
                    source,
                })?;
            file.seek(std::io::SeekFrom::Start(0))
                .map_err(|source| ObserverError::Read {
                    path: self.executable.clone(),
                    source,
                })?;
            if execution_context::hash_executable_and_first_line(&mut file)
                .is_none_or(|(digest, _)| digest != component.digest)
            {
                return Err(source_error(&root, "source_tool_bytes_changed"));
            }
        }
        let output = hex::decode(&record.stdout_hex)
            .map_err(|_| source_error(&root, "source_tool_output_invalid"))?;
        if output.len() > 8192 {
            return Err(source_error(&root, "source_tool_output_byte_limit"));
        }
        reject_duplicate_json_keys(&output)
            .map_err(|_| source_error(&root, "source_tool_output_duplicate_or_invalid"))?;
        let reply: SourceReply = serde_json::from_slice(&output)
            .map_err(|_| source_error(&root, "source_tool_output_schema"))?;
        if reply.schema_version != 1
            || reply.nonce != nonce
            || reply.input_digest != plan.input_digest
            || reply.source_root.as_str().parse::<Digest>().is_err()
        {
            return Err(source_error(&root, "source_tool_response_binding_mismatch"));
        }
        require_current_action_admission_with_caller(
            &root,
            work_item_id,
            "run_verification",
            runtime,
            caller,
        )?;
        if plan_portable_evidence(&root, Some(work_item_id), request.input, runtime, caller)?
            .plan_digest
            != plan.plan_digest
        {
            return Err(source_error(&root, "portable_evidence_plan_changed"));
        }
        Ok(VerifiedPortableSource {
            tool: self.clone(),
            root,
            work_item_id: work_item_id.into(),
            input: request.input.into(),
            raw,
            plan,
            parameters: request.parameters.clone(),
            source_root: reply.source_root,
            runtime: runtime.clone(),
            caller: caller.cloned(),
            observed: std::time::Instant::now(),
            started,
            trace_ref,
        })
    }
}

impl VerifiedPortableSource {
    fn validate(
        &self,
        root: &Path,
        request: &RepositoryVerificationRequest,
        runtime: &RuntimeContext,
        caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
    ) -> Result<
        (
            RepositorySnapshot,
            cockpit_verification::VerificationReceipt,
        ),
        ObserverError,
    > {
        if fs::canonicalize(root).ok().as_ref() != Some(&self.root)
            || request.work_item_id.as_deref() != Some(self.work_item_id.as_str())
            || runtime.runtime_digest != self.runtime.runtime_digest
            || runtime.runtime_version != self.runtime.runtime_version
            || runtime.protocol_version != self.runtime.protocol_version
            || serde_json::to_value(caller).ok() != serde_json::to_value(&self.caller).ok()
            || self.observed.elapsed() > std::time::Duration::from_secs(120)
        {
            return Err(source_error(root, "source_session_changed_or_expired"));
        }
        require_current_action_admission_with_caller(
            root,
            &self.work_item_id,
            "run_verification",
            runtime,
            caller,
        )?;
        task_authorization::require_declared_task_command(root, request, runtime, caller)?;
        let current_plan =
            plan_portable_evidence(root, Some(&self.work_item_id), &self.input, runtime, caller)?;
        if current_plan.plan_digest != self.plan.plan_digest
            || !current_plan.integrity_valid
            || current_plan
                .gaps
                .iter()
                .any(|gap| gap != "independent_execution_source_unavailable")
        {
            return Err(source_error(
                root,
                "source_packet_or_consumer_binding_mismatch",
            ));
        }
        let packet: cockpit_protocol::ci_admission::PortableExecutionEvidence<
            cockpit_protocol::ci_admission::NativeExecutionEnvelope<
                cockpit_verification::VerificationReceipt,
            >,
        > = serde_json::from_slice(&self.raw)
            .map_err(|_| source_error(root, "source_packet_schema"))?;
        if !native_physical_receipt_complete(&packet.native_evidence.receipt, request) {
            return Err(source_error(
                root,
                "source_receipt_not_complete_physical_success",
            ));
        }
        let snapshot = cockpit_git::GitRepository::discover(root)
            .and_then(|git| git.snapshot())
            .map_err(|_| source_error(root, "current_source_unavailable"))?;
        require_verification_preconditions_with_caller(
            root,
            &self.work_item_id,
            runtime,
            &snapshot,
            caller,
        )?;
        validate_verification_request(root, request)?;
        let prepared = prepare_repository_verification_command(root, request, &snapshot)?;
        let pin = prepared
            .execution_identity
            .as_ref()
            .ok_or_else(|| source_error(root, "current_executable_unavailable"))?;
        let authorization = prepared
            .authorized_binding
            .as_ref()
            .ok_or_else(|| source_error(root, "current_profile_not_authorized"))?;
        let expected = native_current_input(
            root,
            request,
            &snapshot,
            runtime,
            &prepared.command,
            pin,
            authorization,
        )?;
        // Ephemeral executable-copy pathnames are producer provenance, not
        // declared command inputs. Preserve them in the original packet. The
        // complete logical executionInput, executable component identities and
        // full effective environment still compare exactly; no hash is changed.
        for (key, value) in expected
            .as_object()
            .ok_or_else(|| source_error(root, "current_input_schema"))?
        {
            if key != "physicalProgram"
                && key != "physicalArgs"
                && packet.native_input.get(key) != Some(value)
            {
                return Err(source_error(
                    root,
                    &format!("source_execution_input_mismatch:{key}"),
                ));
            }
        }
        if packet.native_evidence.receipt.execution_records[0].command_digest
            != prepared.command.command_digest()
        {
            return Err(source_error(root, "source_command_digest_mismatch"));
        }
        let local = lifecycle::capture_local_node_input(
            root,
            request,
            &snapshot,
            runtime,
            &prepared.command,
            Some(pin),
            Some(authorization),
        )?;
        if lifecycle::local_profile_cache_requires_fresh(root, &local, None) {
            return Err(source_error(
                root,
                "newer_failed_pending_or_unproved_execution",
            ));
        }
        Ok((snapshot, packet.native_evidence.receipt))
    }

    pub fn accept(
        self,
        root: &Path,
        request: &RepositoryVerificationRequest,
        runtime: &RuntimeContext,
        caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
    ) -> Result<AcceptedPortableEvidence, ObserverError> {
        self.validate(root, request, runtime, caller)?;
        let record_id = cockpit_protocol::digest_json(&serde_json::json!({
            "domain":"portable-source-admission-v1", "inputDigest":self.plan.input_digest,
            "planDigest":self.plan.plan_digest, "sourceRoot":self.source_root,
            "toolDigest":self.tool.executable_digest, "nodeId":request.node_id, "sourceTrace":self.trace_ref,
        }))
        .map_err(|_| source_error(root, "source_admission_identity"))?
        .to_string();
        let marker = serde_json::to_vec(&serde_json::json!({"schemaVersion":1,
            "recordId":record_id, "inputDigest":self.plan.input_digest}))
        .map_err(|_| source_error(root, "source_admission_marker"))?;
        let record_ref = persist_portable_native_observation(
            root,
            &self.work_item_id,
            &record_id,
            &self.raw,
            &marker,
        )?;
        // A failed recheck does not roll back an already-persisted record.
        self.validate(root, request, runtime, caller)?;
        Ok(AcceptedPortableEvidence {
            report:cockpit_protocol::ci_admission::PortableEvidenceAcceptance {
                schema_version:cockpit_protocol::ci_admission::PORTABLE_EVIDENCE_SCHEMA_VERSION,
                input_digest:self.plan.input_digest.clone(), plan_digest:self.plan.plan_digest.clone(),
                accepted:true, source_verification:cockpit_protocol::ci_admission::PortableSourceVerification::ControlledToolObservation,
                admission_ref:Some(record_ref), gaps:Vec::new(), verification_commands_executed:0, writes_performed:1,
                source_use:None,guidance:None,
            }, source:self, request:request.clone(), record_id,
        })
    }
}

impl AcceptedPortableEvidence {
    pub fn report(&self) -> &cockpit_protocol::ci_admission::PortableEvidenceAcceptance {
        &self.report
    }

    pub fn consume(
        self,
        root: &Path,
        request: &RepositoryVerificationRequest,
        runtime: &RuntimeContext,
        caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
    ) -> Result<RepositoryVerificationRun, ObserverError> {
        self.consume_with_source_report(root, request, runtime, caller)
            .map(|(run, _)| run)
    }

    pub fn consume_with_source_report(
        self,
        root: &Path,
        request: &RepositoryVerificationRequest,
        runtime: &RuntimeContext,
        caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
    ) -> Result<
        (
            RepositoryVerificationRun,
            cockpit_protocol::ci_admission::PortableEvidenceSourceUse,
        ),
        ObserverError,
    > {
        if *request != self.request {
            return Err(source_error(root, "accepted_node_request_changed"));
        }
        self.source.validate(root, request, runtime, caller)?;
        // Independently re-read at the consumption boundary. A retained file
        // or an old live session is not a current-origin observation.
        let fresh = self.source.tool.observe(
            root,
            &self.source.work_item_id,
            PortableSourceRequest {
                input: &self.source.input,
                expected_plan_digest: &self.source.plan.plan_digest,
                parameters: &self.source.parameters,
            },
            runtime,
            caller,
        )?;
        if fresh.source_root != self.source.source_root {
            return Err(source_error(root, "origin_changed_before_consumption"));
        }
        let stored = store::load_portable_record(root, &self.source.work_item_id, &self.record_id)?
            .ok_or_else(|| source_error(root, "accepted_record_missing"))?;
        if stored.input != self.source.raw
            || Some(&stored.record_ref) != self.report.admission_ref.as_ref()
        {
            return Err(source_error(root, "accepted_record_changed"));
        }
        let (final_snapshot, mut receipt) = fresh.validate(root, request, runtime, caller)?;
        receipt.results[0].reused = true;
        receipt.results[0].action = cockpit_verification::PlannedAction::Reuse;
        receipt.results[0].state = cockpit_verification::PlannedState::Fresh;
        receipt.results[0].reason = "fresh_controlled_source_exact_binding".into();
        receipt.results[0].receipt_id = Some(self.record_id.clone());
        receipt.results[0].satisfied_by =
            cockpit_verification::PlannedSatisfaction::PortableSourceObservation {
                admission_ref: stored.record_ref,
                source_root: fresh.source_root.to_string(),
            };
        receipt.execution_records.clear();
        receipt.receipt_candidates.clear();
        receipt.nodes_planned = 1;
        receipt.nodes_executed = 0;
        receipt.nodes_reused = 1;
        receipt.processes_spawned = 0;
        receipt.max_concurrent_processes = 0;
        receipt.process_spawn_failures = 0;
        receipt.rerun_stale = 0;
        receipt.rerun_unknown = 0;
        receipt.protected_nodes_executed = 0;
        receipt.protected_nodes_skipped = 0;
        receipt.planning_elapsed_ms = 0;
        receipt.execution_elapsed_ms = 0;
        receipt.elapsed_ms = 0;
        receipt.plan_receipt = None;
        receipt.cost_observation = None;
        let trace_refs = vec![self.source.trace_ref.clone(), fresh.trace_ref.clone()];
        let receipt_value = serde_json::to_value(&receipt)
            .map_err(|_| source_error(root, "source_consumption_serialization"))?;
        let consumption_record_ref = persist_source_data(
            root,
            &self.source.work_item_id,
            "portable-source-node-consumption",
            &serde_json::json!({
                "nodeId":request.node_id,"inputDigest":self.source.plan.input_digest,
                "planDigest":self.source.plan.plan_digest,"sourceRoot":fresh.source_root,
                "sourceRecordRefs":trace_refs,"receipt":receipt_value,
            }),
        )?;
        // A late failure leaves immutable observations, never an approved result.
        fresh.validate(root, request, runtime, caller)?;
        let source_use = cockpit_protocol::ci_admission::PortableEvidenceSourceUse {
            node_id: request.node_id.clone(),
            source_record_refs: trace_refs,
            consumption_record_ref,
            receipt: receipt_value,
            source_reader_invocations: 2,
            operation_elapsed_ms: self.source.started.elapsed().as_millis(),
        };
        Ok((
            RepositoryVerificationRun {
                receipt,
                final_snapshot,
            },
            source_use,
        ))
    }
}

pub struct PortableSourceAdmissionRequest<'a> {
    pub work_item_id: &'a str,
    pub input: &'a Path,
    pub expected_plan_digest: &'a Digest,
    pub parameters: &'a serde_json::Value,
    pub node_id: Option<&'a str>,
}

/// Current gate of the actual controlled Runtime, not a copied old projection.
pub fn require_portable_source_observation_admission(
    root: &Path,
    wi: &str,
    runtime: &RuntimeContext,
    caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
) -> Result<RepositorySnapshot, ObserverError> {
    validate_work_item_id(wi)?;
    require_current_action_admission_with_caller(root, wi, "run_verification", runtime, caller)?;
    let snapshot = cockpit_git::GitRepository::discover(root)
        .and_then(|g| g.snapshot())
        .map_err(|_| source_error(root, "source_observer_snapshot_unavailable"))?;
    require_verification_preconditions_with_caller(root, wi, runtime, &snapshot, caller)?;
    Ok(snapshot)
}

fn persist_source_data(
    root: &Path,
    wi: &str,
    kind: &str,
    value: &serde_json::Value,
) -> Result<String, ObserverError> {
    let bytes =
        serde_json::to_vec(&serde_json::json!({"schemaVersion":1,"kind":kind,"observation":value}))
            .map_err(|_| source_error(root, "source_observation_serialization"))?;
    let input_digest = Digest::sha256_bytes(&bytes);
    let id = cockpit_protocol::digest_json(&serde_json::json!([
        "portable-observation-data-v1",
        wi,
        kind,
        input_digest
    ]))
    .map_err(|_| source_error(root, "source_observation_identity"))?;
    let marker = serde_json::to_vec(
        &serde_json::json!({"schemaVersion":1,"recordId":id,"inputDigest":input_digest}),
    )
    .map_err(|_| source_error(root, "source_observation_serialization"))?;
    persist_portable_native_observation(root, wi, id.as_str(), &bytes, &marker)
}

pub fn read_portable_source_parameters(path: &Path) -> Result<serde_json::Value, ObserverError> {
    let raw = portable_input_bytes(path)?;
    if raw.len() > 16 * 1024 {
        return Err(source_error(path, "source_selector_byte_limit"));
    }
    reject_duplicate_json_keys(&raw)
        .map_err(|_| source_error(path, "source_selector_duplicate_or_invalid"))?;
    let value: serde_json::Value =
        serde_json::from_slice(&raw).map_err(|_| source_error(path, "source_selector_invalid"))?;
    if !value.is_object() {
        return Err(source_error(path, "source_selector_must_be_object"));
    }
    Ok(value)
}

pub fn declared_portable_verification_request(
    root: &Path,
    wi: &str,
    node: &str,
    runtime: &RuntimeContext,
    caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
) -> Result<RepositoryVerificationRequest, ObserverError> {
    require_portable_source_observation_admission(root, wi, runtime, caller)?;
    let canonical = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let root = canonical.as_path();
    let path = root.join(format!(".ai/work-items/active/{wi}.contract.json"));
    let raw = portable_input_bytes(&path)?;
    reject_duplicate_json_keys(&raw)
        .map_err(|_| source_error(root, "current_contract_invalid_json"))?;
    let contract: cockpit_protocol::Contract =
        serde_json::from_slice(&raw).map_err(|_| source_error(root, "current_contract_schema"))?;
    let mut selected = None;
    for (index, declaration) in contract.verification.iter().enumerate() {
        let (id, text) = match declaration {
            cockpit_protocol::VerificationDeclaration::Legacy(text) => {
                (format!("project-command-{index}"), text)
            }
            cockpit_protocol::VerificationDeclaration::Check(check) => {
                (check.check.clone(), &check.check)
            }
        };
        if id != node {
            continue;
        }
        if selected.is_some() {
            return Err(source_error(root, "declared_node_ambiguous"));
        }
        let (program, args) = cockpit_protocol::parse_verification_argv(text)
            .map_err(|_| source_error(root, "declared_node_argv_invalid"))?;
        let replace = |s: String| {
            if s == "<repo>" {
                root.to_string_lossy().into_owned()
            } else {
                s
            }
        };
        selected = Some(RepositoryVerificationRequest {
            node_id: id,
            program: replace(program),
            args: args.into_iter().map(replace).collect(),
            scope: vec!["**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: runtime.runtime_digest.to_string(),
            base_commit: None,
            workers: 1,
            work_item_id: Some(wi.into()),
            // New explicit source operations use the current approved finite
            // ceiling; no caller/packet can enlarge it or silently use default300.
            timeout_seconds: authorized_timeout_ceiling(root, Some(wi), "task")?
                .map(|v| v.min(cockpit_verification::MAX_ALLOWED_EXECUTION_SECONDS)),
            policy: RepositoryVerificationPolicy::ProfileAuthorized,
        });
    }
    let request = selected.ok_or_else(|| source_error(root, "declared_node_missing"))?;
    task_authorization::require_declared_task_command(root, &request, runtime, caller)?;
    Ok(request)
}

fn require_current_source_profile(
    root: &Path,
    request: &RepositoryVerificationRequest,
    runtime: &RuntimeContext,
    caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
) -> Result<(), ObserverError> {
    let wi = request
        .work_item_id
        .as_deref()
        .ok_or_else(|| source_error(root, "source_requires_work_item"))?;
    let snapshot = require_portable_source_observation_admission(root, wi, runtime, caller)?;
    let prepared = prepare_repository_verification_command(root, request, &snapshot)?;
    native_current_input(
        root,
        request,
        &snapshot,
        runtime,
        &prepared.command,
        prepared
            .execution_identity
            .as_ref()
            .ok_or_else(|| source_error(root, "source_executable_unavailable"))?,
        prepared
            .authorized_binding
            .as_ref()
            .ok_or_else(|| source_error(root, "current_profile_not_authorized"))?,
    )?;
    Ok(())
}

pub fn publish_declared_native_source(
    root: &Path,
    wi: &str,
    node: &str,
    nonce: &str,
    runtime: &RuntimeContext,
    caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
) -> Result<cockpit_protocol::ci_admission::PortableNativeSourceMetadata, ObserverError> {
    publish_declared_native_source_supervised(root, wi, node, nonce, runtime, caller, None)
}

pub fn publish_declared_native_source_supervised(
    root: &Path,
    wi: &str,
    node: &str,
    nonce: &str,
    runtime: &RuntimeContext,
    caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
    supervisor: Option<cockpit_verification::VerificationSupervisorLaunch>,
) -> Result<cockpit_protocol::ci_admission::PortableNativeSourceMetadata, ObserverError> {
    let canonical = fs::canonicalize(root).map_err(|source| ObserverError::Read {
        path: root.into(),
        source,
    })?;
    let root = canonical.as_path();
    if !(32..=128).contains(&nonce.len())
        || !nonce
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(source_error(root, "publisher_nonce_invalid"));
    }
    let request = declared_portable_verification_request(root, wi, node, runtime, caller)?;
    // Reject missing calibration before starting any project command.
    require_current_source_profile(root, &request, runtime, caller)?;
    let invocation = CurrentNativeInvocation::new();
    let executed = run_native_witness_inner(
        root,
        &request,
        runtime,
        caller,
        &invocation,
        supervisor,
        true,
    )?;
    let receipt = serde_json::to_value(&executed.run.receipt)
        .map_err(|_| source_error(root, "publisher_receipt_serialization"))?;
    lifecycle::persist_verification_attempt(
        root,
        wi,
        std::slice::from_ref(&request),
        &executed.run.final_snapshot,
        runtime,
        "execution_completed",
        Some((
            "partial_native_source_publication",
            "One declared node only; not full Contract verification.",
        )),
        Some(&receipt),
    )?;
    let witness = executed
        .witness
        .ok_or_else(|| source_error(root, "publisher_requires_fresh_physical_success"))?;
    let publication =
        publish_native_execution_witness(root, &request, runtime, caller, &invocation, &witness)?;
    let relative = format!(
        ".ai/evidence/{wi}.verification-admissions/portable/objects/{}.bin",
        &publication.input_digest.as_str()[7..]
    );
    // Store object filenames use the complete digest representation.
    let path = root.join(&relative);
    let bytes = portable_input_bytes(&path)?;
    if Digest::sha256_bytes(&bytes) != publication.input_digest {
        return Err(source_error(root, "publisher_object_readback_changed"));
    }
    validate_native_execution_witness(root, &request, runtime, caller, &invocation, &witness)?;
    Ok(
        cockpit_protocol::ci_admission::PortableNativeSourceMetadata {
            schema_version: 1,
            publisher_nonce: nonce.into(),
            runtime_digest: runtime.runtime_digest.clone(),
            input_digest: publication.input_digest.clone(),
            native_root: publication.input_digest,
            portable_path: path.to_string_lossy().into_owned(),
            execution_id: witness
                .execution_id()
                .parse()
                .map_err(|_| source_error(root, "publisher_execution_id_invalid"))?,
        },
    )
}

pub fn accept_portable_evidence_with_source(
    root: &Path,
    request: PortableSourceAdmissionRequest<'_>,
    tool: &PortableSourceTool,
    runtime: &RuntimeContext,
    caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
) -> Result<cockpit_protocol::ci_admission::PortableEvidenceAcceptance, ObserverError> {
    let raw = portable_input_bytes(request.input)?;
    reject_duplicate_json_keys(&raw)
        .map_err(|_| source_error(root, "portable_packet_invalid_json"))?;
    let packet: serde_json::Value = serde_json::from_slice(&raw)
        .map_err(|_| source_error(root, "portable_packet_invalid_json"))?;
    let node = request
        .node_id
        .or_else(|| packet["nativeInput"]["nodeId"].as_str())
        .ok_or_else(|| source_error(root, "portable_node_missing"))?;
    let current =
        declared_portable_verification_request(root, request.work_item_id, node, runtime, caller)?;
    require_current_source_profile(root, &current, runtime, caller)?;
    let source = tool.observe(
        root,
        request.work_item_id,
        PortableSourceRequest {
            input: request.input,
            expected_plan_digest: request.expected_plan_digest,
            parameters: request.parameters,
        },
        runtime,
        caller,
    )?;
    let accepted = source.accept(root, &current, runtime, caller)?;
    let mut report = accepted.report().clone();
    let (_, source_use) = accepted.consume_with_source_report(root, &current, runtime, caller)?;
    report.source_use = Some(source_use);
    report.writes_performed = 4;
    report.guidance=Some(cockpit_protocol::ci_admission::PortableEvidenceGuidance {
        summary:"The source was checked twice and one current declared check was satisfied. Full Work Item verification is unchanged.".into(),
        next_actions:vec!["Inspect the source checks and this node result; run all remaining declared checks before finishing.".into()],
    });
    Ok(report)
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
        source_use: None,
        guidance: Some(cockpit_protocol::ci_admission::PortableEvidenceGuidance {
            summary:"The file is available to inspect; its execution source has not been independently checked.".into(),
            next_actions:vec!["Check the source using a controlled reader, or execute the declared check fresh.".into()],
        }),
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

use cockpit_core::{DecisionState, Digest};
use cockpit_protocol::{
    HumanDecision, OutcomeClaim, OutcomeFinalizationProjection, OutcomeState, OutcomeV2,
    RuntimeContext, TaskOutcomeReport,
};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    ObservationPhase, ObserverError, RepositoryExecutionContext,
    close_decision_is_valid_for_status, read_contract, repository_id,
};

const MAX_OUTCOME_ASSEMBLY_ATTEMPTS: usize = 2;

/// The facts used to assemble a human Outcome are bound to one validated
/// observation boundary. This metadata is diagnostic evidence only; it never
/// grants lifecycle or authorization permission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutcomeAssemblyMetadata {
    pub snapshot_digest: Digest,
    pub facts_digest: Digest,
    pub attempts: usize,
}

/// The renderer and machine protocol intentionally share one finalization
/// projection model. It is an observation for recovery guidance and never
/// decides whether an operation is permitted.
pub type FinalizationProjection = OutcomeFinalizationProjection;

/// Fully assembled, already validated facts consumed by the human renderer.
///
/// Repository reads and governance validation happen in the assembly helpers;
/// `render_human_outcome` only formats this value and does not need a root
/// directory or filesystem access.
#[derive(Clone, Debug)]
pub struct OutcomeRenderInput {
    pub outcome: OutcomeV2,
    pub human_decision: HumanDecisionProjection,
    pub archived_unclosed: bool,
    pub lifecycle_status: String,
    pub finalization: FinalizationProjection,
    pub reason_keys: Vec<String>,
    pub assembly: Option<OutcomeAssemblyMetadata>,
}

pub fn outcome_render_input(
    root: &Path,
    work_item_id: &str,
) -> Result<OutcomeRenderInput, ObserverError> {
    assemble_outcome_render_input(root, work_item_id, None)
}

pub fn outcome_render_input_with_runtime(
    root: &Path,
    work_item_id: &str,
    runtime: &RuntimeContext,
) -> Result<OutcomeRenderInput, ObserverError> {
    assemble_outcome_render_input(root, work_item_id, Some(runtime))
}

/// Assemble render-only repository facts around an already obtained Outcome.
/// This is useful for callers that have a validated projection from another
/// observation boundary while keeping the renderer itself filesystem-free.
pub fn outcome_render_input_from_outcome(root: &Path, outcome: OutcomeV2) -> OutcomeRenderInput {
    build_outcome_render_input(root, outcome)
}

fn build_outcome_render_input(root: &Path, outcome: OutcomeV2) -> OutcomeRenderInput {
    let historical = outcome.historical_status.is_some();
    let superseded = outcome.historical_status.as_deref() == Some("superseded");
    let archived_contract = root
        .join(".ai/work-items/archive")
        .join(format!("{}.contract.json", outcome.work_item_id));
    let archived_unclosed = !historical
        && archived_contract.is_file()
        && !close_decision_is_valid_for_status(root, &outcome.work_item_id, &outcome.repository_id);
    let human_decision = load_human_decision(root, &outcome.work_item_id);
    let lifecycle_status = lifecycle_status(root, &outcome, historical, superseded);
    OutcomeRenderInput {
        finalization: finalization_projection_from_outcome(&outcome),
        reason_keys: governance_reason_keys_from_outcome(&outcome),
        outcome,
        human_decision,
        archived_unclosed,
        lifecycle_status,
        assembly: None,
    }
}

fn assemble_outcome_render_input(
    root: &Path,
    work_item_id: &str,
    runtime: Option<&RuntimeContext>,
) -> Result<OutcomeRenderInput, ObserverError> {
    assemble_outcome_render_input_with_hook(root, work_item_id, runtime, None)
}

fn assemble_outcome_render_input_with_hook(
    root: &Path,
    work_item_id: &str,
    runtime: Option<&RuntimeContext>,
    mut after_assembly: Option<&mut dyn FnMut(usize)>,
) -> Result<OutcomeRenderInput, ObserverError> {
    let mut last_change = None;
    for attempt in 1..=MAX_OUTCOME_ASSEMBLY_ATTEMPTS {
        let context = RepositoryExecutionContext::capture(root)?;
        let contract_path = outcome_contract_path(context.root(), work_item_id)?;
        let observation = context.observe_phase_with_contract_uncached(
            ObservationPhase::BeforeGovernance,
            runtime,
            &contract_path,
        )?;
        let before_facts = assembly_facts_digest(context.root(), work_item_id)?;
        let snapshot_digest = super::snapshot_digest(context.snapshot())?;
        let outcome = super::outcome_v2_internal_with_snapshot(
            context.root(),
            work_item_id,
            runtime,
            Some((context.snapshot(), &snapshot_digest)),
        )?;
        let historical = outcome.historical_status.is_some();
        let superseded = outcome.historical_status.as_deref() == Some("superseded");
        let archived_contract = context
            .root()
            .join(".ai/work-items/archive")
            .join(format!("{work_item_id}.contract.json"));
        let archived_unclosed = !historical
            && archived_contract.is_file()
            && !close_decision_is_valid_for_status(
                context.root(),
                work_item_id,
                &outcome.repository_id,
            );
        let human_decision = load_human_decision(context.root(), work_item_id);
        let lifecycle_status = lifecycle_status(context.root(), &outcome, historical, superseded);
        let finalization = finalization_projection(context.root(), work_item_id, &outcome, runtime);
        let reason_keys = governance_reason_keys(context.root(), work_item_id, &outcome, runtime);
        let mut outcome = outcome;
        outcome.governance_reasons = reason_keys.clone();
        outcome.finalization = Some(finalization.clone());
        let mut input = OutcomeRenderInput {
            finalization,
            reason_keys,
            outcome,
            human_decision,
            archived_unclosed,
            lifecycle_status,
            assembly: None,
        };
        if let Some(hook) = after_assembly.as_mut() {
            hook(attempt);
        }
        let after_facts = assembly_facts_digest(context.root(), work_item_id)?;
        if before_facts != after_facts {
            last_change = Some((before_facts, after_facts));
            continue;
        }
        observation.validate_current()?;
        input.assembly = Some(OutcomeAssemblyMetadata {
            snapshot_digest,
            facts_digest: after_facts,
            attempts: attempt,
        });
        return Ok(input);
    }
    let (before, after) = last_change.expect("bounded Outcome assembly records a change");
    Err(ObserverError::State {
        path: PathBuf::from(".ai/work-items"),
        message: format!(
            "Outcome assembly observation changed during bounded assembly; facts before={before}, after={after}; result is unknown"
        ),
    })
}

fn outcome_contract_path(root: &Path, work_item_id: &str) -> Result<PathBuf, ObserverError> {
    [
        root.join(".ai/work-items/active")
            .join(format!("{work_item_id}.contract.json")),
        root.join(".ai/work-items/archive")
            .join(format!("{work_item_id}.contract.json")),
    ]
    .into_iter()
    .find(|path| path.is_file())
    .ok_or_else(|| ObserverError::State {
        path: root
            .join(".ai/work-items/active")
            .join(format!("{work_item_id}.contract.json")),
        message: "work item Contract not found for Outcome observation".into(),
    })
}

fn assembly_facts_digest(root: &Path, work_item_id: &str) -> Result<Digest, ObserverError> {
    let mut paths = vec![
        format!(".ai/evidence/{work_item_id}.verification.json"),
        format!(".ai/decisions/{work_item_id}.close.json"),
        format!(".ai/decisions/{work_item_id}.recovery.json"),
        format!(".ai/decisions/{work_item_id}.preflight-review.json"),
    ];
    for phase in ["active", "archive"] {
        for suffix in [
            "contract.json",
            "summary.json",
            "outcome.json",
            "task-report.json",
        ] {
            paths.push(format!(".ai/work-items/{phase}/{work_item_id}.{suffix}"));
        }
        paths.push(format!(
            ".ai/work-items/{phase}/{work_item_id}.archive.json"
        ));
    }
    let decisions = root.join(".ai/decisions");
    if let Ok(entries) = fs::read_dir(&decisions) {
        let prefixes = [
            format!("{work_item_id}.finalize"),
            format!("{work_item_id}.recovery"),
            format!("{work_item_id}.preflight-review"),
        ];
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.ends_with(".json") && prefixes.iter().any(|prefix| name.starts_with(prefix)) {
                paths.push(format!(".ai/decisions/{name}"));
            }
        }
    }
    paths.sort();
    paths.dedup();
    let mut bytes = Vec::new();
    for relative in paths {
        bytes.extend_from_slice(relative.as_bytes());
        bytes.push(0);
        let path = root.join(&relative);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                bytes.extend_from_slice(b"symlink")
            }
            Ok(metadata) if metadata.is_file() => {
                bytes.extend_from_slice(&fs::read(&path).map_err(|source| {
                    ObserverError::Read {
                        path: path.clone(),
                        source,
                    }
                })?);
            }
            Ok(_) => bytes.extend_from_slice(b"invalid"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                bytes.extend_from_slice(b"missing")
            }
            Err(source) => return Err(ObserverError::Read { path, source }),
        }
        bytes.push(0xff);
    }
    bytes.extend_from_slice(b"resource-observation\0");
    bytes.extend_from_slice(&resource_observation_facts(root));
    bytes.push(0xff);
    Ok(Digest::sha256_bytes(&bytes))
}

/// Finalization verification reads local branch/worktree state in addition to
/// repository records. Include those observations in the assembly boundary so
/// a cleanup race cannot be rendered from a stitched record-only view.
fn resource_observation_facts(root: &Path) -> Vec<u8> {
    let branch = super::git_text(root, &["branch", "--format=%(refname:short)"])
        .unwrap_or_else(|| "<unavailable>".into());
    let worktrees = super::git_text(root, &["worktree", "list", "--porcelain"])
        .unwrap_or_else(|| "<unavailable>".into());
    format!("branch\0{branch}\0worktrees\0{worktrees}").into_bytes()
}

fn finalization_projection(
    root: &Path,
    work_item_id: &str,
    _outcome: &OutcomeV2,
    runtime: Option<&RuntimeContext>,
) -> FinalizationProjection {
    let contract_path = match outcome_contract_path(root, work_item_id) {
        Ok(path) => path,
        Err(_) => return finalization_projection_unknown("contract_missing"),
    };
    let contract = match read_contract(&contract_path) {
        Ok(contract) => contract,
        Err(_) => return finalization_projection_unknown("contract_invalid"),
    };
    if contract.resource_context.is_none() {
        return FinalizationProjection {
            state: "not_required".into(),
            error_code: None,
            disposition: None,
            action: "human_decision_or_lifecycle".into(),
            reliable: true,
        };
    }
    let receipt_path = root
        .join(".ai/decisions")
        .join(format!("{work_item_id}.finalize.json"));
    match fs::symlink_metadata(&receipt_path) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return FinalizationProjection {
                state: "receipt_missing".into(),
                error_code: Some("receipt_missing".into()),
                disposition: None,
                action: "inspect_resources_and_record_receipt".into(),
                reliable: false,
            };
        }
        Err(_) => return finalization_projection_unknown("receipt_unreadable"),
    }
    match super::verify_resource_finalization_internal(root, work_item_id, runtime) {
        Ok(value) => {
            let disposition = value["disposition"].as_str().map(str::to_owned);
            let action = match disposition.as_deref() {
                Some("retained") => "retain_resources_and_follow_close_rules",
                Some("deleted" | "abandoned") => "record_close_decision_if_required",
                _ => "inspect_finalization_result",
            };
            FinalizationProjection {
                state: "verified".into(),
                error_code: None,
                disposition,
                action: action.into(),
                reliable: true,
            }
        }
        Err(error) => {
            let code = classify_finalization_error(&error.to_string());
            let action = match code.as_str() {
                "identity_mismatch" | "record_corrupt" => "inspect_binding_and_recovery_evidence",
                "cleanup_pending" => "reobserve_resources_before_cleanup",
                "receipt_missing" => "inspect_resources_and_record_receipt",
                _ => "inspect_recovery_conditions_before_action",
            };
            FinalizationProjection {
                state: code.clone(),
                error_code: Some(code),
                disposition: None,
                action: action.into(),
                reliable: false,
            }
        }
    }
}

fn finalization_projection_from_outcome(outcome: &OutcomeV2) -> FinalizationProjection {
    if let Some(projection) = outcome.finalization.clone() {
        return projection;
    }
    if outcome
        .unknowns
        .iter()
        .any(|unknown| unknown == "resource_finalization_pending")
    {
        FinalizationProjection {
            state: "unknown".into(),
            error_code: Some("resource_finalization_pending".into()),
            disposition: None,
            action: "inspect_recovery_conditions_before_action".into(),
            reliable: false,
        }
    } else {
        FinalizationProjection {
            state: "not_observed".into(),
            error_code: None,
            disposition: None,
            action: "use_current_runtime_observation".into(),
            reliable: false,
        }
    }
}

fn finalization_projection_unknown(code: &str) -> FinalizationProjection {
    FinalizationProjection {
        state: "unknown".into(),
        error_code: Some(code.into()),
        disposition: None,
        action: "inspect_recovery_conditions_before_action".into(),
        reliable: false,
    }
}

fn classify_finalization_error(message: &str) -> String {
    let lower = message.to_ascii_lowercase();
    if lower.contains("identity mismatch") || lower.contains("binding") {
        "identity_mismatch".into()
    } else if lower.contains("not a regular")
        || lower.contains("invalid resource finalization")
        || lower.contains("invalid json")
        || lower.contains("schema")
    {
        "record_corrupt".into()
    } else if lower.contains("cleanup postconditions") || lower.contains("cleanup") {
        "cleanup_pending".into()
    } else if lower.contains("not found") || lower.contains("missing") {
        "receipt_missing".into()
    } else {
        "unknown".into()
    }
}

fn governance_reason_keys_from_outcome(outcome: &OutcomeV2) -> Vec<String> {
    let mut keys = Vec::new();
    if let Some(gate) = outcome
        .failed_gate
        .as_deref()
        .or_else(|| outcome.task_outcome_report.as_ref()?.failed_gate.as_deref())
    {
        push_reason_for_gate(&mut keys, gate);
    }
    for unknown in &outcome.unknowns {
        push_reason_for_unknown(&mut keys, unknown);
    }
    if outcome.decision_state == Some(DecisionState::Red) && keys.is_empty() {
        keys.push("governance_reason_unknown".into());
    }
    keys
}

fn governance_reason_keys(
    root: &Path,
    work_item_id: &str,
    outcome: &OutcomeV2,
    runtime: Option<&RuntimeContext>,
) -> Vec<String> {
    let mut keys = governance_reason_keys_from_outcome(outcome);
    let controls = runtime
        .map(|runtime| {
            crate::validate_work_item_governance_controls_with_runtime(root, work_item_id, runtime)
        })
        .unwrap_or_else(|| crate::validate_work_item_governance_controls(root, work_item_id));
    match controls {
        Ok(report) => {
            if !matches!(
                report.scenario_coverage.as_str(),
                "verified" | "not_applicable"
            ) {
                keys.push("scenario_coverage_insufficient".into());
            }
            if !matches!(
                report.acceptance_evidence.as_str(),
                "verified" | "not_applicable"
            ) {
                keys.push("acceptance_evidence_insufficient".into());
            }
            if !matches!(
                report.intent_alignment.as_str(),
                "resolved" | "not_applicable"
            ) {
                keys.push("intent_alignment_insufficient".into());
            }
            if !matches!(
                report.final_dimensions.as_str(),
                "verified" | "not_applicable"
            ) {
                keys.push("final_dimensions_insufficient".into());
            }
            for finding in report.findings {
                push_reason_for_control_code(&mut keys, &finding.code);
            }
            for unknown in report.unknowns {
                push_reason_for_unknown(&mut keys, &unknown);
            }
        }
        Err(_) => keys.push("governance_reason_unknown".into()),
    }
    keys.sort();
    keys.dedup();
    keys
}

fn push_reason_for_gate(keys: &mut Vec<String>, gate: &str) {
    let key = if gate.starts_with("finish.verification") {
        "verification_failed"
    } else if gate.starts_with("finish.preflight") {
        "preflight_failed"
    } else if gate.starts_with("finish.governance") {
        "governance_failed"
    } else if gate.starts_with("finish.lifecycle") {
        "lifecycle_failed"
    } else {
        "governance_gate_failed"
    };
    keys.push(key.into());
}

fn push_reason_for_unknown(keys: &mut Vec<String>, unknown: &str) {
    let key = if unknown == "verification_evidence_missing" {
        "verification_evidence_missing"
    } else if unknown == "evidence_stale"
        || unknown.contains("evidence") && unknown.contains("stale")
    {
        "evidence_expired"
    } else if unknown == "evidence_contradictory" {
        "evidence_invalid"
    } else if unknown == "evidence_unknown" {
        "verification_evidence_unknown"
    } else if unknown == "identity_mismatch" {
        "evidence_identity_mismatch"
    } else if unknown == "outcome_report_invalid" {
        "outcome_report_invalid"
    } else if unknown.starts_with("required_scenario_") || unknown.starts_with("scenario_") {
        "scenario_coverage_insufficient"
    } else if unknown.starts_with("acceptance_") {
        "acceptance_evidence_insufficient"
    } else if unknown.starts_with("intent_alignment") {
        "intent_alignment_insufficient"
    } else if unknown.starts_with("final_dimension") {
        "final_dimensions_insufficient"
    } else if unknown.contains("authority") {
        "authorization_insufficient"
    } else if unknown.contains("scope") || unknown.contains("out_of_range") {
        "scope_out_of_range"
    } else if unknown == "resource_finalization_pending"
        || unknown == "user_visible_benefit_not_declared"
    {
        return;
    } else {
        return;
    };
    keys.push(key.into());
}

fn push_reason_for_control_code(keys: &mut Vec<String>, code: &str) {
    if code.contains("acceptance") {
        keys.push("acceptance_evidence_insufficient".into());
    } else if code.contains("intent_alignment") {
        keys.push("intent_alignment_insufficient".into());
    } else if code.contains("scenario") {
        keys.push("scenario_coverage_insufficient".into());
    } else if code.contains("authority") {
        keys.push("authorization_insufficient".into());
    } else if code.contains("scope") || code.contains("range") {
        keys.push("scope_out_of_range".into());
    } else if code.contains("final_dimension") {
        keys.push("final_dimensions_insufficient".into());
    }
}

/// Select the human-facing Outcome projection without changing the underlying
/// OutcomeV2 or TaskOutcomeReport data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutcomeRenderView {
    Summary,
    Full,
}

/// Render the repository Outcome as an explicit, human-facing handoff.
///
/// This is intentionally shared by the CLI and MCP adapters. The OutcomeV2
/// value is produced and validated by outcome_v2; this function only projects
/// it for a conversation. Contract text remains in its original language and
/// no governance decision is inferred or translated.
pub fn render_human_outcome(input: &OutcomeRenderInput, language: &str) -> String {
    render_full_outcome(input, language)
}

/// Render either the reader-first summary or the complete audit handoff.
pub fn render_human_outcome_with_view(
    input: &OutcomeRenderInput,
    language: &str,
    view: OutcomeRenderView,
) -> String {
    match view {
        OutcomeRenderView::Summary => render_summary_outcome(input, language),
        OutcomeRenderView::Full => render_full_outcome(input, language),
    }
}

/// Render the complete evidence-oriented handoff explicitly.
pub fn render_full_human_outcome(input: &OutcomeRenderInput, language: &str) -> String {
    render_human_outcome_with_view(input, language, OutcomeRenderView::Full)
}

fn render_summary_outcome(input: &OutcomeRenderInput, language: &str) -> String {
    let outcome = &input.outcome;
    let language = normalized_language(language);
    let historical_kind = outcome.historical_status.as_deref();
    let historical = historical_kind.is_some();
    let superseded = historical_kind == Some("superseded");
    let (marker, status) = if superseded {
        match language {
            "zh" => ("🟡", "历史已替代"),
            "ja" => ("🟡", "履歴として置換済み"),
            _ => ("🟡", "Superseded historical item"),
        }
    } else {
        outcome_status(&outcome.state, outcome.decision_state.as_ref(), language)
    };
    let report = &outcome.human_benefit_report;
    let task_report = outcome.task_outcome_report.as_ref();
    let not_recorded = match language {
        "zh" => "未记录",
        "ja" => "未記録",
        _ => "Not recorded",
    };
    let (result_title, key_changes, uncertainty, next_action, status_labels) = match language {
        "zh" => (
            "结果",
            "关键变化",
            "剩余不确定性",
            "人的下一步",
            ("验证状态", "生命周期状态", "人工决定状态", "治理信号"),
        ),
        "ja" => (
            "結果",
            "主な変更",
            "残る不確実性",
            "人間の次のアクション",
            (
                "検証状態",
                "ライフサイクル状態",
                "人間の判断状態",
                "ガバナンスシグナル",
            ),
        ),
        _ => (
            "Result",
            "Key changes",
            "Remaining uncertainty",
            "Human next step",
            (
                "Verification",
                "Lifecycle",
                "Human decision",
                "Governance signal",
            ),
        ),
    };
    let raw_lifecycle = input.lifecycle_status.clone();
    let lifecycle = localized_lifecycle_status(raw_lifecycle.clone(), language);
    let decision_projection = input.human_decision.clone();
    let human_decision_status = localized_human_decision_status(&decision_projection, language);
    let governance_signal = localized_governance_signal(outcome.decision_state.as_ref(), language);
    let localized_summary = if historical {
        if superseded {
            match language {
                "zh" => {
                    "该 Work Item 已作为历史 predecessor 被显式替代；原始证据未被重写，也未按当前 Runtime 重验证。"
                }
                "ja" => {
                    "この Work Item は履歴 predecessor として明示的に置換されました。元の evidence は書き換えず、現在の Runtime では再検証していません。"
                }
                _ => {
                    "This Work Item was explicitly superseded as a historical predecessor; original evidence was not rewritten or revalidated under the current Runtime."
                }
            }
        } else {
            match language {
                "zh" => "历史验证证据未按当前 Runtime 重新验证；这不是当前失败。",
                "ja" => {
                    "履歴の検証 evidence は現在の Runtime で再検証されていません。現在の失敗ではありません。"
                }
                _ => {
                    "Historical verification evidence was not revalidated under the current Runtime; this is not a current failure."
                }
            }
        }
    } else {
        localized_outcome_summary(&outcome.state, outcome.decision_state.as_ref(), language)
    };

    let mut result_items = vec![
        format!("{}: {status}", status_labels.0),
        format!("{}: {lifecycle}", status_labels.1),
        format!("{}: {human_decision_status}", status_labels.2),
        format!("{}: {governance_signal}", status_labels.3),
        localized_summary.to_string(),
    ];
    result_items.push(localized_evidence_refs(
        &outcome.evidence_refs,
        language,
        not_recorded,
    ));

    let key_change_items = task_report
        .map(|report| {
            summary_claims_with_evidence(&report.sections.delivered_changes, language, "change")
        })
        .unwrap_or_default();
    let key_change_items = if key_change_items.is_empty() {
        vec![not_recorded.to_string()]
    } else {
        key_change_items
    };

    let not_ready = localized_not_ready_status(outcome, &input.reason_keys, language);
    let failed_gate = if historical {
        None
    } else {
        outcome
            .failed_gate
            .as_deref()
            .or_else(|| task_report.and_then(|report| report.failed_gate.as_deref()))
    };
    let failed_gate_item = failed_gate.map(|gate| {
        let label = match language {
            "zh" => "失败 gate",
            "ja" => "失敗した gate",
            _ => "Failed gate",
        };
        format!("{label}: {gate}")
    });
    let recovery_action = failed_gate.map(|gate| {
        let label = match language {
            "zh" => "恢复条件",
            "ja" => "復旧条件",
            _ => "Recovery condition",
        };
        format!("{label}: {}", localized_recovery_action(gate, language))
    });
    let mut uncertainty_items = Vec::new();
    let mut stop_items = task_report
        .map(|report| claim_texts(&report.sections.forced_stops))
        .unwrap_or_default();
    if historical {
        stop_items.clear();
    } else if stop_items.is_empty()
        && matches!(
            outcome.state,
            OutcomeState::NotReady | OutcomeState::Unknown
        )
    {
        stop_items.push(not_ready.clone());
    }
    uncertainty_items.extend(stop_items);
    if !historical {
        for reason in &input.reason_keys {
            push_unique(
                &mut uncertainty_items,
                localized_governance_reason(reason, language),
            );
        }
    }
    if let Some(item) = failed_gate_item {
        push_unique(&mut uncertainty_items, item);
    }
    if let Some(action) = recovery_action {
        push_unique(&mut uncertainty_items, action);
    }
    if let Some(report) = task_report {
        for claim in report
            .sections
            .risks
            .iter()
            .chain(report.sections.warnings.iter())
            .chain(report.sections.residual_risks.iter())
        {
            for item in summary_claims_with_evidence(std::slice::from_ref(claim), language, "risk")
            {
                push_unique(&mut uncertainty_items, item);
            }
        }
        for claim in &report.sections.limitations {
            for item in
                summary_claims_with_evidence(std::slice::from_ref(claim), language, "limitation")
            {
                push_unique(&mut uncertainty_items, item);
            }
        }
    }
    let mut unknowns_all = outcome.unknowns.clone();
    unknowns_all.extend(report.unknowns.iter().cloned());
    unknowns_all.sort();
    unknowns_all.dedup();
    uncertainty_items.extend(unknowns_all);
    if report.user_visible_changes.is_empty() && report.affected_users.is_empty() {
        let no_benefit = match language {
            "zh" => "用户可见收益尚未声明。",
            "ja" => "ユーザー向けの効果はまだ宣言されていません。",
            _ => "User-visible benefit has not been declared.",
        };
        push_unique(&mut uncertainty_items, no_benefit.to_string());
    }
    if uncertainty_items.is_empty() {
        uncertainty_items.push(localized_risk_absence(task_report, language));
    } else if !uncertainty_items.iter().any(|item| {
        item.starts_with("Risk findings:")
            || item.starts_with("风险评估：")
            || item.starts_with("リスク評価：")
    }) {
        push_unique(
            &mut uncertainty_items,
            localized_risk_absence(task_report, language),
        );
    }

    if input.archived_unclosed || input.finalization.state != "not_observed" {
        push_unique(
            &mut uncertainty_items,
            localized_finalization_status(&input.finalization, language),
        );
    }
    let next = localized_summary_next_action(
        language,
        outcome,
        &raw_lifecycle,
        historical,
        superseded,
        input.archived_unclosed,
        &input.finalization,
        &input.reason_keys,
        &decision_projection,
    );
    let decision_detail = match &decision_projection {
        HumanDecisionProjection::Missing => human_decision_status.clone(),
        HumanDecisionProjection::Valid {
            decision,
            assurance,
        } => render_human_decision(decision, assurance.as_deref(), language, not_recorded),
        HumanDecisionProjection::Invalid(reason) => {
            let label = match language {
                "zh" => "未知：结构化人工决定记录无效",
                "ja" => "不明：構造化された人間の判断記録が無効です",
                _ => "Unknown: structured human decision record is invalid",
            };
            format!("{label} ({reason})")
        }
    };
    let full_report_hint = match language {
        "zh" => {
            "完整证据报告：CLI 使用 ai-cockpit work-item outcome --repo <repository> --id <work-item> --view full；MCP 使用 work_item_outcome 的 view: full。"
        }
        "ja" => {
            "完全な evidence report：CLI は ai-cockpit work-item outcome --repo <repository> --id <work-item> --view full、MCP は work_item_outcome の view: full を使用します。"
        }
        _ => {
            "Full evidence report: use ai-cockpit work-item outcome --repo <repository> --id <work-item> --view full in CLI, or view: full with MCP work_item_outcome."
        }
    };
    let header = format!(
        "Outcome: {marker} {status} — {}\n{result_title}",
        outcome.work_item_id
    );
    format!(
        "{header}\n- {}\n- {}\n- {}\n- {}\n- {}\n- {}\n\n{key_changes}\n{}\n\n{uncertainty}\n{}\n\n{next_action}\n- {next}\n- {decision_detail}\n- {full_report_hint}",
        result_items[0],
        result_items[1],
        result_items[2],
        result_items[3],
        result_items[4],
        result_items[5],
        bullet_lines(&key_change_items, not_recorded),
        bullet_lines(&uncertainty_items, not_recorded),
    )
}

fn normalized_language(language: &str) -> &str {
    match language {
        "zh" | "ja" => language,
        _ => "en",
    }
}

fn push_unique(items: &mut Vec<String>, item: String) {
    if !item.trim().is_empty() && !items.iter().any(|existing| existing == &item) {
        items.push(item);
    }
}

fn summary_claims_with_evidence(
    claims: &[OutcomeClaim],
    language: &str,
    kind: &str,
) -> Vec<String> {
    claims
        .iter()
        .filter(|claim| !claim.text.trim().is_empty())
        .map(|claim| {
            let text = if kind == "risk" {
                calibrated_risk_claims(std::slice::from_ref(claim), language)
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| claim.text.clone())
            } else {
                format_claim(claim, language, kind)
            };
            if claim.evidence_refs.is_empty() {
                text
            } else {
                format!(
                    "{text} [{}]",
                    localized_evidence_refs(&claim.evidence_refs, language, "Not recorded")
                )
            }
        })
        .collect()
}

fn localized_evidence_refs(refs: &[String], language: &str, not_recorded: &str) -> String {
    let label = match language {
        "zh" => "证据引用",
        "ja" => "evidence 参照",
        _ => "Evidence refs",
    };
    if refs.is_empty() {
        format!("{label}: {not_recorded}")
    } else {
        format!("{label}: {}", refs.join(", "))
    }
}

fn localized_not_ready_status(
    outcome: &OutcomeV2,
    reason_keys: &[String],
    language: &str,
) -> String {
    if reason_keys
        .iter()
        .any(|key| key == "acceptance_evidence_insufficient")
    {
        return localized_governance_reason("acceptance_evidence_insufficient", language);
    }
    if reason_keys
        .iter()
        .any(|key| key == "intent_alignment_insufficient")
    {
        return localized_governance_reason("intent_alignment_insufficient", language);
    }
    if reason_keys
        .iter()
        .any(|key| key == "scenario_coverage_insufficient")
    {
        return localized_governance_reason("scenario_coverage_insufficient", language);
    }
    if reason_keys.iter().any(|key| key == "scope_out_of_range") {
        return localized_governance_reason("scope_out_of_range", language);
    }
    if reason_keys
        .iter()
        .any(|key| key == "authorization_insufficient")
    {
        return localized_governance_reason("authorization_insufficient", language);
    }
    let has = |code: &str| outcome.unknowns.iter().any(|unknown| unknown == code);
    if has("evidence_stale") {
        return match language {
            "zh" => "当前 repository 快照中的验证证据已过期，不能宣称当前结果。".into(),
            "ja" => "現在の repository snapshot に対する検証 evidence が期限切れで、current result は主張できません。".into(),
            _ => "Verification evidence is stale for the current repository snapshot; a current result cannot be claimed.".into(),
        };
    }
    if has("resource_finalization_pending") {
        return match language {
            "zh" => "验证可能有效，但 provider finalization 证据缺失或无效；不能作为终态。".into(),
            "ja" => "検証は有効な可能性がありますが、provider finalization evidence が欠落または無効で、終端状態にはできません。".into(),
            _ => "Verification may be valid, but provider finalization evidence is missing or invalid; the outcome is not terminal.".into(),
        };
    }
    if has("close_decision_pending") || has("close_decision_invalid") {
        return match language {
            "zh" => "验证可能有效，但必需的人工 close 决定缺失或无效；不能作为已关闭。".into(),
            "ja" => "検証は有効な可能性がありますが、必要な人間の close 判断が欠落または無効で、closed とは言えません。".into(),
            _ => "Verification may be valid, but the required human close decision is missing or invalid; the item is not closed.".into(),
        };
    }
    if has("evidence_contradictory")
        || has("evidence_unknown")
        || has("outcome_report_invalid")
        || has("identity_mismatch")
    {
        return match language {
            "zh" => "验证证据无法确认或与当前上下文不一致；结果已停止。".into(),
            "ja" => "検証 evidence を確認できないか現在の context と一致せず、結果は停止しています。".into(),
            _ => "Verification evidence could not be confirmed or does not match this context; the outcome is stopped.".into(),
        };
    }
    match language {
        "zh" => "必需的验证证据尚未生成，不能宣称完成。".into(),
        "ja" => "必須の検証 evidence がまだなく、完了とは言えません。".into(),
        _ => "Required verification evidence is not present; completion cannot be claimed.".into(),
    }
}

#[allow(clippy::too_many_arguments)]
fn localized_summary_next_action(
    language: &str,
    outcome: &OutcomeV2,
    lifecycle: &str,
    historical: bool,
    superseded: bool,
    archived_unclosed: bool,
    finalization: &FinalizationProjection,
    reason_keys: &[String],
    decision: &HumanDecisionProjection,
) -> String {
    if historical {
        return if superseded {
            match language {
                "zh" => "保留原始历史证据；后续工作由 successor Work Item 负责，不要重新解释为当前失败。".into(),
                "ja" => "元の履歴 evidence を保持し、後続作業は successor Work Item で行います。現在の失敗とは解釈しません。".into(),
                _ => "Preserve the historical evidence; the successor owns follow-up work, and this is not a current failure.".into(),
            }
        } else {
            match language {
                "zh" => "保留历史证据；如需当前结果，再用当前 Runtime 重新验证，不要将其解释为当前失败。".into(),
                "ja" => "履歴 evidence を保持し、current result が必要な場合だけ現在の Runtime で再検証してください。現在の失敗とは解釈しません。".into(),
                _ => "Preserve the historical evidence; reverify with the current Runtime only when a current result is needed, and do not treat it as a current failure.".into(),
            }
        };
    }
    if archived_unclosed {
        if matches!(finalization.state.as_str(), "not_observed" | "not_required") {
            return match language {
                "zh" => "审阅归档证据后记录明确的人工 close 决定；完成 close 前不得开始下一个 Work Item。".into(),
                "ja" => "アーカイブ evidence を確認して明示的な人間の close 判断を記録してください。close 完了前に次の Work Item を開始しないでください。".into(),
                _ => "Review the archive evidence and record the explicit human close decision; do not start another Work Item until close is complete.".into(),
            };
        }
        return localized_finalization_action(finalization, language);
    }
    if lifecycle == "closed" {
        return match language {
            "zh" => "该 Work Item 已关闭；本次交接不需要新的人工决定。不要把验证状态扩展解释为其他授权。".into(),
            "ja" => "この Work Item はクローズ済みです。この handoff で新たな人間の判断は不要です。検証状態を別の権限として解釈しないでください。".into(),
            _ => "This Work Item is closed; no new human decision is required by this handoff. Do not broaden verification into another authorization.".into(),
        };
    }
    if let HumanDecisionProjection::Valid { .. } = decision {
        return match language {
            "zh" => "人工决定已记录；仅按该决定的明确范围继续，不能从验证状态推导更高授权。".into(),
            "ja" => "人間の判断は記録済みです。その明示された範囲だけに従い、検証状態から上位の権限を推論しないでください。".into(),
            _ => "A human decision is recorded; follow only its explicit scope and do not infer broader authorization from verification.".into(),
        };
    }
    if let Some(key) = reason_keys.iter().find(|key| {
        matches!(
            key.as_str(),
            "acceptance_evidence_insufficient"
                | "intent_alignment_insufficient"
                | "scenario_coverage_insufficient"
                | "final_dimensions_insufficient"
                | "scope_out_of_range"
                | "authorization_insufficient"
        )
    }) {
        return localized_governance_next_action(key, language);
    }
    if outcome
        .unknowns
        .iter()
        .any(|unknown| unknown == "evidence_stale")
    {
        return match language {
            "zh" => "获取当前 repository 快照对应的有效验证证据并重新验证；在此之前保持停止。".into(),
            "ja" => "現在の repository snapshot に対応する有効な検証 evidence を取得して再検証してください。それまでは停止します。".into(),
            _ => "Obtain valid verification evidence for the current repository snapshot and verify again; remain stopped until then.".into(),
        };
    }
    match (language, &outcome.state, outcome.decision_state.as_ref()) {
        ("zh", OutcomeState::Verified, _) => "审阅证据后再决定是否继续；🟢 不代表已授权合并或发布。".into(),
        ("zh", _, Some(DecisionState::Red)) => "修复无效证据并重新验证；在此之前保持停止。".into(),
        ("zh", _, _) => "补齐缺失证据并重新验证；在此之前保持停止。".into(),
        ("ja", OutcomeState::Verified, _) => "証拠を確認してから続行を判断してください。🟢 はマージやリリースの承認ではありません。".into(),
        ("ja", _, Some(DecisionState::Red)) => "無効な evidence を修復して再検証してください。それまでは停止します。".into(),
        ("ja", _, _) => "不足している evidence を補い、再検証してください。それまでは停止状態を維持します。".into(),
        (_, OutcomeState::Verified, _) => "Review the evidence before deciding whether to proceed; 🟢 does not authorize merge or release.".into(),
        (_, _, Some(DecisionState::Red)) => "Repair the invalid evidence and verify again; remain stopped until then.".into(),
        (_, _, _) => "Repair the missing evidence and verify again; remain stopped until then.".into(),
    }
}

fn localized_governance_reason(key: &str, language: &str) -> String {
    match (language, key) {
        ("zh", "verification_failed") => "验证执行失败；当前结果已停止。".into(),
        ("ja", "verification_failed") => "検証の実行に失敗したため、現在の結果は停止しています。".into(),
        (_, "verification_failed") => "Verification execution failed; the current result is stopped.".into(),
        ("zh", "evidence_invalid") => "验证证据无效；当前结果已停止。".into(),
        ("ja", "evidence_invalid") => "検証 evidence が無効なため、現在の結果は停止しています。".into(),
        (_, "evidence_invalid") => "Verification evidence is invalid; the current result is stopped.".into(),
        ("zh", "evidence_identity_mismatch") => "验证证据身份与当前 Work Item 或 repository 不匹配；当前结果已停止。".into(),
        ("ja", "evidence_identity_mismatch") => "検証 evidence の identity が現在の Work Item または repository と一致せず、結果は停止しています。".into(),
        (_, "evidence_identity_mismatch") => "Verification evidence identity does not match the current Work Item or repository; the result is stopped.".into(),
        ("zh", "evidence_expired") => "验证证据相对于当前 repository 快照已过期；不能宣称当前结果。".into(),
        ("ja", "evidence_expired") => "検証 evidence は現在の repository snapshot に対して期限切れで、current result は主張できません。".into(),
        (_, "evidence_expired") => "Verification evidence is expired for the current repository snapshot; a current result cannot be claimed.".into(),
        ("zh", "verification_evidence_missing") => "验证证据缺失；当前结果尚未准备好。".into(),
        ("ja", "verification_evidence_missing") => "検証 evidence が欠落しており、現在の結果は準備できていません。".into(),
        (_, "verification_evidence_missing") => "Verification evidence is missing; the current result is not ready.".into(),
        ("zh", "verification_evidence_unknown") => "验证证据状态未知；在检查证据前不能继续。".into(),
        ("ja", "verification_evidence_unknown") => "検証 evidence の状態が不明で、evidence を確認するまで続行できません。".into(),
        (_, "verification_evidence_unknown") => "Verification evidence state is unknown; inspect the evidence before proceeding.".into(),
        ("zh", "acceptance_evidence_insufficient") => "验证可以通过，但验收证据不足；这不是验证证据无效。".into(),
        ("ja", "acceptance_evidence_insufficient") => "検証は通過できますが、受入れ evidence が不足しています。検証 evidence が無効という意味ではありません。".into(),
        (_, "acceptance_evidence_insufficient") => "Verification may have passed, but acceptance evidence is insufficient; this does not mean verification evidence is invalid.".into(),
        ("zh", "intent_alignment_insufficient") => "验证可以通过，但意图对齐证据不足；这不是验证证据无效。".into(),
        ("ja", "intent_alignment_insufficient") => "検証は通過できますが、intent alignment evidence が不足しています。検証 evidence が無効という意味ではありません。".into(),
        (_, "intent_alignment_insufficient") => "Verification may have passed, but intent-alignment evidence is insufficient; this does not mean verification evidence is invalid.".into(),
        ("zh", "scenario_coverage_insufficient") => "场景覆盖证据不足；当前治理条件未完成。".into(),
        ("ja", "scenario_coverage_insufficient") => "シナリオ coverage evidence が不足しており、現在の governance 条件は未完了です。".into(),
        (_, "scenario_coverage_insufficient") => "Scenario-coverage evidence is insufficient; the current governance condition is incomplete.".into(),
        ("zh", "final_dimensions_insufficient") => "最终维度证据不足；当前治理条件未完成。".into(),
        ("ja", "final_dimensions_insufficient") => "最終 dimension evidence が不足しており、現在の governance 条件は未完了です。".into(),
        (_, "final_dimensions_insufficient") => "Final-dimension evidence is insufficient; the current governance condition is incomplete.".into(),
        ("zh", "authorization_insufficient") => "适用授权不足或不可确认；验证状态不是授权。".into(),
        ("ja", "authorization_insufficient") => "適用可能な権限が不足または確認できません。検証状態は権限ではありません。".into(),
        (_, "authorization_insufficient") => "Applicable authorization is insufficient or unconfirmed; verification status is not authorization.".into(),
        ("zh", "scope_out_of_range") => "请求超出已声明范围；必须停止并重新检查范围。".into(),
        ("ja", "scope_out_of_range") => "要求が宣言された scope の範囲外で、停止して scope を再確認する必要があります。".into(),
        (_, "scope_out_of_range") => "The request is outside the declared scope; stop and recheck scope.".into(),
        ("zh", "governance_reason_unknown") => "治理阻断原因未知；先检查具体证据和 Runtime 状态，不猜测。".into(),
        ("ja", "governance_reason_unknown") => "governance の停止理由は不明です。推測せず、具体的な evidence と Runtime 状態を確認してください。".into(),
        (_, "governance_reason_unknown") => "The governance stop reason is unknown; inspect concrete evidence and Runtime state instead of guessing.".into(),
        ("zh", "outcome_report_invalid") => "Outcome 报告证据无效或损坏；必须先检查记录。".into(),
        ("ja", "outcome_report_invalid") => "Outcome report evidence が無効または破損しており、先に記録を確認する必要があります。".into(),
        (_, "outcome_report_invalid") => "Outcome report evidence is invalid or corrupt; inspect the record first.".into(),
        ("zh", "preflight_failed") => "preflight 失败；当前治理条件未满足。".into(),
        ("ja", "preflight_failed") => "preflight が失敗し、現在の governance 条件を満たしていません。".into(),
        (_, "preflight_failed") => "Preflight failed; the current governance condition is not satisfied.".into(),
        ("zh", "governance_failed") => "治理检查失败；当前结果已停止。".into(),
        ("ja", "governance_failed") => "governance check が失敗し、現在の結果は停止しています。".into(),
        (_, "governance_failed") => "Governance checks failed; the current result is stopped.".into(),
        ("zh", "lifecycle_failed") => "生命周期门槛失败；当前 Work Item 仍可恢复但未完成。".into(),
        ("ja", "lifecycle_failed") => "ライフサイクル gate が失敗しました。Work Item は復旧可能ですが未完了です。".into(),
        (_, "lifecycle_failed") => "A lifecycle gate failed; the Work Item remains recoverable but is not complete.".into(),
        ("zh", _) => format!("治理原因：{key}。"),
        ("ja", _) => format!("governance 理由：{key}。"),
        (_, _) => format!("Governance reason: {key}."),
    }
}

fn localized_governance_next_action(key: &str, language: &str) -> String {
    match (language, key) {
        ("zh", "acceptance_evidence_insufficient") => "补齐验收证据映射并重新运行对应检查；不需要仅因该缺口重做已通过的验证。".into(),
        ("ja", "acceptance_evidence_insufficient") => "受入れ evidence の mapping と対応する check を補い、検証済みの検査だけを理由なくやり直さず再評価します。".into(),
        (_, "acceptance_evidence_insufficient") => "Complete the acceptance-evidence mapping and rerun its checks; do not redo already-passed verification solely for this gap.".into(),
        ("zh", "intent_alignment_insufficient") => "补齐意图对齐证据并重新评估；不要把验证通过解释成意图已对齐。".into(),
        ("ja", "intent_alignment_insufficient") => "intent alignment evidence を補って再評価し、検証通過を intent alignment 済みとは解釈しません。".into(),
        (_, "intent_alignment_insufficient") => "Complete intent-alignment evidence and reassess; do not treat verification pass as intent alignment.".into(),
        ("zh", "scenario_coverage_insufficient") => "完成缺失场景检查并记录证据，然后重新评估治理条件。".into(),
        ("ja", "scenario_coverage_insufficient") => "不足しているシナリオ check を実行して evidence を記録し、governance 条件を再評価します。".into(),
        (_, "scenario_coverage_insufficient") => "Run the missing scenario checks, record their evidence, and reassess governance.".into(),
        ("zh", "final_dimensions_insufficient") => "补齐最终维度证据并重新评估，不从验证状态推导缺失决定。".into(),
        ("ja", "final_dimensions_insufficient") => "最終 dimension evidence を補って再評価し、検証状態から不足している判断を推論しません。".into(),
        (_, "final_dimensions_insufficient") => "Complete final-dimension evidence and reassess; do not infer the missing decision from verification status.".into(),
        ("zh", "scope_out_of_range") => "停止并让人检查或修订声明范围；没有新的范围授权前不要继续。".into(),
        ("ja", "scope_out_of_range") => "停止し、人間に宣言された scope の確認または修正を依頼します。新しい scope 権限なしに続行しません。".into(),
        (_, "scope_out_of_range") => "Stop and have the declared scope checked or amended; do not proceed without new scope authority.".into(),
        ("zh", "authorization_insufficient") => "检查适用授权及其新鲜度；在有效授权被确认前不要继续。".into(),
        ("ja", "authorization_insufficient") => "適用権限と freshness を確認し、有効な権限が確認されるまで続行しません。".into(),
        (_, "authorization_insufficient") => "Check applicable authorization and freshness; do not proceed until valid authorization is confirmed.".into(),
        _ => localized_governance_reason(key, language),
    }
}

fn localized_finalization_status(projection: &FinalizationProjection, language: &str) -> String {
    let disposition = projection
        .disposition
        .as_deref()
        .map(|value| format!(" ({value})"))
        .unwrap_or_default();
    match (language, projection.state.as_str()) {
        ("zh", "verified") => format!("provider finalization 已验证{disposition}；仍须遵守 Runtime 的 close 条件。"),
        ("ja", "verified") => format!("provider finalization は検証済みです{disposition}。Runtime の close 条件には引き続き従います。"),
        (_, "verified") => format!("Provider finalization is verified{disposition}; Runtime close conditions still apply."),
        ("zh", "receipt_missing") => "provider finalization receipt 缺失；资源事实未知，先重新观察并记录 receipt，不要重复删除。".into(),
        ("ja", "receipt_missing") => "provider finalization receipt が欠落しています。resource facts を再観測して receipt を記録し、削除を繰り返しません。".into(),
        (_, "receipt_missing") => "The provider finalization receipt is missing; re-observe resources and record the receipt, without repeating deletion.".into(),
        ("zh", "record_corrupt") => "provider finalization 记录损坏或无效；先检查并修复绑定证据，不执行未经支持的清理。".into(),
        ("ja", "record_corrupt") => "provider finalization 記録が破損または無効です。先に binding evidence を確認・修復し、未対応の cleanup は実行しません。".into(),
        (_, "record_corrupt") => "The provider finalization record is corrupt or invalid; inspect and repair its bindings before any unsupported cleanup.".into(),
        ("zh", "identity_mismatch") => "provider finalization 身份不匹配；先核对绑定并按恢复规则处理。".into(),
        ("ja", "identity_mismatch") => "provider finalization の identity が不一致です。binding を確認し、復旧ルールに従って処理します。".into(),
        (_, "identity_mismatch") => "Provider finalization identity mismatches; check the binding and follow the recovery rules.".into(),
        ("zh", "cleanup_pending") => "清理后置条件未满足；先重新观察资源，再根据有效计划决定清理。".into(),
        ("ja", "cleanup_pending") => "cleanup の事後条件を満たしていません。resource を再観測し、有効な plan に従って判断します。".into(),
        (_, "cleanup_pending") => "Cleanup postconditions are not satisfied; re-observe resources and act only under a valid plan.".into(),
        ("zh", "not_required") => "当前 Contract 未声明 provider finalization；继续遵守生命周期和人工决定门槛。".into(),
        ("ja", "not_required") => "現在の Contract は provider finalization を宣言していません。ライフサイクルと人間の判断 gate には従います。".into(),
        (_, "not_required") => "The current Contract does not declare provider finalization; lifecycle and human-decision gates still apply.".into(),
        ("zh", _) => format!("provider finalization 状态未知（{}）；先检查恢复条件，不猜测。", projection.state),
        ("ja", _) => format!("provider finalization の状態は不明です（{}）。推測せず復旧条件を確認します。", projection.state),
        (_, _) => format!("Provider finalization state is unknown ({}); inspect recovery conditions instead of guessing.", projection.state),
    }
}

fn localized_finalization_action(projection: &FinalizationProjection, language: &str) -> String {
    match (language, projection.state.as_str(), projection.disposition.as_deref()) {
        ("zh", "verified", Some("retained")) => "receipt 确认资源按计划保留；不要删除，记录或检查 Runtime 允许的 close 决定。".into(),
        ("ja", "verified", Some("retained")) => "receipt は plan に従う resource 保持を確認しています。削除せず、Runtime が許可する close 判断を記録または確認します。".into(),
        (_, "verified", Some("retained")) => "The receipt confirms resources are retained by plan; do not delete them, and record or check the Runtime-permitted close decision.".into(),
        ("zh", "verified", Some("deleted" | "abandoned")) => "finalization 已验证；仅记录必需的人工 close 决定，不能把该建议当作授权。".into(),
        ("ja", "verified", Some("deleted" | "abandoned")) => "finalization は検証済みです。必要な人間の close 判断だけを記録し、この案を権限として扱いません。".into(),
        (_, "verified", Some("deleted" | "abandoned")) => "Finalization is verified; record only the required human close decision, and do not treat this suggestion as authorization.".into(),
        ("zh", "receipt_missing", _) => "先观察精确分支和工作树资源并记录 receipt（使用 work-item finalize-verify）；不要再次删除，完成前不要 close。".into(),
        ("ja", "receipt_missing", _) => "正確な branch と worktree resource を再観測して receipt を記録します（work-item finalize-verify）。削除を繰り返さず、完了前に close しません。".into(),
        (_, "receipt_missing", _) => "Re-observe the exact branch and worktree resources and record the receipt with work-item finalize-verify; do not delete again or close before it is complete.".into(),
        ("zh", "identity_mismatch", _) => "核对 receipt、Contract、Work Item 和 repository 绑定，再按 Runtime 恢复入口处理。".into(),
        ("ja", "identity_mismatch", _) => "receipt、Contract、Work Item、repository の binding を確認し、Runtime の復旧入口に従います。".into(),
        (_, "identity_mismatch", _) => "Check the receipt, Contract, Work Item, and repository bindings, then use the Runtime recovery entry point.".into(),
        ("zh", "record_corrupt", _) => "检查记录完整性并恢复可验证的 receipt；在此之前不执行清理。".into(),
        ("ja", "record_corrupt", _) => "記録の integrity を確認して検証可能な receipt を復旧し、それまでは cleanup を実行しません。".into(),
        (_, "record_corrupt", _) => "Inspect record integrity and recover a verifiable receipt; do not clean up before that.".into(),
        ("zh", "cleanup_pending", _) => "重新观察资源和清理后置条件；只有有效计划确认未清理时才执行精确清理。".into(),
        ("ja", "cleanup_pending", _) => "resource と cleanup の事後条件を再観測します。有効な plan が未 cleanup を確認した場合だけ正確な cleanup を実行します。".into(),
        (_, "cleanup_pending", _) => "Re-observe resources and cleanup postconditions; clean only when a valid plan confirms cleanup is still required.".into(),
        ("zh", "not_required", _) => "审阅当前生命周期状态并记录必需的人工作业；验证状态不扩大为授权。".into(),
        ("ja", "not_required", _) => "現在のライフサイクル状態を確認して必要な人間の判断を記録します。検証状態を権限に拡張しません。".into(),
        (_, "not_required", _) => "Review the current lifecycle state and record any required human decision; verification status is not authorization.".into(),
        ("zh", _, _) => "先检查恢复证据和资源观察；原因不明时保持停止，不执行删除。".into(),
        ("ja", _, _) => "まず復旧 evidence と resource observation を確認します。理由が不明な場合は停止し、削除しません。".into(),
        (_, _, _) => "Inspect recovery evidence and resource observations first; when the reason is unknown, stop and do not delete.".into(),
    }
}

fn render_full_outcome(input: &OutcomeRenderInput, language: &str) -> String {
    let outcome = &input.outcome;
    let language = match language {
        "zh" | "ja" => language,
        _ => "en",
    };
    let historical_kind = outcome.historical_status.as_deref();
    let historical = historical_kind.is_some();
    let superseded = historical_kind == Some("superseded");
    let (marker, status) = if superseded {
        match language {
            "zh" => ("🟡", "历史已替代"),
            "ja" => ("🟡", "履歴として置換済み"),
            _ => ("🟡", "Superseded historical item"),
        }
    } else {
        outcome_status(&outcome.state, outcome.decision_state.as_ref(), language)
    };
    let report = &outcome.human_benefit_report;
    let task_report = outcome.task_outcome_report.as_ref();
    let not_recorded = match language {
        "zh" => "未记录",
        "ja" => "未記録",
        _ => "Not recorded",
    };
    let status_labels = match language {
        "zh" => ("验证状态", "生命周期状态", "人工决定状态", "治理信号"),
        "ja" => (
            "検証状態",
            "ライフサイクル状態",
            "人間の判断状態",
            "ガバナンスシグナル",
        ),
        _ => (
            "Verification",
            "Lifecycle",
            "Human decision",
            "Governance signal",
        ),
    };
    let (
        title,
        completed,
        problems,
        stops,
        resolved,
        avoided,
        remaining,
        unknowns,
        decisions,
        verification,
        impact,
        next_action,
        evidence,
    ) = match language {
        "zh" => (
            "结果",
            "已完成",
            "发现的问题",
            "触发的停止",
            "已解决的问题",
            "避免的风险",
            "剩余风险",
            "未知项",
            "人工决定",
            "验证",
            "影响",
            "下一步",
            "证据",
        ),
        "ja" => (
            "結果",
            "完了したこと",
            "発見された問題",
            "発動した停止",
            "解決した問題",
            "回避したリスク",
            "残存リスク",
            "不明点",
            "人間の判断",
            "検証",
            "影響",
            "次のアクション",
            "証拠",
        ),
        _ => (
            "Task Result",
            "What was completed",
            "Problems found",
            "Stops triggered",
            "Problems resolved",
            "Risks avoided",
            "Remaining risks",
            "Unknowns",
            "Human decisions",
            "Verification",
            "Impact",
            "Next action",
            "Evidence",
        ),
    };
    let not_ready = localized_not_ready_status(outcome, &input.reason_keys, language);
    let no_benefit = match language {
        "zh" => "用户可见收益尚未声明。",
        "ja" => "ユーザー向けの効果はまだ宣言されていません。",
        _ => "User-visible benefit has not been declared.",
    };
    let contract_language = match language {
        "zh" => "验收标准（Contract 原文） / Acceptance criteria (Contract language)",
        "ja" => "受入れ基準（Contract 原文） / Acceptance criteria (Contract language)",
        _ => "Acceptance criteria (Contract language)",
    };
    let lifecycle = localized_lifecycle_status(input.lifecycle_status.clone(), language);
    let human_decision_status = localized_human_decision_status(&input.human_decision, language);
    let governance_signal = localized_governance_signal(outcome.decision_state.as_ref(), language);
    let mut next = (if historical {
        if superseded {
            match language {
                "zh" => {
                    "保留原始历史证据；后续工作由 successor Work Item 负责，不要重新解释为当前失败。"
                }
                "ja" => {
                    "元の履歴 evidence を保持し、後続作業は successor Work Item で行います。現在の失敗とは解釈しません。"
                }
                _ => {
                    "Preserve the historical evidence; the successor owns follow-up work, and this is not a current failure."
                }
            }
        } else {
            match language {
                "zh" => {
                    "保留历史证据；如需当前结果，再用当前 Runtime 重新验证，不要将其解释为当前失败。"
                }
                "ja" => {
                    "履歴 evidence を保持し、current result が必要な場合だけ現在の Runtime で再検証してください。現在の失敗とは解釈しません。"
                }
                _ => {
                    "Preserve the historical evidence; reverify with the current Runtime only when a current result is needed, and do not treat it as a current failure."
                }
            }
        }
    } else {
        match (language, &outcome.state) {
            ("zh", OutcomeState::Verified) => {
                "审阅证据后再决定是否继续；🟢 不代表已授权合并或发布。"
            }
            ("zh", _) if outcome.decision_state == Some(DecisionState::Red) => {
                "修复无效证据并重新验证；在此之前保持停止。"
            }
            ("zh", _) => "补齐缺失证据并重新验证；在此之前保持停止。",
            ("ja", OutcomeState::Verified)
                if outcome.decision_state != Some(DecisionState::Red) =>
            {
                "証拠を確認してから続行を判断してください。🟢 はマージやリリースの承認ではありません。"
            }
            ("ja", _) if outcome.decision_state == Some(DecisionState::Red) => {
                "無効な evidence を修復して再検証してください。それまでは停止します。"
            }
            ("ja", _) => {
                "不足している証拠を補い、再検証してください。それまでは停止状態を維持します。"
            }
            (_, OutcomeState::Verified) if outcome.decision_state != Some(DecisionState::Red) => {
                "Review the evidence before deciding whether to proceed; 🟢 does not authorize merge or release."
            }
            (_, _) if outcome.decision_state == Some(DecisionState::Red) => {
                "Repair the invalid evidence and verify again; remain stopped until then."
            }
            (_, _) => "Repair the missing evidence and verify again; remain stopped until then.",
        }
    }).to_string();
    if let Some(key) = input.reason_keys.iter().find(|key| {
        matches!(
            key.as_str(),
            "acceptance_evidence_insufficient"
                | "intent_alignment_insufficient"
                | "scenario_coverage_insufficient"
                | "final_dimensions_insufficient"
                | "scope_out_of_range"
                | "authorization_insufficient"
        )
    }) {
        next = localized_governance_next_action(key, language);
    }
    if input.archived_unclosed {
        if matches!(
            input.finalization.state.as_str(),
            "not_observed" | "not_required"
        ) {
            next = match language {
                "zh" => "审阅归档证据后记录明确的人工 close 决定；完成 close 前不得开始下一个 Work Item。",
                "ja" => "アーカイブ evidence を確認して明示的な人間の close 判断を記録してください。close 完了前に次の Work Item を開始しないでください。",
                _ => "Review the archive evidence and record the explicit human close decision; do not start another Work Item until close is complete.",
            }
            .into();
        } else {
            next = localized_finalization_action(&input.finalization, language);
        }
    }
    let failed_gate = if historical {
        None
    } else {
        outcome
            .failed_gate
            .as_deref()
            .or_else(|| task_report.and_then(|report| report.failed_gate.as_deref()))
    };
    let recovery_label = match language {
        "zh" => "失败 gate",
        "ja" => "失敗した gate",
        _ => "Failed gate",
    };
    let recovery_action_label = match language {
        "zh" => "恢复条件",
        "ja" => "復旧条件",
        _ => "Recovery condition",
    };
    let failed_gate_item = failed_gate.map(|gate| format!("{recovery_label}: {gate}"));
    let recovery_action = failed_gate.map(|gate| localized_recovery_action(gate, language));
    let mut unknowns_all = outcome.unknowns.clone();
    unknowns_all.extend(report.unknowns.iter().cloned());
    unknowns_all.sort();
    unknowns_all.dedup();
    let mut remaining_risks = Vec::new();
    if let Some(task_report) = task_report {
        remaining_risks.extend(calibrated_risk_claims(
            &task_report.sections.risks,
            language,
        ));
        remaining_risks.extend(calibrated_risk_claims(
            &task_report.sections.warnings,
            language,
        ));
        remaining_risks.extend(calibrated_risk_claims(
            &task_report.sections.residual_risks,
            language,
        ));
        remaining_risks.extend(
            task_report
                .sections
                .limitations
                .iter()
                .map(|claim| format_claim(claim, language, "limitation")),
        );
    }
    if remaining_risks.is_empty() {
        remaining_risks.push(localized_risk_absence(task_report, language));
    }
    remaining_risks.sort();
    remaining_risks.dedup();
    let mut problems_found = task_report
        .map(|report| claim_texts(&report.sections.findings))
        .unwrap_or_default();
    if !historical {
        problems_found.extend(
            input
                .reason_keys
                .iter()
                .map(|key| localized_governance_reason(key, language)),
        );
    } else if matches!(
        outcome.state,
        OutcomeState::NotReady | OutcomeState::Unknown
    ) {
        problems_found.push(not_ready.to_string());
    }
    if let Some(item) = failed_gate_item.as_ref() {
        problems_found.push(item.clone());
    }
    let finalization_relevant =
        input.archived_unclosed || input.finalization.state.as_str() != "not_observed";
    if finalization_relevant {
        problems_found.push(localized_finalization_status(&input.finalization, language));
    }
    let acceptance_results = human_acceptance_results(&outcome.acceptance_results);
    let localized_summary = if historical {
        if superseded {
            match language {
                "zh" => {
                    "该 Work Item 已作为历史 predecessor 被显式替代；原始证据未被重写，也未按当前 Runtime 重验证。"
                }
                "ja" => {
                    "この Work Item は履歴 predecessor として明示的に置換されました。元の evidence は書き換えず、現在の Runtime では再検証していません。"
                }
                _ => {
                    "This Work Item was explicitly superseded as a historical predecessor; original evidence was not rewritten or revalidated under the current Runtime."
                }
            }
        } else {
            match language {
                "zh" => "历史验证证据未按当前 Runtime 重新验证；这不是当前失败。",
                "ja" => {
                    "履歴の検証 evidence は現在の Runtime で再検証されていません。現在の失敗ではありません。"
                }
                _ => {
                    "Historical verification evidence was not revalidated under the current Runtime; this is not a current failure."
                }
            }
        }
    } else {
        localized_outcome_summary(&outcome.state, outcome.decision_state.as_ref(), language)
    };
    let mut completed_items = vec![localized_summary.to_string()];
    if let Some(report) = task_report {
        completed_items.extend(claim_texts(&report.sections.delivered_changes));
    }
    if completed_items.len() == 1 && !acceptance_results.is_empty() {
        completed_items.push(contract_language.to_string());
        completed_items.extend(acceptance_results.clone());
    }
    if !acceptance_results.is_empty()
        && !completed_items.iter().any(|item| item == contract_language)
    {
        completed_items.push(contract_language.to_string());
    }
    let impact_items = if report.user_visible_changes.is_empty() && report.affected_users.is_empty()
    {
        vec![no_benefit.to_string()]
    } else {
        report
            .user_visible_changes
            .iter()
            .chain(report.affected_users.iter())
            .cloned()
            .collect()
    };
    let verification_items = if outcome.evidence_refs.is_empty() {
        vec![status.to_string()]
    } else {
        outcome
            .evidence_refs
            .iter()
            .map(|reference| format!("{status}: {reference}"))
            .collect()
    };
    let decision_items = match &input.human_decision {
        HumanDecisionProjection::Missing => vec![human_decision_status.clone()],
        HumanDecisionProjection::Valid {
            decision,
            assurance,
        } => {
            vec![render_human_decision(
                decision,
                assurance.as_deref(),
                language,
                not_recorded,
            )]
        }
        HumanDecisionProjection::Invalid(reason) => {
            let label = match language {
                "zh" => "未知：结构化人工决定记录无效",
                "ja" => "不明：構造化された人間の判断記録が無効です",
                _ => "Unknown: structured human decision record is invalid",
            };
            vec![format!("{label} ({reason})")]
        }
    };
    let mut stop_items = task_report
        .map(|report| claim_texts(&report.sections.forced_stops))
        .unwrap_or_default();
    if historical {
        stop_items.clear();
    } else if stop_items.is_empty() && !input.reason_keys.is_empty() {
        stop_items = input
            .reason_keys
            .iter()
            .map(|key| localized_governance_reason(key, language))
            .collect();
    } else if matches!(
        outcome.state,
        OutcomeState::NotReady | OutcomeState::Unknown
    ) {
        stop_items = vec![not_ready.to_string()];
    }
    if let Some(action) = recovery_action.as_ref() {
        stop_items.push(format!("{recovery_action_label}: {action}"));
    }
    if finalization_relevant {
        stop_items.push(format!(
            "{recovery_action_label}: {}",
            localized_finalization_action(&input.finalization, language)
        ));
    }
    let resolved_items = task_report
        .map(|report| claim_texts(&report.sections.resolutions))
        .unwrap_or_default();
    let avoided_items = task_report
        .map(|report| claim_texts(&report.sections.avoided_impact))
        .unwrap_or_default();
    let header = format!(
        "Outcome: {marker} {status} — {}\n{title}\n- {}: {}\n- {}: {}\n- {}: {}\n- {}: {}",
        outcome.work_item_id,
        status_labels.0,
        status,
        status_labels.1,
        lifecycle,
        status_labels.2,
        human_decision_status,
        status_labels.3,
        governance_signal,
    );
    format!(
        "{header}\n\n{completed}\n{}\n\n{problems}\n{}\n\n{stops}\n{}\n\n{resolved}\n{}\n\n{avoided}\n{}\n\n{remaining}\n{}\n\n{unknowns}\n{}\n\n{decisions}\n{}\n\n{verification}\n{}\n\n{impact}\n{}\n\n{next_action}\n- {next}\n\n{evidence}\n{}",
        bullet_lines(&completed_items, not_recorded),
        bullet_lines(&problems_found, not_recorded),
        bullet_lines(&stop_items, not_recorded),
        bullet_lines(&resolved_items, not_recorded),
        bullet_lines(&avoided_items, not_recorded),
        bullet_lines(&remaining_risks, not_recorded),
        bullet_lines(&unknowns_all, not_recorded),
        bullet_lines(&decision_items, not_recorded),
        bullet_lines(&verification_items, not_recorded),
        bullet_lines(&impact_items, not_recorded),
        bullet_lines(&outcome.evidence_refs, not_recorded),
    )
}

fn localized_recovery_action(gate: &str, language: &str) -> String {
    match (language, gate) {
        ("zh", "finish.verification") => {
            "记录当前有效的验证证据，重新 preflight，然后重试 finish。".into()
        }
        ("ja", "finish.verification") => {
            "現在の有効な検証 evidence を記録し、preflight を再実行して finish を再試行します。"
                .into()
        }
        (_, "finish.verification") => {
            "Record valid current verification evidence, rerun preflight, and retry finish.".into()
        }
        ("zh", "finish.preflight") => "记录新的非红色 preflight 结果，然后重试 finish。".into(),
        ("ja", "finish.preflight") => {
            "新しい非 red の preflight 結果を記録してから finish を再試行します。".into()
        }
        (_, "finish.preflight") => {
            "Record a fresh non-red preflight result, then retry finish.".into()
        }
        ("zh", _) => "修复失败的治理条件，完成新的检查后重试 finish。".into(),
        ("ja", _) => {
            "失敗した governance 条件を修正し、新しい check 後に finish を再試行します。".into()
        }
        (_, _) => {
            "Repair the failed governance condition, complete fresh checks, and retry finish."
                .into()
        }
    }
}

fn claim_texts(claims: &[OutcomeClaim]) -> Vec<String> {
    claims
        .iter()
        .filter(|claim| !claim.text.trim().is_empty())
        .map(|claim| {
            if claim.inference {
                format!("Inference: {}", claim.text)
            } else {
                claim.text.clone()
            }
        })
        .collect()
}

fn format_claim(claim: &OutcomeClaim, language: &str, kind: &str) -> String {
    let text = if claim.inference {
        format!("Inference: {}", claim.text)
    } else {
        claim.text.clone()
    };
    if kind == "limitation" {
        match language {
            "zh" => format!("限制：{text}"),
            "ja" => format!("制限：{text}"),
            _ => format!("Limitation: {text}"),
        }
    } else {
        text
    }
}

fn calibrated_risk_claims(claims: &[OutcomeClaim], language: &str) -> Vec<String> {
    claims
        .iter()
        .filter(|claim| !claim.text.trim().is_empty())
        .map(|claim| {
            let lower = claim.text.to_ascii_lowercase();
            if lower.contains("test")
                && lower.contains("weak")
                && (lower.contains("no ") || lower.contains("not "))
            {
                let scope = if claim.evidence_refs.is_empty() {
                    match language {
                        "zh" => "引用的检查范围未记录".into(),
                        "ja" => "参照された検査範囲は未記録".into(),
                        _ => "the referenced check scope is not recorded".into(),
                    }
                } else {
                    claim.evidence_refs.join(", ")
                };
                match language {
                    "zh" => format!(
                        "测试弱化扫描：在所引用的检查范围（{scope}）内未触发规则；这不等于证明测试未被弱化。"
                    ),
                    "ja" => format!(
                        "テスト弱化スキャン：参照された検査範囲（{scope}）ではルールは発動していません。テストが弱化されていないことの証明ではありません。"
                    ),
                    _ => format!(
                        "Test-weakening scan: no trigger was recorded for the referenced check scope ({scope}); this does not prove that tests were not weakened."
                    ),
                }
            } else {
                format_claim(claim, language, "risk")
            }
        })
        .collect()
}

fn localized_risk_absence(task_report: Option<&TaskOutcomeReport>, language: &str) -> String {
    if task_report.is_none() {
        return match language {
            "zh" => "风险评估：未记录。".into(),
            "ja" => "リスク評価：未記録。".into(),
            _ => "Risk findings: Not recorded.".into(),
        };
    }
    let has_evidence = task_report.is_some_and(|report| {
        report
            .bindings
            .evidence_refs
            .iter()
            .any(|reference| !reference.trim().is_empty())
    });
    if !has_evidence {
        return match language {
            "zh" => "风险评估：未评估；没有可用的证据范围。".into(),
            "ja" => "リスク評価：未評価。利用可能な evidence 範囲がありません。".into(),
            _ => "Risk findings: Not assessed; no usable evidence scope is recorded.".into(),
        };
    }
    match language {
        "zh" => "风险评估：未记录；现有证据未声明已完成风险检查。".into(),
        "ja" => "リスク評価：未記録。現存する evidence はリスク検査の完了を宣言していません。".into(),
        _ => "Risk findings: Not recorded; the available evidence does not declare that a risk check was completed.".into(),
    }
}

fn bullet_lines(items: &[String], none: &str) -> String {
    if items.is_empty() {
        format!("- {none}")
    } else {
        items
            .iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn human_acceptance_results(results: &[String]) -> Vec<String> {
    let mut normalized: Vec<String> = Vec::new();
    for result in results {
        if result.starts_with(char::is_whitespace) && !normalized.is_empty() {
            if let Some(previous) = normalized.last_mut() {
                previous.push_str(result);
            }
        } else {
            normalized.push(result.clone());
        }
    }
    normalized
}

fn outcome_status(
    state: &OutcomeState,
    decision_state: Option<&DecisionState>,
    language: &str,
) -> (&'static str, &'static str) {
    let decision_state = decision_state.unwrap_or(match state {
        OutcomeState::Verified => &DecisionState::Green,
        OutcomeState::Partial | OutcomeState::NotReady | OutcomeState::Unknown => {
            &DecisionState::Yellow
        }
    });
    let marker = match decision_state {
        DecisionState::Green => "🟢",
        DecisionState::Yellow => "🟡",
        DecisionState::Red => "🔴",
    };
    let label = match (language, state) {
        ("zh", OutcomeState::Verified) => "已声明的验证通过",
        ("zh", OutcomeState::Partial) => "部分验证",
        ("zh", OutcomeState::NotReady) => "验证尚未就绪",
        ("zh", OutcomeState::Unknown) => "验证状态未知",
        ("ja", OutcomeState::Verified) => "宣言された検証済み",
        ("ja", OutcomeState::Partial) => "検証は一部のみ",
        ("ja", OutcomeState::NotReady) => "検証未準備",
        ("ja", OutcomeState::Unknown) => "検証状態不明",
        (_, OutcomeState::Verified) => "Declared verification passed",
        (_, OutcomeState::Partial) => "Partial verification",
        (_, OutcomeState::NotReady) => "Verification not ready",
        (_, OutcomeState::Unknown) => "Verification status unknown",
    };
    (marker, label)
}

fn lifecycle_status(
    root: &Path,
    outcome: &OutcomeV2,
    historical: bool,
    superseded: bool,
) -> String {
    if historical {
        return if superseded {
            "historical_superseded".into()
        } else {
            "historical".into()
        };
    }
    let archive_contract = root
        .join(".ai/work-items/archive")
        .join(format!("{}.contract.json", outcome.work_item_id));
    if archive_contract.is_file() {
        if crate::close_decision_is_valid_for_status(
            root,
            &outcome.work_item_id,
            &outcome.repository_id,
        ) {
            return "closed".into();
        }
        return "archived".into();
    }
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{}.summary.json", outcome.work_item_id));
    fs::read(&summary_path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .and_then(|summary| {
            summary
                .get("state")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "unknown".into())
}

fn localized_lifecycle_status(status: String, language: &str) -> String {
    match (language, status.as_str()) {
        ("zh", "implementation_active") => "实施中".into(),
        ("zh", "checkpointed") => "已建立检查点".into(),
        ("zh", "finish_ready") => "已具备 finish 条件".into(),
        ("zh", "blocked") => "已阻断".into(),
        ("zh", "archived") => "已归档".into(),
        ("zh", "closed") => "已关闭".into(),
        ("zh", "historical") => "历史记录".into(),
        ("zh", "historical_superseded") => "历史记录（已替代）".into(),
        ("zh", _) => "状态未知".into(),
        ("ja", "implementation_active") => "実装中".into(),
        ("ja", "checkpointed") => "チェックポイント済み".into(),
        ("ja", "finish_ready") => "finish 準備完了".into(),
        ("ja", "blocked") => "ブロック中".into(),
        ("ja", "archived") => "アーカイブ済み".into(),
        ("ja", "closed") => "クローズ済み".into(),
        ("ja", "historical") => "履歴".into(),
        ("ja", "historical_superseded") => "履歴（置換済み）".into(),
        ("ja", _) => "状態不明".into(),
        (_, "implementation_active") => "Implementation active".into(),
        (_, "checkpointed") => "Checkpointed".into(),
        (_, "finish_ready") => "Ready to finish".into(),
        (_, "blocked") => "Blocked".into(),
        (_, "archived") => "Archived".into(),
        (_, "closed") => "Closed".into(),
        (_, "historical") => "Historical".into(),
        (_, "historical_superseded") => "Superseded historical record".into(),
        (_, _) => "Unknown".into(),
    }
}

fn localized_human_decision_status(decision: &HumanDecisionProjection, language: &str) -> String {
    match decision {
        HumanDecisionProjection::Missing => match language {
            "zh" => "未记录".into(),
            "ja" => "未記録".into(),
            _ => "Not recorded".into(),
        },
        HumanDecisionProjection::Valid { decision, .. } => match language {
            "zh" => format!("已记录：{}", decision.decision),
            "ja" => format!("記録済み：{}", decision.decision),
            _ => format!("Recorded: {}", decision.decision),
        },
        HumanDecisionProjection::Invalid(_) => match language {
            "zh" => "未知：结构化人工决定记录无效".into(),
            "ja" => "不明：構造化された人間の判断記録が無効".into(),
            _ => "Unknown: structured human decision record is invalid".into(),
        },
    }
}

fn localized_governance_signal(decision_state: Option<&DecisionState>, language: &str) -> String {
    match (language, decision_state) {
        ("zh", Some(DecisionState::Green)) => "绿色；不是人工批准".into(),
        ("zh", Some(DecisionState::Yellow)) => "黄色；需要关注，不是人工批准".into(),
        ("zh", Some(DecisionState::Red)) => "红色；必须停止，不是人工批准".into(),
        ("zh", None) => "未知".into(),
        ("ja", Some(DecisionState::Green)) => "緑。人間の承認ではありません".into(),
        ("ja", Some(DecisionState::Yellow)) => "黄。人間の承認ではありません".into(),
        ("ja", Some(DecisionState::Red)) => "赤。停止が必要で、人間の承認ではありません".into(),
        ("ja", None) => "不明".into(),
        (_, Some(DecisionState::Green)) => "Green; not a human approval".into(),
        (_, Some(DecisionState::Yellow)) => "Yellow; not a human approval".into(),
        (_, Some(DecisionState::Red)) => "Red; stop required, not a human approval".into(),
        (_, None) => "Unknown".into(),
    }
}

fn localized_outcome_summary(
    state: &OutcomeState,
    _decision_state: Option<&DecisionState>,
    language: &str,
) -> &'static str {
    match (language, state) {
        ("zh", OutcomeState::Verified) => "验证证据有效；用户可见收益尚未声明。",
        ("zh", OutcomeState::NotReady) => "结果尚未准备好；具体治理缺口见下方原因。",
        ("zh", OutcomeState::Partial) => "验证证据部分有效；结果仍需关注。",
        ("zh", OutcomeState::Unknown) => "结果状态未知；当前操作已停止，具体原因见下方。",
        ("ja", OutcomeState::Verified) => {
            "検証 evidence は有効ですが、ユーザー向けの効果はまだ宣言されていません。"
        }
        ("ja", OutcomeState::NotReady) => {
            "結果はまだ準備できていません。具体的な governance の不足は下記の理由を参照してください。"
        }
        ("ja", OutcomeState::Partial) => {
            "検証 evidence は一部有効ですが、結果にはまだ確認が必要です。"
        }
        ("ja", OutcomeState::Unknown) => {
            "結果の状態は不明です。現在の操作は停止し、具体的な理由は下記を参照してください。"
        }
        (_, OutcomeState::Verified) => {
            "Verification evidence is valid; user-visible benefit remains explicitly unknown."
        }
        (_, OutcomeState::NotReady) => {
            "The outcome is not ready; see the reasons below for the specific governance gap."
        }
        (_, OutcomeState::Partial) => {
            "Verification evidence is partially valid; the outcome still needs attention."
        }
        (_, OutcomeState::Unknown) => {
            "The outcome state is unknown; the current operation is stopped and the specific reason is listed below."
        }
    }
}

#[derive(Clone, Debug)]
pub enum HumanDecisionProjection {
    Missing,
    Valid {
        decision: HumanDecision,
        assurance: Option<String>,
    },
    Invalid(&'static str),
}

fn load_human_decision(root: &Path, work_item_id: &str) -> HumanDecisionProjection {
    let path = root
        .join(".ai/decisions")
        .join(format!("{work_item_id}.close.json"));
    let Ok(metadata) = fs::symlink_metadata(&path) else {
        return HumanDecisionProjection::Missing;
    };
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return HumanDecisionProjection::Invalid("decision record is not a regular file");
    }
    let Ok(bytes) = fs::read(&path) else {
        return HumanDecisionProjection::Invalid("decision record cannot be read");
    };
    let Ok(record): Result<Value, _> = serde_json::from_slice(&bytes) else {
        return HumanDecisionProjection::Invalid("decision record is not valid JSON");
    };
    if record.get("workItemId").and_then(Value::as_str) != Some(work_item_id) {
        return HumanDecisionProjection::Invalid(
            "decision record Work Item binding is missing or mismatched",
        );
    }
    let expected_repository_id = repository_id(root).to_string();
    if record.get("repositoryId").and_then(Value::as_str) != Some(expected_repository_id.as_str()) {
        return HumanDecisionProjection::Invalid(
            "decision record repository binding is missing or mismatched",
        );
    }
    if record.get("state").and_then(Value::as_str) != Some("closed") {
        return HumanDecisionProjection::Invalid("decision record state is not closed");
    }
    if record.get("decisionState").and_then(Value::as_str) != Some("confirmed") {
        return HumanDecisionProjection::Invalid("decision record is not confirmed");
    }
    let Some(structured) = record.get("structuredDecision").cloned() else {
        return HumanDecisionProjection::Invalid("structured decision is missing");
    };
    let assurance = record
        .get("assurance")
        .or_else(|| structured.get("assurance"))
        .and_then(Value::as_str)
        .filter(|value| {
            matches!(
                *value,
                "self_declared"
                    | "repository_verified"
                    | "provider_verified"
                    | "enterprise_verified"
            )
        })
        .map(str::to_owned);
    let Ok(decision): Result<HumanDecision, _> = serde_json::from_value(structured) else {
        return HumanDecisionProjection::Invalid(
            "structured decision fields are incomplete or unknown",
        );
    };
    for value in [
        decision.decision.as_str(),
        decision.actor.as_str(),
        decision.authority_source.as_str(),
        decision.reason.as_str(),
        decision.decided_at.as_str(),
    ] {
        if value.trim().is_empty() {
            return HumanDecisionProjection::Invalid(
                "structured decision contains an empty required field",
            );
        }
    }
    if record.get("humanDecision").and_then(Value::as_str) != Some(decision.decision.as_str()) {
        return HumanDecisionProjection::Invalid(
            "decision record summary does not match structured decision",
        );
    }
    HumanDecisionProjection::Valid {
        decision,
        assurance,
    }
}

fn render_human_decision(
    decision: &HumanDecision,
    assurance: Option<&str>,
    language: &str,
    not_recorded: &str,
) -> String {
    let (
        decision_label,
        actor_label,
        authority_label,
        assurance_label,
        reason_label,
        evidence_label,
        policy_label,
        decided_label,
        resume_label,
    ) = match language {
        "zh" => (
            "决定",
            "执行人",
            "授权来源",
            "保证级别",
            "理由",
            "证据引用",
            "策略引用",
            "决定时间",
            "恢复条件",
        ),
        "ja" => (
            "判断",
            "実行者",
            "権限の出所",
            "保証レベル",
            "理由",
            "evidence 参照",
            "policy 参照",
            "判断日時",
            "再開条件",
        ),
        _ => (
            "Decision",
            "Actor",
            "Authority source",
            "Assurance level",
            "Reason",
            "Evidence refs",
            "Policy refs",
            "Decided at",
            "Resume condition",
        ),
    };
    let evidence_refs = if decision.evidence_refs.is_empty() {
        not_recorded.to_string()
    } else {
        decision.evidence_refs.join(", ")
    };
    let policy_refs = if decision.policy_refs.is_empty() {
        not_recorded.to_string()
    } else {
        decision.policy_refs.join(", ")
    };
    let resume_condition = decision.resume_condition.as_deref().unwrap_or(not_recorded);
    let unknown_assurance = match language {
        "zh" => "未知",
        "ja" => "不明",
        _ => "Unknown",
    };
    let assurance = assurance.unwrap_or(unknown_assurance);
    format!(
        "{decision_label}: {}\n  {actor_label}: {}\n  {authority_label}: {}\n  {assurance_label}: {assurance}\n  {reason_label}: {}\n  {evidence_label}: {evidence_refs}\n  {policy_label}: {policy_refs}\n  {decided_label}: {}\n  {resume_label}: {resume_condition}",
        decision.decision,
        decision.actor,
        decision.authority_source,
        decision.reason,
        decision.decided_at,
    )
}

#[cfg(test)]
mod render_tests {
    use super::{
        FinalizationProjection, HumanDecisionProjection, OutcomeRenderInput,
        assemble_outcome_render_input_with_hook, render_human_outcome,
    };
    use crate::{
        WorkItemStartOptions, attach, checkpoint_work_item, preflight_work_item,
        record_verification, start_work_item_with_options,
    };
    use cockpit_core::{DecisionState, Digest};
    use cockpit_protocol::{HumanBenefitReport, HumanDecision, OutcomeState, OutcomeV2};
    use std::{fs, process::Command};

    fn base_outcome() -> OutcomeV2 {
        OutcomeV2 {
            schema_version: 2,
            repository_id: "sha256:aaaa".into(),
            work_item_id: "WI-RENDER-TEST".into(),
            state: OutcomeState::Verified,
            decision_state: Some(DecisionState::Green),
            summary: "verified".into(),
            acceptance_results: vec![],
            unknowns: vec![],
            evidence_refs: vec![".ai/evidence/WI-RENDER-TEST.verification.json".into()],
            human_benefit_report: HumanBenefitReport {
                state: OutcomeState::Verified,
                user_visible_changes: vec![],
                affected_users: vec![],
                unknowns: vec![],
                evidence_refs: vec![],
            },
            task_outcome_report: None,
            failed_gate: None,
            recovery_condition: None,
            recovery_decision: None,
            historical_status: None,
            governance_reasons: Vec::new(),
            finalization: None,
        }
    }

    fn input(
        human_decision: HumanDecisionProjection,
        archived_unclosed: bool,
    ) -> OutcomeRenderInput {
        OutcomeRenderInput {
            outcome: base_outcome(),
            human_decision,
            archived_unclosed,
            lifecycle_status: "implementation_active".into(),
            finalization: FinalizationProjection {
                state: "not_observed".into(),
                error_code: None,
                disposition: None,
                action: "use_current_runtime_observation".into(),
                reliable: false,
            },
            reason_keys: Vec::new(),
            assembly: None,
        }
    }

    fn decision() -> HumanDecision {
        HumanDecision {
            decision: "approved".into(),
            actor: "human:owner".into(),
            authority_source: "explicit-test".into(),
            reason: "reviewed the evidence".into(),
            evidence_refs: vec![],
            policy_refs: vec![],
            decided_at: "2026-09-08T00:00:00Z".into(),
            resume_condition: None,
        }
    }

    fn observed_repository() -> tempfile::TempDir {
        let directory = tempfile::tempdir().expect("tempdir");
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .current_dir(directory.path())
                .status()
                .expect("git init")
                .success()
        );
        attach(directory.path()).expect("attach");
        let id = "WI-OBSERVATION-BOUNDARY";
        start_work_item_with_options(
            directory.path(),
            id,
            "test observation-bound Outcome assembly",
            "detect deterministic lifecycle fact changes",
            &["**".into()],
            &WorkItemStartOptions {
                authority: "authorized".into(),
                ..Default::default()
            },
        )
        .expect("start");
        let contract = directory
            .path()
            .join(format!(".ai/work-items/active/{id}.contract.json"));
        preflight_work_item(directory.path(), &contract).expect("preflight");
        checkpoint_work_item(directory.path(), id).expect("checkpoint");
        record_verification(
            directory.path(),
            id,
            &serde_json::json!({"passed": true}),
            "0.2.89",
            &Digest::sha256_bytes(b"test-runtime"),
        )
        .expect("verification");
        directory
    }

    #[test]
    fn outcome_assembly_retries_once_after_deterministic_record_change() {
        let directory = observed_repository();
        let id = "WI-OBSERVATION-BOUNDARY";
        let summary = directory
            .path()
            .join(format!(".ai/work-items/active/{id}.summary.json"));
        let mut hook = |attempt| {
            if attempt == 1 {
                fs::write(
                    &summary,
                    serde_json::to_vec_pretty(&serde_json::json!({
                        "state": "checkpointed",
                        "deterministicMutation": true
                    }))
                    .expect("summary JSON"),
                )
                .expect("mutate summary");
            }
        };
        let input =
            assemble_outcome_render_input_with_hook(directory.path(), id, None, Some(&mut hook))
                .expect("bounded retry succeeds on stable second observation");
        assert_eq!(input.assembly.expect("assembly metadata").attempts, 2);
        assert_eq!(input.outcome.finalization, Some(input.finalization.clone()));
        assert_eq!(input.outcome.governance_reasons, input.reason_keys);
    }

    #[test]
    fn outcome_assembly_stops_after_bounded_record_changes() {
        let directory = observed_repository();
        let id = "WI-OBSERVATION-BOUNDARY";
        let summary = directory
            .path()
            .join(format!(".ai/work-items/active/{id}.summary.json"));
        let mut hook = |attempt| {
            fs::write(
                &summary,
                serde_json::to_vec_pretty(&serde_json::json!({
                    "state": "checkpointed",
                    "deterministicMutation": attempt
                }))
                .expect("summary JSON"),
            )
            .expect("mutate summary");
        };
        let error =
            assemble_outcome_render_input_with_hook(directory.path(), id, None, Some(&mut hook))
                .expect_err("continuous mutation must fail closed");
        assert!(error.to_string().contains("bounded assembly"));
        assert!(error.to_string().contains("result is unknown"));
    }

    #[test]
    fn renderer_is_driven_by_in_memory_facts_for_missing_decision() {
        let text = render_human_outcome(&input(HumanDecisionProjection::Missing, false), "en");
        assert!(text.contains("Human decisions\n- Not recorded"), "{text}");
    }

    #[test]
    fn renderer_preserves_valid_decision_and_assurance() {
        let text = render_human_outcome(
            &input(
                HumanDecisionProjection::Valid {
                    decision: decision(),
                    assurance: Some("repository_verified".into()),
                },
                false,
            ),
            "en",
        );
        assert!(text.contains("Decision: approved"), "{text}");
        assert!(text.contains("Actor: human:owner"), "{text}");
        assert!(
            text.contains("Assurance level: repository_verified"),
            "{text}"
        );
    }

    #[test]
    fn invalid_decision_is_visible_as_unknown() {
        let text = render_human_outcome(
            &input(
                HumanDecisionProjection::Invalid("decision record is not closed"),
                false,
            ),
            "en",
        );
        assert!(
            text.contains("Unknown: structured human decision record is invalid"),
            "{text}"
        );
        assert!(text.contains("decision record is not closed"), "{text}");
    }

    #[test]
    fn archived_unclosed_recommends_close_decision() {
        let text = render_human_outcome(&input(HumanDecisionProjection::Missing, true), "en");
        assert!(
            text.contains("record the explicit human close decision"),
            "{text}"
        );
    }

    #[test]
    fn finalization_recovery_actions_follow_classified_facts() {
        let mut missing = input(HumanDecisionProjection::Missing, true);
        missing.finalization.state = "receipt_missing".into();
        missing.finalization.action = "inspect_resources_and_record_receipt".into();
        let missing_text = render_human_outcome(&missing, "en");
        let missing_summary = super::render_human_outcome_with_view(
            &missing,
            "en",
            super::OutcomeRenderView::Summary,
        );
        assert!(missing_text.contains("Re-observe"), "{missing_text}");
        assert!(missing_summary.contains("Re-observe"), "{missing_summary}");
        assert!(!missing_text.contains("delete the exact"), "{missing_text}");
        assert!(
            !missing_summary.contains("delete the exact"),
            "{missing_summary}"
        );

        let mut retained = input(HumanDecisionProjection::Missing, true);
        retained.finalization.state = "verified".into();
        retained.finalization.disposition = Some("retained".into());
        let retained_text = render_human_outcome(&retained, "en");
        let retained_summary = super::render_human_outcome_with_view(
            &retained,
            "en",
            super::OutcomeRenderView::Summary,
        );
        assert!(retained_text.contains("do not delete"), "{retained_text}");
        assert!(
            retained_text.contains("retained by plan"),
            "{retained_text}"
        );
        assert!(
            retained_summary.contains("do not delete"),
            "{retained_summary}"
        );
        assert!(
            retained_summary.contains("retained by plan"),
            "{retained_summary}"
        );

        let mut deleted = input(HumanDecisionProjection::Missing, true);
        deleted.finalization.state = "verified".into();
        deleted.finalization.disposition = Some("deleted".into());
        let deleted_text = render_human_outcome(&deleted, "en");
        let deleted_summary = super::render_human_outcome_with_view(
            &deleted,
            "en",
            super::OutcomeRenderView::Summary,
        );
        assert!(
            deleted_text.contains("required human close decision"),
            "{deleted_text}"
        );
        assert!(
            deleted_summary.contains("required human close decision"),
            "{deleted_summary}"
        );
        assert!(!deleted_text.contains("delete the exact"), "{deleted_text}");
        assert!(
            !deleted_summary.contains("delete the exact"),
            "{deleted_summary}"
        );

        for (state, action, marker) in [
            (
                "identity_mismatch",
                "inspect_binding_and_recovery_evidence",
                "identity mismatches",
            ),
            (
                "record_corrupt",
                "inspect_binding_and_recovery_evidence",
                "corrupt or invalid",
            ),
        ] {
            let mut classified = input(HumanDecisionProjection::Missing, true);
            classified.finalization.state = state.into();
            classified.finalization.action = action.into();
            for language in ["en", "zh", "ja"] {
                let summary = super::render_human_outcome_with_view(
                    &classified,
                    language,
                    super::OutcomeRenderView::Summary,
                );
                let full = super::render_human_outcome_with_view(
                    &classified,
                    language,
                    super::OutcomeRenderView::Full,
                );
                assert!(
                    summary.contains(marker) || language != "en",
                    "{language}: {summary}"
                );
                assert!(
                    full.contains(marker) || language != "en",
                    "{language}: {full}"
                );
                assert!(
                    summary.contains("inspect")
                        || summary.contains("Inspect")
                        || summary.contains("Check")
                        || summary.contains("检查")
                        || summary.contains("核对")
                        || summary.contains("確認"),
                    "{language}: {summary}"
                );
                assert!(
                    full.contains("inspect")
                        || full.contains("Inspect")
                        || full.contains("Check")
                        || full.contains("检查")
                        || full.contains("核对")
                        || full.contains("確認"),
                    "{language}: {full}"
                );
            }
        }
    }

    #[test]
    fn human_renderer_preserves_blockers_and_unknowns_in_summary() {
        let mut input = input(HumanDecisionProjection::Missing, false);
        input.outcome.state = OutcomeState::Unknown;
        input.outcome.decision_state = Some(DecisionState::Red);
        input.outcome.failed_gate = Some("finish.governance".into());
        input.outcome.recovery_condition = Some("repair governance evidence".into());
        input.outcome.unknowns = vec![
            "acceptance_evidence_missing".into(),
            "intent_alignment_missing".into(),
            "user_visible_benefit_not_declared".into(),
        ];
        input.reason_keys = vec![
            "acceptance_evidence_insufficient".into(),
            "intent_alignment_insufficient".into(),
        ];
        let summary =
            super::render_human_outcome_with_view(&input, "en", super::OutcomeRenderView::Summary);
        assert!(summary.contains("acceptance evidence"), "{summary}");
        assert!(summary.contains("intent-alignment"), "{summary}");
        assert!(
            summary.contains("Failed gate: finish.governance"),
            "{summary}"
        );
        assert!(summary.contains("Recovery condition:"), "{summary}");
        assert!(
            summary.contains("User-visible benefit has not been declared"),
            "{summary}"
        );
    }

    #[test]
    fn superseded_history_is_not_rendered_as_current_failure() {
        let mut input = input(HumanDecisionProjection::Missing, false);
        input.outcome.historical_status = Some("superseded".into());
        let text = render_human_outcome(&input, "en");
        assert!(
            text.starts_with("Outcome: 🟡 Superseded historical item"),
            "{text}"
        );
        assert!(text.contains("this is not a current failure"), "{text}");
    }
}

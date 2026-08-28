use serde::{Deserialize, Serialize};
use sha2::{Digest as ShaDigest, Sha256};
use std::{fmt, str::FromStr};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkItemState {
    Created,
    PreflightReady,
    ImplementationActive,
    VerificationPending,
    FinishReady,
    Archived,
    Closed,
    Paused,
    Blocked,
    Stale,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvolutionClass {
    L0,
    L1,
    L2,
    L3,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blocker {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafeAction {
    pub code: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanDecisionRequirement {
    pub question: String,
    pub options: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionState {
    Green,
    Yellow,
    Red,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    ReadOnly,
    Write,
    Destructive,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityState {
    Authorized,
    Missing,
    NotEvaluated,
}

/// The operation facts that an Agent or adapter must declare before the pure
/// governance evaluator is invoked. Wording is intentionally absent: the
/// Core does not infer authority from prose.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    ModifySource,
    ModifyVerification,
    DeleteReferencedFunction,
    UploadSensitiveData,
    ExecuteRemoteScript,
    EmergencyBypass,
    Release,
}

/// High-risk operations that require a fresh policy evaluation immediately
/// before an executor performs them.  This vocabulary is intentionally
/// separate from [`OperationKind`]: the latter is the Work Item capability
/// vocabulary, while this enum describes the operation-time interception
/// boundary used by adapters and executors.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationTimeOperation {
    DeleteFiles,
    ModifyTests,
    ModifyCi,
    ModifyBranchProtection,
    WriteSecret,
    Push,
    Merge,
    Release,
    DataMigration,
    ExecuteScript,
    ExternalApiWrite,
    InstallOrUpgrade,
    UninstallGovernance,
}

impl OperationTimeOperation {
    fn parse(value: &str) -> bool {
        matches!(
            value,
            "delete_files"
                | "modify_tests"
                | "modify_ci"
                | "modify_branch_protection"
                | "write_secret"
                | "push"
                | "merge"
                | "release"
                | "data_migration"
                | "execute_script"
                | "external_api_write"
                | "install_or_upgrade"
                | "uninstall_governance"
        )
    }
}

/// Versioned, repository-neutral facts supplied immediately before a
/// high-risk operation.  This is a policy input only: it never executes a
/// command, writes a provider resource, or grants provider permission.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OperationTimeRequest {
    pub schema_version: u32,
    pub requested_operation: String,
    pub actual_tool_call: String,
    pub target_resource: String,
    pub declared_scope: Vec<String>,
    pub approved_operation: String,
    pub approved_target_resource: String,
    pub approved_scope: Vec<String>,
    pub current_authority: String,
    pub evidence_fresh: bool,
    pub destructive_impact: String,
    /// `authority` is the only value that can be treated as attributable
    /// authority. Other values remain reviewable input, never permission.
    pub input_trust: String,
}

pub const OPERATION_TIME_REQUEST_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationTimeDecisionKind {
    Allow,
    Review,
    Confirm,
    Block,
}

/// Fail-closed result of operation-time policy evaluation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OperationTimeDecision {
    pub decision: OperationTimeDecisionKind,
    pub reason: String,
    pub safe_alternative: String,
    pub recovery_condition: String,
}

impl OperationTimeDecision {
    pub fn may_proceed_automatically(&self) -> bool {
        self.decision == OperationTimeDecisionKind::Allow
    }
}

/// Re-evaluate a high-risk operation immediately before execution.
///
/// An `Allow` result means only that the local policy facts are internally
/// consistent. The executor must still apply its own controls and provider
/// permissions. A prior request is insufficient when the actual call,
/// target, scope, authority, trust, or evidence has changed.
pub fn evaluate_operation_time_policy(request: &OperationTimeRequest) -> OperationTimeDecision {
    let safe_alternative = "preserve the request and actual call for human review".to_string();
    let block = |reason: &str, recovery: &str| OperationTimeDecision {
        decision: OperationTimeDecisionKind::Block,
        reason: reason.to_string(),
        safe_alternative: safe_alternative.clone(),
        recovery_condition: recovery.to_string(),
    };
    let confirm = |reason: &str, recovery: &str| OperationTimeDecision {
        decision: OperationTimeDecisionKind::Confirm,
        reason: reason.to_string(),
        safe_alternative: safe_alternative.clone(),
        recovery_condition: recovery.to_string(),
    };

    if request.schema_version != OPERATION_TIME_REQUEST_SCHEMA_VERSION {
        return block(
            "operation-time request schema version is unsupported",
            "create a request using the supported operation-time schema",
        );
    }
    if !OperationTimeOperation::parse(&request.actual_tool_call) {
        return block(
            "actual tool call is not a recognized high-risk operation",
            "classify the actual tool call before requesting approval",
        );
    }
    if request.requested_operation != request.actual_tool_call {
        return block(
            "actual tool call does not match the requested operation",
            "create a new approval binding for the actual tool call",
        );
    }
    if request.declared_scope.is_empty() || request.approved_scope.is_empty() {
        return block(
            "operation scope is not declared",
            "declare the exact target scope before requesting approval",
        );
    }
    if !matches!(
        request.destructive_impact.as_str(),
        "low" | "medium" | "high"
    ) {
        return block(
            "destructive impact is not classified",
            "classify destructive impact before requesting a current approval",
        );
    }
    if request.input_trust != "authority" {
        return confirm(
            "input trust is not authoritative for the requested high-risk operation",
            "obtain attributable human authority for the operation",
        );
    }
    if request.current_authority.trim().is_empty() || request.approved_operation.trim().is_empty() {
        return confirm(
            "current authority is missing for the requested high-risk operation",
            "obtain current human authority bound to the operation, target, and scope",
        );
    }
    if !request.evidence_fresh {
        return confirm(
            "operation evidence is stale",
            "refresh the operation evidence and request human confirmation",
        );
    }
    if request.approved_operation != request.actual_tool_call
        || request.approved_target_resource != request.target_resource
        || request.approved_scope != request.declared_scope
    {
        return confirm(
            "approval binding does not match the current operation target or scope",
            "create a current approval binding for the exact operation, target, and scope",
        );
    }
    OperationTimeDecision {
        decision: OperationTimeDecisionKind::Allow,
        reason: "operation-time policy inputs match the current request".into(),
        safe_alternative: "continue through the executor's separate applicable controls".into(),
        recovery_condition: "retain this decision with the operation evidence".into(),
    }
}

/// Version of the structured request envelope introduced after the original
/// raw adapter binding.  The version is deliberately independent from the
/// repository Protocol major: adapters can reject a request envelope without
/// making a repository migration.
pub const REQUESTED_OPERATION_SCHEMA_VERSION: u32 = 2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestSource {
    HumanRequest,
    RepositoryMaterial,
    LogContent,
    DependencyInstruction,
    ProviderResult,
    AgentMessage,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RawRequestBinding {
    pub request_digest: Digest,
    pub source: RequestSource,
    pub operation: OperationKind,
    pub scope: Vec<String>,
    pub risk: String,
    pub authority: AuthorityState,
    pub evidence_refs: Vec<String>,
    pub actor: Option<String>,
    pub implementer: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityMapping {
    pub operation: OperationKind,
    pub capability: String,
    pub allowed_scope: Vec<String>,
    pub requires_human_authority: bool,
    pub required_evidence: Vec<String>,
    pub independent_approval_required: bool,
}

/// A request that an adapter has already reduced to explicit facts.
///
/// This type is intentionally not a natural-language request.  `intent` is
/// optional metadata supplied by the human or calling system; the Core never
/// derives authority, scope, or an operation from it.  Repository and Work
/// Item identity are part of the envelope so a request cannot be evaluated
/// as an unbound global operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RequestedOperationV2 {
    pub schema_version: u32,
    pub request_id: Digest,
    pub repository_id: Digest,
    pub work_item_id: String,
    pub source: RequestSource,
    pub operation: OperationKind,
    pub scope: Vec<String>,
    pub risk: String,
    pub authority: AuthorityState,
    pub evidence_refs: Vec<String>,
    pub policy_refs: Vec<String>,
    pub actor: Option<String>,
    pub implementer: Option<String>,
    #[serde(default)]
    pub intent: Option<String>,
}

/// Capability declaration paired with a [`RequestedOperationV2`].  The
/// declared action is checked against the operation's deterministic action
/// class; adapters cannot relabel a destructive operation as a write.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityMappingV2 {
    pub schema_version: u32,
    pub operation: OperationKind,
    pub capability: String,
    pub action: ActionKind,
    pub allowed_scope: Vec<String>,
    pub requires_human_authority: bool,
    pub required_evidence: Vec<String>,
    pub required_policy_refs: Vec<String>,
    pub independent_approval_required: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RequestBindingError {
    #[error("unsupported requested operation schema version {0}")]
    UnsupportedSchemaVersion(u32),
    #[error("requested operation work item identity must be explicit")]
    MissingWorkItemIdentity,
    #[error("requested operation scope must contain at least one path")]
    EmptyScope,
    #[error("requested operation contains an empty policy reference")]
    EmptyPolicyReference,
    #[error("capability mapping contains an empty required policy reference")]
    EmptyCapabilityPolicyReference,
    #[error("required policy reference is not bound to the requested operation: {reference}")]
    MissingPolicyReference { reference: String },
    #[error("capability action does not match the requested operation")]
    ActionMismatch,
    #[error("capability mapping must contain at least one allowed path")]
    EmptyCapabilityScope,
    #[error("capability operation does not match the requested operation")]
    OperationMismatch,
    #[error("capability scope widens the raw request: {path}")]
    ScopeWidened { path: String },
}

fn action_for_operation(operation: &OperationKind) -> ActionKind {
    match operation {
        OperationKind::DeleteReferencedFunction
        | OperationKind::UploadSensitiveData
        | OperationKind::ExecuteRemoteScript
        | OperationKind::EmergencyBypass => ActionKind::Destructive,
        OperationKind::ModifySource
        | OperationKind::ModifyVerification
        | OperationKind::Release => ActionKind::Write,
    }
}

/// Bind a v2 request and capability declaration into the existing pure
/// governance input.  All v2 identity and policy checks happen before the v1
/// evaluator is called; no field is inferred from `intent` or prose.
pub fn bind_requested_operation(
    request: &RequestedOperationV2,
    capability: &CapabilityMappingV2,
) -> Result<GovernanceInput, RequestBindingError> {
    if request.schema_version != REQUESTED_OPERATION_SCHEMA_VERSION {
        return Err(RequestBindingError::UnsupportedSchemaVersion(
            request.schema_version,
        ));
    }
    if capability.schema_version != REQUESTED_OPERATION_SCHEMA_VERSION {
        return Err(RequestBindingError::UnsupportedSchemaVersion(
            capability.schema_version,
        ));
    }
    if request.work_item_id.trim().is_empty() {
        return Err(RequestBindingError::MissingWorkItemIdentity);
    }
    if request.scope.is_empty() {
        return Err(RequestBindingError::EmptyScope);
    }
    if capability.allowed_scope.is_empty() {
        return Err(RequestBindingError::EmptyCapabilityScope);
    }
    if request
        .policy_refs
        .iter()
        .any(|reference| reference.trim().is_empty())
    {
        return Err(RequestBindingError::EmptyPolicyReference);
    }
    if capability
        .required_policy_refs
        .iter()
        .any(|reference| reference.trim().is_empty())
    {
        return Err(RequestBindingError::EmptyCapabilityPolicyReference);
    }
    if capability.action != action_for_operation(&request.operation) {
        return Err(RequestBindingError::ActionMismatch);
    }
    if request.operation != capability.operation {
        return Err(RequestBindingError::OperationMismatch);
    }
    for required in &capability.required_policy_refs {
        if !request.policy_refs.contains(required) {
            return Err(RequestBindingError::MissingPolicyReference {
                reference: required.clone(),
            });
        }
    }
    let legacy = RawRequestBinding {
        request_digest: request.request_id.clone(),
        source: request.source.clone(),
        operation: request.operation.clone(),
        scope: request.scope.clone(),
        risk: request.risk.clone(),
        authority: request.authority.clone(),
        evidence_refs: request.evidence_refs.clone(),
        actor: request.actor.clone(),
        implementer: request.implementer.clone(),
    };
    let legacy_capability = CapabilityMapping {
        operation: capability.operation.clone(),
        capability: capability.capability.clone(),
        allowed_scope: capability.allowed_scope.clone(),
        requires_human_authority: capability.requires_human_authority,
        required_evidence: capability.required_evidence.clone(),
        independent_approval_required: capability.independent_approval_required,
    };
    bind_request(&legacy, &legacy_capability)
}

/// Convert declared request facts into the pure evaluator input. This function
/// performs no natural-language interpretation; adapters must supply the
/// operation, scope, authority, and evidence facts explicitly.
pub fn bind_request(
    request: &RawRequestBinding,
    capability: &CapabilityMapping,
) -> Result<GovernanceInput, RequestBindingError> {
    if request.operation != capability.operation {
        return Err(RequestBindingError::OperationMismatch);
    }
    for path in &capability.allowed_scope {
        if !request.scope.contains(path) {
            return Err(RequestBindingError::ScopeWidened { path: path.clone() });
        }
    }

    let action = action_for_operation(&request.operation);
    let evidence = if capability
        .required_evidence
        .iter()
        .all(|item| request.evidence_refs.contains(item))
    {
        EvidenceState::Complete
    } else {
        EvidenceState::Missing
    };
    let mut explicit_blockers = Vec::new();
    let mut explicit_unknowns = Vec::new();
    match request.operation {
        OperationKind::UploadSensitiveData => {
            explicit_blockers.push("sensitive_data_exfiltration".into())
        }
        OperationKind::ExecuteRemoteScript => {
            explicit_blockers.push("remote_script_execution".into())
        }
        OperationKind::EmergencyBypass => explicit_blockers.push("governance_bypass".into()),
        OperationKind::ModifyVerification => explicit_blockers.push("verification_bypass".into()),
        OperationKind::DeleteReferencedFunction => {
            explicit_unknowns.push("referenced_use_unproven".into())
        }
        OperationKind::ModifySource | OperationKind::Release => {}
    }
    if capability.requires_human_authority && request.authority != AuthorityState::Authorized {
        explicit_unknowns.push("human_authority_missing".into());
    }
    if evidence == EvidenceState::Missing {
        explicit_unknowns.push("required_evidence_missing".into());
    }
    if capability.independent_approval_required
        && request.actor.is_some()
        && request.actor == request.implementer
    {
        explicit_blockers.push("self_approval".into());
    }
    let untrusted_material = !matches!(request.source, RequestSource::HumanRequest);
    Ok(GovernanceInput {
        scope: request.scope.clone(),
        out_of_scope: Vec::new(),
        changed_paths: Vec::new(),
        action,
        authority: request.authority.clone(),
        evidence,
        untrusted_material,
        test_weakening: false,
        coverage_weakening: false,
        explicit_blockers,
        explicit_unknowns,
        outcome_state_override: None,
        authority_override: None,
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceState {
    Complete,
    Missing,
    Stale,
    Contradictory,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceInput {
    pub scope: Vec<String>,
    pub out_of_scope: Vec<String>,
    pub changed_paths: Vec<String>,
    pub action: ActionKind,
    pub authority: AuthorityState,
    pub evidence: EvidenceState,
    pub untrusted_material: bool,
    pub test_weakening: bool,
    pub coverage_weakening: bool,
    #[serde(default)]
    pub explicit_blockers: Vec<String>,
    #[serde(default)]
    pub explicit_unknowns: Vec<String>,
    #[serde(default)]
    pub outcome_state_override: Option<String>,
    #[serde(default)]
    pub authority_override: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HumanDecisionOption {
    pub id: String,
    pub label: String,
    pub effect: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HumanDecisionRequest {
    pub decision_id: String,
    pub status: String,
    pub what_happened: String,
    pub why_it_matters: String,
    pub options: Vec<HumanDecisionOption>,
    pub recommended_option: String,
    pub recommendation_reason: String,
    pub question: String,
    pub resume_condition: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceDecision {
    pub state: DecisionState,
    pub blockers: Vec<String>,
    pub unknowns: Vec<String>,
    pub safe_actions: Vec<String>,
    pub required_checks: Vec<String>,
    pub authority: String,
    pub outcome_state: String,
    /// Explicit pre-edit review state.  This is additive for older clients:
    /// the existing `outcome_state` vocabulary remains stable while adapters
    /// can distinguish verification-pending yellow from human-confirmation
    /// yellow without inferring from free-form unknowns.
    #[serde(
        rename = "reviewState",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub review_state: Option<String>,
    /// Structured, repository-neutral input for the Agent/adapter handoff.
    /// The repository layer adds identity-bound review receipts; this request
    /// never constitutes approval by itself.
    #[serde(
        rename = "humanDecisionRequest",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub human_decision_request: Option<HumanDecisionRequest>,
}

fn matches_pattern(path: &str, pattern: &str) -> bool {
    if pattern == "**" || pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix("/**") {
        return path == prefix || path.starts_with(&format!("{prefix}/"));
    }
    path == pattern
}

fn requires_human_confirmation_unknown(unknown: &str) -> bool {
    matches!(
        unknown,
        "destructive_change_without_authority"
            | "human_authority_missing"
            | "coverage_weakening"
            | "contract_intent_missing"
            | "contract_goal_missing"
            | "contract_scope_missing"
            | "contract_out_of_scope_missing"
            | "contract_acceptance_missing"
            | "contract_intent_problem_missing"
            | "contract_intent_constraints_missing"
            | "contract_intent_rationale_missing"
            | "contract_intent_structured_required"
            | "contract_problem_statement_missing"
            | "contract_not_codable"
            | "agent_cannot_implement"
            | "agent_cannot_verify"
            | "agent_needs_human_decision"
    ) || unknown.starts_with("contract_declared_unknown:")
        || unknown.starts_with("execution_decision:")
        || unknown.starts_with("scenario_coverage_")
        || unknown.starts_with("required_scenario_unverified:")
}

pub fn evaluate(input: GovernanceInput) -> GovernanceDecision {
    let mut blockers = Vec::new();
    let mut unknowns = Vec::new();
    let mut safe_actions = Vec::new();
    let mut required_checks = Vec::new();

    for finding in &input.explicit_blockers {
        blockers.push(finding.clone());
        match finding.as_str() {
            "scope_exceeded" => {
                safe_actions.push("stop_and_request_new_contract".into());
                required_checks.push("scope".into());
            }
            "destructive_change_without_authority" => {
                safe_actions.push("stop_and_request_human_authority".into());
                required_checks.push("authority".into());
            }
            "unsafe_deletion_request" => {
                safe_actions.push("stop_and_request_human_authority".into());
                required_checks.push("destructive_operation".into());
            }
            "unsupported_completion_claim" => {
                safe_actions.push("remove_claim_or_provide_evidence".into());
                required_checks.push("completion_evidence".into());
            }
            "human_authority_missing" => {
                safe_actions.push("request_human_decision".into());
                required_checks.push("authority".into());
            }
            "contract_intent_missing"
            | "contract_goal_missing"
            | "contract_scope_missing"
            | "contract_out_of_scope_missing"
            | "contract_acceptance_missing" => {
                safe_actions.push("complete_contract".into());
                safe_actions.push("request_human_decision".into());
                required_checks.push("contract_review".into());
            }
            "archive_invalid" => {
                safe_actions.push("preserve_active_work_item".into());
                safe_actions.push("repair_archive_evidence".into());
                required_checks.push("archive_integrity".into());
            }
            "stale_contract" => {
                safe_actions.push("stop_and_refresh_contract".into());
                required_checks.push("contract_freshness".into());
            }
            "cross_work_item_evidence" => {
                safe_actions.push("rerun_evidence_for_current_work_item".into());
                required_checks.push("evidence_binding".into());
            }
            "test_weakening" => {
                safe_actions.push("restore_verification_strength".into());
                safe_actions.push("request_human_decision".into());
                required_checks.push("test_integrity".into());
            }
            "coverage_weakening" => {
                safe_actions.push("restore_coverage_requirement".into());
                safe_actions.push("request_human_decision".into());
                required_checks.push("coverage_integrity".into());
            }
            "repository_material_inspection_unavailable" => {
                safe_actions.push("inspect_repository_material".into());
                safe_actions.push("rerun_preflight".into());
                required_checks.push("input_trust".into());
            }
            "test_weakening_inspection_unavailable" => {
                safe_actions.push("inspect_test_change".into());
                safe_actions.push("rerun_preflight".into());
                required_checks.push("test_integrity".into());
            }
            "coverage_weakening_inspection_unavailable" => {
                safe_actions.push("inspect_coverage_change".into());
                safe_actions.push("rerun_preflight".into());
                required_checks.push("coverage_integrity".into());
            }
            "evidence_contradictory" => {
                safe_actions.push("stop_and_reconcile_evidence".into());
                required_checks.push("evidence_consistency".into());
            }
            _ => safe_actions.push("stop_and_request_human_decision".into()),
        }
    }
    unknowns.extend(input.explicit_unknowns.iter().cloned());
    for unknown in &input.explicit_unknowns {
        match unknown.as_str() {
            "required_evidence_missing" => {
                safe_actions.push("collect_required_evidence".into());
                safe_actions.push("rerun_preflight".into());
                required_checks.push("verification".into());
            }
            "evidence_stale" => {
                safe_actions.push("rerun_affected_checks".into());
                safe_actions.push("rerun_preflight".into());
                required_checks.push("evidence_freshness".into());
            }
            "repository_material_untrusted" => {
                safe_actions.push("treat_material_as_data".into());
                safe_actions.push("continue_with_explicit_policy".into());
                required_checks.push("input_trust".into());
            }
            "provider_result_unknown" => {
                safe_actions.push("obtain_provider_receipt".into());
                safe_actions.push("rerun_preflight".into());
                required_checks.push("external_evidence".into());
            }
            "destructive_change_without_authority" => {
                safe_actions.push("stop_and_request_human_authority".into());
                required_checks.push("authority".into());
                required_checks.push("scope".into());
            }
            "human_authority_missing" => {
                safe_actions.push("request_human_decision".into());
                required_checks.push("authority".into());
            }
            "contract_intent_missing"
            | "contract_goal_missing"
            | "contract_scope_missing"
            | "contract_out_of_scope_missing"
            | "contract_acceptance_missing" => {
                safe_actions.push("complete_contract".into());
                safe_actions.push("request_human_decision".into());
                required_checks.push("contract_review".into());
            }
            "coverage_weakening" => {
                safe_actions.push("restore_coverage_requirement".into());
                safe_actions.push("request_human_decision".into());
                required_checks.push("coverage_integrity".into());
            }
            _ => safe_actions.push("collect_missing_evidence".into()),
        }
    }

    if input.changed_paths.iter().any(|path| {
        !input
            .scope
            .iter()
            .any(|pattern| matches_pattern(path, pattern))
    }) {
        blockers.push("scope_exceeded".into());
        safe_actions.push("stop_and_request_new_contract".into());
        required_checks.push("scope".into());
    }
    if input.changed_paths.iter().any(|path| {
        input
            .out_of_scope
            .iter()
            .any(|pattern| matches_pattern(path, pattern))
    }) {
        blockers.push("out_of_scope_changed".into());
        safe_actions.push("restore_out_of_scope_boundary".into());
        required_checks.push("scope".into());
    }
    if input.action == ActionKind::Destructive && input.authority != AuthorityState::Authorized {
        unknowns.push("destructive_change_without_authority".into());
        safe_actions.push("stop_and_request_human_authority".into());
        required_checks.push("authority".into());
        required_checks.push("scope".into());
    }
    if input.test_weakening {
        blockers.push("test_weakening".into());
        safe_actions.push("restore_verification_strength".into());
        required_checks.push("test_integrity".into());
    }
    if input.coverage_weakening {
        unknowns.push("coverage_weakening".into());
        safe_actions.push("restore_coverage_requirement".into());
        safe_actions.push("request_human_decision".into());
        required_checks.push("coverage_integrity".into());
    }

    match input.evidence {
        EvidenceState::Contradictory => {
            blockers.push("evidence_contradictory".into());
            safe_actions.push("stop_and_reconcile_evidence".into());
            required_checks.push("evidence_consistency".into());
        }
        EvidenceState::Missing => {
            unknowns.push("required_evidence_missing".into());
            safe_actions.push("collect_required_evidence".into());
            required_checks.push("verification".into());
        }
        EvidenceState::Stale => {
            unknowns.push("evidence_stale".into());
            safe_actions.push("rerun_affected_checks".into());
            required_checks.push("evidence_freshness".into());
        }
        EvidenceState::Unknown => {
            unknowns.push("evidence_unknown".into());
            safe_actions.push("rerun_affected_checks".into());
            required_checks.push("verification".into());
        }
        EvidenceState::Complete => {}
    }
    if input.untrusted_material {
        unknowns.push("repository_material_untrusted".into());
        safe_actions.push("treat_material_as_data".into());
        required_checks.push("input_trust".into());
    }

    blockers.sort();
    blockers.dedup();
    unknowns.sort();
    unknowns.dedup();
    safe_actions.sort();
    safe_actions.dedup();
    required_checks.sort();
    required_checks.dedup();

    let state = if !blockers.is_empty() {
        DecisionState::Red
    } else if !unknowns.is_empty() {
        DecisionState::Yellow
    } else {
        DecisionState::Green
    };
    let outcome_state = input.outcome_state_override.unwrap_or_else(|| match state {
        DecisionState::Green => "ready".into(),
        DecisionState::Yellow
            if unknowns
                .iter()
                .any(|unknown| requires_human_confirmation_unknown(unknown)) =>
        {
            "needs_human_decision".into()
        }
        DecisionState::Yellow => "verification_pending".into(),
        DecisionState::Red => "blocked".into(),
    });
    let authority = input.authority_override.unwrap_or_else(|| {
        match input.authority {
            AuthorityState::Authorized => "authorized",
            AuthorityState::Missing => "missing",
            AuthorityState::NotEvaluated => "not_evaluated",
        }
        .into()
    });
    let review_state = if state == DecisionState::Red {
        Some("blocked".into())
    } else if unknowns
        .iter()
        .any(|unknown| requires_human_confirmation_unknown(unknown))
    {
        Some("needs_human_confirmation".into())
    } else if state == DecisionState::Yellow {
        Some("verification_pending".into())
    } else {
        Some("ready".into())
    };
    let human_decision_request = (review_state.as_deref() == Some("needs_human_confirmation"))
        .then(|| HumanDecisionRequest {
            decision_id: "contract-preflight-review".into(),
            status: "needs_human_confirmation".into(),
            what_happened: "The current Contract contains unresolved governance facts.".into(),
            why_it_matters:
                "An Agent must not turn uncertainty about intent, authority, scope, or policy into an implementation decision.".into(),
            options: vec![
                HumanDecisionOption {
                    id: "complete_contract".into(),
                    label: "Complete or amend the Contract".into(),
                    effect: "Provide the missing human-owned facts, then rerun preflight.".into(),
                },
                HumanDecisionOption {
                    id: "confirm_review".into(),
                    label: "Confirm a bounded human decision".into(),
                    effect: "Bind an identity-matched review receipt before checkpoint.".into(),
                },
                HumanDecisionOption {
                    id: "stop_work".into(),
                    label: "Stop the Work Item".into(),
                    effect: "Leave the item recoverable without entering implementation.".into(),
                },
            ],
            recommended_option: "complete_contract".into(),
            recommendation_reason: "The unresolved fields are human-owned and cannot be inferred from repository facts.".into(),
            question: "Which bounded decision should authorize the next step?".into(),
            resume_condition: "A repository-bound human review receipt matches repository, Work Item, Contract, and snapshot digests.".into(),
        });
    GovernanceDecision {
        state,
        blockers,
        unknowns,
        safe_actions,
        required_checks,
        authority,
        outcome_state,
        review_state,
        human_decision_request,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Digest(String);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DigestError {
    #[error("digest must be sha256:<64 lowercase hexadecimal characters>")]
    InvalidFormat,
}

impl Digest {
    pub fn sha256_bytes(bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        Self(format!("sha256:{}", hex::encode(hasher.finalize())))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Digest {
    type Err = DigestError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let Some(hex_part) = value.strip_prefix("sha256:") else {
            return Err(DigestError::InvalidFormat);
        };
        if hex_part.len() != 64
            || !hex_part
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(DigestError::InvalidFormat);
        }
        Ok(Self(value.into()))
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl TryFrom<String> for Digest {
    type Error = DigestError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

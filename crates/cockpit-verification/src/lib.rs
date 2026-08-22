use cockpit_core::Digest;
use cockpit_evidence::{
    EvidenceContext, ReusableReceipt, ReuseAction, ReuseReason, ReuseState, decide_reuse,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub const MAX_CAPTURE_BYTES_PER_STREAM: usize = 64 * 1024;
pub const REUSABLE_RECEIPT_TTL_SECONDS: i64 = 24 * 60 * 60;
pub const MAX_EXECUTION_SECONDS: u64 = 300;

/// A machine-readable measurement captured by a repository-local performance
/// fixture.  The identity fields are intentionally required: a timing value
/// without the runtime and repository that produced it is not release
/// evidence and must not be used by a gate.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PerformanceSample {
    pub name: String,
    pub elapsed_ms: u128,
    pub iterations: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PerformanceBudget {
    pub name: String,
    pub max_elapsed_ms: u128,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PerformanceBaseline {
    pub schema_version: u32,
    pub runtime_version: String,
    pub runtime_digest: String,
    pub repository_id: String,
    pub captured_at: String,
    pub samples: Vec<PerformanceSample>,
    pub budgets: Vec<PerformanceBudget>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PerformanceAssessment {
    pub state: String,
    pub measured: usize,
    pub passed: usize,
    pub failures: Vec<String>,
}

impl PerformanceBaseline {
    pub fn assess(&self) -> PerformanceAssessment {
        let mut failures = Vec::new();
        if self.schema_version != 1 {
            failures.push("unsupported_baseline_schema".into());
        }
        if self.runtime_version.trim().is_empty()
            || !valid_hex_digest(&self.runtime_digest)
            || !valid_hex_digest(&self.repository_id)
            || self.captured_at.trim().is_empty()
        {
            failures.push("runtime_or_repository_identity_missing".into());
        }
        let mut sample_names = BTreeSet::new();
        for sample in &self.samples {
            if !sample_names.insert(sample.name.as_str()) {
                failures.push(format!("duplicate_sample:{}", sample.name));
            }
        }
        let mut budget_names = BTreeSet::new();
        for budget in &self.budgets {
            if !budget_names.insert(budget.name.as_str()) {
                failures.push(format!("duplicate_budget:{}", budget.name));
            }
        }
        let mut measured = 0;
        let mut passed = 0;
        for budget in &self.budgets {
            let Some(sample) = self
                .samples
                .iter()
                .find(|sample| sample.name == budget.name)
            else {
                failures.push(format!("sample_missing:{}", budget.name));
                continue;
            };
            measured += 1;
            if sample.iterations == 0 {
                failures.push(format!("iterations_zero:{}", budget.name));
            } else if sample.elapsed_ms <= budget.max_elapsed_ms {
                passed += 1;
            } else {
                failures.push(format!(
                    "budget_exceeded:{}:{}>{}",
                    budget.name, sample.elapsed_ms, budget.max_elapsed_ms
                ));
            }
        }
        let state = if failures.is_empty() && measured == self.budgets.len() {
            "passed"
        } else {
            "failed"
        };
        PerformanceAssessment {
            state: state.into(),
            measured,
            passed,
            failures,
        }
    }
}

fn valid_hex_digest(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerificationNodeKind {
    Protected,
    Reusable,
    External,
    ProjectCommand,
    Governance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationNode {
    pub id: String,
    pub kind: VerificationNodeKind,
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationPlan {
    pub node_ids: Vec<String>,
    pub max_workers: usize,
}

/// Input to the policy-driven verification planner.  The planner consumes
/// explicit policy layers; it never invents a T3 requirement from an
/// operation name or silently treats T3 as provider/enterprise assurance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyPlannerInput {
    pub operation: String,
    pub stage: String,
    pub protected_gate: Option<String>,
    pub policies: Vec<cockpit_protocol::GovernancePolicy>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PolicyVerificationPlan {
    pub schema_version: u32,
    pub operation: String,
    pub stage: String,
    pub requirement: cockpit_protocol::VerificationRequirement,
    pub source_policy_ids: Vec<String>,
    pub escalation_reasons: Vec<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PolicyPlannerError {
    #[error("policy planner requires at least one ordered policy layer")]
    NoPolicies,
    #[error("policy layer {0} has an empty policy id")]
    EmptyPolicyId(String),
    #[error("policy layer {0} is missing a rule for operation {1}")]
    OperationRuleMissing(String, String),
    #[error("policy {0} has no verification requirement for operation {1}")]
    VerificationRequirementMissing(String, String),
    #[error("policy {0} verification requirement is not traceable to its policy id")]
    PolicyReferenceMissing(String),
    #[error("policy {0} verification requirement does not reference stage {1}")]
    StageReferenceMissing(String, String),
    #[error("policy {0} verification requirement does not reference protected gate {1}")]
    GateReferenceMissing(String, String),
    #[error("invalid verification requirement in policy {0}: {1}")]
    InvalidRequirement(String, String),
    #[error("policy merge failed: {0}")]
    PolicyMerge(String),
}

pub const POLICY_PLANNER_SCHEMA_VERSION: u32 = 1;

/// Resolve the required verification truth from explicit policy/stage/gate
/// inputs.  Missing policy input or missing traceability is an error rather
/// than a default T3 (or a green plan).  Higher tiers and assurance levels
/// are merged independently by the protocol policy overlay rules.
pub fn plan_policy_requirement(
    input: &PolicyPlannerInput,
) -> Result<PolicyVerificationPlan, PolicyPlannerError> {
    if input.policies.is_empty() {
        return Err(PolicyPlannerError::NoPolicies);
    }
    let policy_refs = input.policies.iter().collect::<Vec<_>>();
    for policy in &input.policies {
        if policy.policy_id.trim().is_empty() {
            return Err(PolicyPlannerError::EmptyPolicyId(format!(
                "{:?}",
                policy.layer
            )));
        }
        let rule = policy
            .rules
            .iter()
            .find(|rule| rule.operation == input.operation)
            .ok_or_else(|| {
                PolicyPlannerError::OperationRuleMissing(
                    policy.policy_id.clone(),
                    input.operation.clone(),
                )
            })?;
        let requirement = rule.verification_requirement.as_ref().ok_or_else(|| {
            PolicyPlannerError::VerificationRequirementMissing(
                policy.policy_id.clone(),
                input.operation.clone(),
            )
        })?;
        requirement.validate().map_err(|error| {
            PolicyPlannerError::InvalidRequirement(policy.policy_id.clone(), error)
        })?;
        if !requirement.policy_refs.iter().any(|reference| {
            reference == &policy.policy_id || reference == &format!("policy:{}", policy.policy_id)
        }) {
            return Err(PolicyPlannerError::PolicyReferenceMissing(
                policy.policy_id.clone(),
            ));
        }
        if !requirement
            .stage_refs
            .iter()
            .any(|reference| reference == &input.stage)
        {
            return Err(PolicyPlannerError::StageReferenceMissing(
                policy.policy_id.clone(),
                input.stage.clone(),
            ));
        }
        if let Some(gate) = &input.protected_gate
            && !requirement
                .gate_refs
                .iter()
                .any(|reference| reference == gate)
        {
            return Err(PolicyPlannerError::GateReferenceMissing(
                policy.policy_id.clone(),
                gate.clone(),
            ));
        }
    }
    let effective = cockpit_protocol::merge_policy_layers(&policy_refs)
        .map_err(|error| PolicyPlannerError::PolicyMerge(error.to_string()))?;
    let requirement = effective
        .rules
        .iter()
        .find(|rule| rule.operation == input.operation)
        .and_then(|rule| rule.verification_requirement.clone())
        .ok_or_else(|| {
            PolicyPlannerError::VerificationRequirementMissing(
                effective.policy_id.clone(),
                input.operation.clone(),
            )
        })?;
    let escalation_reasons = input
        .policies
        .iter()
        .filter_map(|policy| {
            policy
                .rules
                .iter()
                .find(|rule| rule.operation == input.operation)
                .and_then(|rule| rule.verification_requirement.as_ref())
                .map(|requirement| format!("{}: {}", policy.policy_id, requirement.reason))
        })
        .collect();
    Ok(PolicyVerificationPlan {
        schema_version: POLICY_PLANNER_SCHEMA_VERSION,
        operation: input.operation.clone(),
        stage: input.stage.clone(),
        requirement,
        source_policy_ids: input
            .policies
            .iter()
            .map(|policy| policy.policy_id.clone())
            .collect(),
        escalation_reasons,
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VerificationResult {
    pub node_id: String,
    pub passed: bool,
    pub reused: bool,
    pub protected: bool,
    pub action: PlannedAction,
    pub state: PlannedState,
    pub reason: String,
    pub receipt_id: Option<String>,
    pub output_digest: Option<String>,
    pub output_truncated: bool,
    pub timed_out: bool,
    pub satisfied_by: PlannedSatisfaction,
}

impl VerificationNode {
    pub fn new(id: &str, kind: VerificationNodeKind, dependencies: Vec<String>) -> Self {
        Self {
            id: id.into(),
            kind,
            dependencies,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct VerificationGraph {
    nodes: BTreeMap<String, VerificationNode>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GraphError {
    #[error("duplicate verification node {0}")]
    Duplicate(String),
    #[error("unknown verification dependency {0}")]
    UnknownDependency(String),
    #[error("verification graph contains a cycle")]
    Cycle,
}

impl VerificationGraph {
    pub fn add(&mut self, node: VerificationNode) -> Result<(), GraphError> {
        if self.nodes.contains_key(&node.id) {
            return Err(GraphError::Duplicate(node.id));
        }
        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }

    pub fn plan(&self) -> Result<Vec<String>, GraphError> {
        let mut indegree = self
            .nodes
            .iter()
            .map(|(id, node)| (id.clone(), node.dependencies.len()))
            .collect::<BTreeMap<_, _>>();
        let mut reverse: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for node in self.nodes.values() {
            for dependency in &node.dependencies {
                if !self.nodes.contains_key(dependency) {
                    return Err(GraphError::UnknownDependency(dependency.clone()));
                }
                reverse
                    .entry(dependency.clone())
                    .or_default()
                    .push(node.id.clone());
            }
        }
        let mut queue = indegree
            .iter()
            .filter_map(|(id, count)| (*count == 0).then_some(id.clone()))
            .collect::<VecDeque<_>>();
        let mut order = Vec::new();
        while let Some(id) = queue.pop_front() {
            order.push(id.clone());
            if let Some(dependents) = reverse.get(&id) {
                for dependent in dependents {
                    let count = indegree.get_mut(dependent).expect("dependent exists");
                    *count -= 1;
                    if *count == 0 {
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }
        if order.len() != self.nodes.len() {
            return Err(GraphError::Cycle);
        }
        Ok(order)
    }

    pub fn protected_ids(&self) -> BTreeSet<String> {
        self.nodes
            .values()
            .filter(|node| node.kind == VerificationNodeKind::Protected)
            .map(|node| node.id.clone())
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReuseCandidate {
    pub receipt: Option<ReusableReceipt>,
    pub current_context: EvidenceContext,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedGateClass {
    Security,
    Scope,
    Governance,
    Coverage,
    SourceBound,
    ProjectCommand,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationReusePolicy {
    Protected(ProtectedGateClass),
    Reusable,
    NeverReuse,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationCommand {
    pub id: String,
    pub program: String,
    pub args: Vec<String>,
    pub current_dir: Option<PathBuf>,
    pub dependencies: Vec<String>,
    reuse_policy: VerificationReusePolicy,
    reuse_candidate: Option<ReuseCandidate>,
    logical_identity: Option<(String, Vec<String>)>,
    environment: Vec<(std::ffi::OsString, std::ffi::OsString)>,
    resource_weight: usize,
}

impl VerificationCommand {
    pub fn new(
        id: &str,
        program: &str,
        args: Vec<String>,
        reuse_policy: VerificationReusePolicy,
    ) -> Self {
        Self {
            id: id.into(),
            program: program.into(),
            args,
            current_dir: None,
            dependencies: Vec::new(),
            reuse_policy,
            reuse_candidate: None,
            logical_identity: None,
            environment: Vec::new(),
            resource_weight: 1,
        }
    }

    pub fn new_pinned(
        id: &str,
        execution_program: &str,
        execution_args: Vec<String>,
        logical_program: &str,
        logical_args: Vec<String>,
        reuse_policy: VerificationReusePolicy,
    ) -> Self {
        let mut command = Self::new(id, execution_program, execution_args, reuse_policy);
        command.logical_identity = Some((logical_program.into(), logical_args));
        command
    }

    pub fn with_reuse_candidate(
        mut self,
        receipt: Option<ReusableReceipt>,
        current_context: EvidenceContext,
    ) -> Self {
        self.reuse_candidate = Some(ReuseCandidate {
            receipt,
            current_context,
        });
        self
    }

    pub fn with_current_dir(mut self, current_dir: impl Into<PathBuf>) -> Self {
        self.current_dir = Some(current_dir.into());
        self
    }

    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }

    pub fn with_environment(
        mut self,
        environment: Vec<(std::ffi::OsString, std::ffi::OsString)>,
    ) -> Self {
        self.environment = environment;
        self
    }

    /// Assign a resource weight used by the bounded scheduler. Zero is
    /// rejected at execution time so malformed plans fail closed.
    pub fn with_resource_weight(mut self, weight: usize) -> Self {
        self.resource_weight = weight;
        self
    }

    pub fn resource_weight(&self) -> usize {
        self.resource_weight
    }

    pub fn command_digest(&self) -> String {
        let current_dir = self
            .current_dir
            .as_ref()
            .map(|path| path.as_os_str().as_encoded_bytes());
        let (program, args) = self
            .logical_identity
            .as_ref()
            .map_or((&self.program, &self.args), |(program, args)| {
                (program, args)
            });
        let identity = serde_json::to_vec(&(program, args, current_dir, self.resource_weight))
            .expect("verification command identity is serializable");
        Digest::sha256_bytes(&identity).to_string()
    }

    pub fn reuse_policy(&self) -> &VerificationReusePolicy {
        &self.reuse_policy
    }

    fn is_protected(&self) -> bool {
        matches!(&self.reuse_policy, VerificationReusePolicy::Protected(_))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlannedAction {
    Reuse,
    Execute,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlannedState {
    Fresh,
    Stale,
    Unknown,
    NotApplicable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlannedReason {
    Evidence(ReuseReason),
    ReuseNotConfigured,
    DependencyRerunRequired,
}

impl PlannedReason {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Evidence(ReuseReason::FreshExactBinding) => "fresh_exact_binding",
            Self::Evidence(ReuseReason::ProtectedNode) => "protected_node",
            Self::Evidence(ReuseReason::EvidenceMissing) => "evidence_missing",
            Self::Evidence(ReuseReason::ReceiptInvalid) => "receipt_invalid",
            Self::Evidence(ReuseReason::ReceiptFailed) => "receipt_failed",
            Self::Evidence(ReuseReason::ReceiptFromFuture) => "receipt_from_future",
            Self::Evidence(ReuseReason::EvidenceExpired) => "evidence_expired",
            Self::Evidence(ReuseReason::BindingMismatch) => "binding_mismatch",
            Self::ReuseNotConfigured => "reuse_not_configured",
            Self::DependencyRerunRequired => "dependency_rerun_required",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlannedSatisfaction {
    Execution,
    ReusedReceipt,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannedVerificationCommand {
    pub command: VerificationCommand,
    pub action: PlannedAction,
    pub state: PlannedState,
    pub reason: PlannedReason,
    pub receipt_id: Option<String>,
    pub satisfied_by: PlannedSatisfaction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationExecutionPlan {
    commands: Vec<PlannedVerificationCommand>,
    planning_elapsed_ms: u128,
}

impl VerificationExecutionPlan {
    pub fn commands(&self) -> &[PlannedVerificationCommand] {
        &self.commands
    }

    pub fn planning_elapsed_ms(&self) -> u128 {
        self.planning_elapsed_ms
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VerificationReceipt {
    /// These fields are omitted from an in-memory execution receipt and are
    /// populated only when the receipt is persisted as Work Item evidence.
    /// Persisted evidence validation requires all four fields, so a raw
    /// execution result cannot be mistaken for repository-bound evidence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_item_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_digest: Option<String>,
    pub results: Vec<VerificationResult>,
    pub receipt_candidates: Vec<ReusableReceipt>,
    pub nodes_planned: usize,
    pub nodes_executed: usize,
    pub nodes_reused: usize,
    pub rerun_stale: usize,
    pub rerun_unknown: usize,
    pub protected_nodes_executed: usize,
    pub protected_nodes_skipped: usize,
    pub planning_elapsed_ms: u128,
    pub execution_elapsed_ms: u128,
    pub processes_spawned: usize,
    pub process_spawn_failures: usize,
    pub git_calls: usize,
    pub files_read: usize,
    pub files_hashed: usize,
    pub elapsed_ms: u128,
    pub passed: bool,
}

/// Identity used to coalesce concurrent verification requests. Every field
/// is explicit so a request from another repository, Work Item, runtime, or
/// command can never observe a different request's result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SingleFlightKey {
    pub repository_id: String,
    pub work_item_id: String,
    pub command_digest: String,
    pub runtime_digest: String,
}

impl SingleFlightKey {
    fn map_key(&self) -> String {
        serde_json::to_string(self).expect("single-flight key is serializable")
    }

    fn is_valid(&self) -> bool {
        !self.repository_id.trim().is_empty()
            && !self.work_item_id.trim().is_empty()
            && self.command_digest.starts_with("sha256:")
            && self.runtime_digest.starts_with("sha256:")
    }
}

struct SingleFlightState {
    result: Mutex<Option<Result<Arc<VerificationReceipt>, String>>>,
    ready: Condvar,
}

/// In-process single-flight coordinator. It is deliberately an ephemeral
/// optimization: no result is persisted or treated as evidence by this type;
/// callers still persist and validate the returned receipt through the normal
/// repository evidence path.
#[derive(Default)]
pub struct SingleFlightCoordinator {
    flights: Mutex<BTreeMap<String, Arc<SingleFlightState>>>,
}

impl SingleFlightCoordinator {
    pub fn execute<F>(
        &self,
        key: SingleFlightKey,
        operation: F,
    ) -> Result<Arc<VerificationReceipt>, String>
    where
        F: FnOnce() -> Result<VerificationReceipt, ExecutionError>,
    {
        if !key.is_valid() {
            return Err("single_flight_key_invalid".into());
        }
        let map_key = key.map_key();
        let (state, leader) = {
            let mut flights = self
                .flights
                .lock()
                .map_err(|_| "single_flight_registry_poisoned".to_string())?;
            if let Some(state) = flights.get(&map_key) {
                (Arc::clone(state), false)
            } else {
                let state = Arc::new(SingleFlightState {
                    result: Mutex::new(None),
                    ready: Condvar::new(),
                });
                flights.insert(map_key.clone(), Arc::clone(&state));
                (state, true)
            }
        };

        if !leader {
            let mut result = state
                .result
                .lock()
                .map_err(|_| "single_flight_result_poisoned".to_string())?;
            while result.is_none() {
                result = state
                    .ready
                    .wait(result)
                    .map_err(|_| "single_flight_result_poisoned".to_string())?;
            }
            return result
                .as_ref()
                .expect("single-flight result initialized")
                .clone();
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation))
            .map_err(|_| "single_flight_operation_panicked".to_string())
            .and_then(|result| result.map(Arc::new).map_err(|error| error.to_string()));
        {
            let mut stored = state
                .result
                .lock()
                .map_err(|_| "single_flight_result_poisoned".to_string())?;
            *stored = Some(result.clone());
            state.ready.notify_all();
        }
        if let Ok(mut flights) = self.flights.lock()
            && flights
                .get(&map_key)
                .is_some_and(|current| Arc::ptr_eq(current, &state))
        {
            flights.remove(&map_key);
        }
        result
    }

    pub fn active_count(&self) -> Result<usize, String> {
        self.flights
            .lock()
            .map(|flights| flights.len())
            .map_err(|_| "single_flight_registry_poisoned".into())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ExecutionError {
    #[error("worker count must be greater than zero")]
    InvalidWorkerCount,
    #[error("resource budget must be greater than zero")]
    InvalidResourceBudget,
    #[error("verification command {0} exceeds the resource budget")]
    CommandExceedsResourceBudget(String),
    #[error("verification worker mutex was poisoned")]
    WorkerPoisoned,
    #[error(transparent)]
    InvalidGraph(#[from] GraphError),
}

pub fn plan_verification_commands(
    commands: Vec<VerificationCommand>,
    now_epoch_seconds: i64,
) -> Result<VerificationExecutionPlan, ExecutionError> {
    let started = Instant::now();
    let mut graph = VerificationGraph::default();
    let mut by_id = BTreeMap::new();
    for command in commands {
        graph.add(VerificationNode::new(
            &command.id,
            if command.is_protected() {
                VerificationNodeKind::Protected
            } else if command.reuse_policy == VerificationReusePolicy::Reusable {
                VerificationNodeKind::Reusable
            } else {
                VerificationNodeKind::ProjectCommand
            },
            command.dependencies.clone(),
        ))?;
        by_id.insert(command.id.clone(), command);
    }

    let order = graph.plan()?;
    let mut actions = BTreeMap::new();
    let mut planned = Vec::with_capacity(order.len());
    for id in order {
        let command = by_id.remove(&id).expect("planned command exists");
        let (mut action, mut state, mut reason, receipt_id) =
            classify_command(&command, now_epoch_seconds);

        if action == PlannedAction::Reuse
            && command
                .dependencies
                .iter()
                .any(|dependency| actions.get(dependency) == Some(&PlannedAction::Execute))
        {
            action = PlannedAction::Execute;
            state = PlannedState::Stale;
            reason = PlannedReason::DependencyRerunRequired;
        }
        actions.insert(id, action.clone());
        planned.push(PlannedVerificationCommand {
            command,
            satisfied_by: if action == PlannedAction::Reuse {
                PlannedSatisfaction::ReusedReceipt
            } else {
                PlannedSatisfaction::Execution
            },
            action,
            state,
            reason,
            receipt_id,
        });
    }
    Ok(VerificationExecutionPlan {
        commands: planned,
        planning_elapsed_ms: started.elapsed().as_millis(),
    })
}

pub fn execute_bounded(
    commands: Vec<VerificationCommand>,
    max_workers: usize,
) -> Result<VerificationReceipt, ExecutionError> {
    let now_epoch_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0);
    execute_bounded_at(commands, max_workers, now_epoch_seconds)
}

pub fn execute_bounded_at(
    commands: Vec<VerificationCommand>,
    max_workers: usize,
    now_epoch_seconds: i64,
) -> Result<VerificationReceipt, ExecutionError> {
    if max_workers == 0 {
        return Err(ExecutionError::InvalidWorkerCount);
    }
    let plan = plan_verification_commands(commands, now_epoch_seconds)?;
    execute_verification_plan_bounded_with_budget_at(
        plan,
        max_workers,
        max_workers,
        now_epoch_seconds,
    )
}

/// Execute with independent worker and resource limits. Resource units are
/// reserved before a process starts and released only after it completes;
/// dependency readiness and protected-node semantics remain unchanged.
pub fn execute_bounded_with_resource_budget(
    commands: Vec<VerificationCommand>,
    max_workers: usize,
    max_resource_units: usize,
) -> Result<VerificationReceipt, ExecutionError> {
    let now_epoch_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0);
    execute_bounded_with_resource_budget_at(
        commands,
        max_workers,
        max_resource_units,
        now_epoch_seconds,
    )
}

pub fn execute_bounded_with_resource_budget_at(
    commands: Vec<VerificationCommand>,
    max_workers: usize,
    max_resource_units: usize,
    now_epoch_seconds: i64,
) -> Result<VerificationReceipt, ExecutionError> {
    if max_workers == 0 {
        return Err(ExecutionError::InvalidWorkerCount);
    }
    if max_resource_units == 0 {
        return Err(ExecutionError::InvalidResourceBudget);
    }
    for command in &commands {
        if command.resource_weight == 0 {
            return Err(ExecutionError::CommandExceedsResourceBudget(
                command.id.clone(),
            ));
        }
        if command.resource_weight > max_resource_units {
            return Err(ExecutionError::CommandExceedsResourceBudget(
                command.id.clone(),
            ));
        }
    }
    let plan = plan_verification_commands(commands, now_epoch_seconds)?;
    execute_verification_plan_bounded_with_budget_at(
        plan,
        max_workers,
        max_resource_units,
        now_epoch_seconds,
    )
}

pub fn execute_verification_plan_bounded(
    plan: VerificationExecutionPlan,
    max_workers: usize,
) -> Result<VerificationReceipt, ExecutionError> {
    let now_epoch_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0);
    execute_verification_plan_bounded_with_budget_at(
        plan,
        max_workers,
        max_workers,
        now_epoch_seconds,
    )
}

fn execute_verification_plan_bounded_with_budget_at(
    plan: VerificationExecutionPlan,
    max_workers: usize,
    max_resource_units: usize,
    now_epoch_seconds: i64,
) -> Result<VerificationReceipt, ExecutionError> {
    if max_workers == 0 {
        return Err(ExecutionError::InvalidWorkerCount);
    }
    if max_resource_units == 0 {
        return Err(ExecutionError::InvalidResourceBudget);
    }
    let started = Instant::now();
    let planning_elapsed_ms = plan.planning_elapsed_ms;
    let planned_count = plan.commands.len();
    let result_plan = plan.commands.clone();
    let nodes_reused = plan
        .commands
        .iter()
        .filter(|entry| entry.action == PlannedAction::Reuse)
        .count();
    let rerun_stale = plan
        .commands
        .iter()
        .filter(|entry| {
            entry.action == PlannedAction::Execute && entry.state == PlannedState::Stale
        })
        .count();
    let rerun_unknown = plan
        .commands
        .iter()
        .filter(|entry| {
            entry.action == PlannedAction::Execute && entry.state == PlannedState::Unknown
        })
        .count();
    let protected_nodes_skipped = plan
        .commands
        .iter()
        .filter(|entry| entry.command.is_protected() && entry.action == PlannedAction::Reuse)
        .count();

    let commands = plan
        .commands
        .into_iter()
        .filter_map(|entry| (entry.action == PlannedAction::Execute).then_some(entry.command))
        .collect::<Vec<_>>();
    for command in &commands {
        if command.resource_weight == 0 || command.resource_weight > max_resource_units {
            return Err(ExecutionError::CommandExceedsResourceBudget(
                command.id.clone(),
            ));
        }
    }
    let worker_count = max_workers.min(commands.len().max(1));
    let scheduler = Arc::new((
        Mutex::new(SchedulerState::new(commands, max_resource_units)),
        Condvar::new(),
    ));
    let mut workers = Vec::with_capacity(worker_count);
    for _ in 0..worker_count {
        let scheduler = Arc::clone(&scheduler);
        workers.push(std::thread::spawn(move || -> Result<(), ExecutionError> {
            loop {
                let command = {
                    let (lock, ready) = &*scheduler;
                    let mut state = lock.lock().map_err(|_| ExecutionError::WorkerPoisoned)?;
                    loop {
                        if let Some(command) = state.take_ready() {
                            break Some(command);
                        }
                        if state.completed == state.total {
                            break None;
                        }
                        state = ready
                            .wait(state)
                            .map_err(|_| ExecutionError::WorkerPoisoned)?;
                    }
                };
                let Some(command) = command else {
                    return Ok(());
                };
                let outcome = execute_captured(&command);
                let (lock, ready) = &*scheduler;
                let mut state = lock.lock().map_err(|_| ExecutionError::WorkerPoisoned)?;
                state.complete(
                    &command.id,
                    command.is_protected(),
                    command.resource_weight,
                    outcome,
                );
                ready.notify_all();
            }
        }));
    }
    for worker in workers {
        worker
            .join()
            .map_err(|_| ExecutionError::WorkerPoisoned)??;
    }
    let metrics = scheduler
        .0
        .lock()
        .map_err(|_| ExecutionError::WorkerPoisoned)?
        .metrics
        .clone();
    let execution_elapsed_ms = started.elapsed().as_millis();
    let mut results = Vec::with_capacity(result_plan.len());
    let mut receipt_candidates = Vec::new();
    for entry in result_plan {
        let reused = entry.action == PlannedAction::Reuse;
        let outcome = metrics.outcomes.get(&entry.command.id);
        let mut receipt_id = reused.then_some(entry.receipt_id).flatten();
        if !reused
            && entry.command.reuse_policy == VerificationReusePolicy::Reusable
            && outcome.is_some_and(|outcome| outcome.passed && !outcome.output_truncated)
            && let (Some(candidate), Some(output_digest)) = (
                entry.command.reuse_candidate.as_ref(),
                outcome.and_then(|outcome| outcome.output_digest.as_deref()),
            )
        {
            let mut context = candidate.current_context.clone();
            context.command_digest = entry.command.command_digest();
            if let Ok(receipt) = ReusableReceipt::new(
                &entry.command.id,
                true,
                context,
                output_digest,
                now_epoch_seconds,
                now_epoch_seconds.saturating_add(REUSABLE_RECEIPT_TTL_SECONDS),
            ) {
                receipt_id = Some(receipt.receipt_id.clone());
                receipt_candidates.push(receipt);
            }
        }
        results.push(VerificationResult {
            node_id: entry.command.id.clone(),
            passed: reused || outcome.is_some_and(|outcome| outcome.passed),
            reused,
            protected: entry.command.is_protected(),
            action: entry.action,
            state: entry.state,
            reason: entry.reason.code().into(),
            receipt_id,
            output_digest: outcome.and_then(|outcome| outcome.output_digest.clone()),
            output_truncated: outcome.is_some_and(|outcome| outcome.output_truncated),
            timed_out: outcome.is_some_and(|outcome| outcome.timed_out),
            satisfied_by: entry.satisfied_by,
        });
    }
    Ok(VerificationReceipt {
        work_item_id: None,
        repository_id: None,
        runtime_version: None,
        runtime_digest: None,
        results,
        receipt_candidates,
        nodes_planned: planned_count,
        nodes_executed: metrics.nodes_executed,
        nodes_reused,
        rerun_stale,
        rerun_unknown,
        protected_nodes_executed: metrics.protected_nodes_executed,
        protected_nodes_skipped,
        planning_elapsed_ms,
        execution_elapsed_ms,
        processes_spawned: metrics.processes_spawned,
        process_spawn_failures: metrics.process_spawn_failures,
        git_calls: 0,
        files_read: 0,
        files_hashed: 0,
        elapsed_ms: planning_elapsed_ms.saturating_add(execution_elapsed_ms),
        passed: metrics.passed && protected_nodes_skipped == 0,
    })
}

fn classify_command(
    command: &VerificationCommand,
    now_epoch_seconds: i64,
) -> (PlannedAction, PlannedState, PlannedReason, Option<String>) {
    if matches!(&command.reuse_policy, VerificationReusePolicy::Protected(_)) {
        return (
            PlannedAction::Execute,
            PlannedState::Unknown,
            PlannedReason::Evidence(ReuseReason::ProtectedNode),
            None,
        );
    }
    if command.reuse_policy == VerificationReusePolicy::NeverReuse {
        return (
            PlannedAction::Execute,
            PlannedState::NotApplicable,
            PlannedReason::ReuseNotConfigured,
            None,
        );
    }
    let Some(candidate) = &command.reuse_candidate else {
        return (
            PlannedAction::Execute,
            PlannedState::Unknown,
            PlannedReason::Evidence(ReuseReason::EvidenceMissing),
            None,
        );
    };
    let mut current_context = candidate.current_context.clone();
    current_context.command_digest = command.command_digest();
    let decision = decide_reuse(
        candidate.receipt.as_ref(),
        &current_context,
        &command.id,
        now_epoch_seconds,
        false,
    );
    let receipt_id = candidate
        .receipt
        .as_ref()
        .map(|receipt| receipt.receipt_id.clone());
    (
        match decision.action {
            ReuseAction::Reuse => PlannedAction::Reuse,
            ReuseAction::Execute => PlannedAction::Execute,
        },
        match decision.state {
            ReuseState::Fresh => PlannedState::Fresh,
            ReuseState::Stale => PlannedState::Stale,
            ReuseState::Unknown => PlannedState::Unknown,
        },
        PlannedReason::Evidence(decision.reason),
        receipt_id,
    )
}

#[derive(Clone, Debug)]
struct ExecutionOutcome {
    spawned: bool,
    passed: bool,
    output_digest: Option<String>,
    output_truncated: bool,
    timed_out: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OutputIdentity<'a> {
    success: bool,
    exit_code: Option<i32>,
    stdout: &'a [u8],
    stderr: &'a [u8],
    stdout_truncated: bool,
    stderr_truncated: bool,
    timed_out: bool,
}

struct StreamCapture {
    bytes: Vec<u8>,
    truncated: bool,
    failed: bool,
    timed_out: bool,
}

struct CaptureWorker {
    receiver: mpsc::Receiver<StreamCapture>,
    cancel: mpsc::Sender<()>,
}

fn execute_captured(command: &VerificationCommand) -> ExecutionOutcome {
    let mut process = Command::new(&command.program);
    process
        .args(&command.args)
        .envs(command.environment.iter().cloned())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(current_dir) = &command.current_dir {
        process.current_dir(current_dir);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        process.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use windows_sys::Win32::System::Threading::CREATE_SUSPENDED;
        process.creation_flags(CREATE_SUSPENDED);
    }
    let mut child = match process.spawn() {
        Ok(child) => child,
        Err(error) => {
            if std::env::var_os("AI_COCKPIT_DEBUG_SPAWN").is_some() {
                eprintln!("ai-cockpit spawn failed for {:?}: {error}", command.program);
            }
            return ExecutionOutcome {
                spawned: false,
                passed: false,
                output_digest: None,
                output_truncated: false,
                timed_out: false,
            };
        }
    };
    let child_id = child.id();
    #[cfg(windows)]
    let process_tree = match WindowsProcessTree::attach(&child) {
        Some(tree) => tree,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return ExecutionOutcome {
                spawned: true,
                passed: false,
                output_digest: None,
                output_truncated: false,
                timed_out: false,
            };
        }
    };
    #[cfg(windows)]
    if !resume_suspended_process(&child) {
        drop(process_tree);
        let _ = child.kill();
        let _ = child.wait();
        return ExecutionOutcome {
            spawned: true,
            passed: false,
            output_digest: None,
            output_truncated: false,
            timed_out: false,
        };
    }
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdout_worker = stdout.map(capture_stream_async);
    let stderr_worker = stderr.map(capture_stream_async);
    let deadline = Instant::now() + Duration::from_secs(MAX_EXECUTION_SECONDS);
    let (status, mut timed_out) = loop {
        match child.try_wait() {
            Ok(Some(status)) => break (Some(status), false),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            Ok(None) => {
                terminate_process_tree(&mut child, child_id);
                break (child.wait().ok(), true);
            }
            Err(_) => {
                terminate_process_tree(&mut child, child_id);
                break (None, false);
            }
        }
    };
    #[cfg(windows)]
    drop(process_tree);
    terminate_descendants(child_id);
    let stdout = receive_capture(stdout_worker);
    let stderr = receive_capture(stderr_worker);
    if stdout.timed_out || stderr.timed_out {
        timed_out = true;
    }
    let Some(status) = status else {
        return ExecutionOutcome {
            spawned: true,
            passed: false,
            output_digest: None,
            output_truncated: stdout.truncated || stderr.truncated,
            timed_out,
        };
    };
    if stdout.failed || stderr.failed {
        return ExecutionOutcome {
            spawned: true,
            passed: false,
            output_digest: None,
            output_truncated: stdout.truncated || stderr.truncated,
            timed_out,
        };
    }
    let identity = OutputIdentity {
        success: status.success(),
        exit_code: status.code(),
        stdout: &stdout.bytes,
        stderr: &stderr.bytes,
        stdout_truncated: stdout.truncated,
        stderr_truncated: stderr.truncated,
        timed_out,
    };
    let output_digest = serde_json::to_vec(&identity)
        .ok()
        .map(|bytes| Digest::sha256_bytes(&bytes).to_string());
    ExecutionOutcome {
        spawned: true,
        passed: status.success() && !timed_out,
        output_digest,
        output_truncated: stdout.truncated || stderr.truncated,
        timed_out,
    }
}

fn terminate_process_tree(child: &mut std::process::Child, child_id: u32) {
    terminate_descendants(child_id);
    let _ = child.kill();
}

#[cfg(unix)]
fn terminate_descendants(child_id: u32) {
    // SAFETY: kill is called with a negative, freshly spawned process-group id.
    unsafe {
        libc::kill(-(child_id as i32), libc::SIGKILL);
    }
}

#[cfg(not(unix))]
fn terminate_descendants(_child_id: u32) {}

#[cfg(windows)]
struct WindowsProcessTree(windows_sys::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl WindowsProcessTree {
    fn attach(child: &std::process::Child) -> Option<Self> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
            SetInformationJobObject,
        };

        let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if job.is_null() {
            return None;
        }
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let configured = unsafe {
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                (&raw const limits).cast(),
                std::mem::size_of_val(&limits) as u32,
            )
        };
        let assigned = configured != 0
            && unsafe { AssignProcessToJobObject(job, child.as_raw_handle().cast()) } != 0;
        if assigned {
            Some(Self(job))
        } else {
            unsafe {
                windows_sys::Win32::Foundation::CloseHandle(job);
            }
            None
        }
    }
}

#[cfg(windows)]
impl Drop for WindowsProcessTree {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.0);
        }
    }
}

#[cfg(windows)]
fn resume_suspended_process(child: &std::process::Child) -> bool {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, INVALID_HANDLE_VALUE},
        System::{
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First,
                Thread32Next,
            },
            Threading::{OpenThread, ResumeThread, THREAD_SUSPEND_RESUME},
        },
    };

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return false;
    }
    let mut entry = THREADENTRY32 {
        dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
        ..THREADENTRY32::default()
    };
    let mut found = false;
    let mut next = unsafe { Thread32First(snapshot, &raw mut entry) } != 0;
    while next {
        if entry.th32OwnerProcessID == child.id() {
            let thread = unsafe { OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID) };
            if !thread.is_null() {
                let resumed = unsafe { ResumeThread(thread) } != u32::MAX;
                unsafe { CloseHandle(thread) };
                if resumed {
                    found = true;
                    break;
                }
            }
        }
        next = unsafe { Thread32Next(snapshot, &raw mut entry) } != 0;
    }
    unsafe { CloseHandle(snapshot) };
    found
}

#[cfg(unix)]
fn capture_stream_async(
    mut stream: impl Read + std::os::fd::AsRawFd + Send + 'static,
) -> CaptureWorker {
    let (sender, receiver) = mpsc::sync_channel(1);
    let (cancel, cancelled) = mpsc::channel();
    std::thread::spawn(move || {
        let mut capture = StreamCapture {
            bytes: Vec::with_capacity(MAX_CAPTURE_BYTES_PER_STREAM),
            truncated: false,
            failed: false,
            timed_out: false,
        };
        let mut buffer = [0_u8; 8192];
        loop {
            if cancelled.try_recv().is_ok() {
                capture.failed = true;
                capture.truncated = true;
                capture.timed_out = true;
                let _ = sender.send(capture);
                return;
            }
            let mut descriptor = libc::pollfd {
                fd: stream.as_raw_fd(),
                events: libc::POLLIN | libc::POLLHUP,
                revents: 0,
            };
            let ready = unsafe { libc::poll(&raw mut descriptor, 1, 10) };
            if ready == 0 {
                continue;
            }
            if ready < 0 {
                if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                capture.failed = true;
                let _ = sender.send(capture);
                return;
            }
            if descriptor.revents & (libc::POLLERR | libc::POLLNVAL) != 0 {
                capture.failed = true;
                let _ = sender.send(capture);
                return;
            }
            match stream.read(&mut buffer) {
                Ok(0) => {
                    let _ = sender.send(capture);
                    return;
                }
                Ok(count) => {
                    let remaining =
                        MAX_CAPTURE_BYTES_PER_STREAM.saturating_sub(capture.bytes.len());
                    let retained = remaining.min(count);
                    capture.bytes.extend_from_slice(&buffer[..retained]);
                    if retained < count {
                        capture.truncated = true;
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(_) => {
                    capture.failed = true;
                    let _ = sender.send(capture);
                    return;
                }
            }
        }
    });
    CaptureWorker { receiver, cancel }
}

#[cfg(windows)]
fn capture_stream_async(
    mut stream: impl Read + std::os::windows::io::AsRawHandle + Send + 'static,
) -> CaptureWorker {
    use windows_sys::Win32::System::Pipes::PeekNamedPipe;

    let (sender, receiver) = mpsc::sync_channel(1);
    let (cancel, cancelled) = mpsc::channel();
    std::thread::spawn(move || {
        let mut capture = StreamCapture {
            bytes: Vec::with_capacity(MAX_CAPTURE_BYTES_PER_STREAM),
            truncated: false,
            failed: false,
            timed_out: false,
        };
        let mut buffer = [0_u8; 8192];
        loop {
            if cancelled.try_recv().is_ok() {
                capture.failed = true;
                capture.truncated = true;
                capture.timed_out = true;
                let _ = sender.send(capture);
                return;
            }
            let mut available = 0_u32;
            let peeked = unsafe {
                PeekNamedPipe(
                    stream.as_raw_handle().cast(),
                    std::ptr::null_mut(),
                    0,
                    std::ptr::null_mut(),
                    &raw mut available,
                    std::ptr::null_mut(),
                )
            };
            if peeked == 0 {
                let error = std::io::Error::last_os_error();
                if matches!(error.raw_os_error(), Some(109 | 232)) {
                    let _ = sender.send(capture);
                } else {
                    capture.failed = true;
                    let _ = sender.send(capture);
                }
                return;
            }
            if available == 0 {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            let read_len = buffer.len().min(available as usize);
            let count = match stream.read(&mut buffer[..read_len]) {
                Ok(0) => {
                    let _ = sender.send(capture);
                    return;
                }
                Ok(count) => count,
                Err(_) => {
                    capture.failed = true;
                    let _ = sender.send(capture);
                    return;
                }
            };
            let remaining = MAX_CAPTURE_BYTES_PER_STREAM.saturating_sub(capture.bytes.len());
            let retained = remaining.min(count);
            capture.bytes.extend_from_slice(&buffer[..retained]);
            if retained < count {
                capture.truncated = true;
            }
        }
    });
    CaptureWorker { receiver, cancel }
}

fn receive_capture(worker: Option<CaptureWorker>) -> StreamCapture {
    let Some(worker) = worker else {
        return timed_out_capture();
    };
    match worker.receiver.recv_timeout(Duration::from_secs(2)) {
        Ok(capture) => capture,
        Err(_) => {
            let _ = worker.cancel.send(());
            worker
                .receiver
                .recv_timeout(Duration::from_millis(250))
                .unwrap_or_else(|_| timed_out_capture())
        }
    }
}

fn timed_out_capture() -> StreamCapture {
    StreamCapture {
        bytes: Vec::new(),
        truncated: true,
        failed: true,
        timed_out: true,
    }
}

struct SchedulerState {
    ready: VecDeque<String>,
    commands: BTreeMap<String, VerificationCommand>,
    remaining_dependencies: BTreeMap<String, usize>,
    dependents: BTreeMap<String, Vec<String>>,
    completed: usize,
    total: usize,
    resource_budget: usize,
    reserved_resources: usize,
    metrics: RuntimeMetrics,
}

impl SchedulerState {
    fn new(commands: Vec<VerificationCommand>, resource_budget: usize) -> Self {
        let executed_ids = commands
            .iter()
            .map(|command| command.id.clone())
            .collect::<BTreeSet<_>>();
        let mut by_id = BTreeMap::new();
        let mut remaining_dependencies = BTreeMap::new();
        let mut dependents: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for command in commands {
            let execution_dependencies = command
                .dependencies
                .iter()
                .filter(|dependency| executed_ids.contains(*dependency))
                .cloned()
                .collect::<Vec<_>>();
            remaining_dependencies.insert(command.id.clone(), execution_dependencies.len());
            for dependency in execution_dependencies {
                dependents
                    .entry(dependency)
                    .or_default()
                    .push(command.id.clone());
            }
            by_id.insert(command.id.clone(), command);
        }
        let ready = remaining_dependencies
            .iter()
            .filter_map(|(id, count)| (*count == 0).then_some(id.clone()))
            .collect::<VecDeque<_>>();
        let total = by_id.len();
        Self {
            ready,
            commands: by_id,
            remaining_dependencies,
            dependents,
            completed: 0,
            total,
            resource_budget,
            reserved_resources: 0,
            metrics: RuntimeMetrics::new(),
        }
    }

    fn take_ready(&mut self) -> Option<VerificationCommand> {
        let position = self.ready.iter().position(|id| {
            self.commands.get(id).is_some_and(|command| {
                self.reserved_resources
                    .saturating_add(command.resource_weight)
                    <= self.resource_budget
            })
        })?;
        let id = self.ready.remove(position)?;
        let command = self.commands.remove(&id)?;
        self.reserved_resources = self
            .reserved_resources
            .saturating_add(command.resource_weight);
        Some(command)
    }

    fn complete(
        &mut self,
        id: &str,
        protected: bool,
        resource_weight: usize,
        outcome: ExecutionOutcome,
    ) {
        self.completed += 1;
        self.reserved_resources = self.reserved_resources.saturating_sub(resource_weight);
        self.metrics.nodes_executed += 1;
        if outcome.spawned {
            self.metrics.processes_spawned += 1;
        } else {
            self.metrics.process_spawn_failures += 1;
        }
        if protected && outcome.spawned {
            self.metrics.protected_nodes_executed += 1;
        }
        if !outcome.passed {
            self.metrics.passed = false;
        }
        self.metrics.outcomes.insert(id.into(), outcome);
        if let Some(dependents) = self.dependents.get(id) {
            for dependent in dependents {
                let remaining = self
                    .remaining_dependencies
                    .get_mut(dependent)
                    .expect("planned dependent exists");
                *remaining -= 1;
                if *remaining == 0 {
                    self.ready.push_back(dependent.clone());
                }
            }
        }
    }
}

#[derive(Clone)]
struct RuntimeMetrics {
    nodes_executed: usize,
    processes_spawned: usize,
    process_spawn_failures: usize,
    protected_nodes_executed: usize,
    passed: bool,
    outcomes: BTreeMap<String, ExecutionOutcome>,
}

impl RuntimeMetrics {
    fn new() -> Self {
        Self {
            nodes_executed: 0,
            processes_spawned: 0,
            process_spawn_failures: 0,
            protected_nodes_executed: 0,
            passed: true,
            outcomes: BTreeMap::new(),
        }
    }
}

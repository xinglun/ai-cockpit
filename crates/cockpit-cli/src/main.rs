use anyhow::{Context, Result};
use clap::{ArgAction, Args, Parser, Subcommand};
use cockpit_agent::AgentExitCode;
use cockpit_git::GitRepository;
use cockpit_knowledge::{Query, query_with_metrics};
use cockpit_protocol::{
    AgentProvider, ConcurrencyBoundary, CoordinationEvent, CoordinationRequest,
    CoordinationRequestState, DataClassification, DelegatedEvidence, ENVIRONMENT_DRIFT_CAPABILITY,
    EvidenceAssurance, EvidencePersistence, EvidenceRetention, HumanDecision, ReleasePlan,
    ReleasePlanEnvelope, ReleaseRequestInput, RepositoryConfig, RuntimeCapabilityBinding,
    RuntimeContext, VerificationDeclaration, VerificationStage, VerificationTier,
    WorktreeRegistration, validate_protocol_version,
};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_historical_work_item_with_runtime, attach, close_work_item_with_decision_and_runtime,
    close_work_item_with_structured_decision_and_runtime, generate_knowledge,
    preflight_work_item_with_runtime_report, prepare_archive_outcome_delivery,
    resolve_archived_verification_route, resolve_verification_route,
    retire_active_work_item_with_runtime, scaffold_work_item,
};
use cockpit_verification::gate_plan::{
    GATE_PLAN_FAILURE_EXIT_CODE, GatePlan, GatePlanError, GatePlanFailure, GatePlanInput,
    failure_metadata, load_manifest, plan_gate_route, validate_gate_plan,
    validate_lifecycle_summary,
};
use serde::de::DeserializeOwned;
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

mod cognitive_benefit;
mod github_evidence;
mod runtime_identity;

/// Inspect, verify and govern repository work with explicit authority and evidence.
#[derive(Debug, Parser)]
#[command(name = "ai-cockpit", version)]
struct Cli {
    /// Explicit request-scoped caller metadata, never inferred from a policy or environment.
    #[arg(long, global = true)]
    task_caller: Option<PathBuf>,
    #[command(subcommand)]
    command: CommandKind,
}

#[derive(Debug)]
struct GatePlanCliFailure {
    detail: String,
    payload: GatePlanFailure,
}

impl GatePlanCliFailure {
    fn from_error(error: anyhow::Error) -> Self {
        let detail = format!("{error:#}");
        Self {
            payload: failure_metadata(&detail),
            detail,
        }
    }
}

impl std::fmt::Display for GatePlanCliFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for GatePlanCliFailure {}

#[derive(Debug, Subcommand)]
enum CommandKind {
    /// Inspect repository identity, integration and Runtime capabilities without modifying them.
    Inspect {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Emit the stable machine-readable JSON projection. This is also
        /// the default output for this read-only diagnostic.
        #[arg(long)]
        json: bool,
    },
    /// Evaluate a Contract against current repository facts and record its preflight decision.
    Preflight {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Path to the Contract JSON file for this operation.
        #[arg(long)]
        contract: PathBuf,
    },
    /// Observe current repository facts and governance without running project checks.
    Observe {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
    },
    /// Emit a deterministic JSONL filesystem isolation manifest in one Rust
    /// process. `--paths-file` accepts a NUL-delimited source path list when
    /// Git, rather than a whole-tree walk, defines the source boundary.
    IsolationManifest {
        /// Root directory whose filesystem entries are inspected.
        #[arg(long)]
        root: PathBuf,
        /// Destination output file; stdout is used when this option is omitted.
        #[arg(long)]
        output: Option<PathBuf>,
        /// NUL-delimited relative source path list supplied by Git.
        #[arg(long)]
        paths_file: Option<PathBuf>,
        /// Relative output directory whose ancestor metadata is masked in a
        /// source-checkout manifest.
        #[arg(long)]
        mask_ancestors_of: Option<PathBuf>,
    },
    /// Attach this repository and write its local AI Cockpit integration records.
    Attach {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
    },
    /// Show current repository governance, evidence and recommended next actions.
    Status {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Emit the stable machine-readable JSON projection. This is also
        /// the default output for this read-only diagnostic.
        #[arg(long)]
        json: bool,
    },
    /// Check compatibility between this Runtime and the attached repository protocol.
    Compatibility {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
    },
    /// Plan or explicitly apply an attached repository protocol migration.
    Migrate {
        #[command(subcommand)]
        command: MigrateCommand,
    },
    /// Declare and start an implementation Work Item through native admission.
    Start {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// Human-owned intent for the Work Item.
        #[arg(long)]
        intent: String,
        /// Concrete goal the Work Item must achieve.
        #[arg(long)]
        goal: String,
        /// Comma-separated repository-relative paths or supported scope patterns.
        #[arg(long, value_delimiter = ',')]
        scope: Vec<String>,
        /// Comma-separated paths or patterns that remain explicitly excluded.
        #[arg(long, value_delimiter = ',')]
        out_of_scope: Vec<String>,
        /// Declared risk classification; it does not grant authority.
        #[arg(long, default_value = "normal")]
        risk: String,
        /// Declared authority state; native evidence must still justify it.
        #[arg(long, default_value = "missing")]
        authority: String,
        /// Repeat this option for multiple criteria.  Acceptance text is
        /// governance prose and may legitimately contain commas; parsing it
        /// as a delimiter silently changes the Contract bytes.
        #[arg(long, action = ArgAction::Append)]
        acceptance: Vec<String>,
        /// Required evidence classes. Supported forms: verification,
        /// verification_receipt, verification-receipt, delegated:<provider>,
        /// delegated_evidence, external_evidence, or a non-empty custom label.
        /// Lifecycle stage names such as public-install belong to later
        /// evidence and are rejected at this boundary.
        #[arg(long, value_delimiter = ',')]
        required_evidence: Vec<String>,
        /// Explicit verification command; repeat for multiple commands. When
        /// omitted, Runtime records the repository-observed default.
        #[arg(long, action = ArgAction::Append)]
        verification: Vec<String>,
        /// JSON array of typed, unverified ScenarioCoverageEntry plans.
        #[arg(long)]
        scenario_coverage: Option<PathBuf>,
        /// Explicit local development-review policy; requires formal owner review.
        #[arg(long)]
        development_review: Option<PathBuf>,
        /// Explicit bounded local policy; declaration alone does not approve it.
        #[arg(long)]
        task_authorization: Option<PathBuf>,
        /// Persist preflight and create one before-edit checkpoint when the
        /// decision has no blockers or human-confirmation boundary.
        #[arg(long, default_value_t = false)]
        prepare: bool,
        /// Source reference to append before preflight; repeat for multiple.
        #[arg(long, action = ArgAction::Append)]
        source: Vec<String>,
    },
    /// Record the native before-edit checkpoint for a Work Item.
    Checkpoint {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
    },
    /// Evaluate current evidence and governance before marking a Work Item finish-ready.
    Finish {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// Emit machine-only output without the human Outcome handoff on stderr.
        #[arg(long)]
        json: bool,
        /// Conversation language for the human Outcome; when omitted the
        /// adapter locale/environment is used.
        #[arg(
            long,
            value_parser = clap::builder::PossibleValuesParser::new(
                cockpit_protocol::WORK_ITEM_OUTCOME_LANGUAGE_VALUES.iter().copied()
            )
        )]
        language: Option<String>,
    },
    /// Archive a finish-ready Work Item with its retained evidence.
    Archive {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// Emit valid JSON; the full Outcome remains in outcomeDelivery while
        /// the separate human handoff on stderr is suppressed.
        #[arg(long)]
        json: bool,
        /// Conversation language for the archived human Outcome delivery.
        #[arg(
            long,
            value_parser = clap::builder::PossibleValuesParser::new(
                cockpit_protocol::WORK_ITEM_OUTCOME_LANGUAGE_VALUES.iter().copied()
            )
        )]
        language: Option<String>,
    },
    /// Archive complete schema-v2 evidence produced by an older Runtime.
    /// This explicit compatibility route does not rerun source verification;
    /// finalization and close remain required after archive.
    ArchiveHistorical {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// Emit valid JSON; the full Outcome remains in outcomeDelivery while
        /// the separate human handoff is suppressed.
        #[arg(long)]
        json: bool,
        /// Conversation language for the archived human Outcome delivery.
        #[arg(
            long,
            value_parser = clap::builder::PossibleValuesParser::new(
                cockpit_protocol::WORK_ITEM_OUTCOME_LANGUAGE_VALUES.iter().copied()
            )
        )]
        language: Option<String>,
    },
    /// Record an explicit human close decision after required lifecycle checks.
    Close {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// Actual human decision to record; this option cannot create approval.
        #[arg(long)]
        human_decision: String,
        /// Actor identity explicitly supplied for the decision or observation.
        #[arg(long)]
        actor: Option<String>,
        /// Concrete source of the actual decision authority.
        #[arg(long)]
        authority_source: Option<String>,
        /// Reason supporting the explicit decision or recovery.
        #[arg(long)]
        reason: Option<String>,
        /// Comma-separated references to the evidence supporting this decision.
        #[arg(long, value_delimiter = ',')]
        evidence_ref: Vec<String>,
        /// Comma-separated exact policy identifiers binding this decision.
        #[arg(long, value_delimiter = ',')]
        policy_ref: Vec<String>,
        /// RFC 3339 timestamp at which the actual decision was made.
        #[arg(long)]
        decided_at: Option<String>,
        /// Explicit condition required before paused work may resume.
        #[arg(long)]
        resume_condition: Option<String>,
        /// Emit machine-only output without the human Outcome handoff on stderr.
        #[arg(long)]
        json: bool,
        /// Conversation language for the human Outcome.
        #[arg(
            long,
            value_parser = clap::builder::PossibleValuesParser::new(
                cockpit_protocol::WORK_ITEM_OUTCOME_LANGUAGE_VALUES.iter().copied()
            )
        )]
        language: Option<String>,
    },
    /// Plan or execute bounded verification with exact command and evidence identities.
    Verify {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item to bind to this operation.
        #[arg(long)]
        work_item: Option<String>,
        /// Explicit verification program; repeat for multiple commands.
        #[arg(long, action = ArgAction::Append)]
        command: Vec<String>,
        /// Select exact current declared node IDs. Repeat to select several;
        /// omission selects the full declaration inventory. Selection is not full coverage.
        #[arg(long, action=ArgAction::Append, requires="work_item", conflicts_with_all=["command","args","archived_recovery"])]
        node: Vec<String>,
        /// Comma-separated program arguments, preserving their declared order.
        #[arg(long, value_delimiter = ',')]
        args: Vec<String>,
        /// Bounded untrusted source manifest for the full current Work Item inventory.
        /// Every source is checked live; remaining declarations run natively.
        #[arg(long, requires = "work_item", conflicts_with_all = ["command", "args", "node", "plan_only", "archived_recovery"])]
        source_bundle: Option<PathBuf>,
        /// Maximum worker count. Work Item verification defaults to serial and
        /// rejects parallel requests until per-node dependencies and outputs
        /// can be verified by the Runtime.
        #[arg(long, default_value_t = 1)]
        workers: usize,
        /// Governance stage: task, pre_ci, pr, merge or release.
        #[arg(long, default_value = "task")]
        stage: String,
        /// Finite command timeout in seconds. Explicit values require a
        /// finite Contract or repository policy ceiling; omission defaults to
        /// 600 seconds for Cargo package tests and 300 seconds otherwise.
        #[arg(long)]
        timeout_seconds: Option<u64>,
        /// Resolve the route and emit its deterministic verification plan
        /// without spawning project verification commands.
        #[arg(long)]
        plan_only: bool,
        /// Required for pr/merge/release when no Work Item Contract supplies
        /// the immutable base revision.
        #[arg(long)]
        base_revision: Option<String>,
        /// Run one fresh, current-Runtime verification for an archived Work
        /// Item and append a source-integration recovery receipt.
        #[arg(long)]
        archived_recovery: bool,
    },
    /// Evaluate the Contract/policy route without writing repository
    /// governance evidence.  CI uses this before executing its command gate.
    Gate {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Path to the Contract JSON file for this operation.
        #[arg(long)]
        contract: PathBuf,
        /// Governance stage: task, pre_ci, pr, merge or release.
        #[arg(long, default_value = "pr")]
        stage: String,
        /// Runner identity to record with this verification or gate.
        #[arg(long, default_value = "hosted")]
        runner: String,
        /// Immutable base revision used for the source comparison.
        #[arg(long)]
        base_revision: Option<String>,
        /// JSON report output path, relative to --repo unless absolute.
        /// Overwrites an existing file; paths inside .ai are rejected.
        #[arg(long)]
        report: Option<PathBuf>,
    },
    /// Validate an existing Contract quality-gate report against its
    /// repository-bound route receipt without executing any gates.
    GateReport {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Path to the existing JSON report used by this operation.
        #[arg(long)]
        report: PathBuf,
        /// Path to the repository-bound route receipt JSON.
        #[arg(long)]
        route_receipt: PathBuf,
    },
    /// Build or validate the shared Rust CI gate plan.  The compatibility
    /// Python route may delegate here so production and regression paths use
    /// one manifest/rule implementation.
    GatePlan {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Path to the gate manifest JSON.
        #[arg(long)]
        manifest: PathBuf,
        /// Base revision for the gate route comparison.
        #[arg(long)]
        base: Option<String>,
        /// Head revision for the gate route comparison.
        #[arg(long)]
        head: Option<String>,
        /// Gate-manifest stage: task, pre_ci, pull_request, merge or release.
        #[arg(long)]
        stage: Option<String>,
        /// Declared risk classification; it does not grant authority.
        #[arg(long)]
        risk: Option<String>,
        /// Path to the Contract JSON file for this operation.
        #[arg(long)]
        contract: Option<PathBuf>,
        /// Requested gate profile: light, standard or strict.
        #[arg(long)]
        profile: Option<String>,
        /// Receipt JSON output path; read it instead when --validate-receipt is set.
        #[arg(long)]
        receipt: PathBuf,
        /// Validate an existing receipt against the same current facts.
        #[arg(long)]
        validate_receipt: bool,
    },
    /// Resolve a typed, identity-bound release request into one canonical
    /// ReleasePlan envelope. This command performs no build, verification, or
    /// publication work.
    ReleasePlan {
        /// Path to the typed JSON input required by this operation.
        #[arg(long)]
        input: PathBuf,
        /// Destination output file; stdout is used when this option is omitted.
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Inspect and manage policy-bound delegated evidence.
    Evidence {
        #[command(subcommand)]
        command: EvidenceCommand,
    },
    /// Query, export or evaluate repository audit observations.
    Audit {
        #[command(subcommand)]
        command: AuditCommand,
    },
    /// Declare, inspect and advance Work Items through native lifecycle boundaries.
    WorkItem {
        #[command(subcommand)]
        command: WorkItemCommand,
    },
    /// Inspect Runtime capabilities and supported interface descriptions.
    Capability {
        #[command(subcommand)]
        command: CapabilityCommand,
    },
    /// Diagnose observed repository or Work Item governance cost and performance.
    Diagnose {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item to bind to this operation.
        #[arg(long)]
        work_item: Option<String>,
    },
    /// Query repository knowledge with explicit source and state filters.
    Knowledge {
        #[command(subcommand)]
        command: KnowledgeCommand,
    },
    /// Review and confirm explicit repository quality-command profiles.
    Profile {
        #[command(subcommand)]
        command: ProfileCommand,
    },
    /// Serve the repository-bound Model Context Protocol interface over stdio.
    Mcp {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
    },
    /// Diagnose repository integration and environment compatibility.
    Doctor {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Emit the stable machine-readable JSON projection. This is also
        /// the default output for this read-only diagnostic.
        #[arg(long)]
        json: bool,
    },
    /// Inspect, install, repair or detach repository agent adapters.
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },
}

#[derive(Debug, Subcommand)]
enum EvidenceCommand {
    /// Inspect portable native execution evidence or request exact admission.
    Bundle {
        #[command(subcommand)]
        command: github_evidence::PortableEvidenceCommand,
    },
    /// Plan GitHub verification evidence from independent read-only provider metadata.
    Github {
        #[command(subcommand)]
        command: github_evidence::GithubEvidenceCommand,
    },
    /// Import delegated evidence metadata and raw bytes under repository policy.
    Import {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item to bind to this operation.
        #[arg(long)]
        work_item: String,
        /// Path to the delegated-evidence metadata JSON.
        #[arg(long)]
        metadata: PathBuf,
        /// Path to the raw delegated-evidence payload file.
        #[arg(long)]
        raw: PathBuf,
    },
    /// Record a bounded observation of delegated evidence without promoting its authority.
    Observe {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item to bind to this operation.
        #[arg(long)]
        work_item: String,
        /// Path to the delegated-evidence metadata JSON.
        #[arg(long)]
        metadata: PathBuf,
        /// Path to the raw delegated-evidence payload file.
        #[arg(long)]
        raw: PathBuf,
    },
    /// List retained delegated evidence for a Work Item.
    List {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item to bind to this operation.
        #[arg(long)]
        work_item: String,
    },
    /// List recorded delegated-evidence observations for a Work Item.
    ListObservations {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item to bind to this operation.
        #[arg(long)]
        work_item: String,
    },
    /// Record classification, storage and retention policy for a Work Item evidence source.
    Policy {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item to bind to this operation.
        #[arg(long)]
        work_item: String,
        /// Data classification to bind to the evidence retention policy.
        #[arg(long)]
        classification: String,
        /// Evidence storage policy for retained metadata and raw bytes.
        #[arg(long)]
        persistence: String,
        /// Maximum evidence retention duration in days.
        #[arg(long)]
        retention_days: Option<u64>,
        /// Explicit evidence expiry timestamp in RFC 3339 format.
        #[arg(long)]
        expires_at: Option<String>,
        /// Disposal action declared by the evidence retention policy.
        #[arg(long)]
        disposal_action: String,
    },
    /// Show evidence expiry and disposal actions without deleting evidence.
    PurgePlan {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Explicit Unix timestamp for a deterministic purge plan.
        #[arg(long)]
        now_epoch_seconds: Option<u64>,
    },
}

#[derive(Debug, Subcommand)]
enum AuditCommand {
    /// Measure and optionally enforce the cognitive-benefit acceptance check.
    CognitiveBenefit {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// AI Cockpit executable used by the cognitive-benefit probe.
        #[arg(long)]
        binary: Option<PathBuf>,
        /// Fail when the cognitive-benefit acceptance check is not satisfied.
        #[arg(long)]
        check: bool,
    },
    /// Export audit events using explicit filters and pagination.
    Export {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Destination output file; stdout is used when this option is omitted.
        #[arg(long)]
        output: Option<PathBuf>,
        #[command(flatten)]
        filters: AuditQueryArgs,
    },
    /// Query repository audit events without modifying their records.
    Query {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        #[command(flatten)]
        filters: AuditQueryArgs,
    },
}

#[derive(Debug, Default, Args)]
struct AuditQueryArgs {
    /// Filter by Work Item identifier.
    #[arg(long)]
    work_item_id: Option<String>,
    /// Inclusive lower timestamp bound for the audit query.
    #[arg(long)]
    from: Option<String>,
    /// Inclusive upper timestamp bound for the audit query.
    #[arg(long)]
    to: Option<String>,
    /// Filter by reported model label; the label remains a caller claim.
    #[arg(long)]
    reported_model: Option<String>,
    /// Filter audit observations by their recorded actor.
    #[arg(long)]
    actor: Option<String>,
    /// Filter by audit event type.
    #[arg(long)]
    event_type: Option<String>,
    /// Maximum number of events to return in this page.
    #[arg(long)]
    limit: Option<u16>,
    /// Pagination cursor returned by the previous query.
    #[arg(long)]
    cursor: Option<String>,
    /// IANA time zone for human-readable timestamps.
    #[arg(long)]
    display_timezone: Option<String>,
}

impl From<AuditQueryArgs> for cockpit_protocol::AuditQueryFilters {
    fn from(args: AuditQueryArgs) -> Self {
        Self {
            work_item_id: args.work_item_id,
            from: args.from,
            to: args.to,
            reported_model: args.reported_model,
            actor: args.actor,
            event_type: args.event_type,
            limit: args.limit,
            cursor: args.cursor,
            display_timezone: args.display_timezone,
        }
    }
}

#[derive(Debug, Subcommand)]
#[command(
    after_help = "Before repository operations, run `ai-cockpit agent first-start --repo <path>` and follow the canonical gate."
)]
enum AgentCommand {
    /// Print the mandatory first-start procedure before any repository operation.
    FirstStart {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
    },
    /// List repository agent adapters and their recorded state.
    List {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
    },
    /// Install the selected agent adapter in this repository.
    Install {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Agent adapter provider to install, inspect or remove.
        #[arg(long, default_value = "auto")]
        provider: AgentProviderArg,
    },
    /// Diagnose the repository agent adapter integration.
    Doctor {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Repair the selected repository agent adapter integration.
    Repair {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Agent adapter provider to install, inspect or remove.
        #[arg(long)]
        provider: Option<AgentProviderArg>,
    },
    /// Remove the selected repository agent adapter integration.
    Detach {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Agent adapter provider to install, inspect or remove.
        #[arg(long)]
        provider: AgentProviderArg,
    },
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum AgentProviderArg {
    Auto,
    GenericAgentsMd,
    Codex,
    Claude,
    Gemini,
    Cursor,
}

impl AgentProviderArg {
    fn provider(&self) -> Option<AgentProvider> {
        match self {
            Self::Auto => None,
            Self::GenericAgentsMd => Some(AgentProvider::GenericAgentsMd),
            Self::Codex => Some(AgentProvider::Codex),
            Self::Claude => Some(AgentProvider::Claude),
            Self::Gemini => Some(AgentProvider::Gemini),
            Self::Cursor => Some(AgentProvider::Cursor),
        }
    }
}

#[derive(Debug, Subcommand)]
enum KnowledgeCommand {
    /// Query knowledge by topic, component, state or Work Item.
    Query {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Knowledge topic to query.
        #[arg(long)]
        topic: Option<String>,
        /// Repository component to include in the knowledge query.
        #[arg(long)]
        component: Option<String>,
        /// Knowledge evidence state to include in the query.
        #[arg(long)]
        state: Option<String>,
        /// Filter by Work Item identifier.
        #[arg(long)]
        work_item_id: Option<String>,
        /// Return the version 2 knowledge projection.
        #[arg(long)]
        v2: bool,
    },
}

#[derive(Debug, Subcommand)]
enum WorkItemCommand {
    /// Inspect an eligible historical activation gap or explicitly record a
    /// newly supplied present-baseline decision; never approves Contract C.
    ActivationRecovery {
        #[command(subcommand)]
        command: WorkItemActivationRecoveryCommand,
    },
    /// Create a not-ready Work Item scaffold without claiming authorization or completion.
    New {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// Work Item mode to scaffold; this does not authorize execution.
        #[arg(long)]
        mode: String,
    },
    /// Append a revalidation after a direct Contract change; it is not needed after a successful amend.
    RevalidateAmendment {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// Reason supporting the explicit decision or recovery.
        #[arg(long)]
        reason: String,
    },
    /// Amend human-owned Contract fields and automatically record amendment revalidation; query status for the next admitted action.
    Amend {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// Legacy JSON object with additive scopeAppend, outOfScopeAppend,
        /// sourcesAppend, verificationAppend, acceptanceAppend,
        /// requiredEvidenceClassesAppend, scenarioCoverageAppend, and/or scenarioCoveragePlanAppend arrays.
        #[arg(long, requires = "reason", conflicts_with = "request")]
        input: Option<PathBuf>,
        /// Rationale for the legacy additive input adapter.
        #[arg(long, requires = "input", conflicts_with = "request")]
        reason: Option<String>,
        /// Strict ContractAmendmentRequest JSON; supports recorded set/clear/remove/replace/reorder operations.
        #[arg(long, conflicts_with_all = ["input", "reason"])]
        request: Option<PathBuf>,
    },
    /// Read the append-only history of accepted Contract amendments.
    Amendments {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
    },
    /// Append a successor revalidation for an archived Contract that was
    /// legitimately amended after its historical verification.  The
    /// predecessor archive/evidence bytes remain immutable.
    RevalidateArchived {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// Successor Work Item id that will verify the current Contract.
        #[arg(long)]
        successor: String,
        /// Reason supporting the explicit decision or recovery.
        #[arg(long)]
        reason: String,
        /// Actor identity explicitly supplied for the decision or observation.
        #[arg(long)]
        actor: String,
        /// Concrete source of the actual decision authority.
        #[arg(long)]
        authority_source: String,
        /// Explicit condition required before paused work may resume.
        #[arg(long)]
        resume_condition: String,
        /// Evidence reference; repeat --evidence-ref for multiple references.
        #[arg(long, action = ArgAction::Append)]
        evidence_ref: Vec<String>,
        /// Exact policy identifier; repeat --policy-ref for multiple identifiers.
        #[arg(long, action = ArgAction::Append)]
        policy_ref: Vec<String>,
    },
    /// Derive the implementation approach from current Work Item declarations.
    Approach {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
    },
    Outcome {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        #[command(flatten)]
        query: cockpit_protocol::WorkItemOutcomeQueryArgs,
    },
    /// Record an explicit repository-bound usage claim through Runtime admission.
    Usage {
        #[command(subcommand)]
        command: WorkItemUsageCommand,
    },
    /// Inspect the exact committed material considered by the review boundary.
    MaterialReview {
        #[command(subcommand)]
        command: WorkItemMaterialReviewCommand,
    },
    /// Prepare a deterministic read-only authorization manifest for explicit same-repository Work Items.
    AuthorizationManifest {
        #[command(subcommand)]
        command: WorkItemAuthorizationManifestCommand,
    },
    /// Move failed-attempt artifacts left by an older/interrupted archive
    /// into the immutable archive and bind them with a reconciliation receipt.
    ReconcileArtifacts {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
    },
    /// Retire an active Work Item as already integrated or explicitly replaced.
    /// The input is a strict ActiveWorkItemRetirementRequest JSON document.
    Retire {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// Path to the typed JSON input required by this operation.
        #[arg(long)]
        input: PathBuf,
    },
    /// Bind branch/worktree/provider/PR context before archive.
    FinalizePlan {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// JSON ResourceFinalizationContext file.
        #[arg(long)]
        input: PathBuf,
    },
    /// Record a strict provider-side finalization receipt after archive.
    Finalize {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// JSON ResourceFinalizationReceipt file.
        #[arg(long)]
        input: PathBuf,
    },
    /// Revalidate the stored finalization receipt and local cleanup state.
    FinalizeVerify {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
    },
    /// Append an exact local branch/worktree cleanup observation for a closed
    /// ordinary Work Item. Provider-bound Work Items keep using finalize.
    OrdinaryCleanup {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
    },
    /// Classify an immutable legacy receipt, or record a first direct-merge
    /// finalization when no predecessor exists, without rewriting history.
    FinalizeRecovery {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// JSON historical recovery receipt or complete direct-merge receipt.
        #[arg(long)]
        input: PathBuf,
    },
    /// Inspect immutable legacy finalization facts and print a recovery
    /// skeleton without writing repository state.
    FinalizeRecoveryPlan {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// Optional real merge commit for a historical direct merge without a PR.
        #[arg(long)]
        merge_commit: Option<String>,
    },
    /// Prepare an explicit present-day legacy source registration without writes.
    LegacyCloseoutSourcePlan {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Source checkout containing the exact retained closeout evidence.
        #[arg(long)]
        source_repo: Option<PathBuf>,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// Path to the typed JSON input required by this operation.
        #[arg(long)]
        input: PathBuf,
    },
    /// Publish only the exact independently retained legacy source plan.
    LegacyCloseoutSourceRecord {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// Path to the typed JSON input required by this operation.
        #[arg(long)]
        input: PathBuf,
    },
    /// Inspect a verified closeout in another checkout without writing either checkout.
    CloseoutRecoveryPlan {
        /// Destination checkout whose missing local closeout is being inspected.
        #[arg(long)]
        repo: PathBuf,
        /// Independent source checkout containing the verified closed Work Item.
        #[arg(long)]
        source_repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
    },
    /// Import exact verified closeout evidence from another checkout of the same repository.
    CloseoutRecover {
        /// Destination checkout whose closeout evidence is incomplete.
        #[arg(long)]
        repo: PathBuf,
        /// Independent source checkout containing the verified closed Work Item.
        #[arg(long)]
        source_repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
    },
    /// Read Work Item lifecycle, authority, evidence and action admission.
    Status {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long, required_unless_present = "all", conflicts_with = "all")]
        id: Option<String>,
        /// Aggregate every active and archived Work Item in stable ID order.
        #[arg(long, conflicts_with = "id")]
        all: bool,
        /// Emit the stable machine-readable status snapshot.
        #[arg(long)]
        json: bool,
        /// Read-only exact action/file/target question; never a reusable permission token.
        #[arg(long, conflicts_with = "all")]
        authorization_request: Option<PathBuf>,
    },
    /// Inspect a Work Item and its declared governance context.
    Inspect {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
    },
    /// Declare dependencies, conflicts and parallel-work intent for a Work Item.
    Declare {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// Comma-separated Work Item dependencies.
        #[arg(long, value_delimiter = ',')]
        depends_on: Vec<String>,
        /// Comma-separated Work Items that conflict with this work.
        #[arg(long, value_delimiter = ',')]
        conflicts_with: Vec<String>,
        /// Declare parallel-work intent; native admission and a slot lease are still required.
        #[arg(long, default_value_t = false)]
        parallelizable: bool,
    },
    /// Validate the Work Item declaration and current governance prerequisites.
    Validate {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// Emit the stable machine-readable validation report.
        #[arg(long)]
        json: bool,
    },
    /// Record typed controls or an actual identity-bound decision through native checks.
    Controls {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// JSON object containing scenarioCoverage, acceptanceEvidence,
        /// evidenceClasses, intentAlignment, finalDimensions, or identity-bound
        /// decisionEvidence. evidenceClasses binds supplied custom classes to
        /// regular repository files, passed results and current digests. An
        /// explicit evidenceClassStages policy allows unavailable finish-only
        /// members to be omitted before finish; pending placeholders still fail.
        #[arg(long)]
        input: PathBuf,
    },
    /// Apply an explicit recovery decision while retaining prior evidence.
    Recover {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// JSON RecoveryDecisionReceipt with retry/successor predecessor bindings.
        #[arg(long)]
        input: PathBuf,
    },
    /// Record an append-only recovery for an already selected multi-hop
    /// successor lineage without creating another successor Work Item.
    RecoverSelectedSuccessorLineage {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// JSON SelectedSuccessorLineageRecoveryReceipt.
        #[arg(long)]
        input: PathBuf,
    },
    /// Record the declared concurrency boundary for a Work Item.
    Boundary {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// JSON file containing a Contract concurrencyBoundary object.
        #[arg(long)]
        file: PathBuf,
    },
    /// Acquire, release or inspect bounded parallel-work slot leases.
    Slot {
        #[command(subcommand)]
        command: WorkItemSlotCommand,
    },
    /// Inspect or mutate the candidate-only cross-Work-Item coordination store.
    Coordination {
        #[command(subcommand)]
        command: WorkItemCoordinationCommand,
    },
    /// Execute an exact composition in an isolated temporary worktree after
    /// refreshing cross-Work-Item admission.
    Composition {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// Exact Work Item coordination generation.
        #[arg(long)]
        generation: u64,
        /// JSON CompositionInput. Repository and state paths are resolved by
        /// this command and are not trusted from the input document.
        #[arg(long)]
        input: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum WorkItemUsageCommand {
    /// Import exact counters from one explicitly supplied completed response.
    Import {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        #[arg(long, help = cockpit_protocol::WORK_ITEM_USAGE_IMPORT_INPUT_DESCRIPTION)]
        input: PathBuf,
    },
    /// Append one strict UsageRecordRequest JSON claim.
    Record {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        #[arg(long, help = cockpit_protocol::WORK_ITEM_USAGE_RECORD_INPUT_DESCRIPTION)]
        input: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum WorkItemMaterialReviewCommand {
    /// Build the canonical read-only material-review request for an active Work Item.
    Plan {
        #[arg(long, help = "Repository path for source identity.")]
        repo: PathBuf,
        #[arg(long, help = "Active Work Item identifier.")]
        id: String,
    },
    /// Record a self-declared typed decision only when Contract opt-in and current Runtime admission allow it; this is not human, provider, or release approval.
    Record {
        #[arg(long, help = "Repository path for current Runtime admission.")]
        repo: PathBuf,
        #[arg(long, help = "Active Work Item identifier.")]
        id: String,
        #[arg(long, help = "Strict MaterialInspectionReviewDecisionInput JSON file.")]
        input: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum WorkItemAuthorizationManifestCommand {
    /// Build the exact member manifest without recording a decision or issuing grants.
    Plan {
        #[arg(long, help = "Repository path for current native member bindings.")]
        repo: PathBuf,
        #[arg(long, help = "Strict TaskAuthorizationBatchRequest JSON file.")]
        input: PathBuf,
    },
    /// Persist one exact owner decision and its independently bound child grants.
    Review {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        #[arg(long, help = "Strict TaskAuthorizationBatchManifest JSON file.")]
        manifest: PathBuf,
        #[arg(long, help = "Strict TaskAuthorizationBatchReview JSON file.")]
        review: PathBuf,
    },
    /// Reserve one start against the selected member's finite quota.
    Start {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        #[arg(long, help = "Strict TaskAuthorizationBatchStartRequest JSON file.")]
        input: PathBuf,
    },
    /// Reserve the initial attempt or a bounded retry of one exact declared check.
    Check {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        #[arg(long, help = "Strict TaskAuthorizationBatchCheckRequest JSON file.")]
        input: PathBuf,
    },
    /// Append the result for one previously reserved check attempt.
    Outcome {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        #[arg(long, help = "Strict TaskAuthorizationBatchOutcomeRequest JSON file.")]
        input: PathBuf,
    },
    /// Resolve one uncertain result after attaching inspection evidence.
    Inspect {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        #[arg(
            long,
            help = "Strict TaskAuthorizationBatchInspectionRequest JSON file."
        )]
        input: PathBuf,
    },
    /// Terminate only the selected member's child grant.
    Terminate {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        #[arg(
            long,
            help = "Strict TaskAuthorizationBatchTerminationRequest JSON file."
        )]
        input: PathBuf,
    },
    /// Read one member's current append-only execution ledger.
    Ledger {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Exact digest of the reviewed authorization batch manifest.
        #[arg(long)]
        manifest_digest: String,
        /// Work Item member whose append-only authorization ledger is read.
        #[arg(long)]
        work_item_id: String,
    },
}

#[derive(Debug, Subcommand)]
enum WorkItemActivationRecoveryCommand {
    /// Read-only exact request, including finite differences and retained unknowns.
    Plan {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
    },
    /// Record an actual newly supplied response for this exact request only.
    Record {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        #[arg(
            long,
            help = "Strict ActivationRecoveryDecision JSON with exact response-source digest."
        )]
        input: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum WorkItemSlotCommand {
    /// Acquire an admitted slot lease for this Work Item.
    Acquire {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
    },
    /// Release the exact slot lease using its recorded identity.
    Release {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// Exact identity of the slot lease to release.
        #[arg(long)]
        lease_id: String,
    },
    /// List current parallel-work slot leases.
    List {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum WorkItemCoordinationCommand {
    /// Inspect current coordination registrations, requests and events.
    Inspect {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
    },
    /// Register a Work Item and generation for coordinated work.
    Register {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Path to the WorktreeRegistration JSON input.
        #[arg(long)]
        input: PathBuf,
    },
    /// Record an impact report for dependency and conflict checks.
    ReportImpact {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Path to the CoordinationEvent JSON impact report.
        #[arg(long)]
        input: PathBuf,
    },
    /// Record a local coordination outcome reference for the specified generation.
    PublishOutcome {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// Exact Work Item coordination generation.
        #[arg(long)]
        generation: u64,
        /// Identity of the local coordination outcome to publish.
        #[arg(long)]
        outcome_id: String,
    },
    /// Record a bounded coordination pause request.
    RequestPause {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Path to the CoordinationRequest JSON input.
        #[arg(long)]
        input: PathBuf,
    },
    /// Record the explicit acknowledgement state of a coordination request.
    Acknowledge {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Identity of the coordination request to acknowledge.
        #[arg(long)]
        request_id: String,
        /// Acknowledgement state to record for this coordination request.
        #[arg(long)]
        state: String,
    },
    /// Resume coordination for the specified Work Item generation.
    Resume {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Work Item identifier.
        #[arg(long)]
        id: String,
        /// Exact Work Item coordination generation.
        #[arg(long)]
        generation: u64,
    },
    /// Recover coordination using an exact event and consumer generation.
    Recover {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Exact coordination event identity to recover from.
        #[arg(long)]
        event_id: String,
        /// Work Item that consumes the recovered coordination event.
        #[arg(long)]
        consumer_work_item_id: String,
        /// Exact generation of the consuming Work Item.
        #[arg(long)]
        consumer_generation: u64,
    },
    /// Check for unrecorded Runtime-observed environment drift without writing.
    CheckEnvironmentDrift {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Path to the EnvironmentDriftRequest JSON to inspect without writing.
        #[arg(long)]
        input: PathBuf,
    },
    /// Persist Runtime-observed environment drift before a dependent action.
    RecordEnvironmentDrift {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Path to the EnvironmentDriftRequest JSON to record.
        #[arg(long)]
        input: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum CapabilityCommand {
    /// Show supported Runtime capabilities or one interface description.
    Show {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Optionally return a deterministic description for one discoverable
        /// interface without observing repository state.
        #[arg(long)]
        surface: Option<String>,
        /// Description encoding; the existing no-surface registry remains JSON.
        #[arg(
            long,
            value_parser = clap::builder::PossibleValuesParser::new(
                cockpit_protocol::CAPABILITY_SHOW_FORMAT_VALUES.iter().copied()
            ),
            default_value = cockpit_protocol::CAPABILITY_SHOW_DEFAULT_FORMAT
        )]
        format: String,
        /// Language for Markdown labels; structured JSON is language-neutral.
        #[arg(
            long,
            value_parser = clap::builder::PossibleValuesParser::new(
                cockpit_protocol::CAPABILITY_SHOW_LANGUAGE_VALUES.iter().copied()
            ),
            default_value = cockpit_protocol::CAPABILITY_SHOW_DEFAULT_LANGUAGE
        )]
        language: String,
    },
}

#[derive(Debug, Subcommand)]
enum ProfileCommand {
    /// Confirm an explicit program and arguments for the repository quality profile.
    Confirm {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Executable to record in the explicit repository quality profile.
        #[arg(long)]
        program: String,
        /// Comma-separated program arguments, preserving their declared order.
        #[arg(long, value_delimiter = ',')]
        args: Vec<String>,
        /// Explicit child execution recipe JSON file, limited to 64 KiB.
        #[arg(long, value_name = "FILE")]
        execution_recipe: Option<PathBuf>,
    },
    /// Generate a repository quality-profile proposal without confirming it.
    Propose {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum MigrateCommand {
    /// Show the repository protocol migration plan without applying it.
    Plan {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
    },
    /// Apply the reviewed repository migration only with explicit approval.
    Apply {
        /// Repository checkout to inspect or operate on.
        #[arg(long)]
        repo: PathBuf,
        /// Apply only after the repository migration plan has been explicitly approved.
        #[arg(long)]
        approved: bool,
    },
}

fn outcome_repository_view(value: &str) -> cockpit_repository::OutcomeRenderView {
    if value == cockpit_protocol::WORK_ITEM_OUTCOME_VIEW_FULL {
        cockpit_repository::OutcomeRenderView::Full
    } else {
        cockpit_repository::OutcomeRenderView::Summary
    }
}

struct GatePlanOptions<'a> {
    repo: &'a Path,
    manifest: &'a Path,
    base: &'a Option<String>,
    head: &'a Option<String>,
    stage: &'a Option<String>,
    risk: &'a Option<String>,
    contract: Option<&'a PathBuf>,
    profile: Option<&'a str>,
    receipt: &'a Path,
    validate_receipt: bool,
}

fn run_gate_plan(options: GatePlanOptions<'_>) -> Result<()> {
    let GatePlanOptions {
        repo,
        manifest,
        base,
        head,
        stage,
        risk,
        contract,
        profile,
        receipt,
        validate_receipt,
    } = options;
    let root = fs::canonicalize(repo).context("canonicalize repository")?;
    let manifest_path = resolve_repo_path(&root, manifest)?;
    let manifest_bytes = fs::read(&manifest_path).context("read gate manifest")?;
    let manifest_value = load_manifest(&manifest_bytes).map_err(|error| anyhow::anyhow!(error))?;
    let manifest_digest = format!("sha256:{}", hex::encode(sha256_bytes(&manifest_bytes)));
    let receipt_path = resolve_repo_path(&root, receipt)?;
    let existing = if validate_receipt {
        let bytes = fs::read(&receipt_path).context("read gate-plan receipt")?;
        Some(
            serde_json::from_slice::<GatePlan>(&bytes)
                .map_err(|error| anyhow::anyhow!("invalid gate-plan receipt: {error}"))?,
        )
    } else {
        None
    };
    let base = base
        .as_deref()
        .or_else(|| existing.as_ref().map(|plan| plan.base_revision.as_str()))
        .ok_or_else(|| {
            anyhow::anyhow!("gate-plan requires --base unless --validate-receipt is used")
        })?;
    let head = head
        .as_deref()
        .or_else(|| existing.as_ref().map(|plan| plan.head_revision.as_str()))
        .unwrap_or("HEAD");
    let stage = stage
        .as_ref()
        .cloned()
        .or_else(|| existing.as_ref().map(|plan| plan.stage.clone()))
        .ok_or_else(|| {
            anyhow::anyhow!("gate-plan requires --stage unless --validate-receipt is used")
        })?;
    let requested_risk = risk
        .clone()
        .or_else(|| existing.as_ref().map(|plan| plan.requested_risk.clone()))
        .unwrap_or_else(|| "normal".into());
    let requested_profile = profile.map(str::to_owned).or_else(|| {
        existing
            .as_ref()
            .and_then(|plan| plan.requested_profile.clone())
    });
    let contract_path = contract.cloned().or_else(|| {
        existing
            .as_ref()
            .and_then(|plan| plan.contract_path.clone())
            .map(PathBuf::from)
    });
    let contract = contract_path
        .as_ref()
        .map(|path| load_contract_fact(&root, path))
        .transpose()?;
    let base_revision = resolve_git_revision(&root, base)?;
    let head_revision = resolve_git_revision(&root, head)?;
    let changed_paths = git_changed_paths(&root, &base_revision, &head_revision)?;
    let input = GatePlanInput {
        base_revision,
        head_revision,
        stage,
        risk: requested_risk,
        requested_profile,
        changed_paths,
        manifest_digest,
        contract_path: contract.as_ref().map(|fact| fact.relative_path.clone()),
        contract_digest: contract.as_ref().map(|fact| fact.digest.clone()),
        contract_risk: contract.as_ref().map(|fact| fact.risk.clone()),
    };
    let plan = plan_gate_route(&manifest_value, &input)
        .map_err(|error| anyhow::anyhow!(format_gate_plan_error(error)))?;
    if let Some(existing) = existing.as_ref() {
        validate_gate_plan(&manifest_value, &input, existing)
            .map_err(|error| anyhow::anyhow!(format_gate_plan_error(error)))?;
        println!("{}", serde_json::to_string_pretty(existing)?);
    } else {
        if let Some(parent) = receipt_path.parent() {
            fs::create_dir_all(parent).context("create gate-plan receipt parent")?;
        }
        fs::write(
            &receipt_path,
            format!("{}\n", serde_json::to_string_pretty(&plan)?),
        )
        .context("write gate-plan receipt")?;
        println!("{}", serde_json::to_string_pretty(&plan)?);
    }
    Ok(())
}

fn run_release_plan(input: &Path, output: Option<&Path>) -> Result<()> {
    let request_bytes = fs::read(input).context("read ReleaseRequest")?;
    let request: ReleaseRequestInput =
        serde_json::from_slice(&request_bytes).context("parse ReleaseRequestInput JSON")?;
    let plan = ReleasePlan::resolve_input(request).map_err(|error| anyhow::anyhow!(error))?;
    let envelope = ReleasePlanEnvelope::new(plan).map_err(|error| anyhow::anyhow!(error))?;
    let serialized = serde_json::to_string_pretty(&envelope)?;
    if let Some(output) = output {
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).context("create ReleasePlan output parent")?;
        }
        fs::write(output, format!("{serialized}\n")).context("write ReleasePlan envelope")?;
    }
    println!("{serialized}");
    Ok(())
}

struct ContractFact {
    relative_path: String,
    digest: String,
    risk: String,
}

fn load_contract_fact(root: &Path, path: &Path) -> Result<ContractFact> {
    let candidate = resolve_repo_path(root, path)?;
    let metadata = fs::symlink_metadata(&candidate).context("inspect route Contract")?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        anyhow::bail!("Contract path must be a regular file");
    }
    let canonical = fs::canonicalize(&candidate).context("canonicalize route Contract")?;
    let relative = canonical
        .strip_prefix(root)
        .map_err(|_| anyhow::anyhow!("Contract path escapes repository"))?
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/");
    let bytes = fs::read(&canonical).context("read route Contract")?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).context("parse route Contract")?;
    let risk = value
        .get("risk")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("Contract risk is missing"))?
        .to_owned();
    let summary = root.join(".ai/work-items/active").join(
        Path::new(&relative)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .strip_suffix(".contract.json")
            .map(|id| format!("{id}.summary.json"))
            .unwrap_or_default(),
    );
    if summary.is_symlink() || (summary.exists() && !summary.is_file()) {
        anyhow::bail!(
            "lifecycle_transition_invalid: active lifecycle Summary is not a regular file"
        );
    }
    if summary.exists() {
        let summary_bytes = fs::read(&summary).map_err(|error| {
            anyhow::anyhow!(
                "lifecycle_transition_invalid: unable to read active lifecycle Summary: {error}"
            )
        })?;
        let summary_value: serde_json::Value =
            serde_json::from_slice(&summary_bytes).map_err(|error| {
                anyhow::anyhow!(
                    "lifecycle_transition_invalid: active lifecycle Summary is malformed: {error}"
                )
            })?;
        validate_lifecycle_summary(&summary_value)
            .map_err(|error| anyhow::anyhow!(format_gate_plan_error(error)))?;
    }
    Ok(ContractFact {
        relative_path: relative,
        digest: format!("sha256:{}", hex::encode(sha256_bytes(&bytes))),
        risk,
    })
}

fn resolve_repo_path(root: &Path, path: &Path) -> Result<PathBuf> {
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    if candidate.exists() {
        let canonical = fs::canonicalize(&candidate).context("canonicalize repository path")?;
        if !canonical.starts_with(root) {
            anyhow::bail!("path escapes repository");
        }
    }
    Ok(candidate)
}

fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes).into()
}

fn run_git_bytes(root: &Path, args: &[String]) -> Result<Vec<u8>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .context("launch git fact query")?;
    if !output.status.success() {
        anyhow::bail!(
            "git fact query failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(output.stdout)
}

fn resolve_git_revision(root: &Path, revision: &str) -> Result<String> {
    let selector = format!("{revision}^{{commit}}");
    let output = run_git_bytes(root, &["rev-parse".into(), "--verify".into(), selector])?;
    let value = String::from_utf8(output).context("git revision is not UTF-8")?;
    let value = value.trim();
    if value.len() != 40 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        anyhow::bail!("invalid Git commit: {revision}");
    }
    Ok(value.to_owned())
}

fn git_changed_paths(root: &Path, base: &str, head: &str) -> Result<Vec<String>> {
    let queries = [
        vec![
            "diff".into(),
            "--name-only".into(),
            "-z".into(),
            format!("{base}...{head}"),
            "--".into(),
        ],
        vec![
            "diff".into(),
            "--name-only".into(),
            "-z".into(),
            head.into(),
            "--".into(),
        ],
        vec![
            "ls-files".into(),
            "--others".into(),
            "--exclude-standard".into(),
            "-z".into(),
        ],
    ];
    let mut paths = Vec::new();
    for query in queries {
        let output = run_git_bytes(root, &query)?;
        for path in output
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
        {
            paths.push(String::from_utf8(path.to_vec()).context("Git path is not UTF-8")?);
        }
    }
    cockpit_verification::gate_plan::normalize_paths(&paths)
        .map_err(|error| anyhow::anyhow!(format_gate_plan_error(error)))
}

fn format_gate_plan_error(error: GatePlanError) -> String {
    match error {
        GatePlanError::LifecycleInvalid(detail) => {
            format!("lifecycle_transition_invalid: {detail}")
        }
        GatePlanError::LifecycleStale(detail) => format!("lifecycle_transition_stale: {detail}"),
        other => other.to_string(),
    }
}

/// Select the language used by the human handoff. Adapters should pass the
/// active conversation language explicitly. The environment fallback keeps
/// direct CLI use useful while preserving one deterministic protocol mapping.
fn output_language(explicit: Option<&str>) -> &'static str {
    let value = explicit
        .map(str::to_owned)
        .or_else(|| std::env::var("AI_COCKPIT_LANGUAGE").ok())
        .or_else(|| std::env::var("LC_ALL").ok())
        .or_else(|| std::env::var("LANGUAGE").ok())
        .or_else(|| std::env::var("LANG").ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    cockpit_protocol::normalize_work_item_outcome_language(&value)
}

fn collaboration_runtime(runtime: &RuntimeContext) -> RuntimeCapabilityBinding {
    RuntimeCapabilityBinding {
        schema_version: cockpit_protocol::COLLABORATION_SCHEMA_VERSION,
        runtime_version: runtime.runtime_version.clone(),
        runtime_digest: runtime.runtime_digest.clone(),
        capability: ENVIRONMENT_DRIFT_CAPABILITY.into(),
    }
}

fn collaboration_store(
    repo: &Path,
    runtime: &RuntimeContext,
    write: bool,
) -> Result<cockpit_repository::CoordinationStore> {
    let git = GitRepository::discover(repo).context("discover repository topology")?;
    let binding = collaboration_runtime(runtime);
    let store = if write {
        cockpit_repository::CoordinationStore::open(&git, binding)
    } else {
        cockpit_repository::CoordinationStore::open_read_only(&git, binding)
    }
    .context("open candidate collaboration store")?;
    Ok(store)
}

fn read_json_file<T: DeserializeOwned>(path: &Path, description: &str) -> Result<T> {
    serde_json::from_slice(&fs::read(path).with_context(|| format!("read {description}"))?)
        .with_context(|| format!("parse {description}"))
}

fn coordination_output(
    store: &cockpit_repository::CoordinationStore,
    result: serde_json::Value,
) -> Result<()> {
    let projection = cockpit_repository::collaboration_projection(store)
        .context("refresh collaboration projection")?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "result": result,
            "projection": projection,
        }))?
    );
    Ok(())
}

fn run_coordination_command(
    command: WorkItemCoordinationCommand,
    runtime: &RuntimeContext,
) -> Result<()> {
    match command {
        WorkItemCoordinationCommand::Inspect { repo } => {
            let store = collaboration_store(&repo, runtime, false)?;
            let projection = cockpit_repository::collaboration_projection(&store)
                .context("inspect collaboration projection")?;
            println!("{}", serde_json::to_string_pretty(&projection)?);
        }
        WorkItemCoordinationCommand::Register { repo, input } => {
            let store = collaboration_store(&repo, runtime, true)?;
            let registration: WorktreeRegistration = read_json_file(&input, "registration")?;
            let result = store.register(registration).context("register Work Item")?;
            coordination_output(&store, serde_json::to_value(result)?)?;
        }
        WorkItemCoordinationCommand::ReportImpact { repo, input } => {
            let store = collaboration_store(&repo, runtime, true)?;
            let event: CoordinationEvent = read_json_file(&input, "impact event")?;
            let result = cockpit_repository::report_impact(&store, event)
                .context("report collaboration impact")?;
            coordination_output(&store, serde_json::to_value(result)?)?;
        }
        WorkItemCoordinationCommand::PublishOutcome {
            repo,
            id,
            generation,
            outcome_id,
        } => {
            let store = collaboration_store(&repo, runtime, true)?;
            let result = cockpit_repository::publish_outcome(&store, &id, generation, &outcome_id)
                .context("publish current Work Item outcome")?;
            coordination_output(&store, serde_json::to_value(result)?)?;
        }
        WorkItemCoordinationCommand::RequestPause { repo, input } => {
            let store = collaboration_store(&repo, runtime, true)?;
            let request: CoordinationRequest = read_json_file(&input, "coordination request")?;
            let result = cockpit_repository::request_safe_pause(&store, request)
                .context("request safe pause")?;
            coordination_output(&store, serde_json::to_value(result)?)?;
        }
        WorkItemCoordinationCommand::Acknowledge {
            repo,
            request_id,
            state,
        } => {
            let store = collaboration_store(&repo, runtime, true)?;
            let state: CoordinationRequestState =
                serde_json::from_value(serde_json::Value::String(state))
                    .context("parse coordination request state")?;
            let result = cockpit_repository::acknowledge_pause(&store, &request_id, state)
                .context("acknowledge safe pause")?;
            coordination_output(&store, serde_json::to_value(result)?)?;
        }
        WorkItemCoordinationCommand::Resume {
            repo,
            id,
            generation,
        } => {
            let store = collaboration_store(&repo, runtime, true)?;
            let result = cockpit_repository::resume_and_re_evaluate(&store, &id, generation)
                .context("resume and re-evaluate Work Item")?;
            coordination_output(&store, serde_json::to_value(result)?)?;
        }
        WorkItemCoordinationCommand::Recover {
            repo,
            event_id,
            consumer_work_item_id,
            consumer_generation,
        } => {
            let store = collaboration_store(&repo, runtime, true)?;
            let result = cockpit_repository::recover_impact(
                &store,
                &event_id,
                &consumer_work_item_id,
                consumer_generation,
            )
            .context("consume recovered impact")?;
            coordination_output(&store, serde_json::to_value(result)?)?;
        }
        WorkItemCoordinationCommand::CheckEnvironmentDrift { repo, input } => {
            let request: cockpit_protocol::EnvironmentDriftRequest =
                read_json_file(&input, "environment drift request")?;
            let git = GitRepository::discover(&repo).context("discover repository topology")?;
            let store = cockpit_repository::CoordinationStore::open_read_only(
                &git,
                collaboration_runtime(runtime),
            )
            .context("open candidate environment drift store read-only")?;
            let pending = store
                .environment_drift_pending(&request)
                .context("check Runtime-observed environment drift")?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({"pending": pending}))?
            );
        }
        WorkItemCoordinationCommand::RecordEnvironmentDrift { repo, input } => {
            let request: cockpit_protocol::EnvironmentDriftRequest =
                read_json_file(&input, "environment drift request")?;
            let git = GitRepository::discover(&repo).context("discover repository topology")?;
            let store =
                cockpit_repository::CoordinationStore::open(&git, collaboration_runtime(runtime))
                    .context("open candidate environment drift store for writing")?;
            let result = store
                .record_environment_drift(&request)
                .context("persist Runtime-observed environment drift")?;
            coordination_output(&store, serde_json::to_value(result)?)?;
        }
    }
    Ok(())
}

fn run_composition_command(
    repo: PathBuf,
    work_item_id: String,
    generation: u64,
    input_path: PathBuf,
    runtime: &RuntimeContext,
) -> Result<()> {
    let mut input: cockpit_verification::CompositionInput =
        read_json_file(&input_path, "composition input")?;
    let candidate = collaboration_runtime(runtime);
    if !input.binding.verifier.same_identity(&candidate) {
        anyhow::bail!(
            "unsupported_runtime_capability: composition verifier identity differs from current candidate"
        )
    }
    input.repository_root = repo.clone();
    let store = collaboration_store(&repo, runtime, true)?;
    let attempt =
        cockpit_repository::run_admitted_composition(&store, &work_item_id, generation, input)
            .context("run admitted composition")?;
    coordination_output(&store, serde_json::to_value(attempt)?)
}

#[cfg(test)]
fn record_ordinary_cleanup_command(
    repo: &Path,
    work_item_id: &str,
    runtime: &RuntimeContext,
) -> Result<serde_json::Value> {
    let receipt =
        cockpit_repository::record_ordinary_cleanup_with_runtime(repo, work_item_id, runtime)
            .context("record ordinary cleanup")?;
    serde_json::to_value(receipt).context("encode ordinary cleanup receipt")
}

fn main() {
    // Run the per-command verification supervisor on the process leader
    // thread so Linux subreaper ownership survives the CLI's normal worker
    // thread and remains available while it reaps adopted children.
    if std::env::args_os().nth(1).as_deref()
        == Some(std::ffi::OsStr::new("__verification-supervisor"))
    {
        if let Err(error) = cockpit_verification::run_verification_supervisor_stdio() {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    // The CLI's dispatch stack exceeds the Windows process default in
    // unoptimized builds. Keep parsing and command execution on an explicit,
    // bounded stack instead of relying on the platform linker default.
    let result = std::thread::Builder::new()
        .name("ai-cockpit-cli".into())
        .stack_size(4 * 1024 * 1024)
        .spawn(run)
        .map_err(|error| anyhow::anyhow!("start CLI worker: {error}"))
        .and_then(|worker| match worker.join() {
            Ok(result) => result,
            Err(payload) => std::panic::resume_unwind(payload),
        });

    if let Err(error) = result {
        if let Some(failure) = error.downcast_ref::<GatePlanCliFailure>() {
            eprintln!(
                "{}",
                serde_json::to_string(&failure.payload).expect("gate-plan failure serializes")
            );
            eprintln!("{}", failure.detail);
            std::process::exit(GATE_PLAN_FAILURE_EXIT_CODE);
        }
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    if std::env::args_os().nth(1).as_deref()
        == Some(std::ffi::OsStr::new("__verification-supervisor"))
    {
        return cockpit_verification::run_verification_supervisor_stdio()
            .map_err(anyhow::Error::msg);
    }
    if std::env::args_os().nth(1).as_deref()
        == Some(std::ffi::OsStr::new("__composition-supervisor"))
    {
        return cockpit_repository::run_composition_supervisor_stdio().map_err(anyhow::Error::msg);
    }
    if std::env::args_os().nth(1).as_deref()
        == Some(std::ffi::OsStr::new("__portable-source-observe"))
    {
        let arguments = std::env::args_os().collect::<Vec<_>>();
        anyhow::ensure!(
            arguments.len() == 3,
            "private source observer requires exactly one JSON argument"
        );
        let request = arguments[2]
            .to_str()
            .context("private source request must be UTF-8")?;
        let runtime =
            runtime_identity::load_current().context("load private source observer identity")?;
        let reply = github_evidence::source_worker(request, &runtime)?;
        println!("{}", serde_json::to_string(&reply)?);
        return Ok(());
    }
    if std::env::args_os().nth(1).as_deref()
        == Some(std::ffi::OsStr::new("__portable-native-publish"))
    {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct NativeRequest {
            schema_version: u32,
            repository_root: PathBuf,
            work_item_id: String,
            node_id: String,
            task_caller: Option<cockpit_protocol::TaskAuthorizationCaller>,
            publisher_nonce: String,
        }
        let args = std::env::args_os().collect::<Vec<_>>();
        anyhow::ensure!(
            args.len() == 3,
            "private native publisher requires one JSON argument"
        );
        let raw = args[2]
            .to_str()
            .context("native publisher request must be UTF-8")?;
        anyhow::ensure!(raw.len() <= 16 * 1024, "native publisher request too large");
        let request: NativeRequest =
            serde_json::from_str(raw).context("strict native publisher request")?;
        anyhow::ensure!(
            request.schema_version == 1,
            "unsupported native publisher request"
        );
        let runtime = runtime_identity::load_current()?;
        require_compatible(&request.repository_root, &runtime)?;
        let result = cockpit_repository::publish_declared_native_source_supervised(
            &request.repository_root,
            &request.work_item_id,
            &request.node_id,
            &request.publisher_nonce,
            &runtime,
            request.task_caller.as_ref(),
            Some(cockpit_verification::VerificationSupervisorLaunch::new(
                std::env::current_exe()?,
                vec!["__verification-supervisor".into()],
            )),
        )?;
        println!("{}", serde_json::to_string(&result)?);
        return Ok(());
    }
    let cli = Cli::parse();
    let task_caller: Option<cockpit_protocol::TaskAuthorizationCaller> = cli
        .task_caller
        .as_ref()
        .map(|path| read_json_file(path, "explicit task caller"))
        .transpose()?;

    if let CommandKind::IsolationManifest {
        root,
        output,
        paths_file,
        mask_ancestors_of,
    } = &cli.command
    {
        return run_isolation_manifest(
            root,
            output.as_ref(),
            paths_file.as_ref(),
            mask_ancestors_of.as_deref(),
        );
    }
    if let CommandKind::GatePlan {
        repo,
        manifest,
        base,
        head,
        stage,
        risk,
        contract,
        profile,
        receipt,
        validate_receipt,
    } = &cli.command
    {
        return run_gate_plan(GatePlanOptions {
            repo,
            manifest,
            base,
            head,
            stage,
            risk,
            contract: contract.as_ref(),
            profile: profile.as_deref(),
            receipt,
            validate_receipt: *validate_receipt,
        })
        .map_err(|error| anyhow::Error::new(GatePlanCliFailure::from_error(error)));
    }
    if let CommandKind::ReleasePlan { input, output } = &cli.command {
        return run_release_plan(input, output.as_deref());
    }
    let runtime_context = runtime_identity::load_current().context("load runtime identity")?;
    match cli.command {
        CommandKind::Inspect { repo, json: _ } => {
            let git = GitRepository::discover(&repo).context("discover repository")?;
            let snapshot = git.snapshot().context("create repository snapshot")?;
            let output = json!({
                "runtimeVersion": &runtime_context.runtime_version,
                "runtimeDigest": &runtime_context.runtime_digest,
                "protocolVersion": runtime_context.protocol_version,
                "repositoryRoot": snapshot.root,
                "gitRoot": snapshot.git_root,
                "head": snapshot.head,
                "changedPaths": snapshot.changed_paths,
                "gitCalls": snapshot.git_calls,
                "treeDigest": snapshot.tree_digest,
                "diffDigest": snapshot.diff_digest,
                "dependencyFingerprint": snapshot.dependency_fingerprint,
                "filesRead": snapshot.files_read,
                "filesHashed": snapshot.files_hashed,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        CommandKind::Preflight { repo, contract } => {
            require_compatible(&repo, &runtime_context)?;
            let result = match task_caller.as_ref() {
                Some(caller) => cockpit_repository::preflight_work_item_with_task_authorization(
                    &repo,
                    &contract,
                    &runtime_context,
                    caller,
                ),
                None => preflight_work_item_with_runtime_report(&repo, &contract, &runtime_context),
            }
            .context("evaluate and record preflight decision")?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        CommandKind::Observe { repo } => {
            let git = GitRepository::discover(&repo).context("discover repository")?;
            let snapshot = git.snapshot().context("create repository snapshot")?;
            let observation = cockpit_repository::observe(&snapshot.root, &snapshot)
                .context("observe repository")?;
            let (evolution, profile_update_proposal) =
                std::fs::read(snapshot.root.join(".ai/project.json"))
                    .ok()
                    .and_then(|bytes| {
                        serde_json::from_slice::<cockpit_repository::AttachedProfile>(&bytes)
                            .ok()
                            .map(|profile| cockpit_protocol::ProjectProfile {
                                profile_version: profile.profile_version,
                                repository_id: profile.repository_id,
                                tests: profile.tests,
                                build_systems: profile.build_systems,
                            })
                    })
                    .map(|profile| {
                        let evolution = cockpit_repository::classify_evolution(
                            &profile,
                            &observation,
                            &snapshot,
                        );
                        let proposal =
                            cockpit_repository::profile_update_proposal(&profile, &evolution);
                        (evolution, proposal)
                    })
                    .unwrap_or_default();
            let mut output = serde_json::to_value(observation)?;
            output["evolution"] = serde_json::to_value(evolution)?;
            output["profileUpdateProposal"] = serde_json::to_value(profile_update_proposal)?;
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        CommandKind::IsolationManifest { .. } => {
            unreachable!("handled before runtime identity load")
        }
        CommandKind::GatePlan { .. } => {
            unreachable!("handled before runtime identity load")
        }
        CommandKind::ReleasePlan { .. } => {
            unreachable!("handled before runtime identity load")
        }
        CommandKind::Attach { repo } => {
            let profile = attach(&repo).context("attach repository")?;
            println!("{}", serde_json::to_string_pretty(&profile)?);
        }
        CommandKind::Status { repo, json: _ } => {
            let repository_status =
                cockpit_repository::status_with_runtime(&repo, Some(&runtime_context))
                    .context("read repository status")?;
            let compatibility = cockpit_repository::compatibility_report(&repo, &runtime_context)
                .context("read repository compatibility")?;
            let mut output = serde_json::to_value(repository_status)?;
            output["compatibility"] = serde_json::to_value(compatibility)?;
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        CommandKind::Compatibility { repo } => {
            let compatibility = cockpit_repository::compatibility_report(&repo, &runtime_context)
                .context("read repository compatibility")?;
            println!("{}", serde_json::to_string_pretty(&compatibility)?);
        }
        CommandKind::Migrate { command } => match command {
            MigrateCommand::Plan { repo } => {
                let plan =
                    cockpit_repository::migration_plan_with_runtime(&repo, Some(&runtime_context))
                        .context("plan repository migration")?;
                println!("{}", serde_json::to_string_pretty(&plan)?);
            }
            MigrateCommand::Apply { repo, approved } => {
                if !approved {
                    anyhow::bail!(
                        "migration changes repository state; rerun with --approved after reviewing migrate plan"
                    )
                }
                let receipt = cockpit_repository::apply_migration(&repo, &runtime_context)
                    .context("apply repository migration")?;
                println!("{}", serde_json::to_string_pretty(&receipt)?);
            }
        },
        CommandKind::Start {
            repo,
            id,
            intent,
            goal,
            scope,
            out_of_scope,
            risk,
            authority,
            acceptance,
            required_evidence,
            verification,
            scenario_coverage,
            development_review,
            task_authorization,
            prepare,
            source,
        } => {
            require_compatible(&repo, &runtime_context)?;
            let options = WorkItemStartOptions {
                task_authorization: task_authorization
                    .map(|path| -> anyhow::Result<_> {
                        Ok(serde_json::from_slice(&fs::read(&path)?)?)
                    })
                    .transpose()?,
                out_of_scope,
                risk,
                authority,
                acceptance_criteria: acceptance,
                required_evidence_classes: required_evidence,
                verification_commands: verification,
                development_review: development_review
                    .map(|path| -> anyhow::Result<_> {
                        Ok(serde_json::from_slice(
                            &std::fs::read(&path).with_context(|| {
                                format!("read development review policy {}", path.display())
                            })?,
                        )?)
                    })
                    .transpose()?,
                scenario_coverage: scenario_coverage
                    .map(|path| -> anyhow::Result<_> {
                        Ok(serde_json::from_slice(
                            &std::fs::read(&path).with_context(|| {
                                format!("read initial scenario plan {}", path.display())
                            })?,
                        )?)
                    })
                    .transpose()?
                    .unwrap_or_default(),
            };
            let report = cockpit_repository::start_work_item_with_preparation(
                &repo,
                &cockpit_repository::WorkItemStartDeclaration {
                    work_item_id: &id,
                    intent: &intent,
                    goal: &goal,
                    scope: &scope,
                    options: &options,
                    sources: &source,
                    prepare,
                },
                &runtime_context,
            )
            .context("start work item with complete declaration")?;
            let mut output = if prepare {
                report.clone()
            } else {
                report["start"].clone()
            };
            if !prepare && !source.is_empty() {
                output["sourceAmendment"] = report["sourceAmendment"].clone();
            }
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        CommandKind::Checkpoint { repo, id } => {
            require_compatible(&repo, &runtime_context)?;
            let receipt = cockpit_repository::checkpoint_work_item_with_caller(
                &repo,
                &id,
                &runtime_context,
                task_caller.as_ref(),
            )
            .context("checkpoint work item")?;
            println!("{}", serde_json::to_string_pretty(&receipt)?);
        }
        CommandKind::Finish {
            repo,
            id,
            json,
            language,
        } => {
            require_compatible(&repo, &runtime_context)?;
            let receipt = match cockpit_repository::finish_work_item_with_caller(
                &repo,
                &id,
                &runtime_context,
                task_caller.as_ref(),
            ) {
                Ok(receipt) => receipt,
                Err(error) => {
                    if !json {
                        emit_blocked_lifecycle_handoff(
                            &repo,
                            &id,
                            &runtime_context,
                            language.as_deref(),
                        );
                    }
                    return Err(error).context("finish work item");
                }
            };
            print_lifecycle_result(
                &repo,
                &id,
                &receipt,
                &runtime_context,
                json,
                false,
                language.as_deref(),
            )?;
        }
        CommandKind::Archive {
            repo,
            id,
            json,
            language,
        } => {
            require_compatible(&repo, &runtime_context)?;
            let receipt = cockpit_repository::archive_work_item_with_caller(
                &repo,
                &id,
                &runtime_context,
                task_caller.as_ref(),
            )
            .context("archive work item")?;
            print_lifecycle_result(
                &repo,
                &id,
                &receipt,
                &runtime_context,
                json,
                true,
                language.as_deref(),
            )?;
        }
        CommandKind::ArchiveHistorical {
            repo,
            id,
            json,
            language,
        } => {
            require_compatible(&repo, &runtime_context)?;
            let receipt = archive_historical_work_item_with_runtime(&repo, &id, &runtime_context)
                .context("archive historical Work Item")?;
            print_lifecycle_result(
                &repo,
                &id,
                &receipt,
                &runtime_context,
                json,
                true,
                language.as_deref(),
            )?;
        }
        CommandKind::Close {
            repo,
            id,
            human_decision,
            actor,
            authority_source,
            reason,
            evidence_ref,
            policy_ref,
            decided_at,
            resume_condition,
            json,
            language,
        } => {
            require_compatible(&repo, &runtime_context)?;
            let receipt = if actor.is_some()
                || authority_source.is_some()
                || reason.is_some()
                || !evidence_ref.is_empty()
                || !policy_ref.is_empty()
                || decided_at.is_some()
                || resume_condition.is_some()
            {
                let decision = HumanDecision {
                    decision: human_decision,
                    actor: actor.ok_or_else(|| {
                        anyhow::anyhow!("--actor is required with structured close fields")
                    })?,
                    authority_source: authority_source.ok_or_else(|| {
                        anyhow::anyhow!(
                            "--authority-source is required with structured close fields"
                        )
                    })?,
                    reason: reason.ok_or_else(|| {
                        anyhow::anyhow!("--reason is required with structured close fields")
                    })?,
                    evidence_refs: evidence_ref,
                    policy_refs: policy_ref,
                    decided_at: decided_at.ok_or_else(|| {
                        anyhow::anyhow!("--decided-at is required with structured close fields")
                    })?,
                    resume_condition,
                };
                close_work_item_with_structured_decision_and_runtime(
                    &repo,
                    &id,
                    &decision,
                    &runtime_context,
                )
                .context("close work item with structured decision")?
            } else {
                close_work_item_with_decision_and_runtime(
                    &repo,
                    &id,
                    &human_decision,
                    &runtime_context,
                )
                .context("close work item")?
            };
            print_lifecycle_result(
                &repo,
                &id,
                &receipt,
                &runtime_context,
                json,
                false,
                language.as_deref(),
            )?;
        }
        command @ CommandKind::Verify { .. } => {
            run_verification_command(command, runtime_context, task_caller)?;
        }
        CommandKind::Gate {
            repo,
            contract,
            stage,
            runner,
            base_revision,
            report,
        } => {
            require_compatible(&repo, &runtime_context)?;
            let stage = match stage.as_str() {
                // The Python CI route uses the provider-facing spelling;
                // Rust keeps `pr` as the protocol enum spelling.
                "pull_request" => VerificationStage::PullRequest,
                other => VerificationStage::parse(other).map_err(|error| anyhow::anyhow!(error))?,
            };
            let quality = cockpit_repository::evaluate_contract_quality_gate(
                &repo,
                &contract,
                stage,
                &runner,
                base_revision.as_deref(),
                &runtime_context,
            )
            .context("evaluate read-only Contract quality gate")?;
            let serialized = serde_json::to_string_pretty(&quality)?;
            if let Some(report) = report {
                let report = if report.is_absolute() {
                    report
                } else {
                    repo.join(report)
                };
                if report
                    .components()
                    .any(|component| component == std::path::Component::Normal(".ai".as_ref()))
                {
                    anyhow::bail!("CI gate report must not be written under .ai");
                }
                if let Some(parent) = report.parent() {
                    std::fs::create_dir_all(parent).context("create CI gate report parent")?;
                }
                std::fs::write(&report, format!("{serialized}\n"))
                    .context("write CI gate report")?;
            }
            println!("{serialized}");
            if quality.state != "passed" {
                anyhow::bail!(
                    "read-only Contract quality gate is {} (decisionState={})",
                    quality.state,
                    quality.decision_state
                );
            }
        }
        CommandKind::GateReport {
            repo,
            report,
            route_receipt,
        } => {
            require_compatible(&repo, &runtime_context)?;
            let report = cockpit_repository::validate_contract_quality_gate_report(
                &repo,
                &report,
                &route_receipt,
            )
            .context("validate Contract quality-gate report")?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::Evidence { command } => match command {
            EvidenceCommand::Bundle { command } => {
                github_evidence::run_portable(command, &runtime_context, task_caller.as_ref())?;
            }
            EvidenceCommand::Github { command } => {
                github_evidence::run(command, &runtime_context, task_caller.as_ref())?;
            }
            EvidenceCommand::Import {
                repo,
                work_item,
                metadata,
                raw,
            } => {
                require_compatible(&repo, &runtime_context)?;
                let evidence: DelegatedEvidence = serde_json::from_slice(
                    &std::fs::read(&metadata).context("read delegated evidence metadata")?,
                )
                .context("parse delegated evidence metadata")?;
                let raw_bytes = std::fs::read(&raw).context("read delegated raw evidence")?;
                let receipt = cockpit_repository::import_delegated_evidence(
                    &repo,
                    &work_item,
                    &evidence,
                    &raw_bytes,
                    &runtime_context,
                )
                .context("import delegated evidence")?;
                println!("{}", serde_json::to_string_pretty(&receipt)?);
            }
            EvidenceCommand::Observe {
                repo,
                work_item,
                metadata,
                raw,
            } => {
                require_compatible(&repo, &runtime_context)?;
                let evidence: DelegatedEvidence = serde_json::from_slice(
                    &std::fs::read(&metadata).context("read delegated evidence metadata")?,
                )
                .context("parse delegated evidence metadata")?;
                let raw_bytes = std::fs::read(&raw).context("read delegated raw evidence")?;
                let observation = cockpit_repository::import_delegated_evidence_observation(
                    &repo,
                    &work_item,
                    &evidence,
                    &raw_bytes,
                    &runtime_context,
                )
                .context("record delegated evidence observation")?;
                println!("{}", serde_json::to_string_pretty(&observation)?);
            }
            EvidenceCommand::List { repo, work_item } => {
                require_compatible(&repo, &runtime_context)?;
                let receipts = cockpit_repository::list_delegated_evidence(&repo, &work_item)
                    .context("list delegated evidence")?;
                println!("{}", serde_json::to_string_pretty(&receipts)?);
            }
            EvidenceCommand::ListObservations { repo, work_item } => {
                require_compatible(&repo, &runtime_context)?;
                let observations =
                    cockpit_repository::list_delegated_evidence_observations(&repo, &work_item)
                        .context("list delegated evidence observations")?;
                println!("{}", serde_json::to_string_pretty(&observations)?);
            }
            EvidenceCommand::Policy {
                repo,
                work_item,
                classification,
                persistence,
                retention_days,
                expires_at,
                disposal_action,
            } => {
                require_compatible(&repo, &runtime_context)?;
                let retention = EvidenceRetention {
                    classification: parse_classification(&classification)?,
                    persistence: parse_persistence(&persistence)?,
                    retention_days,
                    expires_at,
                    disposal_action,
                };
                let policy = cockpit_repository::set_evidence_retention_policy(
                    &repo,
                    &work_item,
                    retention,
                    &runtime_context,
                )
                .context("set evidence retention policy")?;
                println!("{}", serde_json::to_string_pretty(&policy)?);
            }
            EvidenceCommand::PurgePlan {
                repo,
                now_epoch_seconds,
            } => {
                require_compatible(&repo, &runtime_context)?;
                let now_epoch_seconds = now_epoch_seconds.unwrap_or_else(|| {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|duration| duration.as_secs())
                        .unwrap_or_default()
                });
                let plan = cockpit_repository::evidence_purge_plan(&repo, now_epoch_seconds)
                    .context("create deterministic evidence purge plan")?;
                println!("{}", serde_json::to_string_pretty(&plan)?);
            }
        },
        CommandKind::Audit { command } => match command {
            AuditCommand::CognitiveBenefit {
                repo,
                binary,
                check,
            } => cognitive_benefit::run(&repo, binary.as_deref(), check)?,
            AuditCommand::Query { repo, filters } => {
                require_compatible(&repo, &runtime_context)?;
                let page = cockpit_repository::query_audit_events(
                    &repo,
                    &runtime_context,
                    &filters.into(),
                )
                .context("query audit events")?;
                println!("{}", serde_json::to_string_pretty(&page)?);
            }
            AuditCommand::Export {
                repo,
                output,
                filters,
            } => {
                require_compatible(&repo, &runtime_context)?;
                let filters: cockpit_protocol::AuditQueryFilters = filters.into();
                let bytes = if filters == cockpit_protocol::AuditQueryFilters::default() {
                    serde_json::to_vec_pretty(
                        &cockpit_repository::export_audit_events(&repo, &runtime_context)
                            .context("export audit events")?,
                    )?
                } else {
                    serde_json::to_vec_pretty(
                        &cockpit_repository::export_audit_events_filtered(
                            &repo,
                            &runtime_context,
                            &filters,
                        )
                        .context("export filtered audit events")?,
                    )?
                };
                if let Some(output) = output {
                    if let Some(parent) = output.parent() {
                        std::fs::create_dir_all(parent).context("create audit export parent")?;
                    }
                    match std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&output)
                    {
                        Ok(mut file) => {
                            use std::io::Write;
                            file.write_all(&bytes).context("write audit export")?;
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                            let existing =
                                std::fs::read(&output).context("read existing audit export")?;
                            let same_page = serde_json::from_slice::<
                                cockpit_protocol::AuditQueryPage,
                            >(&existing)
                            .ok()
                            .zip(
                                serde_json::from_slice::<cockpit_protocol::AuditQueryPage>(&bytes)
                                    .ok(),
                            )
                            .is_some_and(|(mut old, current)| {
                                old.as_of = current.as_of.clone();
                                old == current
                            });
                            if existing != bytes && !same_page {
                                anyhow::bail!(
                                    "audit export target already exists with different bytes"
                                )
                            }
                        }
                        Err(error) => return Err(error).context("create audit export"),
                    }
                }
                println!("{}", String::from_utf8(bytes)?);
            }
        },
        CommandKind::WorkItem { command } => match command {
            WorkItemCommand::New { repo, id, mode } => {
                require_compatible(&repo, &runtime_context)?;
                let receipt =
                    scaffold_work_item(&repo, &id, &mode).context("create Work Item scaffold")?;
                println!("Work Item scaffold created.");
                println!("\nKnown facts:");
                println!(
                    "  repositoryId              resolved\n  baseRevision              resolved\n  projectProfileDigest      resolved\n  repositorySnapshotDigest resolved"
                );
                println!("\nHuman input required:");
                for field in &receipt.human_input_required {
                    println!("  {field}");
                }
                if let Some(advisory) = receipt.start_advisory.as_ref() {
                    println!("\nStart cleanup advisory: {}", advisory.classification);
                    for warning in &advisory.warnings {
                        println!("  warning: {warning}");
                    }
                    for conflict in &advisory.conflicts {
                        println!("  conflict: {conflict}");
                    }
                    for branch in &advisory.remote_branches {
                        println!("  remote branch: {} ({})", branch.name, branch.head);
                    }
                    for unknown in &advisory.unknowns {
                        println!("  unknown: {unknown}");
                    }
                    println!("\nNext actions:");
                    for action in &advisory.next_actions {
                        println!("  {action}");
                    }
                }
                println!("\nState: {}", receipt.state);
                println!("\n{}", serde_json::to_string_pretty(&receipt)?);
            }
            WorkItemCommand::ActivationRecovery { command } => match command {
                WorkItemActivationRecoveryCommand::Plan { repo, id } => {
                    require_compatible(&repo, &runtime_context)?;
                    let request =
                        cockpit_repository::plan_activation_recovery(&repo, &id, &runtime_context)
                            .context("plan activation present-baseline recovery")?;
                    println!("{}", serde_json::to_string_pretty(&request)?);
                }
                WorkItemActivationRecoveryCommand::Record { repo, id, input } => {
                    require_compatible(&repo, &runtime_context)?;
                    let decision: cockpit_protocol::ActivationRecoveryDecision =
                        serde_json::from_slice(&fs::read(&input).with_context(|| {
                            format!("read recovery decision {}", input.display())
                        })?)
                        .context("decode strict activation recovery decision")?;
                    let record = cockpit_repository::record_activation_recovery(
                        &repo,
                        &id,
                        &decision,
                        &runtime_context,
                    )
                    .context("record activation present-baseline recovery")?;
                    println!("{}", serde_json::to_string_pretty(&record)?);
                }
            },
            WorkItemCommand::RevalidateAmendment { repo, id, reason } => {
                require_compatible(&repo, &runtime_context)?;
                let mut record =
                    cockpit_repository::revalidate_contract_amendment(&repo, &id, &reason)
                        .context("revalidate amended Contract")?;
                record["nextAction"] =
                    next_admitted_work_item_action(&repo, &id, &runtime_context)?;
                println!("{}", serde_json::to_string_pretty(&record)?);
            }
            WorkItemCommand::Amend {
                repo,
                id,
                input,
                reason,
                request,
            } => {
                require_compatible(&repo, &runtime_context)?;
                let mut record = match (input, reason, request) {
                    (Some(input), Some(reason), None) => {
                        let input: serde_json::Value = serde_json::from_slice(
                            &std::fs::read(&input).context("read Contract amendment input")?,
                        )
                        .context("parse Contract amendment input")?;
                        cockpit_repository::amend_work_item_contract_with_runtime(
                            &repo,
                            &id,
                            &input,
                            &reason,
                            &runtime_context,
                        )
                        .context("apply legacy additive Contract amendment")?
                    }
                    (None, None, Some(request)) => {
                        let request: cockpit_protocol::ContractAmendmentRequest =
                            read_json_file(&request, "typed Contract amendment request")?;
                        serde_json::to_value(
                            cockpit_repository::apply_work_item_contract_amendment(
                                &repo,
                                &id,
                                &request,
                                &runtime_context,
                            )
                            .context("apply typed Contract amendment")?,
                        )?
                    }
                    _ => anyhow::bail!(
                        "select exactly one amendment form: --request, or both --input and --reason"
                    ),
                };
                record["nextAction"] =
                    next_admitted_work_item_action(&repo, &id, &runtime_context)?;
                println!("{}", serde_json::to_string_pretty(&record)?);
            }
            WorkItemCommand::Amendments { repo, id } => {
                require_compatible(&repo, &runtime_context)?;
                let history = cockpit_repository::read_work_item_contract_amendments(&repo, &id)
                    .context("read Contract amendment history")?;
                println!("{}", serde_json::to_string_pretty(&history)?);
            }
            WorkItemCommand::RevalidateArchived {
                repo,
                id,
                successor,
                reason,
                actor,
                authority_source,
                resume_condition,
                evidence_ref,
                policy_ref,
            } => {
                require_compatible(&repo, &runtime_context)?;
                let request = cockpit_repository::ArchivedContractRevalidationRequest {
                    successor_work_item_id: successor,
                    reason,
                    actor,
                    authority_source,
                    resume_condition,
                    evidence_refs: evidence_ref,
                    policy_refs: policy_ref,
                };
                let record = cockpit_repository::revalidate_archived_work_item_with_runtime(
                    &repo,
                    &id,
                    &request,
                    &runtime_context,
                )
                .context("record archived Contract revalidation")?;
                println!("{}", serde_json::to_string_pretty(&record)?);
            }
            WorkItemCommand::Approach { repo, id } => {
                require_compatible(&repo, &runtime_context)?;
                let approach = cockpit_repository::implementation_approach(&repo, &id)
                    .context("derive implementation approach")?;
                println!("{}", serde_json::to_string_pretty(&approach)?);
            }
            WorkItemCommand::Outcome { repo, query } => {
                require_compatible(&repo, &runtime_context)?;
                if query.delivery && query.display_timezone.is_some() {
                    anyhow::bail!("displayTimezone is unavailable for immutable archive delivery");
                }
                let language = query.language.map(|language| language.as_str().to_owned());
                if query.delivery {
                    let prepared = prepare_archive_outcome_delivery(
                        &repo,
                        &query.id,
                        &runtime_context,
                        output_language(language.as_deref()),
                    )
                    .context("prepare archived Outcome delivery")?;
                    let result = cockpit_agent::deliver_prepared_outcome(
                        cockpit_agent::OutcomeDeliveryApplicationRequest {
                            repository: &repo,
                            work_item_id: &query.id,
                            delivery: prepared,
                            supplied_progress: None,
                        },
                    )
                    .map_err(|error| anyhow::anyhow!(error))?;
                    let collaboration = cockpit_repository::collaboration_outcome_projection(
                        &repo,
                        &query.id,
                        &runtime_context,
                    );
                    let mut output = serde_json::to_value(&result.delivery)?;
                    output["collaboration"] = serde_json::to_value(&collaboration)?;
                    output["assistantMessageEvents"] = serde_json::to_value(
                        cockpit_agent::assistant_message_events(&result.delivery),
                    )?;
                    output["deliveryReport"] = result.delivery_report_json()?;
                    output["hostDeliveryMode"] = json!(result.host_delivery_mode);
                    output["hostDisplayConfirmation"] = json!(result.host_display_confirmation);
                    if let Some(count) = result.returned_segment_events {
                        output["returnedSegmentEvents"] = json!(count);
                    }
                    if query.json {
                        println!("{}", serde_json::to_string_pretty(&output)?);
                    } else {
                        println!("{}", result.handoff);
                    }
                } else {
                    let input = cockpit_repository::outcome_render_input_with_runtime(
                        &repo,
                        &query.id,
                        &runtime_context,
                    )
                    .context("read Work Item outcome")?;
                    let collaboration = cockpit_repository::collaboration_outcome_projection(
                        &repo,
                        &query.id,
                        &runtime_context,
                    );
                    if query.json {
                        let mut output = serde_json::to_value(&input.outcome)?;
                        output["collaboration"] = serde_json::to_value(&collaboration)?;
                        if let Some(zone) = query.display_timezone.as_deref() {
                            let language = output_language(language.as_deref());
                            output["humanHandoff"] =
                                cockpit_repository::render_human_outcome_with_timezone(
                                    &repo,
                                    &input,
                                    &runtime_context,
                                    language,
                                    outcome_repository_view(query.view.as_str()),
                                    zone,
                                )?
                                .into();
                            output["displayTimezone"] = zone.into();
                        }
                        println!("{}", serde_json::to_string_pretty(&output)?);
                    } else {
                        let language = output_language(language.as_deref());
                        let handoff = if let Some(zone) = query.display_timezone.as_deref() {
                            cockpit_repository::render_human_outcome_with_timezone(
                                &repo,
                                &input,
                                &runtime_context,
                                language,
                                outcome_repository_view(query.view.as_str()),
                                zone,
                            )?
                        } else {
                            cockpit_repository::render_human_outcome_with_view(
                                &input,
                                language,
                                outcome_repository_view(query.view.as_str()),
                            )
                        };
                        println!(
                            "{}\n{}",
                            handoff,
                            cockpit_repository::render_collaboration_outcome(
                                &collaboration,
                                language
                            )
                        );
                    }
                }
            }
            WorkItemCommand::Usage { command } => match command {
                WorkItemUsageCommand::Import { repo, input } => {
                    require_compatible(&repo, &runtime_context)?;
                    let request: cockpit_protocol::UsageImportRequest = serde_json::from_slice(
                        &std::fs::read(&input)
                            .with_context(|| format!("read usage import {}", input.display()))?,
                    )
                    .context("parse strict UsageImportRequest")?;
                    let receipt = cockpit_repository::import_work_item_usage(
                        &repo,
                        &request,
                        &runtime_context,
                    )?;
                    println!("{}", serde_json::to_string_pretty(&receipt)?);
                }
                WorkItemUsageCommand::Record { repo, input } => {
                    require_compatible(&repo, &runtime_context)?;
                    let request: cockpit_protocol::UsageRecordRequest = serde_json::from_slice(
                        &std::fs::read(&input).context("read usage request")?,
                    )
                    .context("parse strict UsageRecordRequest")?;
                    let receipt = cockpit_repository::record_work_item_usage(
                        &repo,
                        &request,
                        &runtime_context,
                    )
                    .context("record Work Item usage")?;
                    println!("{}", serde_json::to_string_pretty(&receipt)?);
                }
            },
            WorkItemCommand::MaterialReview { command } => match command {
                WorkItemMaterialReviewCommand::Plan { repo, id } => {
                    require_compatible(&repo, &runtime_context)?;
                    let request = cockpit_repository::plan_work_item_material_review(&repo, &id)
                        .context("plan Work Item material review")?;
                    println!("{}", serde_json::to_string_pretty(&request)?);
                }
                WorkItemMaterialReviewCommand::Record { repo, id, input } => {
                    require_compatible(&repo, &runtime_context)?;
                    let request: cockpit_protocol::MaterialInspectionReviewDecisionInput =
                        serde_json::from_slice(
                            &fs::read(&input).context("read material review decision input")?,
                        )
                        .context("parse strict MaterialInspectionReviewDecisionInput")?;
                    let receipt = cockpit_repository::record_work_item_material_review_with_caller(
                        &repo,
                        &id,
                        &request,
                        &runtime_context,
                        task_caller.as_ref(),
                    )
                    .context("record Work Item material review decision")?;
                    println!("{}", serde_json::to_string_pretty(&receipt)?);
                }
            },
            WorkItemCommand::AuthorizationManifest { command } => match command {
                WorkItemAuthorizationManifestCommand::Plan { repo, input } => {
                    require_compatible(&repo, &runtime_context)?;
                    let request: cockpit_protocol::TaskAuthorizationBatchRequest =
                        read_json_file(&input, "authorization batch request")?;
                    request
                        .validate()
                        .map_err(anyhow::Error::msg)
                        .context("validate authorization batch request")?;
                    let manifest = cockpit_repository::prepare_task_authorization_batch_manifest(
                        &repo,
                        &request,
                        &runtime_context,
                    )
                    .context("prepare read-only authorization batch manifest")?;
                    println!("{}", serde_json::to_string_pretty(&manifest)?);
                }
                WorkItemAuthorizationManifestCommand::Review {
                    repo,
                    manifest,
                    review,
                } => {
                    require_compatible(&repo, &runtime_context)?;
                    let manifest: cockpit_protocol::TaskAuthorizationBatchManifest =
                        read_json_file(&manifest, "authorization batch manifest")?;
                    manifest
                        .validate()
                        .map_err(anyhow::Error::msg)
                        .context("validate authorization batch manifest")?;
                    let review: cockpit_protocol::TaskAuthorizationBatchReview =
                        read_json_file(&review, "authorization batch review")?;
                    let receipt = cockpit_repository::record_task_authorization_batch_review(
                        &repo,
                        &manifest,
                        &review,
                        &runtime_context,
                    )
                    .context("record exact authorization batch review")?;
                    println!("{}", serde_json::to_string_pretty(&receipt)?);
                }
                WorkItemAuthorizationManifestCommand::Start { repo, input } => {
                    require_compatible(&repo, &runtime_context)?;
                    let request: cockpit_protocol::TaskAuthorizationBatchStartRequest =
                        read_json_file(&input, "authorization batch start request")?;
                    let event = cockpit_repository::reserve_task_authorization_batch_start(
                        &repo,
                        &request,
                        &runtime_context,
                    )
                    .context("reserve authorization batch member start")?;
                    println!("{}", serde_json::to_string_pretty(&event)?);
                }
                WorkItemAuthorizationManifestCommand::Check { repo, input } => {
                    require_compatible(&repo, &runtime_context)?;
                    let request: cockpit_protocol::TaskAuthorizationBatchCheckRequest =
                        read_json_file(&input, "authorization batch check request")?;
                    let event = cockpit_repository::reserve_task_authorization_batch_check_attempt(
                        &repo,
                        &request,
                        &runtime_context,
                    )
                    .context("reserve authorization batch check attempt")?;
                    println!("{}", serde_json::to_string_pretty(&event)?);
                }
                WorkItemAuthorizationManifestCommand::Outcome { repo, input } => {
                    require_compatible(&repo, &runtime_context)?;
                    let request: cockpit_protocol::TaskAuthorizationBatchOutcomeRequest =
                        read_json_file(&input, "authorization batch outcome request")?;
                    let event = cockpit_repository::record_task_authorization_batch_check_outcome(
                        &repo,
                        &request,
                        &runtime_context,
                    )
                    .context("record authorization batch check outcome")?;
                    println!("{}", serde_json::to_string_pretty(&event)?);
                }
                WorkItemAuthorizationManifestCommand::Inspect { repo, input } => {
                    require_compatible(&repo, &runtime_context)?;
                    let request: cockpit_protocol::TaskAuthorizationBatchInspectionRequest =
                        read_json_file(&input, "authorization batch inspection request")?;
                    let event = cockpit_repository::inspect_task_authorization_batch_check_outcome(
                        &repo,
                        &request,
                        &runtime_context,
                    )
                    .context("inspect authorization batch check outcome")?;
                    println!("{}", serde_json::to_string_pretty(&event)?);
                }
                WorkItemAuthorizationManifestCommand::Terminate { repo, input } => {
                    require_compatible(&repo, &runtime_context)?;
                    let request: cockpit_protocol::TaskAuthorizationBatchTerminationRequest =
                        read_json_file(&input, "authorization batch termination request")?;
                    let event = cockpit_repository::terminate_task_authorization_batch_member(
                        &repo,
                        &request,
                        &runtime_context,
                    )
                    .context("terminate authorization batch member")?;
                    println!("{}", serde_json::to_string_pretty(&event)?);
                }
                WorkItemAuthorizationManifestCommand::Ledger {
                    repo,
                    manifest_digest,
                    work_item_id,
                } => {
                    require_compatible(&repo, &runtime_context)?;
                    let manifest_digest: cockpit_core::Digest =
                        serde_json::from_value(json!(manifest_digest))
                            .context("parse authorization batch manifest digest")?;
                    let ledger = cockpit_repository::inspect_task_authorization_batch_member(
                        &repo,
                        &manifest_digest,
                        &work_item_id,
                        &runtime_context,
                    )
                    .context("inspect authorization batch member ledger")?;
                    println!("{}", serde_json::to_string_pretty(&ledger)?);
                }
            },
            WorkItemCommand::ReconcileArtifacts { repo, id } => {
                require_compatible(&repo, &runtime_context)?;
                let receipt = cockpit_repository::reconcile_active_artifacts(&repo, &id)
                    .context("reconcile active Work Item artifacts")?;
                println!("{}", serde_json::to_string_pretty(&receipt)?);
            }
            WorkItemCommand::FinalizePlan { repo, id, input } => {
                require_compatible(&repo, &runtime_context)?;
                let bytes = std::fs::read(&input).context("read resource finalization context")?;
                let context: cockpit_protocol::ResourceFinalizationContext =
                    serde_json::from_slice(&bytes)
                        .context("parse resource finalization context")?;
                let plan = cockpit_repository::plan_resource_finalization_with_caller(
                    &repo,
                    &id,
                    &context,
                    &runtime_context,
                    task_caller.as_ref(),
                )
                .context("plan resource finalization")?;
                println!("{}", serde_json::to_string_pretty(&plan)?);
            }
            WorkItemCommand::Finalize { repo, id, input } => {
                require_compatible(&repo, &runtime_context)?;
                let receipt = cockpit_repository::record_resource_finalization_with_caller(
                    &repo,
                    &id,
                    &input,
                    &runtime_context,
                    task_caller.as_ref(),
                )
                .context("record resource finalization")?;
                println!("{}", serde_json::to_string_pretty(&receipt)?);
            }
            WorkItemCommand::FinalizeVerify { repo, id } => {
                require_compatible(&repo, &runtime_context)?;
                let result = cockpit_repository::verify_resource_finalization_with_caller(
                    &repo,
                    &id,
                    &runtime_context,
                    task_caller.as_ref(),
                )
                .context("verify resource finalization")?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            WorkItemCommand::OrdinaryCleanup { repo, id } => {
                require_compatible(&repo, &runtime_context)?;
                let result = serde_json::to_value(
                    cockpit_repository::record_ordinary_cleanup_with_caller(
                        &repo,
                        &id,
                        &runtime_context,
                        task_caller.as_ref(),
                    )
                    .context("record ordinary cleanup")?,
                )?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            WorkItemCommand::FinalizeRecovery { repo, id, input } => {
                require_compatible(&repo, &runtime_context)?;
                let result =
                    cockpit_repository::record_historical_finalization_recovery_with_caller(
                        &repo,
                        &id,
                        &input,
                        &runtime_context,
                        task_caller.as_ref(),
                    )
                    .context("record historical finalization recovery")?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            WorkItemCommand::FinalizeRecoveryPlan {
                repo,
                id,
                merge_commit,
            } => {
                require_compatible(&repo, &runtime_context)?;
                let result = cockpit_repository::historical_finalization_recovery_plan(
                    &repo,
                    &id,
                    &runtime_context,
                    merge_commit.as_deref(),
                )
                .context("plan historical finalization recovery")?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            WorkItemCommand::LegacyCloseoutSourcePlan {
                repo,
                source_repo,
                id,
                input,
            } => {
                require_compatible(&repo, &runtime_context)?;
                let value = cockpit_repository::read_legacy_closeout_source_input(&input)
                    .context("read bounded legacy source request")?;
                let request: cockpit_protocol::LegacyCloseoutSourceRequest =
                    serde_json::from_value(value).context("parse strict legacy source request")?;
                let result = cockpit_repository::plan_legacy_closeout_source(
                    &repo,
                    source_repo.as_deref(),
                    &id,
                    &request,
                    &runtime_context,
                )
                .context("plan legacy closeout source")?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            WorkItemCommand::LegacyCloseoutSourceRecord { repo, id, input } => {
                require_compatible(&repo, &runtime_context)?;
                let value = cockpit_repository::read_legacy_closeout_source_input(&input)
                    .context("read bounded original legacy source plan")?;
                let plan: cockpit_protocol::LegacyCloseoutSourcePlan =
                    serde_json::from_value(value).context("parse strict legacy source plan")?;
                let result = cockpit_repository::record_legacy_closeout_source(
                    &repo,
                    &id,
                    &plan,
                    &runtime_context,
                )
                .context("record legacy closeout source")?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            WorkItemCommand::CloseoutRecoveryPlan {
                repo,
                source_repo,
                id,
            } => {
                require_compatible(&repo, &runtime_context)?;
                require_compatible(&source_repo, &runtime_context)?;
                let result = cockpit_repository::plan_cross_checkout_closeout_recovery(
                    &repo,
                    &source_repo,
                    &id,
                    &runtime_context,
                )
                .context("plan cross-checkout Work Item closeout recovery")?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            WorkItemCommand::CloseoutRecover {
                repo,
                source_repo,
                id,
            } => {
                require_compatible(&repo, &runtime_context)?;
                require_compatible(&source_repo, &runtime_context)?;
                let result = cockpit_repository::recover_cross_checkout_closeout(
                    &repo,
                    &source_repo,
                    &id,
                    &runtime_context,
                )
                .context("recover cross-checkout Work Item closeout")?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            WorkItemCommand::Status {
                repo,
                id,
                all,
                json,
                authorization_request,
            } => {
                require_compatible(&repo, &runtime_context)?;
                if all {
                    let index = cockpit_repository::work_item_status_index_with_runtime(
                        &repo,
                        &runtime_context,
                    )
                    .context("read all Work Item statuses")?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&index)?);
                    } else {
                        println!("Status: all Work Items");
                        println!("Repository: {}", index.repository_id);
                        println!("Snapshot: {}", index.snapshot_digest);
                        println!("Index digest: {}", index.index_digest);
                        println!("Items: {}", index.items.len());
                        println!(
                            "Counts: green={}, yellow={}, red={}, unknown={}",
                            index.counts["green"],
                            index.counts["yellow"],
                            index.counts["red"],
                            index.counts["unknown"]
                        );
                    }
                    return Ok(());
                }
                let id = id.context("--id is required unless --all is present")?;
                let snapshot = cockpit_repository::work_item_status_snapshot_with_runtime(
                    &repo,
                    &id,
                    &runtime_context,
                )
                .context("read Work Item status")?;
                if let Some(path) = authorization_request {
                    let request: cockpit_protocol::TaskAuthorizationRequest =
                        read_json_file(&path, "task authorization request")?;
                    let assessment = cockpit_repository::assess_task_authorization(
                        &repo,
                        &id,
                        &request,
                        &runtime_context,
                    )?;
                    println!(
                        "{}",
                        serde_json::to_string_pretty(
                            &json!({"status":snapshot, "authorizationAssessment":assessment})
                        )?
                    );
                    return Ok(());
                }
                if json {
                    println!("{}", serde_json::to_string_pretty(&snapshot)?);
                } else {
                    let language = output_language(None);
                    let (label, phase, governance, activity, unknowns, next) = match language {
                        "zh" => ("状态", "生命周期", "治理", "活动健康", "未知项", "下一步"),
                        "ja" => (
                            "Status",
                            "Lifecycle",
                            "Governance",
                            "Activity health",
                            "不明点",
                            "次のアクション",
                        ),
                        _ => (
                            "Status",
                            "Lifecycle",
                            "Governance",
                            "Activity health",
                            "Unknowns",
                            "Next action",
                        ),
                    };
                    println!("{label}: {}", snapshot.work_item_id);
                    println!("{phase}: {}", snapshot.lifecycle_phase);
                    println!("{governance}: {}", snapshot.governance_state);
                    println!("{activity}: {}", snapshot.activity_health);
                    println!(
                        "{unknowns}: {}",
                        if snapshot.unknowns.is_empty() {
                            "None".into()
                        } else {
                            snapshot.unknowns.join(", ")
                        }
                    );
                    println!(
                        "{next}: read `work-item outcome --repo <path> --id {}`",
                        snapshot.work_item_id
                    );
                }
            }
            WorkItemCommand::Inspect { repo, id } => {
                require_compatible(&repo, &runtime_context)?;
                let compatibility = cockpit_repository::work_item_compatibility(&repo, &id)
                    .context("inspect Work Item compatibility")?;
                let approach = cockpit_repository::implementation_approach_read_only(&repo, &id)
                    .context("derive Work Item implementation approach")?;
                let parallel_slots = cockpit_repository::list_parallel_slots(&repo)
                    .context("inspect repository-local parallel slots")?;
                let output = json!({
                    "repositoryId": compatibility.repository_id,
                    "workItemId": id,
                    "compatibility": compatibility,
                    "implementationApproach": approach,
                    "parallelSlots": parallel_slots,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            }
            WorkItemCommand::Declare {
                repo,
                id,
                depends_on,
                conflicts_with,
                parallelizable,
            } => {
                require_compatible(&repo, &runtime_context)?;
                let intelligence = cockpit_repository::set_work_item_intelligence(
                    &repo,
                    &id,
                    depends_on,
                    conflicts_with,
                    parallelizable,
                )
                .context("declare Work Item intelligence")?;
                println!("{}", serde_json::to_string_pretty(&intelligence)?);
            }
            WorkItemCommand::Validate { repo, id, json } => {
                require_compatible(&repo, &runtime_context)?;
                let report =
                    cockpit_repository::validate_work_item_governance_controls_with_runtime(
                        &repo,
                        &id,
                        &runtime_context,
                    )
                    .context("validate Work Item Contract/Summary controls")?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else {
                    println!("Governance controls: {}", report.state);
                    println!("  scenario coverage: {}", report.scenario_coverage);
                    println!("  acceptance evidence: {}", report.acceptance_evidence);
                    println!("  intent alignment: {}", report.intent_alignment);
                    println!("  final dimensions: {}", report.final_dimensions);
                    if !report.unknowns.is_empty() {
                        println!("  unknowns: {}", report.unknowns.join(", "));
                    }
                    for item in report.findings {
                        println!("  [{}] {}: {}", item.severity, item.code, item.message);
                    }
                }
            }
            WorkItemCommand::Controls { repo, id, input } => {
                require_compatible(&repo, &runtime_context)?;
                let controls: serde_json::Value = serde_json::from_slice(
                    &std::fs::read(&input).context("read governance controls input")?,
                )
                .context("parse governance controls input")?;
                let summary = cockpit_repository::record_work_item_governance_controls_with_caller(
                    &repo,
                    &id,
                    &controls,
                    &runtime_context,
                    task_caller.as_ref(),
                )
                .context("record Work Item governance controls")?;
                println!("{}", serde_json::to_string_pretty(&summary)?);
            }
            WorkItemCommand::Retire { repo, id, input } => {
                require_compatible(&repo, &runtime_context)?;
                let request: cockpit_protocol::ActiveWorkItemRetirementRequest =
                    serde_json::from_slice(
                        &std::fs::read(&input).context("read Work Item retirement input")?,
                    )
                    .context("parse Work Item retirement input")?;
                let receipt =
                    retire_active_work_item_with_runtime(&repo, &id, &request, &runtime_context)
                        .context("retire active Work Item")?;
                println!("{}", serde_json::to_string_pretty(&receipt)?);
            }
            WorkItemCommand::Recover { repo, id, input } => {
                require_compatible(&repo, &runtime_context)?;
                let receipt: serde_json::Value = serde_json::from_slice(
                    &std::fs::read(&input).context("read recovery decision receipt")?,
                )
                .context("parse recovery decision receipt")?;
                let receipt = cockpit_repository::record_recovery_decision(
                    &repo,
                    &id,
                    &receipt,
                    &runtime_context,
                )
                .context("record Work Item recovery decision")?;
                println!("{}", serde_json::to_string_pretty(&receipt)?);
            }
            WorkItemCommand::RecoverSelectedSuccessorLineage { repo, id, input } => {
                require_compatible(&repo, &runtime_context)?;
                let receipt: serde_json::Value = serde_json::from_slice(
                    &std::fs::read(&input).context("read selected successor lineage receipt")?,
                )
                .context("parse selected successor lineage receipt")?;
                let receipt = cockpit_repository::record_selected_successor_lineage_recovery(
                    &repo,
                    &id,
                    &receipt,
                    &runtime_context,
                )
                .context("record selected successor lineage recovery")?;
                println!("{}", serde_json::to_string_pretty(&receipt)?);
            }
            WorkItemCommand::Boundary { repo, id, file } => {
                require_compatible(&repo, &runtime_context)?;
                let bytes = std::fs::read(&file).context("read concurrency boundary JSON")?;
                let boundary: ConcurrencyBoundary =
                    serde_json::from_slice(&bytes).context("parse concurrency boundary JSON")?;
                let boundary =
                    cockpit_repository::set_work_item_concurrency_boundary(&repo, &id, boundary)
                        .context("bind Contract concurrency boundary")?;
                println!("{}", serde_json::to_string_pretty(&boundary)?);
            }
            WorkItemCommand::Coordination { command } => {
                run_coordination_command(command, &runtime_context)?;
            }
            WorkItemCommand::Composition {
                repo,
                id,
                generation,
                input,
            } => {
                run_composition_command(repo, id, generation, input, &runtime_context)?;
            }
            WorkItemCommand::Slot { command } => match command {
                WorkItemSlotCommand::Acquire { repo, id } => {
                    require_compatible(&repo, &runtime_context)?;
                    let lease = cockpit_repository::acquire_parallel_slot(&repo, &id)
                        .context("acquire repository-local parallel slot")?;
                    println!("{}", serde_json::to_string_pretty(&lease)?);
                }
                WorkItemSlotCommand::Release { repo, id, lease_id } => {
                    require_compatible(&repo, &runtime_context)?;
                    let lease = cockpit_repository::release_parallel_slot(&repo, &id, &lease_id)
                        .context("release repository-local parallel slot")?;
                    println!("{}", serde_json::to_string_pretty(&lease)?);
                }
                WorkItemSlotCommand::List { repo } => {
                    require_compatible(&repo, &runtime_context)?;
                    let leases = cockpit_repository::list_parallel_slots(&repo)
                        .context("list repository-local parallel slots")?;
                    println!("{}", serde_json::to_string_pretty(&leases)?);
                }
            },
        },
        CommandKind::Capability { command } => match command {
            CapabilityCommand::Show {
                repo,
                surface,
                format,
                language,
            } => {
                if let Some(surface) = surface {
                    let description = cockpit_protocol::interface_description_for_surface(&surface)
                        .ok_or_else(|| anyhow::anyhow!(
                            "unknown capability description surface `{surface}`; supported surfaces: {}",
                            cockpit_protocol::CAPABILITY_SHOW_SURFACE_VALUES.join(", ")
                        ))?;
                    match format.as_str() {
                        cockpit_protocol::CAPABILITY_SHOW_FORMAT_JSON => {
                            println!("{}", serde_json::to_string_pretty(&description)?);
                        }
                        cockpit_protocol::CAPABILITY_SHOW_FORMAT_MARKDOWN => {
                            print!(
                                "{}",
                                cockpit_protocol::render_interface_description_markdown(
                                    &description,
                                    &language,
                                )
                            );
                        }
                        _ => unreachable!("capability format is validated by clap"),
                    }
                } else {
                    require_compatible(&repo, &runtime_context)?;
                    let registry = cockpit_repository::capability_truth_registry_with_runtime(
                        &repo,
                        &runtime_context,
                    )
                    .context("derive capability truth registry")?;
                    println!("{}", serde_json::to_string_pretty(&registry)?);
                }
            }
        },
        CommandKind::Diagnose { repo, work_item } => {
            require_compatible(&repo, &runtime_context)?;
            let diagnosis = cockpit_repository::performance_diagnosis(&repo, work_item.as_deref())
                .context("diagnose governance cost and performance")?;
            println!("{}", serde_json::to_string_pretty(&diagnosis)?);
        }
        CommandKind::Knowledge { command } => match command {
            KnowledgeCommand::Query {
                repo,
                topic,
                component,
                state,
                work_item_id,
                v2,
            } => {
                require_compatible(&repo, &runtime_context)?;
                if v2 {
                    let projection_path = repo.join(".ai/knowledge/index.v2.json");
                    let before = fs::read(&projection_path).ok();
                    let records = cockpit_repository::generate_knowledge_v2(&repo)
                        .context("project knowledge v2")?;
                    let after = fs::read(&projection_path).ok();
                    let materialization = if before.is_none() {
                        "created"
                    } else if before != after {
                        "rebuilt"
                    } else {
                        "reused"
                    };
                    let records = records
                        .into_iter()
                        .filter(|record| {
                            topic.as_ref().is_none_or(|value| &record.topic == value)
                                && component
                                    .as_ref()
                                    .is_none_or(|value| &record.component == value)
                                && state.as_ref().is_none_or(|value| &record.state == value)
                                && work_item_id
                                    .as_ref()
                                    .is_none_or(|value| &record.work_item_id == value)
                        })
                        .collect::<Vec<_>>();
                    let output = json!({
                        "schemaVersion": 2,
                        "projection": {
                            "path": ".ai/knowledge/index.v2.json",
                            "materialization": materialization,
                            "writeBoundary": "repository-local-derived",
                            "authority": "none"
                        },
                        "matchCount": records.len(),
                        "results": records
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                    return Ok(());
                }
                let projection_path = repo.join(".ai/knowledge/index.json");
                let before = fs::read(&projection_path).ok();
                let projection_started = std::time::Instant::now();
                let index = generate_knowledge(&repo).context("project knowledge")?;
                let projection_ms = projection_started.elapsed().as_secs_f64() * 1_000.0;
                let after = fs::read(&projection_path).ok();
                let materialization = if before.is_none() {
                    "created"
                } else if before != after {
                    "rebuilt"
                } else {
                    "reused"
                };
                let query_started = std::time::Instant::now();
                let (results, candidate_record_access_count) = query_with_metrics(
                    &index,
                    &Query {
                        topic,
                        component,
                        state,
                        work_item_id,
                    },
                );
                let query_ms = query_started.elapsed().as_secs_f64() * 1_000.0;
                let output = json!({
                    "schemaVersion": 1,
                    "projection": {
                        "path": ".ai/knowledge/index.json",
                        "materialization": materialization,
                        "writeBoundary": "repository-local-derived",
                        "authority": "none",
                        "sourceDigest": index.source_digest
                    },
                    "metrics": {
                        "knowledgeQuery": {
                            "projectionMs": projection_ms,
                            "queryMs": query_ms,
                            "candidateRecordAccessCount": candidate_record_access_count
                        }
                    },
                    "matchCount": results.len(),
                    "results": results,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            }
        },
        CommandKind::Profile { command } => match command {
            ProfileCommand::Confirm {
                repo,
                program,
                args,
                execution_recipe,
            } => {
                require_compatible(&repo, &runtime_context)?;
                let profile = if let Some(path) = execution_recipe {
                    let recipe = cockpit_repository::read_execution_recipe_file(&path)
                        .context("read execution recipe file")?;
                    cockpit_repository::confirm_profile_update_with_execution_recipe(
                        &repo,
                        &program,
                        &args,
                        Some(recipe),
                    )
                    .context("confirm project profile update")?
                } else {
                    cockpit_repository::confirm_profile_update(&repo, &program, &args)
                        .context("confirm project profile update")?
                };
                println!("{}", serde_json::to_string_pretty(&profile)?);
            }
            ProfileCommand::Propose { repo } => {
                require_compatible(&repo, &runtime_context)?;
                let git = GitRepository::discover(&repo).context("discover repository")?;
                let snapshot = git.snapshot().context("create repository snapshot")?;
                let observation = cockpit_repository::observe(&snapshot.root, &snapshot)
                    .context("observe repository")?;
                let profile_path = snapshot.root.join(".ai/project.json");
                let profile: cockpit_repository::AttachedProfile = serde_json::from_slice(
                    &std::fs::read(&profile_path).context("read attached profile")?,
                )
                .context("parse attached profile")?;
                let projected = cockpit_protocol::ProjectProfile {
                    profile_version: profile.profile_version,
                    repository_id: profile.repository_id.clone(),
                    tests: profile.tests.clone(),
                    build_systems: profile.build_systems.clone(),
                };
                let evolution =
                    cockpit_repository::classify_evolution(&projected, &observation, &snapshot);
                let proposal = cockpit_repository::profile_update_proposal(&projected, &evolution);
                let output = json!({
                    "kind": "project_profile_amendment",
                    "state": "candidate",
                    "status": "proposed",
                    "repositoryId": profile.repository_id,
                    "baseProfileDigest": profile.profile_digest,
                    "repositorySnapshotDigest": cockpit_repository::snapshot_digest(&snapshot)?,
                    "evolution": evolution,
                    "proposal": proposal,
                    "formalBaselineChanged": false,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            }
        },
        CommandKind::Mcp { repo } => {
            require_compatible(&repo, &runtime_context)?;
            let stdin = std::io::stdin();
            let stdout = std::io::stdout();
            cockpit_mcp::serve_with_repo(
                std::io::BufReader::new(stdin.lock()),
                stdout.lock(),
                &repo,
                &runtime_context,
            )
            .context("serve MCP")?;
        }
        CommandKind::Agent { command } => match command {
            AgentCommand::FirstStart { repo: _ } => {
                println!("{}", cockpit_agent::first_start_procedure());
            }
            AgentCommand::List { repo } => {
                require_compatible(&repo, &runtime_context)?;
                let detected =
                    cockpit_agent::detect_providers(&repo).context("list agent surfaces")?;
                println!("{}", serde_json::to_string_pretty(&detected)?);
            }
            AgentCommand::Install { repo, provider } => {
                require_compatible(&repo, &runtime_context)?;
                let provider = match provider.provider() {
                    Some(provider) => provider,
                    None => select_auto_provider(&repo)?,
                };
                let receipt = cockpit_agent::install_adapter(&repo, provider)
                    .context("install repository-owned agent adapter")?;
                println!("{}", serde_json::to_string_pretty(&receipt)?);
            }
            AgentCommand::Doctor { repo, json } => {
                require_compatible(&repo, &runtime_context)?;
                let report = cockpit_agent::doctor(&repo).context("inspect agent state")?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else {
                    println!("State: {}", report.state);
                    if let Some(repository_id) = &report.repository_id {
                        println!("Repository: {repository_id}");
                    }
                    println!("CLI: {}", report.interfaces.cli);
                    println!("MCP: {}", report.interfaces.mcp);
                    for problem in &report.problems {
                        println!("Problem: {problem}");
                    }
                    for action in &report.safe_actions {
                        println!("Safe action: {action}");
                    }
                }
                let code = agent_exit_code(&report.state);
                if code != AgentExitCode::Ready.code() {
                    std::process::exit(code);
                }
            }
            AgentCommand::Repair { repo, provider } => {
                require_compatible(&repo, &runtime_context)?;
                let provider = select_single_provider(&repo, provider)?;
                let receipt = cockpit_agent::repair_adapter(&repo, provider)
                    .context("repair repository-owned agent adapter")?;
                println!("{}", serde_json::to_string_pretty(&receipt)?);
            }
            AgentCommand::Detach { repo, provider } => {
                require_compatible(&repo, &runtime_context)?;
                let provider = provider
                    .provider()
                    .ok_or_else(|| anyhow::anyhow!("detach requires an explicit provider"))?;
                cockpit_agent::detach_adapter(&repo, provider)
                    .context("detach repository-owned agent adapter")?;
                println!("Agent adapter detached.");
            }
        },
        CommandKind::Doctor { repo, json: _ } => {
            let root = std::fs::canonicalize(&repo).context("canonicalize repository")?;
            let config_path = root.join(".ai/cockpit.toml");
            if !config_path.is_file() {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "state": "unattached",
                        "protocolVersion": null,
                        "repositoryId": null,
                        "runtimeCodeInRepository": false,
                        "runtimeVersion": &runtime_context.runtime_version,
                        "runtimeDigest": &runtime_context.runtime_digest,
                    }))?
                );
                return Ok(());
            }
            let config_text =
                std::fs::read_to_string(&config_path).context("read protocol configuration")?;
            let runtime_code_in_repository = contains_runtime_code(&root.join(".ai"));
            let config: RepositoryConfig = match toml::from_str(&config_text) {
                Ok(config) => config,
                Err(error) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "state": "red",
                            "protocolVersion": null,
                            "repositorySchemaVersion": null,
                            "repositoryId": null,
                            "runtimeCodeInRepository": runtime_code_in_repository,
                            "runtimeVersion": &runtime_context.runtime_version,
                            "runtimeDigest": &runtime_context.runtime_digest,
                            "error": error.to_string(),
                        }))?
                    );
                    return Ok(());
                }
            };
            let runtime_code_in_repository = contains_runtime_code(&root.join(".ai"));
            let compatibility =
                cockpit_repository::compatibility_report(&root, &runtime_context).ok();
            let state = if validate_protocol_version(config.protocol_version).is_ok()
                && !runtime_code_in_repository
                && config.repository_id == cockpit_repository::repository_id(&root).to_string()
                && compatibility
                    .as_ref()
                    .is_some_and(|value| value.state == "COMPATIBLE")
            {
                "ok"
            } else {
                "red"
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "state": state,
                    "protocolVersion": config.protocol_version,
                    "repositorySchemaVersion": config.repository_schema_version,
                    "repositoryId": config.repository_id,
                    "runtimeCodeInRepository": runtime_code_in_repository,
                    "runtimeVersion": &runtime_context.runtime_version,
                    "runtimeDigest": &runtime_context.runtime_digest,
                    "compatibility": compatibility,
                }))?
            );
        }
    }
    Ok(())
}

fn run_verification_command(
    command: CommandKind,
    runtime_context: RuntimeContext,
    task_caller: Option<cockpit_protocol::TaskAuthorizationCaller>,
) -> Result<()> {
    let CommandKind::Verify {
        repo,
        work_item,
        command,
        node,
        args,
        source_bundle,
        workers,
        stage,
        timeout_seconds,
        plan_only,
        base_revision,
        archived_recovery,
    } = command
    else {
        anyhow::bail!("internal error: verification dispatch received another command");
    };

    require_compatible(&repo, &runtime_context)?;
    let stage = match stage.as_str() {
        // The hosted route uses the provider-facing spelling while
        // the protocol parser historically used `pr`. Keep both at
        // the CLI boundary so recovery preflight fails only on real
        // repository facts, not on a vocabulary mismatch.
        "pull_request" => VerificationStage::PullRequest,
        other => VerificationStage::parse(other).map_err(|error| anyhow::anyhow!(error))?,
    };
    let root = std::fs::canonicalize(&repo).context("canonicalize repository")?;
    let initial_snapshot = GitRepository::discover(&root)
        .context("discover repository for verification route")?
        .snapshot()
        .context("capture verification route snapshot")?;
    let route = if let Some(work_item_id) = work_item.as_deref() {
        Some(if archived_recovery {
            resolve_archived_verification_route(
                &root,
                work_item_id,
                stage,
                "local",
                &initial_snapshot,
            )
            .context("resolve archived verification recovery route")?
        } else {
            resolve_verification_route(&root, work_item_id, stage, "local", &initial_snapshot)
                .context("resolve policy-bound verification route")?
        })
    } else {
        if archived_recovery {
            anyhow::bail!("--archived-recovery requires --work-item")
        }
        if stage.requires_base_revision()
            && base_revision
                .as_deref()
                .is_none_or(|value| !valid_cli_git_object_id(value))
        {
            anyhow::bail!(
                "verification stage {} requires --base-revision when no Work Item Contract is supplied",
                stage.as_str()
            );
        }
        None
    };
    let explicit = !command.is_empty();
    let commands = if explicit {
        command
            .into_iter()
            .map(|program| DeclaredVerificationCommand {
                node_id: None,
                program,
                args: args.clone(),
            })
            .collect::<Vec<_>>()
    } else if let Some(work_item_id) = work_item.as_deref() {
        let declared = declared_verification_commands(&root, work_item_id, archived_recovery)?;
        if declared.is_empty() {
            if source_bundle.is_some() {
                anyhow::bail!(
                    "--source-bundle requires a nonempty current Contract declaration inventory"
                );
            }
            if !node.is_empty() {
                anyhow::bail!("--node requires an explicit current Contract declaration inventory");
            }
            detected_verification_commands(&root)?
                .into_iter()
                .map(|(program, args)| DeclaredVerificationCommand {
                    node_id: None,
                    program,
                    args,
                })
                .collect()
        } else {
            declared
        }
    } else {
        detected_verification_commands(&root)?
            .into_iter()
            .map(|(program, args)| DeclaredVerificationCommand {
                node_id: None,
                program,
                args,
            })
            .collect()
    };
    let requests = if source_bundle.is_some() {
        if base_revision.is_some()
            && base_revision.as_deref()
                != route
                    .as_ref()
                    .and_then(|route| route.base_revision.as_deref())
        {
            anyhow::bail!(
                "source_bundle_base_revision_override_unsupported: omit --base-revision and use the current Contract route, or run verify without --source-bundle"
            );
        }
        source_bundle_verification_requests(
            &root,
            work_item.as_deref().expect("source bundle requires WI"),
            stage,
            workers,
            timeout_seconds,
            &runtime_context,
            task_caller.as_ref(),
        )?
    } else {
        commands
            .into_iter()
            .enumerate()
            .map(|(index, command)| RepositoryVerificationRequest {
                node_id: command
                    .node_id
                    .unwrap_or_else(|| format!("project-command-{index}")),
                program: command.program,
                args: command.args,
                scope: vec!["**".into()],
                stage: stage.as_str().into(),
                runner: "local".into(),
                runtime_digest: runtime_context.runtime_digest.to_string(),
                base_commit: route
                    .as_ref()
                    .and_then(|route| route.base_revision.clone())
                    .or_else(|| base_revision.clone()),
                workers,
                work_item_id: work_item.clone(),
                timeout_seconds,
                // Detected Work Item commands use the same exact,
                // identity-bound profile authorization as bare `verify`.
                // Explicit custom commands remain fresh unless a future
                // explicit reuse contract is added.
                policy: if explicit {
                    RepositoryVerificationPolicy::NeverReuse
                } else {
                    RepositoryVerificationPolicy::ProfileAuthorized
                },
            })
            .collect::<Vec<_>>()
    };
    let declared_inventory = requests
        .iter()
        .map(|r| r.node_id.clone())
        .collect::<Vec<_>>();
    let requests = select_declared_nodes(requests, &node)?;
    let selected_inventory = requests
        .iter()
        .map(|r| r.node_id.clone())
        .collect::<Vec<_>>();
    let omitted_inventory = declared_inventory
        .iter()
        .filter(|id| !selected_inventory.contains(id))
        .cloned()
        .collect::<Vec<_>>();
    let partial_declaration_selection = !node.is_empty() && !omitted_inventory.is_empty();
    let declared_coverage = json!({"level":"declared_commands","inventory":declared_inventory,
                "selected":selected_inventory,"omitted":omitted_inventory,"complete":omitted_inventory.is_empty()});
    if let Some(work_item_id) = work_item.as_deref()
        && !archived_recovery
    {
        let precondition = cockpit_repository::require_policy_for_verification(&root, work_item_id)
            .and_then(|_| {
                // Governance projections and ordering prerequisites are cheap
                // and deterministic. Reject them before starting the build or
                // test command so a missing registration cannot surface only
                // at finish after the expensive verification has completed.
                cockpit_repository::require_verification_preconditions_with_caller(
                    &root,
                    work_item_id,
                    &runtime_context,
                    &initial_snapshot,
                    task_caller.as_ref(),
                )
            });
        if let Err(error) = precondition {
            let attempt = cockpit_repository::persist_verification_attempt(
                &root,
                work_item_id,
                &requests,
                &initial_snapshot,
                &runtime_context,
                "precondition_rejected",
                Some(("verification_preconditions", &error.to_string())),
                None,
            );
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "state": "blocked",
                    "workItemId": work_item_id,
                    "gate": "verification_preconditions",
                    "diagnostic": error.to_string(),
                    "attempt": match attempt {
                        Ok(value) => value,
                        Err(persist_error) => json!({
                            "state": "persistence_failed",
                            "diagnostic": persist_error.to_string(),
                        }),
                    },
                }))?
            );
            anyhow::bail!("verification preconditions rejected: {error}");
        }
    } else if let Some(work_item_id) = work_item.as_deref() {
        cockpit_repository::require_archived_verification_recovery_preconditions(
            &root,
            work_item_id,
            &initial_snapshot,
            &runtime_context,
        )
        .context("check archived verification recovery preconditions")?;
    }
    if let Some(work_item_id) = work_item.as_deref()
        && workers > 1
        && let Err(error) =
            cockpit_repository::require_serial_work_item_verification(&root, work_item_id, workers)
    {
        let attempt = cockpit_repository::persist_verification_attempt(
            &root,
            work_item_id,
            &requests,
            &initial_snapshot,
            &runtime_context,
            "precondition_rejected",
            Some(("verification_parallelism", &error.to_string())),
            None,
        );
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "state": "blocked",
                "workItemId": work_item_id,
                "gate": "verification_parallelism",
                "diagnostic": error.to_string(),
                "processesSpawned": 0,
                "attempt": match attempt {
                    Ok(value) => value,
                    Err(persist_error) => json!({
                        "state": "persistence_failed",
                        "diagnostic": persist_error.to_string(),
                    }),
                },
            }))?
        );
        anyhow::bail!("verification parallelism rejected: {error}");
    }
    let mut planned_requests = Vec::new();
    let mut coverage_manifest = None;
    let mut workspace_test_manifests = Vec::new();
    for request in requests {
        let is_cargo_workspace = !explicit
            && request.program == "cargo"
            && request
                .args
                .iter()
                .any(|argument| argument == "--workspace");
        if !is_cargo_workspace {
            planned_requests.push(request);
            continue;
        }
        let plan = match cockpit_repository::plan_repository_verification(&root, &request) {
            Ok(plan) => plan,
            Err(error) => {
                if let Some(work_item_id) = work_item.as_deref() {
                    let _ = cockpit_repository::persist_verification_attempt(
                        &root,
                        work_item_id,
                        &planned_requests,
                        &initial_snapshot,
                        &runtime_context,
                        "precondition_rejected",
                        Some(("verification_plan", &error.to_string())),
                        None,
                    );
                }
                return Err(anyhow::anyhow!(
                    "plan repository verification coverage: {error}"
                ));
            }
        };
        if let Some(manifest) = plan.coverage_manifest {
            if manifest.source_program == "cargo"
                && manifest
                    .source_args
                    .first()
                    .is_some_and(|arg| arg == "test")
                && manifest.source_args.iter().any(|arg| arg == "--workspace")
                && manifest.source_args[1..].iter().all(|arg| {
                    matches!(
                        arg.as_str(),
                        "--workspace" | "--locked" | "--all-features" | "--all-targets"
                    )
                })
            {
                workspace_test_manifests.push(manifest.clone());
            }
            if coverage_manifest.is_none() {
                coverage_manifest = Some(manifest);
            }
        }
        planned_requests.extend(plan.requests);
    }
    if workspace_test_manifests.len() == 1 {
        coverage_manifest = workspace_test_manifests.pop();
    }
    let requests = planned_requests;
    let local_node_plan = if !explicit && !archived_recovery && work_item.is_some() {
        cockpit_repository::plan_local_node_reuse(
            &root,
            &requests,
            &initial_snapshot,
            &runtime_context,
        )?
    } else {
        Vec::new()
    };
    let now_epoch_seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0);
    let mut planned_nodes = Vec::with_capacity(requests.len());
    for request in &requests {
        match cockpit_repository::plan_repository_verification_action(
            &root,
            request,
            &initial_snapshot,
            now_epoch_seconds,
        ) {
            Ok(node) => planned_nodes.push(node),
            Err(error) => {
                let attempt = if let Some(work_item_id) = work_item.as_deref() {
                    cockpit_repository::persist_verification_attempt(
                        &root,
                        work_item_id,
                        &requests,
                        &initial_snapshot,
                        &runtime_context,
                        "precondition_rejected",
                        Some(("verification_plan", &error.to_string())),
                        None,
                    )
                } else {
                    Ok(json!({"state": "not_applicable"}))
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "state": "blocked",
                        "workItemId": work_item,
                        "gate": "verification_plan",
                        "diagnostic": error.to_string(),
                        "plannedNodes": planned_nodes,
                        "attempt": match attempt {
                            Ok(value) => value,
                            Err(persist_error) => json!({
                                "state": "persistence_failed",
                                "diagnostic": persist_error.to_string(),
                            }),
                        },
                        "processesSpawned": 0,
                    }))?
                );
                anyhow::bail!("verification planning rejected: {error}");
            }
        }
    }
    for node in &mut planned_nodes {
        if let Some(decision) = local_node_plan.iter().find(|d| node["nodeId"] == d.node_id) {
            node["localNodeReuseReason"] = json!(decision.reason);
            if decision.fresh_required {
                node["action"] = json!("execute");
                node["state"] = json!("unknown");
                node["reason"] = json!(decision.reason);
                node["satisfiedBy"] = json!("execution");
                node["receiptId"] = serde_json::Value::Null;
                node["reuseAuthorization"] = json!("fresh_required_by_native_observation");
            }
            if let Some(candidate) = &decision.candidate {
                node["action"] = json!("reuse");
                node["state"] = json!("fresh");
                node["reason"] = json!("fresh_local_node_exact_binding");
                node["satisfiedBy"] = json!("reused_receipt");
                node["reuseAuthorization"] = json!("identity_bound_local_node");
                node["localNodeEvidence"] = candidate.projection();
                node["localNodeCostCompleteness"] = json!("partial_source_counters_only");
            }
        }
    }
    if plan_only {
        let nodes_reused = planned_nodes
            .iter()
            .filter(|node| node["action"] == "reuse")
            .count();
        let plan = ::serde_json::json!({
            "coverageManifest": coverage_manifest,
            "declaredCoverage": declared_coverage,
            "requests": requests.iter().zip(&planned_nodes).map(|(request, node)| ::serde_json::json!({
                "nodeId": request.node_id,
                "program": request.program,
                "args": request.args,
                "dependencies": [],
                "timeoutSeconds": node["timeoutSeconds"],
            })).collect::<Vec<_>>(),
            "state": "planned",
            "workItemId": work_item,
            "repositoryId": cockpit_repository::repository_id(&root).to_string(),
            "repositorySnapshotDigest": cockpit_repository::snapshot_digest(&initial_snapshot)?.to_string(),
            "runtimeVersion": runtime_context.runtime_version,
            "runtimeDigest": runtime_context.runtime_digest.to_string(),
            "nodesPlanned": planned_nodes.len(),
            "nodesToExecute": planned_nodes.len().saturating_sub(nodes_reused),
            "nodesReused": nodes_reused,
            "processesSpawned": 0,
            "timeoutSeconds": planned_nodes
                .iter()
                .filter_map(|node| node["timeoutSeconds"].as_u64())
                .max()
                .unwrap_or(cockpit_verification::DEFAULT_EXECUTION_SECONDS),
            "plannedNodes": planned_nodes,
        });
        println!("{}", serde_json::to_string_pretty(&plan)?);
        return Ok(());
    }
    // Acquire live sources before creating this run's pending checkpoint.
    // The absent option preserves the ordinary provider-independent route.
    let source_service_started = source_bundle.as_ref().map(|_| std::time::Instant::now());
    let accepted_sources = if let Some(manifest) = source_bundle.as_deref() {
        let wi = work_item.as_deref().expect("source bundle requires WI");
        let tool =
            cockpit_repository::PortableSourceTool::for_current_runtime(&root, &runtime_context)
                .map_err(source_bundle_error)?;
        Some(
            cockpit_repository::acquire_portable_evidence_inventory(
                &root,
                wi,
                &requests,
                manifest,
                &tool,
                &runtime_context,
                task_caller.as_ref(),
            )
            .map_err(source_bundle_error)?,
        )
    } else {
        None
    };
    let interruption_checkpoint = if work_item.is_some() {
        Some(
            cockpit_repository::VerificationRunCheckpoint::prepare(
                &root,
                &requests,
                &planned_nodes,
                &initial_snapshot,
                &runtime_context,
            )
            .context("prepare verification interruption checkpoint")?,
        )
    } else {
        None
    };
    let local_run_identity = interruption_checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.local_run_identity());
    let mut source_runs = Vec::new();
    let mut source_inventory = if let Some(sources) = accepted_sources {
        match cockpit_repository::consume_portable_evidence_inventory(
            &root,
            work_item.as_deref().expect("source bundle requires WI"),
            &requests,
            sources,
            &runtime_context,
            task_caller.as_ref(),
            local_run_identity.as_ref(),
        ) {
            Ok(inventory) => Some(inventory),
            Err(error) => {
                let unstarted = requests
                    .iter()
                    .map(|r| r.node_id.clone())
                    .collect::<Vec<_>>();
                return Err(interrupted_verification_error(
                    interruption_checkpoint.as_ref(),
                    source_runs.iter(),
                    &[],
                    &unstarted,
                    &[],
                    "source_inventory_consume",
                    &source_bundle_error(error).to_string(),
                ));
            }
        }
    } else {
        None
    };
    // Clone only already observed native data for error preservation.
    // Success merges only the actual runs released by finalize below.
    source_runs = source_inventory
        .as_ref()
        .map(|inventory| inventory.observed_runs().to_vec())
        .unwrap_or_default();
    let mut source_coverage = source_inventory
        .as_ref()
        .map(|inventory| inventory.coverage().clone());
    let mut source_uses = source_inventory
        .as_ref()
        .map(|inventory| inventory.source_uses().to_vec())
        .unwrap_or_default();
    // Keep actual source receipts available even if finalize consumes the
    // opaque session and then refuses a late failed/pending observation.
    let source_observations = source_uses
        .iter()
        .map(|used| used.receipt.clone())
        .collect::<Vec<_>>();
    let actual_source_nodes = source_uses
        .iter()
        .map(|used| used.node_id.clone())
        .collect::<Vec<_>>();
    let source_dispatch = match verification_source_dispatch(
        &requests,
        source_coverage.as_ref(),
        &actual_source_nodes,
    ) {
        Ok(dispatch) => dispatch,
        Err(error) => {
            let unstarted = requests
                .iter()
                .filter(|r| !actual_source_nodes.contains(&r.node_id))
                .map(|r| r.node_id.clone())
                .collect::<Vec<_>>();
            return Err(interrupted_verification_error(
                interruption_checkpoint.as_ref(),
                source_runs.iter(),
                &[],
                &unstarted,
                &source_observations,
                "source_inventory_dispatch",
                &source_bundle_error(error).to_string(),
            ));
        }
    };
    let execution_requests = source_dispatch
        .fresh_indices
        .iter()
        .map(|index| &requests[*index])
        .collect::<Vec<_>>();
    let attempt_work_item = work_item.clone();
    let persist_for_run = |snapshot: &cockpit_git::RepositorySnapshot,
                           state: &str,
                           diagnostic: Option<(&str, &str)>,
                           receipt: Option<&serde_json::Value>,
                           supersedes: Option<&str>| {
        let wi = attempt_work_item
            .as_deref()
            .expect("native attempt requires WI");
        if let Some(checkpoint) = interruption_checkpoint.as_ref() {
            checkpoint.persist_local_run_attempt(
                &requests,
                snapshot,
                &runtime_context,
                cockpit_repository::LocalRunAttemptWrite {
                    state,
                    diagnostic,
                    receipt,
                    supersedes,
                },
            )
        } else if let Some(prior) = supersedes {
            cockpit_repository::persist_verification_attempt_superseding(
                &root,
                wi,
                &requests,
                snapshot,
                &runtime_context,
                state,
                diagnostic,
                receipt,
                prior,
            )
        } else {
            cockpit_repository::persist_verification_attempt(
                &root,
                wi,
                &requests,
                snapshot,
                &runtime_context,
                state,
                diagnostic,
                receipt,
            )
        }
    };
    let requires_aggregate_snapshot = requests.len() > 1;
    let service_started = source_service_started.unwrap_or_else(std::time::Instant::now);
    let supervisor_executable = match std::env::current_exe() {
        Ok(executable) => executable,
        Err(error) if source_bundle.is_some() => {
            let unstarted = execution_requests
                .iter()
                .map(|r| r.node_id.clone())
                .collect::<Vec<_>>();
            return Err(interrupted_verification_error(
                interruption_checkpoint.as_ref(),
                source_runs.iter(),
                &[],
                &unstarted,
                &source_observations,
                "supervisor_executable",
                &error.to_string(),
            ));
        }
        Err(error) => {
            return Err(anyhow::Error::from(error).context("resolve governing Runtime executable"));
        }
    };
    let process_concurrency = Arc::new(ProcessConcurrencyObservation::default());
    let mut runs = Vec::with_capacity(requests.len());
    let mut planning_elapsed_ms = 0_u128;
    let mut execution_elapsed_ms = 0_u128;
    {
        let total_requests = requests.len();
        let mut completed_requests = source_dispatch.source_count;
        for (batch_index, batch) in execution_requests.chunks(workers.max(1)).enumerate() {
            let batch_runs = std::thread::scope(|scope| {
                let (progress_sender, progress_receiver) =
                    std::sync::mpsc::channel::<(String, Option<bool>)>();
                let workers = batch
                            .iter().copied()
                            .map(|request| {
                                let progress_sender = progress_sender.clone();
                                let repository_root = &root;
                                let process_concurrency = Arc::clone(&process_concurrency);
                                let caller = task_caller.as_ref();
                                let runtime = &runtime_context;
                                let supervisor_executable = supervisor_executable.clone();
                                let local_node_plan = &local_node_plan;
                                let source_fresh_required = source_dispatch.force_fresh.contains(&request.node_id);
                                let local_run_identity=local_run_identity.as_ref();
                                scope.spawn(move || {
                                    let node_id = request.node_id.clone();
                                    let _ = progress_sender.send((node_id.clone(), None));
                                    let decision=local_node_plan.iter().find(|d|d.node_id==request.node_id);
                                    let (candidate, force_fresh) = verification_local_reuse_inputs(
                                        decision.and_then(|d|d.candidate.as_ref()),
                                        decision.is_some_and(|d|d.fresh_required), source_fresh_required,
                                    );
                                    let reused=match candidate {
                                        Some(candidate)=>cockpit_repository::reuse_local_node_in_run_with_task_authorization(repository_root,request,runtime,caller,candidate,local_run_identity),
                                        None=>Ok(None),
                                    };
                                    let run = reused.and_then(|reused| match reused {
                                        Some(run)=>Ok(run),
                                        None=>cockpit_repository::run_repository_verification_with_local_node_guard(
                                        repository_root,
                                        request,
                                        runtime,
                                        caller,
                                        move |_node_id, _process_id, started| {
                                            process_concurrency.observe(started)
                                        },
                                        Some(cockpit_verification::VerificationSupervisorLaunch::new(
                                            supervisor_executable,
                                            vec!["__verification-supervisor".into()],
                                        )),
                                        cockpit_repository::LocalNodeExecutionContext{own_run:local_run_identity,force_fresh},
                                    ),
                                    });
                                    let passed = run.as_ref().is_ok_and(|run| run.receipt.passed);
                                    let _ = progress_sender.send((node_id, Some(passed)));
                                    run
                                })
                            })
                            .collect::<Vec<_>>();
                drop(progress_sender);
                for (node_id, passed) in progress_receiver {
                    if let Some(passed) = passed {
                        completed_requests = completed_requests.saturating_add(1);
                        let percent =
                            completed_requests.saturating_mul(100) / total_requests.max(1);
                        let result = if passed { "passed" } else { "failed" };
                        eprintln!(
                            "Verification progress: completed node {node_id:?}: {result} ({completed_requests}/{total_requests} complete, {percent}%)"
                        );
                    } else {
                        let percent =
                            completed_requests.saturating_mul(100) / total_requests.max(1);
                        eprintln!(
                            "Verification progress: started node {node_id:?} ({completed_requests}/{total_requests} complete, {percent}%)"
                        );
                    }
                }
                workers
                    .into_iter()
                    .map(|worker| worker.join())
                    .collect::<Vec<_>>()
            });
            let mut completed_batch = Vec::with_capacity(batch_runs.len());
            let mut unavailable = Vec::new();
            let mut unvalidated_observations = Vec::new();
            for (request, run) in batch.iter().copied().zip(batch_runs) {
                match run {
                    Ok(Ok(run)) => completed_batch.push(run),
                    Ok(Err(error)) => {
                        unavailable.push((request.node_id.clone(), error.to_string()));
                        if let cockpit_repository::ObserverError::VerificationCheckpointPublication {
                                    observation, ..
                                } = error {
                                    unvalidated_observations.push(*observation);
                                }
                    }
                    Err(_) => unavailable.push((
                        request.node_id.clone(),
                        "repository verification worker panicked".into(),
                    )),
                }
            }
            planning_elapsed_ms = planning_elapsed_ms.saturating_add(concurrent_phase_elapsed(
                completed_batch
                    .iter()
                    .map(|run| run.receipt.planning_elapsed_ms),
            ));
            execution_elapsed_ms = execution_elapsed_ms.saturating_add(concurrent_phase_elapsed(
                completed_batch
                    .iter()
                    .map(|run| run.receipt.execution_elapsed_ms),
            ));
            runs.extend(completed_batch);
            if !unavailable.is_empty() {
                let next = batch_index * workers.max(1) + batch.len();
                let unstarted = execution_requests[next..]
                    .iter()
                    .map(|request| request.node_id.clone())
                    .collect::<Vec<_>>();
                let message = unavailable
                    .iter()
                    .map(|(node, error)| format!("{node}: {error}"))
                    .collect::<Vec<_>>()
                    .join("; ");
                return Err(interrupted_verification_error(
                    interruption_checkpoint.as_ref(),
                    runs.iter().chain(&source_runs),
                    &unavailable,
                    &unstarted,
                    &verification_interruption_observations(
                        &source_observations,
                        &unvalidated_observations,
                    ),
                    "worker",
                    &message,
                )
                .context("execute repository verification"));
            }
        }
    }
    // Keep completed receipts available while aggregate observation can
    // still fail; no successful node output is consumed by an early ?.
    let aggregate_snapshot = if requires_aggregate_snapshot {
        match cockpit_git::GitRepository::discover(&root).and_then(|git| git.snapshot()) {
            Ok(snapshot) => Some(snapshot),
            Err(error) => {
                return Err(interrupted_verification_error(
                    interruption_checkpoint.as_ref(),
                    runs.iter().chain(&source_runs),
                    &[],
                    &[],
                    &source_observations,
                    "aggregate_snapshot",
                    &error.to_string(),
                )
                .context("capture aggregate verification snapshot"));
            }
        }
    } else {
        None
    };
    // Recheck every held source after fresh execution and before the
    // existing merger/formal path. Never reconstruct runs from JSON.
    if let Some(inventory) = source_inventory.take() {
        match inventory.finalize(&root, &requests, &runtime_context, task_caller.as_ref()) {
            Ok(finalized) => {
                source_coverage = Some(finalized.coverage);
                source_uses = finalized.source_uses;
                runs.extend(finalized.runs);
            }
            Err(error) => {
                return Err(interrupted_verification_error(
                    interruption_checkpoint.as_ref(),
                    runs.iter().chain(&source_runs),
                    &[],
                    &[],
                    &source_observations,
                    "source_inventory_finalize",
                    &source_bundle_error(error).to_string(),
                ));
            }
        }
    }
    let mut run = match merge_verification_runs(runs) {
        Some(run) => run,
        None => {
            return Err(interrupted_verification_error(
                interruption_checkpoint.as_ref(),
                source_runs.iter(),
                &[],
                &[],
                &source_observations,
                "aggregate_merge",
                "no verification runs available to merge",
            )
            .context("merge verification runs"));
        }
    };
    run.receipt.max_concurrent_processes = process_concurrency.peak();
    run.receipt.planning_elapsed_ms = planning_elapsed_ms;
    run.receipt.execution_elapsed_ms = execution_elapsed_ms;
    if let Some(final_snapshot) = aggregate_snapshot {
        run.receipt.git_calls = run
            .receipt
            .git_calls
            .saturating_add(final_snapshot.git_calls);
        run.receipt.files_read = run
            .receipt
            .files_read
            .saturating_add(final_snapshot.files_read);
        run.receipt.files_hashed = run
            .receipt
            .files_hashed
            .saturating_add(final_snapshot.files_hashed);
        run.final_snapshot = final_snapshot;
    }
    run.receipt.elapsed_ms = service_started.elapsed().as_millis();
    run.receipt.repository_id = Some(cockpit_repository::repository_id(&root).to_string());
    run.receipt.runtime_version = Some(runtime_context.runtime_version.clone());
    run.receipt.runtime_digest = Some(runtime_context.runtime_digest.to_string());
    let (initial_tier, final_tier, assurance, selection_reasons, escalations) = route
        .as_ref()
        .map(|route| {
            (
                route.actual_tier,
                route.actual_tier,
                route.actual_assurance,
                route
                    .policy_plan
                    .as_ref()
                    .map(|plan| plan.escalation_reasons.clone())
                    .unwrap_or_else(|| vec!["cli_route_stage_explicit".into()]),
                Vec::new(),
            )
        })
        .unwrap_or((
            VerificationTier::T0,
            VerificationTier::T0,
            EvidenceAssurance::SelfDeclared,
            vec!["cli_route_stage_explicit".into()],
            Vec::new(),
        ));
    let mut plan_receipt = cockpit_verification::VerificationPlanReceipt::new(
        stage,
        initial_tier,
        final_tier,
        assurance,
        selection_reasons,
        escalations,
    )
    .map_err(|error| {
        verification_error_after_source(
            source_bundle.is_some(),
            interruption_checkpoint.as_ref(),
            &run,
            "aggregate_plan_receipt",
            anyhow::anyhow!(error),
        )
    })?;
    if let Some(route) = &route {
        plan_receipt.work_item_id = Some(route.work_item_id.clone());
        plan_receipt.repository_id = Some(cockpit_repository::repository_id(&root).to_string());
        plan_receipt.repository_snapshot_digest = Some(
            cockpit_repository::snapshot_digest(&run.final_snapshot)
                .map_err(|error| {
                    verification_error_after_source(
                        source_bundle.is_some(),
                        interruption_checkpoint.as_ref(),
                        &run,
                        "aggregate_snapshot_digest",
                        error.into(),
                    )
                })?
                .to_string(),
        );
        plan_receipt.base_revision = route.base_revision.clone();
        plan_receipt.affected_paths = route.affected_paths.clone();
        plan_receipt.dependency_confidence = Some(route.dependency_confidence);
        if let Some(plan) = &route.policy_plan {
            plan_receipt.required_tier = Some(plan.requirement.required_tier);
            plan_receipt.required_assurance = Some(plan.requirement.required_assurance);
            plan_receipt.policy_refs = plan.requirement.policy_refs.clone();
        }
    }
    plan_receipt.coverage_manifest = coverage_manifest;
    plan_receipt.executed_nodes = run
        .receipt
        .results
        .iter()
        .filter(|result| !result.reused)
        .map(|result| result.node_id.clone())
        .collect();
    plan_receipt.reused_nodes = run
        .receipt
        .results
        .iter()
        .filter(|result| result.reused)
        .map(|result| result.node_id.clone())
        .collect();
    plan_receipt.execution_elapsed_ms = run.receipt.execution_elapsed_ms;
    plan_receipt.planning_elapsed_ms = run.receipt.planning_elapsed_ms;
    plan_receipt.saved_executions = run.receipt.nodes_reused;
    plan_receipt.timeout_seconds = run.receipt.timeout_seconds;
    run.receipt.plan_receipt = Some(plan_receipt);
    let used_local_nodes = local_node_plan
        .iter()
        .filter_map(|d| d.candidate.as_ref())
        .filter(|candidate| {
            run.receipt.results.iter().any(|r| {
                r.node_id == candidate.node_id()
                    && r.reused
                    && r.receipt_id.as_deref() == candidate.projection()["nodeFactId"].as_str()
            })
        })
        .cloned()
        .collect::<Vec<_>>();
    if !used_local_nodes.is_empty() {
        let mut final_local_phase = "local_node_snapshot";
        let final_local_check = (|| -> Result<()> {
            let current_snapshot = GitRepository::discover(&root)?.snapshot()?;
            if cockpit_repository::snapshot_digest(&current_snapshot)?
                != cockpit_repository::snapshot_digest(&run.final_snapshot)?
            {
                anyhow::bail!("source changed before local reuse finalization");
            }
            final_local_phase = "local_node_preconditions";
            cockpit_repository::require_verification_preconditions_with_caller(
                &root,
                work_item.as_deref().expect("WI local nodes"),
                &runtime_context,
                &current_snapshot,
                task_caller.as_ref(),
            )?;
            final_local_phase = "local_node_candidates";
            cockpit_repository::require_local_node_candidates_current_in_run(
                &root,
                &requests,
                &current_snapshot,
                &runtime_context,
                &used_local_nodes,
                local_run_identity.as_ref(),
            )?;
            Ok(())
        })();
        if let Err(error) = final_local_check {
            return Err(interrupted_verification_error(
                interruption_checkpoint.as_ref(),
                std::slice::from_ref(&run),
                &[],
                &[],
                &[],
                final_local_phase,
                &error.to_string(),
            ));
        }
    }
    reconcile_actual_local_node_projection(&mut planned_nodes, &run.receipt.results).map_err(
        |error| {
            verification_error_after_source(
                source_bundle.is_some(),
                interruption_checkpoint.as_ref(),
                &run,
                "aggregate_result_projection",
                error,
            )
        },
    )?;
    let cost_observation = run.receipt.cost_observation();
    run.receipt.cost_observation = Some(cost_observation);
    // Keep the strict protocol receipt separate from CLI-only planning
    // projections. Persisted Work Item evidence is deserialized with
    // deny_unknown_fields and must remain reusable as a typed receipt.
    let verification_receipt = serde_json::to_value(&run.receipt).map_err(|error| {
        verification_error_after_source(
            source_bundle.is_some(),
            interruption_checkpoint.as_ref(),
            &run,
            "aggregate_receipt_serialization",
            error.into(),
        )
    })?;
    let mut output = verification_receipt.clone();
    output["runtimeVersion"] = serde_json::Value::String(runtime_context.runtime_version.clone());
    output["runtimeDigest"] = serde_json::Value::String(runtime_context.runtime_digest.to_string());
    output["plannedNodes"] = json!(planned_nodes);
    output["declaredCoverage"] = declared_coverage;
    if let Some(coverage) = &source_coverage {
        output["sourceInventoryCoverage"] = json!(coverage);
        // Use the finalizer's actual invocation/cost observations. No
        // fixed reader count or provider-specific field enters Verify.
        output["sourceUses"] = json!(source_uses);
    }
    output["diagnosticSummary"] = json!(cockpit_verification::summarize_diagnostics(
        &run.receipt.execution_records
    ));
    if !run.receipt.passed {
        let failed_nodes = run
            .receipt
            .results
            .iter()
            .filter(|result| !result.passed)
            .map(|result| format!("{} ({})", result.node_id, result.reason))
            .collect::<Vec<_>>();
        if work_item.is_some() {
            let attempt = persist_for_run(
                &initial_snapshot,
                "execution_failed",
                Some((
                    "verification_execution",
                    "one or more verification nodes failed",
                )),
                Some(&verification_receipt),
                None,
            );
            if let Err(error) = &attempt {
                eprintln!("could not persist failed verification attempt: {error}");
            }
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&output).map_err(|error| {
                verification_error_after_source(
                    source_bundle.is_some(),
                    interruption_checkpoint.as_ref(),
                    &run,
                    "verification_output",
                    error.into(),
                )
            })?
        );
        anyhow::bail!(
            "verification command failed for {}; structured failure receipt emitted",
            if failed_nodes.is_empty() {
                "an unknown node".into()
            } else {
                failed_nodes.join(", ")
            }
        );
    }
    if let Some(work_item) = work_item {
        // Persist the execution boundary before attempting to promote
        // it into completion evidence.  A later governance/schema
        // rejection must therefore retain the child results for a
        // safe reclassification or reuse decision.
        let execution_attempt = persist_for_run(&initial_snapshot,"execution_completed",if partial_declaration_selection {Some(("partial_declaration_coverage","Selected declarations completed; omitted declarations remain unverified; no formal evidence promotion."))} else {None},Some(&verification_receipt),None)
                .context("persist verification attempt")
                .map_err(|error| verification_error_after_source(
                    source_bundle.is_some(), interruption_checkpoint.as_ref(), &run,
                    "execution_attempt_publication", error,
                ))?;
        let execution_attempt_id = execution_attempt["attemptId"]
            .as_str()
            .context("persisted verification attempt has no identity")
            .map_err(|error| {
                verification_error_after_source(
                    source_bundle.is_some(),
                    interruption_checkpoint.as_ref(),
                    &run,
                    "execution_attempt_identity",
                    error,
                )
            })?
            .to_owned();
        if partial_declaration_selection {
            // Keep this exact execution attempt, but retain all prior formal
            // evidence, Summary/Outcome and complete-verification retry state.
            if let Some(checkpoint) = interruption_checkpoint.as_ref() {
                output["localRunCompletion"] =
                    match checkpoint.record_local_run_completion(&execution_attempt) {
                        Ok(value) => {
                            json!({"state":"recorded","checkpointId":value["checkpointId"]})
                        }
                        Err(_) => json!({"state":"unavailable","reusePermitted":false}),
                    };
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&output).map_err(|error| {
                    verification_error_after_source(
                        source_bundle.is_some(),
                        interruption_checkpoint.as_ref(),
                        &run,
                        "verification_output",
                        error.into(),
                    )
                })?
            );
            return Ok(());
        }
        if archived_recovery {
            if let Err(error) = cockpit_repository::record_archived_verification_recovery_with_runtime(
                        &root,
                        &work_item,
                        &cockpit_repository::ArchivedVerificationRecoveryRequest {
                            reason: "refresh archived source evidence after reviewed Runtime integration changes".into(),
                            actor: "runtime:ai-cockpit".into(),
                            authority_source: "current Work Item Contract and reviewed recovery invocation".into(),
                            evidence_refs: vec![format!(
                                ".ai/evidence/{work_item}.verification.json"
                            )],
                            policy_refs: route
                                .as_ref()
                                .and_then(|route| route.policy_plan.as_ref())
                                .map(|plan| plan.requirement.policy_refs.clone())
                                .unwrap_or_default(),
                            verification_receipt: verification_receipt.clone(),
                        },
                        &runtime_context,
                        &run.final_snapshot,
                    ) {
                        let recovery_attempt = persist_for_run(&initial_snapshot,"formal_receipt_rejected",Some(("formal_receipt", &error.to_string())),Some(&verification_receipt),Some(&execution_attempt_id));
                        if let Err(persist_error) = recovery_attempt {
                            eprintln!("could not persist receipt rejection attempt: {persist_error}");
                        }
                        anyhow::bail!("record archived verification recovery: {error}");
                    }
        } else {
            if let Err(error) = cockpit_repository::record_verification_with_caller(
                &root,
                &work_item,
                &verification_receipt,
                &runtime_context,
                &run.final_snapshot,
                task_caller.as_ref(),
            ) {
                let recovery_attempt = persist_for_run(
                    &initial_snapshot,
                    "formal_receipt_rejected",
                    Some(("formal_receipt", &error.to_string())),
                    Some(&verification_receipt),
                    Some(&execution_attempt_id),
                );
                if let Err(persist_error) = recovery_attempt {
                    eprintln!("could not persist receipt rejection attempt: {persist_error}");
                }
                return Err(verification_error_after_source(
                    source_bundle.is_some(),
                    interruption_checkpoint.as_ref(),
                    &run,
                    "formal_receipt",
                    anyhow::anyhow!("record verification evidence: {error}"),
                ));
            }
        }
        if let Some(checkpoint) = interruption_checkpoint.as_ref() {
            output["localRunCompletion"] =
                match checkpoint.record_local_run_completion(&execution_attempt) {
                    Ok(value) => {
                        json!({"state":"recorded","checkpointId":value["checkpointId"]})
                    }
                    Err(_) => json!({"state":"unavailable","reusePermitted":false}),
                };
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|error| {
            verification_error_after_source(
                source_bundle.is_some(),
                interruption_checkpoint.as_ref(),
                &run,
                "verification_output",
                error.into(),
            )
        })?
    );
    Ok(())
}

fn run_isolation_manifest(
    root: &Path,
    output: Option<&PathBuf>,
    paths_file: Option<&PathBuf>,
    mask_ancestors_of: Option<&Path>,
) -> Result<()> {
    if let Some(path) = output {
        let mut destination = fs::File::create(path)
            .with_context(|| format!("create isolation manifest output {}", path.display()))?;
        return emit_isolation_manifest(root, paths_file, mask_ancestors_of, &mut destination);
    }
    let stdout = std::io::stdout();
    let mut destination = stdout.lock();
    emit_isolation_manifest(root, paths_file, mask_ancestors_of, &mut destination)
}

fn emit_isolation_manifest<W: std::io::Write>(
    root: &Path,
    paths_file: Option<&PathBuf>,
    mask_ancestors_of: Option<&Path>,
    destination: W,
) -> Result<()> {
    if let Some(paths_file) = paths_file {
        let bytes = fs::read(paths_file)
            .with_context(|| format!("read NUL-delimited path list {}", paths_file.display()))?;
        let paths = bytes
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
            .map(|path| {
                std::str::from_utf8(path)
                    .context("source manifest path list contains non-UTF-8 path")
                    .map(PathBuf::from)
            })
            .collect::<Result<Vec<_>>>()?;
        cockpit_isolation::scan_paths_to_jsonl_with_mask(
            root,
            paths,
            destination,
            mask_ancestors_of,
        )
        .context("scan selected isolation paths")?;
    } else {
        cockpit_isolation::scan_tree_to_jsonl(root, destination).context("scan isolation tree")?;
    }
    Ok(())
}

#[derive(Default)]
struct ProcessConcurrencyObservation {
    active: AtomicUsize,
    peak: AtomicUsize,
}

impl ProcessConcurrencyObservation {
    fn observe(&self, started: bool) -> std::result::Result<(), String> {
        if started {
            let active = self.active.fetch_add(1, Ordering::AcqRel).saturating_add(1);
            self.peak.fetch_max(active, Ordering::AcqRel);
            return Ok(());
        }

        self.active
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |active| {
                active.checked_sub(1)
            })
            .map(|_| ())
            .map_err(|_| "verification process completed without a matching start event".into())
    }

    fn peak(&self) -> usize {
        self.peak.load(Ordering::Acquire)
    }
}

fn interrupted_verification_error<'a>(
    checkpoint: Option<&cockpit_repository::VerificationRunCheckpoint>,
    runs: impl IntoIterator<Item = &'a cockpit_repository::RepositoryVerificationRun>,
    unavailable: &[(String, String)],
    unstarted: &[String],
    unvalidated_observations: &[serde_json::Value],
    phase: &str,
    message: &str,
) -> anyhow::Error {
    let receipts = runs.into_iter().map(|run| &run.receipt).collect::<Vec<_>>();
    let interruption = checkpoint
        .map(|checkpoint| {
            checkpoint.interruption_facts(&receipts, unavailable, unstarted, phase, message)
        })
        .transpose();
    let (interruption, facts_error) = match interruption {
        Ok(value) => (serde_json::json!(value), None),
        Err(error) => (serde_json::Value::Null, Some(error.to_string())),
    };
    let persisted = checkpoint
        .map(|checkpoint| {
            checkpoint.record_interrupted(&receipts, unavailable, unstarted, phase, message)
        })
        .transpose();
    let (checkpoint_value, persistence_error) = match persisted {
        Ok(value) => (serde_json::json!(value), None),
        Err(error) => (serde_json::Value::Null, Some(error.to_string())),
    };
    // This is an interruption report, never the strict complete receipt schema.
    // Even a checkpoint publication failure keeps actual outputs visible while
    // preserving both failures and a nonzero exit status.
    let output = serde_json::json!({
        "state": "partial", "passed": false, "phase": phase,
        "diagnostic": message, "completedReceipts": receipts,
        "postExecutionIdentityValidated": false,
        "unavailableNodes": unavailable.iter().map(|(node, error)| serde_json::json!({
            "nodeId": node, "executionState": "unknown", "diagnostic": error,
        })).collect::<Vec<_>>(),
        "unstartedNodes": unstarted,
        "interruption": interruption, "interruptionFactsError": facts_error,
        "unvalidatedExecutionObservations": unvalidated_observations,
        "checkpoint": checkpoint_value, "checkpointPersistenceError": persistence_error,
    });
    match serde_json::to_string_pretty(&output) {
        Ok(text) => println!("{text}"),
        Err(error) => eprintln!("could not serialize partial verification output: {error}"),
    }
    match persistence_error {
        Some(error) => {
            anyhow::anyhow!("{message}; interruption checkpoint publication failed: {error}")
        }
        None => anyhow::anyhow!("{message}"),
    }
}

fn merge_verification_runs(
    mut runs: Vec<cockpit_repository::RepositoryVerificationRun>,
) -> Option<cockpit_repository::RepositoryVerificationRun> {
    let mut merged = runs.pop()?;
    for mut run in runs {
        merged.receipt.timeout_seconds = merged
            .receipt
            .timeout_seconds
            .max(run.receipt.timeout_seconds);
        merged.receipt.results.append(&mut run.receipt.results);
        merged
            .receipt
            .execution_records
            .append(&mut run.receipt.execution_records);
        merged
            .receipt
            .receipt_candidates
            .append(&mut run.receipt.receipt_candidates);
        merged.receipt.nodes_planned += run.receipt.nodes_planned;
        merged.receipt.nodes_executed += run.receipt.nodes_executed;
        merged.receipt.nodes_reused += run.receipt.nodes_reused;
        merged.receipt.rerun_stale += run.receipt.rerun_stale;
        merged.receipt.rerun_unknown += run.receipt.rerun_unknown;
        merged.receipt.protected_nodes_executed += run.receipt.protected_nodes_executed;
        merged.receipt.protected_nodes_skipped += run.receipt.protected_nodes_skipped;
        merged.receipt.planning_elapsed_ms = merged
            .receipt
            .planning_elapsed_ms
            .max(run.receipt.planning_elapsed_ms);
        merged.receipt.execution_elapsed_ms = merged
            .receipt
            .execution_elapsed_ms
            .max(run.receipt.execution_elapsed_ms);
        merged.receipt.processes_spawned += run.receipt.processes_spawned;
        merged.receipt.max_concurrent_processes = merged
            .receipt
            .max_concurrent_processes
            .max(run.receipt.max_concurrent_processes);
        merged.receipt.process_spawn_failures += run.receipt.process_spawn_failures;
        merged.receipt.git_calls += run.receipt.git_calls;
        merged.receipt.files_read += run.receipt.files_read;
        merged.receipt.files_hashed += run.receipt.files_hashed;
        merged.receipt.elapsed_ms += run.receipt.elapsed_ms;
        merged.receipt.passed &= run.receipt.passed;
    }
    merged
        .receipt
        .results
        .sort_by(|left, right| left.node_id.cmp(&right.node_id));
    merged
        .receipt
        .execution_records
        .sort_by(|left, right| left.node_id.cmp(&right.node_id));
    Some(merged)
}

fn valid_cli_git_object_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[derive(Debug)]
struct DeclaredVerificationCommand {
    node_id: Option<String>,
    program: String,
    args: Vec<String>,
}

fn declared_verification_commands(
    root: &Path,
    work_item_id: &str,
    archived_recovery: bool,
) -> Result<Vec<DeclaredVerificationCommand>> {
    let contract_directory = if archived_recovery {
        "archive"
    } else {
        "active"
    };
    let contract_path = root
        .join(".ai/work-items")
        .join(contract_directory)
        .join(format!("{work_item_id}.contract.json"));
    let contract: cockpit_protocol::Contract = serde_json::from_slice(
        &fs::read(&contract_path)
            .with_context(|| format!("read Work Item Contract {}", contract_path.display()))?,
    )
    .with_context(|| format!("parse Work Item Contract {}", contract_path.display()))?;
    declared_verification_commands_from_declarations(contract.verification, root)
}

fn declared_verification_commands_from_declarations(
    declarations: Vec<VerificationDeclaration>,
    root: &Path,
) -> Result<Vec<DeclaredVerificationCommand>> {
    let repository = root.to_string_lossy().into_owned();
    declarations
        .into_iter()
        .map(|declaration| {
            let (node_id, command) = match declaration {
                VerificationDeclaration::Legacy(command) => (None, command),
                VerificationDeclaration::Check(check) => (Some(check.check.clone()), check.check),
            };
            let (program, args) = parse_declared_verification_command(&command)?;
            let replace_repository = |value: String| {
                if value == "<repo>" {
                    repository.clone()
                } else {
                    value
                }
            };
            Ok(DeclaredVerificationCommand {
                node_id,
                program: replace_repository(program),
                args: args.into_iter().map(replace_repository).collect(),
            })
        })
        .collect()
}

fn detected_verification_commands(root: &Path) -> Result<Vec<(String, Vec<String>)>> {
    if root.join("Cargo.toml").is_file() {
        let mut args = vec!["test".into()];
        if root.join("Cargo.lock").is_file() {
            args.push("--locked".into());
        }
        args.push("--workspace".into());
        Ok(vec![("cargo".into(), args)])
    } else if root.join("package.json").is_file() {
        Ok(vec![("npm".into(), vec!["test".into()])])
    } else {
        anyhow::bail!("no verified project command detected; provide --command")
    }
}

fn parse_declared_verification_command(declaration: &str) -> Result<(String, Vec<String>)> {
    cockpit_protocol::parse_verification_argv(declaration).map_err(anyhow::Error::msg)
}

fn concurrent_phase_elapsed(durations: impl IntoIterator<Item = u128>) -> u128 {
    durations.into_iter().max().unwrap_or_default()
}

fn select_single_provider(
    repo: &std::path::Path,
    requested: Option<AgentProviderArg>,
) -> Result<AgentProvider> {
    if let Some(provider) = requested.and_then(|value| value.provider()) {
        return Ok(provider);
    }
    let report = cockpit_agent::doctor(repo).context("inspect installed agent adapters")?;
    let installed = report
        .adapters
        .into_iter()
        .filter(|adapter| adapter.state == "installed")
        .map(|adapter| adapter.provider)
        .collect::<Vec<_>>();
    if installed.len() != 1 {
        anyhow::bail!(
            "provider selection requires exactly one installed adapter; found {}",
            installed.len()
        );
    }
    Ok(installed.into_iter().next().expect("one provider"))
}

fn parse_classification(value: &str) -> Result<DataClassification> {
    match value.to_ascii_lowercase().as_str() {
        "public" => Ok(DataClassification::Public),
        "internal" => Ok(DataClassification::Internal),
        "confidential" => Ok(DataClassification::Confidential),
        "restricted" => Ok(DataClassification::Restricted),
        "secret_prohibited" | "secret-prohibited" => Ok(DataClassification::SecretProhibited),
        _ => anyhow::bail!("unknown evidence classification: {value}"),
    }
}

fn parse_persistence(value: &str) -> Result<EvidencePersistence> {
    match value.to_ascii_lowercase().as_str() {
        "full_capture" | "full-capture" => Ok(EvidencePersistence::FullCapture),
        "redacted_capture" | "redacted-capture" => Ok(EvidencePersistence::RedactedCapture),
        "digest_only" | "digest-only" => Ok(EvidencePersistence::DigestOnly),
        "no_persistence" | "no-persistence" => Ok(EvidencePersistence::NoPersistence),
        _ => anyhow::bail!("unknown evidence persistence: {value}"),
    }
}

fn print_lifecycle_result(
    repo: &std::path::Path,
    work_item_id: &str,
    receipt: &cockpit_repository::LifecycleReceipt,
    runtime: &cockpit_protocol::RuntimeContext,
    json: bool,
    archive_delivery: bool,
    requested_language: Option<&str>,
) -> Result<()> {
    // Archive delivery is assembled from the same validated observation that
    // produces the human body. Avoid reading a persisted Outcome first and
    // then combining it with a later observation.
    let mut output = serde_json::to_value(receipt)?;
    let mut handoff = None;
    if archive_delivery {
        match prepare_archive_outcome_delivery(
            repo,
            work_item_id,
            runtime,
            output_language(requested_language),
        ) {
            Ok(delivery) => {
                let result = cockpit_agent::deliver_prepared_outcome(
                    cockpit_agent::OutcomeDeliveryApplicationRequest {
                        repository: repo,
                        work_item_id,
                        delivery,
                        supplied_progress: None,
                    },
                )
                .map_err(|error| anyhow::anyhow!(error))?;
                if output.get("outcome").is_none() {
                    if let Some(outcome) = result.delivery.legacy_outcome.as_ref() {
                        output["outcome"] = outcome.clone();
                    } else if let Some(outcome) = result.delivery.outcome.as_ref() {
                        output["outcome"] = serde_json::to_value(outcome)?;
                    }
                }
                if !json {
                    handoff = Some(result.handoff.clone());
                }
                output["outcomeDelivery"] = serde_json::to_value(&result.delivery)?;
                // Keep the complete human handoff in the machine response as
                // well. stdout-only adapters must not reconstruct or lose the
                // body that is emitted on stderr for human invocations.
                output["humanHandoff"] = json!(result.handoff);
                output["assistantMessageEvents"] = serde_json::to_value(
                    cockpit_agent::assistant_message_events(&result.delivery),
                )?;
                output["deliveryReport"] = result.delivery_report_json()?;
                output["hostDeliveryMode"] = json!(result.host_delivery_mode);
                output["hostDisplayConfirmation"] = json!(result.host_display_confirmation);
                if let Some(count) = result.returned_segment_events {
                    output["returnedSegmentEvents"] = json!(count);
                }
            }
            Err(error) => {
                // Archive already succeeded. Preserve the lifecycle Outcome
                // in the machine response while reporting delivery
                // preparation failure separately; never make a consumer
                // re-archive merely because the handoff projection failed.
                output["outcomeDelivery"] = json!({
                    "schemaVersion": cockpit_repository::OUTCOME_DELIVERY_SCHEMA_VERSION,
                    "workItemId": work_item_id,
                    "language": output_language(requested_language),
                    "view": cockpit_protocol::WORK_ITEM_OUTCOME_VIEW_FULL,
                    "deliveryState": "preparation_failed",
                    "hostConfirmation": "unknown",
                    "body": null,
                    "bodySummary": "Full Outcome delivery preparation failed.",
                    "segments": [],
                    "error": error.to_string(),
                    "nextAction": "Keep the archive; inspect the preparation error and retry delivery without archiving again."
                });
                output["assistantMessageEvents"] = json!([]);
                let failure_handoff = format!(
                    "Archive completed, but full Outcome delivery preparation failed: {error}"
                );
                output["humanHandoff"] = json!(failure_handoff);
                if !json {
                    handoff = Some(failure_handoff);
                }
            }
        }
    } else if !json {
        let input =
            cockpit_repository::outcome_render_input_with_runtime(repo, work_item_id, runtime)
                .context("read lifecycle Outcome handoff")?;
        output["outcome"] = input
            .legacy_outcome
            .clone()
            .unwrap_or(serde_json::to_value(&input.outcome)?);
        handoff = Some(cockpit_repository::render_full_human_outcome(
            &input,
            output_language(requested_language),
        ));
    } else {
        let input =
            cockpit_repository::outcome_render_input_with_runtime(repo, work_item_id, runtime)
                .context("read lifecycle Outcome JSON")?;
        output["outcome"] = input
            .legacy_outcome
            .clone()
            .unwrap_or(serde_json::to_value(&input.outcome)?);
    }
    println!("{}", serde_json::to_string_pretty(&output)?);
    if let Some(handoff) = handoff {
        eprintln!("{handoff}");
    }
    Ok(())
}

fn emit_blocked_lifecycle_handoff(
    repo: &std::path::Path,
    work_item_id: &str,
    runtime: &cockpit_protocol::RuntimeContext,
    requested_language: Option<&str>,
) {
    if let Ok(input) =
        cockpit_repository::outcome_render_input_with_runtime(repo, work_item_id, runtime)
    {
        eprintln!(
            "{}",
            cockpit_repository::render_human_outcome(&input, output_language(requested_language),)
        );
    }
}

fn next_admitted_work_item_action(
    repo: &Path,
    work_item_id: &str,
    runtime: &RuntimeContext,
) -> Result<serde_json::Value> {
    let status =
        cockpit_repository::work_item_status_snapshot_with_runtime(repo, work_item_id, runtime)
            .context("refresh Runtime action admission after Contract amendment")?;
    let status = serde_json::to_value(status).context("serialize Work Item status")?;
    Ok(status
        .pointer("/actionExplanation/recommendedAction")
        .cloned()
        .unwrap_or(serde_json::Value::Null))
}

fn require_compatible(
    repo: &std::path::Path,
    runtime: &cockpit_protocol::RuntimeContext,
) -> Result<()> {
    // Explicit `--repo` is still required at the CLI boundary. For a fresh,
    // unattached repository, preserve the legacy ephemeral verification path;
    // once `.ai/cockpit.toml` exists, every stateful/evidence operation is
    // governed by the repository compatibility gate below.
    if !["cockpit.toml", "project.json", "agent-interface.json"]
        .iter()
        .all(|name| repo.join(".ai").join(name).is_file())
    {
        return Ok(());
    }
    let report = cockpit_repository::compatibility_report(repo, runtime)
        .context("read repository compatibility")?;
    if report.state != "COMPATIBLE" {
        anyhow::bail!(
            "repository compatibility is {}; run ai-cockpit migrate plan --repo <repository> and apply the reviewed migration before continuing",
            report.state
        );
    }
    Ok(())
}

fn select_auto_provider(repo: &std::path::Path) -> Result<AgentProvider> {
    let candidates = cockpit_agent::detect_providers(repo)
        .context("detect agent surfaces")?
        .into_iter()
        .filter(|item| item.state == "available" && item.conflict.is_none())
        .collect::<Vec<_>>();
    let targets = candidates
        .iter()
        .map(|item| item.target.clone())
        .collect::<std::collections::BTreeSet<_>>();
    if targets.len() == 1 {
        if let Some(item) = candidates
            .iter()
            .find(|item| item.provider == AgentProvider::Codex)
        {
            return Ok(item.provider.clone());
        }
        if let Some(item) = candidates.into_iter().next() {
            return Ok(item.provider);
        }
    }
    anyhow::bail!(
        "--provider auto requires one unambiguous safe surface; found {} (choose an explicit provider)",
        targets.len()
    )
}

fn agent_exit_code(state: &str) -> i32 {
    match state {
        "VERIFIED" | "CONNECTED" => AgentExitCode::Ready.code(),
        "DEGRADED" | "ATTACHED" | "DISCOVERY_AVAILABLE" | "ADAPTER_INSTALLED" => {
            AgentExitCode::Degraded.code()
        }
        "UNATTACHED" => AgentExitCode::ConfigurationError.code(),
        "CONFLICT" => AgentExitCode::InterventionRequired.code(),
        _ => AgentExitCode::ConfigurationError.code(),
    }
}

fn contains_runtime_code(path: &std::path::Path) -> bool {
    std::fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .any(|entry| {
            let child = entry.path();
            if child.is_dir() {
                contains_runtime_code(&child)
            } else {
                matches!(
                    child.extension().and_then(|extension| extension.to_str()),
                    Some("rs" | "py")
                ) || child.file_name().is_some_and(|name| name == "Makefile.ai")
            }
        })
}

#[cfg(test)]
mod tests {
    use super::{
        CapabilityCommand, Cli, CommandKind, WorkItemCommand, concurrent_phase_elapsed,
        declared_verification_commands, merge_verification_runs,
        parse_declared_verification_command, record_ordinary_cleanup_command,
    };
    use clap::{CommandFactory, Parser};
    use cockpit_core::Digest;
    use cockpit_protocol::{
        RuntimeContext, WORK_ITEM_OUTCOME_CANONICAL_DELIVERY,
        WORK_ITEM_OUTCOME_CANONICAL_DISPLAY_TIMEZONE, WORK_ITEM_OUTCOME_CANONICAL_JSON,
        WORK_ITEM_OUTCOME_CANONICAL_LANGUAGE, WORK_ITEM_OUTCOME_CANONICAL_VIEW,
        WORK_ITEM_OUTCOME_CANONICAL_WORK_ITEM_ID, WORK_ITEM_OUTCOME_CLI_DELIVERY,
        WORK_ITEM_OUTCOME_CLI_DISPLAY_TIMEZONE, WORK_ITEM_OUTCOME_CLI_JSON,
        WORK_ITEM_OUTCOME_CLI_LANGUAGE, WORK_ITEM_OUTCOME_CLI_VIEW,
        WORK_ITEM_OUTCOME_CLI_WORK_ITEM_ID, work_item_outcome_interface_description,
        work_item_outcome_parameter_spec_by_canonical,
    };
    use std::{path::Path, process::Command};

    fn audit_public_help(root: clap::Command) -> (usize, Vec<String>) {
        fn declared_nodes(
            node: &clap::Command,
            path: Vec<String>,
            nodes: &mut Vec<Vec<String>>,
            issues: &mut Vec<String>,
        ) {
            if node.is_hide_set() {
                return;
            }
            let label = path.join(" ");
            nodes.push(path.clone());
            let about = node.get_long_about().or_else(|| node.get_about());
            if about.is_none_or(|about| about.to_string().trim().is_empty()) {
                issues.push(format!("{label}: missing command description"));
            }
            // Collect author-declared arguments before build adds automatic
            // help/version flags or propagates global arguments to children.
            for argument in node.get_arguments().filter(|arg| !arg.is_hide_set()) {
                let help = argument.get_long_help().or_else(|| argument.get_help());
                if help.is_none_or(|help| help.to_string().trim().is_empty()) {
                    issues.push(format!("{label}: missing help for {}", argument.get_id()));
                }
            }
            for child in node.get_subcommands() {
                let mut child_path = path.clone();
                child_path.push(child.get_name().into());
                declared_nodes(child, child_path, nodes, issues);
            }
        }
        let mut nodes = Vec::new();
        let mut issues = Vec::new();
        declared_nodes(&root, vec![root.get_name().into()], &mut nodes, &mut issues);
        // Build a separate tree so inherited globals and automatic help/version
        // flags are checked against actual rendered help, not just declarations.
        let mut built = root.clone();
        built.build();
        fn listing<'a>(text: &'a str, spelling: &str) -> Option<&'a str> {
            text.lines().find(|line| {
                if !line.starts_with("  ") {
                    return false;
                }
                // Only the left-hand listing column counts. A spelling in
                // Usage or prose cannot hide a missing options/commands row.
                line.trim_start()
                    .split("  ")
                    .next()
                    .unwrap_or_default()
                    .split(|character: char| {
                        character.is_whitespace() || ",<>[]".contains(character)
                    })
                    .any(|token| token.eq_ignore_ascii_case(spelling))
            })
        }
        fn listing_block(text: &str, spelling: &str) -> Option<String> {
            let row = listing(text, spelling)?;
            let indent = row.len() - row.trim_start().len();
            let mut lines = text.lines().skip_while(|line| *line != row);
            let mut block = lines.next()?.to_owned();
            for line in lines {
                if !line.trim().is_empty() && line.len() - line.trim_start().len() <= indent {
                    break;
                }
                block.push('\n');
                block.push_str(line);
            }
            Some(block)
        }
        fn normalized(text: &str) -> String {
            text.split_whitespace().collect::<Vec<_>>().join(" ")
        }
        fn has_description(text: &str, expected: Option<&clap::builder::StyledStr>) -> bool {
            expected.is_some_and(|expected| {
                let expected = normalized(&expected.to_string());
                !expected.is_empty() && normalized(text).contains(&expected)
            })
        }
        fn rendered_aliases(text: &str) -> Vec<String> {
            let text = normalized(text);
            text.split_once("[aliases: ")
                .or_else(|| text.split_once("[alias: "))
                .map(|(_, values)| {
                    values
                        .split(']')
                        .next()
                        .unwrap_or_default()
                        .split(',')
                        .map(|value| value.trim().to_owned())
                        .collect()
                })
                .unwrap_or_default()
        }
        fn node_at<'a>(root: &'a clap::Command, path: &[String]) -> &'a clap::Command {
            path.iter().skip(1).fold(root, |node, part| {
                node.find_subcommand(part).expect("declared canonical node")
            })
        }
        for path in &nodes {
            let effective = node_at(&built, path);
            // Clap parsing only: do not call main, dispatch commands, or run
            // either private stdio supervisor with a --help argument.
            for flag in ["-h", "--help"] {
                let mut argv = path.clone();
                argv.push(flag.into());
                match root.clone().try_get_matches_from(argv) {
                    Err(error) if error.kind() == clap::error::ErrorKind::DisplayHelp => {
                        let text = error.to_string();
                        let about = if flag == "-h" {
                            effective.get_about().or_else(|| effective.get_long_about())
                        } else {
                            effective.get_long_about().or_else(|| effective.get_about())
                        };
                        if !has_description(&text, about) {
                            issues.push(format!(
                                "{} {flag}: missing rendered command description",
                                path.join(" ")
                            ));
                        }
                        for argument in effective.get_arguments() {
                            let spelling = argument
                                .get_long()
                                .map(|value| format!("--{value}"))
                                .or_else(|| argument.get_short().map(|value| format!("-{value}")))
                                .unwrap_or_else(|| argument.get_id().to_string());
                            let hidden = argument.is_hide_set()
                                || (flag == "-h" && argument.is_hide_short_help_set())
                                || (flag == "--help" && argument.is_hide_long_help_set());
                            let shown = listing(&text, &spelling).is_some();
                            if shown == hidden {
                                issues.push(format!(
                                    "{} {flag}: {} argument {spelling}",
                                    path.join(" "),
                                    if hidden {
                                        "leaked hidden"
                                    } else {
                                        "missing rendered"
                                    }
                                ));
                            }
                            if !hidden {
                                let block = listing_block(&text, &spelling).unwrap_or_default();
                                let help = if flag == "-h" {
                                    argument.get_help().or_else(|| argument.get_long_help())
                                } else {
                                    argument.get_long_help().or_else(|| argument.get_help())
                                };
                                if !has_description(&block, help) {
                                    issues.push(format!(
                                        "{} {flag}: missing rendered help for {spelling}",
                                        path.join(" ")
                                    ));
                                }
                                let aliases = rendered_aliases(&block);
                                let visible_long =
                                    argument.get_visible_aliases().unwrap_or_default();
                                let visible_short =
                                    argument.get_visible_short_aliases().unwrap_or_default();
                                let declared = argument
                                    .get_all_aliases()
                                    .unwrap_or_default()
                                    .into_iter()
                                    .map(|alias| {
                                        (format!("--{alias}"), visible_long.contains(&alias))
                                    })
                                    .chain(
                                        argument
                                            .get_all_short_aliases()
                                            .unwrap_or_default()
                                            .into_iter()
                                            .map(|alias| {
                                                (
                                                    format!("-{alias}"),
                                                    visible_short.contains(&alias),
                                                )
                                            }),
                                    );
                                for (alias, visible) in declared {
                                    if aliases.contains(&alias) != visible {
                                        issues.push(format!(
                                            "{} {flag}: {} argument alias {alias}",
                                            path.join(" "),
                                            if visible {
                                                "missing visible"
                                            } else {
                                                "leaked hidden"
                                            }
                                        ));
                                    }
                                }
                            }
                        }
                        for child in effective.get_subcommands() {
                            let row = listing(&text, child.get_name());
                            if row.is_some() == child.is_hide_set() {
                                issues.push(format!(
                                    "{} {flag}: {} subcommand {}",
                                    path.join(" "),
                                    if child.is_hide_set() {
                                        "leaked hidden"
                                    } else {
                                        "missing rendered"
                                    },
                                    child.get_name()
                                ));
                            }
                            if !child.is_hide_set() {
                                let block =
                                    listing_block(&text, child.get_name()).unwrap_or_default();
                                // Clap renders a child's short about in both
                                // parent views, while its own --help uses long_about.
                                if !has_description(
                                    &block,
                                    child.get_about().or_else(|| child.get_long_about()),
                                ) {
                                    issues.push(format!(
                                        "{} {flag}: missing rendered subcommand description {}",
                                        path.join(" "),
                                        child.get_name()
                                    ));
                                }
                                let aliases = rendered_aliases(&block);
                                for alias in child.get_visible_aliases() {
                                    if !aliases.iter().any(|value| value == alias) {
                                        issues.push(format!(
                                            "{} {flag}: missing visible alias {alias}",
                                            path.join(" ")
                                        ));
                                    }
                                }
                                for alias in child.get_aliases() {
                                    if aliases.iter().any(|value| value == alias) {
                                        issues.push(format!(
                                            "{} {flag}: leaked hidden alias {alias}",
                                            path.join(" ")
                                        ));
                                    }
                                }
                            }
                        }
                        for private in [
                            "__verification-supervisor",
                            "__composition-supervisor",
                            "__portable-source-observe",
                            "__portable-native-publish",
                        ] {
                            if text.contains(private) {
                                issues.push(format!(
                                    "{}: leaked private helper {private}",
                                    path.join(" ")
                                ));
                            }
                        }
                    }
                    other => issues.push(format!(
                        "{} {flag}: expected DisplayHelp, got {other:?}",
                        path.join(" ")
                    )),
                }
            }
            // Replace one path component at a time: no Cartesian alias
            // explosion, and no handler or private supervisor invocation.
            for depth in 1..path.len() {
                let command = node_at(&built, &path[..=depth]);
                for alias in command.get_all_aliases() {
                    let mut argv = path.clone();
                    argv[depth] = alias.into();
                    argv.push("--help".into());
                    match root.clone().try_get_matches_from(argv) {
                        Err(error) if error.kind() == clap::error::ErrorKind::DisplayHelp => {
                            let mut canonical = path.clone();
                            canonical.push("--help".into());
                            let expected = root
                                .clone()
                                .try_get_matches_from(canonical)
                                .expect_err("help");
                            if error.to_string() != expected.to_string() {
                                issues.push(format!(
                                    "{}: alias {alias} renders a different command",
                                    path.join(" ")
                                ));
                            }
                        }
                        other => issues.push(format!(
                            "{}: alias {alias} failed: {other:?}",
                            path.join(" ")
                        )),
                    }
                }
            }
        }
        (nodes.len(), issues)
    }

    #[test]
    fn every_public_command_and_authored_argument_has_help() {
        let (count, issues) = audit_public_help(Cli::command());
        assert!(count > 0, "the dynamic inventory must not be empty");
        assert!(
            issues.is_empty(),
            "{} public nodes; help defects:\n{}",
            count,
            issues.join("\n")
        );
    }

    #[test]
    fn help_audit_detects_new_undocumented_members_without_fixed_counts() {
        let documented = clap::Command::new("fixture")
            .about("Documented public root")
            .arg(
                clap::Arg::new("global")
                    .long("global")
                    .global(true)
                    .help("Documented global option"),
            )
            .subcommand(clap::Command::new("child").about("Documented child"))
            .subcommand(
                clap::Command::new("hidden")
                    .hide(true)
                    .subcommand(clap::Command::new("not-public")),
            );
        let (count, issues) = audit_public_help(documented.clone());
        assert_eq!(count, 2);
        assert!(issues.is_empty(), "{issues:?}");
        let (_, issues) = audit_public_help(
            documented
                .clone()
                .subcommand(clap::Command::new("new-command")),
        );
        assert!(
            issues
                .iter()
                .any(|issue| issue == "fixture new-command: missing command description")
        );
        let (_, issues) =
            audit_public_help(documented.arg(clap::Arg::new("new-option").long("new-option")));
        assert!(
            issues
                .iter()
                .any(|issue| issue == "fixture: missing help for new-option")
        );
    }

    #[test]
    fn help_audit_detects_rendering_omissions() {
        let documented = clap::Command::new("fixture")
            .about("Documented root")
            .arg(
                clap::Arg::new("visible")
                    .long("visible")
                    .help("Visible option"),
            )
            .subcommand(clap::Command::new("child").about("Visible child"));
        for (template, member) in [
            ("{name}\n{usage}\n{about}\n{subcommands}", "--visible"),
            ("{name}\n{usage}\n{about}\n{options}", "child"),
        ] {
            let (_, issues) = audit_public_help(documented.clone().help_template(template));
            assert!(
                issues.iter().any(|issue| issue.contains(member)),
                "missing rendered member {member} was not detected: {issues:?}"
            );
        }
    }

    #[test]
    fn help_audit_detects_missing_rendered_descriptions_with_names_preserved() {
        let fixture = clap::Command::new("fixture")
            .about("Distinct short root description")
            .long_about("Distinct long root description")
            .arg(clap::Arg::new("public").long("public")
                .help("Distinct short option description")
                .long_help("Distinct long option description"))
            .subcommand(clap::Command::new("child")
                .about("Distinct short child description")
                .long_about("Distinct long child description"))
            .help_template("{name}\nUsage: {usage}\nOptions:\n  --public\n  -h, --help\nCommands:\n  child\n  help\n");
        let (_, issues) = audit_public_help(fixture);
        for view in ["-h", "--help"] {
            for missing in [
                "command description",
                "help for --public",
                "subcommand description child",
            ] {
                assert!(
                    issues.iter().any(
                        |issue| issue == &format!("fixture {view}: missing rendered {missing}")
                    ),
                    "rendered description omission escaped: {view} / {missing}: {issues:?}"
                );
            }
        }
    }

    #[test]
    fn help_audit_checks_view_specific_text_and_each_argument_alias() {
        let fixture = clap::Command::new("fixture")
            .about("Distinct short root text")
            .long_about("Distinct long root text")
            .arg(
                clap::Arg::new("alpha")
                    .long("alpha")
                    .short('a')
                    .visible_short_alias('b')
                    .short_alias('c')
                    .visible_alias("alpha-alt")
                    .alias("alpha-private")
                    .help("Distinct short alpha text")
                    .long_help("Distinct long alpha text"),
            )
            .arg(
                clap::Arg::new("beta")
                    .long("beta")
                    .visible_aliases(["beta-one", "beta-two"])
                    .help("Distinct beta text"),
            );
        let (_, issues) = audit_public_help(fixture.clone());
        assert!(issues.is_empty(), "{issues:?}");
        for (alpha, beta) in [
            ("-b", "--beta-one"),
            ("--alpha-alt", "--beta-two"),
            ("--alpha-private", "--beta"),
        ] {
            let parsed = fixture
                .clone()
                .try_get_matches_from(["fixture", alpha, "A", beta, "B"])
                .expect("declared aliases preserve the canonical argument identity");
            assert_eq!(
                parsed.get_one::<String>("alpha").map(String::as_str),
                Some("A")
            );
            assert_eq!(
                parsed.get_one::<String>("beta").map(String::as_str),
                Some("B")
            );
        }
        let (_, issues) = audit_public_help(
            fixture
                .clone()
                .help_template("{name}\nDistinct long root text\n{usage}\n{options}\n"),
        );
        assert!(
            issues
                .iter()
                .any(|issue| issue == "fixture -h: missing rendered command description"),
            "{issues:?}"
        );
        assert!(
            !issues
                .iter()
                .any(|issue| issue == "fixture --help: missing rendered command description"),
            "{issues:?}"
        );
        let (_, issues) = audit_public_help(
            fixture
                .clone()
                .mut_arg("alpha", |argument| argument.help("")),
        );
        assert!(
            issues
                .iter()
                .any(|issue| issue == "fixture -h: missing rendered help for --alpha"),
            "{issues:?}"
        );
        assert!(
            !issues
                .iter()
                .any(|issue| issue == "fixture --help: missing rendered help for --alpha"),
            "{issues:?}"
        );
        for annotation in ["", "[aliases: -b, --alpha-alt, -c, --alpha-private]"] {
            let (_, issues) = audit_public_help(fixture.clone().help_template(format!(
                "{{name}}\n{{about}}\nOptions:\n  -a, --alpha <ALPHA>  Distinct short alpha text {annotation}\n  --beta <BETA>  Distinct beta text [aliases: --beta-one, --beta-two]\n  -h, --help  Print help\n"
            )));
            for alias in if annotation.is_empty() {
                [
                    "missing visible argument alias -b",
                    "missing visible argument alias --alpha-alt",
                ]
            } else {
                [
                    "leaked hidden argument alias -c",
                    "leaked hidden argument alias --alpha-private",
                ]
            } {
                assert!(
                    issues.iter().any(|issue| issue.ends_with(alias)),
                    "{alias}: {issues:?}"
                );
            }
        }
    }

    #[test]
    fn help_audit_covers_effective_globals_aliases_and_hidden_members() {
        let documented = clap::Command::new("fixture")
            .version("1.2.3")
            .about("Documented root")
            .arg(
                clap::Arg::new("global")
                    .long("global")
                    .global(true)
                    .visible_alias("global-alt")
                    .alias("global-private")
                    .help("Documented global option"),
            )
            .arg(
                clap::Arg::new("hidden-option")
                    .long("hidden-option")
                    .hide(true),
            )
            .subcommand(
                clap::Command::new("child")
                    .about("Documented child")
                    .visible_alias("c")
                    .alias("private-child-alias"),
            )
            .subcommand(clap::Command::new("hidden-child").hide(true));
        let (_, issues) = audit_public_help(documented.clone());
        assert!(issues.is_empty(), "{issues:?}");
        for alias in ["c", "private-child-alias"] {
            let matches = documented
                .clone()
                .try_get_matches_from(["fixture", "--global-alt", "value", alias])
                .expect("aliases select the canonical command without dispatch");
            assert_eq!(matches.subcommand_name(), Some("child"));
            assert_eq!(
                matches.get_one::<String>("global").map(String::as_str),
                Some("value")
            );
        }
        let help = |path: &[&str]| {
            documented
                .clone()
                .try_get_matches_from(path.iter().copied().chain(["--help"]))
                .expect_err("help")
                .to_string()
        };
        let root = help(&["fixture"]);
        let child = help(&["fixture", "child"]);
        for text in [&root, &child] {
            assert!(text.contains("--global"));
            assert!(text.contains("global-alt"));
            assert!(!text.contains("global-private"));
            assert!(!text.contains("hidden-option"));
        }
        assert!(root.contains("[alias: c]"));
        assert!(!root.contains("private-child-alias"));
        assert!(!root.contains("hidden-child"));
        assert!(root.contains("--help") && root.contains("--version"));
        let version = documented
            .clone()
            .try_get_matches_from(["fixture", "--version"])
            .expect_err("version does not dispatch");
        assert_eq!(version.kind(), clap::error::ErrorKind::DisplayVersion);
        let mut missing_global = documented.clone();
        *missing_global.find_subcommand_mut("child").expect("child") = clap::Command::new("child")
            .about("Documented child")
            .help_template("{name}\n{usage}\n{about}");
        let (_, issues) = audit_public_help(missing_global);
        assert!(
            issues
                .iter()
                .any(|issue| issue.contains("fixture child") && issue.contains("--global")),
            "{issues:?}"
        );
        let (_, issues) = audit_public_help(documented.clone().after_help(
            "  --hidden-option  leaked private option\n  hidden-child  leaked private command",
        ));
        assert!(
            issues
                .iter()
                .any(|issue| issue.contains("leaked hidden argument --hidden-option")),
            "{issues:?}"
        );
        assert!(
            issues
                .iter()
                .any(|issue| issue.contains("leaked hidden subcommand hidden-child")),
            "{issues:?}"
        );
        let (_, issues) = audit_public_help(documented.help_template(
            "{name}\n{usage}\n{options}\nCommands:\n  child  Alias metadata omitted",
        ));
        assert!(
            issues
                .iter()
                .any(|issue| issue.contains("missing visible alias c")),
            "{issues:?}"
        );
    }

    #[test]
    fn help_describes_report_output_and_repeated_revalidation_references() {
        let help = |path: &[&str]| {
            let error = Cli::command()
                .try_get_matches_from(path.iter().copied().chain(["--help"]))
                .expect_err("help does not dispatch a command");
            assert_eq!(error.kind(), clap::error::ErrorKind::DisplayHelp);
            error
                .to_string()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        };
        let gate = help(&["ai-cockpit", "gate"]);
        assert!(gate.contains("output path, relative to --repo"));
        assert!(gate.contains("Overwrites an existing file; paths inside .ai are rejected"));
        let revalidate = help(&["ai-cockpit", "work-item", "revalidate-archived"]);
        assert!(revalidate.contains("Evidence reference; repeat --evidence-ref"));
        assert!(revalidate.contains("Exact policy identifier; repeat --policy-ref"));
        let parsed = Cli::try_parse_from([
            "ai-cockpit",
            "work-item",
            "revalidate-archived",
            "--repo",
            ".",
            "--id",
            "WI-PREVIOUS",
            "--successor",
            "WI-CURRENT",
            "--reason",
            "fixture",
            "--actor",
            "agent:fixture",
            "--authority-source",
            "fixture-only",
            "--resume-condition",
            "explicit fixture",
            "--evidence-ref",
            "one,two",
            "--evidence-ref",
            "three",
            "--policy-ref",
            "alpha,beta",
            "--policy-ref",
            "gamma",
        ])
        .expect("existing repeated-argument semantics");
        let CommandKind::WorkItem {
            command:
                WorkItemCommand::RevalidateArchived {
                    evidence_ref,
                    policy_ref,
                    ..
                },
        } = parsed.command
        else {
            panic!("exact command")
        };
        assert_eq!(evidence_ref, ["one,two", "three"]);
        assert_eq!(policy_ref, ["alpha,beta", "gamma"]);
    }

    #[test]
    fn aggregate_receipt_reports_the_longest_node_deadline() {
        let directory = tempfile::tempdir().expect("fixture root");
        std::fs::create_dir_all(directory.path().join("src")).expect("source directory");
        std::fs::write(
            directory.path().join("Cargo.toml"),
            "[package]\nname = \"aggregate-timeout-fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("manifest");
        std::fs::write(directory.path().join("src/lib.rs"), "pub fn fixture() {}\n")
            .expect("source");
        assert!(
            Command::new("git")
                .arg("init")
                .arg("-q")
                .current_dir(directory.path())
                .status()
                .expect("git init")
                .success()
        );
        let request = |node_id: &str, program: &str, args: Vec<String>| {
            cockpit_repository::RepositoryVerificationRequest {
                node_id: node_id.into(),
                program: program.into(),
                args,
                scope: vec!["**".into()],
                stage: "task".into(),
                runner: "local".into(),
                runtime_digest: Digest::sha256_bytes(b"aggregate-test-runtime").to_string(),
                base_commit: None,
                workers: 1,
                work_item_id: None,
                timeout_seconds: None,
                policy: cockpit_repository::RepositoryVerificationPolicy::NeverReuse,
            }
        };
        let package = cockpit_repository::run_repository_verification(
            directory.path(),
            &request(
                "package",
                "cargo",
                vec![
                    "test".into(),
                    "--package".into(),
                    "aggregate-timeout-fixture".into(),
                ],
            ),
        )
        .expect("package verification");
        let other = cockpit_repository::run_repository_verification(
            directory.path(),
            &request("other", "rustc", vec!["--version".into()]),
        )
        .expect("non-package verification");
        assert_eq!(package.receipt.timeout_seconds, Some(600));
        assert_eq!(other.receipt.timeout_seconds, Some(300));
        let merged = merge_verification_runs(vec![package, other]).expect("aggregate receipt");
        assert_eq!(merged.receipt.timeout_seconds, Some(600));
        assert_eq!(merged.receipt.execution_records.len(), 2);
    }

    #[test]
    fn concurrent_phase_telemetry_uses_wall_time_instead_of_summed_worker_time() {
        assert_eq!(concurrent_phase_elapsed([1_000, 1_200]), 1_200);
        assert_eq!(concurrent_phase_elapsed([]), 0);
    }

    #[test]
    fn declared_verification_parser_preserves_environment_prefix_without_shell_evaluation() {
        assert_eq!(
            parse_declared_verification_command(
                "CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/verify cargo test --locked",
            )
            .expect("parse declaration"),
            (
                "env".into(),
                vec![
                    "CARGO_INCREMENTAL=0".into(),
                    "CARGO_TARGET_DIR=/tmp/verify".into(),
                    "cargo".into(),
                    "test".into(),
                    "--locked".into(),
                ],
            )
        );
    }

    #[test]
    fn declared_verification_parser_rejects_shell_syntax() {
        let error = parse_declared_verification_command("cargo test | tee result.log")
            .expect_err("shell pipeline must be rejected before spawn");
        assert!(error.to_string().contains("not a supported argv command"));
    }

    #[test]
    fn archived_recovery_reads_declared_commands_from_the_archive_contract() {
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let archived = declared_verification_commands(
            &repository,
            "WI-1006-runtime-guide-action-outcome",
            true,
        )
        .expect("archived Contract verification declarations");
        assert!(
            archived.iter().any(|command| {
                command.program == "cargo"
                    && command
                        .args
                        .iter()
                        .any(|argument| argument == "--workspace")
            }),
            "archived recovery must retain the workspace verification declaration: {archived:?}"
        );
        assert!(
            archived.iter().any(|command| {
                command.program == "python3"
                    && command
                        .args
                        .iter()
                        .any(|argument| argument == &repository.to_string_lossy())
            }),
            "the repository placeholder must be resolved against the explicit repo root: {archived:?}"
        );

        let active_error = declared_verification_commands(
            &repository,
            "WI-1006-runtime-guide-action-outcome",
            false,
        )
        .expect_err("the predecessor must not regain an active Contract");
        assert!(active_error.to_string().contains("active"));
    }

    #[test]
    fn ordinary_cleanup_parser_requires_repository_and_work_item_identity() {
        let cli = Cli::try_parse_from([
            "ai-cockpit",
            "work-item",
            "ordinary-cleanup",
            "--repo",
            "/tmp/ordinary-cleanup-repository",
            "--id",
            "WI-ORDINARY-CLEANUP",
        ])
        .expect("parse ordinary cleanup command");

        let CommandKind::WorkItem {
            command: WorkItemCommand::OrdinaryCleanup { repo, id },
        } = cli.command
        else {
            panic!("ordinary-cleanup must parse to its dedicated Work Item command")
        };
        assert_eq!(
            repo,
            std::path::PathBuf::from("/tmp/ordinary-cleanup-repository")
        );
        assert_eq!(id, "WI-ORDINARY-CLEANUP");
    }

    #[test]
    fn start_parser_preserves_repeated_explicit_verification_commands() {
        let cli = Cli::try_parse_from([
            "ai-cockpit",
            "start",
            "--repo",
            "/tmp/repository",
            "--id",
            "WI-START-VERIFY",
            "--intent",
            "declare checks",
            "--goal",
            "avoid an unrelated default",
            "--scope",
            "docs/**",
            "--verification",
            "python3 docs-check.py",
            "--verification",
            "cargo test -p cockpit-cli",
        ])
        .expect("parse start command");
        let CommandKind::Start { verification, .. } = cli.command else {
            panic!("start must parse to the start command");
        };
        assert_eq!(
            verification,
            vec!["python3 docs-check.py", "cargo test -p cockpit-cli"]
        );
    }

    #[test]
    fn capability_description_parser_has_a_read_only_surface_and_format() {
        let cli = Cli::try_parse_from([
            "ai-cockpit",
            "capability",
            "show",
            "--repo",
            "/tmp/interface-description-repository",
            "--surface",
            "work-item-outcome",
            "--format",
            "markdown",
            "--language",
            "zh-CN",
        ])
        .expect("parse interface description command");

        let CommandKind::Capability {
            command:
                CapabilityCommand::Show {
                    repo,
                    surface,
                    format,
                    language,
                },
        } = cli.command
        else {
            panic!("capability description must parse to the existing show entry point");
        };
        assert_eq!(
            repo,
            std::path::PathBuf::from("/tmp/interface-description-repository")
        );
        assert_eq!(surface.as_deref(), Some("work-item-outcome"));
        assert_eq!(format, cockpit_protocol::CAPABILITY_SHOW_FORMAT_MARKDOWN);
        assert_eq!(language, "zh-CN");
    }

    #[test]
    fn capability_description_defaults_and_values_come_from_protocol_facts() {
        let root = Cli::command();
        let show = root
            .find_subcommand("capability")
            .and_then(|command| command.find_subcommand("show"))
            .expect("capability show command");

        let format = show
            .get_arguments()
            .find(|argument| argument.get_id().as_str() == "format")
            .expect("capability format argument");
        assert_eq!(
            format
                .get_default_values()
                .iter()
                .map(|value| value.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            vec![cockpit_protocol::CAPABILITY_SHOW_DEFAULT_FORMAT]
        );
        assert_eq!(
            format
                .get_possible_values()
                .into_iter()
                .map(|value| value.get_name().to_owned())
                .collect::<Vec<_>>(),
            cockpit_protocol::CAPABILITY_SHOW_FORMAT_VALUES
        );

        let language = show
            .get_arguments()
            .find(|argument| argument.get_id().as_str() == "language")
            .expect("capability language argument");
        assert_eq!(
            language
                .get_default_values()
                .iter()
                .map(|value| value.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            vec![cockpit_protocol::CAPABILITY_SHOW_DEFAULT_LANGUAGE]
        );
        assert_eq!(
            language
                .get_possible_values()
                .into_iter()
                .map(|value| value.get_name().to_owned())
                .collect::<Vec<_>>(),
            cockpit_protocol::CAPABILITY_SHOW_LANGUAGE_VALUES
        );
    }

    fn render_public_cli_inventory(mut root: clap::Command, language: &str) -> String {
        fn paths(node: &clap::Command, path: Vec<String>, output: &mut Vec<Vec<String>>) {
            if node.is_hide_set() {
                return;
            }
            output.push(path.clone());
            for child in node.get_subcommands() {
                let mut next = path.clone();
                next.push(child.get_name().into());
                paths(child, next, output);
            }
        }
        fn rows(node: &clap::Command, path: &str, output: &mut String) {
            if node.is_hide_set() {
                return;
            }
            let mut arguments = node
                .get_arguments()
                .filter(|argument| !argument.is_hide_set())
                .map(|argument| {
                    let mut names = Vec::new();
                    if let Some(short) = argument.get_short() {
                        names.push(format!("-{short}"));
                    }
                    if let Some(long) = argument.get_long() {
                        names.push(format!("--{long}"));
                    }
                    if names.is_empty() {
                        names.extend(
                            argument
                                .get_value_names()
                                .into_iter()
                                .flatten()
                                .map(ToString::to_string),
                        );
                        if names.is_empty() {
                            names.push(argument.get_id().to_string());
                        }
                    }
                    names.extend(
                        argument
                            .get_visible_short_aliases()
                            .unwrap_or_default()
                            .iter()
                            .map(|alias| format!("-{alias}")),
                    );
                    names.extend(
                        argument
                            .get_visible_aliases()
                            .unwrap_or_default()
                            .iter()
                            .map(|alias| format!("--{alias}")),
                    );
                    format!("`{}`", names.join(" / "))
                })
                .collect::<Vec<_>>();
            arguments.sort();
            let aliases = node
                .get_visible_aliases()
                .map(|alias| format!("`{alias}`"))
                .collect::<Vec<_>>();
            output.push_str(&format!(
                "| `{path}` | {} | {} |\n",
                arguments.join(", "),
                aliases.join(", ")
            ));
        }
        // Capture authored public paths before Clap builds its virtual
        // `help <command>...` tree. Those nodes describe help routing, not a
        // second set of executable commands with the target's argv.
        let mut canonical_paths = Vec::new();
        paths(&root, vec![root.get_name().into()], &mut canonical_paths);
        root.build();
        let (title, note, command, arguments, aliases) =
            match cockpit_protocol::normalize_work_item_outcome_language(language) {
                "zh" => (
                    "完整 CLI 目录",
                    "从当前 Clap 命令树生成；包括继承的全局参数与内建 help/version 参数。这里只列名称，具体语义请查询对应命令的 --help；help [COMMAND]... 使用相同的公开命令路径。",
                    "命令",
                    "参数名称",
                    "可见命令别名",
                ),
                "ja" => (
                    "完全な CLI 一覧",
                    "現在の Clap command tree から生成。継承した global parameter と組み込み help/version 引数を含みます。意味は各 command の --help を確認してください。help [COMMAND]... は同じ公開 command path を使います。",
                    "コマンド",
                    "引数名",
                    "公開コマンド別名",
                ),
                _ => (
                    "Complete CLI inventory",
                    "Generated from the current Clap command tree, including inherited global arguments and built-in help/version arguments. Names only; use each command's --help for semantics. The built-in help [COMMAND]... uses these same public command paths.",
                    "Command",
                    "Argument names",
                    "Visible command aliases",
                ),
            };
        let mut output = format!(
            "<!-- AI_COCKPIT_PUBLIC_CLI:BEGIN -->\n## {title}\n\n{note}\n\n<details>\n<summary>{title}</summary>\n\n| {command} | {arguments} | {aliases} |\n| --- | --- | --- |\n"
        );
        for path in canonical_paths {
            let node = path.iter().skip(1).fold(&root, |node, part| {
                node.find_subcommand(part).expect("declared node")
            });
            rows(node, &path.join(" "), &mut output);
        }
        output.push_str("\n</details>\n<!-- AI_COCKPIT_PUBLIC_CLI:END -->\n");
        output
    }

    fn unique_document_region<'a>(document: &'a str, begin: &str, end: &str) -> Option<&'a str> {
        if document.matches(begin).count() != 1 || document.matches(end).count() != 1 {
            return None;
        }
        let start = document.find(begin)?;
        let finish = document.find(end)?;
        (finish >= start).then(|| &document[start..finish + end.len()])
    }

    #[test]
    fn all_discoverable_interface_facts_have_current_three_language_regions() {
        let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/reference");
        let mut missing = Vec::new();
        for (file, language) in [
            ("commands.md", "en"),
            ("commands.zh-CN.md", "zh-CN"),
            ("commands.ja.md", "ja"),
        ] {
            let document = std::fs::read_to_string(docs.join(file))
                .expect("complete source checkout")
                .replace("\r\n", "\n");
            let mut generated = Vec::new();
            generated.push(render_public_cli_inventory(Cli::command(), language));
            for surface in cockpit_protocol::CAPABILITY_SHOW_SURFACE_VALUES {
                let description = cockpit_protocol::interface_description_for_surface(surface)
                    .expect("registered description");
                let region =
                    cockpit_protocol::render_interface_description_markdown(&description, language);
                generated.push(region);
            }
            for region in &generated {
                let begin = region.lines().next().expect("begin marker");
                let end = region.lines().last().expect("end marker");
                if unique_document_region(&document, begin, end) != Some(region.trim_end()) {
                    missing.push(format!("{file}: missing, stale or ambiguous {begin}"));
                }
                for private in [
                    "__verification-supervisor",
                    "__composition-supervisor",
                    "__portable-source-observe",
                    "__portable-native-publish",
                ] {
                    assert!(!region.contains(private));
                }
            }
            if !missing.is_empty() {
                // Diagnostic source facts only; the test never edits reference
                // files, runs a handler or exposes a production dump command.
                eprintln!(
                    "REFERENCE_REGIONS_JSON {}",
                    serde_json::json!({"file":file,"regions":generated})
                );
            }
        }
        assert!(
            missing.is_empty(),
            "reference drift:\n{}",
            missing.join("\n")
        );
    }

    #[test]
    fn reference_inventory_tracks_new_members_and_rejects_ambiguous_regions() {
        let fixture = clap::Command::new("fixture")
            .arg(clap::Arg::new("global").long("global").global(true))
            .subcommand(clap::Command::new("visible").visible_alias("v"))
            .subcommand(
                clap::Command::new("hidden")
                    .hide(true)
                    .subcommand(clap::Command::new("secret")),
            );
        let rendered = render_public_cli_inventory(fixture.clone(), "en");
        assert!(rendered.contains("| `fixture visible` | `--global`, `-h / --help` | `v` |"));
        assert!(!rendered.contains("hidden") && !rendered.contains("secret"));
        assert!(
            !rendered.contains("| `fixture help visible`"),
            "virtual help paths are not a second command argv"
        );
        let changed = render_public_cli_inventory(
            fixture.subcommand(
                clap::Command::new("new-member")
                    .arg(clap::Arg::new("new-argument").long("new-argument")),
            ),
            "en",
        );
        assert_ne!(rendered, changed);
        assert!(changed.contains("fixture new-member") && changed.contains("--new-argument"));
        let begin = "<!-- begin -->";
        let end = "<!-- end -->";
        let valid = format!("{begin}\nkept\n{end}");
        assert_eq!(
            unique_document_region(&valid, begin, end),
            Some(valid.as_str())
        );
        for invalid in [
            format!("{valid}\n{valid}"),
            format!("{end}\n{begin}"),
            begin.to_owned(),
            end.to_owned(),
        ] {
            assert!(unique_document_region(&invalid, begin, end).is_none());
        }
    }

    fn description_cli_issues(
        mut root: clap::Command,
        description: &cockpit_protocol::InterfaceDescription,
    ) -> Vec<String> {
        use std::{any::TypeId, collections::BTreeSet};
        fn paths(node: &clap::Command, path: Vec<String>, result: &mut Vec<Vec<String>>) {
            if node.is_hide_set() {
                return;
            }
            if !path.is_empty() {
                result.push(path.clone());
            }
            for child in node.get_subcommands() {
                let mut next = path.clone();
                next.push(child.get_name().into());
                paths(child, next, result);
            }
        }
        let mut inventory = Vec::new();
        paths(&root, Vec::new(), &mut inventory);
        let matches = inventory
            .iter()
            .filter(|path| path.join("-") == description.name)
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return vec![format!(
                "{}: ambiguous or missing CLI command",
                description.name
            )];
        }
        root.build();
        let command = matches[0].iter().fold(&root, |node, name| {
            node.find_subcommand(name).expect("canonical node")
        });
        let surfaces = description
            .surfaces
            .iter()
            .filter(|surface| surface.name == "cli")
            .collect::<Vec<_>>();
        if surfaces.len() != 1 {
            return vec![format!(
                "{}: ambiguous or missing CLI description",
                description.name
            )];
        }
        let described = &surfaces[0].parameters;
        let mut issues = Vec::new();
        let expected = described
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect::<BTreeSet<_>>();
        if expected.len() != described.len() {
            issues.push("duplicate described parameter".into());
        }
        let mut actual = command
            .get_arguments()
            .filter(|argument| !argument.is_hide_set())
            .filter(|argument| {
                !matches!(
                    argument.get_action(),
                    clap::ArgAction::Help
                        | clap::ArgAction::HelpShort
                        | clap::ArgAction::HelpLong
                        | clap::ArgAction::Version
                )
            })
            .map(|argument| {
                (
                    argument.get_long().unwrap_or(argument.get_id().as_str()),
                    argument,
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        // Descriptions model request parameters, with CLI context kept explicit:
        // the global task caller is not a request field; Outcome also receives
        // its repository from the surrounding CLI/MCP repository binding.
        match actual.remove("task-caller") {
            Some(argument)
                if argument.is_global_set()
                    && !argument.is_required_set()
                    && argument.get_default_values().is_empty()
                    && argument.get_value_parser().type_id()
                        == TypeId::of::<std::path::PathBuf>() => {}
            _ => issues.push("invalid or missing CLI task-caller context".into()),
        }
        if description.name == cockpit_protocol::WORK_ITEM_OUTCOME_SURFACE {
            match actual.remove("repo") {
                Some(argument)
                    if argument.is_required_set()
                        && argument.get_value_parser().type_id()
                            == TypeId::of::<std::path::PathBuf>() => {}
                _ => issues.push("invalid or missing Outcome repository context".into()),
            }
        }
        if actual.keys().copied().collect::<BTreeSet<_>>() != expected {
            issues.push("CLI parameter inventory differs in either direction".into());
        }
        for parameter in described {
            let Some(argument) = actual.get(parameter.name.as_str()) else {
                continue;
            };
            let defaults = argument
                .get_default_values()
                .iter()
                .map(|value| value.to_string_lossy().into_owned())
                .collect::<Vec<_>>();
            let values = argument
                .get_possible_values()
                .iter()
                .map(|value| value.get_name().to_owned())
                .collect::<Vec<_>>();
            let aliases = argument
                .get_all_aliases()
                .unwrap_or_default()
                .iter()
                .map(|value| value.to_string())
                .chain(
                    argument
                        .get_all_short_aliases()
                        .unwrap_or_default()
                        .iter()
                        .map(char::to_string),
                )
                .collect::<BTreeSet<_>>();
            let parser_type = argument.get_value_parser().type_id();
            let wire_type = if matches!(
                argument.get_action(),
                clap::ArgAction::SetTrue | clap::ArgAction::SetFalse
            ) {
                "boolean"
            } else if !values.is_empty() {
                "enum"
            } else if [
                TypeId::of::<u16>(),
                TypeId::of::<u32>(),
                TypeId::of::<u64>(),
                TypeId::of::<usize>(),
                TypeId::of::<i32>(),
                TypeId::of::<i64>(),
            ]
            .iter()
            .any(|expected| parser_type == *expected)
            {
                "integer"
            } else if parser_type == TypeId::of::<String>()
                || parser_type == TypeId::of::<std::path::PathBuf>()
            {
                "string"
            } else {
                "unsupported"
            };
            if argument.is_required_set() != parameter.required {
                issues.push(format!("{}: required differs", parameter.name));
            }
            if defaults != parameter.default.iter().cloned().collect::<Vec<_>>() {
                issues.push(format!("{}: default differs", parameter.name));
            }
            if values != parameter.enum_values {
                issues.push(format!("{}: enum differs", parameter.name));
            }
            if aliases != parameter.aliases.iter().cloned().collect() {
                issues.push(format!("{}: aliases differ", parameter.name));
            }
            if wire_type != parameter.wire_type {
                issues.push(format!("{}: type differs", parameter.name));
            }
            if parameter.description.trim().is_empty() {
                issues.push(format!("{}: empty description", parameter.name));
            }
        }
        issues
    }

    #[test]
    fn all_discoverable_cli_descriptions_match_actual_arguments_and_reject_drift() {
        for surface in cockpit_protocol::CAPABILITY_SHOW_SURFACE_VALUES {
            let description =
                cockpit_protocol::interface_description_for_surface(surface).expect("description");
            let issues = description_cli_issues(Cli::command(), &description);
            assert!(issues.is_empty(), "{surface}: {issues:?}");
            for field in ["name", "required", "default", "enum", "alias", "type"] {
                let mut changed = description.clone();
                let parameter = &mut changed
                    .surfaces
                    .iter_mut()
                    .find(|surface| surface.name == "cli")
                    .expect("CLI")
                    .parameters[0];
                match field {
                    "name" => parameter.name.push_str("-wrong"),
                    "required" => parameter.required = !parameter.required,
                    "default" => parameter.default = Some("wrong-default".into()),
                    "enum" => parameter.enum_values.push("wrong-enum".into()),
                    "alias" => parameter.aliases.push("wrong-alias".into()),
                    "type" => parameter.wire_type = "wrong-type".into(),
                    _ => unreachable!(),
                }
                assert!(
                    !description_cli_issues(Cli::command(), &changed).is_empty(),
                    "{surface}: {field} drift escaped"
                );
            }
            let mut extra = description.clone();
            let cli = extra
                .surfaces
                .iter_mut()
                .find(|surface| surface.name == "cli")
                .expect("CLI");
            let mut parameter = cli.parameters[0].clone();
            parameter.name = "undisclosed-new-field".into();
            cli.parameters.push(parameter);
            assert!(
                !description_cli_issues(Cli::command(), &extra).is_empty(),
                "extra described field escaped"
            );
            let mut missing = description.clone();
            missing
                .surfaces
                .iter_mut()
                .find(|surface| surface.name == "cli")
                .expect("CLI")
                .parameters
                .pop();
            assert!(
                !description_cli_issues(Cli::command(), &missing).is_empty(),
                "extra actual field escaped"
            );
        }
    }

    #[test]
    fn outcome_description_facts_match_clap_arguments() {
        let root = Cli::command();
        let outcome = root
            .find_subcommand("work-item")
            .and_then(|command| command.find_subcommand("outcome"))
            .expect("work-item outcome command");
        let description = work_item_outcome_interface_description();
        let cli_surface = description
            .surfaces
            .iter()
            .find(|surface| surface.name == "cli")
            .expect("CLI description surface");

        for parameter in &cli_surface.parameters {
            let argument = outcome
                .get_arguments()
                .find(|argument| argument.get_long() == Some(parameter.name.as_str()))
                .unwrap_or_else(|| panic!("missing Clap argument {}", parameter.name));
            let expected_long = match parameter.name.as_str() {
                WORK_ITEM_OUTCOME_CLI_WORK_ITEM_ID => (
                    WORK_ITEM_OUTCOME_CANONICAL_WORK_ITEM_ID,
                    WORK_ITEM_OUTCOME_CLI_WORK_ITEM_ID,
                ),
                WORK_ITEM_OUTCOME_CLI_DELIVERY => (
                    WORK_ITEM_OUTCOME_CANONICAL_DELIVERY,
                    WORK_ITEM_OUTCOME_CLI_DELIVERY,
                ),
                WORK_ITEM_OUTCOME_CLI_JSON => {
                    (WORK_ITEM_OUTCOME_CANONICAL_JSON, WORK_ITEM_OUTCOME_CLI_JSON)
                }
                WORK_ITEM_OUTCOME_CLI_VIEW => {
                    (WORK_ITEM_OUTCOME_CANONICAL_VIEW, WORK_ITEM_OUTCOME_CLI_VIEW)
                }
                WORK_ITEM_OUTCOME_CLI_LANGUAGE => (
                    WORK_ITEM_OUTCOME_CANONICAL_LANGUAGE,
                    WORK_ITEM_OUTCOME_CLI_LANGUAGE,
                ),
                WORK_ITEM_OUTCOME_CLI_DISPLAY_TIMEZONE => (
                    WORK_ITEM_OUTCOME_CANONICAL_DISPLAY_TIMEZONE,
                    WORK_ITEM_OUTCOME_CLI_DISPLAY_TIMEZONE,
                ),
                other => panic!("unexpected CLI interface parameter {other}"),
            };
            assert_eq!(parameter.name, expected_long.1, "{}", expected_long.0);
            assert_eq!(
                work_item_outcome_parameter_spec_by_canonical("cli", expected_long.0)
                    .expect("canonical CLI parameter")
                    .name,
                expected_long.1,
                "{}",
                expected_long.0
            );
            assert_eq!(
                argument.get_long().map(str::to_owned),
                Some(expected_long.1.to_owned()),
                "{}",
                expected_long.0
            );
            assert_eq!(
                argument.is_required_set(),
                parameter.required,
                "{}",
                parameter.name
            );
            assert_eq!(
                argument.get_help().map(ToString::to_string),
                Some(parameter.description.to_owned()),
                "{}",
                parameter.name
            );
            if !parameter.enum_values.is_empty() {
                let values: Vec<_> = argument
                    .get_possible_values()
                    .into_iter()
                    .map(|value| value.get_name().to_owned())
                    .collect();
                assert_eq!(values, parameter.enum_values, "{}", parameter.name);
            }
            if let Some(default) = parameter.default.as_deref()
                && parameter.name == "view"
            {
                assert_eq!(
                    argument
                        .get_default_values()
                        .iter()
                        .map(|value| value.to_string_lossy().into_owned())
                        .collect::<Vec<_>>(),
                    vec![default.to_owned()]
                );
            }
        }
    }

    #[test]
    fn outcome_parser_accepts_the_active_conversation_language() {
        let cli = Cli::try_parse_from([
            "ai-cockpit",
            "work-item",
            "outcome",
            "--repo",
            "/tmp/outcome-language-repository",
            "--id",
            "WI-LANGUAGE",
            "--language",
            "zh-CN",
        ])
        .expect("parse outcome language");

        let CommandKind::WorkItem {
            command: WorkItemCommand::Outcome { query, .. },
        } = cli.command
        else {
            panic!("expected work-item outcome command");
        };
        assert_eq!(
            query.language,
            Some(cockpit_protocol::WorkItemOutcomeLanguage::ZhCn)
        );
        assert_eq!(query.view, cockpit_protocol::WorkItemOutcomeView::Summary);
    }

    #[test]
    fn outcome_parser_returns_the_protocol_owned_query_arguments() {
        let cli = Cli::try_parse_from([
            "ai-cockpit",
            "work-item",
            "outcome",
            "--repo",
            "/tmp/outcome-query-repository",
            "--id",
            "WI-QUERY",
            "--view",
            "full",
            "--language",
            "ja",
        ])
        .expect("parse outcome query");

        let CommandKind::WorkItem {
            command: WorkItemCommand::Outcome { query, .. },
        } = cli.command
        else {
            panic!("expected work-item outcome command");
        };
        assert_eq!(query.id, "WI-QUERY");
        assert_eq!(query.view, cockpit_protocol::WorkItemOutcomeView::Full);
        assert_eq!(
            query.language,
            Some(cockpit_protocol::WorkItemOutcomeLanguage::Ja)
        );
    }

    #[test]
    fn ordinary_cleanup_command_surfaces_repository_boundary_rejection_without_receipt() {
        let directory = tempfile::tempdir().expect("temporary repository");
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .current_dir(directory.path())
                .status()
                .expect("git init")
                .success()
        );
        cockpit_repository::attach(directory.path()).expect("attach repository");
        let before = std::fs::read_dir(directory.path().join(".ai/decisions"))
            .expect("decisions before")
            .count();
        let error = record_ordinary_cleanup_command(
            directory.path(),
            "WI-ORDINARY-CLEANUP-MISSING",
            &RuntimeContext {
                runtime_version: "0.1.0".into(),
                protocol_version: 1,
                runtime_digest: Digest::sha256_bytes(b"cli-ordinary-cleanup-test"),
            },
        )
        .expect_err("missing close binding must be rejected by the repository boundary");

        assert!(error.to_string().contains("record ordinary cleanup"));
        assert_eq!(
            std::fs::read_dir(directory.path().join(".ai/decisions"))
                .expect("decisions after")
                .count(),
            before,
            "a rejected CLI operation must not create a receipt"
        );
    }
}

fn select_declared_nodes(
    requests: Vec<RepositoryVerificationRequest>,
    selected: &[String],
) -> Result<Vec<RepositoryVerificationRequest>> {
    let mut inventory = std::collections::BTreeSet::new();
    for request in &requests {
        if !inventory.insert(request.node_id.clone()) {
            anyhow::bail!("duplicate canonical declared node ID");
        }
    }
    let mut wanted = std::collections::BTreeSet::new();
    for id in selected {
        if !inventory.contains(id) || !wanted.insert(id.clone()) {
            anyhow::bail!("unknown or duplicate declared node ID: {id}");
        }
    }
    Ok(requests
        .into_iter()
        .filter(|r| selected.is_empty() || wanted.contains(&r.node_id))
        .collect())
}
#[cfg(test)]
mod declared_node_selection_tests {
    use super::*;
    #[test]
    fn declared_node_selection_preserves_inventory_order_and_rejects_unknown_duplicate_ids() {
        let request = |id: &str| RepositoryVerificationRequest {
            node_id: id.into(),
            program: "cargo".into(),
            args: vec!["fmt".into()],
            scope: vec!["**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: "runtime".into(),
            base_commit: None,
            workers: 1,
            work_item_id: Some("WI-X".into()),
            timeout_seconds: Some(30),
            policy: RepositoryVerificationPolicy::NeverReuse,
        };
        let selected = select_declared_nodes(
            vec![request("first"), request("second")],
            &["second".into()],
        )
        .unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].node_id, "second");
        assert_eq!(selected[0].program, "cargo");
        assert!(select_declared_nodes(vec![request("first")], &["unknown".into()]).is_err());
        assert!(
            select_declared_nodes(vec![request("first")], &["first".into(), "first".into()])
                .is_err()
        );
        assert!(select_declared_nodes(vec![request("first"), request("first")], &[]).is_err());
        let both = select_declared_nodes(
            vec![request("first"), request("second")],
            &["second".into(), "first".into()],
        )
        .unwrap();
        assert_eq!(
            both.iter().map(|r| r.node_id.as_str()).collect::<Vec<_>>(),
            vec!["first", "second"]
        );
    }
    #[test]
    fn declared_node_parser_rejects_execution_overrides_and_requires_wi() {
        for argv in [
            vec!["ai-cockpit", "verify", "--repo", ".", "--node", "n"],
            vec![
                "ai-cockpit",
                "verify",
                "--repo",
                ".",
                "--work-item",
                "WI-X",
                "--node",
                "n",
                "--command",
                "sh",
            ],
            vec![
                "ai-cockpit",
                "verify",
                "--repo",
                ".",
                "--work-item",
                "WI-X",
                "--node",
                "n",
                "--args",
                "-c,true",
            ],
            vec![
                "ai-cockpit",
                "verify",
                "--repo",
                ".",
                "--work-item",
                "WI-X",
                "--node",
                "n",
                "--archived-recovery",
            ],
        ] {
            assert!(Cli::try_parse_from(argv).is_err());
        }
        assert!(
            Cli::try_parse_from([
                "ai-cockpit",
                "verify",
                "--repo",
                ".",
                "--work-item",
                "WI-X",
                "--node",
                "canonical",
                "--plan-only"
            ])
            .is_ok()
        );
    }
}

fn reconcile_actual_local_node_projection(
    planned_nodes: &mut [serde_json::Value],
    results: &[cockpit_verification::VerificationResult],
) -> anyhow::Result<()> {
    for node in planned_nodes {
        if let Some(result) = results.iter().find(|r| node["nodeId"] == r.node_id) {
            let local_used = result.reused
                && node["localNodeEvidence"]["nodeFactId"].as_str() == result.receipt_id.as_deref();
            if !local_used {
                let object = node.as_object_mut().expect("native planned node");
                if object.remove("localNodeEvidence").is_some() {
                    object.insert(
                        "localNodeReuseReason".into(),
                        json!("planned_candidate_not_used"),
                    );
                }
                object.remove("localNodeCostCompleteness");
                object.insert(
                    "reuseAuthorization".into(),
                    json!(if result.reused {
                        "profile_authorized"
                    } else {
                        "executed"
                    }),
                );
            }
            node["action"] = serde_json::to_value(&result.action)?;
            node["state"] = serde_json::to_value(&result.state)?;
            node["reason"] = json!(result.reason);
            node["satisfiedBy"] = serde_json::to_value(&result.satisfied_by)?;
            node["receiptId"] = json!(result.receipt_id);
        }
    }

    Ok(())
}

#[cfg(test)]
mod local_projection_regression_tests {
    use super::*;
    #[test]
    fn invalid_candidate_fallback_clears_all_reuse_claims() {
        let mut planned = vec![
            json!({"nodeId":"n","action":"reuse","state":"fresh","reason":"fresh_local_node_exact_binding","satisfiedBy":"reused_receipt","receiptId":"old","localNodeEvidence":{"nodeFactId":"old"},"localNodeCostCompleteness":"source_only","reuseAuthorization":"local"}),
        ];
        let result = cockpit_verification::VerificationResult {
            node_id: "n".into(),
            passed: true,
            reused: false,
            protected: false,
            action: cockpit_verification::PlannedAction::Execute,
            state: cockpit_verification::PlannedState::Unknown,
            reason: "candidate_invalidated_before_execution".into(),
            receipt_id: None,
            output_digest: None,
            output_truncated: false,
            timed_out: false,
            satisfied_by: cockpit_verification::PlannedSatisfaction::Execution,
        };
        reconcile_actual_local_node_projection(&mut planned, &[result]).unwrap();
        assert_eq!(planned[0]["action"], "execute");
        assert_eq!(planned[0]["state"], "unknown");
        assert_eq!(
            planned[0]["reason"],
            "candidate_invalidated_before_execution"
        );
        assert_eq!(planned[0]["reuseAuthorization"], "executed");
        assert!(planned[0]["receiptId"].is_null());
        assert!(planned[0].get("localNodeEvidence").is_none());
        assert!(planned[0].get("localNodeCostCompleteness").is_none());
        assert_ne!(planned[0]["satisfiedBy"], "reused_receipt");
    }
}

#[cfg(test)]
mod verify_source_bundle_tests {
    use super::{Cli, CommandKind};
    use clap::{CommandFactory, Parser, error::ErrorKind};
    use std::path::PathBuf;

    // These tests exercise the actual CLI grammar only. They perform no source
    // observation, command execution, source acceptance or formal promotion.
    #[test]
    fn source_bundle_selects_the_full_current_wi_inventory_without_command_overrides() {
        let argv = [
            "ai-cockpit",
            "verify",
            "--repo",
            ".",
            "--work-item",
            "WI-X",
            "--source-bundle",
            "provided-manifest.json",
        ];
        let parsed = Cli::try_parse_from(argv)
            .expect("TARGET RED: generic verify source-bundle route is missing");
        match parsed.command {
            CommandKind::Verify {
                work_item,
                command,
                node,
                args,
                plan_only,
                archived_recovery,
                ..
            } => {
                assert_eq!(work_item.as_deref(), Some("WI-X"));
                assert!(command.is_empty() && node.is_empty() && args.is_empty());
                assert!(!plan_only && !archived_recovery);
            }
            _ => panic!("expected ordinary verify command"),
        }
        let matches = Cli::command().try_get_matches_from(argv).unwrap();
        let verify = matches.subcommand_matches("verify").unwrap();
        assert_eq!(
            verify.get_one::<PathBuf>("source_bundle"),
            Some(&PathBuf::from("provided-manifest.json")),
        );
    }

    #[test]
    fn source_bundle_requires_an_explicit_work_item_before_repository_actions() {
        let error = Cli::try_parse_from([
            "ai-cockpit",
            "verify",
            "--repo",
            ".",
            "--source-bundle",
            "provided-manifest.json",
        ])
        .expect_err("source bundle without a Work Item must not resolve a route");
        assert_eq!(error.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn source_bundle_conflicts_with_every_partial_or_recovery_execution_selector() {
        for conflict in [
            vec!["--command", "git"],
            vec!["--args=--version"],
            vec!["--node", "project-command-0"],
            vec!["--plan-only"],
            vec!["--archived-recovery"],
        ] {
            let mut argv = vec![
                "ai-cockpit",
                "verify",
                "--repo",
                ".",
                "--work-item",
                "WI-X",
                "--source-bundle",
                "provided-manifest.json",
            ];
            argv.extend(conflict);
            let error = Cli::try_parse_from(argv)
                .expect_err("source inventory must not mix with partial or recovery execution");
            assert_eq!(error.kind(), ErrorKind::ArgumentConflict);
        }
    }

    #[test]
    fn source_bundle_parser_preserves_ordinary_verify_routes() {
        for argv in [
            vec!["ai-cockpit", "verify", "--repo", "."],
            vec!["ai-cockpit", "verify", "--repo", ".", "--work-item", "WI-X"],
            vec![
                "ai-cockpit",
                "verify",
                "--repo",
                ".",
                "--command",
                "git",
                "--args=--version",
            ],
        ] {
            let parsed = Cli::try_parse_from(argv).unwrap();
            match parsed.command {
                CommandKind::Verify {
                    node,
                    plan_only,
                    archived_recovery,
                    ..
                } => {
                    assert!(node.is_empty());
                    assert!(!plan_only && !archived_recovery);
                }
                _ => panic!("expected ordinary verify command"),
            }
        }
    }
}

#[cfg(test)]
mod profile_execution_recipe_tests {
    use super::{Cli, CommandKind, ProfileCommand};
    use clap::{CommandFactory, Parser};
    use std::path::PathBuf;

    // Parser-only coverage: no recipe file or project/profile is opened and
    // no executable is invoked or profile confirmation performed.
    #[test]
    fn profile_confirm_accepts_an_explicit_execution_recipe_path() {
        let argv = [
            "ai-cockpit",
            "profile",
            "confirm",
            "--repo",
            "provided-repository",
            "--program",
            "/concrete/cargo",
            "--args=check,--locked",
            "--execution-recipe",
            "provided-recipe.json",
        ];
        let parsed = Cli::try_parse_from(argv)
            .expect("TARGET RED: explicit typed profile execution-recipe CLI input is missing");
        match parsed.command {
            CommandKind::Profile {
                command:
                    ProfileCommand::Confirm {
                        repo,
                        program,
                        args,
                        ..
                    },
            } => {
                assert_eq!(repo, PathBuf::from("provided-repository"));
                assert_eq!(program, "/concrete/cargo");
                assert_eq!(args, vec!["check", "--locked"]);
            }
            _ => panic!("expected profile confirm command"),
        }
        let matches = Cli::command().try_get_matches_from(argv).unwrap();
        let confirm = matches
            .subcommand_matches("profile")
            .unwrap()
            .subcommand_matches("confirm")
            .unwrap();
        assert_eq!(
            confirm.get_one::<PathBuf>("execution_recipe"),
            Some(&PathBuf::from("provided-recipe.json")),
        );
    }

    #[test]
    fn profile_confirm_keeps_the_existing_no_recipe_cli_form() {
        let parsed = Cli::try_parse_from([
            "ai-cockpit",
            "profile",
            "confirm",
            "--repo",
            "provided-repository",
            "--program",
            "/concrete/cargo",
            "--args=check,--locked",
        ])
        .unwrap();
        match parsed.command {
            CommandKind::Profile {
                command:
                    ProfileCommand::Confirm {
                        repo,
                        program,
                        args,
                        ..
                    },
            } => {
                assert_eq!(repo, PathBuf::from("provided-repository"));
                assert_eq!(program, "/concrete/cargo");
                assert_eq!(args, vec!["check", "--locked"]);
            }
            _ => panic!("expected profile confirm command"),
        }
    }
}

/// Dispatch data derived only from the live inventory session at the call site.
/// This private projection never creates an execution or source capability.
struct VerificationSourceDispatch {
    fresh_indices: Vec<usize>,
    force_fresh: std::collections::BTreeSet<String>,
    source_count: usize,
}

fn verification_source_dispatch(
    requests: &[RepositoryVerificationRequest],
    coverage: Option<&cockpit_protocol::ci_admission::PortableInventoryCoverage>,
    actual_source_nodes: &[String],
) -> Result<VerificationSourceDispatch> {
    let Some(coverage) = coverage else {
        if !actual_source_nodes.is_empty() {
            anyhow::bail!("source results without a live inventory session");
        }
        return Ok(VerificationSourceDispatch {
            fresh_indices: (0..requests.len()).collect(),
            force_fresh: Default::default(),
            source_count: 0,
        });
    };
    let declared = requests
        .iter()
        .map(|r| r.node_id.clone())
        .collect::<Vec<_>>();
    let inventory = declared
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    if coverage.schema_version != 1
        || declared.is_empty()
        || inventory.len() != declared.len()
        || coverage.declared_nodes != declared
    {
        anyhow::bail!("source inventory differs from complete current declarations");
    }
    let unique_current = |nodes: &[String]| -> Result<std::collections::BTreeSet<String>> {
        let unique = nodes
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        if unique.len() != nodes.len() || !unique.is_subset(&inventory) {
            anyhow::bail!("duplicate or unknown node in source inventory");
        }
        Ok(unique)
    };
    let sourced = unique_current(&coverage.source_nodes)?;
    let fresh = unique_current(&coverage.fresh_required_nodes)?;
    let observed = unique_current(actual_source_nodes)?;
    if sourced != observed
        || !sourced.is_disjoint(&fresh)
        || sourced.len() + fresh.len() != declared.len()
        || coverage.source_coverage_complete != (sourced.len() == declared.len())
    {
        anyhow::bail!("source inventory is not an exact observed/fresh partition");
    }
    Ok(VerificationSourceDispatch {
        fresh_indices: requests
            .iter()
            .enumerate()
            .filter_map(|(index, request)| fresh.contains(&request.node_id).then_some(index))
            .collect(),
        force_fresh: fresh,
        source_count: sourced.len(),
    })
}

fn verification_local_reuse_inputs<T>(
    candidate: Option<&T>,
    local_fresh_required: bool,
    source_fresh_required: bool,
) -> (Option<&T>, bool) {
    if source_fresh_required {
        (None, true)
    } else {
        (candidate, candidate.is_some() || local_fresh_required)
    }
}

fn verification_interruption_observations(
    source_receipts: &[serde_json::Value],
    existing_observations: &[serde_json::Value],
) -> Vec<serde_json::Value> {
    source_receipts
        .iter()
        .chain(existing_observations)
        .cloned()
        .collect()
}

fn source_bundle_verification_requests(
    root: &Path,
    wi: &str,
    stage: VerificationStage,
    workers: usize,
    timeout_seconds: Option<u64>,
    runtime: &RuntimeContext,
    caller: Option<&cockpit_protocol::TaskAuthorizationCaller>,
) -> Result<Vec<RepositoryVerificationRequest>> {
    cockpit_repository::current_portable_inventory_requests(
        root,
        wi,
        &cockpit_repository::PortableInventoryRoute {
            stage,
            workers,
            timeout_seconds,
        },
        runtime,
        caller,
    )
    .map_err(source_bundle_error)
}

fn source_bundle_error(error: impl std::fmt::Display) -> anyhow::Error {
    anyhow::anyhow!(
        "{error}; check the bundle source or run verify without --source-bundle for native verification"
    )
}

fn verification_error_after_source(
    source_enabled: bool,
    checkpoint: Option<&cockpit_repository::VerificationRunCheckpoint>,
    run: &cockpit_repository::RepositoryVerificationRun,
    phase: &str,
    error: anyhow::Error,
) -> anyhow::Error {
    if source_enabled {
        interrupted_verification_error(
            checkpoint,
            std::slice::from_ref(run),
            &[],
            &[],
            &[],
            phase,
            &error.to_string(),
        )
    } else {
        error
    }
}

#[cfg(test)]
mod verify_source_dispatch_tests {
    use super::*;

    fn requests() -> Vec<RepositoryVerificationRequest> {
        [
            "project-command-0",
            "project-command-51",
            "project-command-2",
        ]
        .into_iter()
        .map(|id| RepositoryVerificationRequest {
            node_id: id.into(),
            program: "git".into(),
            args: vec!["--version".into()],
            scope: vec!["**".into()],
            stage: "task".into(),
            runner: "local".into(),
            runtime_digest: "fixture-runtime".into(),
            base_commit: None,
            workers: 1,
            work_item_id: Some("WI-X".into()),
            timeout_seconds: Some(30),
            policy: RepositoryVerificationPolicy::ProfileAuthorized,
        })
        .collect()
    }

    fn coverage() -> cockpit_protocol::ci_admission::PortableInventoryCoverage {
        cockpit_protocol::ci_admission::PortableInventoryCoverage {
            schema_version: 1,
            declared_nodes: vec![
                "project-command-0".into(),
                "project-command-51".into(),
                "project-command-2".into(),
            ],
            source_nodes: vec!["project-command-51".into()],
            fresh_required_nodes: vec!["project-command-2".into(), "project-command-0".into()],
            source_coverage_complete: false,
            gaps: vec![],
        }
    }

    // Pure dispatch-data tests only. These values do not represent source
    // capabilities and no mock receipt becomes an executable run.
    #[test]
    fn source_dispatch_retains_full_inventory_and_native_fresh_execution_order() {
        let requests = requests();
        let original = requests.clone();
        let partition = verification_source_dispatch(
            &requests,
            Some(&coverage()),
            &["project-command-51".into()],
        )
        .unwrap();
        assert_eq!(partition.fresh_indices, vec![0, 2]);
        assert_eq!(partition.source_count, 1);
        assert_eq!(
            partition.force_fresh.into_iter().collect::<Vec<_>>(),
            vec!["project-command-0", "project-command-2"]
        );
        assert_eq!(requests, original);
        let ordinary = verification_source_dispatch(&requests, None, &[]).unwrap();
        assert_eq!(ordinary.fresh_indices, vec![0, 1, 2]);
        assert_eq!(ordinary.source_count, 0);
        assert!(ordinary.force_fresh.is_empty());
        let mut all_sources = coverage();
        all_sources.source_nodes = all_sources.declared_nodes.clone();
        all_sources.fresh_required_nodes.clear();
        all_sources.source_coverage_complete = true;
        let all =
            verification_source_dispatch(&requests, Some(&all_sources), &all_sources.source_nodes)
                .unwrap();
        assert!(all.fresh_indices.is_empty() && all.force_fresh.is_empty());
        assert_eq!(all.source_count, 3);
        let mut no_sources = coverage();
        no_sources.source_nodes.clear();
        no_sources.fresh_required_nodes = no_sources.declared_nodes.clone();
        let none = verification_source_dispatch(&requests, Some(&no_sources), &[]).unwrap();
        assert_eq!(none.fresh_indices, vec![0, 1, 2]);
        assert_eq!(none.force_fresh.len(), 3);
        assert_eq!(none.source_count, 0);
    }

    #[test]
    fn source_dispatch_rejects_incomplete_ambiguous_or_unobserved_partitions() {
        let requests = requests();
        let mut cases = Vec::new();
        let mut changed = coverage();
        changed.declared_nodes.pop();
        cases.push(changed);
        let mut changed = coverage();
        changed.source_nodes.push("unknown".into());
        cases.push(changed);
        let mut changed = coverage();
        changed.source_nodes.push("project-command-51".into());
        cases.push(changed);
        let mut changed = coverage();
        changed
            .fresh_required_nodes
            .push("project-command-51".into());
        cases.push(changed);
        let mut changed = coverage();
        changed.fresh_required_nodes.pop();
        cases.push(changed);
        let mut changed = coverage();
        changed.source_coverage_complete = true;
        cases.push(changed);
        for coverage in cases {
            assert!(
                verification_source_dispatch(
                    &requests,
                    Some(&coverage),
                    &["project-command-51".into()],
                )
                .is_err()
            );
        }
        assert!(verification_source_dispatch(&requests, Some(&coverage()), &[]).is_err());
        assert!(
            verification_source_dispatch(&requests, None, &["project-command-51".into()]).is_err()
        );
    }

    #[test]
    fn source_required_fresh_removes_the_local_candidate_before_reuse() {
        let candidate = 17;
        let (selected, force_fresh) =
            verification_local_reuse_inputs(Some(&candidate), false, true);
        assert!(selected.is_none() && force_fresh);
        let (selected, force_fresh) =
            verification_local_reuse_inputs(Some(&candidate), false, false);
        assert_eq!(selected, Some(&17));
        assert!(force_fresh);
        let (selected, force_fresh) = verification_local_reuse_inputs::<i32>(None, false, false);
        assert!(selected.is_none() && !force_fresh);
        let (selected, force_fresh) = verification_local_reuse_inputs::<i32>(None, true, false);
        assert!(selected.is_none() && force_fresh);
    }

    #[test]
    fn interruption_retains_each_source_receipt_and_existing_worker_observation() {
        let first = json!({"nodeId":"project-command-51","rawReceipt":"source-a"});
        let second = json!({"nodeId":"project-command-51","rawReceipt":"source-b"});
        let worker =
            json!({"nodeId":"project-command-0","observation":"checkpoint publication failed"});
        let sources = vec![first.clone(), second.clone()];
        let existing = vec![worker.clone()];
        assert_eq!(
            verification_interruption_observations(&sources, &existing),
            vec![first, second, worker]
        );
        assert_eq!(sources.len(), 2);
        assert_eq!(existing.len(), 1);
    }
}

#[cfg(all(test, unix))]
mod source_bundle_canonical_route_tests {
    use super::{Cli, CommandKind};
    use clap::Parser;
    use cockpit_protocol::{RuntimeContext, TaskAuthorizationCaller, VerificationStage};
    use cockpit_repository as repository;
    use serde_json::{Value, json};
    use std::{
        ffi::OsString,
        fs,
        path::{Path, PathBuf},
        process::Command,
    };

    const WI: &str = "WI-CLI-SOURCE-INVENTORY";

    // The following tests exercise Clap -> the actual production request
    // helper -> real native/source SDK consumption. They do not invoke the
    // complete run_verification_command or an external CLI end-to-end, and the
    // trusted SDK worker is not real provider/cross-host authentication.
    fn owned_repository(
        declarations: &[&str],
    ) -> (tempfile::TempDir, RuntimeContext, TaskAuthorizationCaller) {
        let root = tempfile::Builder::new()
            .prefix("cli source route ")
            .tempdir()
            .unwrap();
        fs::write(root.path().join("tracked.txt"), "before\n").unwrap();
        if declarations.iter().any(|text| text.contains("--workspace")) {
            fs::write(
                root.path().join("Cargo.toml"),
                "[workspace]\nmembers = []\nresolver = \"3\"\n",
            )
            .unwrap();
        }
        for args in [
            vec!["init", "-q", "--initial-branch=main"],
            vec!["config", "core.autocrlf", "false"],
            vec!["add", "."],
            vec![
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-qm",
                "fixture",
            ],
        ] {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root.path())
                    .status()
                    .unwrap()
                    .success()
            );
        }
        repository::attach(root.path()).unwrap();
        // Finite AUTHORIZATION ceiling, not a default; installed only in the
        // disposable fixture before native profile/WI creation.
        fs::write(root.path().join(".ai/policy.json"), serde_json::to_vec_pretty(&json!({
            "schemaVersion":1,"project":{"policyId":"cli-source-route-fixture-v1","layer":"project","rules":[{
                "operation":"modify_source","approvalMode":"single_authorized_human","requiredEvidence":[],
                "verificationRequirement":{"schemaVersion":1,"requiredTier":"T0","requiredAssurance":"self_declared",
                    "policyRefs":["cli-source-route-fixture-v1"],"stageRefs":["task","pr"],"gateRefs":[],
                    "maxTimeoutSeconds":900,"reason":"Finite fixture ceiling; default CLI timeout stays None"}
            }]}
        })).unwrap()).unwrap();
        repository::confirm_profile_update(root.path(), "git", &["--version".into()]).unwrap();
        let runtime = RuntimeContext {
            runtime_version: "1.0.1".into(),
            protocol_version: 1,
            runtime_digest: cockpit_core::Digest::sha256_bytes(
                b"cli canonical route fixture; not provider proof",
            ),
        };
        let caller = TaskAuthorizationCaller {
            schema_version: 1,
            actor: "agent:cli-source-route-fixture".into(),
            authority_source: "fixture:synthetic-owner-not-human-or-provider-authentication".into(),
            assurance: cockpit_protocol::AssuranceLevel::SelfDeclared,
        };
        repository::start_work_item_with_options_and_runtime(
            root.path(),
            WI,
            "exercise exact source-bundle CLI inputs",
            "preserve native input and source boundaries",
            &["tracked.txt".into()],
            &repository::WorkItemStartOptions {
                authority: "authorized".into(),
                acceptance_criteria: vec![
                    "only actual native/source facts may replace this exact declared check".into(),
                ],
                verification_commands: declarations.iter().map(|text| (*text).into()).collect(),
                task_authorization: Some(cockpit_protocol::TaskAuthorizationPolicy {
                    schema_version: 1,
                    agent_actor: caller.actor.clone(),
                    authority_source: caller.authority_source.clone(),
                    allowed_actions: vec![
                        "run_preflight".into(),
                        "checkpoint".into(),
                        "run_verification".into(),
                    ],
                    closeout_actions: vec![],
                    expires_at: Some("2099-01-01T00:00:00Z".into()),
                    public_target: None,
                }),
                ..Default::default()
            },
            &runtime,
        )
        .unwrap();
        let contract_path = root
            .path()
            .join(format!(".ai/work-items/active/{WI}.contract.json"));
        repository::preflight_work_item_with_runtime_report(root.path(), &contract_path, &runtime)
            .unwrap();
        let contract: Value = serde_json::from_slice(&fs::read(&contract_path).unwrap()).unwrap();
        let summary: Value = serde_json::from_slice(
            &fs::read(
                root.path()
                    .join(format!(".ai/work-items/active/{WI}.summary.json")),
            )
            .unwrap(),
        )
        .unwrap();
        let recorded_at = summary["updatedAt"]
            .as_str()
            .expect("actual native Summary timestamp");
        repository::record_work_item_governance_controls_with_runtime(root.path(), WI, &json!({"decisionEvidence":{
            "schemaVersion":1,"decisionId":"contract-preflight-review","decision":"confirm_review","workItemId":WI,
            "repositoryId":repository::repository_id(root.path()),"contractDigest":cockpit_protocol::digest_json(&contract).unwrap(),
            "preflightDecisionDigest":summary["preflightDecisionDigest"],"repositorySnapshotDigest":summary["preflightRepositorySnapshotDigest"],
            "recordedAt":recorded_at,"recordedBy":"agent:cli-source-route-test-recorder",
            "reason":"Synthetic disposable fixture; not a real user confirmation or provider authentication."
        }}), &runtime).unwrap();
        repository::checkpoint_work_item_with_caller(root.path(), WI, &runtime, Some(&caller))
            .unwrap();
        (root, runtime, caller)
    }

    fn parsed_source_route(
        root: &Path,
        host: &Path,
        caller: &TaskAuthorizationCaller,
        stage: &str,
        workers: usize,
    ) -> (
        PathBuf,
        String,
        VerificationStage,
        usize,
        Option<u64>,
        PathBuf,
        TaskAuthorizationCaller,
    ) {
        let caller_path = host.join("caller.json");
        fs::write(&caller_path, serde_json::to_vec(caller).unwrap()).unwrap();
        let manifest = host.join("source-bundle.json");
        let argv = vec![
            OsString::from("ai-cockpit"),
            OsString::from("--task-caller"),
            caller_path.into_os_string(),
            OsString::from("verify"),
            OsString::from("--repo"),
            root.as_os_str().to_owned(),
            OsString::from("--work-item"),
            OsString::from(WI),
            OsString::from("--stage"),
            OsString::from(stage),
            OsString::from("--workers"),
            OsString::from(workers.to_string()),
            OsString::from("--source-bundle"),
            manifest.clone().into_os_string(),
        ];
        eprintln!("cli-source-fixture: before actual Clap parse");
        let cli = Cli::try_parse_from(argv).expect("actual Clap source-bundle route");
        eprintln!("cli-source-fixture: parsed actual route");
        let explicit_caller: TaskAuthorizationCaller =
            super::read_json_file(cli.task_caller.as_ref().unwrap(), "explicit task caller")
                .unwrap();
        assert_eq!(
            serde_json::to_value(&explicit_caller).unwrap(),
            serde_json::to_value(caller).unwrap()
        );
        match cli.command {
            CommandKind::Verify {
                repo,
                work_item,
                stage,
                workers,
                timeout_seconds,
                source_bundle,
                command,
                args,
                node,
                plan_only,
                archived_recovery,
                ..
            } => {
                assert!(command.is_empty() && args.is_empty() && node.is_empty());
                assert!(!plan_only && !archived_recovery);
                assert_eq!(timeout_seconds, None, "Clap omission must stay None");
                (
                    repo,
                    work_item.unwrap(),
                    VerificationStage::parse(&stage).unwrap(),
                    workers,
                    timeout_seconds,
                    source_bundle.unwrap(),
                    explicit_caller,
                )
            }
            _ => panic!("expected verify"),
        }
    }

    fn actual_source_round_trip(stage_text: &str) {
        let (root, runtime, fixture_caller) = owned_repository(&["git --version"]);
        let host = tempfile::tempdir().unwrap();
        let host_path = fs::canonicalize(host.path()).unwrap();
        let canonical = fs::canonicalize(root.path()).unwrap();
        let (repo, wi, stage, workers, timeout, manifest, caller) =
            parsed_source_route(&canonical, &host_path, &fixture_caller, stage_text, 1);
        eprintln!("cli-source-fixture: native checkpoint complete; build inventory");
        let requests = super::source_bundle_verification_requests(
            &repo,
            &wi,
            stage,
            workers,
            timeout,
            &runtime,
            Some(&caller),
        )
        .expect("TARGET RED: parsed CLI route must build the same exact native source inventory");
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].stage, stage.as_str());
        assert_eq!(requests[0].workers, 1);
        assert_eq!(
            requests[0].timeout_seconds, None,
            "finite ceiling is never an implicit request timeout"
        );
        let snapshot = cockpit_git::GitRepository::discover(&repo)
            .unwrap()
            .snapshot()
            .unwrap();
        let route =
            repository::resolve_verification_route(&repo, &wi, stage, "local", &snapshot).unwrap();
        assert_eq!(requests[0].base_commit, route.base_revision);
        assert!(requests[0].base_commit.is_some());
        let invocation = repository::CurrentNativeInvocation::new();
        let observed = repository::run_repository_verification_with_native_witness(
            &repo,
            &requests[0],
            &runtime,
            Some(&caller),
            &invocation,
            None,
        )
        .expect("actual bounded native git child under the parsed route");
        assert!(observed.run.receipt.passed);
        assert_eq!(observed.run.receipt.processes_spawned, 1);
        assert_eq!(
            observed.run.receipt.timeout_seconds,
            Some(cockpit_verification::DEFAULT_EXECUTION_SECONDS)
        );
        assert!(
            observed
                .run
                .receipt
                .timeout_seconds
                .expect("one physical node has an actual timeout")
                < 900
        );
        let witness = observed
            .witness
            .expect("actual successful physical child must supply its live witness");
        let published = repository::publish_native_execution_witness(
            &repo,
            &requests[0],
            &runtime,
            Some(&caller),
            &invocation,
            &witness,
        )
        .unwrap();
        repository::persist_verification_attempt(
            &repo,
            &wi,
            &requests,
            &observed.run.final_snapshot,
            &runtime,
            "execution_completed",
            None,
            Some(&serde_json::to_value(&observed.run.receipt).unwrap()),
        )
        .unwrap();
        let input = repo.join(format!(
            ".ai/evidence/{wi}.verification-admissions/portable/objects/{}.bin",
            &published.input_digest.as_str()[7..]
        ));
        let plan =
            repository::plan_portable_evidence(&repo, Some(&wi), &input, &runtime, Some(&caller))
                .unwrap();
        assert!(plan.integrity_valid && !plan.may_replace_execution);
        fs::write(&manifest, serde_json::to_vec(&json!({"schemaVersion":1,"entries":[{
            "nodeId":requests[0].node_id,"input":input,"expectedPlanDigest":plan.plan_digest,"sourceParameters":{}
        }]})).unwrap()).unwrap();
        let counter = host_path.join("reader-count");
        let counter_literal = serde_json::to_string(&counter).unwrap();
        let origin =
            cockpit_core::Digest::sha256_bytes(b"trusted offline CLI SDK host/worker fixture only");
        let code = format!(
            "#!/usr/bin/python3\nimport sys,json\np={counter_literal}\ntry:\n n=int(open(p).read())\nexcept FileNotFoundError:\n n=0\nwith open(p,'w') as f:f.write(str(n+1))\nr=json.loads(sys.argv[-1])\nprint(json.dumps({{'schemaVersion':1,'nonce':r['nonce'],'inputDigest':r['inputDigest'],'sourceRoot':'{origin}'}}))\n"
        );
        let script = host_path.join("controlled-source.py");
        fs::write(&script, &code).unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();
        // The test SDK host fixes these bytes; no parsed caller flag or packet
        // chooses a tool, and no JSON claim constructs the returned live cap.
        let tool = repository::PortableSourceTool::new(
            script,
            cockpit_core::Digest::sha256_bytes(code.as_bytes()),
        );
        let accepted = repository::acquire_portable_evidence_inventory(
            &repo,
            &wi,
            &requests,
            &manifest,
            &tool,
            &runtime,
            Some(&caller),
        )
        .expect(
            "TARGET RED: actual parsed PR/None or Task/None request must survive core acquisition",
        );
        assert_eq!(accepted.len(), 1);
        assert_eq!(fs::read_to_string(&counter).unwrap(), "1");
        let snapshot = cockpit_git::GitRepository::discover(&repo)
            .unwrap()
            .snapshot()
            .unwrap();
        let now = i64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        )
        .unwrap_or(i64::MAX);
        let planned = requests
            .iter()
            .map(|request| {
                repository::plan_repository_verification_action(&repo, request, &snapshot, now)
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let checkpoint = repository::VerificationRunCheckpoint::prepare(
            &repo, &requests, &planned, &snapshot, &runtime,
        )
        .unwrap();
        let own_run = checkpoint.local_run_identity();
        let staged = repository::consume_portable_evidence_inventory(
            &repo,
            &wi,
            &requests,
            accepted,
            &runtime,
            Some(&caller),
            Some(&own_run),
        )
        .expect("TARGET RED: actual CLI-built request must consume its exact live source");
        assert!(staged.coverage().source_coverage_complete);
        assert!(staged.coverage().fresh_required_nodes.is_empty());
        assert_eq!(staged.observed_runs().len(), 1);
        let finalized = staged.finalize(&repo, &requests, &runtime, Some(&caller))
            .expect("TARGET RED: final current/own-run checks must preserve parsed route and None timeout");
        assert_eq!(finalized.runs.len(), 1);
        assert!(finalized.runs[0].receipt.passed);
        assert_eq!(
            (
                finalized.runs[0].receipt.nodes_executed,
                finalized.runs[0].receipt.nodes_reused,
                finalized.runs[0].receipt.processes_spawned
            ),
            (0, 1, 0)
        );
        assert_eq!(
            finalized.runs[0].receipt.timeout_seconds,
            observed.run.receipt.timeout_seconds
        );
        assert_eq!(finalized.source_uses[0].source_reader_invocations, 3);
        assert_eq!(fs::read_to_string(&counter).unwrap(), "3");
        assert!(
            !repo
                .join(format!(".ai/evidence/{wi}.verification.json"))
                .exists(),
            "source-only fixture is not formal lifecycle acceptance"
        );
    }

    // Full native fixture + the real deeply nested Clap tree has more stack
    // than the small parser-only tests. Bound only this test thread, preserve
    // its original panic, and leave production/global stack settings unchanged.
    fn on_cli_fixture_stack(test: impl FnOnce() + Send + 'static) {
        let result = std::thread::Builder::new()
            .name("cli-native-source-fixture".into())
            .stack_size(8 * 1024 * 1024)
            .spawn(test)
            .expect("start bounded-stack fixture thread")
            .join();
        if let Err(panic) = result {
            std::panic::resume_unwind(panic);
        }
    }

    #[test]
    fn source_bundle_pr_default_none_reaches_actual_core_finalization() {
        on_cli_fixture_stack(|| actual_source_round_trip("pr"));
    }

    #[test]
    fn source_bundle_task_finite_ceiling_preserves_none_through_actual_core_finalization() {
        on_cli_fixture_stack(|| actual_source_round_trip("task"));
    }

    fn evidence_files(root: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
        let directory = root.join(".ai/evidence");
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
            Err(error) => panic!("read native observation catalog: {error}"),
        };
        let mut files = entries
            .map(|entry| {
                let path = entry.unwrap().path();
                let metadata = fs::symlink_metadata(&path).unwrap();
                assert!(!metadata.file_type().is_symlink());
                let bytes = if metadata.is_file() {
                    Some(fs::read(&path).unwrap())
                } else {
                    assert!(metadata.is_dir());
                    None
                };
                (path, bytes)
            })
            .collect::<Vec<_>>();
        files.sort_by(|left, right| left.0.cmp(&right.0));
        files
    }

    #[test]
    fn source_bundle_workspace_and_parallel_requests_reject_before_native_or_reader_activity() {
        on_cli_fixture_stack(unsupported_source_bundle_routes);
    }

    fn unsupported_source_bundle_routes() {
        for (declaration, workers, expected) in [
            (
                "cargo test --locked --workspace -- --test-threads=1",
                1,
                "portable_source_bundle_workspace_unsupported",
            ),
            (
                "git --version",
                2,
                "portable_source_bundle_workers_unsupported",
            ),
        ] {
            let declarations = if expected == "portable_source_bundle_workspace_unsupported" {
                vec!["git --version", declaration]
            } else {
                vec![declaration]
            };
            let (root, runtime, fixture_caller) = owned_repository(&declarations);
            let host = tempfile::tempdir().unwrap();
            let host_path = fs::canonicalize(host.path()).unwrap();
            let canonical = fs::canonicalize(root.path()).unwrap();
            let (repo, wi, stage, workers, timeout, _manifest, caller) =
                parsed_source_route(&canonical, &host_path, &fixture_caller, "pr", workers);
            let before = evidence_files(&repo);
            if expected == "portable_source_bundle_workspace_unsupported" {
                let legacy = repository::declared_portable_verification_request(
                    &repo, &wi, "project-command-0", &runtime, Some(&caller),
                ).expect("old single-node selection may choose git beside an unrelated workspace declaration");
                assert_eq!(
                    (legacy.program.as_str(), legacy.args.as_slice()),
                    ("git", ["--version".to_owned()].as_slice())
                );
                assert_eq!(legacy.stage, "task");
                assert_eq!(
                    evidence_files(&repo),
                    before,
                    "legacy declaration lookup is read-only"
                );
            }
            let error = super::source_bundle_verification_requests(
                &repo,
                &wi,
                stage,
                workers,
                timeout,
                &runtime,
                Some(&caller),
            )
            .expect_err(
                "unsupported source-bundle route must stop before observation or execution",
            );
            assert!(
                error.to_string().contains(expected),
                "{expected}: {error:#}"
            );
            assert_eq!(
                evidence_files(&repo),
                before,
                "request rejection must not append any native execution/reader observation"
            );
            assert!(
                !repo
                    .join(format!(".ai/evidence/{wi}.verification.json"))
                    .exists()
            );
            assert!(!host_path.join("reader-count").exists());
        }
    }
}

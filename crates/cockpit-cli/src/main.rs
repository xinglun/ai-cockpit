use anyhow::{Context, Result};
use clap::{ArgAction, Parser, Subcommand};
use cockpit_agent::AgentExitCode;
use cockpit_git::GitRepository;
use cockpit_knowledge::{Query, query};
use cockpit_protocol::{
    AgentProvider, ConcurrencyBoundary, DataClassification, DelegatedEvidence, EvidenceAssurance,
    EvidencePersistence, EvidenceRetention, HumanDecision, RepositoryConfig, VerificationStage,
    VerificationTier, validate_protocol_version,
};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_work_item_with_runtime, attach, checkpoint_work_item,
    close_work_item_with_decision_and_runtime,
    close_work_item_with_structured_decision_and_runtime, finish_work_item_with_runtime,
    generate_knowledge, plan_resource_finalization, preflight_work_item_with_runtime,
    record_resource_finalization, resolve_verification_route, run_repository_verification,
    scaffold_work_item, start_work_item_with_options, verify_resource_finalization,
};
use serde_json::json;
use std::{fs, path::PathBuf};

mod runtime_identity;

#[derive(Debug, Parser)]
#[command(name = "ai-cockpit", version)]
struct Cli {
    #[command(subcommand)]
    command: CommandKind,
}

#[derive(Debug, Subcommand)]
enum CommandKind {
    Inspect {
        #[arg(long)]
        repo: PathBuf,
    },
    Preflight {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        contract: PathBuf,
    },
    Observe {
        #[arg(long)]
        repo: PathBuf,
    },
    Attach {
        #[arg(long)]
        repo: PathBuf,
    },
    Status {
        #[arg(long)]
        repo: PathBuf,
    },
    Compatibility {
        #[arg(long)]
        repo: PathBuf,
    },
    Migrate {
        #[command(subcommand)]
        command: MigrateCommand,
    },
    Start {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
        #[arg(long)]
        intent: String,
        #[arg(long)]
        goal: String,
        #[arg(long, value_delimiter = ',')]
        scope: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        out_of_scope: Vec<String>,
        #[arg(long, default_value = "normal")]
        risk: String,
        #[arg(long, default_value = "missing")]
        authority: String,
        /// Repeat this option for multiple criteria.  Acceptance text is
        /// governance prose and may legitimately contain commas; parsing it
        /// as a delimiter silently changes the Contract bytes.
        #[arg(long, action = ArgAction::Append)]
        acceptance: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        required_evidence: Vec<String>,
    },
    Checkpoint {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
    },
    Finish {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
        /// Emit machine-only output without the human Outcome handoff on stderr.
        #[arg(long)]
        json: bool,
    },
    Archive {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
        /// Emit machine-only output without the human Outcome handoff on stderr.
        #[arg(long)]
        json: bool,
    },
    Close {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
        #[arg(long)]
        human_decision: String,
        #[arg(long)]
        actor: Option<String>,
        #[arg(long)]
        authority_source: Option<String>,
        #[arg(long)]
        reason: Option<String>,
        #[arg(long, value_delimiter = ',')]
        evidence_ref: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        policy_ref: Vec<String>,
        #[arg(long)]
        decided_at: Option<String>,
        #[arg(long)]
        resume_condition: Option<String>,
        /// Emit machine-only output without the human Outcome handoff on stderr.
        #[arg(long)]
        json: bool,
    },
    Verify {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        work_item: Option<String>,
        #[arg(long, action = ArgAction::Append)]
        command: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        args: Vec<String>,
        #[arg(long, default_value_t = 2)]
        workers: usize,
        #[arg(long, default_value = "task")]
        stage: String,
        /// Required for pr/merge/release when no Work Item Contract supplies
        /// the immutable base revision.
        #[arg(long)]
        base_revision: Option<String>,
    },
    /// Evaluate the Contract/policy route without writing repository
    /// governance evidence.  CI uses this before executing its command gate.
    Gate {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        contract: PathBuf,
        #[arg(long, default_value = "pr")]
        stage: String,
        #[arg(long, default_value = "hosted")]
        runner: String,
        #[arg(long)]
        base_revision: Option<String>,
        #[arg(long)]
        report: Option<PathBuf>,
    },
    Evidence {
        #[command(subcommand)]
        command: EvidenceCommand,
    },
    Audit {
        #[command(subcommand)]
        command: AuditCommand,
    },
    WorkItem {
        #[command(subcommand)]
        command: WorkItemCommand,
    },
    Capability {
        #[command(subcommand)]
        command: CapabilityCommand,
    },
    Diagnose {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        work_item: Option<String>,
    },
    Knowledge {
        #[command(subcommand)]
        command: KnowledgeCommand,
    },
    Profile {
        #[command(subcommand)]
        command: ProfileCommand,
    },
    Mcp {
        #[arg(long)]
        repo: PathBuf,
    },
    Doctor {
        #[arg(long)]
        repo: PathBuf,
    },
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },
}

#[derive(Debug, Subcommand)]
enum EvidenceCommand {
    Import {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        work_item: String,
        #[arg(long)]
        metadata: PathBuf,
        #[arg(long)]
        raw: PathBuf,
    },
    List {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        work_item: String,
    },
    Policy {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        work_item: String,
        #[arg(long)]
        classification: String,
        #[arg(long)]
        persistence: String,
        #[arg(long)]
        retention_days: Option<u64>,
        #[arg(long)]
        expires_at: Option<String>,
        #[arg(long)]
        disposal_action: String,
    },
    PurgePlan {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        now_epoch_seconds: Option<u64>,
    },
}

#[derive(Debug, Subcommand)]
enum AuditCommand {
    Export {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum AgentCommand {
    List {
        #[arg(long)]
        repo: PathBuf,
    },
    Install {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long, default_value = "auto")]
        provider: AgentProviderArg,
    },
    Doctor {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        json: bool,
    },
    Repair {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        provider: Option<AgentProviderArg>,
    },
    Detach {
        #[arg(long)]
        repo: PathBuf,
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
    Query {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        topic: Option<String>,
        #[arg(long)]
        component: Option<String>,
        #[arg(long)]
        state: Option<String>,
        #[arg(long)]
        work_item_id: Option<String>,
        #[arg(long)]
        v2: bool,
    },
}

#[derive(Debug, Subcommand)]
enum WorkItemCommand {
    New {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
        #[arg(long)]
        mode: String,
    },
    /// Append a Contract-amendment revalidation without rewriting the
    /// immutable before_edit checkpoint.
    RevalidateAmendment {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
        #[arg(long)]
        reason: String,
    },
    Approach {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
    },
    Outcome {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
        /// Emit the stable machine-readable Outcome JSON instead of the human handoff.
        #[arg(long)]
        json: bool,
    },
    /// Move failed-attempt artifacts left by an older/interrupted archive
    /// into the immutable archive and bind them with a reconciliation receipt.
    ReconcileArtifacts {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
    },
    /// Bind branch/worktree/provider/PR context before archive.
    FinalizePlan {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
        /// JSON ResourceFinalizationContext file.
        #[arg(long)]
        input: PathBuf,
    },
    /// Record a strict provider-side finalization receipt after archive.
    Finalize {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
        /// JSON ResourceFinalizationReceipt file.
        #[arg(long)]
        input: PathBuf,
    },
    /// Revalidate the stored finalization receipt and local cleanup state.
    FinalizeVerify {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
    },
    /// Classify an immutable legacy finalization receipt without rewriting it.
    FinalizeRecovery {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
        /// JSON HistoricalFinalizationRecoveryReceipt file.
        #[arg(long)]
        input: PathBuf,
    },
    /// Inspect immutable legacy finalization facts and print a recovery
    /// skeleton without writing repository state.
    FinalizeRecoveryPlan {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
        /// Optional real merge commit for a historical direct merge without a PR.
        #[arg(long)]
        merge_commit: Option<String>,
    },
    Status {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long, required_unless_present = "all", conflicts_with = "all")]
        id: Option<String>,
        /// Aggregate every active and archived Work Item in stable ID order.
        #[arg(long, conflicts_with = "id")]
        all: bool,
        /// Emit the stable machine-readable status snapshot.
        #[arg(long)]
        json: bool,
    },
    Inspect {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
    },
    Declare {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
        #[arg(long, value_delimiter = ',')]
        depends_on: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        conflicts_with: Vec<String>,
        #[arg(long, default_value_t = false)]
        parallelizable: bool,
    },
    Validate {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
        /// Emit the stable machine-readable validation report.
        #[arg(long)]
        json: bool,
    },
    Controls {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
        /// JSON object containing scenarioCoverage, acceptanceEvidence,
        /// intentAlignment, finalDimensions, or identity-bound decisionEvidence.
        #[arg(long)]
        input: PathBuf,
    },
    Recover {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
        /// JSON RecoveryDecisionReceipt with retry/successor predecessor bindings.
        #[arg(long)]
        input: PathBuf,
    },
    Boundary {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
        /// JSON file containing a Contract concurrencyBoundary object.
        #[arg(long)]
        file: PathBuf,
    },
    Slot {
        #[command(subcommand)]
        command: WorkItemSlotCommand,
    },
}

#[derive(Debug, Subcommand)]
enum WorkItemSlotCommand {
    Acquire {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
    },
    Release {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
        #[arg(long)]
        lease_id: String,
    },
    List {
        #[arg(long)]
        repo: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum CapabilityCommand {
    Show {
        #[arg(long)]
        repo: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum ProfileCommand {
    Confirm {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        program: String,
        #[arg(long, value_delimiter = ',')]
        args: Vec<String>,
    },
    Propose {
        #[arg(long)]
        repo: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum MigrateCommand {
    Plan {
        #[arg(long)]
        repo: PathBuf,
    },
    Apply {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        approved: bool,
    },
}

/// Select the language used by the human handoff. The agent-facing dialog is
/// localized by the conversation layer; the CLI falls back to the user's
/// locale so the same report is useful when invoked directly.
fn output_language() -> &'static str {
    let value = std::env::var("AI_COCKPIT_LANGUAGE")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LANGUAGE"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default()
        .to_ascii_lowercase();
    if value.starts_with("zh") {
        "zh"
    } else if value.starts_with("ja") {
        "ja"
    } else {
        "en"
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let runtime_context = runtime_identity::load_current().context("load runtime identity")?;
    match cli.command {
        CommandKind::Inspect { repo } => {
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
            let decision = preflight_work_item_with_runtime(&repo, &contract, &runtime_context)
                .context("evaluate and record preflight decision")?;
            println!("{}", serde_json::to_string_pretty(&decision)?);
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
        CommandKind::Attach { repo } => {
            let profile = attach(&repo).context("attach repository")?;
            println!("{}", serde_json::to_string_pretty(&profile)?);
        }
        CommandKind::Status { repo } => {
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
        } => {
            require_compatible(&repo, &runtime_context)?;
            let receipt = start_work_item_with_options(
                &repo,
                &id,
                &intent,
                &goal,
                &scope,
                &WorkItemStartOptions {
                    out_of_scope,
                    risk,
                    authority,
                    acceptance_criteria: acceptance,
                    required_evidence_classes: required_evidence,
                },
            )
            .context("start work item")?;
            println!("{}", serde_json::to_string_pretty(&receipt)?);
        }
        CommandKind::Checkpoint { repo, id } => {
            require_compatible(&repo, &runtime_context)?;
            let receipt = checkpoint_work_item(&repo, &id).context("checkpoint work item")?;
            println!("{}", serde_json::to_string_pretty(&receipt)?);
        }
        CommandKind::Finish { repo, id, json } => {
            require_compatible(&repo, &runtime_context)?;
            let receipt = match finish_work_item_with_runtime(&repo, &id, &runtime_context) {
                Ok(receipt) => receipt,
                Err(error) => {
                    if !json {
                        emit_blocked_lifecycle_handoff(&repo, &id, &runtime_context);
                    }
                    return Err(error).context("finish work item");
                }
            };
            print_lifecycle_result(&repo, &id, &receipt, &runtime_context, json)?;
        }
        CommandKind::Archive { repo, id, json } => {
            require_compatible(&repo, &runtime_context)?;
            let receipt = archive_work_item_with_runtime(&repo, &id, &runtime_context)
                .context("archive work item")?;
            print_lifecycle_result(&repo, &id, &receipt, &runtime_context, json)?;
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
            print_lifecycle_result(&repo, &id, &receipt, &runtime_context, json)?;
        }
        CommandKind::Verify {
            repo,
            work_item,
            command,
            args,
            workers,
            stage,
            base_revision,
        } => {
            require_compatible(&repo, &runtime_context)?;
            let stage = VerificationStage::parse(&stage).map_err(|error| anyhow::anyhow!(error))?;
            let root = std::fs::canonicalize(&repo).context("canonicalize repository")?;
            let initial_snapshot = GitRepository::discover(&root)
                .context("discover repository for verification route")?
                .snapshot()
                .context("capture verification route snapshot")?;
            let route = if let Some(work_item_id) = work_item.as_deref() {
                Some(
                    resolve_verification_route(
                        &root,
                        work_item_id,
                        stage,
                        "local",
                        &initial_snapshot,
                    )
                    .context("resolve policy-bound verification route")?,
                )
            } else {
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
            if let Some(work_item_id) = work_item.as_deref() {
                cockpit_repository::require_policy_for_verification(&root, work_item_id)
                    .context("enforce verification policy")?;
            }
            let explicit = !command.is_empty();
            let (programs, command_args) = if explicit {
                (command, args)
            } else if root.join("Cargo.toml").is_file() {
                (
                    vec!["cargo".into()],
                    vec!["test".into(), "--workspace".into()],
                )
            } else if root.join("package.json").is_file() {
                (vec!["npm".into()], vec!["test".into()])
            } else {
                anyhow::bail!("no verified project command detected; provide --command")
            };
            let requests = programs
                .into_iter()
                .enumerate()
                .map(|(index, program)| RepositoryVerificationRequest {
                    node_id: format!("project-command-{index}"),
                    program,
                    args: command_args.clone(),
                    scope: vec!["**".into()],
                    stage: stage.as_str().into(),
                    runner: "local".into(),
                    runtime_digest: runtime_context.runtime_digest.to_string(),
                    base_commit: route
                        .as_ref()
                        .and_then(|route| route.base_revision.clone())
                        .or_else(|| base_revision.clone()),
                    workers,
                    // Detected Work Item commands use the same exact,
                    // identity-bound profile authorization as bare `verify`.
                    // Explicit custom commands remain fresh unless a future
                    // explicit reuse contract is added; this keeps dynamic
                    // reuse precise without treating arbitrary commands as
                    // cacheable.
                    policy: if explicit {
                        RepositoryVerificationPolicy::NeverReuse
                    } else {
                        RepositoryVerificationPolicy::ProfileAuthorized
                    },
                })
                .collect::<Vec<_>>();
            let requires_aggregate_snapshot = requests.len() > 1;
            let service_started = std::time::Instant::now();
            let mut runs = Vec::with_capacity(requests.len());
            let mut planning_elapsed_ms = 0_u128;
            let mut execution_elapsed_ms = 0_u128;
            for batch in requests.chunks(workers.max(1)) {
                let batch_runs = std::thread::scope(|scope| {
                    batch
                        .iter()
                        .map(|request| scope.spawn(|| run_repository_verification(&root, request)))
                        .collect::<Vec<_>>()
                        .into_iter()
                        .map(|worker| worker.join())
                        .collect::<Vec<_>>()
                });
                let mut completed_batch = Vec::with_capacity(batch_runs.len());
                for run in batch_runs {
                    let run = run
                        .map_err(|_| anyhow::anyhow!("repository verification worker panicked"))?
                        .context("execute repository verification")?;
                    completed_batch.push(run);
                }
                planning_elapsed_ms = planning_elapsed_ms.saturating_add(concurrent_phase_elapsed(
                    completed_batch
                        .iter()
                        .map(|run| run.receipt.planning_elapsed_ms),
                ));
                execution_elapsed_ms =
                    execution_elapsed_ms.saturating_add(concurrent_phase_elapsed(
                        completed_batch
                            .iter()
                            .map(|run| run.receipt.execution_elapsed_ms),
                    ));
                runs.extend(completed_batch);
            }
            let mut run = merge_verification_runs(runs).context("merge verification runs")?;
            run.receipt.planning_elapsed_ms = planning_elapsed_ms;
            run.receipt.execution_elapsed_ms = execution_elapsed_ms;
            if requires_aggregate_snapshot {
                let final_snapshot = cockpit_git::GitRepository::discover(&root)
                    .context("discover repository for aggregate snapshot")?
                    .snapshot()
                    .context("capture aggregate verification snapshot")?;
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
            .map_err(|error| anyhow::anyhow!(error))?;
            if let Some(route) = &route {
                plan_receipt.work_item_id = Some(route.work_item_id.clone());
                plan_receipt.repository_id =
                    Some(cockpit_repository::repository_id(&root).to_string());
                plan_receipt.repository_snapshot_digest =
                    Some(cockpit_repository::snapshot_digest(&run.final_snapshot)?.to_string());
                plan_receipt.base_revision = route.base_revision.clone();
                plan_receipt.affected_paths = route.affected_paths.clone();
                plan_receipt.dependency_confidence = Some(route.dependency_confidence);
                if let Some(plan) = &route.policy_plan {
                    plan_receipt.required_tier = Some(plan.requirement.required_tier);
                    plan_receipt.required_assurance = Some(plan.requirement.required_assurance);
                    plan_receipt.policy_refs = plan.requirement.policy_refs.clone();
                }
            }
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
            run.receipt.plan_receipt = Some(plan_receipt);
            let cost_observation = run.receipt.cost_observation();
            run.receipt.cost_observation = Some(cost_observation);
            let mut output = serde_json::to_value(&run.receipt)?;
            output["runtimeVersion"] =
                serde_json::Value::String(runtime_context.runtime_version.clone());
            output["runtimeDigest"] =
                serde_json::Value::String(runtime_context.runtime_digest.to_string());
            if let Some(work_item) = work_item {
                cockpit_repository::record_verification_with_runtime(
                    &root,
                    &work_item,
                    &output,
                    &runtime_context,
                    &run.final_snapshot,
                )
                .context("record verification evidence")?;
            }
            println!("{}", serde_json::to_string_pretty(&output)?);
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
        CommandKind::Evidence { command } => match command {
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
            EvidenceCommand::List { repo, work_item } => {
                require_compatible(&repo, &runtime_context)?;
                let receipts = cockpit_repository::list_delegated_evidence(&repo, &work_item)
                    .context("list delegated evidence")?;
                println!("{}", serde_json::to_string_pretty(&receipts)?);
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
            AuditCommand::Export { repo, output } => {
                require_compatible(&repo, &runtime_context)?;
                let manifest = cockpit_repository::export_audit_events(&repo, &runtime_context)
                    .context("export audit events")?;
                if let Some(output) = output {
                    let bytes = serde_json::to_vec_pretty(&manifest)?;
                    if output.exists() {
                        let existing =
                            std::fs::read(&output).context("read existing audit export")?;
                        if existing != bytes {
                            anyhow::bail!("audit export target already exists with different bytes")
                        }
                    } else {
                        if let Some(parent) = output.parent() {
                            std::fs::create_dir_all(parent)
                                .context("create audit export parent")?;
                        }
                        std::fs::write(&output, &bytes).context("write audit export")?;
                    }
                }
                println!("{}", serde_json::to_string_pretty(&manifest)?);
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
                println!("\nState: {}", receipt.state);
                println!("\n{}", serde_json::to_string_pretty(&receipt)?);
            }
            WorkItemCommand::RevalidateAmendment { repo, id, reason } => {
                require_compatible(&repo, &runtime_context)?;
                let record = cockpit_repository::revalidate_contract_amendment(&repo, &id, &reason)
                    .context("revalidate amended Contract")?;
                println!("{}", serde_json::to_string_pretty(&record)?);
            }
            WorkItemCommand::Approach { repo, id } => {
                require_compatible(&repo, &runtime_context)?;
                let approach = cockpit_repository::implementation_approach(&repo, &id)
                    .context("derive implementation approach")?;
                println!("{}", serde_json::to_string_pretty(&approach)?);
            }
            WorkItemCommand::Outcome { repo, id, json } => {
                require_compatible(&repo, &runtime_context)?;
                let outcome =
                    cockpit_repository::outcome_v2_with_runtime(&repo, &id, &runtime_context)
                        .context("read Work Item outcome")?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&outcome)?);
                } else {
                    println!(
                        "{}",
                        cockpit_repository::render_human_outcome(
                            &repo,
                            &outcome,
                            output_language(),
                        )
                    );
                }
            }
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
                let plan = plan_resource_finalization(&repo, &id, &context)
                    .context("plan resource finalization")?;
                println!("{}", serde_json::to_string_pretty(&plan)?);
            }
            WorkItemCommand::Finalize { repo, id, input } => {
                require_compatible(&repo, &runtime_context)?;
                let receipt = record_resource_finalization(&repo, &id, &input, &runtime_context)
                    .context("record resource finalization")?;
                println!("{}", serde_json::to_string_pretty(&receipt)?);
            }
            WorkItemCommand::FinalizeVerify { repo, id } => {
                require_compatible(&repo, &runtime_context)?;
                let result = verify_resource_finalization(&repo, &id, &runtime_context)
                    .context("verify resource finalization")?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            WorkItemCommand::FinalizeRecovery { repo, id, input } => {
                require_compatible(&repo, &runtime_context)?;
                let result = cockpit_repository::record_historical_finalization_recovery(
                    &repo,
                    &id,
                    &input,
                    &runtime_context,
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
            WorkItemCommand::Status {
                repo,
                id,
                all,
                json,
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
                if json {
                    println!("{}", serde_json::to_string_pretty(&snapshot)?);
                } else {
                    let language = output_language();
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
                let summary =
                    cockpit_repository::record_work_item_governance_controls(&repo, &id, &controls)
                        .context("record Work Item governance controls")?;
                println!("{}", serde_json::to_string_pretty(&summary)?);
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
            CapabilityCommand::Show { repo } => {
                require_compatible(&repo, &runtime_context)?;
                let registry = cockpit_repository::capability_truth_registry_with_runtime(
                    &repo,
                    &runtime_context,
                )
                .context("derive capability truth registry")?;
                println!("{}", serde_json::to_string_pretty(&registry)?);
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
                let index = generate_knowledge(&repo).context("project knowledge")?;
                let after = fs::read(&projection_path).ok();
                let materialization = if before.is_none() {
                    "created"
                } else if before != after {
                    "rebuilt"
                } else {
                    "reused"
                };
                let results = query(
                    &index,
                    &Query {
                        topic,
                        component,
                        state,
                        work_item_id,
                    },
                );
                let output = json!({
                    "schemaVersion": 1,
                    "projection": {
                        "path": ".ai/knowledge/index.json",
                        "materialization": materialization,
                        "writeBoundary": "repository-local-derived",
                        "authority": "none",
                        "sourceDigest": index.source_digest
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
            } => {
                require_compatible(&repo, &runtime_context)?;
                let profile = cockpit_repository::confirm_profile_update(&repo, &program, &args)
                    .context("confirm project profile update")?;
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
        CommandKind::Doctor { repo } => {
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

fn merge_verification_runs(
    mut runs: Vec<cockpit_repository::RepositoryVerificationRun>,
) -> Option<cockpit_repository::RepositoryVerificationRun> {
    let mut merged = runs.pop()?;
    for mut run in runs {
        merged.receipt.results.append(&mut run.receipt.results);
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
    Some(merged)
}

fn valid_cli_git_object_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
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

fn lifecycle_output(
    repo: &std::path::Path,
    work_item_id: &str,
    receipt: &cockpit_repository::LifecycleReceipt,
) -> Result<serde_json::Value> {
    let mut output = serde_json::to_value(receipt)?;
    for directory in ["active", "archive"] {
        let path = repo
            .join(".ai/work-items")
            .join(directory)
            .join(format!("{work_item_id}.outcome.json"));
        if path.is_file() {
            let bytes =
                std::fs::read(&path).with_context(|| format!("read outcome {}", path.display()))?;
            output["outcome"] = serde_json::from_slice(&bytes).context("parse outcome")?;
            break;
        }
    }
    Ok(output)
}

fn print_lifecycle_result(
    repo: &std::path::Path,
    work_item_id: &str,
    receipt: &cockpit_repository::LifecycleReceipt,
    runtime: &cockpit_protocol::RuntimeContext,
    json: bool,
) -> Result<()> {
    let output = lifecycle_output(repo, work_item_id, receipt)?;
    let handoff = if json {
        None
    } else {
        let outcome = cockpit_repository::outcome_v2_with_runtime(repo, work_item_id, runtime)
            .context("read lifecycle Outcome handoff")?;
        Some(cockpit_repository::render_human_outcome(
            repo,
            &outcome,
            output_language(),
        ))
    };
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
) {
    if let Ok(outcome) = cockpit_repository::outcome_v2_with_runtime(repo, work_item_id, runtime) {
        eprintln!(
            "{}",
            cockpit_repository::render_human_outcome(repo, &outcome, output_language(),)
        );
    }
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
    use super::concurrent_phase_elapsed;

    #[test]
    fn concurrent_phase_telemetry_uses_wall_time_instead_of_summed_worker_time() {
        assert_eq!(concurrent_phase_elapsed([1_000, 1_200]), 1_200);
        assert_eq!(concurrent_phase_elapsed([]), 0);
    }
}

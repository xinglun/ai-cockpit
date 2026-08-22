use anyhow::{Context, Result};
use clap::{ArgAction, Parser, Subcommand};
use cockpit_agent::AgentExitCode;
use cockpit_git::GitRepository;
use cockpit_knowledge::{Query, query};
use cockpit_protocol::{
    AgentProvider, ConcurrencyBoundary, DataClassification, DelegatedEvidence, EvidencePersistence,
    EvidenceRetention, HumanDecision, RepositoryConfig, validate_protocol_version,
};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_work_item_with_runtime, attach, checkpoint_work_item,
    close_work_item_with_decision_and_runtime,
    close_work_item_with_structured_decision_and_runtime, finish_work_item_with_runtime,
    generate_knowledge, preflight_work_item_with_runtime, run_repository_verification,
    scaffold_work_item, start_work_item_with_options, status,
};
use serde_json::json;
use std::path::PathBuf;

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
        #[arg(long, value_delimiter = ',')]
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
    },
    Archive {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        id: String,
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
        /// JSON object containing only scenarioCoverage, acceptanceEvidence,
        /// intentAlignment, or finalDimensions.
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
            let observation = cockpit_repository::observe_cached(&snapshot.root, &snapshot)
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
            let repository_status = status(&repo).context("read repository status")?;
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
                let plan = cockpit_repository::migration_plan(&repo)
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
        CommandKind::Finish { repo, id } => {
            require_compatible(&repo, &runtime_context)?;
            let receipt = finish_work_item_with_runtime(&repo, &id, &runtime_context)
                .context("finish work item")?;
            println!(
                "{}",
                serde_json::to_string_pretty(&lifecycle_output(&repo, &id, &receipt)?)?
            );
        }
        CommandKind::Archive { repo, id } => {
            require_compatible(&repo, &runtime_context)?;
            let receipt = archive_work_item_with_runtime(&repo, &id, &runtime_context)
                .context("archive work item")?;
            println!(
                "{}",
                serde_json::to_string_pretty(&lifecycle_output(&repo, &id, &receipt)?)?
            );
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
            println!(
                "{}",
                serde_json::to_string_pretty(&lifecycle_output(&repo, &id, &receipt)?)?
            );
        }
        CommandKind::Verify {
            repo,
            work_item,
            command,
            args,
            workers,
        } => {
            require_compatible(&repo, &runtime_context)?;
            let root = std::fs::canonicalize(&repo).context("canonicalize repository")?;
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
                    stage: "task".into(),
                    runner: "local".into(),
                    runtime_digest: runtime_context.runtime_digest.to_string(),
                    base_commit: None,
                    workers,
                    policy: if explicit || work_item.is_some() {
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
            WorkItemCommand::Inspect { repo, id } => {
                require_compatible(&repo, &runtime_context)?;
                let compatibility = cockpit_repository::work_item_compatibility(&repo, &id)
                    .context("inspect Work Item compatibility")?;
                let approach = cockpit_repository::implementation_approach(&repo, &id)
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
                let registry = cockpit_repository::capability_truth_registry(&repo)
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
                    let records = cockpit_repository::generate_knowledge_v2(&repo)
                        .context("project knowledge v2")?;
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
                    let output = json!({"schemaVersion": 2, "matchCount": records.len(), "results": records});
                    println!("{}", serde_json::to_string_pretty(&output)?);
                    return Ok(());
                }
                let index = generate_knowledge(&repo).context("project knowledge")?;
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

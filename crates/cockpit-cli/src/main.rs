use anyhow::{Context, Result};
use clap::{ArgAction, Parser, Subcommand};
use cockpit_agent::AgentExitCode;
use cockpit_git::GitRepository;
use cockpit_knowledge::{Query, query};
use cockpit_protocol::{AgentProvider, Contract, RepositoryConfig, validate_protocol_version};
use cockpit_repository::{
    RepositoryVerificationPolicy, RepositoryVerificationRequest, WorkItemStartOptions,
    archive_work_item, attach, checkpoint_work_item, close_work_item_with_decision,
    finish_work_item, generate_knowledge, run_repository_verification, scaffold_work_item,
    start_work_item_with_options, status,
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
    WorkItem {
        #[command(subcommand)]
        command: WorkItemCommand,
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
            let git = GitRepository::discover(&repo).context("discover repository")?;
            let snapshot = git.snapshot().context("create repository snapshot")?;
            let contract: Contract =
                serde_json::from_slice(&std::fs::read(&contract).context("read contract")?)
                    .context("parse contract")?;
            cockpit_protocol::validate_protocol_version(contract.protocol_version)
                .context("validate contract protocol")?;
            let decision =
                cockpit_repository::governance_decision_for_contract(&repo, &contract, &snapshot)
                    .context("evaluate governance decision")?;
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
            let receipt = finish_work_item(&repo, &id).context("finish work item")?;
            println!("{}", serde_json::to_string_pretty(&receipt)?);
        }
        CommandKind::Archive { repo, id } => {
            require_compatible(&repo, &runtime_context)?;
            let receipt = archive_work_item(&repo, &id).context("archive work item")?;
            println!("{}", serde_json::to_string_pretty(&receipt)?);
        }
        CommandKind::Close {
            repo,
            id,
            human_decision,
        } => {
            require_compatible(&repo, &runtime_context)?;
            let receipt = close_work_item_with_decision(&repo, &id, &human_decision)
                .context("close work item")?;
            println!("{}", serde_json::to_string_pretty(&receipt)?);
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
                cockpit_repository::record_verification_with_snapshot(
                    &root,
                    &work_item,
                    &output,
                    &runtime_context.runtime_version,
                    &runtime_context.runtime_digest,
                    &run.final_snapshot,
                )
                .context("record verification evidence")?;
            }
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
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
        },
        CommandKind::Knowledge { command } => match command {
            KnowledgeCommand::Query {
                repo,
                topic,
                component,
                state,
                work_item_id,
            } => {
                require_compatible(&repo, &runtime_context)?;
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

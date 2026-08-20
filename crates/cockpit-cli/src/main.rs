use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cockpit_core::{ActionKind, AuthorityState, EvidenceState, GovernanceInput, evaluate};
use cockpit_git::GitRepository;
use cockpit_knowledge::{Query, query};
use cockpit_mcp::serve;
use cockpit_protocol::{Contract, RepositoryConfig};
use cockpit_repository::{
    archive_work_item, attach, checkpoint_work_item, close_work_item, contract_freshness_findings,
    finish_work_item, generate_knowledge, observe, start_work_item, status,
};
use cockpit_verification::{
    VerificationCommand, VerificationGraph, VerificationNode, VerificationNodeKind, execute_bounded,
};
use serde_json::json;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "ai-cockpit", version = "0.1.0")]
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
    },
    Verify {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        work_item: Option<String>,
        #[arg(long)]
        command: Option<String>,
        #[arg(long, value_delimiter = ',')]
        args: Vec<String>,
        #[arg(long, default_value_t = 2)]
        workers: usize,
    },
    Knowledge {
        #[command(subcommand)]
        command: KnowledgeCommand,
    },
    Mcp {
        #[arg(long)]
        repo: Option<PathBuf>,
    },
    Doctor {
        #[arg(long)]
        repo: PathBuf,
    },
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

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        CommandKind::Inspect { repo } => {
            let git = GitRepository::discover(&repo).context("discover repository")?;
            let snapshot = git.snapshot().context("create repository snapshot")?;
            let output = json!({
                "runtimeVersion": "0.1.0",
                "runtimeDigest": cockpit_core::Digest::sha256_bytes(b"ai-cockpit-0.1.0").to_string(),
                "protocolVersion": 1,
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
            let git = GitRepository::discover(&repo).context("discover repository")?;
            let snapshot = git.snapshot().context("create repository snapshot")?;
            let contract: Contract =
                serde_json::from_slice(&std::fs::read(&contract).context("read contract")?)
                    .context("parse contract")?;
            cockpit_protocol::validate_protocol_version(contract.protocol_version)
                .context("validate contract protocol")?;
            let evidence = if contract.required_evidence_classes.is_empty() {
                EvidenceState::Complete
            } else {
                EvidenceState::Missing
            };
            let action = if contract.risk.to_ascii_lowercase().contains("destructive") {
                ActionKind::Destructive
            } else {
                ActionKind::Write
            };
            let authority = if contract.authority == "authorized" {
                AuthorityState::Authorized
            } else {
                AuthorityState::Missing
            };
            let explicit_blockers = contract_freshness_findings(&repo, &contract, &snapshot)
                .context("validate contract freshness")?;
            let decision = evaluate(GovernanceInput {
                scope: contract.scope,
                out_of_scope: contract.out_of_scope,
                changed_paths: snapshot.changed_paths,
                action,
                authority,
                evidence,
                untrusted_material: false,
                test_weakening: false,
                coverage_weakening: false,
                explicit_blockers,
                explicit_unknowns: vec![],
                outcome_state_override: None,
                authority_override: None,
            });
            println!("{}", serde_json::to_string_pretty(&decision)?);
        }
        CommandKind::Observe { repo } => {
            let git = GitRepository::discover(&repo).context("discover repository")?;
            let snapshot = git.snapshot().context("create repository snapshot")?;
            let observation = observe(&snapshot.root, &snapshot).context("observe repository")?;
            let (evolution, profile_update_proposal) =
                std::fs::read(snapshot.root.join(".ai/project.json"))
                    .ok()
                    .and_then(|bytes| {
                        serde_json::from_slice::<cockpit_protocol::ProjectProfile>(&bytes).ok()
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
            println!("{}", serde_json::to_string_pretty(&repository_status)?);
        }
        CommandKind::Start {
            repo,
            id,
            intent,
            goal,
            scope,
        } => {
            let receipt =
                start_work_item(&repo, &id, &intent, &goal, &scope).context("start work item")?;
            println!("{}", serde_json::to_string_pretty(&receipt)?);
        }
        CommandKind::Checkpoint { repo, id } => {
            let receipt = checkpoint_work_item(&repo, &id).context("checkpoint work item")?;
            println!("{}", serde_json::to_string_pretty(&receipt)?);
        }
        CommandKind::Finish { repo, id } => {
            let receipt = finish_work_item(&repo, &id).context("finish work item")?;
            println!("{}", serde_json::to_string_pretty(&receipt)?);
        }
        CommandKind::Archive { repo, id } => {
            let receipt = archive_work_item(&repo, &id).context("archive work item")?;
            println!("{}", serde_json::to_string_pretty(&receipt)?);
        }
        CommandKind::Close { repo, id } => {
            let receipt = close_work_item(&repo, &id).context("close work item")?;
            println!("{}", serde_json::to_string_pretty(&receipt)?);
        }
        CommandKind::Verify {
            repo,
            work_item,
            command,
            args,
            workers,
        } => {
            let root = std::fs::canonicalize(&repo).context("canonicalize repository")?;
            let (program, command_args) = if let Some(command) = command {
                (command, args)
            } else if root.join("Cargo.toml").is_file() {
                ("cargo".into(), vec!["test".into(), "--workspace".into()])
            } else if root.join("package.json").is_file() {
                ("npm".into(), vec!["test".into()])
            } else {
                anyhow::bail!("no verified project command detected; provide --command")
            };
            let snapshot = GitRepository::discover(&root)
                .context("discover repository for verification")?
                .snapshot()
                .context("snapshot repository for verification")?;
            let mut graph = VerificationGraph::default();
            graph
                .add(VerificationNode::new(
                    "project-command",
                    VerificationNodeKind::Protected,
                    vec![],
                ))
                .context("build verification graph")?;
            let plan = graph.plan().context("plan verification graph")?;
            let commands = plan
                .into_iter()
                .map(|id| {
                    VerificationCommand::new(&id, &program, command_args.clone())
                        .with_protected(true)
                        .with_current_dir(&root)
                })
                .collect();
            let mut receipt = execute_bounded(commands, workers).context("execute verification")?;
            receipt.git_calls = snapshot.git_calls;
            receipt.files_read += snapshot.files_read;
            receipt.files_hashed += snapshot.files_hashed;
            let output = json!({
                "nodesPlanned": receipt.nodes_planned,
                "nodesExecuted": receipt.nodes_executed,
                "nodesReused": receipt.nodes_reused,
                "processesSpawned": receipt.processes_spawned,
                "gitCalls": receipt.git_calls,
                "filesRead": receipt.files_read,
                "filesHashed": receipt.files_hashed,
                "elapsedMs": receipt.elapsed_ms,
                "passed": receipt.passed,
            });
            if let Some(work_item) = work_item {
                cockpit_repository::record_verification(
                    &root,
                    &work_item,
                    &output,
                    "0.1.0",
                    &cockpit_core::Digest::sha256_bytes(b"ai-cockpit-0.1.0"),
                )
                .context("record verification evidence")?;
            }
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        CommandKind::Knowledge { command } => match command {
            KnowledgeCommand::Query {
                repo,
                topic,
                component,
                state,
                work_item_id,
            } => {
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
        CommandKind::Mcp { repo } => {
            let stdin = std::io::stdin();
            let stdout = std::io::stdout();
            if let Some(repo) = repo {
                cockpit_mcp::serve_with_repo(
                    std::io::BufReader::new(stdin.lock()),
                    stdout.lock(),
                    &repo,
                )
                .context("serve MCP")?;
            } else {
                serve(std::io::BufReader::new(stdin.lock()), stdout.lock()).context("serve MCP")?;
            }
        }
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
                        "runtimeVersion": "0.1.0",
                    }))?
                );
                return Ok(());
            }
            let config_text =
                std::fs::read_to_string(&config_path).context("read protocol configuration")?;
            let config: RepositoryConfig =
                toml::from_str(&config_text).context("parse protocol configuration")?;
            let runtime_code_in_repository = contains_runtime_code(&root.join(".ai"));
            let state = if config.protocol_version == 1 && !runtime_code_in_repository {
                "ok"
            } else {
                "red"
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "state": state,
                    "protocolVersion": config.protocol_version,
                    "repositoryId": config.repository_id,
                    "runtimeCodeInRepository": runtime_code_in_repository,
                    "runtimeVersion": "0.1.0",
                }))?
            );
        }
    }
    Ok(())
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

use anyhow::{Context, Result};
use clap::Subcommand;
use cockpit_protocol::ci_admission::GithubEvidenceSelector;
use cockpit_protocol::{RuntimeContext, TaskAuthorizationCaller};
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub(crate) enum GithubEvidenceCommand {
    /// Collect fixed-origin GitHub main-run metadata and explain missing verification proofs; never accepts evidence.
    Plan {
        /// Repository containing the current Work Item and caller policy.
        #[arg(long)]
        repo: PathBuf,
        /// Active Work Item whose current authority and source bindings are checked.
        #[arg(long)]
        work_item: String,
        /// Positive GitHub workflow run identity in the fixed repository.
        #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
        run_id: u64,
        /// Exact positive run attempt; the latest attempt is never substituted.
        #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
        attempt: u32,
        /// Positive quality-job identity in that exact attempt.
        #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
        job_id: u64,
        /// Positive artifact metadata identity; no archive is downloaded or accepted.
        #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
        artifact_id: u64,
    },
}

pub(crate) fn run(
    command: GithubEvidenceCommand,
    runtime: &RuntimeContext,
    caller: Option<&TaskAuthorizationCaller>,
) -> Result<()> {
    let GithubEvidenceCommand::Plan {
        repo,
        work_item,
        run_id,
        attempt,
        job_id,
        artifact_id,
    } = command;
    super::require_compatible(&repo, runtime)?;
    let plan = cockpit_repository::plan_github_verification_admission(
        &repo,
        &work_item,
        &GithubEvidenceSelector {
            run_id,
            attempt,
            job_id,
            artifact_id,
        },
        runtime,
        caller,
    )
    .context("plan GitHub verification evidence without admission")?;
    println!("{}", serde_json::to_string_pretty(&plan)?);
    Ok(())
}

#[derive(Debug, Subcommand)]
pub(crate) enum PortableEvidenceCommand {
    /// Inspect a local portable bundle without execution or writes; source claims remain unverified.
    Plan {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        input: PathBuf,
        /// Optional consumer Work Item; not required merely to inspect evidence.
        #[arg(long)]
        work_item: Option<String>,
    },
    /// Request admission under current authority and independent source proof; a file cannot authenticate itself.
    Accept {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        work_item: String,
        #[arg(long)]
        input: PathBuf,
        #[arg(long, value_parser = parse_plan_digest)]
        expected_plan_digest: cockpit_core::Digest,
    },
}

fn parse_plan_digest(value: &str) -> Result<cockpit_core::Digest, String> {
    value
        .parse::<cockpit_core::Digest>()
        .map_err(|error| error.to_string())
}

pub(crate) fn run_portable(
    command: PortableEvidenceCommand,
    runtime: &RuntimeContext,
    caller: Option<&TaskAuthorizationCaller>,
) -> Result<()> {
    match command {
        PortableEvidenceCommand::Plan {
            repo,
            input,
            work_item,
        } => {
            super::require_compatible(&repo, runtime)?;
            let plan = cockpit_repository::plan_portable_evidence(
                &repo,
                work_item.as_deref(),
                &input,
                runtime,
                caller,
            )
            .context("inspect portable evidence without execution or writes")?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        PortableEvidenceCommand::Accept {
            repo,
            work_item,
            input,
            expected_plan_digest,
        } => {
            super::require_compatible(&repo, runtime)?;
            let acceptance = cockpit_repository::accept_portable_evidence(
                &repo,
                &work_item,
                &input,
                &expected_plan_digest,
                runtime,
                caller,
            )
            .context("request portable evidence admission under current authority")?;
            println!("{}", serde_json::to_string_pretty(&acceptance)?);
        }
    }
    Ok(())
}

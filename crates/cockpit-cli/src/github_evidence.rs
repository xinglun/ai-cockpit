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
        /// Repository checkout whose protocol compatibility is checked.
        #[arg(long)]
        repo: PathBuf,
        /// Local portable evidence JSON file; contents never authenticate their author.
        #[arg(long)]
        input: PathBuf,
        /// Optional consumer Work Item; not required merely to inspect evidence.
        #[arg(long)]
        work_item: Option<String>,
    },
    /// Request admission under current authority and independent source proof; a file cannot authenticate itself.
    Accept {
        /// Repository checkout bound to the consumer Work Item.
        #[arg(long)]
        repo: PathBuf,
        /// Consumer Work Item; current caller and native action admission are rechecked.
        #[arg(long)]
        work_item: String,
        /// Local portable evidence JSON file inspected by the shared service.
        #[arg(long)]
        input: PathBuf,
        /// Exact sha256 plan digest returned by evidence bundle plan; not a source credential.
        #[arg(long, value_parser = parse_plan_digest)]
        expected_plan_digest: String,
    },
}

fn parse_plan_digest(value: &str) -> Result<String, String> {
    value
        .parse::<cockpit_core::Digest>()
        .map_err(|error| error.to_string())?;
    Ok(value.to_owned())
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
            let expected_plan_digest = expected_plan_digest
                .parse::<cockpit_core::Digest>()
                .context("validate exact portable evidence plan digest")?;
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

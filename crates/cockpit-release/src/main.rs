use std::{fs, path::PathBuf, process::ExitCode};

use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use cockpit_release::{
    archive::{ArchiveTarget, PackageInput, inspect_archive, package_archive},
    formula::{FormulaSource, render_formula},
    handoff::{Destination, HandoffDocument, Issuer, ReleaseBinding},
    manifest::{ReleaseManifest, write_checksums},
};

#[derive(Debug, Parser)]
#[command(
    name = "cockpit-release",
    version,
    about = "ai-cockpit release boundary tooling"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Package {
        #[arg(long)]
        executable: PathBuf,
        #[arg(long)]
        license: PathBuf,
        #[arg(long)]
        readme: PathBuf,
        #[arg(long)]
        target: String,
        #[arg(long)]
        output: PathBuf,
    },
    Inspect {
        #[arg(long)]
        archive: PathBuf,
        #[arg(long)]
        target: String,
    },
    Validate {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        dist: PathBuf,
    },
    Manifest {
        #[arg(long)]
        dist: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        version: String,
        #[arg(long)]
        tag: String,
        #[arg(long)]
        commit: String,
        #[arg(long)]
        cargo_lock_sha256: String,
    },
    Formula {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        fixture_base_url: Option<String>,
    },
    Handoff {
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        workflow_ref: String,
        #[arg(long)]
        run_id: u64,
        #[arg(long)]
        tag: String,
        #[arg(long)]
        commit: String,
        #[arg(long)]
        provider_release_id: u64,
        #[arg(long)]
        manifest_sha256: String,
        #[arg(long)]
        formula_sha256: String,
        #[arg(long)]
        issued_at: String,
        #[arg(long)]
        expires_at: String,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("cockpit-release: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command {
        Command::Package {
            executable,
            license,
            readme,
            target,
            output,
        } => {
            let input = PackageInput {
                executable,
                license,
                readme,
                target: ArchiveTarget::from_rust_target(&target)?,
            };
            let record = package_archive(&input, &output)?;
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({
                    "filename": record.filename,
                    "bytes": record.bytes,
                    "sha256": record.sha256,
                }))?
            );
        }
        Command::Inspect { archive, target } => {
            let inspection = inspect_archive(&archive, ArchiveTarget::from_rust_target(&target)?)?;
            println!("{}", serde_json::to_string(&inspection.members)?);
        }
        Command::Validate { manifest, dist } => {
            let manifest = ReleaseManifest::parse_str(&fs::read_to_string(manifest)?)?;
            let validated = manifest.validate_staged(&dist)?;
            println!("validated {} files", validated.files.len());
        }
        Command::Manifest {
            dist,
            output,
            version,
            tag,
            commit,
            cargo_lock_sha256,
        } => {
            let manifest = ReleaseManifest::from_staged_dist(
                &version,
                &tag,
                &commit,
                &cargo_lock_sha256,
                &dist,
            )?;
            fs::write(&output, manifest.canonical_bytes()?)?;
            write_checksums(&manifest, &dist)?;
            manifest.validate_staged(&dist)?;
        }
        Command::Formula {
            manifest,
            output,
            fixture_base_url,
        } => {
            let manifest = ReleaseManifest::parse_str(&fs::read_to_string(manifest)?)?;
            let source = fixture_base_url.map_or(
                FormulaSource::Production {
                    release_origin: "https://github.com/xinglun/ai-cockpit/releases/download/"
                        .into(),
                },
                |base_url| FormulaSource::Fixture { base_url },
            );
            fs::write(output, render_formula(&manifest, source)?)?;
        }
        Command::Handoff {
            output,
            workflow_ref,
            run_id,
            tag,
            commit,
            provider_release_id,
            manifest_sha256,
            formula_sha256,
            issued_at,
            expires_at,
        } => {
            let issued_at: DateTime<Utc> = issued_at.parse::<DateTime<Utc>>()?;
            let expires_at: DateTime<Utc> = expires_at.parse::<DateTime<Utc>>()?;
            let handoff = HandoffDocument::new(
                Issuer {
                    repository: "xinglun/ai-cockpit".into(),
                    workflow_ref,
                    run_id,
                },
                Destination {
                    repository: "xinglun/homebrew-tap".into(),
                    base_ref: "main".into(),
                    path: "Formula/ai-cockpit.rb".into(),
                },
                ReleaseBinding {
                    tag,
                    commit,
                    provider_release_id,
                    manifest_sha256,
                    formula_sha256,
                },
                "open_pull_request".into(),
                issued_at,
                expires_at,
            )?;
            fs::write(output, handoff.canonical_bytes()?)?;
        }
    }
    Ok(())
}

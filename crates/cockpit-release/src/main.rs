use std::{fs, path::PathBuf, process::ExitCode};

use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use cockpit_release::{
    archive::{ArchiveTarget, PackageInput, inspect_archive, package_archive},
    formula::{FormulaSource, render_formula},
    handoff::{Destination, HandoffDocument, Issuer, ReleaseBinding},
    manifest::{ReleaseManifest, write_checksums},
    provider::verify_existing_release,
    resume::{PhaseReceiptStore, plan_for_phase},
    sbom::{bind_sbom_file, validate_sbom_binding},
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
    BindSbom {
        #[arg(long)]
        sbom: PathBuf,
        #[arg(long)]
        archive: PathBuf,
        #[arg(long)]
        target: String,
        #[arg(long)]
        version: String,
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
    Checksums {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        dist: PathBuf,
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
    ValidateHandoff {
        #[arg(long)]
        handoff: PathBuf,
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
    },
    AcceptancePlan {
        #[arg(long, default_value = "full")]
        scope: String,
        #[arg(long)]
        identity: PathBuf,
        #[arg(long)]
        receipts: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    AcceptanceRecord {
        #[arg(long, default_value = "full")]
        scope: String,
        #[arg(long)]
        identity: PathBuf,
        #[arg(long)]
        receipts: PathBuf,
        #[arg(long)]
        phase: String,
        #[arg(long)]
        evidence: PathBuf,
        #[arg(long, default_value_t = 1)]
        attempt: u32,
    },
    AcceptanceRecordFailure {
        #[arg(long, default_value = "full")]
        scope: String,
        #[arg(long)]
        identity: PathBuf,
        #[arg(long)]
        receipts: PathBuf,
        #[arg(long)]
        phase: String,
        #[arg(long)]
        failure_kind: String,
        #[arg(long)]
        failure_code: String,
        #[arg(long)]
        diagnostic: String,
        #[arg(long, default_value_t = 1)]
        attempt: u32,
    },
    VerifyProviderRelease {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        dist: PathBuf,
        #[arg(long)]
        release_json: PathBuf,
        #[arg(long)]
        tag_commit: String,
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
        Command::BindSbom {
            sbom,
            archive,
            target,
            version,
        } => {
            let target = ArchiveTarget::from_rust_target(&target)?;
            bind_sbom_file(&sbom, &archive, target, &version)?;
            validate_sbom_binding(&sbom, &archive, target, &version)?;
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
        }
        Command::Checksums { manifest, dist } => {
            let manifest = ReleaseManifest::parse_str(&fs::read_to_string(manifest)?)?;
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
        Command::ValidateHandoff {
            handoff,
            tag,
            commit,
            provider_release_id,
            manifest_sha256,
            formula_sha256,
        } => {
            let document = HandoffDocument::parse_str(&fs::read_to_string(handoff)?)?;
            document.validate(Utc::now())?;
            if document.release.tag != tag
                || document.release.commit != commit
                || document.release.provider_release_id != provider_release_id
                || document.release.manifest_sha256 != manifest_sha256
                || document.release.formula_sha256 != formula_sha256
            {
                return Err(Box::new(cockpit_release::ReleaseError::Invalid(
                    "handoff does not bind the requested public Release".into(),
                )));
            }
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({
                    "tag": document.release.tag,
                    "commit": document.release.commit,
                    "providerReleaseId": document.release.provider_release_id,
                    "requestId": document.request_id,
                }))?
            );
        }
        Command::AcceptancePlan {
            scope,
            identity,
            receipts,
            output,
        } => {
            let identity: cockpit_release::acceptance::ReleaseIdentity =
                serde_json::from_slice(&fs::read(identity)?)?;
            let store = PhaseReceiptStore::load(&receipts, &identity)?;
            let scope = parse_acceptance_scope(&scope)?;
            let plan = store.plan_for_scope(scope)?;
            let document = serde_json::json!({
                "schemaVersion": 1,
                "scope": scope,
                "identityDigest": identity.digest(),
                "actions": plan.actions,
            });
            fs::write(output, serde_json::to_vec_pretty(&document)?)?;
        }
        Command::AcceptanceRecord {
            scope,
            identity,
            receipts,
            phase,
            evidence,
            attempt,
        } => {
            let identity: cockpit_release::acceptance::ReleaseIdentity =
                serde_json::from_slice(&fs::read(identity)?)?;
            let phase = parse_release_phase(&phase)?;
            let scope = parse_acceptance_scope(&scope)?;
            let store = PhaseReceiptStore::update_atomic(&receipts, &identity, |store| {
                store.record_success(phase, attempt, phase.as_str(), &evidence)
            })?;
            if let Some(action) = plan_for_phase(&store.plan_for_scope(scope)?, phase) {
                println!("{}", serde_json::to_string(action)?);
            }
        }
        Command::AcceptanceRecordFailure {
            scope,
            identity,
            receipts,
            phase,
            failure_kind,
            failure_code,
            diagnostic,
            attempt,
        } => {
            let identity: cockpit_release::acceptance::ReleaseIdentity =
                serde_json::from_slice(&fs::read(identity)?)?;
            let phase = parse_release_phase(&phase)?;
            let scope = parse_acceptance_scope(&scope)?;
            let kind = parse_failure_kind(&failure_kind)?;
            let failure =
                cockpit_release::recovery::PhaseFailure::new(kind, failure_code, diagnostic);
            let store = PhaseReceiptStore::update_atomic(&receipts, &identity, |store| {
                store.record_failure(phase, attempt, failure)
            })?;
            if let Some(action) = plan_for_phase(&store.plan_for_scope(scope)?, phase) {
                println!("{}", serde_json::to_string(action)?);
            }
        }
        Command::VerifyProviderRelease {
            manifest,
            dist,
            release_json,
            tag_commit,
        } => {
            let receipt = verify_existing_release(&manifest, &dist, &release_json, &tag_commit)?;
            println!("{}", serde_json::to_string(&receipt)?);
        }
    }
    Ok(())
}

fn parse_failure_kind(value: &str) -> Result<cockpit_release::recovery::FailureKind, String> {
    use cockpit_release::recovery::FailureKind;
    match value {
        "interruption" => Ok(FailureKind::Interruption),
        "timeout" => Ok(FailureKind::Timeout),
        "network" => Ok(FailureKind::Network),
        "runner" => Ok(FailureKind::Runner),
        "cleanup" => Ok(FailureKind::Cleanup),
        "input_changed" => Ok(FailureKind::InputChanged),
        "validation" => Ok(FailureKind::Validation),
        "identity_mismatch" => Ok(FailureKind::IdentityMismatch),
        "scope_changed" => Ok(FailureKind::ScopeChanged),
        "authority_changed" => Ok(FailureKind::AuthorityChanged),
        "base_changed" => Ok(FailureKind::BaseChanged),
        "already_published" => Ok(FailureKind::AlreadyPublished),
        "unknown" => Ok(FailureKind::Unknown),
        _ => Err(format!("unknown failure kind: {value}")),
    }
}

fn parse_release_phase(value: &str) -> Result<cockpit_release::acceptance::ReleasePhase, String> {
    use cockpit_release::acceptance::ReleasePhase;
    match value {
        "prepare" => Ok(ReleasePhase::Prepare),
        "source_verification_build" => Ok(ReleasePhase::SourceVerificationBuild),
        "candidate_acceptance" => Ok(ReleasePhase::CandidateAcceptance),
        "publish" => Ok(ReleasePhase::Publish),
        "public_acceptance" => Ok(ReleasePhase::PublicAcceptance),
        "close" => Ok(ReleasePhase::Close),
        _ => Err(format!("unknown release phase: {value}")),
    }
}

fn parse_acceptance_scope(
    value: &str,
) -> Result<cockpit_release::acceptance::AcceptanceScope, String> {
    use cockpit_release::acceptance::AcceptanceScope;
    match value {
        "full" => Ok(AcceptanceScope::Full),
        "candidate" => Ok(AcceptanceScope::Candidate),
        "public" => Ok(AcceptanceScope::Public),
        _ => Err(format!("unknown acceptance scope: {value}")),
    }
}

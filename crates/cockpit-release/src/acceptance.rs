//! Identity-bound release acceptance data.
//!
//! This module deliberately contains no process, filesystem, or network code.
//! It is the durable data boundary that an executor can use to decide whether
//! a phase receipt belongs to the current release attempt.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::recovery::PhaseFailure;

/// The ordered stages of the release acceptance graph.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleasePhase {
    Prepare,
    SourceVerificationBuild,
    CandidateAcceptance,
    Publish,
    PublicAcceptance,
    Close,
}

/// Independent acceptance paths within the candidate/public phases. Each
/// path owns a separate isolation root and receipt, so a retry can reuse the
/// other paths without rebuilding or republishing.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AcceptancePath {
    CandidateFreshInstall,
    CandidateNMinusOneUpgrade,
    PublicFreshInstall,
    PublicNMinusOneUpgrade,
    PublicVersionConsistency,
}

/// The portion of the release graph owned by one adopter acceptance job.
/// Candidate jobs run before publication; public jobs start from an already
/// published identity. Keeping the scope explicit prevents a partial receipt
/// store from making an unrelated prerequisite appear to have failed.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AcceptanceScope {
    Full,
    Candidate,
    Public,
}

impl AcceptanceScope {
    pub const fn phases(self) -> &'static [ReleasePhase] {
        match self {
            Self::Full => ReleasePhase::all(),
            Self::Candidate => &[
                ReleasePhase::Prepare,
                ReleasePhase::SourceVerificationBuild,
                ReleasePhase::CandidateAcceptance,
                ReleasePhase::Close,
            ],
            Self::Public => &[
                ReleasePhase::Prepare,
                ReleasePhase::SourceVerificationBuild,
                ReleasePhase::Publish,
                ReleasePhase::PublicAcceptance,
                ReleasePhase::Close,
            ],
        }
    }
}

impl AcceptancePath {
    pub const fn all() -> &'static [Self; 5] {
        &[
            Self::CandidateFreshInstall,
            Self::CandidateNMinusOneUpgrade,
            Self::PublicFreshInstall,
            Self::PublicNMinusOneUpgrade,
            Self::PublicVersionConsistency,
        ]
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CandidateFreshInstall => "candidate_fresh_install",
            Self::CandidateNMinusOneUpgrade => "candidate_n_minus_one_upgrade",
            Self::PublicFreshInstall => "public_fresh_install",
            Self::PublicNMinusOneUpgrade => "public_n_minus_one_upgrade",
            Self::PublicVersionConsistency => "public_version_consistency",
        }
    }

    pub const fn phase(self) -> ReleasePhase {
        match self {
            Self::CandidateFreshInstall | Self::CandidateNMinusOneUpgrade => {
                ReleasePhase::CandidateAcceptance
            }
            Self::PublicFreshInstall
            | Self::PublicNMinusOneUpgrade
            | Self::PublicVersionConsistency => ReleasePhase::PublicAcceptance,
        }
    }
}

impl ReleasePhase {
    pub const fn all() -> &'static [Self; 6] {
        &[
            Self::Prepare,
            Self::SourceVerificationBuild,
            Self::CandidateAcceptance,
            Self::Publish,
            Self::PublicAcceptance,
            Self::Close,
        ]
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Prepare => "prepare",
            Self::SourceVerificationBuild => "source_verification_build",
            Self::CandidateAcceptance => "candidate_acceptance",
            Self::Publish => "publish",
            Self::PublicAcceptance => "public_acceptance",
            Self::Close => "close",
        }
    }

    pub const fn prerequisite(self) -> Option<Self> {
        match self {
            Self::Prepare => None,
            Self::SourceVerificationBuild => Some(Self::Prepare),
            Self::CandidateAcceptance => Some(Self::SourceVerificationBuild),
            Self::Publish => Some(Self::CandidateAcceptance),
            Self::PublicAcceptance => Some(Self::Publish),
            Self::Close => Some(Self::PublicAcceptance),
        }
    }

    pub const fn parallel_acceptance_paths(self) -> &'static [AcceptancePath] {
        match self {
            Self::CandidateAcceptance => &[
                AcceptancePath::CandidateFreshInstall,
                AcceptancePath::CandidateNMinusOneUpgrade,
            ],
            Self::PublicAcceptance => &[
                AcceptancePath::PublicFreshInstall,
                AcceptancePath::PublicNMinusOneUpgrade,
                AcceptancePath::PublicVersionConsistency,
            ],
            _ => &[],
        }
    }
}

/// State persisted for one phase attempt.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Interrupted,
    TimedOut,
    Blocked,
}

impl PhaseStatus {
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Interrupted | Self::TimedOut | Self::Blocked
        )
    }
}

/// Cleanup state is part of reuse validity. A successful phase with unfinished
/// cleanup cannot be treated as a completed reusable receipt.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupState {
    NotRequired,
    Pending,
    Succeeded,
    Failed,
}

impl CleanupState {
    pub const fn allows_reuse(self) -> bool {
        matches!(self, Self::NotRequired | Self::Succeeded)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceIdentity {
    pub repository: String,
    pub commit: String,
    pub cargo_lock_digest: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtifactIdentity {
    pub version: String,
    pub tag: String,
    pub manifest_digest: String,
    pub assets: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeIdentity {
    pub version: String,
    pub digest: String,
}

/// The exact isolated roots used by an acceptance run. These are identity
/// inputs, not proof that the roots have already been cleaned.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IsolationRootPolicy {
    pub home: String,
    pub xdg_config_home: String,
    pub tmp: String,
    pub cargo_home: String,
}

/// All inputs that can make a phase receipt unsafe to reuse.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReleaseIdentity {
    pub source: SourceIdentity,
    pub candidate: ArtifactIdentity,
    pub previous: Option<ArtifactIdentity>,
    pub runtime: RuntimeIdentity,
    pub target: String,
    pub isolation: IsolationRootPolicy,
}

impl ReleaseIdentity {
    /// Return the content-bound identity digest used by phase receipts.
    ///
    /// Physical isolation roots are intentionally excluded from this key.
    /// They are retained in the serialized identity for audit, but a resumed
    /// receipt may be restored under a different runner/output directory.
    /// The evidence digest and current receipt directory still have to
    /// validate before a phase can be reused.
    pub fn digest(&self) -> String {
        let bytes = serde_json::to_vec(&ReleaseIdentityDigest::from(self))
            .expect("ReleaseIdentity digest payload is serializable");
        let digest = Sha256::digest(bytes);
        format!("sha256:{}", hex::encode(digest))
    }

    /// Return the pre-relocation digest used by older receipt stores.
    pub(crate) fn legacy_digest(&self) -> String {
        let bytes = serde_json::to_vec(self).expect("ReleaseIdentity is serializable");
        let digest = Sha256::digest(bytes);
        format!("sha256:{}", hex::encode(digest))
    }

    /// Compare release inputs while allowing only the physical runner roots
    /// to move.  Source, artifacts, Runtime, target, and the isolation-root
    /// roles remain part of the binding.
    pub(crate) fn binding_matches(&self, other: &Self) -> bool {
        self.source == other.source
            && self.candidate == other.candidate
            && self.previous == other.previous
            && self.runtime == other.runtime
            && self.target == other.target
            && logical_isolation(&self.isolation) == logical_isolation(&other.isolation)
    }

    pub(crate) fn digest_matches(&self, digest: &str) -> bool {
        digest == self.digest() || digest == self.legacy_digest()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseIdentityDigest<'a> {
    source: &'a SourceIdentity,
    candidate: &'a ArtifactIdentity,
    previous: &'a Option<ArtifactIdentity>,
    runtime: &'a RuntimeIdentity,
    target: &'a str,
    isolation: LogicalIsolationRootPolicy,
}

impl<'a> From<&'a ReleaseIdentity> for ReleaseIdentityDigest<'a> {
    fn from(identity: &'a ReleaseIdentity) -> Self {
        Self {
            source: &identity.source,
            candidate: &identity.candidate,
            previous: &identity.previous,
            runtime: &identity.runtime,
            target: &identity.target,
            isolation: logical_isolation(&identity.isolation),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct LogicalIsolationRootPolicy {
    home: &'static str,
    xdg_config_home: &'static str,
    tmp: &'static str,
    cargo_home: &'static str,
}

fn logical_isolation(_isolation: &IsolationRootPolicy) -> LogicalIsolationRootPolicy {
    LogicalIsolationRootPolicy {
        home: "home",
        xdg_config_home: "xdg_config_home",
        tmp: "tmp",
        cargo_home: "cargo_home",
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PhaseBoundaries {
    pub started_at: String,
    pub finished_at: Option<String>,
    pub elapsed_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceReference {
    pub id: String,
    pub path: String,
    pub digest: String,
}

/// A single append-only phase result. The `identity_digest` must equal the
/// digest of `identity`; recovery rejects a receipt that violates that
/// binding instead of guessing which field is authoritative.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PhaseResult {
    pub schema_version: u32,
    pub phase: ReleasePhase,
    pub status: PhaseStatus,
    pub identity: ReleaseIdentity,
    pub identity_digest: String,
    pub attempt: u32,
    pub boundaries: PhaseBoundaries,
    pub evidence: Option<EvidenceReference>,
    pub failure: Option<PhaseFailure>,
    pub cleanup: CleanupState,
}

impl PhaseResult {
    pub fn succeeded(
        phase: ReleasePhase,
        identity: ReleaseIdentity,
        attempt: u32,
        boundaries: PhaseBoundaries,
        evidence: EvidenceReference,
    ) -> Self {
        let identity_digest = identity.digest();
        Self {
            schema_version: 1,
            phase,
            status: PhaseStatus::Succeeded,
            identity,
            identity_digest,
            attempt,
            boundaries,
            evidence: Some(evidence),
            failure: None,
            cleanup: CleanupState::NotRequired,
        }
    }

    pub fn failed(
        phase: ReleasePhase,
        identity: ReleaseIdentity,
        attempt: u32,
        boundaries: PhaseBoundaries,
        failure: PhaseFailure,
    ) -> Self {
        Self::failed_with_status(
            phase,
            identity,
            attempt,
            boundaries,
            failure.status(),
            failure,
        )
    }

    pub fn failed_with_status(
        phase: ReleasePhase,
        identity: ReleaseIdentity,
        attempt: u32,
        boundaries: PhaseBoundaries,
        status: PhaseStatus,
        failure: PhaseFailure,
    ) -> Self {
        debug_assert!(matches!(
            status,
            PhaseStatus::Failed | PhaseStatus::Interrupted | PhaseStatus::TimedOut
        ));
        let identity_digest = identity.digest();
        Self {
            schema_version: 1,
            phase,
            status,
            identity,
            identity_digest,
            attempt,
            boundaries,
            evidence: None,
            failure: Some(failure),
            cleanup: CleanupState::NotRequired,
        }
    }

    pub fn binding_matches(&self, expected: &ReleaseIdentity) -> bool {
        self.identity.binding_matches(expected)
            && self.identity.digest_matches(&self.identity_digest)
    }
}

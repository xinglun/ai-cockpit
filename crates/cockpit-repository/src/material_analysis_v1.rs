//! Version-owned V1 analysis closure audited from 84fb04ee43e33842865a1544a5588466174c0639.
//! Wire identities, scanning rules, policy models and Git operations are fixed.
//! Current admission and no-follow I/O remain outside this compatibility layer.
use super::material_recipe::{
    self as codec, AssuranceV1 as MaterialInspectionReviewAssurance,
    DecisionV1 as MaterialInspectionReviewDecision, EntryV1 as MaterialReviewEntry,
    EvidenceRefV1 as MaterialInspectionReviewEvidenceRef,
    ReceiptV1 as MaterialInspectionReviewDecisionReceipt, RequestV1 as MaterialReviewRequest,
    ScannerAssessmentV1 as MaterialScannerAssessment, UnknownCauseV1,
};
use super::material_review::{MaterialReviewDecisionValidationError, MaterialReviewRequestError};
use super::{ObserverError, repository_id};
use crate::rust_material::{
    MaterialUnknownCause, RustMaterialAssessment, RustMaterialDiagnosis,
    contains_strong_instruction_injection, diagnose_rust_material_v1,
};
use cockpit_core::Digest;
use cockpit_git::{
    BoundedGitOutput, ChangeContentState, ChangeEvidence, ChangeKind, GitError, GitRepository,
    RepositorySnapshot,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use thiserror::Error;
const MAX_CHANGE_TEXT_BYTES: usize = 262_144;
const MATERIAL_INSPECTION_REVIEW_CAPABILITY: &str = "material-inspection-review";
const MATERIAL_INSPECTION_REVIEW_DECISION_SCHEMA_VERSION: u32 = 1;
const SCANNER_SEMANTIC_VERSION: &str = "rust-material-v1";
const ANALYSIS_TARGET_SEMANTIC_PROFILE: &str = "cross-platform-rust-source-v1";
const MAX_MATERIAL_BLOB_BYTES: u64 = 16 * 1024 * 1024;
const MAX_MATERIAL_CHANGED_FILES: usize = 256;
const MAX_MATERIAL_TOTAL_BLOB_BYTES: u64 = 32 * 1024 * 1024;
const MAX_MATERIAL_TOTAL_TEXT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_PATCH_BYTES: usize = 64 * 1024 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum VerificationTierV1 {
    #[serde(rename = "T0")]
    T0,
    #[serde(rename = "T1")]
    T1,
    #[serde(rename = "T2")]
    T2,
    #[serde(rename = "T3")]
    T3,
}

impl VerificationTierV1 {
    pub const fn rank(self) -> u8 {
        match self {
            Self::T0 => 0,
            Self::T1 => 1,
            Self::T2 => 2,
            Self::T3 => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum PolicyAssuranceV1 {
    SelfDeclared,
    RepositoryVerified,
    ProviderVerified,
    EnterpriseVerified,
}

// Assurance ordering is part of the fixed V1 policy merge.
impl PolicyAssuranceV1 {
    pub const fn rank(self) -> u8 {
        match self {
            Self::SelfDeclared => 0,
            Self::RepositoryVerified => 1,
            Self::ProviderVerified => 2,
            Self::EnterpriseVerified => 3,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct VerificationRequirementV1 {
    pub schema_version: u32,
    pub required_tier: VerificationTierV1,
    pub required_assurance: PolicyAssuranceV1,
    pub policy_refs: Vec<String>,
    pub stage_refs: Vec<String>,
    pub gate_refs: Vec<String>,
    pub reason: String,
    /// Maximum timeout a caller may request for a verification command. This
    /// is an authorization ceiling, not a default and never weakens checks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_timeout_seconds: Option<u64>,
}

impl VerificationRequirementV1 {
    pub const fn is_at_least_as_strict_as(&self, parent: &Self) -> bool {
        self.required_tier.rank() >= parent.required_tier.rank()
            && self.required_assurance.rank() >= parent.required_assurance.rank()
            && match (parent.max_timeout_seconds, self.max_timeout_seconds) {
                (Some(parent), Some(child)) => child <= parent,
                _ => true,
            }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum PolicyLayerV1 {
    Organization,
    Project,
    WorkItem,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ApprovalModeV1 {
    NoHumanApprovalForLowRisk,
    SingleAuthorizedHuman,
    MultiPartyApproval,
    ExternalProviderApproval,
}

impl ApprovalModeV1 {
    fn strength(&self) -> u8 {
        match self {
            Self::NoHumanApprovalForLowRisk => 0,
            Self::SingleAuthorizedHuman => 1,
            Self::MultiPartyApproval | Self::ExternalProviderApproval => 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct PolicyRuleV1 {
    pub operation: String,
    pub approval_mode: ApprovalModeV1,
    pub required_evidence: Vec<String>,
    #[serde(default)]
    pub verification_requirement: Option<VerificationRequirementV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct PolicyV1 {
    pub policy_id: String,
    pub layer: PolicyLayerV1,
    pub rules: Vec<PolicyRuleV1>,
}

/// Repository-owned policy input. A missing file means that the repository has
/// not opted into an organization/project policy yet; an existing file is
/// parsed strictly and cannot contain adapter instructions or unknown policy
/// fields.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct PolicyDocumentV1 {
    pub schema_version: u32,
    #[serde(default)]
    pub organization: Option<PolicyV1>,
    #[serde(default)]
    pub project: Option<PolicyV1>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub(super) enum PolicyErrorV1 {
    #[error("lower policy weakens {operation}: {field}")]
    Weakening { operation: String, field: String },
    #[error("policy layer is out of order: expected {expected}, got {actual}")]
    InvalidLayer { expected: String, actual: String },
}

fn policy_layer_name(layer: &PolicyLayerV1) -> &'static str {
    match layer {
        PolicyLayerV1::Organization => "organization",
        PolicyLayerV1::Project => "project",
        PolicyLayerV1::WorkItem => "work_item",
    }
}

fn policy_layers_in_order(layers: &[&PolicyV1]) -> bool {
    layers.windows(2).all(|pair| {
        let rank = |layer: &PolicyLayerV1| match layer {
            PolicyLayerV1::Organization => 0,
            PolicyLayerV1::Project => 1,
            PolicyLayerV1::WorkItem => 2,
        };
        rank(&pair[0].layer) <= rank(&pair[1].layer)
    })
}

/// Validate an overlay without forcing every lower layer to repeat inherited
/// rules. An explicitly weaker rule is rejected; omission leaves the parent
/// rule in force.
pub fn validate_policy_overlay(parent: &PolicyV1, child: &PolicyV1) -> Result<(), PolicyErrorV1> {
    for parent_rule in &parent.rules {
        let Some(child_rule) = child
            .rules
            .iter()
            .find(|rule| rule.operation == parent_rule.operation)
        else {
            continue;
        };
        if child_rule.approval_mode.strength() < parent_rule.approval_mode.strength() {
            return Err(PolicyErrorV1::Weakening {
                operation: parent_rule.operation.clone(),
                field: "approvalMode".into(),
            });
        }
        let parent_evidence = parent_rule
            .required_evidence
            .iter()
            .collect::<std::collections::BTreeSet<_>>();
        let child_evidence = child_rule
            .required_evidence
            .iter()
            .collect::<std::collections::BTreeSet<_>>();
        if !parent_evidence.is_subset(&child_evidence) {
            return Err(PolicyErrorV1::Weakening {
                operation: parent_rule.operation.clone(),
                field: "requiredEvidence".into(),
            });
        }
        if let (Some(parent_requirement), Some(child_requirement)) = (
            &parent_rule.verification_requirement,
            &child_rule.verification_requirement,
        ) && !child_requirement.is_at_least_as_strict_as(parent_requirement)
        {
            return Err(PolicyErrorV1::Weakening {
                operation: parent_rule.operation.clone(),
                field: "verificationRequirement".into(),
            });
        }
    }
    Ok(())
}

/// Merge organization, project, and optional Work Item policies into one
/// effective policy. Child rules may add requirements or strengthen approval,
/// but they may never weaken a parent rule. Rule order is deterministic.
pub(super) fn merge_policy_layers(layers: &[&PolicyV1]) -> Result<PolicyV1, PolicyErrorV1> {
    if !policy_layers_in_order(layers)
        && let Some(policy) = layers.first()
    {
        return Err(PolicyErrorV1::InvalidLayer {
            expected: "ascending organization/project/work_item".into(),
            actual: policy_layer_name(&policy.layer).into(),
        });
    }
    let mut effective = PolicyV1 {
        policy_id: "effective:none".into(),
        layer: PolicyLayerV1::Organization,
        rules: Vec::new(),
    };
    let mut ids = Vec::new();
    for (index, policy) in layers.iter().enumerate() {
        let expected = match index {
            0 => None,
            _ => Some(match policy.layer {
                PolicyLayerV1::Organization => "project or work_item",
                PolicyLayerV1::Project => "work_item",
                PolicyLayerV1::WorkItem => "no lower layer",
            }),
        };
        if let Some(expected) = expected {
            let previous = layers[index - 1].layer.clone();
            let valid = match previous {
                PolicyLayerV1::Organization => {
                    matches!(
                        policy.layer,
                        PolicyLayerV1::Organization | PolicyLayerV1::Project
                    )
                }
                PolicyLayerV1::Project => {
                    matches!(
                        policy.layer,
                        PolicyLayerV1::Project | PolicyLayerV1::WorkItem
                    )
                }
                PolicyLayerV1::WorkItem => matches!(policy.layer, PolicyLayerV1::WorkItem),
            };
            if !valid {
                return Err(PolicyErrorV1::InvalidLayer {
                    expected: expected.into(),
                    actual: policy_layer_name(&policy.layer).into(),
                });
            }
        }
        if index > 0 {
            validate_policy_overlay(&effective, policy)?;
        }
        for rule in &policy.rules {
            if let Some(existing) = effective
                .rules
                .iter_mut()
                .find(|existing| existing.operation == rule.operation)
            {
                let mut evidence = existing.required_evidence.clone();
                for item in &rule.required_evidence {
                    if !evidence.contains(item) {
                        evidence.push(item.clone());
                    }
                }
                existing.approval_mode = rule.approval_mode.clone();
                existing.required_evidence = evidence;
                if let Some(child_requirement) = &rule.verification_requirement {
                    existing.verification_requirement = Some(
                        existing
                            .verification_requirement
                            .as_ref()
                            .map(|parent_requirement| {
                                merge_verification_requirements(
                                    parent_requirement,
                                    child_requirement,
                                )
                            })
                            .unwrap_or_else(|| child_requirement.clone()),
                    );
                }
            } else {
                effective.rules.push(rule.clone());
            }
        }
        effective.layer = policy.layer.clone();
        ids.push(policy.policy_id.clone());
    }
    if !ids.is_empty() {
        effective.policy_id = format!("effective:{}", ids.join(":"));
    }
    Ok(effective)
}

fn merge_verification_requirements(
    parent: &VerificationRequirementV1,
    child: &VerificationRequirementV1,
) -> VerificationRequirementV1 {
    let mut merged = parent.clone();
    if child.required_tier.rank() > merged.required_tier.rank() {
        merged.required_tier = child.required_tier;
    }
    if child.required_assurance.rank() > merged.required_assurance.rank() {
        merged.required_assurance = child.required_assurance;
    }
    merged.max_timeout_seconds = match (parent.max_timeout_seconds, child.max_timeout_seconds) {
        (Some(parent), Some(child)) => Some(parent.min(child)),
        (Some(parent), None) => Some(parent),
        (None, Some(child)) => Some(child),
        (None, None) => None,
    };
    for reference in &child.policy_refs {
        if !merged.policy_refs.contains(reference) {
            merged.policy_refs.push(reference.clone());
        }
    }
    for reference in &child.stage_refs {
        if !merged.stage_refs.contains(reference) {
            merged.stage_refs.push(reference.clone());
        }
    }
    for reference in &child.gate_refs {
        if !merged.gate_refs.contains(reference) {
            merged.gate_refs.push(reference.clone());
        }
    }
    merged.reason = format!("{}; {}", parent.reason, child.reason);
    merged
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ProfileV1 {
    pub schema_version: u32,
    pub permitted_unknown: String,
    pub permitted_cause: String,
    pub assurance: String,
    pub reviewer_actor: String,
    pub authority_source: String,
    pub accept_residual_risk: bool,
}

impl ProfileV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("materialInspectionReview schemaVersion must be 1".into());
        }
        if self.permitted_unknown != "repository_material_inspection_unavailable" {
            return Err("materialInspectionReview permittedUnknown is unsupported".into());
        }
        if self.permitted_cause != "readable_committed_rust_syntax_unknown" {
            return Err("materialInspectionReview permittedCause is unsupported".into());
        }
        if self.assurance != "self_declared" {
            return Err("materialInspectionReview assurance must be self_declared".into());
        }
        let actor = self.reviewer_actor.strip_prefix("agent:");
        if !actor.is_some_and(|actor| {
            actor.len() <= 128
                && actor
                    .as_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_alphanumeric)
                && actor.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'/')
                })
        }) {
            return Err(
                "materialInspectionReview reviewerActor must be a concrete agent actor".into(),
            );
        }
        if self.authority_source.trim().is_empty()
            || self.authority_source.trim() != self.authority_source
        {
            return Err("materialInspectionReview authoritySource must be concrete".into());
        }
        if !self.accept_residual_risk {
            return Err("materialInspectionReview acceptResidualRisk must be true".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContractV1 {
    repository_id: String,
    work_item_id: String,
    base_revision: String,
    #[serde(default)]
    governance_policy: Option<PolicyV1>,
    #[serde(default)]
    governance_profile: Option<serde_json::Value>,
    #[serde(default)]
    required_runtime_capabilities: Vec<String>,
}
impl ContractV1 {
    fn material_inspection_review_profile(&self) -> Result<Option<ProfileV1>, String> {
        let Some(value) = self
            .governance_profile
            .as_ref()
            .and_then(serde_json::Value::as_object)
            .and_then(|value| value.get("materialInspectionReview"))
        else {
            return Ok(None);
        };
        let profile: ProfileV1 =
            serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
        profile.validate()?;
        Ok(Some(profile))
    }
}
const CONTRACT_FIELDS_V1: &[&str] = &[
    "protocolVersion",
    "contractVersion",
    "repositoryId",
    "workItemId",
    "mode",
    "title",
    "state",
    "createdAt",
    "intent",
    "goal",
    "scope",
    "outOfScope",
    "risk",
    "authority",
    "acceptanceCriteria",
    "requiredEvidenceClasses",
    "sources",
    "verification",
    "baseRevision",
    "projectProfileDigest",
    "repositorySnapshotDigest",
    "resourceContext",
    "baseCommit",
    "baselineDirtyPaths",
    "archiveSequence",
    "resumeHistory",
    "predecessorWorkItemId",
    "predecessorContractDigest",
    "recoveryDecisionPath",
    "synchronizationCheckpoint",
    "synchronizationHistory",
    "guidelines",
    "acceptance",
    "preReviewWarnings",
    "authorityEvidence",
    "operation",
    "governancePolicy",
    "problemStatement",
    "riskAssessment",
    "agentCapability",
    "executionDecision",
    "destructiveChangePolicy",
    "rollbackNote",
    "rollbackPlan",
    "unknowns",
    "notCodable",
    "scenarioCoverage",
    "concurrencyBoundary",
    "checkpointPolicy",
    "humanDecisionPoints",
    "documentationImpact",
    "performanceImpact",
    "residualRiskExpectation",
    "governanceProfile",
    "requestedOperation",
    "implementationSurface",
    "restrictedWriteApproval",
    "adoptionBootstrapPaths",
    "requiredRuntimeCapabilities",
];

fn contract_from_bytes(bytes: &[u8]) -> Result<(ContractV1, Digest), String> {
    super::reject_duplicate_json_keys(bytes)?;
    let raw: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    let object = raw.as_object().ok_or("legacy Contract must be an object")?;
    if object
        .keys()
        .any(|key| !CONTRACT_FIELDS_V1.contains(&key.as_str()))
    {
        return Err("unsupported legacy Contract field semantics".into());
    }
    let digest = json_digest(&raw).map_err(|e| e.to_string())?;
    let contract: ContractV1 = serde_json::from_value(raw).map_err(|e| e.to_string())?;
    contract.material_inspection_review_profile()?;
    Ok((contract, digest))
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct SignalsV1 {
    pub untrusted_material: bool,
    pub test_weakening: bool,
    pub coverage_weakening: bool,
    pub unknowns: Vec<String>,
    pub findings: Vec<String>,
}

impl From<MaterialUnknownCause> for UnknownCauseV1 {
    fn from(value: MaterialUnknownCause) -> Self {
        match value {
            MaterialUnknownCause::MissingCompleteSource => Self::MissingCompleteSource,
            MaterialUnknownCause::NonTextMaterial => Self::NonTextMaterial,
            MaterialUnknownCause::SourceOverBudget => Self::SourceOverBudget,
            MaterialUnknownCause::InvalidProvenance => Self::InvalidProvenance,
            MaterialUnknownCause::ParseUnavailable => Self::ParseUnavailable,
            MaterialUnknownCause::AnalysisIncomplete => Self::AnalysisIncomplete,
            MaterialUnknownCause::ReadableCommittedRustSyntaxUnknown => {
                Self::ReadableCommittedRustSyntaxUnknown
            }
        }
    }
}
fn is_test_path(path: &str) -> bool {
    let normalized = path.to_ascii_lowercase();
    let file_name = normalized.rsplit('/').next().unwrap_or(&normalized);
    let spec_source_name = [
        ".rs", ".py", ".js", ".mjs", ".cjs", ".jsx", ".ts", ".mts", ".cts", ".tsx", ".go", ".java",
        ".kt", ".swift", ".sh", ".rb", ".php", ".cs", ".c", ".cc", ".cpp", ".h", ".hpp",
    ]
    .iter()
    .any(|extension| {
        file_name.strip_suffix(extension).is_some_and(|stem| {
            stem == "spec" || stem.ends_with("_spec") || stem.ends_with(".spec")
        })
    });
    normalized.starts_with("tests/")
        || normalized.starts_with("test/")
        || normalized.contains("/tests/")
        || normalized.contains("/test/")
        || normalized.contains("/spec/")
        || file_name.contains("test")
        || spec_source_name
}

fn is_coverage_path(path: &str) -> bool {
    let normalized = path.to_ascii_lowercase();
    let name = normalized.rsplit('/').next().unwrap_or(&normalized);
    matches!(
        name,
        ".coveragerc"
            | "coverage.json"
            | "coverage.yaml"
            | "coverage.yml"
            | "pyproject.toml"
            | "tox.ini"
            | "package.json"
    ) || name.starts_with("jest.config.")
}

fn is_textual_material_path(path: &str) -> bool {
    let normalized = path.to_ascii_lowercase();
    [
        ".md", ".txt", ".json", ".toml", ".yaml", ".yml", ".rs", ".py", ".js", ".mjs", ".cjs",
        ".ts", ".mts", ".cts", ".tsx", ".jsx", ".java", ".kt", ".swift", ".go", ".sh",
    ]
    .iter()
    .any(|extension| normalized.ends_with(extension))
}

/// The reference inventory is a large machine-generated conformance ledger.
/// Its own strict checker is the authority for its schema and path coverage;
/// treating an oversized JSON diff as source prose would otherwise make every
/// legitimate rebaseline permanently yellow.  This exemption is deliberately
/// exact and does not weaken inspection for other test or repository files.
fn is_strictly_checked_conformance_manifest(path: &str) -> bool {
    path == "tests/conformance/reference_file_inventory.json"
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MultilineSourceString {
    PythonSingle,
    PythonDouble,
    TripleDouble,
    Backtick,
    GoRaw,
    RustRaw(usize),
    Quoted(u8),
}

#[derive(Default)]
struct SkipSourceContext {
    block_comment_depth: usize,
    multiline_string: Option<MultilineSourceString>,
}

impl SkipSourceContext {
    fn is_code(&self) -> bool {
        self.block_comment_depth == 0 && self.multiline_string.is_none()
    }
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$') || !byte.is_ascii()
}

fn has_marker_boundary(line: &str, index: usize) -> bool {
    line.as_bytes()
        .get(index.wrapping_sub(1))
        .is_none_or(|byte| !is_identifier_byte(*byte) && *byte != b'.')
}

fn has_call(line: &str, marker: &str) -> bool {
    line.match_indices(marker).any(|(index, _)| {
        has_marker_boundary(line, index)
            && line[index + marker.len()..].trim_start().starts_with('(')
    })
}

fn has_call_or_jest_each(line: &str, marker: &str) -> bool {
    line.match_indices(marker).any(|(index, _)| {
        if !has_marker_boundary(line, index) {
            return false;
        }
        let suffix = line[index + marker.len()..].trim_start();
        suffix.starts_with('(')
            || suffix
                .strip_prefix(".each")
                .is_some_and(|after_each| after_each.trim_start().starts_with('('))
    })
}

fn has_bare_python_skip_decorator(line: &str) -> bool {
    line.match_indices("@pytest.mark.skip").any(|(index, _)| {
        has_marker_boundary(line, index)
            && line[index + "@pytest.mark.skip".len()..].trim().is_empty()
    })
}

fn has_python_skip_marker(line: &str) -> bool {
    has_bare_python_skip_decorator(line)
        || [
            "@pytest.mark.skip",
            "@pytest.mark.skipif",
            "@unittest.skip",
            "@unittest.skipIf",
            "pytest.skip",
            "pytest.skipif",
        ]
        .iter()
        .any(|marker| has_call(line, marker))
}

fn has_python_parameter_marker_candidate(line: &str) -> bool {
    ["pytest.mark.skipif", "pytest.mark.skip"]
        .iter()
        .any(|marker| {
            line.match_indices(marker).any(|(index, _)| {
                if !has_marker_boundary(line, index) {
                    return false;
                }
                let suffix = line[index + marker.len()..].trim_start();
                if marker.ends_with("skipif") {
                    suffix.starts_with('(')
                } else {
                    suffix.is_empty() || suffix.starts_with(['(', ',', ')', ']'])
                }
            })
        })
}

fn has_javascript_skip_marker(line: &str) -> bool {
    ["it.skip", "test.skip", "describe.skip"]
        .iter()
        .any(|marker| has_call_or_jest_each(line, marker))
        || ["xit", "xtest", "xdescribe", "this.skip"]
            .iter()
            .any(|marker| has_call(line, marker))
}

fn has_java_skip_annotation(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'@' {
            index += 1;
            continue;
        }
        let start = index + 1;
        let mut end = start;
        while bytes
            .get(end)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'$'))
        {
            end += 1;
        }
        if end == start {
            index += 1;
            continue;
        }
        let annotation = &line[start..end];
        let suffix = &line[end..];
        if matches!(annotation.rsplit('.').next(), Some("Disabled" | "Ignore"))
            && (suffix.is_empty()
                || suffix.starts_with(char::is_whitespace)
                || suffix.trim_start().starts_with('('))
        {
            return true;
        }
        index = end;
    }
    false
}

fn is_javascript_extension(extension: &str) -> bool {
    matches!(
        extension,
        "js" | "mjs" | "cjs" | "jsx" | "ts" | "mts" | "cts" | "tsx"
    )
}

// These extensions have a narrowly recognized test-disable syntax below.
// Other test-like paths still retain deletion, assertion-loss, and CI-bypass
// detection without interpreting a generic word or method named `skip`.
fn skip_marker_for_language(extension: &str, line: &str) -> bool {
    match extension {
        "rs" => line.match_indices("#[ignore").any(|(index, _)| {
            has_marker_boundary(line, index)
                && line[index + "#[ignore".len()..]
                    .trim_start()
                    .starts_with(['=', ']'])
        }),
        "py" => has_python_skip_marker(line),
        "js" | "mjs" | "cjs" | "jsx" | "ts" | "mts" | "cts" | "tsx" => {
            has_javascript_skip_marker(line)
        }
        "java" | "kt" => has_java_skip_annotation(line),
        "go" => ["t.Skip", "t.Skipf", "t.SkipNow"]
            .iter()
            .any(|marker| has_call(line, marker)),
        "swift" => ["XCTSkip", "XCTSkipIf"]
            .iter()
            .any(|marker| has_call(line, marker)),
        _ => false,
    }
}

fn matching_parenthesis(source: &str, opening: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (index, byte) in source.as_bytes().iter().enumerate().skip(opening) {
        match byte {
            b'(' => depth += 1,
            b')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn top_level_argument_ranges(source: &str, start: usize, end: usize) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut stack = Vec::new();
    let mut argument_start = start;
    for index in start..end {
        match source.as_bytes()[index] {
            b'(' => stack.push(b')'),
            b'[' => stack.push(b']'),
            b'{' => stack.push(b'}'),
            byte @ (b')' | b']' | b'}') if stack.last() == Some(&byte) => {
                stack.pop();
            }
            b',' if stack.is_empty() => {
                ranges.push((argument_start, index));
                argument_start = index + 1;
            }
            _ => {}
        }
    }
    if argument_start < end {
        ranges.push((argument_start, end));
    }
    ranges
}

fn has_python_parameterized_skip_marker(
    source_lines: &[String],
    changed_lines: &BTreeSet<usize>,
) -> bool {
    let source = source_lines.join("\n");
    for (parameter_index, _) in source.match_indices("pytest.param") {
        if !has_marker_boundary(&source, parameter_index) {
            continue;
        }
        let suffix = &source[parameter_index + "pytest.param".len()..];
        let opening = suffix.len() - suffix.trim_start().len();
        if !suffix[opening..].starts_with('(') {
            continue;
        }
        let opening = parameter_index + "pytest.param".len() + opening;
        let Some(closing) = matching_parenthesis(&source, opening) else {
            continue;
        };
        for (argument_start, argument_end) in
            top_level_argument_ranges(&source, opening + 1, closing)
        {
            let argument = &source[argument_start..argument_end];
            let Some(equal) = argument.find('=') else {
                continue;
            };
            if argument[..equal].trim() != "marks" {
                continue;
            }
            let value_start = argument_start + equal + 1;
            let value = &source[value_start..argument_end];
            for marker in ["pytest.mark.skipif", "pytest.mark.skip"] {
                for (marker_offset, _) in value.match_indices(marker) {
                    let marker_index = value_start + marker_offset;
                    if !has_marker_boundary(&source, marker_index) {
                        continue;
                    }
                    let suffix = value[marker_offset + marker.len()..].trim_start();
                    let marker_is_used = if marker.ends_with("skipif") {
                        suffix.starts_with('(')
                    } else {
                        suffix.is_empty() || suffix.starts_with(['(', ',', ')', ']'])
                    };
                    if marker_is_used {
                        let line_index = source.as_bytes()[..marker_index]
                            .iter()
                            .filter(|byte| **byte == b'\n')
                            .count();
                        if changed_lines.contains(&line_index) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

fn balanced_expression_end(
    extension: &str,
    source: &str,
    start: usize,
    open: u8,
    close: u8,
) -> Option<usize> {
    let expression = &source[start..];
    let mut context = SkipSourceContext::default();
    let mut code = String::with_capacity(expression.len());
    for (index, line) in expression.split('\n').enumerate() {
        if index > 0 {
            code.push('\n');
        }
        code.push_str(&code_only_skip_source_line(extension, line, &mut context));
    }
    let bytes = code.as_bytes();
    let mut depth = 1usize;
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == open {
            depth += 1;
        } else if bytes[index] == close {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(start + index);
            }
        }
        index += 1;
    }
    None
}

fn expression_may_disable_test(extension: &str, expression: &str) -> bool {
    skip_marker_for_language(extension, expression)
        || matches!(extension, "kt" | "swift")
            && ["skip", "assume", "testaborted", "xctskip"]
                .iter()
                .any(|marker| expression.to_ascii_lowercase().contains(marker))
}

fn has_javascript_template_skip_candidate(source: &str, changed_lines: &BTreeSet<usize>) -> bool {
    let bytes = source.as_bytes();
    let mut quote_index = 0;
    while quote_index < bytes.len() {
        if bytes[quote_index] != b'`' {
            quote_index += 1;
            continue;
        }
        let Some(literal_end) = quoted_literal_end(bytes, quote_index, 1, b'`') else {
            return false;
        };
        let mut content_index = quote_index + 1;
        while content_index < literal_end {
            if bytes.get(content_index..content_index + 2) == Some(b"${") {
                let escaped = bytes[..content_index]
                    .iter()
                    .rev()
                    .take_while(|byte| **byte == b'\\')
                    .count()
                    % 2
                    == 1;
                if escaped {
                    content_index += 2;
                    continue;
                }
                let expression_start = content_index + 2;
                if let Some(expression_end) =
                    balanced_expression_end("js", source, expression_start, b'{', b'}')
                {
                    if expression_end < literal_end
                        && has_javascript_regex_brace_ambiguity(
                            source,
                            expression_start,
                            expression_end,
                        )
                        && expression_may_disable_test("js", &source[expression_start..literal_end])
                        && changed_line_intersects_range(
                            source,
                            content_index,
                            literal_end,
                            changed_lines,
                        )
                    {
                        return true;
                    }
                    if expression_end < literal_end
                        && expression_may_disable_test(
                            "js",
                            &source[expression_start..expression_end],
                        )
                        && changed_line_intersects_range(
                            source,
                            content_index,
                            expression_end + 1,
                            changed_lines,
                        )
                    {
                        return true;
                    }
                    content_index = expression_end + 1;
                    continue;
                }
                return expression_may_disable_test("js", &source[expression_start..literal_end])
                    && changed_line_intersects_range(
                        source,
                        content_index,
                        literal_end,
                        changed_lines,
                    );
            }
            content_index += 1;
        }
        quote_index = literal_end + 1;
    }
    false
}

fn has_javascript_regex_brace_ambiguity(
    source: &str,
    expression_start: usize,
    expression_end: usize,
) -> bool {
    // This bounded scanner does not lex JavaScript regular-expression bodies.
    // If a slash occurs before a brace it mistook for the interpolation end,
    // the caller checks the entire interpolation for skip syntax and defers
    // the decision instead of treating that brace as code.
    source[expression_start..=expression_end].contains('/')
}

fn quoted_literal_end(
    bytes: &[u8],
    opening: usize,
    delimiter_len: usize,
    quote: u8,
) -> Option<usize> {
    let mut index = opening + delimiter_len;
    while index + delimiter_len <= bytes.len() {
        if bytes[index] == b'\\' {
            index = (index + 2).min(bytes.len());
        } else if bytes[index..index + delimiter_len]
            .iter()
            .all(|byte| *byte == quote)
        {
            return Some(index);
        } else {
            index += 1;
        }
    }
    None
}

fn interpolated_string_end(
    extension: &str,
    source: &str,
    opening: usize,
    delimiter_len: usize,
    raw_hashes: usize,
) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut index = opening + delimiter_len;
    while index + delimiter_len <= bytes.len() {
        if bytes[index..index + delimiter_len]
            .iter()
            .all(|byte| *byte == b'"')
            && (extension != "swift"
                || bytes
                    .get(index + delimiter_len..index + delimiter_len + raw_hashes)
                    .is_some_and(|hashes| hashes.iter().all(|byte| *byte == b'#')))
        {
            return Some(index);
        }
        if extension == "swift"
            && swift_interpolation_opening_len(bytes, index, raw_hashes).is_some()
        {
            let expression_start =
                index + swift_interpolation_opening_len(bytes, index, raw_hashes)?;
            let expression_end =
                balanced_expression_end(extension, source, expression_start, b'(', b')')?;
            index = expression_end + 1;
            continue;
        }
        if extension == "kt" && bytes.get(index..index + 2) == Some(b"${") {
            let expression_start = index + 2;
            let expression_end =
                balanced_expression_end(extension, source, expression_start, b'{', b'}')?;
            index = expression_end + 1;
            continue;
        }
        if bytes[index] == b'\\'
            && ((extension == "swift" && raw_hashes == 0) || delimiter_len == 1)
        {
            index = (index + 2).min(bytes.len());
        } else {
            index += 1;
        }
    }
    None
}

fn swift_interpolation_opening_len(bytes: &[u8], index: usize, raw_hashes: usize) -> Option<usize> {
    if bytes.get(index) != Some(&b'\\')
        || !bytes
            .get(index + 1..index + 1 + raw_hashes)
            .is_some_and(|hashes| hashes.iter().all(|byte| *byte == b'#'))
        || bytes.get(index + 1 + raw_hashes) != Some(&b'(')
    {
        return None;
    }
    Some(raw_hashes + 2)
}

fn preceding_swift_raw_hashes(bytes: &[u8], quote_index: usize) -> usize {
    let mut cursor = quote_index;
    while cursor > 0 && bytes[cursor - 1] == b'#' {
        cursor -= 1;
    }
    quote_index - cursor
}

fn changed_line_intersects_range(
    source: &str,
    start: usize,
    end: usize,
    changed_lines: &BTreeSet<usize>,
) -> bool {
    let first_line = source.as_bytes()[..start]
        .iter()
        .filter(|byte| **byte == b'\n')
        .count();
    let last_line = source.as_bytes()[..end.min(source.len())]
        .iter()
        .filter(|byte| **byte == b'\n')
        .count();
    (first_line..=last_line).any(|line| changed_lines.contains(&line))
}

fn changed_text_after(source: &str, start: usize, changed_lines: &BTreeSet<usize>) -> String {
    let mut changed_text = String::new();
    let mut line_start = 0;
    for (line_index, line) in source.split_inclusive('\n').enumerate() {
        let line_end = line_start + line.trim_end_matches('\n').len();
        if changed_lines.contains(&line_index) && line_end > start {
            let segment_start = start.max(line_start);
            changed_text.push_str(&source[segment_start..line_end]);
            changed_text.push('\n');
        }
        line_start += line.len();
    }
    changed_text
}

fn has_python_fstring_skip_candidate(source: &str, changed_lines: &BTreeSet<usize>) -> bool {
    let bytes = source.as_bytes();
    let mut quote_index = 0;
    while quote_index < bytes.len() {
        let quote = bytes[quote_index];
        if !matches!(quote, b'\'' | b'"') {
            quote_index += 1;
            continue;
        }
        let formatted_prefix_start = if quote_index > 0
            && matches!(bytes[quote_index - 1], b'f' | b'F')
            && (quote_index == 1 || !is_identifier_byte(bytes[quote_index - 2]))
        {
            Some(quote_index - 1)
        } else if quote_index > 1
            && matches!(
                &bytes[quote_index - 2..quote_index],
                b"fr" | b"fR" | b"Fr" | b"FR" | b"rf" | b"rF" | b"Rf" | b"RF"
            )
            && (quote_index == 2 || !is_identifier_byte(bytes[quote_index - 3]))
        {
            Some(quote_index - 2)
        } else {
            None
        };
        let Some(_prefix_start) = formatted_prefix_start else {
            quote_index += 1;
            continue;
        };
        let delimiter_len =
            if bytes.get(quote_index..quote_index + 3) == Some(&[quote, quote, quote]) {
                3
            } else {
                1
            };
        let Some(literal_end) = quoted_literal_end(bytes, quote_index, delimiter_len, quote) else {
            quote_index += 1;
            continue;
        };
        let content_start = quote_index + delimiter_len;
        let mut content_index = content_start;
        while content_index < literal_end {
            if bytes[content_index] == b'{' {
                if bytes.get(content_index + 1) == Some(&b'{') {
                    content_index += 2;
                    continue;
                }
                let expression_start = content_index + 1;
                // PEP 701 permits the expression to reuse the f-string's
                // quote. If the outer delimiter scan ended inside that
                // expression, keep scanning its balanced body and fail closed.
                if let Some(expression_end) =
                    balanced_expression_end("py", source, expression_start, b'{', b'}')
                    && expression_may_disable_test("py", &source[expression_start..expression_end])
                    && changed_line_intersects_range(
                        source,
                        content_index,
                        expression_end + 1,
                        changed_lines,
                    )
                {
                    return true;
                }
            }
            content_index += 1;
        }
        quote_index = literal_end + delimiter_len;
    }
    false
}

fn has_kotlin_or_swift_interpolation_candidate(
    extension: &str,
    source: &str,
    changed_lines: &BTreeSet<usize>,
) -> bool {
    let bytes = source.as_bytes();
    let mut quote_index = 0;
    while quote_index < bytes.len() {
        if bytes[quote_index] != b'"' {
            quote_index += 1;
            continue;
        }
        let delimiter_len = if bytes.get(quote_index..quote_index + 3) == Some(b"\"\"\"") {
            3
        } else {
            1
        };
        let raw_hashes = if extension == "swift" {
            preceding_swift_raw_hashes(bytes, quote_index)
        } else {
            0
        };
        let Some(literal_end) =
            interpolated_string_end(extension, source, quote_index, delimiter_len, raw_hashes)
        else {
            quote_index += 1;
            continue;
        };
        let content_start = quote_index + delimiter_len;
        let content = &source[content_start..literal_end];
        match extension {
            "kt" => {
                for (index, _) in content.match_indices("${") {
                    let expression_start = content_start + index + 2;
                    if let Some(expression_end) =
                        balanced_expression_end(extension, source, expression_start, b'{', b'}')
                        && expression_end < literal_end
                        && expression_may_disable_test(
                            extension,
                            &source[expression_start..expression_end],
                        )
                        && changed_line_intersects_range(
                            source,
                            content_start + index,
                            expression_end + 1,
                            changed_lines,
                        )
                    {
                        return true;
                    }
                }
            }
            "swift" => {
                for (index, _) in content.match_indices('\\') {
                    let Some(opening_len) =
                        swift_interpolation_opening_len(content.as_bytes(), index, raw_hashes)
                    else {
                        continue;
                    };
                    let expression_start = content_start + index + opening_len;
                    if let Some(expression_end) =
                        balanced_expression_end(extension, source, expression_start, b'(', b')')
                    {
                        let changed_suffix = changed_text_after(
                            source,
                            expression_end.saturating_add(1),
                            changed_lines,
                        );
                        if expression_end < literal_end
                            && source[expression_start..=expression_end].contains('/')
                            && expression_may_disable_test(extension, &changed_suffix)
                        {
                            return true;
                        }
                        if expression_end < literal_end
                            && expression_may_disable_test(
                                extension,
                                &source[expression_start..expression_end],
                            )
                            && changed_line_intersects_range(
                                source,
                                content_start + index,
                                expression_end + 1,
                                changed_lines,
                            )
                        {
                            return true;
                        }
                    }
                }
            }
            _ => {}
        }
        quote_index = literal_end + delimiter_len;
    }
    false
}

fn has_skip_interpolation_candidate_in_source(
    extension: &str,
    source: &str,
    changed_lines: &BTreeSet<usize>,
) -> bool {
    match extension {
        "js" | "mjs" | "cjs" | "jsx" | "ts" | "mts" | "cts" | "tsx" => {
            has_javascript_template_skip_candidate(source, changed_lines)
        }
        "py" => has_python_fstring_skip_candidate(source, changed_lines),
        "kt" | "swift" => {
            has_kotlin_or_swift_interpolation_candidate(extension, source, changed_lines)
        }
        _ => false,
    }
}

fn rust_raw_string_open(bytes: &[u8], index: usize) -> Option<(usize, usize)> {
    if index > 0 && (bytes[index - 1].is_ascii_alphanumeric() || bytes[index - 1] == b'_') {
        return None;
    }
    let marker_start = if bytes.get(index..index + 2) == Some(b"br") {
        index + 2
    } else if bytes.get(index) == Some(&b'r') {
        index + 1
    } else {
        return None;
    };
    let mut cursor = marker_start;
    while bytes.get(cursor) == Some(&b'#') {
        cursor += 1;
    }
    (bytes.get(cursor) == Some(&b'"')).then_some((cursor - marker_start, cursor + 1))
}

fn close_multiline_string(
    bytes: &[u8],
    index: usize,
    string: MultilineSourceString,
) -> Option<usize> {
    match string {
        MultilineSourceString::PythonSingle if bytes.get(index..index + 3) == Some(b"'''") => {
            Some(index + 3)
        }
        MultilineSourceString::PythonDouble | MultilineSourceString::TripleDouble
            if bytes.get(index..index + 3) == Some(b"\"\"\"") =>
        {
            Some(index + 3)
        }
        MultilineSourceString::Backtick | MultilineSourceString::GoRaw
            if bytes.get(index) == Some(&b'`') =>
        {
            Some(index + 1)
        }
        MultilineSourceString::Quoted(quote) if bytes.get(index) == Some(&quote) => Some(index + 1),
        MultilineSourceString::RustRaw(hashes)
            if bytes.get(index) == Some(&b'"')
                && bytes
                    .get(index + 1..index + 1 + hashes)
                    .is_some_and(|closing| closing.iter().all(|byte| *byte == b'#')) =>
        {
            Some(index + 1 + hashes)
        }
        _ => None,
    }
}

fn code_only_skip_source_line(
    extension: &str,
    line: &str,
    context: &mut SkipSourceContext,
) -> String {
    let bytes = line.as_bytes();
    let mut code = vec![b' '; bytes.len()];
    let nested_comments = matches!(extension, "rs" | "kt" | "swift");
    let block_comments = matches!(
        extension,
        "rs" | "js"
            | "mjs"
            | "cjs"
            | "jsx"
            | "ts"
            | "mts"
            | "cts"
            | "tsx"
            | "java"
            | "kt"
            | "swift"
            | "go"
    );
    let mut index = 0;
    while index < bytes.len() {
        if let Some(string) = context.multiline_string {
            if let Some(next) = close_multiline_string(bytes, index, string) {
                context.multiline_string = None;
                index = next;
            } else if matches!(
                string,
                MultilineSourceString::PythonSingle
                    | MultilineSourceString::PythonDouble
                    | MultilineSourceString::Backtick
                    | MultilineSourceString::Quoted(_)
            ) && bytes[index] == b'\\'
            {
                index = (index + 2).min(bytes.len());
            } else {
                index += 1;
            }
            continue;
        }
        if context.block_comment_depth > 0 {
            if nested_comments && bytes.get(index..index + 2) == Some(b"/*") {
                context.block_comment_depth += 1;
                index += 2;
            } else if bytes.get(index..index + 2) == Some(b"*/") {
                context.block_comment_depth -= 1;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }

        if matches!(extension, "py") && bytes.get(index..index + 3) == Some(b"'''") {
            context.multiline_string = Some(MultilineSourceString::PythonSingle);
            index += 3;
            continue;
        }
        if matches!(extension, "py" | "java" | "kt" | "swift")
            && bytes.get(index..index + 3) == Some(b"\"\"\"")
        {
            context.multiline_string = Some(if extension == "py" {
                MultilineSourceString::PythonDouble
            } else {
                MultilineSourceString::TripleDouble
            });
            index += 3;
            continue;
        }
        if extension == "rs"
            && let Some((hashes, next)) = rust_raw_string_open(bytes, index)
        {
            context.multiline_string = Some(MultilineSourceString::RustRaw(hashes));
            index = next;
            continue;
        }
        if is_javascript_extension(extension) && bytes[index] == b'`' {
            context.multiline_string = Some(MultilineSourceString::Backtick);
            index += 1;
            continue;
        }
        if extension == "go" && bytes[index] == b'`' {
            context.multiline_string = Some(MultilineSourceString::GoRaw);
            index += 1;
            continue;
        }
        if block_comments && bytes.get(index..index + 2) == Some(b"/*") {
            context.block_comment_depth = 1;
            index += 2;
            continue;
        }
        if (extension != "py" && bytes.get(index..index + 2) == Some(b"//"))
            || (extension == "py" && bytes[index] == b'#')
        {
            break;
        }
        if matches!(bytes[index], b'\'' | b'"') {
            let quote = bytes[index];
            if extension == "rs" && quote == b'\'' {
                let mut cursor = index + 1;
                let mut closes_on_line = false;
                while cursor < bytes.len() {
                    if bytes[cursor] == b'\\' {
                        cursor = (cursor + 2).min(bytes.len());
                    } else if bytes[cursor] == b'\'' {
                        closes_on_line = true;
                        break;
                    } else {
                        cursor += 1;
                    }
                }
                if !closes_on_line {
                    index += 1;
                    continue;
                }
            }
            index += 1;
            let mut closed = false;
            while index < bytes.len() {
                if bytes[index] == b'\\' {
                    index = (index + 2).min(bytes.len());
                } else if bytes[index] == quote {
                    index += 1;
                    closed = true;
                    break;
                } else {
                    index += 1;
                }
            }
            if !closed {
                context.multiline_string = Some(MultilineSourceString::Quoted(quote));
            }
            continue;
        }
        code[index] = bytes[index];
        index += 1;
    }
    String::from_utf8(code).expect("masking source bytes preserves UTF-8")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SkipMarkerScan {
    Finding,
    Clean,
    Ambiguous,
}

fn contains_skip_marker(path: &str, change: &cockpit_git::ChangeEvidence) -> SkipMarkerScan {
    let extension = path
        .rsplit('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !matches!(
        extension.as_str(),
        "rs" | "py"
            | "js"
            | "mjs"
            | "cjs"
            | "jsx"
            | "ts"
            | "mts"
            | "cts"
            | "tsx"
            | "java"
            | "kt"
            | "go"
            | "swift"
    ) {
        return SkipMarkerScan::Clean;
    }
    let Some(source) = change.after_text.as_deref() else {
        let all_added_lines = (0..change.added_lines.len()).collect::<BTreeSet<_>>();
        let added_source = change.added_lines.join("\n");
        let parameterized_skip = extension == "py"
            && has_python_parameterized_skip_marker(&change.added_lines, &all_added_lines);
        return if parameterized_skip
            || has_skip_interpolation_candidate_in_source(
                &extension,
                &added_source,
                &all_added_lines,
            )
            || change.added_lines.iter().any(|line| {
                skip_marker_for_language(&extension, line)
                    || (extension == "py" && has_python_parameter_marker_candidate(line))
            }) {
            SkipMarkerScan::Ambiguous
        } else {
            SkipMarkerScan::Clean
        };
    };
    let source_lines = source.lines().collect::<Vec<_>>();
    let mut changed_lines = BTreeSet::new();
    if !change.added_line_origins.is_empty() {
        if change.added_line_origins.len() != change.added_lines.len() {
            return SkipMarkerScan::Ambiguous;
        }
        for (origin, added_line) in change.added_line_origins.iter().zip(&change.added_lines) {
            let Some(line_index) = origin.after_line.checked_sub(1) else {
                return SkipMarkerScan::Ambiguous;
            };
            if source_lines
                .get(line_index)
                .is_none_or(|line| line.trim_end_matches('\r') != added_line)
            {
                return SkipMarkerScan::Ambiguous;
            }
            changed_lines.insert(line_index);
        }
    } else if change.kind == ChangeKind::Added
        || source_lines.join("\n") == change.added_lines.join("\n")
    {
        changed_lines.extend(0..source_lines.len());
    } else if change.added_lines.is_empty() {
        return SkipMarkerScan::Clean;
    } else if change.added_lines.iter().any(|line| {
        skip_marker_for_language(&extension, line)
            || (extension == "py" && has_python_parameter_marker_candidate(line))
    }) || has_skip_interpolation_candidate_in_source(
        &extension,
        &change.added_lines.join("\n"),
        &(0..change.added_lines.len()).collect(),
    ) || (extension == "py"
        && has_python_parameterized_skip_marker(
            &change.added_lines,
            &(0..change.added_lines.len()).collect(),
        ))
    {
        return SkipMarkerScan::Ambiguous;
    } else {
        return SkipMarkerScan::Clean;
    }

    if has_skip_interpolation_candidate_in_source(&extension, source, &changed_lines) {
        return SkipMarkerScan::Ambiguous;
    }

    let mut context = SkipSourceContext::default();
    let mut code_lines = Vec::with_capacity(source_lines.len());
    for (index, line) in source_lines.iter().enumerate() {
        let code_line = code_only_skip_source_line(&extension, line, &mut context);
        if changed_lines.contains(&index) && skip_marker_for_language(&extension, &code_line) {
            return SkipMarkerScan::Finding;
        }
        code_lines.push(code_line);
    }
    if extension == "py" && has_python_parameterized_skip_marker(&code_lines, &changed_lines) {
        return SkipMarkerScan::Finding;
    }
    if context.is_code() {
        SkipMarkerScan::Clean
    } else {
        SkipMarkerScan::Ambiguous
    }
}

fn assertion_count(lines: &[String]) -> usize {
    lines
        .iter()
        .filter(|line| {
            let line = line.to_ascii_lowercase();
            ["assert", "expect(", "xctassert", "tothrow("]
                .iter()
                .any(|marker| line.contains(marker))
        })
        .count()
}

fn contains_test_bypass(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    text.lines().any(|line| {
        line.contains("continue-on-error: true")
            || line.contains("allow_failure: true")
            || line.contains("|| true")
    })
}

fn assignment(line: &str) -> Option<(String, String)> {
    let (name, value) = line.split_once('=').or_else(|| line.split_once(':'))?;
    Some((
        name.trim().to_ascii_lowercase(),
        value.trim().trim_matches(['\'', '"']).to_ascii_lowercase(),
    ))
}

fn first_number(value: &str) -> Option<f64> {
    let start = value.find(|character: char| character.is_ascii_digit())?;
    let number = value[start..]
        .chars()
        .take_while(|character| character.is_ascii_digit() || *character == '.')
        .collect::<String>();
    number.parse().ok()
}

fn coverage_weakened(removed: &[String], added: &[String]) -> bool {
    let removed = removed
        .iter()
        .filter_map(|line| assignment(line))
        .collect::<Vec<_>>();
    let added = added
        .iter()
        .filter_map(|line| assignment(line))
        .collect::<Vec<_>>();
    for (name, before) in &removed {
        if matches!(name.as_str(), "fail_under" | "threshold" | "minimum")
            && let Some((_, after)) = added.iter().find(|(candidate, _)| candidate == name)
            && let (Some(before), Some(after)) = (first_number(before), first_number(after))
            && after < before
        {
            return true;
        }
        if matches!(name.as_str(), "source" | "source_pkgs")
            && let Some((_, after)) = added.iter().find(|(candidate, _)| candidate == name)
        {
            let before = before
                .split([',', ' '])
                .filter(|item| !item.is_empty())
                .count();
            let after = after
                .split([',', ' '])
                .filter(|item| !item.is_empty())
                .count();
            if after < before {
                return true;
            }
        }
    }
    added.iter().any(|(name, value)| {
        matches!(
            name.as_str(),
            "omit" | "exclude" | "exclude_lines" | "coveragepathignorepatterns"
        ) && !removed
            .iter()
            .any(|(old_name, old_value)| old_name == name && old_value == value)
    })
}

pub(super) fn derive_governance_signals(snapshot: &RepositorySnapshot) -> SignalsV1 {
    derive_governance_signals_with_diagnostics(snapshot).0
}

pub(super) fn derive_governance_signals_with_diagnostics(
    snapshot: &RepositorySnapshot,
) -> (SignalsV1, BTreeMap<String, RustMaterialDiagnosis>) {
    let mut result = SignalsV1::default();
    let mut rust_diagnostics = BTreeMap::new();
    for change in &snapshot.change_evidence {
        if change.path.starts_with(".ai/") {
            continue;
        }
        let test_path = is_test_path(&change.path);
        let coverage_path = is_coverage_path(&change.path);
        let inspectable = matches!(
            change.content_state,
            ChangeContentState::Text | ChangeContentState::Deleted
        );
        if !inspectable {
            let strictly_checked_manifest = is_strictly_checked_conformance_manifest(&change.path);
            if test_path && !strictly_checked_manifest {
                result
                    .unknowns
                    .push("test_weakening_inspection_unavailable".into());
            }
            if coverage_path {
                result
                    .unknowns
                    .push("coverage_weakening_inspection_unavailable".into());
            }
            if is_textual_material_path(&change.path) && !strictly_checked_manifest {
                result
                    .unknowns
                    .push("repository_material_inspection_unavailable".into());
            }
            continue;
        }

        let added_text = change.added_lines.join("\n");
        // Untracked material has no Git patch lines yet, so inspect its bounded
        // current text. For tracked files, inspect only added lines; scanning
        // the whole source file would match detector examples embedded in the
        // Runtime itself and produce a false governance finding.
        let material_text = if change.added_lines.is_empty() && change.kind == ChangeKind::Added {
            change.after_text.as_deref().unwrap_or("")
        } else {
            &added_text
        };
        if change.path.to_ascii_lowercase().ends_with(".rs") {
            let diagnosis = diagnose_rust_material_v1(change);
            rust_diagnostics.insert(change.path.clone(), diagnosis);
            match diagnosis.assessment {
                RustMaterialAssessment::Finding => {
                    result.untrusted_material = true;
                    result.findings.push("repository_prompt_injection".into());
                }
                RustMaterialAssessment::Unknown => result
                    .unknowns
                    .push("repository_material_inspection_unavailable".into()),
                RustMaterialAssessment::Clean => {}
            }
        } else if contains_strong_instruction_injection(material_text) {
            result.untrusted_material = true;
            result.findings.push("repository_prompt_injection".into());
        }
        let skip_marker = if test_path {
            contains_skip_marker(&change.path, change)
        } else {
            SkipMarkerScan::Clean
        };
        if skip_marker == SkipMarkerScan::Ambiguous {
            result
                .unknowns
                .push("test_weakening_inspection_unavailable".into());
        }
        if test_path
            && (change.kind == ChangeKind::Deleted
                || skip_marker == SkipMarkerScan::Finding
                || assertion_count(&change.removed_lines) > assertion_count(&change.added_lines)
                || contains_test_bypass(&added_text))
        {
            result.test_weakening = true;
            result.findings.push("test_weakening".into());
        }
        if coverage_path && coverage_weakened(&change.removed_lines, &change.added_lines) {
            result.coverage_weakening = true;
            result.findings.push("coverage_weakening".into());
        }
    }
    result.unknowns.sort();
    result.unknowns.dedup();
    result.findings.sort();
    result.findings.dedup();
    (result, rust_diagnostics)
}

fn policy_document(root: &Path) -> Result<Option<PolicyDocumentV1>, ObserverError> {
    let path = root.join(".ai/policy.json");
    if !path.is_file() {
        return Ok(None);
    }
    let value = super::read_json(&path)?;
    let document: PolicyDocumentV1 =
        serde_json::from_value(value).map_err(|error| ObserverError::State {
            path: path.clone(),
            message: format!("invalid governance policy: {error}"),
        })?;
    if document.schema_version != 1 {
        return Err(ObserverError::State {
            path,
            message: format!(
                "unsupported governance policy schema {}",
                document.schema_version
            ),
        });
    }
    if document.organization.is_none() && document.project.is_none() {
        return Err(ObserverError::State {
            path,
            message: "governance policy must define organization or project policy".into(),
        });
    }
    if document
        .organization
        .as_ref()
        .is_some_and(|policy| !matches!(policy.layer, PolicyLayerV1::Organization))
        || document
            .project
            .as_ref()
            .is_some_and(|policy| !matches!(policy.layer, PolicyLayerV1::Project))
    {
        return Err(ObserverError::State {
            path,
            message: "governance policy layer does not match its document slot".into(),
        });
    }
    Ok(Some(document))
}

/// Return the effective repository + Work Item policy, if the repository has
/// opted into policy enforcement. Policy bytes remain repository-local and
/// are never inferred from natural-language requests.
fn effective_policy_for_contract(
    root: &Path,
    contract: &ContractV1,
) -> Result<Option<PolicyV1>, ObserverError> {
    let Some(document) = policy_document(root)? else {
        return Ok(None);
    };
    let mut layers = Vec::new();
    if let Some(organization) = document.organization.as_ref() {
        layers.push(organization);
    }
    if let Some(project) = document.project.as_ref() {
        layers.push(project);
    }
    if let Some(work_item) = contract.governance_policy.as_ref() {
        if !matches!(work_item.layer, PolicyLayerV1::WorkItem) {
            return Err(ObserverError::State {
                path: root.join(".ai/work-items/active"),
                message: "Work Item governance policy must use layer=work_item".into(),
            });
        }
        layers.push(work_item);
    }
    merge_policy_layers(&layers)
        .map(Some)
        .map_err(|error| ObserverError::State {
            path: root.join(".ai/policy.json"),
            message: error.to_string(),
        })
}

fn git_output_error(output: &BoundedGitOutput) -> MaterialReviewRequestError {
    MaterialReviewRequestError::Git(String::from_utf8_lossy(&output.stderr).into_owned())
}

fn reject_nonstandard_index_flags(git: &GitRepository) -> Result<(), MaterialReviewRequestError> {
    let output = git
        .material_v1_index_flags_bounded(MAX_PATCH_BYTES)
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    if !output.success {
        return Err(git_output_error(&output));
    }
    for record in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        if record.len() < 2 {
            return Err(MaterialReviewRequestError::Identity(
                "malformed index flags".into(),
            ));
        }
        let (flag, path) = record.split_at(2);
        if path != b".ai" && !path.starts_with(b".ai/") && flag != b"H " {
            return Err(MaterialReviewRequestError::DirtySource(format!(
                "nonstandard index flag for {}",
                String::from_utf8_lossy(path)
            )));
        }
    }
    Ok(())
}

fn committed_blob_ids(
    git: &GitRepository,
    revision: &str,
) -> Result<BTreeMap<String, String>, MaterialReviewRequestError> {
    let output = git
        .material_v1_committed_tree_bounded(revision, MAX_PATCH_BYTES)
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    if !output.success {
        return Err(git_output_error(&output));
    }
    let mut objects = BTreeMap::new();
    for record in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let Some(tab) = record.iter().position(|byte| *byte == b'\t') else {
            return Err(MaterialReviewRequestError::Identity(
                "malformed committed tree entry".into(),
            ));
        };
        let metadata = &record[..tab];
        let path = &record[tab + 1..];
        let path = std::str::from_utf8(path)
            .map_err(|_| MaterialReviewRequestError::Identity("non-UTF-8 committed path".into()))?;
        if path == ".ai" || path.starts_with(".ai/") {
            continue;
        }
        let fields = std::str::from_utf8(metadata).map_err(|_| {
            MaterialReviewRequestError::Identity("malformed committed tree metadata".into())
        })?;
        let mut fields = fields.split_whitespace();
        let (Some(_mode), Some(kind), Some(id)) = (fields.next(), fields.next(), fields.next())
        else {
            return Err(MaterialReviewRequestError::Identity(
                "malformed committed tree metadata".into(),
            ));
        };
        if kind != "blob" {
            return Err(MaterialReviewRequestError::Identity(
                "non-blob committed material".into(),
            ));
        }
        objects.insert(path.to_owned(), id.to_owned());
    }
    Ok(objects)
}

fn no_hunk_blob_identity_is_proven(
    change: &cockpit_git::ChangeEvidence,
    base_blob_id: Option<&str>,
    head_blob_id: &str,
    deleted_base_blob_ids: &BTreeSet<String>,
    head_blob_is_empty: bool,
) -> bool {
    if !change.added_lines.is_empty() || !change.removed_lines.is_empty() {
        return true;
    }
    match change.kind {
        ChangeKind::Modified => base_blob_id == Some(head_blob_id),
        ChangeKind::Added => head_blob_is_empty || deleted_base_blob_ids.contains(head_blob_id),
        ChangeKind::Renamed => {
            base_blob_id == Some(head_blob_id) || deleted_base_blob_ids.contains(head_blob_id)
        }
        _ => false,
    }
}

fn committed_blob(
    git: &GitRepository,
    id: &str,
    max_output_bytes: usize,
    aggregate_budget_limited: bool,
) -> Result<Vec<u8>, MaterialReviewRequestError> {
    let output =
        git.material_v1_blob_bounded(id, max_output_bytes)
            .map_err(|error| match error {
                GitError::OutputLimitExceeded { .. } if aggregate_budget_limited => {
                    MaterialReviewRequestError::BudgetExceeded {
                        budget: "total_blob_bytes",
                        limit: MAX_MATERIAL_TOTAL_BLOB_BYTES,
                    }
                }
                GitError::OutputLimitExceeded { .. } => MaterialReviewRequestError::Identity(
                    "committed source exceeds per-blob request budget".into(),
                ),
                error => MaterialReviewRequestError::Git(error.to_string()),
            })?;
    if !output.success {
        return Err(git_output_error(&output));
    }
    Ok(output.stdout)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestEntry<'a> {
    path: &'a str,
    kind: &'a str,
    content_state: &'a str,
    added_lines: &'a [String],
    added_line_origins: Vec<(usize, usize)>,
    removed_lines: &'a [String],
    changed_hunk_digest: &'a Digest,
    after_blob_digest: &'a Option<Digest>,
}

fn digest_json(value: &impl Serialize) -> Result<Digest, serde_json::Error> {
    serde_json::to_vec(value).map(|bytes| Digest::sha256_bytes(&bytes))
}
fn json_digest(value: &impl Serialize) -> Result<Digest, MaterialReviewRequestError> {
    digest_json(value).map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))
}

pub(super) fn changed_hunk_digest(
    change: &ChangeEvidence,
) -> Result<Digest, MaterialReviewRequestError> {
    let origins = change
        .added_line_origins
        .iter()
        .map(|origin| (origin.after_line, origin.hunk_index))
        .collect::<Vec<_>>();
    json_digest(&(&change.added_lines, &origins, &change.removed_lines))
}

fn valid_digest(digest: &Digest) -> bool {
    digest.as_str().parse::<Digest>().is_ok()
}

fn valid_utc_runtime_timestamp(value: &str) -> bool {
    value.trim() == value
        && value.ends_with('Z')
        && chrono::DateTime::parse_from_rfc3339(value)
            .is_ok_and(|timestamp| timestamp.offset().local_minus_utc() == 0)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DecisionInputV1 {
    pub schema_version: u32,
    pub decision: MaterialInspectionReviewDecision,
    pub request_digest: Digest,
    pub reviewer_actor: String,
    pub authority_source: String,
    pub assurance: MaterialInspectionReviewAssurance,
    pub evidence_refs: Vec<MaterialInspectionReviewEvidenceRef>,
    pub rationale: String,
    pub residual_risk: String,
}

fn build_request_v1(
    root: &Path,
    contract: &ContractV1,
    contract_digest: &Digest,
    implementation_digest: &Digest,
) -> Result<MaterialReviewRequest, MaterialReviewRequestError> {
    let git = GitRepository::discover(root)
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    let root = git.root();
    if contract.repository_id != repository_id(root).to_string() {
        return Err(MaterialReviewRequestError::ContractIdentity);
    }
    reject_nonstandard_index_flags(&git)?;
    let working = git
        .material_source_snapshot_v1_bounded(MAX_PATCH_BYTES)
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    let dirty = working
        .changed_paths
        .iter()
        .filter(|path| *path != ".ai" && !path.starts_with(".ai/"))
        .cloned()
        .collect::<Vec<_>>();
    if !dirty.is_empty() {
        return Err(MaterialReviewRequestError::DirtySource(dirty.join(", ")));
    }
    let mut snapshot = git
        .material_source_snapshot_against_v1_bounded(&contract.base_revision, MAX_PATCH_BYTES)
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    let head = snapshot
        .head
        .clone()
        .ok_or_else(|| MaterialReviewRequestError::Identity("committed HEAD is required".into()))?;
    if working.head.as_deref() != Some(head.as_str())
        || working.source_tree_digest != snapshot.source_tree_digest.as_deref().unwrap_or_default()
    {
        return Err(MaterialReviewRequestError::Identity(
            "source changed during request observation".into(),
        ));
    }
    let is_ancestor = git
        .material_v1_is_ancestor_bounded(&contract.base_revision, &head, MAX_PATCH_BYTES)
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    if !is_ancestor {
        return Err(MaterialReviewRequestError::Identity(
            "Contract base is not an ancestor of committed HEAD".into(),
        ));
    }
    snapshot
        .change_evidence
        .sort_by(|left, right| left.path.cmp(&right.path));
    let changed_file_count = snapshot
        .change_evidence
        .iter()
        .filter(|change| change.path != ".ai" && !change.path.starts_with(".ai/"))
        .count();
    if changed_file_count > MAX_MATERIAL_CHANGED_FILES {
        return Err(MaterialReviewRequestError::BudgetExceeded {
            budget: "changed_file_count",
            limit: MAX_MATERIAL_CHANGED_FILES as u64,
        });
    }
    let source_snapshot_digest = snapshot
        .source_tree_digest
        .as_deref()
        .ok_or_else(|| {
            MaterialReviewRequestError::Identity("source tree digest is missing".into())
        })?
        .parse::<Digest>()
        .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?;
    let base_objects = committed_blob_ids(&git, &contract.base_revision)?;
    let committed_objects = committed_blob_ids(&git, &head)?;
    let deleted_base_blob_ids = snapshot
        .change_evidence
        .iter()
        .filter(|change| change.kind == ChangeKind::Deleted)
        .filter_map(|change| base_objects.get(&change.path).cloned())
        .collect::<BTreeSet<_>>();
    let mut committed_digests = BTreeMap::new();
    let mut total_blob_bytes = 0_u64;
    let mut total_text_bytes = 0_u64;
    for change in snapshot
        .change_evidence
        .iter_mut()
        .filter(|change| change.path != ".ai" && !change.path.starts_with(".ai/"))
    {
        if change.kind == ChangeKind::Deleted {
            continue;
        }
        let id = committed_objects.get(&change.path).ok_or_else(|| {
            MaterialReviewRequestError::SourceUnavailable {
                path: change.path.clone(),
                reason: "source has no committed blob".into(),
            }
        })?;
        let remaining_blob_bytes = MAX_MATERIAL_TOTAL_BLOB_BYTES.saturating_sub(total_blob_bytes);
        if remaining_blob_bytes == 0 {
            return Err(MaterialReviewRequestError::BudgetExceeded {
                budget: "total_blob_bytes",
                limit: MAX_MATERIAL_TOTAL_BLOB_BYTES,
            });
        }
        let blob_limit = MAX_MATERIAL_BLOB_BYTES.min(remaining_blob_bytes);
        let bytes = committed_blob(
            &git,
            id,
            blob_limit as usize,
            remaining_blob_bytes < MAX_MATERIAL_BLOB_BYTES,
        )?;
        total_blob_bytes = total_blob_bytes.saturating_add(bytes.len() as u64);
        // The committed object supplies the digest and scan bytes. Checkout
        // filters can normalize line endings, so working-tree bytes are never
        // compared with a blob ID.
        committed_digests.insert(change.path.clone(), Digest::sha256_bytes(&bytes));
        if matches!(
            change.content_state,
            ChangeContentState::Binary | ChangeContentState::TooLarge
        ) {
            continue;
        }
        if bytes.len() > 4 * 1024 * 1024 {
            change.content_state = ChangeContentState::TooLarge;
            change.after_text = None;
            continue;
        }
        if change.content_state == ChangeContentState::Unavailable
            && change.added_lines.is_empty()
            && change.removed_lines.is_empty()
            && !no_hunk_blob_identity_is_proven(
                change,
                base_objects.get(&change.path).map(String::as_str),
                id,
                &deleted_base_blob_ids,
                bytes.is_empty(),
            )
        {
            continue;
        }
        match String::from_utf8(bytes) {
            Ok(text) => {
                if change.content_state == ChangeContentState::Unavailable
                    && text.len() > MAX_CHANGE_TEXT_BYTES
                {
                    change.content_state = ChangeContentState::TooLarge;
                    change.after_text = None;
                } else {
                    let next_text_bytes = total_text_bytes.saturating_add(text.len() as u64);
                    if next_text_bytes > MAX_MATERIAL_TOTAL_TEXT_BYTES {
                        return Err(MaterialReviewRequestError::BudgetExceeded {
                            budget: "total_text_bytes",
                            limit: MAX_MATERIAL_TOTAL_TEXT_BYTES,
                        });
                    }
                    change.content_state = ChangeContentState::Text;
                    change.after_text = Some(text);
                    total_text_bytes = next_text_bytes;
                }
            }
            Err(_) => {
                change.content_state = ChangeContentState::Binary;
                change.after_text = None;
            }
        }
    }
    let (signals, rust_diagnostics) = derive_governance_signals_with_diagnostics(&snapshot);
    let mut entries = Vec::new();
    let mut manifest_hasher = Sha256::new();
    manifest_hasher.update(b"ai-cockpit:material-manifest:v1\0");
    for change in snapshot
        .change_evidence
        .iter()
        .filter(|change| change.path != ".ai" && !change.path.starts_with(".ai/"))
    {
        let after_blob_digest = if change.kind == ChangeKind::Deleted {
            None
        } else {
            super::material_review::verify_checkout_source(root, &change.path)?;
            committed_digests.get(&change.path).cloned()
        };
        let kind = match change.kind {
            ChangeKind::Added => "added",
            ChangeKind::Modified => "modified",
            ChangeKind::Deleted => "deleted",
            ChangeKind::Renamed => "renamed",
            ChangeKind::Copied => "copied",
            ChangeKind::Unknown => "unknown",
        }
        .to_owned();
        let content_state = match change.content_state {
            ChangeContentState::Text => "text",
            ChangeContentState::Binary => "binary",
            ChangeContentState::TooLarge => "toolarge",
            ChangeContentState::Deleted => "deleted",
            ChangeContentState::Unavailable => "unavailable",
        }
        .to_owned();
        let origins = change
            .added_line_origins
            .iter()
            .map(|origin| (origin.after_line, origin.hunk_index))
            .collect::<Vec<_>>();
        let changed_hunk_digest = changed_hunk_digest(change)?;
        let diagnosis = rust_diagnostics.get(&change.path);
        let scanner_assessment = if !matches!(
            change.content_state,
            ChangeContentState::Text | ChangeContentState::Deleted
        ) {
            MaterialScannerAssessment::Unknown
        } else if let Some(diagnosis) = diagnosis {
            match diagnosis.assessment {
                RustMaterialAssessment::Clean => MaterialScannerAssessment::Clean,
                RustMaterialAssessment::Finding => MaterialScannerAssessment::Finding,
                RustMaterialAssessment::Unknown => MaterialScannerAssessment::Unknown,
            }
        } else {
            let added_text = change.added_lines.join("\n");
            let material_text = if change.added_lines.is_empty() && change.kind == ChangeKind::Added
            {
                change.after_text.as_deref().unwrap_or("")
            } else {
                &added_text
            };
            if contains_strong_instruction_injection(material_text) {
                MaterialScannerAssessment::Finding
            } else {
                MaterialScannerAssessment::Clean
            }
        };
        let complete_provenance = change.added_lines.len() == change.added_line_origins.len();
        let unknown_cause = if diagnosis
            .is_some_and(|diagnosis| diagnosis.assessment == RustMaterialAssessment::Unknown)
            && !complete_provenance
        {
            Some(MaterialUnknownCause::InvalidProvenance)
        } else if let Some(diagnosis) = diagnosis {
            diagnosis.unknown_cause
        } else if !matches!(
            change.content_state,
            ChangeContentState::Text | ChangeContentState::Deleted
        ) {
            Some(match change.content_state {
                ChangeContentState::Binary => MaterialUnknownCause::NonTextMaterial,
                ChangeContentState::TooLarge => MaterialUnknownCause::SourceOverBudget,
                _ => MaterialUnknownCause::MissingCompleteSource,
            })
        } else {
            None
        };
        let reviewable =
            unknown_cause == Some(MaterialUnknownCause::ReadableCommittedRustSyntaxUnknown);
        let entry = MaterialReviewEntry {
            path: change.path.clone(),
            kind,
            content_state,
            scanner_assessment,
            unknown_cause: unknown_cause.map(UnknownCauseV1::from),
            reviewable,
            changed_hunk_digest,
            after_blob_digest,
        };
        let manifest_entry = ManifestEntry {
            path: &entry.path,
            kind: &entry.kind,
            content_state: &entry.content_state,
            added_lines: &change.added_lines,
            added_line_origins: origins,
            removed_lines: &change.removed_lines,
            changed_hunk_digest: &entry.changed_hunk_digest,
            after_blob_digest: &entry.after_blob_digest,
        };
        let bytes = serde_json::to_vec(&manifest_entry)
            .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?;
        manifest_hasher.update((bytes.len() as u64).to_le_bytes());
        manifest_hasher.update(bytes);
        entries.push(entry);
    }
    let patch_digest = snapshot
        .diff_digest
        .parse::<Digest>()
        .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?;
    manifest_hasher.update(patch_digest.as_str().as_bytes());
    let material_manifest_digest = Digest::sha256_bytes(&manifest_hasher.finalize());
    let policy = effective_policy_for_contract(root, contract)
        .map_err(|error| MaterialReviewRequestError::Identity(error.to_string()))?;
    let profile = contract
        .material_inspection_review_profile()
        .map_err(MaterialReviewRequestError::Identity)?;
    let profile_digest = profile.as_ref().map(json_digest).transpose()?;
    let review_enabled = profile.is_some()
        && contract
            .required_runtime_capabilities
            .iter()
            .any(|capability| capability == MATERIAL_INSPECTION_REVIEW_CAPABILITY);
    let finding_codes = signals.findings;
    let blocked_by_finding = !finding_codes.is_empty();
    let mut request = MaterialReviewRequest {
        schema_version: 1,
        repository_id: contract.repository_id.clone(),
        work_item_id: contract.work_item_id.clone(),
        contract_digest: contract_digest.clone(),
        immutable_contract_base_revision: contract.base_revision.clone(),
        source_snapshot_digest,
        material_manifest_digest,
        scanner_semantic_version: SCANNER_SEMANTIC_VERSION.into(),
        analysis_implementation_digest: implementation_digest.clone(),
        analysis_target_semantic_profile: ANALYSIS_TARGET_SEMANTIC_PROFILE.into(),
        material_inspection_review_profile_digest: profile_digest,
        effective_policy_digest: json_digest(&policy)?,
        entries,
        raw_unknown_codes: signals.unknowns,
        finding_codes,
        blocked_by_finding,
        review_enabled,
        review_diagnostic: (!review_enabled).then(|| "material_review_not_enabled".into()),
        request_digest: Digest::sha256_bytes(b"uncomputed"),
        reviewed_source_head: head,
    };
    request.request_digest =
        codec::request_digest_v1(&request).map_err(MaterialReviewRequestError::Identity)?;
    let final_snapshot = git
        .material_source_snapshot_v1_bounded(MAX_PATCH_BYTES)
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    reject_nonstandard_index_flags(&git)?;
    let final_dirty = final_snapshot
        .changed_paths
        .iter()
        .any(|path| path != ".ai" && !path.starts_with(".ai/"));
    if final_dirty
        || final_snapshot.head.as_deref() != Some(request.reviewed_source_head.as_str())
        || final_snapshot.source_tree_digest != working.source_tree_digest
    {
        return Err(MaterialReviewRequestError::Identity(
            "source changed during request observation".into(),
        ));
    }
    Ok(request)
}

fn validate_decision_v1(
    contract: &ContractV1,
    contract_digest: &Digest,
    request: &MaterialReviewRequest,
    input: &DecisionInputV1,
    recorded_by: &str,
    recorded_at: &str,
) -> Result<MaterialInspectionReviewDecisionReceipt, MaterialReviewDecisionValidationError> {
    let profile = contract
        .material_inspection_review_profile()
        .map_err(MaterialReviewDecisionValidationError::InvalidProfile)?
        .ok_or(MaterialReviewDecisionValidationError::ReviewNotEnabled)?;
    if !contract
        .required_runtime_capabilities
        .iter()
        .any(|capability| capability == MATERIAL_INSPECTION_REVIEW_CAPABILITY)
        || !request.review_enabled
    {
        return Err(MaterialReviewDecisionValidationError::ReviewNotEnabled);
    }
    let profile_digest = json_digest(&profile)
        .map_err(|_| MaterialReviewDecisionValidationError::RequestIdentityMismatch)?;
    if request.schema_version != 1
        || request.repository_id != contract.repository_id
        || request.work_item_id != contract.work_item_id
        || request.contract_digest != *contract_digest
        || request.immutable_contract_base_revision != contract.base_revision
        || request.material_inspection_review_profile_digest.as_ref() != Some(&profile_digest)
    {
        return Err(MaterialReviewDecisionValidationError::RequestIdentityMismatch);
    }
    if request.blocked_by_finding
        || !request.finding_codes.is_empty()
        || request
            .entries
            .iter()
            .any(|entry| entry.scanner_assessment == MaterialScannerAssessment::Finding)
    {
        return Err(MaterialReviewDecisionValidationError::FindingPresent);
    }
    if request.raw_unknown_codes.len() != 1
        || request.raw_unknown_codes[0] != profile.permitted_unknown
    {
        return Err(MaterialReviewDecisionValidationError::UnexpectedUnknown);
    }
    let mut reviewable_unknowns = 0;
    for entry in &request.entries {
        if entry.scanner_assessment == MaterialScannerAssessment::Unknown {
            reviewable_unknowns += 1;
            if !entry.reviewable
                || entry.unknown_cause != Some(UnknownCauseV1::ReadableCommittedRustSyntaxUnknown)
            {
                return Err(MaterialReviewDecisionValidationError::UnexpectedUnknown);
            }
        }
    }
    if reviewable_unknowns == 0 {
        return Err(MaterialReviewDecisionValidationError::NoReviewableUnknown);
    }
    if !valid_digest(&request.contract_digest)
        || !valid_digest(&request.source_snapshot_digest)
        || !valid_digest(&request.material_manifest_digest)
        || !valid_digest(&request.analysis_implementation_digest)
        || !valid_digest(&request.effective_policy_digest)
        || !request
            .material_inspection_review_profile_digest
            .as_ref()
            .is_some_and(valid_digest)
        || request.entries.iter().any(|entry| {
            !valid_digest(&entry.changed_hunk_digest)
                || entry
                    .after_blob_digest
                    .as_ref()
                    .is_some_and(|digest| !valid_digest(digest))
        })
    {
        return Err(MaterialReviewDecisionValidationError::RequestIdentityMismatch);
    }
    let canonical_request_digest = codec::request_digest_v1(request)
        .map_err(|_| MaterialReviewDecisionValidationError::RequestIdentityMismatch)?;
    if !valid_digest(&request.request_digest)
        || canonical_request_digest != request.request_digest
        || input.request_digest != request.request_digest
        || !valid_digest(&input.request_digest)
    {
        return Err(MaterialReviewDecisionValidationError::RequestDigestMismatch);
    }
    if input.schema_version != MATERIAL_INSPECTION_REVIEW_DECISION_SCHEMA_VERSION
        || input.assurance != MaterialInspectionReviewAssurance::SelfDeclared
        || profile.assurance != "self_declared"
        || input.rationale.trim().is_empty()
        || input.rationale.trim() != input.rationale
        || input.residual_risk.trim().is_empty()
        || input.residual_risk.trim() != input.residual_risk
        || input.reviewer_actor.trim().is_empty()
        || input.reviewer_actor.trim() != input.reviewer_actor
        || input.authority_source.trim().is_empty()
        || input.authority_source.trim() != input.authority_source
    {
        return Err(MaterialReviewDecisionValidationError::InvalidInput);
    }
    if input.reviewer_actor != profile.reviewer_actor {
        return Err(MaterialReviewDecisionValidationError::ReviewerMismatch);
    }
    if input.authority_source != profile.authority_source {
        return Err(MaterialReviewDecisionValidationError::AuthorityMismatch);
    }
    let mut evidence_refs = input.evidence_refs.clone();
    let mut evidence_paths = BTreeSet::new();
    if evidence_refs.is_empty()
        || evidence_refs.iter().any(|reference| {
            reference.path.trim().is_empty()
                || reference.path.trim() != reference.path
                || !valid_digest(&reference.digest)
                || !evidence_paths.insert(reference.path.clone())
        })
    {
        return Err(MaterialReviewDecisionValidationError::InvalidEvidenceReferences);
    }
    evidence_refs.sort_by(|left, right| left.path.cmp(&right.path));
    if recorded_by.trim().is_empty()
        || recorded_by.trim() != recorded_by
        || !valid_utc_runtime_timestamp(recorded_at)
    {
        return Err(MaterialReviewDecisionValidationError::InvalidRuntimeProvenance);
    }

    let mut receipt = MaterialInspectionReviewDecisionReceipt {
        schema_version: MATERIAL_INSPECTION_REVIEW_DECISION_SCHEMA_VERSION,
        repository_id: request.repository_id.clone(),
        work_item_id: request.work_item_id.clone(),
        contract_digest: request.contract_digest.clone(),
        material_manifest_digest: request.material_manifest_digest.clone(),
        profile_digest,
        request_digest: request.request_digest.clone(),
        reviewed_source_head: Some(request.reviewed_source_head.clone()),
        decision: MaterialInspectionReviewDecision::AcceptPermittedUnknowns,
        reviewer_actor: input.reviewer_actor.clone(),
        recorded_by: recorded_by.to_owned(),
        authority_source: input.authority_source.clone(),
        assurance: input.assurance,
        evidence_refs,
        rationale: input.rationale.clone(),
        residual_risk: input.residual_risk.clone(),
        recorded_at: recorded_at.to_owned(),
        receipt_digest: Digest::sha256_bytes(b"uncomputed"),
    };
    receipt.receipt_digest = receipt
        .canonical_digest()
        .map_err(MaterialReviewDecisionValidationError::ReceiptIntegrity)?;
    receipt
        .validate_integrity()
        .map_err(MaterialReviewDecisionValidationError::ReceiptIntegrity)?;
    Ok(receipt)
}

fn policy_from_current(policy: &cockpit_protocol::GovernancePolicy) -> PolicyV1 {
    use cockpit_protocol::{ApprovalMode, EvidenceAssurance, PolicyLayer, VerificationTier};
    let cockpit_protocol::GovernancePolicy {
        policy_id,
        layer,
        rules,
    } = policy;
    let layer = match layer {
        PolicyLayer::Organization => PolicyLayerV1::Organization,
        PolicyLayer::Project => PolicyLayerV1::Project,
        PolicyLayer::WorkItem => PolicyLayerV1::WorkItem,
    };
    let rules = rules
        .iter()
        .map(|rule| {
            let cockpit_protocol::PolicyRule {
                operation,
                approval_mode,
                required_evidence,
                verification_requirement,
            } = rule;
            let approval_mode = match approval_mode {
                ApprovalMode::NoHumanApprovalForLowRisk => {
                    ApprovalModeV1::NoHumanApprovalForLowRisk
                }
                ApprovalMode::SingleAuthorizedHuman => ApprovalModeV1::SingleAuthorizedHuman,
                ApprovalMode::MultiPartyApproval => ApprovalModeV1::MultiPartyApproval,
                ApprovalMode::ExternalProviderApproval => ApprovalModeV1::ExternalProviderApproval,
            };
            let verification_requirement = verification_requirement.as_ref().map(|requirement| {
                let cockpit_protocol::VerificationRequirement {
                    schema_version,
                    required_tier,
                    required_assurance,
                    policy_refs,
                    stage_refs,
                    gate_refs,
                    reason,
                    max_timeout_seconds,
                } = requirement;
                VerificationRequirementV1 {
                    schema_version: *schema_version,
                    required_tier: match required_tier {
                        VerificationTier::T0 => VerificationTierV1::T0,
                        VerificationTier::T1 => VerificationTierV1::T1,
                        VerificationTier::T2 => VerificationTierV1::T2,
                        VerificationTier::T3 => VerificationTierV1::T3,
                    },
                    required_assurance: match required_assurance {
                        EvidenceAssurance::SelfDeclared => PolicyAssuranceV1::SelfDeclared,
                        EvidenceAssurance::RepositoryVerified => {
                            PolicyAssuranceV1::RepositoryVerified
                        }
                        EvidenceAssurance::ProviderVerified => PolicyAssuranceV1::ProviderVerified,
                        EvidenceAssurance::EnterpriseVerified => {
                            PolicyAssuranceV1::EnterpriseVerified
                        }
                    },
                    policy_refs: policy_refs.clone(),
                    stage_refs: stage_refs.clone(),
                    gate_refs: gate_refs.clone(),
                    reason: reason.clone(),
                    max_timeout_seconds: *max_timeout_seconds,
                }
            });
            PolicyRuleV1 {
                operation: operation.clone(),
                approval_mode,
                required_evidence: required_evidence.clone(),
                verification_requirement,
            }
        })
        .collect();
    PolicyV1 {
        policy_id: policy_id.clone(),
        layer,
        rules,
    }
}
pub(super) fn build_request_v1_from_current(
    root: &Path,
    value: &cockpit_protocol::Contract,
    digest: &Digest,
    implementation: &Digest,
) -> Result<MaterialReviewRequest, MaterialReviewRequestError> {
    validate_compiled_dependency_profile().map_err(MaterialReviewRequestError::Identity)?;
    let contract = ContractV1 {
        repository_id: value.repository_id.clone(),
        work_item_id: value.work_item_id.clone(),
        base_revision: value.base_revision.clone(),
        governance_policy: value.governance_policy.as_ref().map(policy_from_current),
        governance_profile: value.governance_profile.clone(),
        required_runtime_capabilities: value.required_runtime_capabilities.clone(),
    };
    contract
        .material_inspection_review_profile()
        .map_err(MaterialReviewRequestError::Identity)?;
    build_request_v1(root, &contract, digest, implementation)
}

pub(super) fn build_request_v1_from_bytes(
    root: &Path,
    bytes: &[u8],
    implementation: &Digest,
) -> Result<MaterialReviewRequest, MaterialReviewRequestError> {
    validate_compiled_dependency_profile().map_err(MaterialReviewRequestError::Identity)?;
    let (contract, digest) =
        contract_from_bytes(bytes).map_err(MaterialReviewRequestError::Identity)?;
    build_request_v1(root, &contract, &digest, implementation)
}
pub(super) fn validate_legacy_decision_v1(
    bytes: &[u8],
    request: &MaterialReviewRequest,
    receipt: &MaterialInspectionReviewDecisionReceipt,
) -> Result<(), MaterialReviewDecisionValidationError> {
    let (contract, digest) = contract_from_bytes(bytes)
        .map_err(MaterialReviewDecisionValidationError::InvalidProfile)?;
    let input = DecisionInputV1 {
        schema_version: 1,
        decision: receipt.decision,
        request_digest: receipt.request_digest.clone(),
        reviewer_actor: receipt.reviewer_actor.clone(),
        authority_source: receipt.authority_source.clone(),
        assurance: receipt.assurance,
        evidence_refs: receipt.evidence_refs.clone(),
        rationale: receipt.rationale.clone(),
        residual_risk: receipt.residual_risk.clone(),
    };
    let expected = validate_decision_v1(
        &contract,
        &digest,
        request,
        &input,
        &receipt.recorded_by,
        &receipt.recorded_at,
    )?;
    if expected != *receipt {
        return Err(MaterialReviewDecisionValidationError::ReceiptIntegrity(
            "original V1 receipt fields do not match fixed validation".into(),
        ));
    }
    Ok(())
}
// Limited bridge for the audited, pre-manifest V1 generator. Identity is
// reconstructed from raw Git objects; it is never a consumer hash exception.
pub(super) fn rebuild_historical_request_v1(
    root: &Path,
    contract_bytes: &[u8],
    reviewed_source_head: &str,
) -> Result<MaterialReviewRequest, MaterialReviewRequestError> {
    let git = GitRepository::discover(root)
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    let working = git
        .material_source_snapshot_v1_bounded(MAX_PATCH_BYTES)
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?;
    let head = working.head.as_deref().ok_or_else(|| {
        MaterialReviewRequestError::Identity("historical material requires committed HEAD".into())
    })?;
    if !git
        .material_v1_is_ancestor_bounded(reviewed_source_head, head, MAX_PATCH_BYTES)
        .map_err(|error| MaterialReviewRequestError::Git(error.to_string()))?
    {
        return Err(MaterialReviewRequestError::Identity(
            "historical review source is not an ancestor".into(),
        ));
    }
    let reviewed = committed_blob_ids(&git, reviewed_source_head)?;
    let current = committed_blob_ids(&git, head)?;
    for path in IMPLEMENTATION_PATHS_V1 {
        let original = reviewed.get(*path).ok_or_else(|| {
            MaterialReviewRequestError::Identity(format!("legacy generator source missing: {path}"))
        })?;
        if current.get(*path) != Some(original) {
            return Err(MaterialReviewRequestError::Identity(format!(
                "immutable historical generator source changed: {path}"
            )));
        }
    }
    let implementation = historical_implementation_digest_v1(root, reviewed_source_head)?;
    codec::select_legacy_generator(&implementation)
        .map_err(MaterialReviewRequestError::Identity)?;
    let mut request = build_request_v1_from_bytes(root, contract_bytes, &implementation)?;
    if request.reviewed_source_head != head {
        return Err(MaterialReviewRequestError::Identity(
            "source changed during historical proof".into(),
        ));
    }
    // All material and generator continuity checks precede provenance rebinding.
    // V1 excluded this exact field from its semantic request identity.
    request.reviewed_source_head = reviewed_source_head.to_owned();
    Ok(request)
}

const IMPLEMENTATION_PATHS_V1: &[&str] = &[
    "Cargo.lock",
    "crates/cockpit-core/src/lib.rs",
    "crates/cockpit-git/src/lib.rs",
    "crates/cockpit-protocol/src/lib.rs",
    "crates/cockpit-protocol/src/contract_amendment.rs",
    "crates/cockpit-repository/Cargo.toml",
    "crates/cockpit-repository/src/governance_controls.rs",
    "crates/cockpit-repository/src/lib.rs",
    "crates/cockpit-repository/src/material_review.rs",
    "crates/cockpit-repository/src/rust_material.rs",
    "crates/cockpit-repository/src/status_projection.rs",
];
pub(super) fn historical_implementation_digest_v1(
    root: &Path,
    revision: &str,
) -> Result<Digest, MaterialReviewRequestError> {
    let git = GitRepository::discover(root)
        .map_err(|e| MaterialReviewRequestError::Git(e.to_string()))?;
    let objects = committed_blob_ids(&git, revision)?;
    let mut sources = Vec::new();
    for path in IMPLEMENTATION_PATHS_V1 {
        let id = objects.get(*path).ok_or_else(|| {
            MaterialReviewRequestError::Identity(format!("legacy generator source missing: {path}"))
        })?;
        let bytes = committed_blob(&git, id, MAX_MATERIAL_BLOB_BYTES as usize, false)?;
        sources.push((*path, bytes));
    }
    let borrowed = sources
        .iter()
        .map(|(path, bytes)| (*path, bytes.as_slice()))
        .collect::<Vec<_>>();
    Ok(codec::implementation_digest_v1(&borrowed))
}
const LOCKED_DEPENDENCIES_V1: &[(&str, &str, &str)] = &[
    (
        "autocfg",
        "1.5.1",
        "f2032f911046de80f0a198e0901378627c33f59ea0ac00e363d481118bd70a53",
    ),
    (
        "block-buffer",
        "0.10.4",
        "3078c7629b62d3f0439517fa394996acacc5cbc91c5a20d8c658e77abd503a71",
    ),
    (
        "cfg-if",
        "1.0.4",
        "9330f8b2ff13f34540b44e946ef35111825727b38d33286ef986142615121801",
    ),
    (
        "chrono",
        "0.4.45",
        "1aa79e62e7697b8e29b513a68abacf485adcd1fe8284a4316c5ae868e6633327",
    ),
    (
        "cpufeatures",
        "0.2.17",
        "59ed5838eebb26a2bb2e58f6d5b5316989ae9d08bab10e0e6d103e656d1b0280",
    ),
    (
        "crypto-common",
        "0.1.7",
        "78c8292055d1c1df0cce5d180393dc8cce0abec0a7102adb6c7b1eef6016d60a",
    ),
    (
        "digest",
        "0.10.7",
        "9ed9a281f7bc9b7576e61468ba615a66a5c8cfdff42420a70aa82701a3b1e292",
    ),
    (
        "generic-array",
        "0.14.7",
        "85649ca51fd72272d7821adaf274ad91c288277713d9c18820d8499a7ff69e9a",
    ),
    (
        "hex",
        "0.4.3",
        "7f24254aa9a54b5c858eaee2f5bccdb46aaf0e486a595ed5fd8f86ba55232a70",
    ),
    (
        "itoa",
        "1.0.18",
        "8f42a60cbdf9a97f5d2305f08a87dc4e09308d1276d28c869c684d7777685682",
    ),
    (
        "memchr",
        "2.8.3",
        "cf8baf1c55e62ffcace7a9f06f4bd9cd3f0c4beb022d3b367256b91b87513d98",
    ),
    (
        "num-traits",
        "0.2.19",
        "071dfc062690e90b734c0b2273ce72ad0ffa95f0c74596bc250dcfd960262841",
    ),
    (
        "proc-macro2",
        "1.0.107",
        "985e7ec9bb745e6ce6535b544d84d6cd6f7ad8bd711c398938ae983b91a766d9",
    ),
    (
        "quote",
        "1.0.47",
        "1fbf4db142a473a8d80c26bbf18454ed458bf8d26c8219c331daecfdbd079001",
    ),
    (
        "serde",
        "1.0.229",
        "4148590afebada386688f18773da617792bf2ef03ffc1e4cbd2b1d45b023e0ba",
    ),
    (
        "serde_core",
        "1.0.229",
        "67dca2c9c51e58a4791a4b1ed58308b39c64224d349a935ab5039aa360942a48",
    ),
    (
        "serde_derive",
        "1.0.229",
        "e7a5d71263a5a7d47b41f6b3f06ba276f10cc18b0931f1799f710578e2309348",
    ),
    (
        "serde_json",
        "1.0.151",
        "c841b55ecdae098c80dcae9cf767f6f8a0c2cdb3416bbef72181df4d0fe73f14",
    ),
    (
        "sha2",
        "0.10.9",
        "a7507d819769d01a365ab707794a4084392c824f54a7a6a7862f8c3d0892b283",
    ),
    (
        "syn",
        "2.0.119",
        "872831b642d1a07999a962a351ed35b955ea2cfc8f3862091e2a240a84f17297",
    ),
    (
        "syn",
        "3.0.3",
        "53e9bae58849f64dfa4f5d5ae372c8341f7305f82a3868709269343628b659a3",
    ),
    (
        "typenum",
        "1.20.1",
        "b6f5e870be6c3b371b77fe0ee0bafb859fa4964b4404c27de1d380043c4dda20",
    ),
    (
        "unicode-ident",
        "1.0.24",
        "e6e4313cd5fcd3dad5cafa179702e2b244f760991f45397d14d4ebf38247da75",
    ),
    (
        "version_check",
        "0.9.5",
        "0b928f33d975fc6ad9f86c8f283853ad26bdd5b10b7f1542aa2fa15e2289105a",
    ),
    (
        "zmij",
        "1.0.23",
        "29666d0abbfad1e3dc4dcf6144730dd3a3ab225bbbdac83319345b1b44ccfc1b",
    ),
];

pub(super) fn validate_compiled_dependency_profile() -> Result<(), String> {
    let lock: toml::Value =
        toml::from_str(include_str!("../../../Cargo.lock")).map_err(|e| e.to_string())?;
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or("lock packages unavailable")?;
    for (name, version, checksum) in LOCKED_DEPENDENCIES_V1 {
        if !packages.iter().any(|package| {
            package.get("name").and_then(toml::Value::as_str) == Some(*name)
                && package.get("version").and_then(toml::Value::as_str) == Some(*version)
                && package.get("checksum").and_then(toml::Value::as_str) == Some(*checksum)
        }) {
            return Err(format!(
                "unsupported V1 dependency profile: {name}@{version}"
            ));
        }
    }
    Ok(())
}
#[cfg(test)]
pub(super) fn validate_resolved_features(metadata: &serde_json::Value) -> Result<(), String> {
    let expected: &[(&str, &str, &[&str])] = &[
        ("serde_json", "1.0.151", &["default", "std"]),
        (
            "syn",
            "2.0.119",
            &[
                "clone-impls",
                "default",
                "derive",
                "extra-traits",
                "full",
                "parsing",
                "printing",
                "proc-macro",
                "visit",
                "visit-mut",
            ],
        ),
        (
            "syn",
            "3.0.3",
            &[
                "clone-impls",
                "default",
                "derive",
                "full",
                "parsing",
                "printing",
                "proc-macro",
            ],
        ),
        (
            "proc-macro2",
            "1.0.107",
            &["default", "proc-macro", "span-locations"],
        ),
        (
            "serde",
            "1.0.229",
            &["default", "derive", "serde_derive", "std"],
        ),
    ];
    let packages = metadata["packages"]
        .as_array()
        .ok_or("resolved packages missing")?;
    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .ok_or("resolved features missing")?;
    for (name, version, features) in expected {
        let package = packages
            .iter()
            .find(|p| p["name"] == *name && p["version"] == *version)
            .ok_or_else(|| format!("resolved V1 package missing {name}@{version}"))?;
        let node = nodes
            .iter()
            .find(|node| node["id"] == package["id"])
            .ok_or("resolved node missing")?;
        let actual = node["features"]
            .as_array()
            .ok_or("feature array missing")?
            .iter()
            .map(|value| value.as_str().unwrap_or(""))
            .collect::<BTreeSet<_>>();
        let expected = features.iter().copied().collect::<BTreeSet<_>>();
        if actual != expected {
            return Err(format!(
                "unsupported resolved V1 features for {name}@{version}: {actual:?}"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cockpit_core::Digest;
    use serde_json::{Value, json};
    use std::{fs, path::Path, process::Command};

    const REVIEWED: &str = "84fb04ee43e33842865a1544a5588466174c0639";
    const GENERATOR: &str =
        "sha256:8b5a0b834a01d7e2b88d08a8d1de3f1f347ce22b0ab2f90745cb476eb3e71d7c";
    const CONTRACT: &str =
        ".ai/work-items/active/WI-1078-v1-0-1-post-merge-sync-finalization.contract.json";
    const WI1090_REVIEWED: &str = "6048fdcc2a8acb3da373ab496cc56efb5f3732ef";
    const WI1090_ARCHIVE: &str = "262cd5a269e4069d588207ab6acac0f9a99c44e8";
    const WI1090_GENERATOR: &str =
        "sha256:ccf952067cbe6eba3dfa116f7a3d87293d69f6c7ddb5747953157ff63493cc83";
    const WI1090_CONTRACT: &str =
        ".ai/work-items/active/WI-1090-evidence-docs-recovery.contract.json";
    const WI1090_RECEIPT: &str = ".ai/evidence/material-inspection-review/WI-1090-evidence-docs-recovery/f98cf3c3e5146c83cf10afd3c1046c9753fa240dd094d592235059fb5e8f4b6a.json";

    fn historical_fixture() -> (tempfile::TempDir, std::path::PathBuf) {
        historical_fixture_at(REVIEWED)
    }

    fn historical_fixture_at(revision: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let directory = tempfile::tempdir().unwrap();
        let repository = directory.path().join("checkout");
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let output = Command::new("git")
            .args(["clone", "--shared", "--no-checkout", "-q"])
            .arg(source)
            .arg(&repository)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let output = Command::new("git")
            .arg("-C")
            .arg(&repository)
            .args(["checkout", "--detach", "-q", revision])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        (directory, repository)
    }

    fn historical_blob(root: &Path, revision: &str, path: &str) -> Vec<u8> {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["show", &format!("{revision}:{path}")])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output.stdout
    }

    #[test]
    fn real_1090_ccf_historical_bridge_preserves_original_request_receipt_and_provenance() {
        let (_directory, root) = historical_fixture_at(WI1090_REVIEWED);
        let raw = fs::read(root.join(WI1090_CONTRACT)).unwrap();
        assert_eq!(
            Digest::sha256_bytes(&raw).as_str(),
            "sha256:e1d0ad6f5ff811b3b8195b477b08e0facbee9ba712ac346a24fd1a1be6160034"
        );
        let original_bytes = historical_blob(&root, WI1090_ARCHIVE, WI1090_RECEIPT);
        let original_raw_digest = Digest::sha256_bytes(&original_bytes);
        assert_eq!(
            original_raw_digest.as_str(),
            "sha256:4d65cb70233799323a332c25056fa85b41d0372534b1f8effcc151eb95ddb727"
        );
        let summary: Value = serde_json::from_slice(&historical_blob(
            &root,
            WI1090_ARCHIVE,
            ".ai/work-items/archive/WI-1090-evidence-docs-recovery.summary.json",
        ))
        .unwrap();
        assert_eq!(summary["materialReviewReceipt"]["path"], WI1090_RECEIPT);
        assert_eq!(
            summary["materialReviewReceipt"]["digest"],
            json!(original_raw_digest)
        );
        let original = codec::decode_receipt_v1(&original_bytes).unwrap();
        assert_eq!(
            original.receipt_digest.as_str(),
            "sha256:1c6b5fe99cb8d242be86d07b030ed92044df6b95edbc5024087e17457f16c639"
        );
        assert_eq!(
            original.reviewed_source_head.as_deref(),
            Some(WI1090_REVIEWED)
        );
        assert_eq!(
            original.assurance,
            MaterialInspectionReviewAssurance::SelfDeclared
        );
        // Original audit claims are retained as self-declared historical facts;
        // this codec test does not authenticate a human or grant CURRENT work.
        assert_eq!(original.reviewer_actor, "agent:Codex");
        assert_eq!(
            original.recorded_by,
            "runtime:1.0.1:sha256:e967d02a49806d0481272625bdc9813dc913a7aca1773aa734d4a095367b0a98"
        );
        assert_eq!(original.recorded_at, "2026-10-20T03:38:47Z");
        let implementation = historical_implementation_digest_v1(&root, WI1090_REVIEWED).unwrap();
        assert_eq!(implementation.as_str(), WI1090_GENERATOR);
        // Exercise the frozen kernel before reaching generator selection. Its
        // complete identity must reproduce the actual receipt, not a new hash.
        let direct = build_request_v1_from_bytes(&root, &raw, &implementation).unwrap();
        assert_eq!(direct.analysis_implementation_digest, implementation);
        assert_eq!(direct.reviewed_source_head, WI1090_REVIEWED);
        assert_eq!(direct.repository_id, original.repository_id);
        assert_eq!(direct.work_item_id, "WI-1090-evidence-docs-recovery");
        assert_eq!(direct.contract_digest, original.contract_digest);
        assert_eq!(
            direct.material_manifest_digest.as_str(),
            "sha256:76009df3df2d11067ad3e78261ba55899349dce0046866e0723da893f724525b"
        );
        assert_eq!(
            direct.material_inspection_review_profile_digest.as_ref(),
            Some(&original.profile_digest)
        );
        assert_eq!(
            direct.request_digest.as_str(),
            "sha256:f98cf3c3e5146c83cf10afd3c1046c9753fa240dd094d592235059fb5e8f4b6a"
        );
        assert_eq!(
            codec::request_digest_v1(&direct).unwrap(),
            direct.request_digest
        );
        validate_legacy_decision_v1(&raw, &direct, &original).unwrap();
        let output = Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(["checkout", "--detach", "-q", WI1090_ARCHIVE])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            historical_implementation_digest_v1(&root, WI1090_ARCHIVE).unwrap(),
            implementation
        );
        let rebuilt = rebuild_historical_request_v1(&root, &raw, WI1090_REVIEWED)
            .expect("the audited original ccf generator must select the exact frozen V1 recipe");
        assert_eq!(rebuilt.analysis_implementation_digest, implementation);
        assert_eq!(rebuilt.reviewed_source_head, WI1090_REVIEWED);
        assert_eq!(rebuilt.request_digest, direct.request_digest);
        assert_eq!(
            codec::request_digest_v1(&rebuilt).unwrap(),
            rebuilt.request_digest
        );
        validate_legacy_decision_v1(&raw, &rebuilt, &original).unwrap();
        assert_eq!(fs::read(root.join(WI1090_RECEIPT)).unwrap(), original_bytes);
        assert_eq!(
            summary["materialReviewReceipt"]["receiptDigest"],
            json!(original.receipt_digest)
        );
    }

    #[test]
    fn real_1090_ccf_historical_bridge_keeps_generator_and_foreign_history_guards() {
        let (_directory, root) = historical_fixture_at(WI1090_REVIEWED);
        let raw = fs::read(root.join(WI1090_CONTRACT)).unwrap();
        assert!(rebuild_historical_request_v1(&root, &raw, &"0".repeat(40)).is_err());
        let path = root.join("crates/cockpit-core/src/lib.rs");
        let mut bytes = fs::read(&path).unwrap();
        bytes.extend_from_slice(b"\n// immutable original ccf generator input changed\n");
        fs::write(&path, bytes).unwrap();
        let output = Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(["add", "crates/cockpit-core/src/lib.rs"])
            .output()
            .unwrap();
        assert!(output.status.success());
        let output = Command::new("git")
            .arg("-C")
            .arg(&root)
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-qm",
                "change immutable original ccf generator input",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let error = rebuild_historical_request_v1(&root, &raw, WI1090_REVIEWED)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("immutable historical generator source changed"),
            "{error}"
        );
    }

    #[test]
    fn real_1078_full_v1_request_rebuild_matches_original_contract_and_receipt() {
        let (_directory, root) = historical_fixture();
        let implementation = historical_implementation_digest_v1(&root, REVIEWED).unwrap();
        assert_eq!(implementation.as_str(), GENERATOR);
        crate::material_recipe::select_legacy_generator(&implementation).unwrap();
        let raw = fs::read(root.join(CONTRACT)).unwrap();
        let request = build_request_v1_from_bytes(&root, &raw, &implementation).unwrap();
        assert_eq!(
            request.contract_digest.as_str(),
            "sha256:e88235839caaea350e71070a0f161aaa6117680161d8fde2ae007a3686c9e63a"
        );
        assert_eq!(
            request.request_digest.as_str(),
            "sha256:b7816e7a21171070ff1503b8dd6d4f369259420828a0db415ee4a1a418197774"
        );
        let vectors: Value = serde_json::from_str(include_str!(
            "../tests/fixtures/material_review_versions.json"
        ))
        .unwrap();
        let receipt = crate::material_recipe::decode_receipt_v1(
            &serde_json::to_vec(&vectors["legacyV1"][0]["receipt"]).unwrap(),
        )
        .unwrap();
        validate_legacy_decision_v1(&raw, &request, &receipt).unwrap();
    }

    #[test]
    fn v1_rejects_unknown_contract_and_profile_semantics() {
        let (_directory, root) = historical_fixture();
        let raw = fs::read(root.join(CONTRACT)).unwrap();
        let implementation: Digest = GENERATOR.parse().unwrap();
        let mut contract: Value = serde_json::from_slice(&raw).unwrap();
        contract["unrecognizedAuthoritySemantics"] = json!(true);
        assert!(
            build_request_v1_from_bytes(
                &root,
                &serde_json::to_vec(&contract).unwrap(),
                &implementation
            )
            .is_err()
        );
        contract
            .as_object_mut()
            .unwrap()
            .remove("unrecognizedAuthoritySemantics");
        contract["governanceProfile"]["materialInspectionReview"]["unrecognizedScope"] =
            json!(true);
        assert!(
            build_request_v1_from_bytes(
                &root,
                &serde_json::to_vec(&contract).unwrap(),
                &implementation
            )
            .is_err()
        );
    }

    #[test]
    fn historical_bridge_preserves_real_pre_finish_and_archive_requests() {
        let (_directory, root) = historical_fixture();
        let raw = fs::read(root.join(CONTRACT)).unwrap();
        for revision in [
            "ba4350249313b2ffd95c5bae52a3ffff8e9597a9",
            "d18be4f8624d08f91fe1c961dfed287cfe9543df",
        ] {
            let status = Command::new("git")
                .arg("-C")
                .arg(&root)
                .args(["checkout", "--detach", "-q", revision])
                .status()
                .unwrap();
            assert!(status.success());
            let request = rebuild_historical_request_v1(&root, &raw, REVIEWED).unwrap();
            assert_eq!(request.reviewed_source_head, REVIEWED);
            assert_eq!(
                request.request_digest.as_str(),
                "sha256:b7816e7a21171070ff1503b8dd6d4f369259420828a0db415ee4a1a418197774"
            );
            assert_eq!(
                codec::request_digest_v1(&request).unwrap(),
                request.request_digest
            );
        }
    }

    #[test]
    fn historical_bridge_rejects_changed_generator_input_and_foreign_history() {
        let (_directory, root) = historical_fixture();
        let raw = fs::read(root.join(CONTRACT)).unwrap();
        assert!(rebuild_historical_request_v1(&root, &raw, &"0".repeat(40)).is_err());
        let path = root.join("crates/cockpit-core/src/lib.rs");
        let mut bytes = fs::read(&path).unwrap();
        bytes.extend_from_slice(b"\n// generator input changed\n");
        fs::write(&path, bytes).unwrap();
        let status = Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(["add", "crates/cockpit-core/src/lib.rs"])
            .status()
            .unwrap();
        assert!(status.success());
        let status = Command::new("git")
            .arg("-C")
            .arg(&root)
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-qm",
                "change immutable generator input",
            ])
            .status()
            .unwrap();
        assert!(status.success());
        let error = rebuild_historical_request_v1(&root, &raw, REVIEWED)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("immutable historical generator source changed"),
            "{error}"
        );
    }

    #[test]
    fn parser_dependency_profile_checks_actual_resolved_features_and_checksums() {
        validate_compiled_dependency_profile().unwrap();
        let rustc = Command::new("rustc").arg("-vV").output().unwrap();
        assert!(rustc.status.success(), "rustc host query failed");
        let version = String::from_utf8(rustc.stdout).unwrap();
        let host = version
            .lines()
            .find_map(|line| line.strip_prefix("host: "))
            .unwrap();
        let output = Command::new("cargo")
            .args([
                "metadata",
                "--locked",
                "--offline",
                "--all-features",
                "--filter-platform",
                host,
                "--format-version",
                "1",
                "--manifest-path",
            ])
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut metadata: Value = serde_json::from_slice(&output.stdout).unwrap();
        validate_resolved_features(&metadata).unwrap();
        let package = metadata["packages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|package| package["name"] == "serde_json")
            .unwrap()["id"]
            .clone();
        let node = metadata["resolve"]["nodes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|node| node["id"] == package)
            .unwrap();
        node["features"]
            .as_array_mut()
            .unwrap()
            .push(json!("preserve_order"));
        assert!(validate_resolved_features(&metadata).is_err());
    }
}

//! Validation for Contract/Summary governance projections.
//!
//! The reference template owns the business scenarios and final acceptance
//! claims.  This module only validates their shape, identity and evidence
//! bindings.  It deliberately does not generate scenarios, acceptance
//! decisions, or final-dimension evidence.

use crate::{ObserverError, repository_id};
use cockpit_core::Digest;
use cockpit_protocol::{Contract, PreflightDecisionEvidence, RuntimeContext};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

/// The exact dimension names used by the reference final acceptance oracle.
pub const FINAL_DIMENSIONS: [&str; 20] = [
    "installation",
    "upgrade",
    "uninstall",
    "lifecycle",
    "absurd_tests",
    "injection",
    "unknown",
    "agent_self_assertion",
    "real_adopter",
    "provider_evidence",
    "enterprise_boundary",
    "task_outcome",
    "documentation",
    "multilingual",
    "performance",
    "code_quality",
    "stale_assets",
    "recovery",
    "capability_truth",
    "north_star",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceFinding {
    pub code: String,
    pub message: String,
    pub severity: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceControlsReport {
    pub state: String,
    pub scenario_coverage: String,
    pub acceptance_evidence: String,
    pub intent_alignment: String,
    pub final_dimensions: String,
    pub unknowns: Vec<String>,
    pub findings: Vec<GovernanceFinding>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FinalDimensionItem {
    pub status: String,
    pub evidence: Vec<String>,
    #[serde(default)]
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FinalDimensionsReceipt {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub contract_digest: Digest,
    pub summary_digest: Digest,
    pub runtime_version: String,
    pub runtime_digest: Digest,
    pub decision: String,
    pub dimensions: BTreeMap<String, FinalDimensionItem>,
    #[serde(default)]
    pub limitations: Vec<String>,
    #[serde(default)]
    pub four_pillar_projection: Option<BTreeMap<String, Vec<String>>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalDimensionsReport {
    pub state: String,
    pub decision: Option<String>,
    pub unknowns: Vec<String>,
    pub findings: Vec<GovernanceFinding>,
}

fn finding(code: &str, message: impl Into<String>, severity: &str) -> GovernanceFinding {
    GovernanceFinding {
        code: code.into(),
        message: message.into(),
        severity: severity.into(),
    }
}

fn high_risk(contract: &Value) -> bool {
    let risk = contract
        .get("risk")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let operation = contract
        .get("operation")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    risk.contains("high")
        || risk.contains("destructive")
        || [
            "release",
            "release_distribution",
            "installer",
            "auth",
            "ci",
            "migration",
            "security",
            "api_change",
        ]
        .iter()
        .any(|candidate| operation == *candidate)
}

fn nonempty_string(value: Option<&Value>) -> bool {
    value
        .and_then(Value::as_str)
        .is_some_and(|s| !s.trim().is_empty())
}

fn array(value: Option<&Value>) -> Option<&Vec<Value>> {
    value.and_then(Value::as_array)
}

fn regular_file(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
        .unwrap_or(false)
}

/// Validate Contract/Summary scenario coverage.  A high-risk Contract must
/// declare coverage and provide a Summary projection; missing or unverified
/// required scenarios are a hard failure.  Normal-risk legacy Contracts may
/// omit the optional projection.
pub fn validate_scenario_coverage_values(
    contract: &Value,
    summary: &Value,
) -> (String, Vec<String>, Vec<GovernanceFinding>) {
    let high_risk = high_risk(contract);
    let Some(contract_entries) = array(contract.get("scenarioCoverage")) else {
        if high_risk {
            return (
                "blocked".into(),
                vec!["scenario_coverage_required_for_high_risk".into()],
                vec![finding(
                    "scenario_coverage_required_for_high_risk",
                    "high-risk Contract has no scenarioCoverage declaration",
                    "error",
                )],
            );
        }
        return ("not_applicable".into(), Vec::new(), Vec::new());
    };
    if contract_entries.is_empty() {
        return (
            "blocked".into(),
            vec!["scenario_coverage_empty".into()],
            vec![finding(
                "scenario_coverage_empty",
                "scenarioCoverage must contain an explicit scenario",
                "error",
            )],
        );
    }
    let entries = if let Some(summary_entries) = array(summary.get("scenarioCoverage")) {
        let contract_names: BTreeSet<&str> = contract_entries
            .iter()
            .filter_map(|entry| entry.get("scenario").and_then(Value::as_str))
            .collect();
        let summary_names: BTreeSet<&str> = summary_entries
            .iter()
            .filter_map(|entry| entry.get("scenario").and_then(Value::as_str))
            .collect();
        if contract_names != summary_names {
            return (
                "blocked".into(),
                vec!["scenario_contract_summary_mismatch".into()],
                vec![finding(
                    "scenario_contract_summary_mismatch",
                    "Contract and Summary scenarioCoverage names do not match",
                    "error",
                )],
            );
        }
        summary_entries
    } else if high_risk {
        return (
            "blocked".into(),
            vec!["scenario_coverage_summary_missing".into()],
            vec![finding(
                "scenario_coverage_summary_missing",
                "high-risk scenario coverage has no Summary evidence projection",
                "error",
            )],
        );
    } else {
        contract_entries
    };
    let mut names = BTreeSet::new();
    let mut unknowns = Vec::new();
    let mut findings = Vec::new();
    let mut blocked = false;
    for (index, entry) in entries.iter().enumerate() {
        let name = entry
            .get("scenario")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if name.trim().is_empty() {
            blocked = true;
            findings.push(finding(
                "scenario_name_missing",
                format!("scenarioCoverage[{index}] has no scenario name"),
                "error",
            ));
            continue;
        }
        if !names.insert(name.to_string()) {
            blocked = true;
            findings.push(finding(
                "scenario_duplicate",
                format!("scenarioCoverage contains duplicate scenario {name}"),
                "error",
            ));
        }
        let required = entry
            .get("required")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if entry.get("required").and_then(Value::as_bool).is_none() {
            blocked = true;
            findings.push(finding(
                "scenario_required_invalid",
                format!("scenario {name} must declare required as boolean"),
                "error",
            ));
        }
        let status = entry
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if !matches!(status, "verified" | "unverified" | "not_applicable") {
            blocked = true;
            findings.push(finding(
                "scenario_status_invalid",
                format!("scenario {name} has unsupported status"),
                "error",
            ));
            continue;
        }
        let has_evidence = array(entry.get("evidence")).is_some_and(|items| {
            !items.is_empty() && items.iter().all(|item| nonempty_string(Some(item)))
        });
        if status == "verified" && !has_evidence {
            blocked = true;
            findings.push(finding(
                "scenario_verified_without_evidence",
                format!("verified scenario {name} has no evidence"),
                "error",
            ));
        }
        if status == "not_applicable" && !nonempty_string(entry.get("reason")) {
            blocked = true;
            findings.push(finding(
                "scenario_not_applicable_without_reason",
                format!("not_applicable scenario {name} has no reason"),
                "error",
            ));
        }
        if required && status == "unverified" {
            unknowns.push(format!("required_scenario_unverified:{name}"));
            if high_risk {
                blocked = true;
                findings.push(finding(
                    "required_scenario_unverified",
                    format!("required scenario {name} is unverified"),
                    "error",
                ));
            }
        }
    }
    if blocked {
        ("blocked".into(), unknowns, findings)
    } else if unknowns.is_empty() {
        ("verified".into(), unknowns, findings)
    } else {
        ("unknown".into(), unknowns, findings)
    }
}

/// Preflight-only scenario unknowns. This deliberately inspects Contract
/// declarations without requiring the later Summary evidence projection, so
/// a high-risk item stops for human review before checkpoint/verification.
pub fn scenario_coverage_preflight_unknowns(contract: &Value) -> Vec<String> {
    if !high_risk(contract) {
        return Vec::new();
    }
    let Some(entries) = array(contract.get("scenarioCoverage")) else {
        return vec!["scenario_coverage_required_for_high_risk".into()];
    };
    if entries.is_empty() {
        return vec!["scenario_coverage_empty".into()];
    }
    let mut unknowns = Vec::new();
    for entry in entries {
        let name = entry
            .get("scenario")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("unknown");
        let status = entry
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or_default();
        match status {
            "unverified" if entry.get("required").and_then(Value::as_bool) == Some(true) => {
                // A required scenario that can only be executed after the
                // implementation may authorize implementation when its
                // expected result and concrete verification plan are both
                // declared. This is planning evidence only; the Summary
                // guard and finish gate still require executed evidence.
                let expected = entry
                    .get("expected")
                    .or_else(|| entry.get("expectedResult"));
                let planned =
                    nonempty_string(expected) && nonempty_string(entry.get("verificationPlan"));
                if !planned {
                    unknowns.push(format!("required_scenario_unverified:{name}"));
                }
            }
            "verified" => {
                if array(entry.get("evidence")).is_none_or(|items| items.is_empty()) {
                    unknowns.push(format!("scenario_coverage_evidence_missing:{name}"));
                }
            }
            "not_applicable" => {
                if !nonempty_string(entry.get("reason")) {
                    unknowns.push(format!("scenario_coverage_reason_missing:{name}"));
                }
            }
            "unverified" => {}
            _ => unknowns.push(format!("scenario_coverage_status_invalid:{name}")),
        }
    }
    unknowns
}

fn acceptance_ids(contract: &Contract) -> (Vec<String>, bool, Vec<GovernanceFinding>) {
    let mut ids = Vec::new();
    let mut numbered = false;
    let mut unnumbered = false;
    let mut findings = Vec::new();
    for criterion in &contract.acceptance_criteria {
        let Some((prefix, _)) = criterion.split_once(':') else {
            unnumbered = true;
            continue;
        };
        if let Some(suffix) = prefix.strip_prefix('A') {
            numbered = true;
            if suffix.is_empty() || !suffix.bytes().all(|b| b.is_ascii_digit()) {
                findings.push(finding(
                    "acceptance_id_invalid",
                    format!("invalid acceptance identifier {prefix}"),
                    "error",
                ));
            } else if !ids.insert_unique(prefix.to_string()) {
                findings.push(finding(
                    "acceptance_id_duplicate",
                    format!("duplicate acceptance identifier {prefix}"),
                    "error",
                ));
            }
        } else {
            unnumbered = true;
        }
    }
    if numbered && unnumbered {
        findings.push(finding(
            "acceptance_id_mixed",
            "numbered and unnumbered acceptance criteria cannot share one evidence mapping",
            "error",
        ));
    }
    (ids, numbered, findings)
}

trait InsertUnique {
    fn insert_unique(&mut self, value: String) -> bool;
}

impl InsertUnique for Vec<String> {
    fn insert_unique(&mut self, value: String) -> bool {
        if self.iter().any(|item| item == &value) {
            false
        } else {
            self.push(value);
            true
        }
    }
}

/// Validate stable acceptance IDs and Summary `acceptanceEvidence`.  Legacy
/// unnumbered criteria intentionally remain compatible and are reported as
/// `not_applicable` rather than being silently assigned IDs.
pub fn validate_acceptance_evidence_values(
    contract: &Contract,
    summary: &Value,
) -> (String, Vec<String>, Vec<GovernanceFinding>) {
    let (ids, numbered, mut findings) = acceptance_ids(contract);
    if !numbered {
        return ("not_applicable".into(), Vec::new(), findings);
    }
    let Some(items) = array(summary.get("acceptanceEvidence")) else {
        return (
            "unknown".into(),
            vec!["acceptance_evidence_missing".into()],
            vec![finding(
                "acceptance_evidence_missing",
                "numbered acceptance criteria have no acceptanceEvidence projection",
                "error",
            )],
        );
    };
    let mut seen = BTreeSet::new();
    for item in items {
        let id = item
            .get("acceptanceId")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if !ids.iter().any(|known| known == id) {
            findings.push(finding(
                "acceptance_id_unknown",
                format!("acceptanceEvidence references unknown id {id}"),
                "error",
            ));
            continue;
        }
        if !seen.insert(id.to_string()) {
            findings.push(finding(
                "acceptance_evidence_duplicate",
                format!("acceptanceEvidence repeats {id}"),
                "error",
            ));
        }
        let Some(evidence) = array(item.get("evidence")) else {
            findings.push(finding(
                "acceptance_evidence_missing_items",
                format!("acceptance {id} has no evidence items"),
                "error",
            ));
            continue;
        };
        if evidence.is_empty() {
            findings.push(finding(
                "acceptance_evidence_missing_items",
                format!("acceptance {id} has no evidence items"),
                "error",
            ));
        }
        for evidence_item in evidence {
            for key in ["type", "path", "locator", "verification"] {
                if !nonempty_string(evidence_item.get(key)) {
                    findings.push(finding(
                        "acceptance_evidence_field_missing",
                        format!("acceptance {id} evidence is missing {key}"),
                        "error",
                    ));
                }
            }
        }
    }
    for id in &ids {
        if !seen.contains(id) {
            findings.push(finding(
                "acceptance_evidence_item_missing",
                format!("acceptance {id} has no evidence mapping"),
                "error",
            ));
        }
    }
    if findings.iter().any(|item| item.severity == "error") {
        ("blocked".into(), Vec::new(), findings)
    } else {
        ("verified".into(), Vec::new(), findings)
    }
}

/// Validate the optional intent alignment projection.  Missing alignment is
/// visible as unknown when intent is declared, never promoted to resolved.
pub fn validate_intent_alignment_values(
    contract: &Contract,
    summary: &Value,
) -> (String, Vec<String>, Vec<GovernanceFinding>) {
    if contract.intent.is_empty() {
        return ("not_applicable".into(), Vec::new(), Vec::new());
    }
    let Some(alignment) = summary.get("intentAlignment") else {
        return (
            "unknown".into(),
            vec!["intent_alignment_missing".into()],
            vec![finding(
                "intent_alignment_missing",
                "declared intent has no alignment projection",
                "warning",
            )],
        );
    };
    let state = alignment
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !matches!(state, "resolved" | "unknown" | "unresolved") {
        return (
            "blocked".into(),
            vec!["intent_alignment_state_invalid".into()],
            vec![finding(
                "intent_alignment_state_invalid",
                "intentAlignment.state must be resolved, unknown, or unresolved",
                "error",
            )],
        );
    }
    let evidence = array(alignment.get("evidence")).is_some_and(|items| {
        !items.is_empty() && items.iter().all(|item| nonempty_string(Some(item)))
    });
    if state == "resolved" && !evidence {
        return (
            "blocked".into(),
            vec!["intent_alignment_evidence_missing".into()],
            vec![finding(
                "intent_alignment_evidence_missing",
                "resolved intent alignment requires evidence",
                "error",
            )],
        );
    }
    if state == "unresolved" && !nonempty_string(alignment.get("reason")) {
        return (
            "blocked".into(),
            vec!["intent_alignment_reason_missing".into()],
            vec![finding(
                "intent_alignment_reason_missing",
                "unresolved intent alignment requires a reason",
                "error",
            )],
        );
    }
    (state.into(), Vec::new(), Vec::new())
}

/// Validate a final-dimensions receipt against the exact reference dimension
/// set.  The receipt is immutable input: this function never fills missing
/// dimensions or changes a decision.
pub fn validate_final_dimensions_value(
    value: &Value,
    expected_repository_id: Option<&str>,
    expected_work_item_id: Option<&str>,
) -> FinalDimensionsReport {
    validate_final_dimensions_value_with_runtime(
        value,
        expected_repository_id,
        expected_work_item_id,
        None,
    )
}

pub fn validate_final_dimensions_value_with_runtime(
    value: &Value,
    expected_repository_id: Option<&str>,
    expected_work_item_id: Option<&str>,
    expected_runtime: Option<&RuntimeContext>,
) -> FinalDimensionsReport {
    let mut report = FinalDimensionsReport {
        state: "verified".into(),
        ..FinalDimensionsReport::default()
    };
    let receipt: FinalDimensionsReceipt = match serde_json::from_value(value.clone()) {
        Ok(receipt) => receipt,
        Err(error) => {
            report.state = "blocked".into();
            report.unknowns.push("final_dimensions_malformed".into());
            report.findings.push(finding(
                "final_dimensions_malformed",
                error.to_string(),
                "error",
            ));
            return report;
        }
    };
    report.decision = Some(receipt.decision.clone());
    if receipt.schema_version != 1 {
        report.state = "blocked".into();
        report.findings.push(finding(
            "final_dimensions_schema_invalid",
            "unsupported final dimensions schemaVersion",
            "error",
        ));
    }
    if expected_repository_id.is_some_and(|expected| expected != receipt.repository_id) {
        report.state = "blocked".into();
        report.findings.push(finding(
            "final_dimensions_repository_mismatch",
            "final dimensions repository identity does not match context",
            "error",
        ));
    }
    if expected_work_item_id.is_some_and(|expected| expected != receipt.work_item_id) {
        report.state = "blocked".into();
        report.findings.push(finding(
            "final_dimensions_work_item_mismatch",
            "final dimensions Work Item does not match context",
            "error",
        ));
    }
    if receipt.runtime_version.trim().is_empty() {
        report.state = "blocked".into();
        report.findings.push(finding(
            "final_runtime_version_missing",
            "final dimensions runtimeVersion must not be empty",
            "error",
        ));
    }
    if let Some(runtime) = expected_runtime {
        if receipt.runtime_version != runtime.runtime_version {
            report.state = "blocked".into();
            report.findings.push(finding(
                "final_runtime_version_mismatch",
                "final dimensions runtimeVersion does not match the current Runtime",
                "error",
            ));
        }
        if receipt.runtime_digest != runtime.runtime_digest {
            report.state = "blocked".into();
            report.findings.push(finding(
                "final_runtime_digest_mismatch",
                "final dimensions runtimeDigest does not match the current Runtime",
                "error",
            ));
        }
    }
    let expected: BTreeSet<String> = FINAL_DIMENSIONS.iter().map(|item| (*item).into()).collect();
    let actual: BTreeSet<String> = receipt.dimensions.keys().cloned().collect();
    for missing in expected.difference(&actual) {
        report.state = "blocked".into();
        report
            .unknowns
            .push(format!("final_dimension_missing:{missing}"));
        report.findings.push(finding(
            "final_dimension_missing",
            format!("missing final dimension {missing}"),
            "error",
        ));
    }
    for extra in actual.difference(&expected) {
        report.state = "blocked".into();
        report.findings.push(finding(
            "final_dimension_unknown",
            format!("unknown final dimension {extra}"),
            "error",
        ));
    }
    for (name, dimension) in &receipt.dimensions {
        if dimension.status.trim().is_empty() {
            report.state = "blocked".into();
            report.findings.push(finding(
                "final_dimension_status_missing",
                format!("dimension {name} has no status"),
                "error",
            ));
        }
        if dimension.status == "verified" && dimension.evidence.is_empty() {
            report.state = "blocked".into();
            report.findings.push(finding(
                "final_dimension_evidence_missing",
                format!("verified dimension {name} has no evidence"),
                "error",
            ));
        }
    }
    if !matches!(receipt.decision.as_str(), "GO" | "CONDITIONAL_GO" | "NO_GO") {
        report.state = "blocked".into();
        report.findings.push(finding(
            "final_decision_invalid",
            "final dimensions decision must be GO, CONDITIONAL_GO, or NO_GO",
            "error",
        ));
    }
    if receipt.decision == "GO" {
        for required in ["real_adopter", "provider_evidence"] {
            if receipt
                .dimensions
                .get(required)
                .is_none_or(|item| item.status != "verified")
            {
                report.state = "blocked".into();
                report.findings.push(finding(
                    "go_prerequisite_missing",
                    format!("GO requires verified {required}"),
                    "error",
                ));
            }
        }
    }
    if let Some(pillars) = receipt.four_pillar_projection
        && pillars.contains_key("4D")
    {
        report.state = "blocked".into();
        report.findings.push(finding(
            "ambiguous_four_dimension_field",
            "literal 4D field is not part of the protocol",
            "error",
        ));
    }
    report
}

/// Read a Work Item's active or archived Contract/Summary and produce the
/// unified validation report used by CLI/MCP integrations.
pub fn validate_work_item_governance_controls(
    root: &Path,
    work_item_id: &str,
) -> Result<GovernanceControlsReport, ObserverError> {
    validate_work_item_governance_controls_internal(root, work_item_id, None)
}

pub fn validate_work_item_governance_controls_with_runtime(
    root: &Path,
    work_item_id: &str,
    runtime: &RuntimeContext,
) -> Result<GovernanceControlsReport, ObserverError> {
    validate_work_item_governance_controls_internal(root, work_item_id, Some(runtime))
}

fn validate_work_item_governance_controls_internal(
    root: &Path,
    work_item_id: &str,
    runtime: Option<&RuntimeContext>,
) -> Result<GovernanceControlsReport, ObserverError> {
    let ai = root.join(".ai");
    let candidates = [
        ai.join(format!("work-items/active/{work_item_id}.contract.json")),
        ai.join(format!("work-items/archive/{work_item_id}.archive.json")),
    ];
    let path = candidates
        .iter()
        .find(|path| regular_file(path))
        .ok_or_else(|| ObserverError::State {
            path: candidates[0].clone(),
            message: "Work Item Contract/Archive not found".into(),
        })?;
    let value: Value =
        serde_json::from_slice(&fs::read(path).map_err(|source| ObserverError::Read {
            path: path.clone(),
            source,
        })?)
        .map_err(|error| ObserverError::State {
            path: path.clone(),
            message: error.to_string(),
        })?;
    let is_archive_manifest = path
        .file_name()
        .is_some_and(|name| name.to_string_lossy().ends_with(".archive.json"));
    let contract_path = if is_archive_manifest {
        value
            .pointer("/files/contractPath")
            .and_then(Value::as_str)
            .map(|relative| root.join(relative))
            .ok_or_else(|| ObserverError::State {
                path: path.clone(),
                message: "archive manifest has no contractPath".into(),
            })?
    } else {
        path.clone()
    };
    let contract_value: Value =
        serde_json::from_slice(&fs::read(&contract_path).map_err(|source| {
            ObserverError::Read {
                path: contract_path.clone(),
                source,
            }
        })?)
        .map_err(|error| ObserverError::State {
            path: contract_path.clone(),
            message: error.to_string(),
        })?;
    let contract: Contract =
        serde_json::from_value(contract_value.clone()).map_err(|error| ObserverError::State {
            path: contract_path.clone(),
            message: format!("invalid Contract: {error}"),
        })?;
    let summary_path = if is_archive_manifest {
        value
            .pointer("/files/summaryPath")
            .and_then(Value::as_str)
            .map(|relative| root.join(relative))
            .ok_or_else(|| ObserverError::State {
                path: path.clone(),
                message: "archive manifest has no summaryPath".into(),
            })?
    } else {
        ai.join(format!("work-items/active/{work_item_id}.summary.json"))
    };
    if fs::symlink_metadata(&summary_path).is_ok() && !regular_file(&summary_path) {
        return Err(ObserverError::State {
            path: summary_path,
            message: "Work Item Summary must be a regular non-symlink file".into(),
        });
    }
    let summary: Value = if regular_file(&summary_path) {
        serde_json::from_slice(
            &fs::read(&summary_path).map_err(|source| ObserverError::Read {
                path: summary_path.clone(),
                source,
            })?,
        )
        .map_err(|error| ObserverError::State {
            path: summary_path,
            message: error.to_string(),
        })?
    } else {
        Value::Object(Default::default())
    };
    let (scenario_state, mut unknowns, mut findings) =
        validate_scenario_coverage_values(&contract_value, &summary);
    let (acceptance_state, acceptance_unknowns, acceptance_findings) =
        validate_acceptance_evidence_values(&contract, &summary);
    unknowns.extend(acceptance_unknowns);
    findings.extend(acceptance_findings);
    let (intent_state, intent_unknowns, intent_findings) =
        validate_intent_alignment_values(&contract, &summary);
    unknowns.extend(intent_unknowns);
    findings.extend(intent_findings);
    let (final_state, final_unknowns, final_findings) =
        if let Some(receipt) = summary.get("finalDimensions") {
            let report = validate_final_dimensions_value_with_runtime(
                receipt,
                Some(&repository_id(root).to_string()),
                Some(work_item_id),
                runtime,
            );
            (report.state, report.unknowns, report.findings)
        } else {
            ("not_applicable".into(), Vec::new(), Vec::new())
        };
    unknowns.extend(final_unknowns);
    findings.extend(final_findings);
    let state = if findings.iter().any(|item| item.severity == "error") {
        "blocked"
    } else if unknowns.is_empty() {
        "verified"
    } else {
        "unknown"
    };
    Ok(GovernanceControlsReport {
        state: state.into(),
        scenario_coverage: scenario_state,
        acceptance_evidence: acceptance_state,
        intent_alignment: intent_state,
        final_dimensions: final_state,
        unknowns,
        findings,
    })
}

/// Persist an explicitly supplied governance projection into the active
/// Summary. The Runtime only accepts the known projection keys (including the
/// identity-bound preflight decision receipt) and never changes Contract facts,
/// lifecycle state, or verification receipts.
pub fn record_work_item_governance_controls(
    root: &Path,
    work_item_id: &str,
    controls: &Value,
) -> Result<Value, ObserverError> {
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    if !regular_file(&summary_path) {
        return Err(ObserverError::State {
            path: summary_path,
            message: "Work Item Summary must be a regular non-symlink file".into(),
        });
    }
    let mut summary =
        serde_json::from_slice::<Value>(&fs::read(&summary_path).map_err(|source| {
            ObserverError::Read {
                path: summary_path.clone(),
                source,
            }
        })?)
        .map_err(|error| ObserverError::State {
            path: summary_path.clone(),
            message: error.to_string(),
        })?;
    let Some(object) = controls.as_object() else {
        return Err(ObserverError::State {
            path: summary_path,
            message: "governance controls input must be a JSON object".into(),
        });
    };
    for key in object.keys() {
        if !matches!(
            key.as_str(),
            "scenarioCoverage"
                | "acceptanceEvidence"
                | "intentAlignment"
                | "finalDimensions"
                | "decisionEvidence"
        ) {
            return Err(ObserverError::State {
                path: summary_path,
                message: format!("unsupported governance projection field {key}"),
            });
        }
    }
    let Some(summary_object) = summary.as_object_mut() else {
        return Err(ObserverError::State {
            path: summary_path,
            message: "Work Item Summary must be a JSON object".into(),
        });
    };
    let decision_evidence = if let Some(value) = object.get("decisionEvidence") {
        Some(validate_preflight_decision_evidence(
            root,
            work_item_id,
            value,
        )?)
    } else {
        None
    };
    for key in [
        "scenarioCoverage",
        "acceptanceEvidence",
        "intentAlignment",
        "finalDimensions",
    ] {
        if let Some(value) = object.get(key) {
            if value.is_null() {
                summary_object.remove(key);
            } else {
                summary_object.insert(key.into(), value.clone());
            }
        }
    }
    if let Some(evidence) = decision_evidence {
        let value = serde_json::to_value(&evidence).map_err(|error| ObserverError::State {
            path: summary_path.clone(),
            message: error.to_string(),
        })?;
        summary_object.insert("decisionEvidence".into(), value.clone());
        let decision_path = root
            .join(".ai/decisions")
            .join(format!("{work_item_id}.preflight-review.json"));
        // Never replace an existing path (including a symlink) while
        // persisting a governance receipt.  A replacement would make a
        // malicious link look like a successful repository-local write.
        if fs::symlink_metadata(&decision_path).is_ok() {
            return Err(ObserverError::State {
                path: decision_path,
                message: "preflight decision evidence already exists; rerun preflight before recording a replacement".into(),
            });
        }
        crate::atomic_json(&decision_path, &value)?;
    }
    summary_object.insert("updatedAt".into(), chrono::Utc::now().to_rfc3339().into());
    let bytes = serde_json::to_vec_pretty(&summary).map_err(|error| ObserverError::State {
        path: summary_path.clone(),
        message: error.to_string(),
    })?;
    let temporary = summary_path.with_extension(format!("json.tmp-{}", std::process::id()));
    fs::write(&temporary, bytes).map_err(|source| ObserverError::Read {
        path: temporary.clone(),
        source,
    })?;
    fs::rename(&temporary, &summary_path).map_err(|source| ObserverError::Read {
        path: summary_path.clone(),
        source,
    })?;
    Ok(summary)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PreflightDecisionEvidenceState {
    Missing,
    Valid,
    Invalid,
}

/// Inspect the active Summary and its repository-local receipt without
/// mutating either file. A malformed, stale, foreign, or partially written
/// receipt is deliberately distinguishable from an absent receipt so callers
/// can stop rather than silently treat tampering as a missing optional field.
pub(crate) fn preflight_decision_evidence_state(
    root: &Path,
    work_item_id: &str,
    contract_digest: &Digest,
    preflight_decision_digest: &Digest,
    snapshot_digest: &Digest,
) -> PreflightDecisionEvidenceState {
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let decision_path = root
        .join(".ai/decisions")
        .join(format!("{work_item_id}.preflight-review.json"));
    if !regular_file(&summary_path) {
        return PreflightDecisionEvidenceState::Invalid;
    }
    let Some(summary) = fs::read(&summary_path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
    else {
        return PreflightDecisionEvidenceState::Invalid;
    };
    let Some(value) = summary.get("decisionEvidence").cloned() else {
        return if fs::symlink_metadata(&decision_path).is_ok() {
            PreflightDecisionEvidenceState::Invalid
        } else {
            PreflightDecisionEvidenceState::Missing
        };
    };
    let Ok(evidence) = serde_json::from_value::<PreflightDecisionEvidence>(value.clone()) else {
        return PreflightDecisionEvidenceState::Invalid;
    };
    let valid = evidence.schema_version == 1
        && evidence.decision_id == "contract-preflight-review"
        && evidence.decision == "confirm_review"
        && evidence.work_item_id == work_item_id
        && evidence.repository_id == repository_id(root).to_string()
        && evidence.contract_digest == *contract_digest
        && evidence.preflight_decision_digest == *preflight_decision_digest
        && evidence.repository_snapshot_digest == *snapshot_digest
        && chrono::DateTime::parse_from_rfc3339(&evidence.recorded_at).is_ok()
        && !evidence.recorded_by.trim().is_empty()
        && !evidence.reason.trim().is_empty()
        && regular_file(&decision_path)
        && fs::read(&decision_path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
            .is_some_and(|stored| stored == value.clone());
    if valid {
        PreflightDecisionEvidenceState::Valid
    } else {
        PreflightDecisionEvidenceState::Invalid
    }
}

fn validate_preflight_decision_evidence(
    root: &Path,
    work_item_id: &str,
    value: &Value,
) -> Result<PreflightDecisionEvidence, ObserverError> {
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.summary.json"));
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{work_item_id}.contract.json"));
    let evidence: PreflightDecisionEvidence =
        serde_json::from_value(value.clone()).map_err(|error| ObserverError::State {
            path: summary_path.clone(),
            message: format!("invalid preflight decision evidence: {error}"),
        })?;
    if evidence.schema_version != 1 {
        return Err(ObserverError::State {
            path: summary_path.clone(),
            message: "unsupported preflight decision evidence schemaVersion".into(),
        });
    }
    if evidence.decision_id != "contract-preflight-review" || evidence.decision != "confirm_review"
    {
        return Err(ObserverError::State {
            path: summary_path.clone(),
            message: "preflight decision evidence must select confirm_review".into(),
        });
    }
    if evidence.work_item_id != work_item_id {
        return Err(ObserverError::State {
            path: summary_path.clone(),
            message: "preflight decision evidence Work Item identity mismatch".into(),
        });
    }
    let expected_repository_id = repository_id(root).to_string();
    if evidence.repository_id != expected_repository_id {
        return Err(ObserverError::State {
            path: summary_path.clone(),
            message: "preflight decision evidence repository identity mismatch".into(),
        });
    }
    let expected_contract_digest = crate::contract_digest(&contract_path)?;
    if evidence.contract_digest != expected_contract_digest {
        return Err(ObserverError::State {
            path: contract_path,
            message: "preflight decision evidence Contract digest mismatch".into(),
        });
    }
    let summary: Value =
        serde_json::from_slice(
            &fs::read(&summary_path).map_err(|source| ObserverError::Read {
                path: summary_path.clone(),
                source,
            })?,
        )
        .map_err(|error| ObserverError::State {
            path: summary_path.clone(),
            message: error.to_string(),
        })?;
    let expected_preflight = summary
        .get("preflightDecisionDigest")
        .and_then(Value::as_str)
        .ok_or_else(|| ObserverError::State {
            path: summary_path.clone(),
            message: "preflight decision must be recorded before decision evidence".into(),
        })?
        .parse::<Digest>()
        .map_err(|error| ObserverError::State {
            path: summary_path.clone(),
            message: format!("invalid recorded preflight decision digest: {error}"),
        })?;
    if evidence.preflight_decision_digest != expected_preflight {
        return Err(ObserverError::State {
            path: summary_path.clone(),
            message: "preflight decision evidence digest mismatch".into(),
        });
    }
    let expected_snapshot = summary
        .get("preflightRepositorySnapshotDigest")
        .and_then(Value::as_str)
        .ok_or_else(|| ObserverError::State {
            path: summary_path.clone(),
            message: "preflight snapshot digest is missing".into(),
        })?
        .parse::<Digest>()
        .map_err(|error| ObserverError::State {
            path: summary_path.clone(),
            message: format!("invalid recorded preflight snapshot digest: {error}"),
        })?;
    if evidence.repository_snapshot_digest != expected_snapshot {
        return Err(ObserverError::State {
            path: summary_path.clone(),
            message: "preflight decision evidence snapshot digest mismatch".into(),
        });
    }
    if evidence.recorded_by.trim().is_empty()
        || evidence.reason.trim().is_empty()
        || chrono::DateTime::parse_from_rfc3339(&evidence.recorded_at).is_err()
    {
        return Err(ObserverError::State {
            path: summary_path,
            message:
                "preflight decision evidence requires a valid RFC3339 timestamp, actor, and reason"
                    .into(),
        });
    }
    Ok(evidence)
}

/// Helper for callers that already hold a typed Contract and Summary.
pub fn validate_contract_summary_controls(
    contract: &Contract,
    contract_value: &Value,
    summary: &Value,
) -> GovernanceControlsReport {
    validate_contract_summary_controls_internal(contract, contract_value, summary, None)
}

pub fn validate_contract_summary_controls_with_runtime(
    contract: &Contract,
    contract_value: &Value,
    summary: &Value,
    runtime: &RuntimeContext,
) -> GovernanceControlsReport {
    validate_contract_summary_controls_internal(contract, contract_value, summary, Some(runtime))
}

fn validate_contract_summary_controls_internal(
    contract: &Contract,
    contract_value: &Value,
    summary: &Value,
    runtime: Option<&RuntimeContext>,
) -> GovernanceControlsReport {
    let (scenario_state, mut unknowns, mut findings) =
        validate_scenario_coverage_values(contract_value, summary);
    let (acceptance_state, acceptance_unknowns, acceptance_findings) =
        validate_acceptance_evidence_values(contract, summary);
    unknowns.extend(acceptance_unknowns);
    findings.extend(acceptance_findings);
    let (intent_state, intent_unknowns, intent_findings) =
        validate_intent_alignment_values(contract, summary);
    unknowns.extend(intent_unknowns);
    findings.extend(intent_findings);
    let (final_state, final_unknowns, final_findings) = summary
        .get("finalDimensions")
        .map(|value| {
            let report = validate_final_dimensions_value_with_runtime(
                value,
                None,
                Some(&contract.work_item_id),
                runtime,
            );
            (report.state, report.unknowns, report.findings)
        })
        .unwrap_or_else(|| ("not_applicable".into(), Vec::new(), Vec::new()));
    unknowns.extend(final_unknowns);
    findings.extend(final_findings);
    GovernanceControlsReport {
        state: if findings.iter().any(|item| item.severity == "error") {
            "blocked".into()
        } else if unknowns.is_empty() {
            "verified".into()
        } else {
            "unknown".into()
        },
        scenario_coverage: scenario_state,
        acceptance_evidence: acceptance_state,
        intent_alignment: intent_state,
        final_dimensions: final_state,
        unknowns,
        findings,
    }
}

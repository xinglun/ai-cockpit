use cockpit_core::Digest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const GATE_PLAN_SCHEMA_VERSION: u32 = 1;
pub const GATE_PLAN_FAILURE_EXIT_CODE: i32 = 2;
const PROFILES: [&str; 3] = ["light", "standard", "strict"];
const STAGES: [&str; 5] = ["task", "pre_ci", "pull_request", "merge", "release"];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GateManifest {
    pub gates: Vec<GateDefinition>,
    pub path_profiles: ProfilePatterns,
    pub profile_order: Vec<String>,
    pub release_owned_patterns: Vec<String>,
    pub schema_version: u32,
    pub stage_floors: BTreeMap<String, String>,
    pub unknown_profile: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfilePatterns {
    pub light: Vec<String>,
    pub standard: Vec<String>,
    pub strict: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GateDefinition {
    pub category: String,
    pub command: Vec<String>,
    #[serde(default)]
    pub covers: Option<Vec<String>>,
    pub id: String,
    pub minimum_profile: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GatePlanInput {
    pub base_revision: String,
    pub head_revision: String,
    pub stage: String,
    pub risk: String,
    pub requested_profile: Option<String>,
    pub changed_paths: Vec<String>,
    pub manifest_digest: String,
    pub contract_path: Option<String>,
    pub contract_digest: Option<String>,
    pub contract_risk: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PathDecision {
    pub path: String,
    pub profile: String,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GatePlan {
    pub schema_version: u32,
    pub kind: String,
    pub base_revision: String,
    pub head_revision: String,
    pub stage: String,
    pub risk: String,
    pub requested_risk: String,
    pub requested_profile: Option<String>,
    pub manifest_digest: String,
    pub changed_paths: Vec<String>,
    pub contract_path: Option<String>,
    pub contract_digest: Option<String>,
    pub automatic_profile: String,
    pub path_decisions: Vec<PathDecision>,
    pub reasons: Vec<String>,
    pub required_gate_ids: Vec<String>,
    pub selected_profile: String,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GatePlanFailure {
    pub state: String,
    pub failure_code: String,
    pub remediation: String,
}

pub fn failure_metadata(detail: &str) -> GatePlanFailure {
    let normalized = detail.to_ascii_lowercase();
    let (failure_code, remediation) = if normalized.contains("lifecycle_transition_stale")
        || (normalized.contains("lifecycle transition") && normalized.contains("stale"))
    {
        (
            "lifecycle_transition_stale",
            "use the Runtime recovery path, refresh evidence, and push only the repaired state",
        )
    } else if normalized.contains("lifecycle_transition_invalid")
        || (normalized.contains("lifecycle transition") && normalized.contains("invalid"))
    {
        (
            "lifecycle_transition_invalid",
            "restore the declared lifecycle order and checkpoint/preflight bindings before pushing",
        )
    } else if normalized.contains("required_evidence_missing") {
        (
            "required_evidence_missing",
            "collect the Contract-required evidence and rerun the declared verification",
        )
    } else if normalized.contains("reference") && normalized.contains("inventory") {
        (
            "reference_inventory_mismatch",
            "refresh the pinned reference inventory and rerun the conformance check",
        )
    } else {
        (
            "quality_route_failed",
            "inspect the bound route receipt and rerun the declared repository gate locally",
        )
    };
    GatePlanFailure {
        state: "failed".into(),
        failure_code: failure_code.into(),
        remediation: remediation.into(),
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GatePlanError {
    #[error("gate manifest JSON is invalid: {0}")]
    ManifestJson(String),
    #[error("gate manifest schema is invalid: {0}")]
    Manifest(String),
    #[error("unsupported verification stage: {0}")]
    Stage(String),
    #[error("unsupported profile: {0}")]
    Profile(String),
    #[error("unsafe changed path: {0}")]
    UnsafePath(String),
    #[error("gate plan contract risk is missing")]
    ContractRiskMissing,
    #[error("gate plan lifecycle transition is invalid: {0}")]
    LifecycleInvalid(String),
    #[error("gate plan lifecycle transition is stale: {0}")]
    LifecycleStale(String),
    #[error("gate plan JSON is invalid: {0}")]
    PlanJson(String),
    #[error("gate plan receipt does not match current repository facts")]
    PlanMismatch,
}

pub fn load_manifest(bytes: &[u8]) -> Result<GateManifest, GatePlanError> {
    let manifest: GateManifest = serde_json::from_slice(bytes)
        .map_err(|error| GatePlanError::ManifestJson(error.to_string()))?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

pub fn validate_manifest(manifest: &GateManifest) -> Result<(), GatePlanError> {
    if manifest.schema_version != 2 {
        return Err(GatePlanError::Manifest("schemaVersion must be 2".into()));
    }
    if manifest.profile_order != PROFILES {
        return Err(GatePlanError::Manifest(
            "profileOrder must be [light, standard, strict]".into(),
        ));
    }
    if !PROFILES.contains(&manifest.unknown_profile.as_str()) {
        return Err(GatePlanError::Manifest("unknownProfile is invalid".into()));
    }
    if manifest.gates.is_empty() {
        return Err(GatePlanError::Manifest("gates must not be empty".into()));
    }
    let mut ids = BTreeSet::new();
    let mut commands = BTreeSet::new();
    for gate in &manifest.gates {
        if gate.id.trim().is_empty() || !ids.insert(gate.id.clone()) {
            return Err(GatePlanError::Manifest(
                "gate IDs must be sorted and unique".into(),
            ));
        }
        if gate.command.is_empty() || gate.command.iter().any(|value| value.trim().is_empty()) {
            return Err(GatePlanError::Manifest(format!(
                "gate {} command must be a non-empty list",
                gate.id
            )));
        }
        if !commands.insert(gate.command.clone()) {
            return Err(GatePlanError::Manifest(
                "gate commands must be unique".into(),
            ));
        }
        if !PROFILES.contains(&gate.minimum_profile.as_str()) {
            return Err(GatePlanError::Manifest(format!(
                "gate {} minimumProfile is invalid",
                gate.id
            )));
        }
        if let Some(covers) = &gate.covers {
            if covers.is_empty() {
                return Err(GatePlanError::Manifest(format!(
                    "gate {} covers must be a non-empty list",
                    gate.id
                )));
            }
            if covers.iter().any(|value| value.trim().is_empty()) {
                return Err(GatePlanError::Manifest(format!(
                    "gate {} covers contains an empty value",
                    gate.id
                )));
            }
        }
    }
    if manifest
        .gates
        .windows(2)
        .any(|window| window[0].id >= window[1].id)
    {
        return Err(GatePlanError::Manifest(
            "gate IDs must be sorted and unique".into(),
        ));
    }
    for profile in [
        &manifest.path_profiles.light,
        &manifest.path_profiles.standard,
        &manifest.path_profiles.strict,
    ] {
        if profile.is_empty() || profile.iter().any(|value| value.trim().is_empty()) {
            return Err(GatePlanError::Manifest(
                "pathProfiles must contain non-empty pattern lists".into(),
            ));
        }
    }
    if manifest.release_owned_patterns.is_empty()
        || manifest
            .release_owned_patterns
            .iter()
            .any(|value| value.trim().is_empty())
    {
        return Err(GatePlanError::Manifest(
            "releaseOwnedPatterns must contain non-empty patterns".into(),
        ));
    }
    if manifest.stage_floors.len() != STAGES.len()
        || STAGES.iter().any(|stage| {
            manifest
                .stage_floors
                .get(*stage)
                .is_none_or(|profile| !PROFILES.contains(&profile.as_str()))
        })
    {
        return Err(GatePlanError::Manifest(
            "stageFloors must define every verification stage".into(),
        ));
    }
    Ok(())
}

pub fn plan_gate_route(
    manifest: &GateManifest,
    input: &GatePlanInput,
) -> Result<GatePlan, GatePlanError> {
    validate_manifest(manifest)?;
    if !STAGES.contains(&input.stage.as_str()) {
        return Err(GatePlanError::Stage(input.stage.clone()));
    }
    let changed_paths = normalize_paths(&input.changed_paths)?;
    let effective_risk = if input.risk.trim().eq_ignore_ascii_case("high")
        || input.risk.trim().eq_ignore_ascii_case("critical")
        || input.risk.trim().eq_ignore_ascii_case("destructive")
    {
        input.risk.clone()
    } else {
        input
            .contract_risk
            .clone()
            .unwrap_or_else(|| input.risk.clone())
    };
    let mut path_decisions = Vec::with_capacity(changed_paths.len());
    for path in &changed_paths {
        let (profile, reason) = classify_path(manifest, path);
        path_decisions.push(PathDecision {
            path: path.clone(),
            profile,
            reason,
        });
    }
    let mut automatic = path_decisions
        .iter()
        .map(|decision| decision.profile.as_str())
        .max_by_key(|profile| profile_rank(profile))
        .unwrap_or(manifest.unknown_profile.as_str())
        .to_owned();
    let mut reasons = path_decisions
        .iter()
        .filter(|decision| decision.profile == automatic)
        .map(|decision| decision.reason.clone())
        .collect::<Vec<_>>();
    let stage_floor = manifest
        .stage_floors
        .get(&input.stage)
        .expect("validated stage floor");
    if profile_rank(stage_floor) > profile_rank(&automatic) {
        automatic = stage_floor.clone();
        reasons.push(format!(
            "stage {} requires at least {}",
            input.stage, stage_floor
        ));
    }
    if ["high", "critical", "destructive"]
        .iter()
        .any(|risk| effective_risk.trim().eq_ignore_ascii_case(risk))
        && automatic != "strict"
    {
        automatic = "strict".into();
        reasons.push(format!("risk {} requires strict", effective_risk));
    }
    let selected = match &input.requested_profile {
        None => automatic.clone(),
        Some(profile) => {
            validate_profile(profile)?;
            if profile_rank(profile) < profile_rank(&automatic) {
                return Err(GatePlanError::Profile(format!(
                    "explicit profile {} cannot lower automatic profile {}",
                    profile, automatic
                )));
            }
            if profile_rank(profile) > profile_rank(&automatic) {
                reasons.push(format!("explicit escalation to {}", profile));
            }
            profile.clone()
        }
    };
    reasons.sort();
    reasons.dedup();
    if reasons.is_empty() {
        reasons.push(format!("empty diff defaults to {}", automatic));
    }
    let required_gate_ids = manifest
        .gates
        .iter()
        .filter(|gate| profile_rank(&selected) >= profile_rank(&gate.minimum_profile))
        .map(|gate| gate.id.clone())
        .collect();
    let mut plan = GatePlan {
        schema_version: GATE_PLAN_SCHEMA_VERSION,
        kind: "repository_quality_route".into(),
        base_revision: input.base_revision.clone(),
        head_revision: input.head_revision.clone(),
        stage: input.stage.clone(),
        risk: effective_risk,
        requested_risk: input.risk.clone(),
        requested_profile: input.requested_profile.clone(),
        manifest_digest: input.manifest_digest.clone(),
        changed_paths,
        contract_path: input.contract_path.clone(),
        contract_digest: input.contract_digest.clone(),
        automatic_profile: automatic,
        path_decisions,
        reasons,
        required_gate_ids,
        selected_profile: selected,
        receipt_digest: String::new(),
    };
    plan.receipt_digest = canonical_digest_without_receipt(&plan)?;
    Ok(plan)
}

pub fn validate_gate_plan(
    manifest: &GateManifest,
    input: &GatePlanInput,
    plan: &GatePlan,
) -> Result<(), GatePlanError> {
    if plan.receipt_digest != canonical_digest_without_receipt(plan)? {
        return Err(GatePlanError::PlanMismatch);
    }
    let expected = plan_gate_route(manifest, input)?;
    if &expected != plan {
        return Err(GatePlanError::PlanMismatch);
    }
    Ok(())
}

pub fn normalize_paths(paths: &[String]) -> Result<Vec<String>, GatePlanError> {
    let mut normalized = BTreeSet::new();
    for raw in paths {
        let value = raw.replace('\\', "/");
        if value.is_empty() || value.starts_with('/') || value.starts_with("./") {
            return Err(GatePlanError::UnsafePath(raw.clone()));
        }
        let mut parts = Vec::new();
        for part in value.split('/') {
            match part {
                "" | "." => {}
                ".." => return Err(GatePlanError::UnsafePath(raw.clone())),
                part => parts.push(part),
            }
        }
        if parts.is_empty() {
            return Err(GatePlanError::UnsafePath(raw.clone()));
        }
        normalized.insert(parts.join("/"));
    }
    Ok(normalized.into_iter().collect())
}

pub fn validate_lifecycle_summary(summary: &Value) -> Result<(), GatePlanError> {
    let object = summary
        .as_object()
        .ok_or_else(|| GatePlanError::LifecycleInvalid("Summary must be an object".into()))?;
    let state = object.get("state").and_then(Value::as_str);
    if !matches!(
        state,
        Some("implementation_active" | "checkpointed" | "finish_ready")
    ) {
        return Err(GatePlanError::LifecycleInvalid(format!(
            "active lifecycle state {:?} cannot enter the CI route",
            state
        )));
    }
    if matches!(state, Some("checkpointed" | "finish_ready"))
        && object.get("checkpointCount").and_then(Value::as_u64) != Some(1)
    {
        return Err(GatePlanError::LifecycleInvalid(
            "checkpointed lifecycle does not contain exactly one checkpoint".into(),
        ));
    }
    if state == Some("finish_ready")
        && object.get("preflightState").and_then(Value::as_str) != Some("green")
    {
        return Err(GatePlanError::LifecycleInvalid(
            "finish_ready lifecycle is not backed by a green preflight".into(),
        ));
    }
    if object
        .get("failedGate")
        .is_some_and(|value| !value.is_null())
    {
        return Err(GatePlanError::LifecycleStale(
            "active lifecycle contains a failed transition marker".into(),
        ));
    }
    if object
        .get("finalizationState")
        .and_then(Value::as_str)
        .is_some_and(|value| matches!(value, "stale" | "invalid" | "blocked"))
    {
        return Err(GatePlanError::LifecycleStale(
            "active lifecycle contains a stale transition marker".into(),
        ));
    }
    Ok(())
}

fn classify_path(manifest: &GateManifest, path: &str) -> (String, String) {
    if manifest
        .release_owned_patterns
        .iter()
        .any(|pattern| wildcard_match(pattern, path))
    {
        return (
            "strict".into(),
            format!("release-owned path requires strict: {path}"),
        );
    }
    let matches = [
        ("light", &manifest.path_profiles.light),
        ("standard", &manifest.path_profiles.standard),
        ("strict", &manifest.path_profiles.strict),
    ]
    .into_iter()
    .filter(|(_, patterns)| patterns.iter().any(|pattern| wildcard_match(pattern, path)))
    .map(|(profile, _)| profile)
    .collect::<Vec<_>>();
    if let Some(profile) = matches
        .into_iter()
        .max_by_key(|profile| profile_rank(profile))
    {
        return (profile.into(), format!("{profile} path policy: {path}"));
    }
    (
        manifest.unknown_profile.clone(),
        format!(
            "unknown path defaults to {}: {path}",
            manifest.unknown_profile
        ),
    )
}

fn validate_profile(profile: &str) -> Result<(), GatePlanError> {
    if PROFILES.contains(&profile) {
        Ok(())
    } else {
        Err(GatePlanError::Profile(profile.into()))
    }
}

fn profile_rank(profile: &str) -> usize {
    PROFILES
        .iter()
        .position(|candidate| *candidate == profile)
        .unwrap_or(usize::MAX)
}

fn wildcard_match(pattern: &str, value: &str) -> bool {
    let pattern = pattern.as_bytes();
    let value = value.as_bytes();
    let mut table = vec![vec![false; value.len() + 1]; pattern.len() + 1];
    table[0][0] = true;
    for index in 1..=pattern.len() {
        if pattern[index - 1] == b'*' {
            table[index][0] = table[index - 1][0];
        }
    }
    for pattern_index in 1..=pattern.len() {
        for value_index in 1..=value.len() {
            table[pattern_index][value_index] = match pattern[pattern_index - 1] {
                b'*' => {
                    table[pattern_index - 1][value_index] || table[pattern_index][value_index - 1]
                }
                b'?' => table[pattern_index - 1][value_index - 1],
                byte => byte == value[value_index - 1] && table[pattern_index - 1][value_index - 1],
            };
        }
    }
    table[pattern.len()][value.len()]
}

fn canonical_digest_without_receipt(plan: &GatePlan) -> Result<String, GatePlanError> {
    let mut value =
        serde_json::to_value(plan).map_err(|error| GatePlanError::PlanJson(error.to_string()))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| GatePlanError::PlanJson("gate plan must serialize to an object".into()))?;
    object.remove("receiptDigest");
    let bytes = canonical_json(&value);
    Ok(Digest::sha256_bytes(&bytes).to_string())
}

fn canonical_json(value: &Value) -> Vec<u8> {
    match value {
        Value::Object(object) => {
            let mut sorted = BTreeMap::new();
            for (key, value) in object {
                sorted.insert(key, canonical_json(value));
            }
            let mut bytes = Vec::from(b"{".as_slice());
            for (index, (key, value)) in sorted.into_iter().enumerate() {
                if index > 0 {
                    bytes.push(b',');
                }
                bytes.extend(serde_json::to_vec(key).expect("JSON string cannot fail"));
                bytes.push(b':');
                bytes.extend(value);
            }
            bytes.push(b'}');
            bytes
        }
        Value::Array(values) => {
            let mut bytes = Vec::from(b"[".as_slice());
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    bytes.push(b',');
                }
                bytes.extend(canonical_json(value));
            }
            bytes.push(b']');
            bytes
        }
        _ => serde_json::to_vec(value).expect("JSON scalar serialization cannot fail"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> GateManifest {
        GateManifest {
            gates: vec![
                GateDefinition {
                    category: "ci".into(),
                    command: vec!["true".into()],
                    covers: None,
                    id: "ci_light".into(),
                    minimum_profile: "light".into(),
                },
                GateDefinition {
                    category: "workspace".into(),
                    command: vec!["cargo".into(), "test".into()],
                    covers: None,
                    id: "workspace_standard".into(),
                    minimum_profile: "standard".into(),
                },
            ],
            path_profiles: ProfilePatterns {
                light: vec!["docs/**".into()],
                standard: vec!["src/**".into()],
                strict: vec![".github/**".into()],
            },
            profile_order: PROFILES.iter().map(|value| (*value).into()).collect(),
            release_owned_patterns: vec!["release/**".into()],
            schema_version: 2,
            stage_floors: STAGES
                .iter()
                .map(|stage| ((*stage).into(), "light".into()))
                .collect(),
            unknown_profile: "strict".into(),
        }
    }

    fn input(paths: &[&str]) -> GatePlanInput {
        GatePlanInput {
            base_revision: "a".repeat(40),
            head_revision: "b".repeat(40),
            stage: "pull_request".into(),
            risk: "normal".into(),
            changed_paths: paths.iter().map(|path| (*path).into()).collect(),
            manifest_digest: "sha256:manifest".into(),
            ..GatePlanInput::default()
        }
    }

    #[test]
    fn selects_highest_path_profile_and_gate_ids_in_manifest_order() {
        let plan =
            plan_gate_route(&manifest(), &input(&["docs/readme.md", "src/lib.rs"])).expect("plan");
        assert_eq!(plan.selected_profile, "standard");
        assert_eq!(
            plan.required_gate_ids,
            vec!["ci_light", "workspace_standard"]
        );
        assert_eq!(plan.changed_paths, vec!["docs/readme.md", "src/lib.rs"]);
    }

    #[test]
    fn rejects_unsafe_paths_before_rule_selection() {
        let error = plan_gate_route(&manifest(), &input(&["../outside.txt"]))
            .expect_err("path must be rejected");
        assert!(matches!(error, GatePlanError::UnsafePath(_)));
    }

    #[test]
    fn receipt_digest_is_stable_and_validation_reuses_same_facts() {
        let manifest = manifest();
        let input = input(&["docs/readme.md"]);
        let plan = plan_gate_route(&manifest, &input).expect("plan");
        validate_gate_plan(&manifest, &input, &plan).expect("same facts validate");
        let mut changed = input.clone();
        changed.changed_paths = vec!["src/lib.rs".into()];
        assert_eq!(
            validate_gate_plan(&manifest, &changed, &plan),
            Err(GatePlanError::PlanMismatch)
        );
    }

    #[test]
    fn lifecycle_boundary_rejects_future_or_stale_states() {
        let invalid = serde_json::json!({"state":"closed"});
        assert!(matches!(
            validate_lifecycle_summary(&invalid),
            Err(GatePlanError::LifecycleInvalid(_))
        ));
        let stale = serde_json::json!({"state":"implementation_active","failedGate":"x"});
        assert!(matches!(
            validate_lifecycle_summary(&stale),
            Err(GatePlanError::LifecycleStale(_))
        ));
    }
}

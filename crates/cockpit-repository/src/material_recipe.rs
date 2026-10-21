//! Version-owned wire recipes. No current Contract, service request, scanner
//! DTO, or verifier source bytes participate in selecting these recipes.
use cockpit_core::Digest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest as _, Sha256};
use std::{
    collections::BTreeMap,
    io::{self, Write},
};

pub(super) const ARTIFACT_LIMIT: usize = 4 * 1024 * 1024;
pub(super) const REQUEST_RECIPE_V2: &str = "ai-cockpit:material-review-request:v2";
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum UnknownCauseV1 {
    MissingCompleteSource,
    NonTextMaterial,
    SourceOverBudget,
    InvalidProvenance,
    ParseUnavailable,
    AnalysisIncomplete,
    ReadableCommittedRustSyntaxUnknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ScannerAssessmentV1 {
    Clean,
    Finding,
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct EntryV1 {
    pub(super) path: String,
    pub(super) kind: String,
    pub(super) content_state: String,
    pub(super) scanner_assessment: ScannerAssessmentV1,
    pub(super) unknown_cause: Option<UnknownCauseV1>,
    pub(super) reviewable: bool,
    pub(super) changed_hunk_digest: Digest,
    pub(super) after_blob_digest: Option<Digest>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct RequestV1 {
    pub(super) schema_version: u32,
    pub(super) repository_id: String,
    pub(super) work_item_id: String,
    pub(super) contract_digest: Digest,
    pub(super) immutable_contract_base_revision: String,
    pub(super) source_snapshot_digest: Digest,
    pub(super) material_manifest_digest: Digest,
    pub(super) scanner_semantic_version: String,
    pub(super) analysis_implementation_digest: Digest,
    pub(super) analysis_target_semantic_profile: String,
    pub(super) material_inspection_review_profile_digest: Option<Digest>,
    pub(super) effective_policy_digest: Digest,
    pub(super) entries: Vec<EntryV1>,
    pub(super) raw_unknown_codes: Vec<String>,
    pub(super) finding_codes: Vec<String>,
    pub(super) blocked_by_finding: bool,
    pub(super) review_enabled: bool,
    pub(super) review_diagnostic: Option<String>,
    pub(super) request_digest: Digest,
    /// Provenance only. It is excluded from request_digest so an identical
    /// source manifest in a legitimate descendant remains comparable.
    pub(super) reviewed_source_head: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct RecipeManifestV2 {
    pub(super) request_recipe_id: String,
    pub(super) manifest_recipe_id: String,
    pub(super) hunk_recipe_id: String,
    pub(super) policy_recipe_id: String,
    pub(super) profile_recipe_id: String,
    pub(super) analysis_semantic_version: String,
    pub(super) target_semantic_profile: String,
    pub(super) dependency_profile: String,
}
impl RecipeManifestV2 {
    pub(super) fn supported() -> Self {
        Self {
            request_recipe_id: REQUEST_RECIPE_V2.into(),
            manifest_recipe_id: "ai-cockpit:material-manifest:v1".into(),
            hunk_recipe_id: "ai-cockpit:material-hunk-tuple-json:v1".into(),
            policy_recipe_id: "ai-cockpit:material-policy-wire-json:v1".into(),
            profile_recipe_id: "ai-cockpit:material-profile-wire-json:v1".into(),
            analysis_semantic_version: "committed-material-analysis-v1".into(),
            target_semantic_profile: "cross-platform-rust-source-v1".into(),
            dependency_profile: "material-analysis-locked-dependencies-v1".into(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct RequestV2 {
    pub(super) schema_version: u32,
    pub(super) recipe_manifest: RecipeManifestV2,
    pub(super) repository_id: String,
    pub(super) work_item_id: String,
    pub(super) contract_digest: Digest,
    pub(super) immutable_contract_base_revision: String,
    pub(super) source_snapshot_digest: Digest,
    pub(super) material_manifest_digest: Digest,
    pub(super) scanner_semantic_version: String,
    pub(super) analysis_implementation_digest: Digest,
    pub(super) analysis_target_semantic_profile: String,
    pub(super) material_inspection_review_profile_digest: Option<Digest>,
    pub(super) effective_policy_digest: Digest,
    pub(super) entries: Vec<EntryV1>,
    pub(super) raw_unknown_codes: Vec<String>,
    pub(super) finding_codes: Vec<String>,
    pub(super) blocked_by_finding: bool,
    pub(super) review_enabled: bool,
    pub(super) review_diagnostic: Option<String>,
    pub(super) request_digest: Digest,
    /// Provenance only. It is excluded from request_digest so an identical
    /// source manifest in a legitimate descendant remains comparable.
    pub(super) reviewed_source_head: String,
}
impl RequestV2 {
    pub(super) fn to_v1(&self) -> RequestV1 {
        RequestV1 {
            schema_version: 1,
            repository_id: self.repository_id.clone(),
            work_item_id: self.work_item_id.clone(),
            contract_digest: self.contract_digest.clone(),
            immutable_contract_base_revision: self.immutable_contract_base_revision.clone(),
            source_snapshot_digest: self.source_snapshot_digest.clone(),
            material_manifest_digest: self.material_manifest_digest.clone(),
            scanner_semantic_version: self.scanner_semantic_version.clone(),
            analysis_implementation_digest: self.analysis_implementation_digest.clone(),
            analysis_target_semantic_profile: self.analysis_target_semantic_profile.clone(),
            material_inspection_review_profile_digest: self
                .material_inspection_review_profile_digest
                .clone(),
            effective_policy_digest: self.effective_policy_digest.clone(),
            entries: self.entries.clone(),
            raw_unknown_codes: self.raw_unknown_codes.clone(),
            finding_codes: self.finding_codes.clone(),
            blocked_by_finding: self.blocked_by_finding,
            review_enabled: self.review_enabled,
            review_diagnostic: self.review_diagnostic.clone(),
            request_digest: self.request_digest.clone(),
            reviewed_source_head: self.reviewed_source_head.clone(),
        }
    }
    pub(super) fn from_v1(value: RequestV1) -> Self {
        Self {
            schema_version: 2,
            recipe_manifest: RecipeManifestV2::supported(),
            repository_id: value.repository_id,
            work_item_id: value.work_item_id,
            contract_digest: value.contract_digest,
            immutable_contract_base_revision: value.immutable_contract_base_revision,
            source_snapshot_digest: value.source_snapshot_digest,
            material_manifest_digest: value.material_manifest_digest,
            scanner_semantic_version: value.scanner_semantic_version,
            analysis_implementation_digest: value.analysis_implementation_digest,
            analysis_target_semantic_profile: value.analysis_target_semantic_profile,
            material_inspection_review_profile_digest: value
                .material_inspection_review_profile_digest,
            effective_policy_digest: value.effective_policy_digest,
            entries: value.entries,
            raw_unknown_codes: value.raw_unknown_codes,
            finding_codes: value.finding_codes,
            blocked_by_finding: value.blocked_by_finding,
            review_enabled: value.review_enabled,
            review_diagnostic: value.review_diagnostic,
            request_digest: value.request_digest,
            reviewed_source_head: value.reviewed_source_head,
        }
    }
}

fn digest_json<T: Serialize>(value: &T) -> Result<Digest, String> {
    serde_json::to_vec(value)
        .map(|bytes| Digest::sha256_bytes(&bytes))
        .map_err(|e| e.to_string())
}
fn sorted(value: Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.into_iter().map(sorted).collect()),
        Value::Object(values) => Value::Object(
            values
                .into_iter()
                .map(|(key, value)| (key, sorted(value)))
                .collect::<BTreeMap<_, _>>()
                .into_iter()
                .collect(),
        ),
        other => other,
    }
}
pub(super) fn request_digest_v1(request: &RequestV1) -> Result<Digest, String> {
    if request.schema_version != 1 {
        return Err("unsupported V1 request schema".into());
    }
    let mut value = serde_json::to_value(request).map_err(|e| e.to_string())?;
    let fields = value.as_object_mut().ok_or("request must be an object")?;
    for field in ["requestDigest", "reviewedSourceHead", "reviewDiagnostic"] {
        fields.remove(field);
    }
    digest_json(&("ai-cockpit:material-review-request:v1", sorted(value)))
}
fn request_digest_v2(request: &RequestV2) -> Result<Digest, String> {
    if request.schema_version != 2
        || request.recipe_manifest != RecipeManifestV2::supported()
        || request.scanner_semantic_version != "rust-material-v1"
        || request.analysis_target_semantic_profile != "cross-platform-rust-source-v1"
    {
        return Err("unsupported material request recipe or analysis dependency profile".into());
    }
    let mut value = serde_json::to_value(request).map_err(|e| e.to_string())?;
    let fields = value.as_object_mut().ok_or("request must be an object")?;
    for field in [
        "requestDigest",
        "reviewedSourceHead",
        "reviewDiagnostic",
        "analysisImplementationDigest",
    ] {
        fields.remove(field);
    }
    digest_json(&(REQUEST_RECIPE_V2, sorted(value)))
}
struct LimitedWriter {
    bytes: Vec<u8>,
}
impl Write for LimitedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.bytes.len().saturating_add(bytes.len()) > ARTIFACT_LIMIT {
            return Err(io::Error::other(
                "canonical material request exceeds 4194304 bytes",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub(super) struct SealedRequestV2 {
    pub(super) request: RequestV2,
    pub(super) canonical_bytes: Vec<u8>,
    pub(super) raw_digest: Digest,
}
pub(super) fn seal_v2(mut request: RequestV2) -> Result<SealedRequestV2, String> {
    request.request_digest = request_digest_v2(&request)?;
    if request.entries.len() > 256 {
        return Err("material request exceeds 256 entries".into());
    }
    let value = sorted(serde_json::to_value(&request).map_err(|e| e.to_string())?);
    validate_depth(&value, 0)?;
    let mut writer = LimitedWriter { bytes: Vec::new() };
    serde_json::to_writer(&mut writer, &value).map_err(|e| e.to_string())?;
    let raw_digest = Digest::sha256_bytes(&writer.bytes);
    Ok(SealedRequestV2 {
        request,
        canonical_bytes: writer.bytes,
        raw_digest,
    })
}
fn validate_depth(value: &Value, depth: usize) -> Result<(), String> {
    if depth > 32 {
        return Err("material artifact exceeds nesting depth32".into());
    }
    match value {
        Value::Array(values) => {
            if values.len() > 256 {
                return Err("material artifact array exceeds256 entries".into());
            }
            for value in values {
                validate_depth(value, depth + 1)?;
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                validate_depth(value, depth + 1)?;
            }
        }
        Value::Number(number) if !number.is_u64() => {
            return Err("V2 material artifacts require unsigned integer wire values".into());
        }
        _ => {}
    }
    Ok(())
}
pub(super) fn decode_v2(bytes: &[u8]) -> Result<SealedRequestV2, String> {
    if bytes.is_empty() || bytes.len() > ARTIFACT_LIMIT {
        return Err("material artifact size is invalid".into());
    }
    // This shared guard only rejects malformed JSON; it never supplies recipe fields.
    super::reject_duplicate_json_keys(bytes)?;
    let value: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    validate_depth(&value, 0)?;
    let request: RequestV2 = serde_json::from_value(value).map_err(|e| e.to_string())?;
    let expected = request.request_digest.clone();
    let sealed = seal_v2(request)?;
    if sealed.request.request_digest != expected || sealed.canonical_bytes != bytes {
        return Err("material request digest or canonical artifact bytes do not match".into());
    }
    Ok(sealed)
}
pub(super) fn decode_receipt_v1(bytes: &[u8]) -> Result<ReceiptV1, String> {
    super::reject_duplicate_json_keys(bytes)?;
    let receipt: ReceiptV1 = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    receipt.validate_integrity()?;
    Ok(receipt)
}
pub(super) fn select_legacy_generator(digest: &Digest) -> Result<(), String> {
    match digest.as_str() {
        "sha256:8b5a0b834a01d7e2b88d08a8d1de3f1f347ce22b0ab2f90745cb476eb3e71d7c"
        // WI1090's original generator at 6048fdcc maps to this frozen V1
        // recipe; reconstruction retains its original ccf provenance.
        | "sha256:ccf952067cbe6eba3dfa116f7a3d87293d69f6c7ddb5747953157ff63493cc83" => Ok(()),
        _ => Err("historical material generator has no unambiguous audited V1 mapping".into()),
    }
}
pub(super) fn implementation_digest_v1(sources: &[(&str, &[u8])]) -> Digest {
    let mut hasher = Sha256::new();
    hasher.update(b"ai-cockpit:material-analysis-implementation:v1\0");
    for (path, bytes) in sources {
        hasher.update((path.len() as u64).to_le_bytes());
        hasher.update(path.as_bytes());
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
    }
    Digest::sha256_bytes(&hasher.finalize())
}
pub const MATERIAL_INSPECTION_REVIEW_DECISION_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum DecisionV1 {
    AcceptPermittedUnknowns,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum AssuranceV1 {
    SelfDeclared,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct EvidenceRefV1 {
    pub path: String,
    pub digest: Digest,
}

/// Typed, self-declared review evidence. `recorded_by` and `reviewer_actor`
/// are separate provenance facts; neither field authenticates a real-world
/// identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ReceiptV1 {
    pub schema_version: u32,
    pub repository_id: String,
    pub work_item_id: String,
    pub contract_digest: Digest,
    pub material_manifest_digest: Digest,
    pub profile_digest: Digest,
    pub request_digest: Digest,
    /// Provenance outside request equality; it must be an ancestor of a
    /// later consumer head before a hosted projection can reuse this receipt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed_source_head: Option<String>,
    pub decision: DecisionV1,
    pub reviewer_actor: String,
    pub recorded_by: String,
    pub authority_source: String,
    pub assurance: AssuranceV1,
    pub evidence_refs: Vec<EvidenceRefV1>,
    pub rationale: String,
    pub residual_risk: String,
    pub recorded_at: String,
    pub receipt_digest: Digest,
}

impl ReceiptV1 {
    pub fn canonical_digest(&self) -> Result<Digest, String> {
        let mut value = serde_json::to_value(self)
            .map_err(|error| format!("cannot encode material review receipt: {error}"))?;
        let fields = value
            .as_object_mut()
            .ok_or_else(|| "material review receipt did not encode as a JSON object".to_owned())?;
        if fields.remove("receiptDigest").is_none() {
            return Err("material review receipt digest field is missing".into());
        }
        digest_json(&("ai-cockpit:material-inspection-review-decision:v1", value))
            .map_err(|error| format!("cannot digest material review receipt: {error}"))
    }

    pub fn validate_integrity(&self) -> Result<(), String> {
        for digest in [
            &self.contract_digest,
            &self.material_manifest_digest,
            &self.profile_digest,
            &self.request_digest,
            &self.receipt_digest,
        ]
        .into_iter()
        .chain(self.evidence_refs.iter().map(|reference| &reference.digest))
        {
            digest
                .as_str()
                .parse::<Digest>()
                .map_err(|_| "material review receipt contains an invalid digest".to_owned())?;
        }
        if self.schema_version != MATERIAL_INSPECTION_REVIEW_DECISION_SCHEMA_VERSION {
            return Err("unsupported material review receipt schema version".into());
        }
        if self.canonical_digest()? != self.receipt_digest {
            return Err("material review receipt digest does not match canonical content".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cockpit_core::Digest;
    use serde_json::{Value, json};

    fn request_v1() -> RequestV1 {
        let digest = Digest::sha256_bytes(b"synthetic material identity");
        serde_json::from_value(json!({
            "schemaVersion":1,"repositoryId":"synthetic:repository","workItemId":"WI-SYNTHETIC",
            "contractDigest":digest,"immutableContractBaseRevision":"1111111111111111111111111111111111111111",
            "sourceSnapshotDigest":digest,"materialManifestDigest":digest,
            "scannerSemanticVersion":"rust-material-v1","analysisImplementationDigest":digest,
            "analysisTargetSemanticProfile":"cross-platform-rust-source-v1",
            "materialInspectionReviewProfileDigest":null,"effectivePolicyDigest":digest,
            "entries":[],"rawUnknownCodes":[],"findingCodes":[],"blockedByFinding":false,
            "reviewEnabled":false,"reviewDiagnostic":null,"requestDigest":digest,
            "reviewedSourceHead":"2222222222222222222222222222222222222222"
        })).unwrap()
    }

    #[test]
    fn frozen_v1_codec_ignores_only_its_declared_provenance() {
        let mut request = request_v1();
        let original = request_digest_v1(&request).unwrap();
        request.reviewed_source_head = "3333333333333333333333333333333333333333".into();
        request.review_diagnostic = Some("synthetic diagnostic".into());
        request.request_digest = Digest::sha256_bytes(b"excluded cached digest");
        assert_eq!(request_digest_v1(&request).unwrap(), original);
        request.analysis_implementation_digest = Digest::sha256_bytes(b"another generator");
        assert_ne!(request_digest_v1(&request).unwrap(), original);
    }

    #[test]
    fn frozen_v1_receipt_keeps_the_real_original_digest() {
        let vectors: Value = serde_json::from_str(include_str!(
            "../tests/fixtures/material_review_versions.json"
        ))
        .unwrap();
        let bytes = serde_json::to_vec(&vectors["legacyV1"][0]["receipt"]).unwrap();
        let receipt = decode_receipt_v1(&bytes).unwrap();
        receipt.validate_integrity().unwrap();
        assert_eq!(
            receipt.canonical_digest().unwrap().as_str(),
            "sha256:547e8981ba971c66b0b522ecf23885ed68dabd1619fbc25931d91184e875dcb0"
        );
    }

    #[test]
    fn v2_semantic_identity_and_reviewed_raw_artifact_are_separate() {
        let first = seal_v2(RequestV2::from_v1(request_v1())).unwrap();
        let mut second_request = first.request.clone();
        second_request.analysis_implementation_digest = Digest::sha256_bytes(b"another generator");
        second_request.reviewed_source_head = "3333333333333333333333333333333333333333".into();
        let second = seal_v2(second_request).unwrap();
        assert_eq!(first.request.request_digest, second.request.request_digest);
        assert_ne!(first.raw_digest, second.raw_digest);
        assert_ne!(first.canonical_bytes, second.canonical_bytes);
    }

    #[test]
    fn v2_full_canonical_artifact_enforces_the_exact_four_mib_boundary() {
        let mut request = RequestV2::from_v1(request_v1());
        request.review_diagnostic = Some(String::new());
        let base = seal_v2(request.clone()).unwrap().canonical_bytes.len();
        let cap = 4 * 1024 * 1024;
        for limit in [cap - 1, cap] {
            request.review_diagnostic = Some("x".repeat(limit - base));
            assert_eq!(
                seal_v2(request.clone()).unwrap().canonical_bytes.len(),
                limit
            );
        }
        request.review_diagnostic = Some("x".repeat(cap + 1 - base));
        assert!(seal_v2(request).is_err());
    }

    #[test]
    fn v2_decoder_rejects_noncanonical_duplicate_unknown_and_changed_semantic_data() {
        let sealed = seal_v2(RequestV2::from_v1(request_v1())).unwrap();
        decode_v2(&sealed.canonical_bytes).unwrap();
        let mut whitespace = sealed.canonical_bytes.clone();
        whitespace.push(b'\n');
        assert!(decode_v2(&whitespace).is_err());
        let duplicate = String::from_utf8(sealed.canonical_bytes.clone())
            .unwrap()
            .replace(
                "\"schemaVersion\":2",
                "\"schemaVersion\":2,\"schemaVersion\":2",
            );
        assert!(decode_v2(duplicate.as_bytes()).is_err());
        let mut value: Value = serde_json::from_slice(&sealed.canonical_bytes).unwrap();
        value["other"] = json!(true);
        assert!(decode_v2(&serde_json::to_vec(&value).unwrap()).is_err());
        value.as_object_mut().unwrap().remove("other");
        value["blockedByFinding"] = json!(true);
        assert!(decode_v2(&serde_json::to_vec(&value).unwrap()).is_err());
    }

    #[test]
    fn legacy_generator_bridge_is_explicit_and_fails_closed() {
        let known: Digest =
            "sha256:8b5a0b834a01d7e2b88d08a8d1de3f1f347ce22b0ab2f90745cb476eb3e71d7c"
                .parse()
                .unwrap();
        select_legacy_generator(&known).unwrap();
        assert!(select_legacy_generator(&Digest::sha256_bytes(b"unmapped generator")).is_err());
    }
    #[test]
    fn legacy_generator_bridge_accepts_original_wi1090_ccf_provenance() {
        let original: Digest =
            "sha256:ccf952067cbe6eba3dfa116f7a3d87293d69f6c7ddb5747953157ff63493cc83"
                .parse()
                .unwrap();
        select_legacy_generator(&original)
            .expect("the independently audited WI1090 generator has the frozen V1 recipe");
        let changed: Digest =
            "sha256:ccf952067cbe6eba3dfa116f7a3d87293d69f6c7ddb5747953157ff63493cc82"
                .parse()
                .unwrap();
        assert!(select_legacy_generator(&changed).is_err());
        assert!(select_legacy_generator(&Digest::sha256_bytes(b"unmapped generator")).is_err());
    }
    #[test]
    fn v2_multibyte_and_escaped_text_obey_encoded_byte_limits() {
        let mut request = RequestV2::from_v1(request_v1());
        request.review_diagnostic = Some(String::new());
        let base = seal_v2(request.clone()).unwrap().canonical_bytes.len();
        let cap = 4 * 1024 * 1024;
        for symbol in ["é", "\n"] {
            let encoded = serde_json::to_vec(symbol).unwrap().len() - 2;
            request.review_diagnostic =
                Some(symbol.repeat((cap - base) / encoded) + &"x".repeat((cap - base) % encoded));
            let sealed = seal_v2(request.clone()).unwrap();
            assert_eq!(sealed.canonical_bytes.len(), cap);
            decode_v2(&sealed.canonical_bytes).unwrap();
            request.review_diagnostic.as_mut().unwrap().push('x');
            assert!(seal_v2(request.clone()).is_err());
        }
    }

    #[test]
    fn v2_limits_nesting_arrays_and_supported_analysis_labels() {
        let mut request = RequestV2::from_v1(request_v1());
        request.raw_unknown_codes = vec!["synthetic".into(); 257];
        assert!(seal_v2(request).is_err());
        let mut request = RequestV2::from_v1(request_v1());
        request.scanner_semantic_version = "unknown-scanner".into();
        assert!(seal_v2(request).is_err());
        let sealed = seal_v2(RequestV2::from_v1(request_v1())).unwrap();
        let mut value: Value = serde_json::from_slice(&sealed.canonical_bytes).unwrap();
        let mut deep = json!(null);
        for _ in 0..34 {
            deep = json!({"next":deep});
        }
        value["reviewDiagnostic"] = deep;
        let error = match decode_v2(&serde_json::to_vec(&value).unwrap()) {
            Err(error) => error,
            Ok(_) => panic!("deep artifact admitted"),
        };
        assert!(error.contains("depth32"), "{error}");
    }
}

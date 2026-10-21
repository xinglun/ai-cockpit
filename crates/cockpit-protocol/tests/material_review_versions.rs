use cockpit_core::Digest;
use cockpit_protocol::{
    MaterialInspectionReviewDecisionInput, MaterialInspectionReviewDecisionReceipt,
};
use serde_json::{Value, json};

fn frozen_receipt() -> Value {
    let vectors: Value = serde_json::from_str(include_str!(
        "../../cockpit-repository/tests/fixtures/material_review_versions.json"
    ))
    .unwrap();
    vectors["legacyV1"][0]["receipt"].clone()
}

fn synthetic_v2_receipt() -> MaterialInspectionReviewDecisionReceipt {
    let mut value = frozen_receipt();
    let request = Digest::sha256_bytes(b"synthetic V2 semantic request");
    let bytes = b"synthetic full request artifact";
    let artifact = Digest::sha256_bytes(bytes);
    let name = format!(
        "{}.{}.request.json",
        request.as_str().strip_prefix("sha256:").unwrap(),
        artifact.as_str().strip_prefix("sha256:").unwrap()
    );
    value["schemaVersion"] = json!(2);
    value["requestDigest"] = json!(request);
    value["requestArtifact"] = json!({
        "path": format!(".ai/evidence/material-inspection-review/WI-1078-v1-0-1-post-merge-sync-finalization/{name}"),
        "rawDigest": artifact,
        "byteLength": bytes.len(),
        "requestDigest": request,
        "recipeId": "ai-cockpit:material-review-request:v2"
    });
    let mut receipt: MaterialInspectionReviewDecisionReceipt =
        serde_json::from_value(value).unwrap();
    receipt.receipt_digest = receipt.canonical_digest().unwrap();
    receipt
}

#[test]
fn legacy_v1_receipt_preserves_original_wire_bytes_and_digest() {
    let value = frozen_receipt();
    let current: MaterialInspectionReviewDecisionReceipt =
        serde_json::from_value(value.clone()).unwrap();
    let original: legacy_reader_v1::MaterialInspectionReviewDecisionReceipt =
        serde_json::from_value(value).unwrap();
    current.validate_integrity().unwrap();
    original.validate_integrity().unwrap();
    assert!(current.request_artifact.is_none());
    assert_eq!(
        current.canonical_digest().unwrap(),
        original.canonical_digest().unwrap()
    );
    assert_eq!(
        current.receipt_digest.as_str(),
        "sha256:547e8981ba971c66b0b522ecf23885ed68dabd1619fbc25931d91184e875dcb0"
    );
    let bytes = serde_json::to_vec_pretty(&current).unwrap();
    assert_eq!(bytes.len(), 2852);
    assert_eq!(
        Digest::sha256_bytes(&bytes).as_str(),
        "sha256:dedd05464f1ffe377a4ae506220b3378ce1be1a5103a4301d9c3df898d9f1a7f"
    );
}

#[test]
fn actual_frozen_old_decoder_rejects_v2_fields_and_version_for_the_right_reason() {
    let receipt = synthetic_v2_receipt();
    receipt.validate_integrity().unwrap();
    let mut value = serde_json::to_value(&receipt).unwrap();
    let error =
        serde_json::from_value::<legacy_reader_v1::MaterialInspectionReviewDecisionReceipt>(
            value.clone(),
        )
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("unknown field `requestArtifact`"),
        "{error}"
    );
    value.as_object_mut().unwrap().remove("requestArtifact");
    let decoded: legacy_reader_v1::MaterialInspectionReviewDecisionReceipt =
        serde_json::from_value(value).unwrap();
    assert_eq!(
        decoded.validate_integrity().unwrap_err(),
        "unsupported material review receipt schema version"
    );
}

#[test]
fn v2_receipt_integrity_covers_the_complete_artifact_reference() {
    let original = synthetic_v2_receipt();
    original.validate_integrity().unwrap();
    for field in [
        "path",
        "rawDigest",
        "byteLength",
        "requestDigest",
        "recipeId",
    ] {
        let mut value = serde_json::to_value(&original).unwrap();
        value["requestArtifact"][field] = match field {
            "rawDigest" | "requestDigest" => json!(Digest::sha256_bytes(b"other artifact")),
            "byteLength" => json!(1),
            _ => json!("other"),
        };
        let changed: MaterialInspectionReviewDecisionReceipt =
            serde_json::from_value(value).unwrap();
        assert!(
            changed.validate_integrity().is_err(),
            "unbound field {field}"
        );
    }
    let mut missing = original.clone();
    missing.request_artifact = None;
    assert!(missing.validate_integrity().is_err());
    let mut disguised = original;
    disguised.schema_version = 1;
    assert!(disguised.validate_integrity().is_err());
}

#[test]
fn v2_decision_binds_raw_artifact_when_semantic_digest_is_identical() {
    let receipt = synthetic_v2_receipt();
    let artifact = receipt.request_artifact.as_ref().unwrap();
    let input: MaterialInspectionReviewDecisionInput = serde_json::from_value(json!({
        "schemaVersion": 2,
        "decision": "accept_permitted_unknowns",
        "requestDigest": receipt.request_digest,
        "requestArtifactDigest": artifact.raw_digest,
        "requestArtifactByteLength": artifact.byte_length,
        "reviewerActor": "agent:synthetic-reviewer",
        "authoritySource": "synthetic:bounded-test",
        "assurance": "self_declared",
        "evidenceRefs": [],
        "rationale": "Synthetic wire binding test.",
        "residualRisk": "Synthetic only; no real approval."
    }))
    .unwrap();
    input
        .validate_request_binding(
            2,
            &receipt.request_digest,
            Some((&artifact.raw_digest, artifact.byte_length)),
        )
        .unwrap();
    let other = Digest::sha256_bytes(b"same semantics but different generator provenance");
    assert!(
        input
            .validate_request_binding(
                2,
                &receipt.request_digest,
                Some((&other, artifact.byte_length))
            )
            .is_err()
    );
    assert!(
        input
            .validate_request_binding(
                2,
                &receipt.request_digest,
                Some((&artifact.raw_digest, artifact.byte_length + 1))
            )
            .is_err()
    );
    assert!(
        input
            .validate_request_binding(1, &receipt.request_digest, None)
            .is_err()
    );
    let mut legacy = input;
    legacy.schema_version = 1;
    assert!(
        legacy
            .validate_request_binding(
                2,
                &receipt.request_digest,
                Some((&artifact.raw_digest, artifact.byte_length))
            )
            .is_err()
    );
    assert!(
        legacy
            .validate_request_binding(1, &receipt.request_digest, None)
            .is_err()
    );
}

// Audited production decoder copied from 84fb04ee43e33842865a1544a5588466174c0639.
// Fragment SHA256: 4bca2c066d52b74ecb81b36fb924de26c2f4e4082b2adf63d316a4c3f8aa8034.
// Its local schema constant and receipt type are independent of the new reader.
mod legacy_reader_v1 {
    use cockpit_core::Digest;
    use cockpit_protocol::digest_json;
    use serde::{Deserialize, Serialize};

    pub const MATERIAL_INSPECTION_REVIEW_DECISION_SCHEMA_VERSION: u32 = 1;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum MaterialInspectionReviewDecision {
        AcceptPermittedUnknowns,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum MaterialInspectionReviewAssurance {
        SelfDeclared,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    pub struct MaterialInspectionReviewEvidenceRef {
        pub path: String,
        pub digest: Digest,
    }

    /// Typed, self-declared review evidence. `recorded_by` and `reviewer_actor`
    /// are separate provenance facts; neither field authenticates a real-world
    /// identity.
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    pub struct MaterialInspectionReviewDecisionReceipt {
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
        pub decision: MaterialInspectionReviewDecision,
        pub reviewer_actor: String,
        pub recorded_by: String,
        pub authority_source: String,
        pub assurance: MaterialInspectionReviewAssurance,
        pub evidence_refs: Vec<MaterialInspectionReviewEvidenceRef>,
        pub rationale: String,
        pub residual_risk: String,
        pub recorded_at: String,
        pub receipt_digest: Digest,
    }

    impl MaterialInspectionReviewDecisionReceipt {
        pub fn canonical_digest(&self) -> Result<Digest, String> {
            let mut value = serde_json::to_value(self)
                .map_err(|error| format!("cannot encode material review receipt: {error}"))?;
            let fields = value.as_object_mut().ok_or_else(|| {
                "material review receipt did not encode as a JSON object".to_owned()
            })?;
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
                return Err(
                    "material review receipt digest does not match canonical content".into(),
                );
            }
            Ok(())
        }
    }
}

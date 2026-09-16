use cockpit_core::Digest;
use cockpit_protocol::{
    ResourceFinalizationPullRequestIdentity, SelectedSuccessorLineageEdge,
    SelectedSuccessorLineageHumanDecision, SelectedSuccessorLineageNode,
    SelectedSuccessorLineageRecoveryReceipt, validate_selected_successor_lineage_recovery,
};

fn digest(label: &str) -> Digest {
    Digest::sha256_bytes(label.as_bytes())
}

fn node(id: &str) -> SelectedSuccessorLineageNode {
    SelectedSuccessorLineageNode {
        work_item_id: id.into(),
        contract_path: format!(".ai/work-items/archive/{id}.contract.json"),
        contract_digest: digest("contract"),
        summary_path: format!(".ai/work-items/archive/{id}.summary.json"),
        summary_digest: digest("summary"),
        outcome_path: format!(".ai/work-items/archive/{id}.outcome.json"),
        outcome_digest: digest("outcome"),
        events_path: format!(".ai/work-items/archive/{id}.events.jsonl"),
        events_digest: digest("events"),
        verification_path: format!(".ai/evidence/{id}.verification.json"),
        verification_digest: digest("verification"),
        archive_manifest_path: format!(".ai/work-items/archive/{id}.archive.json"),
        archive_manifest_digest: digest("archive"),
        close_path: format!(".ai/decisions/{id}.close.json"),
        close_digest: digest("close"),
        finalization_path: format!(".ai/decisions/{id}.finalize.json"),
        finalization_digest: digest("finalize"),
        finalization_provider: "github".into(),
        finalization_pull_request: ResourceFinalizationPullRequestIdentity {
            number: 1,
            url: "https://github.com/example/repo/pull/1".into(),
            head_revision: "head".into(),
            base_branch: "main".into(),
            base_remote: "origin".into(),
            base_revision: "base".into(),
            merge_commit: Some("merge".into()),
        },
        finalization_runtime_version: "0.2.92".into(),
        finalization_runtime_digest: digest("runtime"),
        human_decision: SelectedSuccessorLineageHumanDecision {
            actor: "human:owner".into(),
            authority_source: "explicit-user-authorization".into(),
            reason: "recover the selected lineage".into(),
            evidence_refs: vec![".ai/evidence/lineage.json".into()],
            policy_refs: vec!["AGENTS.md".into()],
            decided_at: "2026-09-16T09:00:00Z".into(),
            resume_condition: "preserve all original bytes".into(),
        },
    }
}

fn receipt() -> SelectedSuccessorLineageRecoveryReceipt {
    SelectedSuccessorLineageRecoveryReceipt {
        schema_version: 1,
        receipt_id: "selected-successor-lineage-recovery".into(),
        root_work_item_id: "WI-ROOT".into(),
        repository_id: digest("repository").to_string(),
        edges: vec![
            SelectedSuccessorLineageEdge {
                sequence: 0,
                predecessor_work_item_id: "WI-ROOT".into(),
                successor_work_item_id: "WI-MIDDLE".into(),
                recovery_path: ".ai/decisions/WI-ROOT.recovery.json".into(),
                recovery_digest: digest("root-recovery"),
            },
            SelectedSuccessorLineageEdge {
                sequence: 1,
                predecessor_work_item_id: "WI-MIDDLE".into(),
                successor_work_item_id: "WI-TERMINAL".into(),
                recovery_path: ".ai/decisions/WI-MIDDLE.recovery.json".into(),
                recovery_digest: digest("middle-recovery"),
            },
        ],
        nodes: vec![node("WI-ROOT"), node("WI-MIDDLE"), node("WI-TERMINAL")],
        runtime_version: "0.2.92".into(),
        runtime_digest: digest("runtime"),
    }
}

#[test]
fn selected_lineage_recovery_shape_is_strict_and_adjacent() {
    let value = serde_json::to_value(receipt()).unwrap();
    let parsed: SelectedSuccessorLineageRecoveryReceipt = serde_json::from_value(value).unwrap();
    validate_selected_successor_lineage_recovery(&parsed).unwrap();
    assert_eq!(parsed.nodes[2].work_item_id, "WI-TERMINAL");
}

#[test]
fn selected_lineage_recovery_rejects_disconnected_edges() {
    let mut value = serde_json::to_value(receipt()).unwrap();
    value["edges"][1]["predecessorWorkItemId"] = serde_json::json!("WI-OTHER");
    let parsed: SelectedSuccessorLineageRecoveryReceipt = serde_json::from_value(value).unwrap();
    assert!(validate_selected_successor_lineage_recovery(&parsed).is_err());
}

#[test]
fn selected_lineage_recovery_rejects_unknown_fields() {
    let mut value = serde_json::to_value(receipt()).unwrap();
    value["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<SelectedSuccessorLineageRecoveryReceipt>(value).is_err());
}

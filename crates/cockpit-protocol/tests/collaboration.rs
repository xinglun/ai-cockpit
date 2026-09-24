use cockpit_core::Digest;
use cockpit_protocol::{
    COLLABORATION_CAPABILITY, CollaborationDeclaration, CompositionBinding, ConsumedOutcome,
    Contract, IntegrationResponsibility, OutcomeStage, ProvidedOutcome, ResourceClaim,
    ResourceClaimMode, RuntimeCapabilityBinding, WorktreeRegistration,
};

fn digest(label: &str) -> Digest {
    Digest::sha256_bytes(label.as_bytes())
}

fn declaration() -> CollaborationDeclaration {
    CollaborationDeclaration {
        provided_outcomes: vec![ProvidedOutcome {
            outcome_id: "provider-api".into(),
            interface_contract: "api-v1".into(),
            behavior_contract: "returns stable values".into(),
            published_head: "0123456789012345678901234567890123456789".into(),
            stage: OutcomeStage::ComposableHead,
            evidence_refs: vec!["target/provider.json".into()],
        }],
        consumed_outcomes: vec![ConsumedOutcome {
            provider_work_item_id: "WI-PROVIDER".into(),
            outcome_id: "provider-api".into(),
            minimum_stage: OutcomeStage::InterfaceStable,
            verification_required: true,
        }],
        resource_claims: vec![ResourceClaim {
            resource_id: "crates/cockpit-protocol/src/lib.rs".into(),
            mode: ResourceClaimMode::Exclusive,
            serial: true,
        }],
        integration_responsibility: IntegrationResponsibility {
            responsible_work_item_id: "WI-INTEGRATION".into(),
            target_branch: "main".into(),
            composition_order: vec!["WI-PROVIDER".into(), "WI-CONSUMER".into()],
            rationale: "provider before consumer".into(),
        },
        composition_verification: cockpit_protocol::CompositionVerification {
            compatibility_constraints: vec!["api-v1".into()],
            required_scenarios: vec!["composition_premerge_failure".into()],
            reusable_nodes: vec!["workspace-format".into()],
        },
    }
}

fn runtime_binding() -> RuntimeCapabilityBinding {
    RuntimeCapabilityBinding {
        schema_version: 1,
        runtime_version: "0.2.113".into(),
        runtime_digest: digest("candidate-runtime"),
        capability: COLLABORATION_CAPABILITY.into(),
    }
}

#[test]
fn collaboration_identities_round_trip_strictly() {
    let registration = WorktreeRegistration {
        schema_version: 1,
        repository_id: digest("repo"),
        work_item_id: "WI-CONSUMER".into(),
        contract_digest: digest("contract"),
        worktree_path: "/tmp/worktree".into(),
        branch: "codex/consumer".into(),
        head: "0123456789012345678901234567890123456789".into(),
        generation: 2,
        declaration: declaration(),
        runtime: runtime_binding(),
    };
    let encoded = serde_json::to_value(&registration).expect("encode");
    let decoded: WorktreeRegistration = serde_json::from_value(encoded.clone()).expect("decode");
    assert_eq!(decoded, registration);
    let mut unknown = encoded;
    unknown["unknownField"] = serde_json::json!(true);
    assert!(serde_json::from_value::<WorktreeRegistration>(unknown).is_err());
}

#[test]
fn composable_head_is_not_merged_target_and_old_contracts_remain_readable() {
    assert!(OutcomeStage::InterfaceStable.can_transition_to(OutcomeStage::ComposableHead));
    assert!(OutcomeStage::ComposableHead.can_transition_to(OutcomeStage::MergedTarget));
    assert!(!OutcomeStage::ComposableHead.satisfies(OutcomeStage::MergedTarget));
    assert!(OutcomeStage::ComposableHead.satisfies(OutcomeStage::InterfaceStable));

    let legacy = serde_json::json!({
        "protocolVersion": 1,
        "repositoryId": "sha256:repo",
        "intent": "legacy intent",
        "goal": "legacy goal",
        "scope": ["src/**"],
        "outOfScope": [],
        "risk": "normal",
        "authority": "authorized",
        "acceptanceCriteria": ["legacy acceptance"],
        "requiredEvidenceClasses": [],
        "baseRevision": "legacy-base",
        "projectProfileDigest": "sha256:profile",
        "repositorySnapshotDigest": "sha256:snapshot"
    });
    let contract: Contract = serde_json::from_value(legacy).expect("legacy contract");
    contract.validate().expect("legacy contract validates");
}

#[test]
fn mismatched_runtime_binding_is_rejected_before_collaboration() {
    let old = RuntimeCapabilityBinding {
        schema_version: 1,
        runtime_version: "0.2.105".into(),
        runtime_digest: digest("fixed-runtime"),
        capability: "lifecycle_v1".into(),
    };
    let error = old.validate_candidate().expect_err("old runtime rejected");
    assert!(error.to_string().contains("unsupported_runtime_capability"));
}

#[test]
fn composition_binding_round_trips_with_exact_participant_order() {
    let binding = CompositionBinding {
        schema_version: 1,
        repository_id: digest("repo"),
        binding_id: "composition-1".into(),
        target_branch: "main".into(),
        target_sha: "0123456789012345678901234567890123456789".into(),
        participant_work_items: vec!["WI-PROVIDER".into(), "WI-CONSUMER".into()],
        participant_heads: vec![
            "1111111111111111111111111111111111111111".into(),
            "2222222222222222222222222222222222222222".into(),
        ],
        contract_digests: vec![digest("provider-contract"), digest("consumer-contract")],
        verifier: runtime_binding(),
    };
    let encoded = serde_json::to_string(&binding).expect("encode");
    assert_eq!(
        serde_json::from_str::<CompositionBinding>(&encoded).unwrap(),
        binding
    );
}

use cockpit_core::Digest;
use cockpit_protocol::{ProtocolError, RepositoryConfig, validate_protocol_version};

#[test]
fn protocol_v1_is_accepted() {
    assert!(validate_protocol_version(1).is_ok());
}

#[test]
fn unsupported_major_is_fail_closed() {
    let error = validate_protocol_version(2).expect_err("protocol v2 must not be accepted");
    assert!(matches!(error, ProtocolError::UnsupportedMajor(2)));
}

#[test]
fn repository_config_round_trips_through_toml() {
    let config = RepositoryConfig {
        protocol_version: 1,
        repository_schema_version: 1,
        repository_id: "example".into(),
    };
    let encoded = toml::to_string(&config).expect("config serializes");
    let decoded: RepositoryConfig = toml::from_str(&encoded).expect("config parses");
    assert_eq!(decoded, config);
}

#[test]
fn legacy_repository_files_default_to_schema_one() {
    let config: RepositoryConfig =
        toml::from_str("protocol_version = 1\nrepository_id = \"example\"\n")
            .expect("legacy config parses");
    assert_eq!(config.repository_schema_version, 1);

    let manifest = serde_json::json!({
        "schemaVersion": 1,
        "protocolVersion": 1,
        "interfaceVersion": 1,
        "repositoryId": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "rootBinding": {"type": "manifest-parent"},
        "capabilities": ["status"],
        "interfaces": {"cli": {"available": true}, "mcp": {"available": false}},
        "adapter": {"required": false},
        "adapterState": "unconfigured"
    });
    let manifest: cockpit_protocol::AgentInterfaceManifest =
        serde_json::from_value(manifest).expect("legacy manifest parses");
    assert_eq!(manifest.repository_schema_version, 1);
}

#[test]
fn digest_rejects_non_sha256_shape() {
    assert!("sha256:abcd".parse::<Digest>().is_err());
}

#[test]
fn project_profile_digest_is_stable_for_equal_profiles() {
    let profile = cockpit_protocol::ProjectProfile {
        profile_version: 1,
        repository_id: "example".into(),
        tests: vec![],
        build_systems: vec!["cargo".into()],
    };
    let first = cockpit_protocol::digest_json(&profile).expect("digest");
    let second = cockpit_protocol::digest_json(&profile).expect("digest");
    assert_eq!(first, second);
}

#[test]
fn project_profile_rejects_unknown_fields() {
    let value = serde_json::json!({
        "profileVersion": 1,
        "repositoryId": "example",
        "tests": [],
        "buildSystems": [],
        "futurePolicy": "must-not-be-ignored"
    });
    assert!(serde_json::from_value::<cockpit_protocol::ProjectProfile>(value).is_err());
}

#[test]
fn agent_interface_manifest_is_strict_and_round_trips() {
    let manifest = cockpit_protocol::AgentInterfaceManifest {
        schema_version: 1,
        protocol_version: 1,
        repository_schema_version: 1,
        interface_version: 1,
        repository_id: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .into(),
        root_binding: cockpit_protocol::AgentRootBinding {
            binding_type: "manifest-parent".into(),
        },
        capabilities: vec!["inspect".into(), "work-item-scaffold".into()],
        interfaces: cockpit_protocol::AgentInterfaces {
            cli: cockpit_protocol::AgentInterfaceAvailability {
                available: true,
                transport: None,
            },
            mcp: cockpit_protocol::AgentInterfaceAvailability {
                available: true,
                transport: Some("stdio".into()),
            },
        },
        adapter: cockpit_protocol::AgentAdapterCompatibility { required: false },
        adapter_state: "unconfigured".into(),
    };
    let value = serde_json::to_value(&manifest).expect("manifest serializes");
    assert_eq!(
        serde_json::from_value::<cockpit_protocol::AgentInterfaceManifest>(value.clone())
            .expect("manifest parses"),
        manifest
    );
    let mut unknown = value;
    unknown["futureCapability"] = serde_json::json!(true);
    assert!(serde_json::from_value::<cockpit_protocol::AgentInterfaceManifest>(unknown).is_err());
}

#[test]
fn repository_context_keeps_runtime_root_out_of_repository_context() {
    let context = cockpit_protocol::RepositoryContext {
        root: std::path::PathBuf::from("/repo"),
        git_root: std::path::PathBuf::from("/repo"),
        config: RepositoryConfig {
            protocol_version: 1,
            repository_schema_version: 1,
            repository_id: "repo".into(),
        },
    };
    assert_eq!(context.root, context.git_root);
    assert_eq!(context.config.protocol_version, 1);
}

#[test]
fn agent_manifest_rejects_unknown_fields_and_unsupported_interface_version() {
    let value = serde_json::json!({
        "schemaVersion": 1,
        "protocolVersion": 1,
        "interfaceVersion": 2,
        "repositoryId": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "rootBinding": {"type": "manifest-parent"},
        "capabilities": ["status"],
        "interfaces": {"cli": {"available": true}, "mcp": {"available": false}},
        "adapter": {"required": false},
        "adapterState": "unconfigured",
        "futurePolicy": true
    });
    assert!(serde_json::from_value::<cockpit_protocol::AgentInterfaceManifest>(value).is_err());

    let unsupported = serde_json::json!({
        "schemaVersion": 1,
        "protocolVersion": 1,
        "interfaceVersion": 99,
        "repositoryId": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "rootBinding": {"type": "manifest-parent"},
        "capabilities": ["status"],
        "interfaces": {"cli": {"available": true}, "mcp": {"available": false}},
        "adapter": {"required": false},
        "adapterState": "unconfigured"
    });
    let manifest = serde_json::from_value::<cockpit_protocol::AgentInterfaceManifest>(unsupported)
        .expect("wire parsing is separate from version validation");
    assert_eq!(manifest.interface_version, 99);
    assert!(
        cockpit_protocol::validate_agent_interface_version(manifest.interface_version).is_err()
    );
}

#[test]
fn managed_adapter_record_round_trips_canonical_json() {
    let record = cockpit_protocol::ManagedAdapterRecord {
        provider: cockpit_protocol::AgentProvider::Codex,
        adapter_version: 1,
        target: "AGENTS.md".into(),
        mode: "managed-section".into(),
        repository_id: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .into(),
        installed_digest: "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            .into(),
    };
    let encoded = cockpit_protocol::canonical_json(&record).expect("record serializes");
    let decoded: cockpit_protocol::ManagedAdapterRecord =
        serde_json::from_slice(&encoded).expect("record parses");
    assert_eq!(decoded, record);

    let mut unknown: serde_json::Value = serde_json::from_slice(&encoded).expect("json");
    unknown["futureField"] = serde_json::json!(true);
    assert!(serde_json::from_value::<cockpit_protocol::ManagedAdapterRecord>(unknown).is_err());
}

#[test]
fn doctor_report_rejects_unknown_fields() {
    let value = serde_json::json!({
        "schemaVersion": 1,
        "state": "ATTACHED",
        "repositoryId": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "attachment": {"state": "valid"},
        "manifest": {"state": "valid"},
        "adapters": [],
        "interfaces": {"cli": "verified", "mcp": "available"},
        "problems": [],
        "safeActions": [],
        "futureState": "must-not-be-ignored"
    });
    assert!(serde_json::from_value::<cockpit_protocol::AgentDoctorReport>(value).is_err());
}

#[test]
fn enterprise_authority_and_human_decision_are_strict_and_auditable() {
    let authority = cockpit_protocol::AuthorityEvidence {
        assurance: cockpit_protocol::AssuranceLevel::ProviderVerified,
        actor: "provider:github:user-42".into(),
        authority_source: "github-team/security-maintainers".into(),
        operations: vec!["release".into()],
        policy_refs: vec!["org-release-v1".into()],
        evidence_refs: vec![".ai/evidence/github-approval.json".into()],
    };
    let decision = cockpit_protocol::HumanDecision {
        decision: "approved".into(),
        actor: "provider:github:user-42".into(),
        authority_source: "github-team/security-maintainers".into(),
        reason: "fresh verification and bounded scope".into(),
        evidence_refs: vec![".ai/evidence/WI-42.verification.json".into()],
        policy_refs: vec!["org-release-v1".into()],
        decided_at: "2026-08-21T19:00:00Z".into(),
        resume_condition: None,
    };
    let encoded = cockpit_protocol::canonical_json(&(authority.clone(), decision.clone()))
        .expect("enterprise records serialize");
    let decoded: (
        cockpit_protocol::AuthorityEvidence,
        cockpit_protocol::HumanDecision,
    ) = serde_json::from_slice(&encoded).expect("enterprise records parse");
    assert_eq!(decoded, (authority, decision));

    let mut unknown = serde_json::to_value(decoded.0).expect("authority json");
    unknown["untrustedClaim"] = serde_json::json!(true);
    assert!(serde_json::from_value::<cockpit_protocol::AuthorityEvidence>(unknown).is_err());

    let mut unknown_decision = serde_json::to_value(decoded.1).expect("decision json");
    unknown_decision["futureApprovalMode"] = serde_json::json!("dual_control");
    assert!(serde_json::from_value::<cockpit_protocol::HumanDecision>(unknown_decision).is_err());
}

#[test]
fn organization_policy_cannot_be_weakened_by_a_lower_layer() {
    let organization = cockpit_protocol::GovernancePolicy {
        policy_id: "org-production-v1".into(),
        layer: cockpit_protocol::PolicyLayer::Organization,
        rules: vec![cockpit_protocol::PolicyRule {
            operation: "production_destructive".into(),
            approval_mode: cockpit_protocol::ApprovalMode::SingleAuthorizedHuman,
            required_evidence: vec!["hosted_ci".into()],
        }],
    };
    let weakened = cockpit_protocol::GovernancePolicy {
        policy_id: "project-local".into(),
        layer: cockpit_protocol::PolicyLayer::Project,
        rules: vec![cockpit_protocol::PolicyRule {
            operation: "production_destructive".into(),
            approval_mode: cockpit_protocol::ApprovalMode::NoHumanApprovalForLowRisk,
            required_evidence: vec![],
        }],
    };
    let error = cockpit_protocol::validate_policy_overlay(&organization, &weakened)
        .expect_err("lower layer must not weaken organization policy");
    assert!(matches!(
        error,
        cockpit_protocol::PolicyError::Weakening { .. }
    ));
}

#[test]
fn policy_document_merges_layers_without_allowing_weakening() {
    let organization = cockpit_protocol::GovernancePolicy {
        policy_id: "org-release-v1".into(),
        layer: cockpit_protocol::PolicyLayer::Organization,
        rules: vec![cockpit_protocol::PolicyRule {
            operation: "release".into(),
            approval_mode: cockpit_protocol::ApprovalMode::SingleAuthorizedHuman,
            required_evidence: vec!["hosted_ci".into()],
        }],
    };
    let project = cockpit_protocol::GovernancePolicy {
        policy_id: "project-release-v2".into(),
        layer: cockpit_protocol::PolicyLayer::Project,
        rules: vec![cockpit_protocol::PolicyRule {
            operation: "release".into(),
            approval_mode: cockpit_protocol::ApprovalMode::SingleAuthorizedHuman,
            required_evidence: vec!["hosted_ci".into(), "sbom".into()],
        }],
    };
    let effective = cockpit_protocol::merge_policy_layers(&[&organization, &project])
        .expect("stronger project policy is valid");
    assert_eq!(
        effective.rules[0].required_evidence,
        vec!["hosted_ci", "sbom"]
    );
    assert_eq!(
        effective.policy_id,
        "effective:org-release-v1:project-release-v2"
    );
}

#[test]
fn policy_document_rejects_unknown_fields() {
    let value = serde_json::json!({
        "schemaVersion": 1,
        "organization": null,
        "project": null,
        "futureRule": true
    });
    assert!(serde_json::from_value::<cockpit_protocol::GovernancePolicyDocument>(value).is_err());
}

#[test]
fn sensitive_evidence_policy_rejects_secret_full_capture_and_accepts_digest_only() {
    let full = cockpit_protocol::EvidenceRetention {
        classification: cockpit_protocol::DataClassification::SecretProhibited,
        persistence: cockpit_protocol::EvidencePersistence::FullCapture,
        retention_days: None,
        expires_at: None,
        disposal_action: "purge".into(),
    };
    assert!(cockpit_protocol::validate_evidence_retention(&full).is_err());

    let digest_only = cockpit_protocol::EvidenceRetention {
        persistence: cockpit_protocol::EvidencePersistence::DigestOnly,
        ..full
    };
    assert!(cockpit_protocol::validate_evidence_retention(&digest_only).is_ok());
}

#[test]
fn delegated_evidence_and_audit_event_bind_external_identity_without_claiming_ownership() {
    let evidence = cockpit_protocol::DelegatedEvidence {
        provider: "github".into(),
        subject: "run:123".into(),
        origin: "https://github.com/example/repo/actions/runs/123".into(),
        assurance: cockpit_protocol::AssuranceLevel::ProviderVerified,
        collected_at: "2026-08-21T19:00:00Z".into(),
        digest: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .parse()
            .expect("digest"),
        validity: cockpit_protocol::EvidenceValidity::Valid,
        raw_evidence_ref: ".ai/evidence/external/github-run-123.json".into(),
    };
    let event = cockpit_protocol::AuditEvent {
        event_id: "event-123".into(),
        repository_id: "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            .into(),
        work_item_id: Some("WI-42".into()),
        runtime_version: "0.2.2".into(),
        runtime_digest: "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
            .parse()
            .expect("digest"),
        timestamp: "2026-08-21T19:01:00Z".into(),
        event_type: "external_evidence_bound".into(),
        evidence_refs: vec![evidence.raw_evidence_ref.clone()],
    };
    let value = serde_json::to_value((&evidence, &event)).expect("audit records serialize");
    assert_eq!(value[0]["provider"], "github");
    assert_eq!(value[1]["eventType"], "external_evidence_bound");
}

#[test]
fn delegated_evidence_receipt_is_strict_and_binds_repository_and_work_item() {
    let receipt = cockpit_protocol::DelegatedEvidenceReceipt {
        schema_version: 1,
        repository_id: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .into(),
        work_item_id: "WI-EXTERNAL".into(),
        evidence: cockpit_protocol::DelegatedEvidence {
            provider: "github".into(),
            subject: "run:123".into(),
            origin: "https://github.com/example/repo/actions/runs/123".into(),
            assurance: cockpit_protocol::AssuranceLevel::ProviderVerified,
            collected_at: "2026-08-21T19:00:00Z".into(),
            digest: "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                .parse()
                .expect("digest"),
            validity: cockpit_protocol::EvidenceValidity::Valid,
            raw_evidence_ref: ".ai/evidence/external/github-run-123.json".into(),
        },
        runtime_version: "0.2.2".into(),
        runtime_digest: "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
            .parse()
            .expect("runtime digest"),
        bound_at: "2026-08-21T19:01:00Z".into(),
    };
    let value = serde_json::to_value(&receipt).expect("receipt serializes");
    assert_eq!(value["workItemId"], "WI-EXTERNAL");
    assert_eq!(
        serde_json::from_value::<cockpit_protocol::DelegatedEvidenceReceipt>(value.clone())
            .expect("receipt parses"),
        receipt
    );
    let mut unknown = value;
    unknown["providerSignature"] = serde_json::json!("fake");
    assert!(serde_json::from_value::<cockpit_protocol::DelegatedEvidenceReceipt>(unknown).is_err());
}

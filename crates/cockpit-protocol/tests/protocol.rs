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
        repository_id: "example".into(),
    };
    let encoded = toml::to_string(&config).expect("config serializes");
    let decoded: RepositoryConfig = toml::from_str(&encoded).expect("config parses");
    assert_eq!(decoded, config);
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

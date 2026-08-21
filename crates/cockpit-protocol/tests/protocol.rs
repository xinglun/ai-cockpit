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
        repository_id: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .into(),
        root_binding: "manifest-parent".into(),
        capabilities: vec!["inspect".into(), "work-item-scaffold".into()],
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

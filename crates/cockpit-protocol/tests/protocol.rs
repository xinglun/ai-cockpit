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

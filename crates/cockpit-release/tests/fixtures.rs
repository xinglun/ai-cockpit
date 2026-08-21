use std::{fs, path::PathBuf};

use cockpit_release::manifest::ReleaseManifest;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/release/fixtures")
        .join(name)
}

#[test]
fn valid_release_fixture_matches_the_wire_contract() {
    let value = fs::read_to_string(fixture("manifest-valid.json")).unwrap();
    let manifest = ReleaseManifest::parse_str(&value).unwrap();
    assert_eq!(manifest.version, "0.1.0");
    assert_eq!(manifest.artifacts().len(), 5);
}

#[test]
fn invalid_release_fixture_is_rejected_strictly() {
    let value = fs::read_to_string(fixture("manifest-invalid.json")).unwrap();
    let error = ReleaseManifest::parse_str(&value).expect_err("unknown fields must fail closed");
    assert!(error.to_string().contains("unknown"));
}

use cockpit_release::{
    formula::{FormulaSource, render_formula},
    manifest::ReleaseManifest,
};

fn manifest() -> ReleaseManifest {
    let targets = [
        ("aarch64-apple-darwin", "macos", "arm64", "tar.gz"),
        ("aarch64-unknown-linux-gnu", "linux", "arm64", "tar.gz"),
        ("x86_64-apple-darwin", "macos", "x86_64", "tar.gz"),
        ("x86_64-pc-windows-msvc", "windows", "x86_64", "zip"),
        ("x86_64-unknown-linux-gnu", "linux", "x86_64", "tar.gz"),
    ];
    let artifacts = targets
        .into_iter()
        .map(|(target, os, architecture, extension)| {
            serde_json::json!({
                "target": target,
                "os": os,
                "architecture": architecture,
                "runnerImage": match target {
                    "aarch64-apple-darwin" => "macos-15",
                    "aarch64-unknown-linux-gnu" => "ubuntu-24.04-arm",
                    "x86_64-apple-darwin" => "macos-15-intel",
                    "x86_64-pc-windows-msvc" => "windows-2025",
                    "x86_64-unknown-linux-gnu" => "ubuntu-24.04",
                    _ => unreachable!(),
                },
                "archive": {"filename": format!("ai-cockpit-v0.1.0-{target}.{extension}"), "bytes": 3, "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},
                "sbom": {"filename": format!("ai-cockpit-v0.1.0-{target}.spdx.json"), "bytes": 3, "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"},
                "provenanceSubject": format!("ai-cockpit-v0.1.0-{target}.{extension}"),
            })
        })
        .collect::<Vec<_>>();
    ReleaseManifest::parse_str(
        &serde_json::json!({
            "schemaVersion": 1,
            "product": "ai-cockpit",
            "package": "cockpit-cli",
            "version": "0.1.0",
            "tag": "v0.1.0",
            "commit": "0000000000000000000000000000000000000000",
            "cargoLockSha256": "1111111111111111111111111111111111111111111111111111111111111111",
            "artifacts": artifacts,
        })
        .to_string(),
    )
    .unwrap()
}

#[test]
fn production_formula_is_stable_and_contains_both_macos_variants() {
    let manifest = manifest();
    let first = render_formula(
        &manifest,
        FormulaSource::Production {
            release_origin: "https://github.com/xinglun/ai-cockpit/releases/download/".into(),
        },
    )
    .unwrap();
    let second = render_formula(
        &manifest,
        FormulaSource::Production {
            release_origin: "https://github.com/xinglun/ai-cockpit/releases/download/".into(),
        },
    )
    .unwrap();
    assert_eq!(first, second);
    assert!(first.contains("on_arm"));
    assert!(first.contains("on_intel"));
    assert!(first.contains("version \"0.1.0\""));
    assert!(
        first.contains(
            "sha256 \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\""
        )
    );
    assert!(first.contains("bin.install \"ai-cockpit\""));
}

#[test]
fn production_formula_rejects_non_https_origin() {
    let error = render_formula(
        &manifest(),
        FormulaSource::Production {
            release_origin: "http://github.com/xinglun/ai-cockpit/releases/download/".into(),
        },
    )
    .expect_err("production formula must require HTTPS");
    assert!(error.to_string().contains("HTTPS"));
}

#[test]
fn fixture_formula_requires_loopback_and_is_marked_test_only() {
    let formula = render_formula(
        &manifest(),
        FormulaSource::Fixture {
            base_url: "http://127.0.0.1:43127/".into(),
        },
    )
    .unwrap();
    assert!(formula.contains("TEST-ONLY"));
    assert!(formula.contains("http://127.0.0.1:43127/"));

    let error = render_formula(
        &manifest(),
        FormulaSource::Fixture {
            base_url: "http://192.168.1.2:43127/".into(),
        },
    )
    .expect_err("fixture must stay loopback");
    assert!(error.to_string().contains("loopback"));
}

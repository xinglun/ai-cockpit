use std::{path::PathBuf, process::Command};

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates directory")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn rust_check_matches_the_existing_python_gate_without_rewriting_evidence() {
    let repo = repository_root();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let evidence_json = repo.join(".ai/evidence/WI-750-p1-cognitive-benefit-current-base.json");
    let evidence_markdown =
        repo.join(".ai/evidence/external/WI-750-p1-cognitive-benefit-current-base.md");
    let before_json = std::fs::read(&evidence_json).ok();
    let before_markdown = std::fs::read(&evidence_markdown).ok();

    let python = Command::new("python3")
        .arg(repo.join("tests/evaluation/WI-750-p1-cognitive-benefit-current-base.py"))
        .args(["--repo", repo.to_str().expect("utf-8 repo")])
        .args(["--binary", binary, "--check"])
        .output()
        .expect("run Python oracle");
    assert!(
        python.status.success(),
        "Python oracle failed: {}",
        String::from_utf8_lossy(&python.stderr)
    );

    let rust = Command::new(binary)
        .args(["audit", "cognitive-benefit", "--repo"])
        .arg(&repo)
        .arg("--check")
        .output()
        .expect("run Rust evaluator");
    assert!(
        rust.status.success(),
        "Rust evaluator failed: {}",
        String::from_utf8_lossy(&rust.stderr)
    );
    let python_summary: serde_json::Value =
        serde_json::from_slice(&python.stdout).expect("Python summary JSON");
    let rust_summary: serde_json::Value =
        serde_json::from_slice(&rust.stdout).expect("Rust summary JSON");
    assert_eq!(rust_summary, python_summary);
    assert_eq!(std::fs::read(&evidence_json).ok(), before_json);
    assert_eq!(std::fs::read(&evidence_markdown).ok(), before_markdown);
}

#[test]
fn missing_archived_outcome_fails_without_creating_evidence() {
    let repo = tempfile::tempdir().expect("isolated missing-archive repository");
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["audit", "cognitive-benefit", "--repo"])
        .arg(repo.path())
        .arg("--check")
        .output()
        .expect("run Rust evaluator");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("missing real archived Outcome structure"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!repo.path().join(".ai/evidence").exists());
}

#[test]
fn generated_json_and_markdown_match_python_in_an_isolated_checkout() {
    let temp = tempfile::tempdir().expect("isolated checkout parent");
    let checkout = temp.path().join("oracle-checkout");
    let clone = Command::new("git")
        .args(["clone", "--shared", "--quiet"])
        .arg(repository_root())
        .arg(&checkout)
        .output()
        .expect("clone repository fixture");
    assert!(
        clone.status.success(),
        "clone failed: {}",
        String::from_utf8_lossy(&clone.stderr)
    );
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let python = Command::new("python3")
        .arg(checkout.join("tests/evaluation/WI-750-p1-cognitive-benefit-current-base.py"))
        .arg("--repo")
        .arg(&checkout)
        .args(["--binary", binary])
        .output()
        .expect("run Python evaluator");
    assert!(
        python.status.success(),
        "Python evaluator failed: {}",
        String::from_utf8_lossy(&python.stderr)
    );
    let json_path = checkout.join(".ai/evidence/WI-750-p1-cognitive-benefit-current-base.json");
    let markdown_path =
        checkout.join(".ai/evidence/external/WI-750-p1-cognitive-benefit-current-base.md");
    let python_json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&json_path).expect("Python JSON artifact"))
            .expect("Python JSON report");
    let python_markdown = std::fs::read(&markdown_path).expect("Python Markdown artifact");

    let rust = Command::new(binary)
        .args(["audit", "cognitive-benefit", "--repo"])
        .arg(&checkout)
        .args(["--binary", binary])
        .output()
        .expect("run Rust evaluator");
    assert!(
        rust.status.success(),
        "Rust evaluator failed: {}",
        String::from_utf8_lossy(&rust.stderr)
    );
    let rust_json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&json_path).expect("Rust JSON artifact"))
            .expect("Rust JSON report");
    assert_eq!(rust_json, python_json, "full report JSON differs");
    assert_eq!(
        std::fs::read(&markdown_path).expect("Rust Markdown artifact"),
        python_markdown,
        "Markdown differs"
    );
}

#[test]
fn malformed_answer_key_fixture_is_rejected_before_check_writes() {
    let temp = tempfile::tempdir().expect("isolated checkout parent");
    let checkout = temp.path().join("malformed-fixture-checkout");
    let clone = Command::new("git")
        .args(["clone", "--shared", "--quiet"])
        .arg(repository_root())
        .arg(&checkout)
        .output()
        .expect("clone repository fixture");
    assert!(
        clone.status.success(),
        "clone failed: {}",
        String::from_utf8_lossy(&clone.stderr)
    );
    let fixture = checkout.join("tests/conformance/fixtures/scope-exceeded/input.json");
    std::fs::write(&fixture, b"[]\n").expect("replace fixture with a JSON array");
    let evidence_json = checkout.join(".ai/evidence/WI-750-p1-cognitive-benefit-current-base.json");
    let evidence_markdown =
        checkout.join(".ai/evidence/external/WI-750-p1-cognitive-benefit-current-base.md");
    let before_json = std::fs::read(&evidence_json).ok();
    let before_markdown = std::fs::read(&evidence_markdown).ok();
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let python = Command::new("python3")
        .arg(checkout.join("tests/evaluation/WI-750-p1-cognitive-benefit-current-base.py"))
        .arg("--repo")
        .arg(&checkout)
        .args(["--binary", binary, "--check"])
        .output()
        .expect("run Python oracle");
    assert!(!python.status.success());
    assert!(String::from_utf8_lossy(&python.stderr).contains("expected JSON object"));

    let rust = Command::new(binary)
        .args(["audit", "cognitive-benefit", "--repo"])
        .arg(&checkout)
        .args(["--binary", binary, "--check"])
        .output()
        .expect("run Rust evaluator");
    assert!(!rust.status.success());
    assert!(
        String::from_utf8_lossy(&rust.stderr).contains("expected JSON object"),
        "Rust must reject the malformed fixture as a type error: {}",
        String::from_utf8_lossy(&rust.stderr)
    );
    assert_eq!(std::fs::read(&evidence_json).ok(), before_json);
    assert_eq!(std::fs::read(&evidence_markdown).ok(), before_markdown);
}

#[cfg(unix)]
#[test]
fn repository_gate_runs_without_invoking_python() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().expect("isolated command path");
    let fake_python = temp.path().join("python3");
    std::fs::write(
        &fake_python,
        b"#!/bin/sh\necho 'python3 evaluator must not run' >&2\nexit 86\n",
    )
    .expect("write Python tripwire");
    std::fs::set_permissions(&fake_python, std::fs::Permissions::from_mode(0o755))
        .expect("make tripwire executable");
    let mut paths = vec![temp.path().to_path_buf()];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").expect("PATH available"),
    ));
    let path = std::env::join_paths(paths).expect("join PATH");
    let repo = repository_root();
    let output = Command::new("bash")
        .arg(repo.join("tests/evaluation/WI-750-p1-cognitive-benefit-current-base_test.sh"))
        .env("PATH", path)
        .output()
        .expect("run repository gate");
    assert!(
        output.status.success(),
        "repository gate still depends on Python or failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let summary: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("gate summary JSON");
    assert_eq!(summary["cases"], 7);
    assert_eq!(summary["cognitiveBenefitValidated"], false);
}

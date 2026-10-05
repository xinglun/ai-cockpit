use std::{path::PathBuf, process::Command};

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates directory")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn python_executable() -> &'static str {
    if cfg!(windows) { "python" } else { "python3" }
}

fn python_oracle_command() -> Command {
    let mut command = Command::new(python_executable());
    // The oracle decodes captured Rust CLI output. Force UTF-8 mode so Windows
    // ACP settings cannot misdecode those child-process streams.
    command.env("PYTHONUTF8", "1");
    command
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

    let python = python_oracle_command()
        .arg(repo.join("tests/evaluation/WI-750-p1-cognitive-benefit-current-base.py"))
        .args(["--repo", repo.to_str().expect("utf-8 repo")])
        .args(["--binary", binary, "--check"])
        .output()
        .expect("launch Python oracle");
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
        .expect("launch Rust evaluator");
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
        .expect("launch Rust evaluator");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("missing real archived Outcome structure"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!repo.path().join(".ai/evidence").exists());
}

#[test]
fn missing_explicit_binary_fails_before_archive_lookup_without_writing_evidence() {
    let repo = tempfile::tempdir().expect("isolated repository");
    let missing = repo.path().join("missing-ai-cockpit");
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["audit", "cognitive-benefit", "--repo"])
        .arg(repo.path())
        .arg("--binary")
        .arg(&missing)
        .output()
        .expect("launch Rust evaluator");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("explicit --binary") && stderr.contains("does not exist"),
        "invalid explicit binary must be the reported failure: {stderr}"
    );
    assert!(!repo.path().join(".ai/evidence").exists());
    assert!(!repo.path().join("target").exists());
}

#[cfg(unix)]
#[test]
fn non_executable_explicit_binary_does_not_fall_back_or_build() {
    use std::os::unix::fs::PermissionsExt;

    let repo = tempfile::tempdir().expect("isolated repository");
    let selected = repo.path().join("not-executable");
    std::fs::write(&selected, b"not a program").expect("write selected file");
    std::fs::set_permissions(&selected, std::fs::Permissions::from_mode(0o644))
        .expect("make selected file non-executable");

    let bin_dir = repo.path().join("bin");
    std::fs::create_dir(&bin_dir).expect("create isolated PATH");
    let cargo_tripwire = bin_dir.join("cargo");
    std::fs::write(
        &cargo_tripwire,
        b"#!/bin/sh\nprintf invoked > \"$AI_COCKPIT_TEST_CARGO_MARKER\"\nexit 86\n",
    )
    .expect("write Cargo tripwire");
    std::fs::set_permissions(&cargo_tripwire, std::fs::Permissions::from_mode(0o755))
        .expect("make Cargo tripwire executable");
    let marker = repo.path().join("cargo-was-invoked");
    let mut paths = vec![bin_dir];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").expect("PATH available"),
    ));
    let path = std::env::join_paths(paths).expect("join PATH");
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["audit", "cognitive-benefit", "--repo"])
        .arg(repo.path())
        .arg("--binary")
        .arg(&selected)
        .env("PATH", path)
        .env("AI_COCKPIT_TEST_CARGO_MARKER", &marker)
        .output()
        .expect("launch Rust evaluator");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("explicit --binary") && stderr.contains("not executable"),
        "non-executable selection must be the reported failure: {stderr}"
    );
    assert!(!marker.exists(), "must not invoke fallback Cargo build");
    assert!(!repo.path().join(".ai/evidence").exists());
    assert!(!repo.path().join("target").exists());
}

#[cfg(windows)]
#[test]
fn windows_invalid_regular_binary_fails_without_fallback_or_artifact_writes() {
    for check_only in [false, true] {
        let temp = tempfile::tempdir().expect("isolated checkout parent");
        let checkout = temp.path().join("invalid-binary-checkout");
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
        let selected = checkout.join("not-executable.txt");
        std::fs::write(&selected, b"not a Windows executable").expect("write selected file");
        let evidence_json =
            checkout.join(".ai/evidence/WI-750-p1-cognitive-benefit-current-base.json");
        let evidence_markdown =
            checkout.join(".ai/evidence/external/WI-750-p1-cognitive-benefit-current-base.md");
        let before_json = std::fs::read(&evidence_json).ok();
        let before_markdown = std::fs::read(&evidence_markdown).ok();
        let mut command = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"));
        command
            .args(["audit", "cognitive-benefit", "--repo"])
            .arg(&checkout)
            .arg("--binary")
            .arg(&selected);
        if check_only {
            command.arg("--check");
        }
        let output = command.output().expect("launch Rust evaluator");
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("render ")
                && !stderr.contains("missing real archived Outcome structure"),
            "invalid selected file must fail on actual process launch, not fall back: {stderr}"
        );
        assert_eq!(std::fs::read(&evidence_json).ok(), before_json);
        assert_eq!(std::fs::read(&evidence_markdown).ok(), before_markdown);
        assert!(!checkout.join("target").exists());
    }
}

#[cfg(unix)]
#[test]
fn cli_entrypoint_runs_with_windows_sized_main_stack() {
    let output = Command::new("sh")
        .args(["-c", "ulimit -s 1024; exec \"$1\" --version", "sh"])
        .arg(env!("CARGO_BIN_EXE_ai-cockpit"))
        .output()
        .expect("launch CLI with a Windows-sized main stack");

    assert!(
        output.status.success(),
        "CLI must start with a 1 MiB main stack: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(env!("CARGO_PKG_VERSION")),
        "CLI version output must remain available"
    );
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
    let python = python_oracle_command()
        .arg(checkout.join("tests/evaluation/WI-750-p1-cognitive-benefit-current-base.py"))
        .arg("--repo")
        .arg(&checkout)
        .args(["--binary", binary])
        .output()
        .expect("launch Python evaluator");
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
        .expect("launch Rust evaluator");
    assert!(
        rust.status.success(),
        "Rust evaluator failed: {}",
        String::from_utf8_lossy(&rust.stderr)
    );
    let rust_json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&json_path).expect("Rust JSON artifact"))
            .expect("Rust JSON report");
    assert_eq!(
        rust_json["runtimeBinary"], python_json["runtimeBinary"],
        "runtimeBinary must preserve the oracle's resolved-path representation"
    );
    let cases = rust_json["cases"].as_array().expect("report cases");
    let normal_completion = cases
        .iter()
        .find(|case| case["id"] == "normal-completion")
        .expect("normal-completion case");
    let expected_outcome = PathBuf::from(".ai")
        .join("work-items")
        .join("archive")
        .join("WI-663-wi659-outcome-trust-replacement.outcome.json");
    let expected_contract = PathBuf::from(".ai")
        .join("work-items")
        .join("archive")
        .join("WI-663-wi659-outcome-trust-replacement.contract.json");
    assert_eq!(
        normal_completion["sourceOutcome"].as_str(),
        expected_outcome.to_str(),
        "sourceOutcome must use native repository-relative path separators"
    );
    assert_eq!(
        normal_completion["sourceContract"].as_str(),
        expected_contract.to_str(),
        "sourceContract must use native repository-relative path separators"
    );
    let scope_exceeded = cases
        .iter()
        .find(|case| case["id"] == "scope-exceeded")
        .expect("scope-exceeded case");
    let expected_fixture = PathBuf::from("tests")
        .join("conformance")
        .join("fixtures")
        .join("scope-exceeded")
        .join("input.json");
    assert_eq!(
        scope_exceeded["fixture"].as_str(),
        expected_fixture.to_str(),
        "fixture must use native repository-relative path separators"
    );
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
    let python = python_oracle_command()
        .arg(checkout.join("tests/evaluation/WI-750-p1-cognitive-benefit-current-base.py"))
        .arg("--repo")
        .arg(&checkout)
        .args(["--binary", binary, "--check"])
        .output()
        .expect("launch Python oracle");
    assert!(!python.status.success());
    assert!(String::from_utf8_lossy(&python.stderr).contains("expected JSON object"));

    let rust = Command::new(binary)
        .args(["audit", "cognitive-benefit", "--repo"])
        .arg(&checkout)
        .args(["--binary", binary, "--check"])
        .output()
        .expect("launch Rust evaluator");
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
        .expect("launch repository gate");
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

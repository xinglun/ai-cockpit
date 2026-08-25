use std::fs;
use std::process::Command;

fn git(root: &std::path::Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .expect("git")
            .success()
    );
}

fn repository(name: &str) -> tempfile::TempDir {
    let root = tempfile::Builder::new()
        .prefix(name)
        .tempdir()
        .expect("tempdir");
    git(root.path(), &["init", "-q"]);
    git(root.path(), &["config", "user.name", "CLI gate test"]);
    git(
        root.path(),
        &["config", "user.email", "cli-gate@example.invalid"],
    );
    fs::write(root.path().join("README.md"), "fixture\n").expect("fixture");
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-qm", "base"]);
    let attach = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["attach", "--repo"])
        .arg(root.path())
        .output()
        .expect("attach");
    assert!(attach.status.success(), "{:?}", attach.stderr);
    root
}

fn start(root: &tempfile::TempDir, required_evidence: Option<&str>) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"));
    command.args(["start", "--repo"]).arg(root.path()).args([
        "--id",
        "WI-CLI-GATE",
        "--intent",
        "validate the CI Contract gate",
        "--goal",
        "keep CI governance read-only",
        "--scope",
        "crates/**",
        "--out-of-scope",
        "target/**",
        "--authority",
        "authorized",
        "--acceptance",
        "the gate is identity bound",
    ]);
    if let Some(required_evidence) = required_evidence {
        command.args(["--required-evidence", required_evidence]);
    }
    let output = command.output().expect("start");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn contract(root: &tempfile::TempDir) -> std::path::PathBuf {
    root.path()
        .join(".ai/work-items/active/WI-CLI-GATE.contract.json")
}

#[test]
fn gate_cli_emits_green_report_without_writing_ai() {
    let root = repository("cockpit-cli-gate-green-");
    start(&root, None);
    let before = fs::read_dir(root.path().join(".ai")).expect("ai").count();
    let output_path = root.path().join("target/ci-gate.json");
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["gate", "--repo"])
        .arg(root.path())
        .args(["--contract"])
        .arg(contract(&root))
        .args(["--stage", "pull_request", "--runner", "hosted", "--report"])
        .arg(&output_path)
        .output()
        .expect("gate");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&output_path).unwrap()).unwrap();
    assert_eq!(report["state"], "passed");
    assert_eq!(report["decisionState"], "green");
    assert_eq!(report["stage"], "pr");
    assert_eq!(report["runner"], "hosted");
    assert_eq!(
        fs::read_dir(root.path().join(".ai")).unwrap().count(),
        before,
        "gate must not add .ai entries"
    );
}

#[test]
fn gate_cli_stops_when_required_evidence_is_missing() {
    let root = repository("cockpit-cli-gate-yellow-");
    start(&root, Some("verification"));
    let output_path = root.path().join("target/ci-gate.json");
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["gate", "--repo"])
        .arg(root.path())
        .args(["--contract"])
        .arg(contract(&root))
        .args(["--stage", "pr", "--runner", "hosted", "--report"])
        .arg(&output_path)
        .output()
        .expect("gate");
    assert!(!output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&output_path).unwrap()).unwrap();
    assert_eq!(report["state"], "blocked");
    assert_eq!(report["decisionState"], "yellow");
    assert_eq!(report["unknowns"][0], "required_evidence_missing");
}

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
    fs::write(root.path().join(".git/info/exclude"), "").expect("empty repository excludes");
    let empty_global_excludes = root.path().join(".git/empty-global-excludes");
    fs::write(&empty_global_excludes, "").expect("empty global excludes");
    let configure_excludes = Command::new("git")
        .args(["config", "core.excludesFile"])
        .arg(&empty_global_excludes)
        .current_dir(root.path())
        .status()
        .expect("disable inherited global Git excludes");
    assert!(
        configure_excludes.success(),
        "configure fixture Git excludes"
    );
    fs::write(root.path().join(".gitignore"), "/target/\n").expect("Cargo target ignore rule");
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

fn route_receipt(root: &tempfile::TempDir, report: &serde_json::Value) -> std::path::PathBuf {
    let path = root.path().join("target/route-receipt.json");
    let route = serde_json::json!({
        "schemaVersion": 1,
        "kind": "repository_quality_route",
        "baseRevision": report["comparisonBaseRevision"],
        "stage": "pull_request",
        "contractPath": ".ai/work-items/active/WI-CLI-GATE.contract.json",
        "contractDigest": report["contractFileDigest"],
    });
    fs::write(
        &path,
        format!("{}\n", serde_json::to_string_pretty(&route).unwrap()),
    )
    .expect("route receipt");
    path
}

#[test]
fn gate_report_cli_validates_repository_bound_report() {
    let root = repository("cockpit-cli-gate-report-");
    start(&root, None);
    let target_is_ignored = Command::new("git")
        .args(["check-ignore", "--quiet", "target/ci-gate.json"])
        .current_dir(root.path())
        .status()
        .expect("check Cargo target output ignore rule");
    assert!(
        target_is_ignored.success(),
        "generated quality-gate reports must not become source changes in the isolated repository"
    );
    let report_path = root.path().join("target/ci-gate.json");
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["gate", "--repo"])
        .arg(root.path())
        .args(["--contract"])
        .arg(contract(&root))
        .args(["--stage", "pull_request", "--runner", "hosted", "--report"])
        .arg(&report_path)
        .output()
        .expect("gate");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&report_path).unwrap()).unwrap();
    let route_path = route_receipt(&root, &report);
    let validated = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["gate-report", "--repo"])
        .arg(root.path())
        .args(["--report"])
        .arg(&report_path)
        .args(["--route-receipt"])
        .arg(&route_path)
        .output()
        .expect("gate report");
    assert!(
        validated.status.success(),
        "{}",
        String::from_utf8_lossy(&validated.stderr)
    );
    let validated_report: serde_json::Value =
        serde_json::from_slice(&validated.stdout).expect("validated report");
    assert_eq!(validated_report["state"], "passed");
    assert_eq!(validated_report["decisionState"], "green");
}

#[test]
fn gate_report_cli_rejects_unignored_out_of_scope_source() {
    let root = repository("cockpit-cli-gate-report-unignored-");
    start(&root, None);
    let report_path = root.path().join("target/ci-gate.json");
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["gate", "--repo"])
        .arg(root.path())
        .args(["--contract"])
        .arg(contract(&root))
        .args(["--stage", "pull_request", "--runner", "hosted", "--report"])
        .arg(&report_path)
        .output()
        .expect("gate");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&report_path).unwrap()).unwrap();
    let route_path = route_receipt(&root, &report);
    fs::write(
        root.path().join("unignored-out-of-scope.txt"),
        "unexpected source\n",
    )
    .expect("unignored out-of-scope source");
    let ignored_probe = Command::new("git")
        .args(["check-ignore", "--quiet", "--no-index"])
        .arg(root.path().join("unignored-out-of-scope.txt"))
        .current_dir(root.path())
        .status()
        .expect("check unignored out-of-scope source");
    assert_eq!(
        ignored_probe.code(),
        Some(1),
        "negative fixture must remain visible to Git status"
    );

    let validated = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["gate-report", "--repo"])
        .arg(root.path())
        .args(["--report"])
        .arg(&report_path)
        .args(["--route-receipt"])
        .arg(&route_path)
        .output()
        .expect("gate report");
    assert!(!validated.status.success());
    assert!(
        String::from_utf8_lossy(&validated.stderr).contains(
            "Contract gate report does not match freshly recomputed canonical material projection"
        ),
        "{}",
        String::from_utf8_lossy(&validated.stderr)
    );
}

#[test]
fn gate_report_cli_rejects_tampered_repository_identity() {
    let root = repository("cockpit-cli-gate-report-tamper-");
    start(&root, None);
    let report_path = root.path().join("target/ci-gate.json");
    let output = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["gate", "--repo"])
        .arg(root.path())
        .args(["--contract"])
        .arg(contract(&root))
        .args(["--stage", "pull_request", "--runner", "hosted", "--report"])
        .arg(&report_path)
        .output()
        .expect("gate");
    assert!(output.status.success());
    let mut report: serde_json::Value =
        serde_json::from_slice(&fs::read(&report_path).unwrap()).unwrap();
    report["repositoryId"] = serde_json::Value::String(format!("sha256:{}", "0".repeat(64)));
    fs::write(
        &report_path,
        format!("{}\n", serde_json::to_string_pretty(&report).unwrap()),
    )
    .expect("tampered report");
    let route_path = route_receipt(&root, &report);
    let validated = Command::new(env!("CARGO_BIN_EXE_ai-cockpit"))
        .args(["gate-report", "--repo"])
        .arg(root.path())
        .args(["--report"])
        .arg(&report_path)
        .args(["--route-receipt"])
        .arg(&route_path)
        .output()
        .expect("gate report");
    assert!(!validated.status.success());
    assert!(
        String::from_utf8_lossy(&validated.stderr).contains("repositoryId"),
        "{}",
        String::from_utf8_lossy(&validated.stderr)
    );
}

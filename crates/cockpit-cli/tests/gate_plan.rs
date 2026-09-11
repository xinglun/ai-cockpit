use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn failure_json(output: &std::process::Output) -> Value {
    output
        .stderr
        .split(|byte| *byte == b'\n')
        .filter_map(|line| serde_json::from_slice(line).ok())
        .find(|value: &Value| value["state"] == "failed")
        .expect("structured gate-plan failure")
}

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("launch git");
    assert!(output.status.success(), "git failed: {:?}", output);
    String::from_utf8(output.stdout).expect("git output")
}

#[test]
fn gate_plan_cli_reads_git_facts_and_validates_the_same_receipt() {
    let fixture = tempdir().expect("fixture");
    let root = fixture.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.name", "Gate Plan Test"]);
    git(root, &["config", "user.email", "gate-plan@example.invalid"]);
    fs::write(root.join("README.md"), "base\n").expect("base");
    fs::write(root.join(".gitignore"), "target/\n").expect("gitignore");
    let manifest = root.join("manifest.json");
    fs::write(
        &manifest,
        r#"{
          "gates": [{"category":"ci","command":["true"],"id":"ci_light","minimumProfile":"light"}],
          "pathProfiles":{"light":["docs/**"],"standard":["src/**"],"strict":[".github/**"]},
          "profileOrder":["light","standard","strict"],
          "releaseOwnedPatterns":["release/**"],
          "schemaVersion":2,
          "stageFloors":{"task":"light","pre_ci":"light","pull_request":"light","merge":"strict","release":"strict"},
          "unknownProfile":"strict"
        }"#,
    )
    .expect("manifest");
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "base"]);
    let base = git(root, &["rev-parse", "HEAD"]).trim().to_owned();
    fs::create_dir(root.join("docs")).expect("docs");
    fs::write(root.join("docs/readme.md"), "changed\n").expect("changed");
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "docs"]);
    let head = git(root, &["rev-parse", "HEAD"]).trim().to_owned();

    let receipt = root.join("target/route.json");
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let planned = Command::new(binary)
        .args([
            "gate-plan",
            "--repo",
            root.to_str().unwrap(),
            "--manifest",
            manifest.to_str().unwrap(),
            "--base",
            &base,
            "--head",
            &head,
            "--stage",
            "pull_request",
            "--risk",
            "normal",
            "--receipt",
            receipt.to_str().unwrap(),
        ])
        .output()
        .expect("run gate-plan");
    assert!(planned.status.success(), "gate-plan failed: {:?}", planned);
    let route: Value =
        serde_json::from_slice(&fs::read(&receipt).expect("receipt")).expect("route JSON");
    assert_eq!(route["selectedProfile"], "light");
    assert_eq!(route["changedPaths"], serde_json::json!(["docs/readme.md"]));
    assert!(
        route["receiptDigest"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );

    let validated = Command::new(binary)
        .args([
            "gate-plan",
            "--repo",
            root.to_str().unwrap(),
            "--manifest",
            manifest.to_str().unwrap(),
            "--receipt",
            receipt.to_str().unwrap(),
            "--validate-receipt",
        ])
        .output()
        .expect("validate gate-plan");
    assert!(
        validated.status.success(),
        "validation failed: {:?}",
        validated
    );
}

#[test]
fn gate_plan_cli_rejects_empty_covers_with_compatibility_failure_fields() {
    let fixture = tempdir().expect("fixture");
    let root = fixture.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.name", "Gate Plan Test"]);
    git(root, &["config", "user.email", "gate-plan@example.invalid"]);
    fs::write(root.join("README.md"), "base\n").expect("base");
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "base"]);
    let manifest = root.join("manifest.json");
    fs::write(
        &manifest,
        r#"{
          "gates": [{"category":"ci","command":["true"],"covers":[],"id":"ci_light","minimumProfile":"light"}],
          "pathProfiles":{"light":["docs/**"],"standard":["src/**"],"strict":[".github/**"]},
          "profileOrder":["light","standard","strict"],
          "releaseOwnedPatterns":["release/**"],
          "schemaVersion":2,
          "stageFloors":{"task":"light","pre_ci":"light","pull_request":"light","merge":"strict","release":"strict"},
          "unknownProfile":"strict"
        }"#,
    )
    .expect("manifest");

    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let output = Command::new(binary)
        .args([
            "gate-plan",
            "--repo",
            root.to_str().unwrap(),
            "--manifest",
            manifest.to_str().unwrap(),
            "--base",
            "HEAD",
            "--stage",
            "pull_request",
            "--receipt",
            root.join("route.json").to_str().unwrap(),
        ])
        .output()
        .expect("run gate-plan");
    assert_eq!(output.status.code(), Some(2), "stderr: {:?}", output);
    let failure = failure_json(&output);
    assert_eq!(failure["state"], "failed");
    assert_eq!(failure["failureCode"], "quality_route_failed");
    assert!(
        failure["remediation"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
}

#[cfg(unix)]
#[test]
fn gate_plan_cli_rejects_a_summary_symlink_with_invalid_lifecycle_failure() {
    use std::os::unix::fs::symlink;

    let fixture = tempdir().expect("fixture");
    let root = fixture.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.name", "Gate Plan Test"]);
    git(root, &["config", "user.email", "gate-plan@example.invalid"]);
    fs::write(root.join("README.md"), "base\n").expect("base");
    fs::create_dir_all(root.join(".ai/work-items/active")).expect("active");
    let contract = root.join(".ai/work-items/active/WI-SYMLINK.contract.json");
    fs::write(&contract, r#"{"risk":"normal"}"#).expect("contract");
    fs::write(
        root.join("summary-target.json"),
        r#"{"state":"implementation_active"}"#,
    )
    .expect("summary target");
    symlink(
        root.join("summary-target.json"),
        root.join(".ai/work-items/active/WI-SYMLINK.summary.json"),
    )
    .expect("summary symlink");
    let manifest = root.join("manifest.json");
    fs::write(
        &manifest,
        r#"{
          "gates": [{"category":"ci","command":["true"],"id":"ci_light","minimumProfile":"light"}],
          "pathProfiles":{"light":["docs/**"],"standard":["src/**"],"strict":[".github/**"]},
          "profileOrder":["light","standard","strict"],
          "releaseOwnedPatterns":["release/**"],
          "schemaVersion":2,
          "stageFloors":{"task":"light","pre_ci":"light","pull_request":"light","merge":"strict","release":"strict"},
          "unknownProfile":"strict"
        }"#,
    )
    .expect("manifest");
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "base"]);
    let base = git(root, &["rev-parse", "HEAD"]).trim().to_owned();

    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let output = Command::new(binary)
        .args([
            "gate-plan",
            "--repo",
            root.to_str().unwrap(),
            "--manifest",
            manifest.to_str().unwrap(),
            "--base",
            &base,
            "--head",
            &base,
            "--stage",
            "pull_request",
            "--contract",
            contract.to_str().unwrap(),
            "--receipt",
            root.join("route.json").to_str().unwrap(),
        ])
        .output()
        .expect("run gate-plan");
    assert_eq!(output.status.code(), Some(2), "stderr: {:?}", output);
    let failure = failure_json(&output);
    assert_eq!(failure["failureCode"], "lifecycle_transition_invalid");
    assert_eq!(
        failure["remediation"],
        "restore the declared lifecycle order and checkpoint/preflight bindings before pushing"
    );
}

#[test]
fn gate_plan_cli_reports_base_mismatch_with_compatibility_failure_fields() {
    let fixture = tempdir().expect("fixture");
    let root = fixture.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.name", "Gate Plan Test"]);
    git(root, &["config", "user.email", "gate-plan@example.invalid"]);
    fs::write(root.join("README.md"), "base\n").expect("base");
    let manifest = root.join("manifest.json");
    fs::write(
        &manifest,
        r#"{
          "gates": [{"category":"ci","command":["true"],"id":"ci_light","minimumProfile":"light"}],
          "pathProfiles":{"light":["docs/**"],"standard":["src/**"],"strict":[".github/**"]},
          "profileOrder":["light","standard","strict"],
          "releaseOwnedPatterns":["release/**"],
          "schemaVersion":2,
          "stageFloors":{"task":"light","pre_ci":"light","pull_request":"light","merge":"strict","release":"strict"},
          "unknownProfile":"strict"
        }"#,
    )
    .expect("manifest");
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "base"]);
    let base = git(root, &["rev-parse", "HEAD"]).trim().to_owned();
    fs::write(root.join("README.md"), "head\n").expect("head");
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "head"]);
    let head = git(root, &["rev-parse", "HEAD"]).trim().to_owned();
    let receipt = root.join("route.json");
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let planned = Command::new(binary)
        .args([
            "gate-plan",
            "--repo",
            root.to_str().unwrap(),
            "--manifest",
            manifest.to_str().unwrap(),
            "--base",
            &base,
            "--head",
            &head,
            "--stage",
            "pull_request",
            "--receipt",
            receipt.to_str().unwrap(),
        ])
        .output()
        .expect("plan route");
    assert!(planned.status.success(), "plan failed: {:?}", planned);

    let output = Command::new(binary)
        .args([
            "gate-plan",
            "--repo",
            root.to_str().unwrap(),
            "--manifest",
            manifest.to_str().unwrap(),
            "--base",
            &head,
            "--head",
            &head,
            "--stage",
            "pull_request",
            "--receipt",
            receipt.to_str().unwrap(),
            "--validate-receipt",
        ])
        .output()
        .expect("validate route");
    assert_eq!(output.status.code(), Some(2), "stderr: {:?}", output);
    let failure = failure_json(&output);
    assert_eq!(failure["failureCode"], "quality_route_failed");
    assert!(
        failure["remediation"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
}

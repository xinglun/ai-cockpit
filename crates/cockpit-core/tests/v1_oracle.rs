use serde::Deserialize;
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct SemanticResult {
    #[serde(default)]
    case: String,
    decision_state: String,
    blockers: Vec<String>,
    unknowns: Vec<String>,
    safe_actions: Vec<String>,
    required_checks: Vec<String>,
    authority: String,
    outcome_state: String,
}

impl SemanticResult {
    fn normalize(&mut self) {
        self.blockers.sort();
        self.unknowns.sort();
        self.safe_actions.sort();
        self.required_checks.sort();
    }
}

fn conformance_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/conformance")
}

fn locked_reference_commit() -> String {
    let lock = fs::read_to_string(conformance_root().join("v1-reference.lock"))
        .expect("read V1 reference lock");
    lock.lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("commit = \"")
                .and_then(|value| value.strip_suffix('"'))
        })
        .filter(|commit| commit.len() == 40 && commit.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .expect("v1-reference.lock must contain a 40-character commit")
        .to_owned()
}

fn reference_head(reference_root: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .args(["-C"])
        .arg(reference_root)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|error| format!("run git for V1 reference identity: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "read V1 reference identity: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn require_locked_reference(reference_root: &Path) -> Result<(), String> {
    let locked = locked_reference_commit();
    let actual = reference_head(reference_root)?;
    if actual != locked {
        return Err(format!(
            "V1 reference identity mismatch: expected {locked}, got {actual}"
        ));
    }
    Ok(())
}

fn expected_results() -> BTreeMap<String, SemanticResult> {
    let root = conformance_root();
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join("manifest.json")).expect("read conformance manifest"),
    )
    .expect("parse conformance manifest");
    manifest["cases"]
        .as_array()
        .expect("manifest cases")
        .iter()
        .map(|case| {
            let name = case.as_str().expect("case name");
            let bytes = fs::read(root.join("fixtures").join(name).join("expected.json"))
                .expect("read expected semantics");
            let mut expected: SemanticResult =
                serde_json::from_slice(&bytes).expect("parse expected semantics");
            expected.case = name.to_owned();
            expected.normalize();
            (name.to_owned(), expected)
        })
        .collect()
}

fn run_oracle(reference_root: &Path) -> Result<BTreeMap<String, SemanticResult>, String> {
    require_locked_reference(reference_root)?;
    let script = conformance_root().join("v1_oracle.py");
    let output = Command::new("python3")
        .arg(&script)
        .arg("--reference-root")
        .arg(reference_root)
        .arg("--fixtures")
        .arg(conformance_root().join("fixtures"))
        .output()
        .map_err(|error| format!("execute V1 Oracle adapter: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "V1 Oracle adapter failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let results: Vec<SemanticResult> = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("decode V1 Oracle output: {error}"))?;
    let mut indexed = BTreeMap::new();
    for mut result in results {
        result.normalize();
        if indexed.insert(result.case.clone(), result).is_some() {
            return Err("V1 Oracle returned a duplicate case".into());
        }
    }
    Ok(indexed)
}

#[test]
fn wrong_reference_commit_is_rejected_before_oracle_execution() {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let error = require_locked_reference(&project_root).expect_err("wrong repository must fail");
    assert!(error.contains("V1 reference identity mismatch"), "{error}");
}

#[test]
#[ignore = "requires AI_COCKPIT_V1_ROOT pointing to the locked external V1 checkout"]
fn executable_v1_oracle_matches_all_canonical_semantics() {
    let reference_root = env::var_os("AI_COCKPIT_V1_ROOT")
        .map(PathBuf::from)
        .expect("AI_COCKPIT_V1_ROOT is required");
    let expected = expected_results();
    let actual = run_oracle(&reference_root).expect("execute locked V1 Oracle");
    assert_eq!(
        actual.keys().collect::<Vec<_>>(),
        expected.keys().collect::<Vec<_>>()
    );
    for (case, expected) in expected {
        assert_eq!(
            actual.get(&case),
            Some(&expected),
            "V1 semantic case {case}"
        );
    }
}
